#![deny(unused_imports)]

use config::BabyriteConfig;
use config::LoggerFormat;
use serenity::prelude::GatewayIntents;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod config;
pub mod handler;
pub mod preview;

#[derive(serde::Deserialize)]
pub struct BabyriteEnv {
    pub config_file_path: Option<String>,
    pub discord_api_token: String,
}

pub fn babyrite_envs() -> &'static BabyriteEnv {
    static BABYRITE_ENV: std::sync::OnceLock<BabyriteEnv> = std::sync::OnceLock::new();
    BABYRITE_ENV.get_or_init(|| {
        envy::from_env()
            .expect("Failed to read environment variables. Do you set the environment variables?")
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    BabyriteConfig::init();
    let config = BabyriteConfig::get();
    let envs = babyrite_envs();

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
