use std::sync::Arc;

use anyhow::{Context, Error, Ok};
use serenity::{http::Http, model::channel::Message};

use crate::adapters::embed::convert_embed;
use crate::model::embed::EmbedMessageImage;
use crate::model::{
    embed::{EmbedMessage, EmbedMessageAuthor, EmbedMessageFooter},
    id::DiscordIds,
    message::CitationMessage,
};

const PERSONAL_COLOR: u32 = 0xb586f7;
const WARN_COLOR: u32 = 0xfff700;
const ERROR_COLOR: u32 = 0xFF0012;

pub async fn send_citation_embed(
    ids: DiscordIds,
    http: &Arc<Http>,
    target_message: &Message,
) -> anyhow::Result<()> {
    let citation_message = get_citation_message(ids, &http).await?;

    let embed_footer = EmbedMessageFooter::builder()
        .text(citation_message.channel_name)
        .build();
    let embed_author = EmbedMessageAuthor::builder()
        .name(citation_message.author_name)
        .icon_url(citation_message.author_avatar_url)
        .build();
    let embed_image = EmbedMessageImage::builder()
        .url(citation_message.attachment_image_url)
        .build();

    let embed_object = EmbedMessage::builder()
        .description(Some(citation_message.content))
        .footer(Some(embed_footer))
        .image(Some(embed_image))
        .author(Some(embed_author))
        .timestamp(Some(citation_message.create_at))
        .color(Some(PERSONAL_COLOR))
        .build();

    target_message
        .channel_id
        .send_message(http, |m| {
            m.reference_message(target_message);
            m.allowed_mentions(|mention| {
                mention.replied_user(true);
                mention
            });
            m.set_embed(convert_embed(embed_object))
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

    match guild_channels.get(&channel_id) {
        Some(channel) => {
            if channel.is_nsfw() || !channel.is_text_based() {
                return Err(anyhow::anyhow!("[ID: {}]のチャンネルはNSFWに指定されているか, テキストベースのチャンネルではありません", channel_id));
            }

            let message = channel
                .message(http, message_id)
                .await
                .context("メッセージの取得に失敗しました")?;

            let author = message.clone().author;
            // アバター画像が存在していなくても埋め込み生成時に無視される
            let author_icon_url = author.avatar_url();

            let attachment_url = if !message.attachments.is_empty() {
                message
                    .attachments
                    .first()
                    .map(|attachment| attachment.url.clone())
            } else {
                None
            };

            Ok(CitationMessage::builder()
                .content(message.content)
                .attachment_image_url(attachment_url)
                .author_name(author.name)
                .author_avatar_url(author_icon_url)
                .channel_name(channel.clone().name)
                .create_at(message.timestamp)
                .build())
        }
        None => Err(anyhow::anyhow!(
            "[ID: {}]のチャンネルを見つけることが出来ませんでした",
            channel_id
        )),
    }
}

pub async fn send_warn_embed(
    http: &Arc<Http>,
    message: &Message,
    error_reason: &str,
) -> anyhow::Result<()> {
    let embed_object = EmbedMessage::builder()
        .title(Some("警告".to_string()))
        .description(Some(format!("```\n{}\n```", error_reason)))
        .color(Some(WARN_COLOR))
        .build();

    message
        .channel_id
        .send_message(http, |m| {
            m.reference_message(message);
            m.allowed_mentions(|mention| {
                mention.replied_user(true);
                mention
            });
            m.set_embed(convert_embed(embed_object))
        })
        .await
        .context("警告メッセージの送信に失敗しました")?;

    Ok(())
}

pub async fn send_error_embed(
    http: &Arc<Http>,
    message: &Message,
    error_reason: &Error,
) -> anyhow::Result<()> {
    let embed_object = EmbedMessage::builder()
        .title(Some("エラー".to_string()))
        .description(Some(format!("```\n{}\n```", error_reason)))
        .color(Some(ERROR_COLOR))
        .build();

    message
        .channel_id
        .send_message(http, |m| {
            m.reference_message(message);
            m.allowed_mentions(|mention| {
                mention.replied_user(true);
                mention
            });
            m.set_embed(convert_embed(embed_object))
        })
        .await
        .context("エラーメッセージの送信に失敗しました")?;

    Ok(())
}
