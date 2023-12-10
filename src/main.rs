use std::time::Duration;

use anyhow::Ok;
use dotenvy::dotenv;
use env::{BabyriteEnv, BABYRITE_ENV};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::client::discord_client;

mod adapters;
mod client;
mod env;
mod event;
mod macros;
mod model;

#[allow(dead_code)]
pub static DEFAULT_TIMEOUT_DURATION: Duration = Duration::from_secs(10);
pub static VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("babyrite=debug"))?,
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default tracing-subscriber.");

    // 環境変数の読み込み. 読み込み後は [&BABYRITE_ENV.get().unwrap().<key>] でアクセスできる.
    BABYRITE_ENV
        .set(envy::from_env::<BabyriteEnv>().expect("Failed to load environment variables."))
        .unwrap();

    discord_client(&BABYRITE_ENV.get().unwrap().discord_api_token).await?;

    Ok(())
}
