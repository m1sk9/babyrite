//! Event handling module for Discord events.
//!
//! This module implements the serenity [`EventHandler`] trait to handle
//! Discord gateway events such as ready and message events.

use crate::config::BabyriteConfig;
use crate::expand::ExpandedContent;
use crate::expand::discord::MessageLinkIDs;
use crate::expand::github::GitHubPermalink;
use serenity::all::{ActivityData, Context, EventHandler, Message, Ready};
use serenity::prelude::TypeMapKey;
use serenity_builder::model::message::{SerenityMessage, SerenityMessageMentionType};

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

        let text = &request.content;
        let config = BabyriteConfig::get();
        let mut results = Vec::new();

        // Discord link expansion
        for ids in MessageLinkIDs::parse_all(text) {
            if ids.guild_id != request_guild_id {
                continue;
            }

            tracing::info!(
                "Begin generating the preview. (Requester: {})",
                &request.author.name
            );

            match ids.fetch(&ctx).await {
                Ok(content) => results.push(content),
                Err(e) => tracing::error!("{}", e),
            }
        }

        // GitHub Permalink expansion (can be disabled via config)
        if config.features.github_permalink {
            let permalinks = GitHubPermalink::parse_all(text);
            if !permalinks.is_empty() {
                let data = ctx.data.read().await;
                if let Some(http_client) = data.get::<HttpClient>() {
                    for permalink in permalinks {
                        tracing::info!(
                            "Begin expanding GitHub permalink. (Requester: {})",
                            &request.author.name
                        );

                        match permalink.fetch(http_client).await {
                            Ok(content) => results.push(content),
                            Err(e) => tracing::error!("{}", e),
                        }
                    }
                } else {
                    tracing::error!("HTTP client not found in TypeMap");
                }
            }
        }

        if results.is_empty() {
            return;
        }

        send_expanded_contents(&ctx, &request, results).await;
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

    // Send embeds if any
    if !embeds.is_empty() {
        let message_builder = SerenityMessage::builder()
            .embeds(embeds)
            .mention_type(SerenityMessageMentionType::Reply(Box::new(request.clone())))
            .build();

        let converted_message = match message_builder.convert() {
            Ok(m) => m,
            Err(e) => {
                tracing::error!(?e);
                return;
            }
        };

        if let Err(e) = request
            .channel_id
            .send_message(&ctx.http, converted_message)
            .await
        {
            tracing::error!("Failed to send preview: {:?}", e);
            return;
        }
    }

    // Send code blocks as plain messages
    for block in code_blocks {
        if let Err(e) = request.channel_id.say(&ctx.http, &block).await {
            tracing::error!("Failed to send code block: {:?}", e);
        }
    }

    tracing::info!("Preview sent successfully.");
}
