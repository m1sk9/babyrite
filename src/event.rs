use crate::model::CHANNEL_LIST_CACHE;
use anyhow::Context as _;
use once_cell::sync::Lazy;
use regex::Regex;
use serenity::all::{Context, CreateAllowedMentions, GuildChannel, Message, MessageType, Ready};
use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage};
use serenity::client::EventHandler;
use serenity::gateway::ActivityData;
use serenity::model::prelude::{ChannelId, GuildId, MessageId};

pub struct EvHander;

static DISCORD_LINK_PATTERN: &str =
    r"https://(?:ptb\.|canary\.)?discord\.com/channels/(\d+)/(\d+)/(\d+)";
static MESSAGE_LINK_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(DISCORD_LINK_PATTERN).unwrap());
static SKIP_MESSAGE_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    let pattern = format!("<{}>", DISCORD_LINK_PATTERN);
    Regex::new(&pattern).unwrap()
});

#[serenity::async_trait]
impl EventHandler for EvHander {
    #[tracing::instrument(skip_all)]
    async fn message(&self, ctx: Context, message: Message) {
        if message.is_private() || message.author.bot {
            return;
        }

        let content = &message.content;
        if !MESSAGE_LINK_REGEX.is_match(content) || SKIP_MESSAGE_LINK_REGEX.is_match(content) {
            return;
        }
        let msg_link = MESSAGE_LINK_REGEX.find(content).unwrap().as_str();
        let captures = MESSAGE_LINK_REGEX.captures(msg_link).unwrap();

        let result: Result<Vec<u64>, _> = (1..=3)
            .map(|i| {
                captures
                    .get(i)
                    .ok_or("Failed to get capture group")
                    .and_then(|m| m.as_str().parse::<u64>().map_err(|_| "Failed to parse ID"))
            })
            .collect();

        let ids = match result {
            Ok(ids) => crate::model::DiscordID::builder()
                .guild_id(GuildId::new(ids[0]))
                .channel_id(ChannelId::new(ids[1]))
                .message_id(MessageId::new(ids[2]))
                .build(),
            Err(cause) => {
                tracing::error!(%cause, "Failed to extract IDs from message link.");
                return;
            }
        };

        if ids.guild_id != message.guild_id.unwrap() {
            tracing::warn!("Message link is not from the same guild.");
            return;
        }

        let target_ch = match CHANNEL_LIST_CACHE.get(&ids.channel_id).await {
            Some(ch) => ch,
            None => {
                let ch = match get_channel_by_discord_api(ids.channel_id, ids.guild_id, &ctx).await
                {
                    Ok(ch) => ch,
                    Err(cause) => {
                        tracing::error!(%cause, "Failed to retrieve channel.");
                        return;
                    }
                };
                CHANNEL_LIST_CACHE.insert(ids.channel_id, ch.clone()).await;
                ch
            }
        };

        if target_ch.is_nsfw() {
            return;
        }

        let result = target_ch.message(&ctx.http.clone(), ids.message_id).await;
        let target_msg = match result {
            Ok(msg) => msg,
            Err(cause) => {
                tracing::error!(%cause, "Failed to retrieve message.");
                return;
            }
        };

        if target_msg.kind != MessageType::Regular && target_msg.kind != MessageType::InlineReply {
            return;
        }

        if !target_msg.embeds.is_empty() && target_msg.content.is_empty() {
            return;
        }

        let author = crate::model::CitationMessageAuthor::builder()
            .name(target_msg.author.name.clone())
            .icon_url(target_msg.author.avatar_url())
            .build();

        let attachment_image_url: Option<String> = if !target_msg.attachments.is_empty() {
            target_msg
                .attachments
                .first()
                .map(|attachment| attachment.clone().url)
        } else {
            None
        };

        let sticker_url: Option<String> = if !target_msg.sticker_items.is_empty() {
            target_msg
                .sticker_items
                .first()
                // Note: if invalid sticker, return None
                .map(|sticker| sticker.clone().image_url().unwrap())
        } else {
            None
        };

        let citation_msg = crate::model::CitationMessage::builder()
            .content(target_msg.content)
            .author(author)
            .channel_name(target_ch.name)
            .create_at(target_msg.timestamp)
            .attachment_image_url(attachment_image_url)
            .sticker_image_url(sticker_url)
            .build();

        let embed = CreateEmbed::default()
            .description(citation_msg.content)
            .color(0xb586f7)
            .timestamp(citation_msg.create_at)
            .footer(CreateEmbedFooter::new(citation_msg.channel_name))
            .author(
                CreateEmbedAuthor::new(citation_msg.author.name)
                    .icon_url(citation_msg.author.icon_url.unwrap_or_default()),
            )
            .image(citation_msg.attachment_image_url.unwrap_or_default())
            .thumbnail(citation_msg.sticker_image_url.unwrap_or_default());

        let mention = CreateAllowedMentions::default().replied_user(true);
        let reply_msg = CreateMessage::default()
            .embed(embed)
            .reference_message(&message.clone())
            .allowed_mentions(mention);

        let result = message
            .channel_id
            .clone()
            .send_message(&ctx.http, reply_msg)
            .await;
        if let Err(cause) = result {
            tracing::error!(%cause, "Failed to send citation message.");
        }
    }

    #[tracing::instrument(skip_all)]
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("Starting...");

        let version = env!("CARGO_PKG_VERSION");
        ctx.set_activity(Some(ActivityData::playing(format!("v{version}"))));

        tracing::info!(?version, "Running Babyrite");
        tracing::info!(username = %ready.user.name, user_id = %ready.user.id, "Connected to Discord API.");

        // TODO: add create guild command logic.
    }
}

pub async fn get_channel_by_discord_api(
    channel_id: ChannelId,
    guild_id: GuildId,
    http: &Context,
) -> anyhow::Result<GuildChannel> {
    let guild_channels = guild_id
        .channels(&http)
        .await
        .context("Failed to retrieve channel list.")?;

    let channel = match guild_channels.get(&channel_id) {
        Some(channel) => channel.clone(),
        None => {
            let guild_threads = guild_id.get_active_threads(http).await?;
            let thread = match guild_threads.threads.iter().find(|c| c.id == channel_id) {
                Some(channel) => channel.clone(),
                None => {
                    anyhow::bail!("Channel not found.");
                }
            };
            thread
        }
    };
    Ok(channel)
}
