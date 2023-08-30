use std::time::Duration;

use anyhow::Ok;
use tracing::{info, warn};

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

    let _guard = if cfg!(feature = "enable_sentry") {
        info!("Feature flag: [enable_sentry] is enabled.");

        let _guard = sentry::init((
            "https://d6c0b0cc260b77c0e0d4c2e7c77d54c3@o4505761355988992.ingest.sentry.io/4505775155970048", sentry::ClientOptions {
                release: sentry::release_name!(),
                enable_profiling: true,
                ..Default::default()
            }
        ));

        info!("Sentry is initialized.");
        Some(_guard)
    } else {
        info!("Feature flag: [enable_sentry] is disabled.");
        warn!("Sentry's error monitoring feature has been disabled! Errors are not reported to Sentry.");
        None
    };

    let token = env_var("DISCORD_API_TOKEN");

    discord_client(token).await?;

    Ok(())
}
