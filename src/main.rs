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

use futures::{future, Future, Stream};

use hyper::client::HttpConnector;
use hyper::service::service_fn;
use hyper::{header, Body, Client, Method, Request, Response, Server, StatusCode};
use hyper_tls::HttpsConnector;

static NOTFOUND: &[u8] = b"Not Found";
static CLUBHOUSE_URL_WORKFLOWS: &str = "https://api.clubhouse.io/api/v2/workflows?token={token}";
static CLUBHOUSE_URL_STORY: &str = "http://127.0.0.1:1337/web_api";

#[derive(Serialize, Deserialize, Debug)]
struct QueryDeployable {
    story_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct WorkflowState {
    description: String,
    id: u64,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Workflow {
    id: u64,
    name: String,
    states: Vec<WorkflowState>,
}

fn env_var_required(key: &str) -> String {
    trace!("Loading env var {}", key);
    env::var(key).expect(&format!("Missing env var {}", key))
}

fn stream_as_string(stream: hyper::Chunk) -> String {
    String::from_utf8(stream.to_vec()).expect("Invalid body string")
}

fn response_examples(
    req: Request<Body>,
    client: &Client<HttpsConnector<HttpConnector>>,
    config: &BouncerConfig,
) -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/deployable") => {
            let q = req.into_body().concat2().and_then(|stream| {
                let body = stream_as_string(stream);
                let q: QueryDeployable = serde_json::from_str(&body).expect("Invalid tickets JSON");
                warn!("{:?}", q);
                future::ok(q)
            });

            let get_workflows = client
                .get(
                    format!(
                        "https://api.clubhouse.io/api/v2/workflows?token={token}",
                        token = config.clubhouse_api_token
                    ).parse()
                        .unwrap(),
                )
                .and_then(|res| {
                    trace!("clubhouse esponse: {}", res.status());
                    res.into_body().concat2().and_then(|stream| {
                        let body = stream_as_string(stream);
                        future::ok(serde_json::from_str::<Vec<Workflow>>(&body).unwrap())
                    })
                });

            // let a: String = serde_json::to_string(&q).expect("Error serializing to JSON");
            // let a = future::ok(String::from("spam"));
            let res = get_workflows.and_then(|s| {
                let b = serde_json::to_string(&s).unwrap();
                future::ok(
                    Response::builder()
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(b))
                        .unwrap(),
                )
            });

            Box::new(res)
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

#[derive(Debug, Clone)]
struct BouncerConfig {
    bouncer_credentials: String,
    clubhouse_api_token: String,
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

        info!("clubhouse-bouncer ready on http://{}", addr);

        server
    }));
}
