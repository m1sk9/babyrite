use crate::model::message::CitationMessage;
use serenity::builder::CreateEmbed;
use serenity::builder::{CreateEmbedAuthor, CreateEmbedFooter};
use tracing::{debug, info};

pub fn build_citation_embed(message: CitationMessage) -> anyhow::Result<CreateEmbed> {
    let footer = CreateEmbedFooter::new(message.channel_name);
    let author = CreateEmbedAuthor::new(message.author_name).icon_url(
        message
            .author_avatar_url
            .unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_string()),
    );
    let embed = CreateEmbed::new()
        .description(message.content)
        .footer(footer)
        .timestamp(message.create_at)
        .author(author)
        .color(0xb586f7);

    let embed = if let Some(image_url) = message.attachment_image_url {
        embed.image(image_url)
    } else {
        embed
    };

    debug!("{:?}", embed);
    info!("--- Generation of embed is complete.");
    Ok(embed)
}
