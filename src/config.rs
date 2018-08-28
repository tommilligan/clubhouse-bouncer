extern crate hyper;
extern crate url;

use url::Url;

#[derive(Debug, Clone)]
/// Central app configuration and helper methods are stored here
pub struct BouncerConfig {
    /// Incoming authorization for clubhouse-bouncer requests
    pub bouncer_credentials: Vec<String>,
    /// Outgoing authorization for Clubhouse API requests
    pub clubhouse_api_token: String,
}

impl BouncerConfig {
    /// Given a `url::Url`, authorizes it using Clubhouse credentials
    ///
    /// # Arguments
    ///
    /// * `url` - A clubhouse url to be authorized
    pub fn authorize_clubhouse_url(&self, url: &mut Url) -> () {
        url.query_pairs_mut()
            .append_pair("token", &self.clubhouse_api_token);
    }

    /// Given a `hyper::Request`, check it is authorized
    ///
    /// # Arguments
    ///
    /// * `req` - A hyper request
    pub fn validate_bouncer_authorization(&self, req: &hyper::Request<hyper::Body>) -> bool {
        let auth = req.headers().get(hyper::header::AUTHORIZATION);
        match auth {
            Some(a) => self.bouncer_credentials.iter().any(|cred| cred == a),
            None => false,
        }
    }
}
