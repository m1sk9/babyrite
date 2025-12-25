#![allow(clippy::enum_variant_names)]

use once_cell::sync::Lazy;
use regex::Regex;
use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::{message::Message, Channel, ChannelType},
    id::{
        marker::{ChannelMarker, GuildMarker, MessageMarker},
        Id,
    },
};
use url::Url;

use crate::cache::CacheArgs;

pub static MESSAGE_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https://(?:ptb\.|canary\.)?discord\.com/channels/(\d+)/(\d+)/(\d+)").unwrap()
});

#[derive(Debug)]
pub struct MessageLinkIDs {
    pub guild_id: Id<GuildMarker>,
    pub channel_id: Id<ChannelMarker>,
    pub message_id: Id<MessageMarker>,
}

#[derive(Debug)]
pub struct Preview {
    pub message: Message,
    pub channel: Channel,
}

#[derive(thiserror::Error, Debug)]
pub enum PreviewError {
    #[error("Failed to retrieve from cache.")]
    Cache,
    #[error("NSFW content previews are not permitted, but the channel is marked as NSFW.")]
    Nsfw,
    #[error("The channel is a private channel or private thread.")]
    Permission,
    #[error(transparent)]
    TwilightHttpError(#[from] twilight_http::Error),
    #[error(transparent)]
    DeserializeError(#[from] twilight_http::response::DeserializeBodyError),
}

impl MessageLinkIDs {
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

                let guild_id: u64 = captures.get(1)?.as_str().parse().ok()?;
                let channel_id: u64 = captures.get(2)?.as_str().parse().ok()?;
                let message_id: u64 = captures.get(3)?.as_str().parse().ok()?;

                Some(MessageLinkIDs {
                    guild_id: Id::new(guild_id),
                    channel_id: Id::new(channel_id),
                    message_id: Id::new(message_id),
                })
            }
            _ => None,
        }
    }
}

impl Preview {
    pub async fn get(args: MessageLinkIDs, http: &HttpClient) -> Result<Preview, PreviewError> {
        let caches = CacheArgs {
            guild_id: args.guild_id,
            channel_id: args.channel_id,
        };

        let channel = caches.get(http).await.map_err(|_| PreviewError::Cache)?;

        if channel.nsfw.unwrap_or(false) {
            return Err(PreviewError::Nsfw);
        }

        if matches!(
            channel.kind,
            ChannelType::Private | ChannelType::PrivateThread
        ) {
            return Err(PreviewError::Permission);
        }

        let message = http
            .message(args.channel_id, args.message_id)
            .await?
            .model()
            .await?;

        Ok(Preview { message, channel })
    }
}
