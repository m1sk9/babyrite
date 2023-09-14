use crate::model::embed::{
    EmbedMessage, EmbedMessageAuthor, EmbedMessageFooter, EmbedMessageImage,
};
use crate::model::message::CitationMessage;
use serenity::{builder::CreateEmbed, utils::Colour};

fn convert_embed(
    EmbedMessage {
        title,
        author,
        description,
        fields,
        footer,
        color,
        url,
        image,
        thumbnail,
        timestamp,
    }: EmbedMessage,
) -> CreateEmbed {
    let mut create_embed = CreateEmbed::default();

    if let Some(title) = title {
        create_embed.title(title);
    }

    if let Some(author) = author {
        create_embed.author(|a| {
            a.name(author.name);
            if let Some(url) = author.url {
                a.url(url);
            }
            if let Some(icon_url) = author.icon_url {
                a.icon_url(icon_url);
            };
            a
        });
    }

    if let Some(description) = description {
        create_embed.description(description);
    }

    if let Some(fields) = fields {
        fields.into_iter().for_each(|x| {
            create_embed.field(x.name, x.value, x.inline.unwrap_or(false));
        })
    }

    if let Some(footer) = footer {
        create_embed.footer(|f| {
            f.text(footer.text);
            if let Some(icon_url) = footer.icon_url {
                f.icon_url(icon_url);
            };
            f
        });
    }

    if let Some(color) = color {
        create_embed.color(Colour(color));
    }

    if let Some(url) = url {
        create_embed.url(url);
    }

    if let Some(image) = image {
        if let Some(image_url) = image.url {
            create_embed.image(image_url);
        }
    }

    if let Some(thumbnail) = thumbnail {
        create_embed.thumbnail(thumbnail.url);
    }

    if let Some(timestamp) = timestamp {
        create_embed.timestamp(timestamp);
    }

    create_embed
}

pub fn build_citation_embed(message: CitationMessage) -> anyhow::Result<CreateEmbed> {
    let footer = EmbedMessageFooter::builder()
        .text(message.channel_name)
        .build();
    let author = EmbedMessageAuthor::builder()
        .name(message.author_name)
        .icon_url(message.author_avatar_url)
        .build();
    let image = EmbedMessageImage::builder()
        .url(message.attachment_image_url)
        .build();

    let embed = EmbedMessage::builder()
        .description(Some(message.content))
        .footer(Some(footer))
        .image(Some(image))
        .author(Some(author))
        .timestamp(Some(message.create_at))
        .color(Some(0xb586f7))
        .build();

    Ok(convert_embed(embed))
}
