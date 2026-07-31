//! Event handling module for Discord events.
//!
//! This module implements the serenity [`EventHandler`] trait to handle
//! Discord gateway events such as ready and message events.

use crate::config::BabyriteConfig;
use crate::expand::{EXPANDERS, ExpandContext, ExpandedContent};
use serenity::all::{
    ActivityData, Context, CreateAllowedMentions, CreateMessage, EventHandler, Message, Ready,
};
use serenity::futures::future::join_all;
use serenity_builder::model::message::{SerenityMessage, SerenityMessageMentionType};
use tracing::Instrument;

/// Event handler for Babyrite bot.
pub struct BabyriteEventHandler;

#[serenity::async_trait]
impl EventHandler for BabyriteEventHandler {
    async fn ready(&self, ctx: Context, bot: Ready) {
        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        ctx.set_activity(ActivityData::custom(format!("Running {}", version)).into());
        tracing::info!("Running {}, {} is connected!", version, bot.user.name);
    }

    async fn message(&self, ctx: Context, request: Message) {
        if request.author.bot {
            return;
        }

        let Some(request_guild_id) = request.guild_id else {
            return;
        };

        // Correlation span: every log emitted while handling this message
        // carries these fields, so a single request can be traced end-to-end
        // (e.g. via Grafana Loki). `request.id` is the unique Discord message
        // ID and serves as the correlation key.
        let span = tracing::info_span!(
            "message",
            message_id = %request.id,
            guild_id = %request_guild_id,
            channel_id = %request.channel_id,
            author = %request.author.name,
        );

        async {
            let config = BabyriteConfig::get();

            // Expanders are independent of each other, so run them concurrently.
            // `join_all` preserves registration order in the combined results.
            let cx = ExpandContext {
                ctx: &ctx,
                message: &request,
                guild_id: request_guild_id,
            };
            let results: Vec<ExpandedContent> = join_all(
                EXPANDERS
                    .iter()
                    .filter(|expander| expander.enabled(config))
                    .map(|expander| expander.expand_all(&cx)),
            )
            .await
            .into_iter()
            .flatten()
            .collect();

            if results.is_empty() {
                tracing::debug!("no expandable content found");
                return;
            }

            send_expanded_contents(&ctx, &request, results).await;
        }
        .instrument(span)
        .await;
    }
}

/// Sends expanded contents as a reply to the original message.
async fn send_expanded_contents(ctx: &Context, request: &Message, results: Vec<ExpandedContent>) {
    let mut embeds = Vec::new();
    let mut code_blocks = Vec::new();

    for result in results {
        match result {
            ExpandedContent::Embed(embed) => embeds.push(*embed),
            ExpandedContent::CodeBlock {
                language,
                code,
                metadata,
            } => {
                let code = crate::utils::defuse_code_fences(&code);
                code_blocks.push(format!("{metadata}\n```{language}\n{code}\n```"));
            }
        }
    }

    let embed_count = embeds.len();
    let code_block_count = code_blocks.len();

    // Send embeds if any
    if !embeds.is_empty() {
        let message_builder = SerenityMessage::builder()
            .embeds(embeds)
            .mention_type(SerenityMessageMentionType::Reply(Box::new(request.clone())))
            .build();

        let converted_message = match message_builder.convert() {
            Ok(m) => m,
            Err(e) => {
                tracing::error!(error = ?e, "failed to convert embed message");
                return;
            }
        };

        if let Err(e) = request
            .channel_id
            .send_message(&ctx.http, converted_message)
            .await
        {
            tracing::error!(error = ?e, "failed to send preview");
            return;
        }
    }

    // Send code blocks as plain messages.
    //
    // Not `say`: it leaves `allowed_mentions` unset, which makes Discord parse
    // every mention in the content under the bot's permissions. The content is
    // fetched from a repository the requester chose, so that would let it borrow
    // mention rights the requester may not hold. The embed path already sends an
    // explicit `allowed_mentions`.
    for block in code_blocks {
        let message = CreateMessage::new()
            .content(&block)
            .allowed_mentions(CreateAllowedMentions::new());
        if let Err(e) = request.channel_id.send_message(&ctx.http, message).await {
            tracing::error!(error = ?e, "failed to send code block");
        }
    }

    tracing::info!(
        embeds = embed_count,
        code_blocks = code_block_count,
        "preview sent"
    );
}
