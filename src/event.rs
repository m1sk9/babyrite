//! Event handling module for Discord events.
//!
//! This module implements the serenity [`EventHandler`] trait to handle
//! Discord gateway events such as ready and message events.

use crate::preview::{MessageLinkIDs, Preview};
use serenity::all::{ActivityData, Context, EventHandler, Message, Ready};
use serenity_builder::model::{
    embed::SerenityEmbed,
    message::{SerenityMessage, SerenityMessageMentionType},
};

/// Event handler for Babyrite bot.
pub struct BabyriteEventHandler;

#[serenity::async_trait]
impl EventHandler for BabyriteEventHandler {
    async fn ready(&self, ctx: Context, bot: Ready) {
        ctx.set_activity(
            ActivityData::custom(format!("Running v{}", env!("CARGO_PKG_VERSION"))).into(),
        );
        tracing::info!("{} is connected!", bot.user.name);
    }

    async fn message(&self, ctx: Context, request: Message) {
        if request.author.bot {
            return;
        }

        let request_guild_id = match request.guild_id {
            Some(id) => id,
            None => return,
        };

        let ids = match MessageLinkIDs::parse(&request.content) {
            Some(ids) if ids.guild_id == request_guild_id => ids,
            _ => return,
        };

        tracing::info!(
            "Begin generating the preview. (Requester: {})",
            &request.author.name
        );

        let preview = match Preview::get(ids, &ctx).await {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("{}", e);
                return;
            }
        };

        let (message, channel) = (preview.message, preview.channel);
        let embed = SerenityEmbed::builder()
            .description(message.content)
            .author_name(message.author.name.clone())
            .author_icon_url(message.author.avatar_url().unwrap_or_default())
            .footer_text(channel.name)
            .timestamp(message.timestamp)
            .color(0x7A4AFFu32)
            .image_url(
                message
                    .attachments
                    .first()
                    .map(|a| a.url.clone())
                    .unwrap_or_default(),
            )
            .build();

        let request_channel_id = request.channel_id;
        let message_builder = SerenityMessage::builder()
            .embeds(vec![embed])
            .mention_type(SerenityMessageMentionType::Reply(Box::new(request)))
            .build();

        let converted_message = match message_builder.convert() {
            Ok(m) => m,
            Err(e) => {
                tracing::error!(?e);
                return;
            }
        };

        if let Err(e) = request_channel_id
            .send_message(&ctx.http, converted_message)
            .await
        {
            tracing::error!("Failed to send preview: {:?}", e);
        }

        tracing::info!("Preview sent successfully.")
    }
}
