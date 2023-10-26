use std::sync::Arc;

use anyhow::{Context, Ok};
use serenity::{http::Http, model::channel::Message};

use crate::adapters::embed::build_citation_embed;
use crate::model::{id::DiscordIds, message::CitationMessage};

pub async fn send_citation_embed(
    ids: DiscordIds,
    http: &Arc<Http>,
    target_message: &Message,
) -> anyhow::Result<()> {
    let message = get_citation_message(ids, &http).await?;
    let embed = build_citation_embed(message).context("埋め込みの生成に失敗しました")?;

    target_message
        .channel_id
        .send_message(http, |m| {
            m.reference_message(target_message);
            m.allowed_mentions(|mention| {
                mention.replied_user(true);
                mention
            });
            m.set_embed(embed)
        })
        .await
        .context("引用メッセージの送信に失敗しました")?;

    Ok(())
}

async fn get_citation_message(
    DiscordIds {
        guild_id,
        channel_id,
        message_id,
    }: DiscordIds,
    http: &Arc<Http>,
) -> anyhow::Result<CitationMessage> {
    let guild_channels = guild_id
        .channels(&http)
        .await
        .context("チャンネルリストの取得に失敗しました")?;

    let target_channel = match guild_channels.get(&channel_id) {
        Some(channel) => channel.clone(),
        None => {
            let guild_threads = guild_id
                .get_active_threads(http)
                .await
                .context("スレッドリストの取得に失敗しました")?;
            let thread = match guild_threads.threads.iter().find(|c| &c.id == &channel_id) {
                Some(channel) => channel.clone(),
                None => {
                    return Err(anyhow::anyhow!("引用元チャンネルは見つかりませんでした."));
                }
            };
            thread
        }
    };

    if target_channel.is_nsfw() {
        return Err(anyhow::anyhow!("引用元チャンネルはNSFWに指定されています"));
    }

    let target_message = target_channel
        .message(http, message_id)
        .await
        .context("メッセージの取得に失敗しました")?;

    if !target_message.embeds.is_empty() {
        return Err(anyhow::anyhow!(
            "引用しようとしたメッセージは埋め込みを含んでいるため引用できませんでした"
        ));
    }

    let author = target_message.clone().author;
    let author_name = if author.bot {
        format!("{} [🤖]", author.tag())
    } else {
        author.name.clone()
    };
    // アバターが存在していなくても埋め込みに問題はない
    let author_icon_url = author.avatar_url();

    let attachment_url = if !target_message.attachments.is_empty() {
        target_message
            .attachments
            .first()
            .map(|attachment| attachment.clone().url)
    } else {
        None
    };

    Ok(CitationMessage::builder()
        .content(target_message.content)
        .attachment_image_url(attachment_url)
        .author_name(author_name)
        .author_avatar_url(author_icon_url)
        .channel_name(target_channel.clone().name)
        .create_at(target_message.timestamp)
        .build())
}
