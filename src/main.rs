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

use futures::{future, Future};

use hyper::client::HttpConnector;
use hyper::service::service_fn;
use hyper::{Body, Client, Method, Request, Response, Server};
use hyper_tls::HttpsConnector;

mod bouncer;
mod clubhouse;
mod config;
use config::BouncerConfig;
mod routes;

fn env_var_required(key: &str) -> String {
    trace!("Loading env var {}", key);
    env::var(key).expect(&format!("Missing env var {}", key))
}

fn worker(
    req: Request<Body>,
    client: &Client<HttpsConnector<HttpConnector>>,
    config: &BouncerConfig,
) -> impl Future<Item = Response<Body>, Error = hyper::Error> {
    let res = if config.validate_bouncer_authorization(&req) {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/deployable") => routes::deployable(req, client, config),
            _ => routes::not_found(),
        }
    } else {
        routes::unauthorized()
    };
    res.or_else(|err| {
        error!("{:?}", err);
        future::ok(routes::internal_server_error())
    })
}

fn main() {
    pretty_env_logger::init();

    let port: String = env::var("PORT").unwrap_or(String::from("2686"));
    let address: String = env::var("ADDRESS").unwrap_or(String::from("0.0.0.0"));
    let addr = format!("{}:{}", &address, &port)
        .parse()
        .expect("Invalid server binding address");

    // Calculate static config to pass down to workers
    let bouncer_config = BouncerConfig {
        bouncer_credentials: env_var_required("BOUNCER_CREDENTIALS")
            .split(",")
            .map(|s| s.to_owned())
            .collect(),
        clubhouse_api_token: env_var_required("CLUBHOUSE_API_TOKEN"),
    };

    hyper::rt::run(future::lazy(move || {
        // 4 is number of blocking threads for DNS
        let https = HttpsConnector::new(4).expect("Failed to generate https connector");
        // Share a `Client` with all `Service`s
        let client = Client::builder().build::<_, hyper::Body>(https);

        let new_service = move || {
            // Move a clone of `client` into the `service_fn`.
            let client = client.clone();
            let bouncer_config = bouncer_config.clone();
            service_fn(move |req| worker(req, &client, &bouncer_config))
        };

        let server = Server::bind(&addr)
            .serve(new_service)
            .map_err(|e| error!("server error: {}", e));

        warn!("clubhouse-bouncer ready on http://{}", addr);
        server
    }));
}
