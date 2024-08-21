use once_cell::sync::Lazy;
use regex::Regex;
use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter};

pub static DISCORD_LINK_PATTERN: &str =
    r"https://(?:ptb\.|canary\.)?discord\.com/channels/(\d+)/(\d+)/(\d+)";
pub static MESSAGE_LINK_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(DISCORD_LINK_PATTERN).unwrap());
pub static SKIP_MESSAGE_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    let pattern = format!("<{}>", DISCORD_LINK_PATTERN);
    Regex::new(&pattern).unwrap()
});

#[derive(Default, Debug, typed_builder::TypedBuilder)]
pub struct CitationMessageAuthor {
    pub name: String,

    #[builder(default = None)]
    pub icon_url: Option<String>,
}

#[derive(Default, Debug, typed_builder::TypedBuilder)]
pub struct CitationMessage {
    pub content: String,

    pub author: CitationMessageAuthor,

    pub channel_name: String,

    pub create_at: serenity::model::Timestamp,

    pub attachment_image_url: Option<String>,
}

impl CitationMessage {
    pub fn to_embed(&self) -> CreateEmbed {
        CreateEmbed::default()
            .description(&self.content)
            .timestamp(self.create_at)
            .color(0xb586f7)
            .author(
                CreateEmbedAuthor::new(&self.author.name).icon_url(
                    self.author
                        .icon_url
                        .clone()
                        .unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_string()),
                ),
            )
            .footer(CreateEmbedFooter::new(&self.channel_name))
            .image(
                self.attachment_image_url
                    .clone()
                    .map(|url| url.to_string())
                    .unwrap_or_default(),
            )
    }
}
