use serenity::all::{ActivityData, Context, Ready};
use serenity::prelude::EventHandler;

pub struct ReadyHandler;

#[serenity::async_trait]
impl EventHandler for ReadyHandler {
    async fn ready(&self, ctx: Context, bot: Ready) {
        ctx.set_activity(
            ActivityData::custom(format!("Running v{}", env!("CARGO_PKG_VERSION"))).into(),
        );
        tracing::info!("{} is connected!", bot.user.name);
    }
}
