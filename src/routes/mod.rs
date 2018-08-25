use std::collections::HashMap;

extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate pretty_env_logger;
extern crate serde_json;
extern crate url;

use futures::{future, stream, Future, Stream};

use hyper::client::HttpConnector;
use hyper::{header, Body, Client, Request, Response, StatusCode};
use hyper_tls::HttpsConnector;

use url::Url;

use bouncer::{Deployable, QueryDeployable, StoryState};
use clubhouse;
use config::BouncerConfig;

static NOTFOUND: &[u8] = b"Not Found";
static UNAUTHORIZED: &[u8] = b"Unauthorized";

pub fn not_found() -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
    warn!("404 Not Found");
    // Return 404 not found response.
    let body = Body::from(NOTFOUND);
    Box::new(future::ok(
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body)
            .unwrap(),
    ))
}

pub fn unauthorized() -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
    warn!("401 Unauthorized");
    Box::new(future::ok(
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from(UNAUTHORIZED))
            .unwrap(),
    ))
}

pub fn deployable(
    req: Request<Body>,
    client: &Client<HttpsConnector<HttpConnector>>,
    config: &BouncerConfig,
) -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
    trace!("Started deployable");
    let get_story_ids = req.into_body().concat2().and_then(|stream| {
        let q: QueryDeployable = serde_json::from_slice(&stream).expect("Invalid tickets JSON");
        warn!("{:?}", q);
        future::ok(q.story_ids)
    });

    let mut url_workflows = Url::parse("https://api.clubhouse.io/api/v2/workflows").unwrap();
    config.authorize_clubhouse_url(&mut url_workflows);

    let get_workflows = client
        .get(url_workflows.as_str().parse().unwrap())
        .and_then(|res| {
            trace!("clubhouse response: {}", res.status());
            res.into_body().concat2().and_then(|stream| {
                future::ok(serde_json::from_slice::<Vec<clubhouse::Workflow>>(&stream).unwrap())
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
                &futures_config.authorize_clubhouse_url(&mut url_story);
                futures_client
                    .get(url_story.as_str().parse().unwrap())
                    .and_then(|res| {
                        trace!("clubhouse response: {}", res.status());
                        res.into_body().concat2().and_then(|stream| {
                            let story =
                                serde_json::from_slice::<clubhouse::Story>(&stream).unwrap();
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
                        clubhouse::WorkflowState,
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
                                deployable: state.is_deployable(),
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
