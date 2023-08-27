use dotenvy::dotenv;
use std::env;

pub fn load_dotenv() {
    dotenv().ok();
}

pub fn env_var(key: &str) -> String {
    match env::var(key) {
        Ok(var) => var,
        Err(why) => panic!("{}: {}", why, key),
    }
}
