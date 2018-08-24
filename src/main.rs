use std::env;

fn env_var_required(key: &str) -> String {
    env::var(key).expect(&format!("Missing env var {}", key))
}

fn main() {
    let cluhouse_api_token: String = env_var_required("CLUBHOUSE_API_TOKEN");
    let bouncer_credentials: String = env_var_required("BOUNCER_CREDENTIALS");

    println!("{}", &bouncer_credentials);
    println!("Hello, world!");
}
