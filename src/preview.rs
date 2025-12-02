use once_cell::sync::Lazy;
use regex::Regex;
use serenity::all::{ChannelId, ChannelType, Context, GuildChannel, GuildId, Message, MessageId};
use url::Url;

use crate::cache::CacheArgs;

pub static MESSAGE_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https://(?:ptb\.|canary\.)?discord\.com/channels/(\d+)/(\d+)/(\d+)").unwrap()
});

#[derive(serde::Deserialize, Debug)]
pub struct MessageLinkIDs {
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

#[derive(serde::Deserialize, Debug)]
pub struct Preview {
    pub message: Message,
    pub channel: GuildChannel,
}

#[derive(thiserror::Error, Debug)]
pub enum PreviewError {
    #[error("Failed to retrieve from cache.")]
    Cache,
    #[error("NSFW content previews are not permitted, but the channel is marked as NSFW.")]
    Nsfw,
    #[error("The channel is a private channel or private thread.")]
    Permission,
    #[allow(clippy::enum_variant_names)]
    #[error(transparent)]
    SerenityError(#[from] serenity::Error),
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
