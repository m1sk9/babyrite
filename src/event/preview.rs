use serenity::all::{Context, Message, PermissionOverwriteType, ReactionType};
use serenity::builder::{CreateAllowedMentions, CreateMessage};
use serenity::model::Permissions;
use serenity::prelude::EventHandler;

pub struct PreviewHandler;

#[derive(thiserror::Error, Debug)]
pub enum PreviewHandlerError {
    #[error("Failed to get preview: {0}")]
    FailedToGetPreview(#[from] anyhow::Error),
}

#[serenity::async_trait]
impl EventHandler for PreviewHandler {
    async fn message(&self, ctx: Context, request: Message) {
        if request.author.bot {
            return;
        };

        let config = crate::PreviewConfig::get_config();
        let Some(ids) = crate::message::MessageLinkIDs::parse_url(&request.content) else {
            return;
        };

        tracing::info!(
            "Start processing citation requests from {} ({:?})",
            request.author.name,
            ids
        );
        let preview = match ids.get_message(&ctx).await {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to get preview: {:?}", e);
                return;
            }
        };
        let (message, channel) = (preview.preview_message, preview.preview_channel);
        tracing::debug!("Message: {:?}, Channel: {:?}", message, channel);

        // Verify that: @everyone on the previewer channel does not have read permission (i.e. limit channel)
        if channel.nsfw && !config.is_allow_nsfw
            || channel.permission_overwrites.iter().any(|p| {
                matches!(p.kind, PermissionOverwriteType::Role(_))
                    && p.deny.contains(Permissions::VIEW_CHANNEL)
            })
        {
            return;
        }

        let embed = crate::utils::embed::BabyriteEmbed::builder()
            .description(message.content.clone())
            .author(
                crate::utils::embed::BabyriteEmbedAuthor::builder()
                    .name(message.author.name.clone())
                    .icon_url(message.author.avatar_url())
                    .build(),
            )
            .footer(
                crate::utils::embed::BabyriteEmbedFooter::builder()
                    .text(channel.name.clone())
                    .build(),
            )
            .timestamp(message.timestamp)
            .image(
                message
                    .attachments
                    .first()
                    .map(|a| a.url.clone())
                    .unwrap_or_default(),
            )
            .color(0x7A4AFF)
            .build();
        let preview = CreateMessage::default()
            .embed(embed.build())
            .reference_message(&request)
            .reactions(match config.is_deletable {
                true => vec![ReactionType::Unicode("🗑️".to_string())],
                false => vec![],
            })
            .allowed_mentions(CreateAllowedMentions::new().replied_user(config.is_mention));

        if let Err(e) = request.channel_id.send_message(&ctx.http, preview).await {
            tracing::error!("Failed to send preview: {:?}", e);
        }

        tracing::info!("-- Preview sent successfully.");
    }
}
