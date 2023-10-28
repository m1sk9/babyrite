use moka::future::Cache;
use once_cell::sync::Lazy;
use serenity::model::prelude::{ChannelId, GuildChannel};

pub static CHANNEL_LIST_CACHE: Lazy<Cache<ChannelId, GuildChannel>> =
    Lazy::new(|| Cache::new(10_000));
