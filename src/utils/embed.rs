// It is a self-builder to generate customized embeds for babyrite,
// with some properties compatible with Serenity's CreateEmbed,
// but with some changes, such as default values and no longer being of type Option<T> type.

use moka::ops::compute::Op;
use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter};
use serenity::model::{Colour, Timestamp};

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct BabyriteEmbed {
    #[builder(default, setter(into))]
    pub title: String,
    #[builder(setter(into))]
    pub description: String,
    #[builder(default, setter(strip_option, into))]
    pub url: Option<String>,
    #[builder(default, setter(strip_option, into))]
    pub timestamp: Option<Timestamp>,
    pub color: u32,
    pub footer: BabyriteEmbedFooter,
    #[builder(default, setter(strip_option, into))]
    pub image: Option<String>,
    #[builder(default, setter(strip_option, into))]
    pub thumbnail: Option<String>,
    pub author: BabyriteEmbedAuthor,
}

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct BabyriteEmbedAuthor {
    pub name: String,
    #[builder(default, setter(into))]
    pub icon_url: Option<String>,
}

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct BabyriteEmbedFooter {
    pub text: String,
    #[builder(default, setter(into))]
    pub icon_url: Option<String>,
}

impl Default for BabyriteEmbed {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            url: None,
            timestamp: None,
            color: 0x7A4AFF,
            footer: BabyriteEmbedFooter {
                text: format!("v{}", env!("CARGO_PKG_VERSION")),
                icon_url: Some("https://cdn.discordapp.com/embed/avatars/0.png".to_string()),
            },
            image: None,
            thumbnail: None,
            author: BabyriteEmbedAuthor {
                name: "Babyrite".to_string(),
                icon_url: Some("https://cdn.discordapp.com/embed/avatars/0.png".to_string()),
            },
        }
    }
}

impl BabyriteEmbed {
    pub fn build(&self) -> CreateEmbed {
        CreateEmbed::default()
            .title(self.title.clone())
            .description(self.description.clone())
            .url(self.url.clone().unwrap_or_default())
            .timestamp(self.timestamp.unwrap_or_default())
            .colour(Colour::new(self.color))
            .footer(
                CreateEmbedFooter::new(self.footer.text.clone())
                    .icon_url(self.footer.icon_url.clone().unwrap_or_default()),
            )
            .image(self.image.clone().unwrap_or_default())
            .thumbnail(self.thumbnail.clone().unwrap_or_default())
            .author(
                CreateEmbedAuthor::new(self.author.name.clone())
                    .icon_url(self.author.icon_url.clone().unwrap_or_default()),
            )
    }
}
