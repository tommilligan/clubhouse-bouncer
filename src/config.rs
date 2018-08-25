extern crate url;

use url::Url;

#[derive(Debug, Clone)]
pub struct BouncerConfig {
    pub bouncer_credentials: String,
    pub clubhouse_api_token: String,
}

impl BouncerConfig {
    pub fn authorize_clubhouse_url(&self, u: &mut Url) -> () {
        u.query_pairs_mut()
            .append_pair("token", &self.clubhouse_api_token);
    }
}
