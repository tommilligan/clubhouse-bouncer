use std::env;

extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use futures::{future, Future, Stream};

use hyper::{Body, Client, Method, Request, Response, Server, StatusCode, header};
use hyper::client::HttpConnector;
use hyper::service::service_fn;

static NOTFOUND: &[u8] = b"Not Found";

#[derive(Serialize, Deserialize, Debug)]
struct QueryDeployable {
    story_ids: Vec<String>
}

fn env_var_required(key: &str) -> String {
    trace!("Loading env var {}", key);
    env::var(key).expect(&format!("Missing env var {}", key))
}

fn response_examples(req: Request<Body>, client: &Client<HttpConnector>)
    -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>
{
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/deployable") => {

            req.into_body().concat2().and_then(|stream| {
                let body = String::from_utf8(stream.to_vec()).expect("Invalid string body");
                warn!("Got body; {}", &body);
                let q: QueryDeployable = serde_json::from_str(&body).expect("Invalid tickets JSON");
                let a: String = serde_json::to_string(&q).expect("Error serializing to JSON");
                future::ok(Response::builder()
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(a))
                    .unwrap()
                )
            }).boxed()
        }
        _ => {
            // Return 404 not found response.
            let body = Body::from(NOTFOUND);
            Box::new(future::ok(Response::builder()
                                         .status(StatusCode::NOT_FOUND)
                                         .body(body)
                                         .unwrap()))
        }
    }
}

fn main() {
    pretty_env_logger::init();

    let cluhouse_api_token: String = env_var_required("CLUBHOUSE_API_TOKEN");
    let bouncer_credentials: String = env_var_required("BOUNCER_CREDENTIALS");
    let port: String = env::var("PORT").unwrap_or(String::from("2686"));

    let addr = format!("127.0.0.1:{}", &port).parse().unwrap();

    hyper::rt::run(future::lazy(move || {
        // Share a `Client` with all `Service`s
        let client = Client::new();

        let new_service = move || {
            // Move a clone of `client` into the `service_fn`.
            let client = client.clone();
            service_fn(move |req| {
                response_examples(req, &client)
            })
        };

        let server = Server::bind(&addr)
            .serve(new_service)
            .map_err(|e| error!("server error: {}", e));

        info!("clubhouse-bouncer ready on http://{}", addr);

        server
    }));
}
