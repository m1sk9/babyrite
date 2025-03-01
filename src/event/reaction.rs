use crate::utils::config::PreviewConfig;
use serenity::all::{EventHandler, Reaction};
use serenity::client::Context;

pub struct ReactionHandler;

#[serenity::async_trait]
impl EventHandler for ReactionHandler {
    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        if reaction.emoji.to_string() != "🗑️" || !PreviewConfig::get_config().is_deletable {
            return;
        }

        match reaction.message(&ctx.http).await {
            Ok(m) => {
                if m.author.id != ctx.cache.current_user().id {
                    return;
                }

                if let Some(referenced_message) = m.referenced_message.clone() {
                    if referenced_message.author.id != reaction.user_id.unwrap_or_default() {
                        return;
                    }
                } else {
                    return;
                }

                if let Err(e) = m.delete(&ctx.http).await {
                    tracing::error!("Failed to delete preview: {:?}", e);
                }

                tracing::info!(
                    "Deleted preview. Requested by {}",
                    reaction.user_id.unwrap_or_default()
                );
            }
            _ => {
                tracing::error!("Failed to get preview");
            }
        }
    }
}
