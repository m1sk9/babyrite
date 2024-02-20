use serenity::all::{Context, Ready};
use serenity::client::EventHandler;
use serenity::gateway::ActivityData;

pub struct EvHander;

#[serenity::async_trait]
impl EventHandler for EvHander {

    #[tracing::instrument(skip_all)]
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("Starting...");

        let version = env!("CARGO_PKG_VERSION");
        ctx.set_activity(Some(ActivityData::playing(format!("v{version}"))));

        tracing::info!(?version, "Running Babyrite");
        tracing::info!(username = %ready.user.name, user_id = %ready.user.id, "Connected to Discord API.");

        // TODO: add create guild command logic.
    }
}
