use std::sync::Arc;

use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::message::Message,
    gateway::payload::incoming::Ready,
    id::marker::GuildMarker,
    user::User,
};
use twilight_util::builder::embed::{EmbedAuthorBuilder, EmbedBuilder, EmbedFooterBuilder, ImageSource};

use crate::preview::{MessageLinkIDs, Preview};

/// Get the avatar URL for a user
fn get_avatar_url(user: &User) -> String {
    match &user.avatar {
        Some(hash) => {
            let ext = if hash.is_animated() { "gif" } else { "png" };
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.{}",
                user.id, hash, ext
            )
        }
        None => {
            // Default avatar based on user ID
            let index = (user.id.get() >> 22) % 6;
            format!("https://cdn.discordapp.com/embed/avatars/{}.png", index)
        }
    }
}

pub fn on_ready(ready: &Ready) {
    tracing::info!("{} is connected!", ready.user.name);
}

pub async fn on_message(http: Arc<HttpClient>, request: Message) {
    if request.author.bot {
        return;
    }

    let request_guild_id: twilight_model::id::Id<GuildMarker> = match request.guild_id {
        Some(id) => id,
        None => return,
    };

    let ids = match MessageLinkIDs::parse(&request.content) {
        Some(ids) if ids.guild_id == request_guild_id => ids,
        _ => return,
    };

    tracing::info!(
        "Begin generating the preview. (Requester: {})",
        &request.author.name
    );

    let preview = match Preview::get(ids, &http).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("{}", e);
            return;
        }
    };

    let (message, channel) = (preview.message, preview.channel);

    // Build embed using twilight-util
    let mut author_builder = EmbedAuthorBuilder::new(&message.author.name);
    if let Ok(source) = ImageSource::url(get_avatar_url(&message.author)) {
        author_builder = author_builder.icon_url(source);
    }

    let mut embed_builder = EmbedBuilder::new()
        .description(&message.content)
        .author(author_builder.build())
        .color(0x7A4AFF)
        .timestamp(message.timestamp);

    // Add footer with channel name
    if let Some(name) = &channel.name {
        embed_builder = embed_builder.footer(EmbedFooterBuilder::new(name).build());
    }

    // Add image if attachment exists
    if let Some(attachment) = message.attachments.first()
        && let Ok(source) = ImageSource::url(&attachment.url)
    {
        embed_builder = embed_builder.image(source);
    }

    let embed = embed_builder.build();

    // Send reply using twilight-http
    if let Err(e) = http
        .create_message(request.channel_id)
        .embeds(&[embed])
        .reply(request.id)
        .await
    {
        tracing::error!("Failed to send preview: {:?}", e);
        return;
    }

    tracing::info!("Preview sent successfully.")
}
