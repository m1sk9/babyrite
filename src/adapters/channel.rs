use anyhow::Context;
use serenity::{
    http::Http,
    model::prelude::{ChannelId, GuildChannel, GuildId},
};
use std::sync::Arc;

pub async fn get_channel(
    channel_id: ChannelId,
    guild_id: GuildId,
    http: &Arc<Http>,
) -> anyhow::Result<GuildChannel> {
    let guild_channels = guild_id
        .channels(&http)
        .await
        .context("チャンネルリストの取得に失敗しました")?;

    let channel = match guild_channels.get(&channel_id) {
        Some(channel) => channel.clone(),
        None => {
            let guild_threads = guild_id.get_active_threads(http).await?;
            let thread = match guild_threads.threads.iter().find(|c| c.id == channel_id) {
                Some(channel) => channel.clone(),
                None => {
                    return Err(anyhow::anyhow!("引用元チャンネルは見つかりませんでした."));
                }
            };
            thread
        }
    };

    Ok(channel)
}
