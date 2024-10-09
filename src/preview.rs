use once_cell::sync::Lazy;
use regex::Regex;
use serenity::all::{ChannelId, Colour, Context, CreateEmbed, GuildChannel, GuildId, MessageId};
use serenity::builder::{CreateEmbedAuthor, CreateEmbedFooter};
use serenity::model::channel::PermissionOverwriteType;
use serenity::model::Permissions;
use tracing::log::debug;
use url::Url;

pub static MESSAGE_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https://(?:ptb\.|canary\.)?discord\.com/channels/(\d+)/(\d+)/(\d+)").unwrap()
});

pub static SKIP_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<https://(?:ptb\.|canary\.)?discord\.com/channels/(\d+)/(\d+)/(\d+)>").unwrap()
});

#[derive(serde::Deserialize, Debug)]
pub struct MessagePreviewIDs {
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

#[derive(Default, Debug)]
pub struct MessagePreviewAuthor {
    pub name: String,
    pub icon_url: Option<String>,
    pub color: Option<Colour>,
}

#[derive(Default, Debug)]
pub struct MessagePreview {
    pub content: String,
    pub author: MessagePreviewAuthor,
    pub channel_name: String,
    pub create_at: serenity::model::Timestamp,
    pub attachment_image_url: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum MessagePreviewError {
    #[error("{0} not found")]
    NotFound(String),
    #[error("Failed to parse URL")]
    URLParseError(#[from] url::ParseError),
    #[error("Failed to get guild: {0}")]
    GuildError(String),
    #[error("Failed to get channel: {0}")]
    ChannelError(String),
    #[error("Failed to get message: {0}")]
    MessageError(String),
    #[error("Internal error!: {0}")]
    InternalError(anyhow::Error),
}

pub static MESSAGE_PREVIEW_CHANNEL_CACHE: Lazy<moka::future::Cache<ChannelId, GuildChannel>> = {
    Lazy::new(|| {
        moka::future::CacheBuilder::new(1000)
            .name("message_preview_channel_cache")
            .time_to_idle(std::time::Duration::from_secs(3600))
            .build()
    })
};

impl MessagePreviewIDs {
    pub fn find_from_str(content: &str) -> anyhow::Result<MessagePreviewIDs, MessagePreviewError> {
        let message_link = Url::parse(content)?;
        if !matches!(
            message_link.host_str(),
            Some("discord.com") | Some("canary.discord.com") | Some("ptb.discord.com")
        ) {
            return Err(MessagePreviewError::URLParseError(
                url::ParseError::RelativeUrlWithoutBase,
            ));
        }

        let mut segments = message_link.path_segments().unwrap();
        if segments.next() != Some("channels") {
            return Err(MessagePreviewError::URLParseError(
                url::ParseError::RelativeUrlWithoutBase,
            ));
        }

        let guild_id = GuildId::new(segments.next().unwrap().parse().unwrap());
        let channel_id = ChannelId::new(segments.next().unwrap().parse().unwrap());
        let message_id = MessageId::new(segments.next().unwrap().parse().unwrap());

        Ok(MessagePreviewIDs {
            guild_id,
            channel_id,
            message_id,
        })
    }

    pub async fn get_preview(
        &self,
        ctx: &Context,
        is_private: bool,
    ) -> anyhow::Result<MessagePreview, MessagePreviewError> {
        let config = crate::config::BabyriteConfig::get();
        let guild = self.guild_id;

        let channel =
            if let Some(channel) = MESSAGE_PREVIEW_CHANNEL_CACHE.get(&self.channel_id).await {
                channel
            } else {
                let channels = guild
                    .channels(&ctx.http)
                    .await
                    .map_err(|_| MessagePreviewError::NotFound("Guild".to_string()))?;
                if let Some(channel) = channels.get(&self.channel_id) {
                    channel.clone()
                } else {
                    let guild_threads = guild
                        .get_active_threads(&ctx.http)
                        .await
                        .map_err(|_| MessagePreviewError::NotFound("Active threads".to_string()))?;
                    guild_threads
                        .threads
                        .iter()
                        .find(|c| c.id == self.channel_id)
                        .cloned()
                        .ok_or_else(|| MessagePreviewError::NotFound("Channel".to_string()))?
                }
            };
        debug!("channel: {:?}", channel);

        if !config.preview.bypass_guild_check && guild != channel.guild_id {
            return Err(MessagePreviewError::GuildError(
                "mismatched guilds (set `bypass_guilds` to `true` to enable citation)".to_string(),
            ));
        }

        if !channel.is_text_based() || channel.nsfw {
            return Err(MessagePreviewError::ChannelError(
                "Channel is not text-based or NSFW".to_string(),
            ));
        }

        if !is_private
            && channel.permission_overwrites.iter().any(|p| {
                matches!(p.kind, PermissionOverwriteType::Role(_))
                    && p.deny.contains(Permissions::VIEW_CHANNEL)
            })
        {
            return Err(MessagePreviewError::ChannelError(
                "Channel is not viewable".to_string(),
            ));
        }

        let message = channel
            .message(&ctx.http, self.message_id)
            .await
            .map_err(|_| MessagePreviewError::MessageError("Message not found".to_string()))?;
        let content = message.content.clone();
        let author = MessagePreviewAuthor {
            name: message.author.name.clone(),
            icon_url: message.author.avatar_url(),
            color: ctx
                .http
                .get_user(message.author.id)
                .await
                .unwrap()
                .accent_colour,
        };
        let attachment_image_url = message
            .attachments
            .first()
            .map(|attachment| attachment.url.clone());

        if content.is_empty() && attachment_image_url.is_none() {
            return Err(MessagePreviewError::MessageError(
                "Message is empty".to_string(),
            ));
        }

        Ok(MessagePreview {
            content,
            author,
            channel_name: channel.name.clone(),
            create_at: message.timestamp,
            attachment_image_url,
        })
    }

    pub fn generate_preview(preview: MessagePreview) -> CreateEmbed {
        CreateEmbed::default()
            .color(preview.author.color.unwrap_or(Colour::new(0xFA6300)))
            .description(preview.content)
            .author(
                CreateEmbedAuthor::new(&preview.author.name).icon_url(
                    preview
                        .author
                        .icon_url
                        .as_deref()
                        .unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png"),
                ),
            )
            .footer(CreateEmbedFooter::new(&preview.channel_name))
            .timestamp(preview.create_at)
            .image(preview.attachment_image_url.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_from_str_valid_url() {
        let url =
            "https://discord.com/channels/123456789012345678/987654321098765432/123456789012345678";
        let result = MessagePreviewIDs::find_from_str(url).unwrap();
        assert_eq!(result.guild_id, GuildId::new(123456789012345678));
        assert_eq!(result.channel_id, ChannelId::new(987654321098765432));
        assert_eq!(result.message_id, MessageId::new(123456789012345678));
    }

    #[test]
    fn test_find_from_str_invalid_url() {
        let url =
            "https://example.com/channels/123456789012345678/987654321098765432/123456789012345678";
        let result = MessagePreviewIDs::find_from_str(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_from_str_discordapp_domain() {
        let url =
            "https://discordapp.com/channels/123456789012345678/987654321098765432/123456789012345678";
        let result = MessagePreviewIDs::find_from_str(url);
        assert!(result.is_err());
    }
}
