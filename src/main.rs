use serenity::prelude::GatewayIntents;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod env;
mod handler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::info!("Starting babyrite at v{}", env!("CARGO_PKG_VERSION"));

    dotenvy::dotenv().expect("Failed to load .env file. Do you placed the .env in the executable directory?");

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("babyrite=debug"));
    let subscriber = FmtSubscriber::builder().with_env_filter(filter).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing_subscriber as global default");

    let envs = env::babyrite_envs();
    // "メッセージの取得", "ギルド内メッセージへのアクセス" に該当する
    let intents = GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGES;
    let mut client = serenity::Client::builder(&envs.discord_api_token, intents)
        .event_handler(handler::BabyriteHandler)
        .await.expect("Failed to create a new client");

    if let Err(why) = client.start().await {
        Err(anyhow::anyhow!("An error occurred while running the client: {:?}", why))?;
    }
    Ok(())
}
