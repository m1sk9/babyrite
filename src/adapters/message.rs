use std::sync::Arc;

use anyhow::{Context, Ok};
use serenity::builder::{CreateAllowedMentions, CreateMessage};
use serenity::{http::Http, model::channel::Message};
use tracing::info;

use crate::adapters::channel::get_channel;
use crate::adapters::embed::build_citation_embed;
use crate::model::cache::CHANNEL_LIST_CACHE;
use crate::model::{id::DiscordIds, message::CitationMessage};

pub async fn send_citation_embed(
    ids: DiscordIds,
    http: &Arc<Http>,
    target_message: &Message,
) -> anyhow::Result<()> {
    let message = get_citation_message(ids, http).await?;
    let embed = build_citation_embed(message).context("埋め込みの生成に失敗しました")?;

    let allowed_mentions = CreateAllowedMentions::default().replied_user(true);
    let message_builder = CreateMessage::new()
        .reference_message(target_message)
        .allowed_mentions(allowed_mentions)
        .embed(embed);

    target_message
        .channel_id
        .send_message(http, message_builder)
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
    let target_channel = CHANNEL_LIST_CACHE
        .get_with(channel_id, async move {
            get_channel(channel_id, guild_id, http).await.unwrap()
        })
        .await;

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

    info!("--- 引用メッセージの取得が完了しました.");
    Ok(CitationMessage::builder()
        .content(target_message.content)
        .attachment_image_url(attachment_url)
        .author_name(author_name)
        .author_avatar_url(author_icon_url)
        .channel_name(target_channel.clone().name)
        .create_at(target_message.timestamp)
        .build())
}
