use crate::model::embed::EmbedMessage;
use serenity::{builder::CreateEmbed, utils::Colour};

pub fn convert_embed(
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
