use std::time::Duration;

use anyhow::Ok;

use crate::{
    client::discord_client,
    env::{env_var, load_dotenv},
};

mod adapters;
mod client;
mod env;
mod event;
mod model;

#[allow(dead_code)]
pub static DEFAULT_TIMEOUT_DURATION: Duration = Duration::from_secs(10);
pub static VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_dotenv();
    tracing_subscriber::fmt().compact().init();

    let token = env_var("DISCORD_API_TOKEN");

    discord_client(token).await?;

    Ok(())
}
