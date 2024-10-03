use serenity::all::{Context, Ready};
use serenity::client::EventHandler;
use serenity::gateway::ActivityData;
use tracing::{debug, info};

pub struct BabyriteHandler;

#[serenity::async_trait]
impl EventHandler for BabyriteHandler {
    async fn ready(&self, ctx: Context, client: Ready) {
        info!(name: "babyrite Initialize Ready", "babyrite is ready! Connected as {}, id: {}", &client.user.name, &client.user.id);
        debug!(name: "babyrite Initialize Ready", "client: {:?}", client);
        ctx.set_activity(Some(ActivityData::playing(format!(
            "v{}",
            env!("CARGO_PKG_VERSION")
        ))));
    }
}
