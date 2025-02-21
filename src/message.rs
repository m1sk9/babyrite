use crate::utils::cache::get_channel_from_cache;
use once_cell::sync::Lazy;
use regex::Regex;
use serenity::all::{ChannelId, GuildChannel, GuildId, Message, MessageId};
use url::Url;

pub static MESSAGE_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https://(?:ptb\.|canary\.)?discord\.com/channels/(\d+)/(\d+)/(\d+)").unwrap()
});

#[derive(serde::Deserialize, Debug)]
pub struct MessageLinkIDs {
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

#[derive(Debug)]
pub struct MessagePreview {
    pub preview_message: Message,
    pub preview_channel: GuildChannel,
}

impl MessageLinkIDs {
    pub fn parse_url(text: &str) -> Option<MessageLinkIDs> {
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

    pub async fn get_message(
        &self,
        ctx: &serenity::prelude::Context,
    ) -> anyhow::Result<MessagePreview> {
        let guild = self.guild_id;
        let channel = get_channel_from_cache(self.channel_id, guild, ctx).await?;
        let message = channel.message(&ctx.http, self.message_id).await?;

        Ok(MessagePreview {
            preview_message: message,
            preview_channel: channel,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        let url = "https://canary.discord.com/channels/1331992336129069118/1331992336560947271/1332012065077854368";
        let ids = MessageLinkIDs::parse_url(url).unwrap();
        assert_eq!(ids.guild_id, GuildId::new(1331992336129069118));
        assert_eq!(ids.channel_id, ChannelId::new(1331992336560947271));
        assert_eq!(ids.message_id, MessageId::new(1332012065077854368));
    }

    #[test]
    fn test_invalid_domain() {
        let url = "https://discord.gg/invite";
        let ids = MessageLinkIDs::parse_url(url);
        assert!(ids.is_none());
    }
}
