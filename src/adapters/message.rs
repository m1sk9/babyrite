use std::sync::Arc;

use anyhow::{Context, Ok};
use serenity::all::{ChannelId, GuildChannel, GuildId};
use serenity::builder::{CreateAllowedMentions, CreateMessage};
use serenity::{http::Http, model::channel::Message, model::user::User};
use tracing::{debug, info};

use crate::adapters::embed::build_citation_embed;
use crate::model::cache::CHANNEL_LIST_CACHE;
use crate::model::message::CitationMessageAuthor;
use crate::model::{id::DiscordIds, message::CitationMessage};

pub async fn send_citation_embed(
    ids: DiscordIds,
    http: &Arc<Http>,
    target_message: &Message,
) -> anyhow::Result<()> {
    let message = get_citation_message(ids, http).await?;
    debug!("{:?}", &message);

    let embed = build_citation_embed(message).context("Failed to generate embed.")?;

    let allowed_mentions = CreateAllowedMentions::default().replied_user(true);
    let message_builder = CreateMessage::new()
        .reference_message(target_message)
        .allowed_mentions(allowed_mentions)
        .embed(embed);

    target_message
        .channel_id
        .send_message(http, message_builder)
        .await
        .context("Failed to send quote message.")?;

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
            get_citation_channel(channel_id, guild_id, http)
                .await
                .unwrap()
        })
        .await;
    debug!("{:?}", &CHANNEL_LIST_CACHE);

    if target_channel.is_nsfw() {
        anyhow::bail!("The channel is designated NSFW.")
    }

    let target_message = target_channel
        .message(http, message_id)
        .await
        .context("Failed to retrieve message.")?;

    if !target_message.embeds.is_empty() && target_message.content.is_empty() {
        anyhow::bail!("Message could not be citation because it contained embed.");
    }

    let author = get_citation_author(&target_message.clone().author)
        .await
        .context("Failed to retrieve author.")?;

    let attachment_url = if !target_message.attachments.is_empty() {
        target_message
            .attachments
            .first()
            .map(|attachment| attachment.clone().url)
    } else {
        None
    };

    let sticker_url = if !target_message.sticker_items.is_empty() {
        target_message
            .sticker_items
            .first()
            // Note: if invalid sticker, return None
            .map(|sticker| sticker.clone().image_url().unwrap())
    } else {
        None
    };

    info!("--- Retrieval of citation message has been completed.");
    Ok(CitationMessage::builder()
        .content(target_message.content)
        .attachment_image_url(attachment_url)
        .sticker_url(sticker_url)
        .author(author)
        .channel_name(target_channel.clone().name)
        .create_at(target_message.timestamp)
        .build())
}

pub async fn get_citation_channel(
    channel_id: ChannelId,
    guild_id: GuildId,
    http: &Arc<Http>,
) -> anyhow::Result<GuildChannel> {
    let guild_channels = guild_id
        .channels(&http)
        .await
        .context("Failed to retrieve channel list.")?;

    let channel = match guild_channels.get(&channel_id) {
        Some(channel) => {
            debug!("Channel found from Discord API: {:?}.", channel);
            channel.clone()
        }
        None => {
            let guild_threads = guild_id.get_active_threads(http).await?;
            let thread = match guild_threads.threads.iter().find(|c| c.id == channel_id) {
                Some(channel) => {
                    debug!("Thread found from Discord API: {:?}.", channel);
                    channel.clone()
                }
                None => {
                    anyhow::bail!("Channel not found.");
                }
            };
            thread
        }
    };

    info!("--- Channel acquisition is complete.");
    Ok(channel)
}

pub async fn get_citation_author(author: &User) -> anyhow::Result<CitationMessageAuthor> {
    let name = author.name.clone();
    let icon_url = author.avatar_url();

    Ok(CitationMessageAuthor::builder()
        .name(name)
        .icon_url(icon_url)
        .build())
}
