use serenity::model::prelude::{ChannelId, GuildChannel};
use tracing::{debug, info};

use crate::model::cache::CHANNEL_LIST_CACHE;

pub async fn save_cache(key: ChannelId, value: GuildChannel) {
    CHANNEL_LIST_CACHE.insert(key, value).await;
    debug!(
        "キャッシュへの保存が完了しました.: {:?}",
        CHANNEL_LIST_CACHE
    );
    info!("キャッシュの初期化が完了しました.")
}

pub async fn get_cache(key_id: ChannelId) -> Option<GuildChannel> {
    let list = CHANNEL_LIST_CACHE.get(&key_id).await;
    debug!("キャッシュを取得しました: {:?}", key_id);
    list
}
