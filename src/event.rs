//! Event handling module for Discord events.
//!
//! This module implements the serenity [`EventHandler`] trait to handle
//! Discord gateway events such as ready and message events.

use crate::cache::CacheArgs;
use crate::config::BabyriteConfig;
use crate::expand::ExpandedContent;
use crate::expand::discord::MessageLinkIDs;
use crate::expand::github::GitHubPermalink;
use serenity::all::{ActivityData, Context, EventHandler, Message, Ready};
use serenity::prelude::TypeMapKey;
use serenity_builder::model::message::{SerenityMessage, SerenityMessageMentionType};
use tracing::Instrument;

/// TypeMap key for the shared reqwest HTTP client.
pub struct HttpClient;

impl TypeMapKey for HttpClient {
    type Value = reqwest::Client;
}

/// Event handler for Babyrite bot.
pub struct BabyriteEventHandler;

#[serenity::async_trait]
impl EventHandler for BabyriteEventHandler {
    async fn ready(&self, ctx: Context, bot: Ready) {
        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        ctx.set_activity(ActivityData::custom(format!("Running {}", version)).into());
        tracing::info!("Running {}, {} is connected!", version, bot.user.name);
    }

    async fn message(&self, ctx: Context, request: Message) {
        if request.author.bot {
            return;
        }

        let request_guild_id = match request.guild_id {
            Some(id) => id,
            None => return,
        };

        // Correlation span: every log emitted while handling this message
        // carries these fields, so a single request can be traced end-to-end
        // (e.g. via Grafana Loki). `request.id` is the unique Discord message
        // ID and serves as the correlation key.
        let span = tracing::info_span!(
            "message",
            message_id = %request.id,
            guild_id = %request_guild_id,
            channel_id = %request.channel_id,
            author = %request.author.name,
        );

        async {
            let text = &request.content;
            let config = BabyriteConfig::get();
            let mut results = Vec::new();

            // Discord link expansion
            let discord_links = MessageLinkIDs::parse_all(text);
            if !discord_links.is_empty() {
                tracing::debug!(count = discord_links.len(), "parsed Discord links");
                // Resolve the source channel once. The expanded preview is posted here,
                // so it is needed to verify the link target is at least as visible as
                // this channel. If it cannot be resolved, skip Discord expansion (but
                // still allow GitHub expansion below).
                match (CacheArgs {
                    guild_id: request_guild_id,
                    channel_id: request.channel_id,
                })
                .get(&ctx)
                .await
                {
                    Ok(source_channel) => {
                        for ids in discord_links {
                            if ids.guild_id != request_guild_id {
                                tracing::debug!(
                                    link_guild_id = %ids.guild_id,
                                    "skipping cross-guild Discord link"
                                );
                                continue;
                            }

                            match ids.fetch(&ctx, &source_channel).await {
                                Ok(content) => results.push(content),
                                Err(e) => {
                                    tracing::error!(error = %e, "failed to expand Discord link")
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "failed to resolve source channel");
                    }
                }
            }

            // GitHub Permalink expansion (can be disabled via config)
            if config.features.github_permalink {
                let permalinks = GitHubPermalink::parse_all(text);
                if !permalinks.is_empty() {
                    tracing::debug!(count = permalinks.len(), "parsed GitHub permalinks");
                    let data = ctx.data.read().await;
                    if let Some(http_client) = data.get::<HttpClient>() {
                        for permalink in permalinks {
                            match permalink.fetch(http_client).await {
                                Ok(content) => results.push(content),
                                Err(e) => {
                                    tracing::error!(error = %e, "failed to expand GitHub permalink")
                                }
                            }
                        }
                    } else {
                        tracing::error!("HTTP client not found in TypeMap");
                    }
                }
            }

            if results.is_empty() {
                tracing::debug!("no expandable content found");
                return;
            }

            send_expanded_contents(&ctx, &request, results).await;
        }
        .instrument(span)
        .await;
    }
}

/// Sends expanded contents as a reply to the original message.
async fn send_expanded_contents(ctx: &Context, request: &Message, results: Vec<ExpandedContent>) {
    let mut embeds = Vec::new();
    let mut code_blocks = Vec::new();

    for result in results {
        match result {
            ExpandedContent::Embed(embed) => embeds.push(*embed),
            ExpandedContent::CodeBlock {
                language,
                code,
                metadata,
            } => {
                code_blocks.push(format!("{metadata}\n```{language}\n{code}\n```"));
            }
        }
    }

    let embed_count = embeds.len();
    let code_block_count = code_blocks.len();

    // Send embeds if any
    if !embeds.is_empty() {
        let message_builder = SerenityMessage::builder()
            .embeds(embeds)
            .mention_type(SerenityMessageMentionType::Reply(Box::new(request.clone())))
            .build();

        let converted_message = match message_builder.convert() {
            Ok(m) => m,
            Err(e) => {
                tracing::error!(error = ?e, "failed to convert embed message");
                return;
            }
        };

        if let Err(e) = request
            .channel_id
            .send_message(&ctx.http, converted_message)
            .await
        {
            tracing::error!(error = ?e, "failed to send preview");
            return;
        }
    }

    // Send code blocks as plain messages
    for block in code_blocks {
        if let Err(e) = request.channel_id.say(&ctx.http, &block).await {
            tracing::error!(error = ?e, "failed to send code block");
        }
    }

    tracing::info!(
        embeds = embed_count,
        code_blocks = code_block_count,
        "preview sent"
    );
}
