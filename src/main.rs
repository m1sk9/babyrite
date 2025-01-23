#![deny(clippy::all)]

mod cache;
mod config;
mod event;
mod message;

use crate::config::PreviewConfig;
use crate::event::preview::PreviewHandler;
use crate::event::ready::ReadyHandler;
use serenity::all::GatewayIntents;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(serde::Deserialize)]
pub struct EnvConfig {
    pub discord_api_token: String,
    pub config_file_path: Option<String>,
}

pub fn get_env_config() -> &'static EnvConfig {
    static ENV_CONFIG: std::sync::OnceLock<EnvConfig> = std::sync::OnceLock::new();
    ENV_CONFIG.get_or_init(|| envy::from_env().expect("Failed to load environment configuration."))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    PreviewConfig::init()?;
    let envs = get_env_config();
    match PreviewConfig::get_feature_flag("json_logging") {
        true => {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "babyrite=debug,serenity=debug".into()),
                )
                .with(tracing_subscriber::fmt::layer().json())
                .init();
            tracing::info!("Feature Flag : Log output in JSON format is now enabled.");
        }
        false => {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "babyrite=debug,serenity=debug".into()),
                )
                .with(tracing_subscriber::fmt::layer().compact())
                .init();
        }
    }

    let mut client = serenity::Client::builder(
        &envs.discord_api_token,
        GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGES,
    )
    .event_handler(ReadyHandler)
    .event_handler(PreviewHandler)
    .await
    .expect("Failed to initialize client.");

    if let Err(why) = client.start().await {
        return Err(anyhow::anyhow!(why));
    }

    Ok(())
}
