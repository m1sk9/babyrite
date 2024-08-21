use crate::model::config::BabyriteConfig;
#[deny(unused_imports)]
use serenity::prelude::GatewayIntents;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod env;
mod handler;
mod model;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::info!("Starting babyrite at v{}", env!("CARGO_PKG_VERSION"));

    if let Err(cause) = dotenvy::dotenv() {
        tracing::warn!(%cause, "Failed to load environment variables from .env file.");
    }

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("babyrite=debug"));
    let subscriber = FmtSubscriber::builder().with_env_filter(filter).finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing_subscriber as global default.");

    BabyriteConfig::init();
    let config = BabyriteConfig::get();
    tracing::info!("Configuration: {:?}", config);

    if config.bypass_guilds {
        tracing::warn!(
            "The guild bypass setting is enabled. Quote messages between different guilds. "
        )
    }

    let envs = env::babyrite_envs();
    // "メッセージの取得", "ギルド内メッセージへのアクセス" に該当する
    let intents = GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGES;
    let mut client = serenity::Client::builder(&envs.discord_api_token, intents)
        .event_handler(handler::BabyriteHandler)
        .await
        .expect("Failed to create a new client");

    if let Err(why) = client.start().await {
        Err(anyhow::anyhow!(
            "An error occurred while running the client: {:?}",
            why
        ))?;
    }
    Ok(())
}
