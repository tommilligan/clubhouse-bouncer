use std::collections::HashMap;
use std::env;

extern crate futures;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;

use futures::{future, stream, Future, Stream};

use hyper::client::HttpConnector;
use hyper::service::service_fn;
use hyper::{header, Body, Client, Method, Request, Response, Server, StatusCode};
use hyper_tls::HttpsConnector;

use url::Url;

mod config;

use config::BouncerConfig;

static NOTFOUND: &[u8] = b"Not Found";
static DEPLOYABLE: [&str; 2] = ["Ready for Deploy", "Completed"];

fn is_deployable(description: &str) -> bool {
    DEPLOYABLE.iter().any(|s| s == &description)
}

#[derive(Serialize, Deserialize, Debug)]
struct QueryDeployable {
    story_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WorkflowState {
    id: u64,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StoryState {
    deployable: bool,
    story: Story,
    state: WorkflowState,
}

#[derive(Serialize, Deserialize, Debug)]
struct Workflow {
    id: u64,
    name: String,
    states: Vec<WorkflowState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Story {
    id: u64,
    name: String,
    workflow_state_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Deployable {
    deployable: bool,
    story_states: Vec<StoryState>,
}

fn env_var_required(key: &str) -> String {
    trace!("Loading env var {}", key);
    env::var(key).expect(&format!("Missing env var {}", key))
}

fn response_examples(
    req: Request<Body>,
    client: &Client<HttpsConnector<HttpConnector>>,
    config: &BouncerConfig,
) -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/deployable") => {
            let get_story_ids = req.into_body().concat2().and_then(|stream| {
                let q: QueryDeployable =
                    serde_json::from_slice(&stream).expect("Invalid tickets JSON");
                warn!("{:?}", q);
                future::ok(q.story_ids)
            });

            let mut url_workflows =
                Url::parse("https://api.clubhouse.io/api/v2/workflows").unwrap();
            url_workflows
                .query_pairs_mut()
                .append_pair("token", &config.clubhouse_api_token);

            let get_workflows = client
                .get(url_workflows.as_str().parse().unwrap())
                .and_then(|res| {
                    trace!("clubhouse response: {}", res.status());
                    res.into_body().concat2().and_then(|stream| {
                        future::ok(serde_json::from_slice::<Vec<Workflow>>(&stream).unwrap())
                    })
                });

            let url_stories = Url::parse("https://api.clubhouse.io/api/v2/stories/").unwrap();

            // TODO do we need to clone here?
            let futures_client = client.clone();
            let futures_config = config.clone();
            let deployable = get_story_ids.and_then(move |story_ids| {
                let stream_stories = stream::iter_ok(story_ids)
                    .map(move |story_id: String| {
                        let mut url_story = url_stories.clone().join(&story_id).unwrap();
                        url_story
                            .query_pairs_mut()
                            .append_pair("token", &futures_config.clubhouse_api_token);
                        futures_client
                            .get(url_story.as_str().parse().unwrap())
                            .and_then(|res| {
                                trace!("clubhouse response: {}", res.status());
                                res.into_body().concat2().and_then(|stream| {
                                    let story = serde_json::from_slice::<Story>(&stream).unwrap();
                                    Ok(story)
                                })
                            })
                    })
                    .collect();
                stream_stories.and_then(|get_stories_data| {
                    future::join_all(get_stories_data)
                        .join(get_workflows)
                        .and_then(|(stories_data, workflows)| {
                            let mut workflow_lookup: HashMap<
                                u64,
                                WorkflowState,
                            > = HashMap::new();
                            for workflow in workflows.into_iter() {
                                for state in workflow.states.into_iter() {
                                    workflow_lookup.insert(state.id, state);
                                }
                            }

                            let story_states: Vec<StoryState> = stories_data
                                .into_iter()
                                .map(|story| {
                                    let state = workflow_lookup
                                        .get(&story.workflow_state_id)
                                        .unwrap()
                                        .to_owned();
                                    StoryState {
                                        deployable: is_deployable(&state.name),
                                        state: state,
                                        story: story,
                                    }
                                })
                                .collect();

                            let deployable = Deployable {
                                deployable: story_states
                                    .iter()
                                    .all(|story_state| story_state.deployable),
                                story_states: story_states,
                            };

                            future::ok(
                                Response::builder()
                                    .header(header::CONTENT_TYPE, "application/json")
                                    .body(Body::from(serde_json::to_string(&deployable).unwrap()))
                                    .unwrap(),
                            )
                        })
                })
            });
            Box::new(deployable)
        }
        _ => {
            // Return 404 not found response.
            let body = Body::from(NOTFOUND);
            Box::new(future::ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(body)
                    .unwrap(),
            ))
        }
    }
}

fn main() {
    pretty_env_logger::init();

    let port: String = env::var("PORT").unwrap_or(String::from("2686"));
    let addr = format!("127.0.0.1:{}", &port).parse().unwrap();

    // Calculate static config to pass down to workers
    let bouncer_config = BouncerConfig {
        bouncer_credentials: env_var_required("BOUNCER_CREDENTIALS"),
        clubhouse_api_token: env_var_required("CLUBHOUSE_API_TOKEN"),
    };

    hyper::rt::run(future::lazy(move || {
        // 4 is number of blocking threads for DNS
        let https = HttpsConnector::new(4).unwrap();
        // Share a `Client` with all `Service`s
        let client = Client::builder().build::<_, hyper::Body>(https);

        let new_service = move || {
            // Move a clone of `client` into the `service_fn`.
            let client = client.clone();
            let bouncer_config = bouncer_config.clone();
            service_fn(move |req| response_examples(req, &client, &bouncer_config))
        };

        let server = Server::bind(&addr)
            .serve(new_service)
            .map_err(|e| error!("server error: {}", e));

        warn!("clubhouse-bouncer ready on http://{}", addr);

        server
    }));
}
