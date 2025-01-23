#![deny(clippy::all)]

mod event;

use serenity::all::GatewayIntents;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct Handler;

#[derive(serde::Deserialize)]
pub struct EnvConfig {
    pub feature_flag: Option<String>,
    pub discord_api_token: String,
}

pub fn get_env_config() -> &'static EnvConfig {
    static ENV_CONFIG: std::sync::OnceLock<EnvConfig> = std::sync::OnceLock::new();
    ENV_CONFIG.get_or_init(|| envy::from_env().expect("Failed to load environment configuration."))
}

impl EnvConfig {
    pub fn get_feature(&self, feature: &str) -> bool {
        self.feature_flag
            .as_ref()
            .is_some_and(|f| f.contains(feature))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let envs = get_env_config();
    // FIXME: I want to refactor this to be more idiomatic.
    match envs.get_feature("json_logging") {
        true => {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "babyrite=debug,serenity=debug".into()),
                )
                .with(tracing_subscriber::fmt::layer().json())
                .init();
            tracing::info!("JSON logging enabled.");
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

    // TODO: Add configuration logic here.
    // ...

    let mut client = serenity::Client::builder(
        &envs.discord_api_token,
        GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGES,
    )
    .event_handler(Handler)
    .await
    .expect("Failed to initialize client.");

    if let Err(why) = client.start().await {
        return Err(anyhow::anyhow!(why));
    }

    Ok(())
}
