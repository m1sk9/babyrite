//! Message preview module.
//!
//! This module provides functionality for parsing Discord message links
//! and generating previews of the linked messages.

use once_cell::sync::Lazy;
use regex::Regex;
use serenity::all::{ChannelId, ChannelType, Context, GuildChannel, GuildId, Message, MessageId};
use url::Url;

use crate::cache::CacheArgs;

/// Regex pattern for matching Discord message links.
///
/// Supports production, PTB, and Canary Discord URLs.
pub static MESSAGE_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https://(?:ptb\.|canary\.)?discord\.com/channels/(\d+)/(\d+)/(\d+)").unwrap()
});

/// Parsed IDs from a Discord message link.
#[derive(serde::Deserialize, Debug)]
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

/// Errors that can occur when generating a preview.
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
    /// Parses a Discord message link from text.
    ///
    /// Returns `Some(MessageLinkIDs)` if a valid message link is found,
    /// otherwise returns `None`.
    pub fn parse(text: &str) -> Option<MessageLinkIDs> {
        if !MESSAGE_LINK_REGEX.is_match(text) {
            return None;
        }

        match MESSAGE_LINK_REGEX.captures(text) {
            Some(captures) => {
                let url = Url::parse(captures.get(0)?.as_str()).ok()?;

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
            }
            _ => None,
        }
    }
}

impl Preview {
    /// Retrieves a preview for the given message link.
    ///
    /// Fetches the message and channel information, validating that
    /// the channel is not NSFW and is publicly accessible.
    pub async fn get(args: MessageLinkIDs, ctx: &Context) -> Result<Preview, PreviewError> {
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

        let message = channel.message(&ctx.http, args.message_id).await?;
        Ok(Preview { message, channel })
    }
}
