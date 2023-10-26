use std::time::Duration;

use anyhow::Ok;
use dotenvy::dotenv;
use env::{BabyriteEnv, BABYRITE_ENV};

use crate::client::discord_client;

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
    dotenv().ok();
    tracing_subscriber::fmt().compact().init();

    // 環境変数の読み込み. 読み込み後は [&BABYRITE_ENV.get().unwrap().<key>] でアクセスできる.
    BABYRITE_ENV
        .set(envy::from_env::<BabyriteEnv>().expect("環境変数の読み込みに失敗しました."))
        .unwrap();

    discord_client(&BABYRITE_ENV.get().unwrap().discord_api_token).await?;

    Ok(())
}
