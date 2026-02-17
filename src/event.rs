//! Event handling module for Discord events.
//!
//! This module implements the serenity [`EventHandler`] trait to handle
//! Discord gateway events such as ready and message events.

use crate::expand::ExpandedContent;
use crate::expand::discord::MessageLinkIDs;
use serenity::all::{ActivityData, Context, EventHandler, Message, Ready};
use serenity_builder::model::message::{SerenityMessage, SerenityMessageMentionType};

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

        let text = &request.content;
        let mut results = Vec::new();

        // Discord link expansion
        for ids in MessageLinkIDs::parse_all(text) {
            if ids.guild_id != request_guild_id {
                continue;
            }

            tracing::info!(
                "Begin generating the preview. (Requester: {})",
                &request.author.name
            );

            match ids.fetch(&ctx).await {
                Ok(content) => results.push(content),
                Err(e) => tracing::error!("{}", e),
            }
        }

        if results.is_empty() {
            return;
        }

        send_expanded_contents(&ctx, &request, results).await;
    }
}

/// Sends expanded contents as a reply to the original message.
async fn send_expanded_contents(ctx: &Context, request: &Message, results: Vec<ExpandedContent>) {
    let mut embeds = Vec::new();

    for result in results {
        match result {
            ExpandedContent::Embed(embed) => embeds.push(*embed),
        }
    }

    if embeds.is_empty() {
        return;
    }

    let message_builder = SerenityMessage::builder()
        .embeds(embeds)
        .mention_type(SerenityMessageMentionType::Reply(Box::new(request.clone())))
        .build();

    let converted_message = match message_builder.convert() {
        Ok(m) => m,
        Err(e) => {
            tracing::error!(?e);
            return;
        }
    };

    if let Err(e) = request
        .channel_id
        .send_message(&ctx.http, converted_message)
        .await
    {
        tracing::error!("Failed to send preview: {:?}", e);
        return;
    }

    tracing::info!("Preview sent successfully.");
}
