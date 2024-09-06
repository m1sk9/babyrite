use crate::model::config::BabyriteConfig;
use model::config::LoggerFormat;
#[deny(unused_imports)]
use serenity::prelude::GatewayIntents;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod env;
mod handler;
mod model;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::info!("Starting babyrite at v{}", env!("CARGO_PKG_VERSION"));

    dotenvy::dotenv().ok();

    BabyriteConfig::init();
    let config = BabyriteConfig::get();
    let envs = env::babyrite_envs();

    match config.logger_format {
        LoggerFormat::Compact => {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "babyrite=debug,serenity=debug".into()),
                )
                .with(tracing_subscriber::fmt::layer().compact())
                .init();
        }
        LoggerFormat::Json => {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "babyrite=debug,serenity=debug".into()),
                )
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
    }

    tracing::info!("Configuration: {:?}", config);
    if config.bypass_guilds {
        tracing::warn!(
            "The guild bypass setting is enabled. Quote messages between different guilds. "
        )
    }

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
