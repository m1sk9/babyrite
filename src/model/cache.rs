use moka::future::{Cache, CacheBuilder};
use once_cell::sync::Lazy;
use serenity::model::prelude::{ChannelId, GuildChannel};
use std::time::Duration;

pub static CHANNEL_LIST_CACHE: Lazy<Cache<ChannelId, GuildChannel>> = Lazy::new(|| {
    CacheBuilder::new(1000)
        .time_to_live(Duration::from_secs(3600))
        .time_to_idle(Duration::from_secs(30 * 30))
        .build()
});
