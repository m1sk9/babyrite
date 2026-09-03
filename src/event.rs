//! Event handling module for Discord events.
//!
//! This module implements the serenity [`EventHandler`] trait to handle
//! Discord gateway events such as ready and message events.

use crate::cache::invalidate_channel;
use crate::config::BabyriteConfig;
use crate::expand::{EXPANDERS, ExpandContext, ExpandedContent};
use serenity::all::{
    ActivityData, Context, EventHandler, GuildChannel, Message, PartialGuildChannel, Ready,
};
use serenity::futures::future::join_all;
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

    // The six handlers below exist only to keep the channel caches from
    // outliving the permissions they hold. `check_visibility` decides whether a
    // link may be expanded from the cached permission overwrites, so a cached
    // channel that Discord has since restricted keeps being treated as visible.
    // Creations and deletions are included because the guild's channel list is
    // cached as one value, and a stale list answers for channels that no longer
    // exist and misses ones that now do.

    async fn channel_create(&self, _ctx: Context, channel: GuildChannel) {
        invalidate_channel(channel.guild_id, channel.id).await;
    }

    async fn channel_update(&self, _ctx: Context, _old: Option<GuildChannel>, new: GuildChannel) {
        invalidate_channel(new.guild_id, new.id).await;
    }

    async fn channel_delete(
        &self,
        _ctx: Context,
        channel: GuildChannel,
        _messages: Option<Vec<Message>>,
    ) {
        invalidate_channel(channel.guild_id, channel.id).await;
    }

    async fn thread_create(&self, _ctx: Context, thread: GuildChannel) {
        invalidate_channel(thread.guild_id, thread.id).await;
    }

    async fn thread_update(&self, _ctx: Context, _old: Option<GuildChannel>, new: GuildChannel) {
        invalidate_channel(new.guild_id, new.id).await;
    }

    async fn thread_delete(
        &self,
        _ctx: Context,
        thread: PartialGuildChannel,
        _full_thread_data: Option<GuildChannel>,
    ) {
        invalidate_channel(thread.guild_id, thread.id).await;
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
///
/// A failed send is reported and skipped rather than aborting the rest: the
/// expansions are independent, so one rejected message must not silence the
/// others.
async fn send_expanded_contents(ctx: &Context, request: &Message, results: Vec<ExpandedContent>) {
    let embeds = results
        .iter()
        .filter(|result| matches!(result, ExpandedContent::Embed(_)))
        .count();
    let code_blocks = results.len() - embeds;

    let messages = crate::reply::build_messages(request, results);
    let total = messages.len();
    let mut sent = 0;
    for message in messages {
        match request.channel_id.send_message(&ctx.http, message).await {
            Ok(_) => sent += 1,
            Err(e) => tracing::error!(error = ?e, "failed to send expanded content"),
        }
    }

    // `sent`/`total` count messages, not expansions: every embed shares one.
    if sent == total {
        tracing::info!(embeds, code_blocks, "preview sent");
    } else {
        tracing::warn!(embeds, code_blocks, sent, total, "preview partially sent");
    }
}
