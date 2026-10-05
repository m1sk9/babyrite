//! Event handling module for Discord events.
//!
//! [`handle`] is spawned once per gateway event received by the shard in
//! `main`, and dispatches it to the handling it needs.

use crate::cache::{invalidate_channel, invalidate_guild_roles};
use crate::config::BabyriteConfig;
use crate::context::BotContext;
use crate::expand::{EXPANDERS, ExpandContext, ExpandedContent};
use futures_util::future::join_all;
use std::sync::Arc;
use tracing::Instrument;
use twilight_gateway::Event;
use twilight_model::channel::{Channel, Message};
use twilight_model::id::{
    Id,
    marker::{ChannelMarker, MessageMarker},
};

/// Handles one gateway event.
pub async fn handle(ctx: Arc<BotContext>, event: Event) {
    // Every arm but `Ready`, `GatewayClose` and `MessageCreate` exists only to
    // keep the caches from outliving the permissions they hold.
    // `check_visibility` decides whether a link may be expanded from the cached
    // permission overwrites and role permissions, so a cached channel or role
    // that Discord has since restricted keeps being treated as visible.
    // Channel creations and deletions are included because the guild's channel
    // list is cached as one value, and a stale list answers for channels that
    // no longer exist and misses ones that now do.
    match event {
        Event::Ready(ready) => {
            tracing::info!(
                version = env!("CARGO_PKG_VERSION"),
                user = %ready.user.name,
                guilds = ready.guilds.len(),
                "connected"
            );
        }
        Event::ChannelCreate(channel) => on_channel_change(&channel).await,
        Event::ChannelUpdate(channel) => on_channel_change(&channel).await,
        Event::ChannelDelete(channel) => on_channel_change(&channel).await,
        Event::ThreadCreate(thread) => on_channel_change(&thread).await,
        Event::ThreadUpdate(thread) => on_channel_change(&thread).await,
        Event::ThreadDelete(thread) => invalidate_channel(thread.guild_id, thread.id).await,
        Event::RoleCreate(role) => invalidate_guild_roles(role.guild_id).await,
        Event::RoleUpdate(role) => invalidate_guild_roles(role.guild_id).await,
        Event::RoleDelete(role) => invalidate_guild_roles(role.guild_id).await,
        // Role events missed while disconnected are never replayed; the guild
        // arriving again is the only sign that its roles may have changed.
        Event::GuildCreate(guild) => invalidate_guild_roles(guild.id()).await,
        Event::GatewayClose(frame) => {
            tracing::warn!(?frame, "gateway connection closed; reconnecting");
        }
        Event::MessageCreate(message) => on_message(&ctx, &message).await,
        _ => {}
    }
}

async fn on_channel_change(channel: &Channel) {
    if let Some(guild_id) = channel.guild_id {
        invalidate_channel(guild_id, channel.id).await;
    }
}

async fn on_message(ctx: &BotContext, request: &Message) {
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
            ctx,
            message: request,
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

        send_expanded_contents(&ctx.http, request.channel_id, request.id, results).await;
    }
    .instrument(span)
    .await;
}

/// Sends expanded contents as a reply to the original message.
///
/// A failed send is reported and skipped rather than aborting the rest: the
/// expansions are independent, so one rejected message must not silence the
/// others.
async fn send_expanded_contents(
    http: &twilight_http::Client,
    channel_id: Id<ChannelMarker>,
    request_id: Id<MessageMarker>,
    results: Vec<ExpandedContent>,
) {
    let embeds = results
        .iter()
        .filter(|result| matches!(result, ExpandedContent::Embed(_)))
        .count();
    let code_blocks = results.len() - embeds;

    let messages = crate::reply::build_messages(request_id, results);
    let total = messages.len();
    let mut sent = 0;
    for message in &messages {
        let mut request = http
            .create_message(channel_id)
            .allowed_mentions(Some(&message.allowed_mentions));
        if let Some(content) = message.content.as_deref() {
            request = request.content(content);
        }
        if !message.embeds.is_empty() {
            request = request.embeds(&message.embeds);
        }
        if let Some(id) = message.reply_to {
            request = request.reply(id);
        }
        match request.await {
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
