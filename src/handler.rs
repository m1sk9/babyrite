use serenity::all::{Context, Ready};
use serenity::client::EventHandler;
use serenity::gateway::ActivityData;

pub struct BabyriteHandler;

#[serenity::async_trait]
impl EventHandler for BabyriteHandler {
    async fn ready(&self, ctx: Context, client: Ready) {
        tracing::info!("Connected as {}, id: {}", &client.user.name, &client.user.id);
        ctx.set_activity(Some(ActivityData::playing(format!("babyrite v{}", env!("CARGO_PKG_VERSION")))));
        tracing::debug!("client: {:?}", client);
        tracing::info!("babyrite is ready!");
    }
}
