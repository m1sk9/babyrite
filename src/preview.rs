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
    /// Extracts MessagePreviewIDs from a given string containing a Discord message link.
    ///
    /// # Arguments
    ///
    /// * `content` - A string slice that holds the content to search for a Discord message link.
    ///
    /// # Returns
    ///
    /// * `anyhow::Result<MessagePreviewIDs, MessagePreviewError>` - A result containing either the extracted MessagePreviewIDs or an error.
    pub fn find_from_str(content: &str) -> anyhow::Result<MessagePreviewIDs, MessagePreviewError> {
        let url = MESSAGE_LINK_REGEX
            .captures(content)
            .and_then(|cap| cap.get(0).map(|m| m.as_str().to_string()));
        let url = url.ok_or_else(|| MessagePreviewError::NotFound("URL".to_string()))?;
        let message_link = Url::parse(&url)?;

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

    /// Retrieves a message preview based on the stored IDs.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context in which the command was executed.
    /// * `is_private` - A boolean indicating if the message is in a private channel.
    ///
    /// # Returns
    ///
    /// * `anyhow::Result<MessagePreview, MessagePreviewError>` - A result containing either the message preview or an error.
    pub async fn get_preview(
        &self,
        ctx: &Context,
        is_private: bool,
    ) -> anyhow::Result<MessagePreview, MessagePreviewError> {
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

        if guild != channel.guild_id {
            return Err(MessagePreviewError::GuildError(
                "Mismatched guilds".to_string(),
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

    /// Generates an embed for the message preview.
    ///
    /// # Arguments
    ///
    /// * `preview` - The message preview to generate an embed for.
    ///
    /// # Returns
    ///
    /// * `CreateEmbed` - The generated embed.
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
    /// Tests the `find_from_str` function with a valid URL.
    fn test_find_from_str_valid_url() {
        let url =
            "https://discord.com/channels/123456789012345678/987654321098765432/123456789012345678";
        let result = MessagePreviewIDs::find_from_str(url).unwrap();
        assert_eq!(result.guild_id, GuildId::new(123456789012345678));
        assert_eq!(result.channel_id, ChannelId::new(987654321098765432));
        assert_eq!(result.message_id, MessageId::new(123456789012345678));
    }

    #[test]
    /// Tests the `find_from_str` function with an invalid URL.
    fn test_find_from_str_invalid_url() {
        let url =
            "https://example.com/channels/123456789012345678/987654321098765432/123456789012345678";
        let result = MessagePreviewIDs::find_from_str(url);
        assert!(result.is_err());
    }

    #[test]
    /// Tests the `find_from_str` function with a discordapp.com domain.
    fn test_find_from_str_discordapp_domain() {
        let url =
            "https://discordapp.com/channels/123456789012345678/987654321098765432/123456789012345678";
        let result = MessagePreviewIDs::find_from_str(url);
        assert!(result.is_err());
    }

    #[test]
    /// Tests the `find_from_str` function with a contain text.
    fn test_find_from_str_contain_text() {
        let url_before_text = "hello https://discord.com/channels/123456789012345678/987654321098765432/123456789012345678";
        let url_after_text = "https://discord.com/channels/123456789012345678/987654321098765432/123456789012345678 hello";
        let result_before = MessagePreviewIDs::find_from_str(url_before_text);
        let result_after = MessagePreviewIDs::find_from_str(url_after_text);
        assert!(result_before.is_ok());
        assert!(result_after.is_ok());
    }
}
