use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BabyriteEnv {
    pub discord_api_token: String,
}

pub static BABYRITE_ENV: OnceCell<BabyriteEnv> = OnceCell::new();
