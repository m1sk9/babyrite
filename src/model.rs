use once_cell::sync::Lazy;
use serenity::all::GuildChannel;
use serenity::model::prelude::{ChannelId, GuildId, MessageId};
use serenity::model::Timestamp;
use typed_builder::TypedBuilder;

#[derive(serde::Serialize, serde::Deserialize, Debug, TypedBuilder)]
pub struct DiscordID {
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

pub static CHANNEL_LIST_CACHE: Lazy<moka::future::Cache<ChannelId, GuildChannel>> =
    Lazy::new(|| {
        moka::future::CacheBuilder::new(100)
            .time_to_live(std::time::Duration::from_secs(60))
            .build()
    });

#[derive(serde::Serialize, serde::Deserialize, Debug, TypedBuilder)]
pub struct CitationMessageAuthor {
    pub name: String,
    pub icon_url: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, TypedBuilder)]
pub struct CitationMessage {
    pub content: String,
    pub author: CitationMessageAuthor,
    pub channel_name: String,
    pub create_at: Timestamp,
    pub attachment_image_url: Option<String>,
    pub sticker_image_url: Option<String>,
}
