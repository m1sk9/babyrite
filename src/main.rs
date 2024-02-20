mod event;
mod model;

use serenity::model::gateway::GatewayIntents;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BabyriteEnv {
    pub discord_api_token: String,
    pub sentry_dsn: Option<String>,
}

pub fn envs() -> &'static BabyriteEnv {
    static CACHE: std::sync::OnceLock<BabyriteEnv> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| envy::from_env().expect("Failed to load environment variables."))
}

#[tokio::main]
async fn main() {
    if let Err(cause) = dotenvy::dotenv() {
        tracing::warn!(%cause, "Failed to load environment variables from .env file.");
    }

    let envs = envs();

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("babyrite=debug"));
    let subscriber = FmtSubscriber::builder().with_env_filter(filter).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber.");

    let guild = envs.sentry_dsn.as_ref().map(|dsn| {
        sentry::init((
            dsn.as_str(),
            sentry::ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            }
        ))
    });

    if guild.is_some() {
        tracing::info!("Sentry is initialized.");
    } else {
        tracing::warn!("Sentry is not initialized.");
    }

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let result = serenity::Client::builder(&envs.discord_api_token, intents)
        .event_handler(event::EvHander)
        .await;

    let mut client = match result {
        Ok(ret) => ret,
        Err(cause) => {
            return tracing::error!(%cause, "Failed to create discord client.");
        }
    };

    if let Err(cause) = client.start().await {
        tracing::error!(%cause, "Failed to start discord client.");
    }
}
