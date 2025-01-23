use crate::config::PreviewConfig;
use crate::message::MessageLinkIDs;
use serenity::all::{Context, CreateEmbed, Message, PermissionOverwriteType, ReactionType};
use serenity::builder::{
    CreateAllowedMentions, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage,
};
use serenity::model::Permissions;
use serenity::prelude::EventHandler;

pub struct PreviewHandler;

#[derive(thiserror::Error, Debug)]
pub enum PreviewHandlerError {
    #[error("Failed to get preview: {0}")]
    FailedToGetPreview(#[from] anyhow::Error),
}

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct PreviewEmbedArgs {
    pub content: String,
    pub author_name: String,
    pub author_avatar: Option<String>,
    pub channel_name: String,
    pub create_at: serenity::model::Timestamp,
    pub attachment_url: Option<String>,
}

#[serenity::async_trait]
impl EventHandler for PreviewHandler {
    async fn message(&self, ctx: Context, request: Message) {
        // check if the message is command or bot
        if request.content.starts_with("b!") || request.author.bot {
            return;
        };

        let config = PreviewConfig::get_config();
        let Some(ids) = MessageLinkIDs::parse_url(&request.content) else {
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

        let args = PreviewEmbedArgs::builder()
            .content(message.content.clone())
            .author_name(message.author.name.clone())
            .author_avatar(message.author.avatar_url().clone())
            .channel_name(channel.name)
            .create_at(message.timestamp)
            .attachment_url(message.attachments.first().map(|a| a.url.clone()))
            .build();
        let preview = CreateMessage::default()
            .embed(generate_preview(args))
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

fn generate_preview(args: PreviewEmbedArgs) -> CreateEmbed {
    CreateEmbed::default()
        .description(args.content)
        .author(
            CreateEmbedAuthor::new(&args.author_name).icon_url(
                args.author_avatar
                    .as_deref()
                    .unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png"),
            ),
        )
        .footer(CreateEmbedFooter::new(&args.channel_name))
        .timestamp(args.create_at)
        .image(args.attachment_url.unwrap_or_default())
        .color(0x7A4AFF)
}
