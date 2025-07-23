#![deny(clippy::all)]

mod event;
mod message;
mod utils;

use serenity::all::GatewayIntents;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utils::config::PreviewConfig;

#[derive(serde::Deserialize, Debug)]
pub struct EnvConfig {
    pub discord_api_token: String,
    #[serde(default)]
    #[serde(deserialize_with = "crate::utils::config::empty_string_as_none")]
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
    tracing::debug!("Config: {:?}", PreviewConfig::get_config());

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
        GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::GUILD_MESSAGE_REACTIONS,
    )
    .event_handler(event::preview::PreviewHandler)
    .event_handler(event::reaction::ReactionHandler)
    .event_handler(event::ready::ReadyHandler)
    .await
    .expect("Failed to initialize client.");

    if let Err(why) = client.start().await {
        return Err(anyhow::anyhow!(why));
    }

    Ok(())
}
