use crate::VERSION;
use serenity::{
    async_trait,
    model::prelude::Ready,
    prelude::{Context, EventHandler},
};
use tracing::info;

pub struct EvHandler;

#[async_trait]
impl EventHandler for EvHandler {
    async fn ready(&self, _: Context, bot: Ready) {
        info!(
            "Connected to {name}(ID:{id}). (Using babyrite v{version}).",
            name = bot.user.name,
            id = bot.user.id,
            version = VERSION
        )
    }
}
