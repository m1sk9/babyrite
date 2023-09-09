use std::sync::Arc;

use anyhow::{Context, Ok};
use serenity::{http::Http, model::channel::Message};

use crate::adapters::embed::convert_embed;
use crate::model::embed::EmbedMessageImage;
use crate::model::{
    embed::{EmbedMessage, EmbedMessageAuthor, EmbedMessageFooter},
    id::DiscordIds,
    message::CitationMessage,
};

pub async fn send_citation_embed(
    ids: DiscordIds,
    http: &Arc<Http>,
    target_message: &Message,
) -> anyhow::Result<()> {
    let message = get_citation_message(ids, &http).await?;

    let footer = EmbedMessageFooter::builder()
        .text(message.channel_name)
        .build();
    let author = EmbedMessageAuthor::builder()
        .name(message.author_name)
        .icon_url(message.author_avatar_url)
        .build();
    let image = EmbedMessageImage::builder()
        .url(message.attachment_image_url)
        .build();

    let embed = EmbedMessage::builder()
        .description(Some(message.content))
        .footer(Some(footer))
        .image(Some(image))
        .author(Some(author))
        .timestamp(Some(message.create_at))
        .color(Some(0xb586f7))
        .build();

    target_message
        .channel_id
        .send_message(http, |m| {
            m.reference_message(target_message);
            m.allowed_mentions(|mention| {
                mention.replied_user(true);
                mention
            });
            m.set_embed(convert_embed(embed))
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
        Some(channel) => channel,
        None => return Err(anyhow::anyhow!("引用元チャンネルが見つかりませんでした")),
    };

    if target_channel.is_nsfw() {
        return Err(anyhow::anyhow!("引用元チャンネルはNSFWに指定されています"));
    }

    if !target_channel.is_text_based() {
        return Err(anyhow::anyhow!(
            "引用元チャンネルはテキストベースのチャンネルではありません"
        ));
    }

    let target_message = target_channel
        .message(http, message_id)
        .await
        .context("メッセージの取得に失敗しました")?;

    let author = target_message.clone().author;
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
        .author_name(author.name)
        .author_avatar_url(author_icon_url)
        .channel_name(target_channel.clone().name)
        .create_at(target_message.timestamp)
        .build())
}
