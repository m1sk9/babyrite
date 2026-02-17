//! Discord message link expansion.
//!
//! This module provides functionality for parsing Discord message links
//! and generating embed previews of the linked messages.
//!
//! Migrated from `preview.rs` with support for multiple link expansion.

use regex::Regex;
use serenity::all::{
    ChannelId, ChannelType, Context, GuildChannel, GuildId, Message, MessageId,
    PermissionOverwriteType, Permissions, RoleId,
};
use serenity_builder::model::embed::SerenityEmbed;
use std::collections::HashSet;
use std::sync::LazyLock;
use url::Url;

use super::{ExpandError, ExpandedContent};
use crate::cache::CacheArgs;

/// Regex pattern for matching Discord message links.
///
/// Supports production, PTB, and Canary Discord URLs.
pub static MESSAGE_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https://(?:ptb\.|canary\.)?discord\.com/channels/(\d+)/(\d+)/(\d+)").unwrap()
});

/// Parsed IDs from a Discord message link.
#[derive(Debug)]
pub struct MessageLinkIDs {
    /// The guild ID from the message link.
    pub guild_id: GuildId,
    /// The channel ID from the message link.
    pub channel_id: ChannelId,
    /// The message ID from the message link.
    pub message_id: MessageId,
}

/// A preview containing the message and its channel.
#[derive(serde::Deserialize, Debug)]
pub struct Preview {
    /// The message to preview.
    pub message: Message,
    /// The channel containing the message.
    pub channel: GuildChannel,
}

/// Errors that can occur when generating a Discord message preview.
#[derive(thiserror::Error, Debug)]
pub enum PreviewError {
    /// Failed to retrieve channel information from cache.
    #[error("Failed to retrieve from cache.")]
    Cache,
    /// The target channel is marked as NSFW.
    #[error("NSFW content previews are not permitted, but the channel is marked as NSFW.")]
    Nsfw,
    /// The target channel is private or a private thread.
    #[error("The channel is a private channel or private thread.")]
    Permission,
    /// An error occurred while communicating with Discord.
    #[allow(clippy::enum_variant_names)]
    #[error(transparent)]
    SerenityError(#[from] serenity::Error),
}

impl MessageLinkIDs {
    /// Parses all Discord message links from the given text.
    ///
    /// Returns a `Vec<MessageLinkIDs>` containing all valid message links found.
    ///
    /// Note: Duplicate URLs are ignored, and a maximum of 3 links are returned.
    pub fn parse_all(text: &str) -> Vec<MessageLinkIDs> {
        let mut seen_urls = HashSet::new();
        MESSAGE_LINK_REGEX
            .captures_iter(text)
            .filter_map(|captures| {
                let full_url = captures.get(0)?.as_str();
                if !seen_urls.insert(full_url.to_string()) {
                    return None;
                }

                let url = Url::parse(full_url).ok()?;

                if !matches!(
                    url.domain(),
                    Some("discord.com") | Some("canary.discord.com") | Some("ptb.discord.com")
                ) {
                    return None;
                }

                let guild_id = GuildId::new(captures.get(1)?.as_str().parse().ok()?);
                let channel_id = ChannelId::new(captures.get(2)?.as_str().parse().ok()?);
                let message_id = MessageId::new(captures.get(3)?.as_str().parse().ok()?);

                Some(MessageLinkIDs {
                    guild_id,
                    channel_id,
                    message_id,
                })
            })
            .take(3) // Limit to maximum 3 links
            .collect()
    }

    /// Fetches the linked message and returns an embed preview.
    pub async fn fetch(&self, ctx: &Context) -> Result<ExpandedContent, ExpandError> {
        let preview = Preview::get(self, ctx).await?;
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

        Ok(ExpandedContent::Embed(Box::new(embed)))
    }
}

impl Preview {
    /// Retrieves a preview for the given message link.
    ///
    /// Fetches the message and channel information, validating that
    /// the channel is not NSFW and is publicly accessible.
    async fn get(args: &MessageLinkIDs, ctx: &Context) -> Result<Preview, PreviewError> {
        let caches = CacheArgs {
            guild_id: args.guild_id,
            channel_id: args.channel_id,
        };

        let channel = caches.get(ctx).await.map_err(|_| PreviewError::Cache)?;

        if channel.nsfw {
            return Err(PreviewError::Nsfw);
        }

        if matches!(
            channel.kind,
            ChannelType::Private | ChannelType::PrivateThread
        ) {
            return Err(PreviewError::Permission);
        }

        let everyone_role_id = RoleId::new(args.guild_id.get());
        if channel
            .permission_overwrites
            .iter()
            .any(|overwrite| {
                matches!(overwrite.kind, PermissionOverwriteType::Role(role_id) if role_id == everyone_role_id)
                    && overwrite.deny.contains(Permissions::VIEW_CHANNEL)
            })
        {
            return Err(PreviewError::Permission);
        }

        let message = channel.message(&ctx.http, args.message_id).await?;
        Ok(Preview { message, channel })
    }
}
