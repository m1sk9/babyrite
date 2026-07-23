//! Event handling module for Discord events.
//!
//! This module implements the serenity [`EventHandler`] trait to handle
//! Discord gateway events such as ready and message events.

use crate::cache::CacheArgs;
use crate::config::BabyriteConfig;
use crate::expand::ExpandedContent;
use crate::expand::discord::MessageLinkIDs;
use crate::expand::github::GitHubPermalink;
use crate::reaction;
use serenity::all::{
    ActivityData, Context, CreateAllowedMentions, CreateMessage, EventHandler, Message,
    Permissions, Reaction, ReactionType, Ready,
};
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

            // Mention-prefixed commands (e.g. `@babyrite ping`) take priority
            // over link expansion below. A message starting with the bot's
            // mention followed by an unrecognized word isn't necessarily a
            // command attempt, though — e.g. "@babyrite check this out:
            // <link>" — so an unrecognized command only gets its "Unknown
            // command" hint if the message has no expandable links either;
            // otherwise the links are expanded as normal.
            let mut unknown_command = None;
            if config.features.commands {
                let bot_id = ctx.cache.current_user().id;
                match crate::command::parse(text, bot_id) {
                    Some(crate::command::Command::Unknown(word)) => unknown_command = Some(word),
                    Some(command) => {
                        tracing::debug!(?command, "handling mention command");
                        crate::command::execute(&ctx, &request, command).await;
                        return;
                    }
                    None => {}
                }
            }

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
                if let Some(word) = unknown_command {
                    let command = crate::command::Command::Unknown(word);
                    tracing::debug!(?command, "handling mention command");
                    crate::command::execute(&ctx, &request, command).await;
                } else {
                    tracing::debug!("no expandable content found");
                }
                return;
            }

            send_expanded_contents(&ctx, &request, results).await;
        }
        .instrument(span)
        .await;
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let config = BabyriteConfig::get();
        if !config.features.reactions {
            return;
        }

        let bot_id = ctx.cache.current_user().id;

        // The bot's own delete reaction (attached right after sending a preview,
        // see `attach_delete_reaction`) would otherwise re-trigger this handler.
        if reaction.user_id == Some(bot_id) {
            return;
        }

        // Unrecognized emoji are ignored outright — this is the entire
        // enforcement of "every reaction except the recognized ones is ignored".
        let Some(action) = reaction::from_emoji(&reaction.emoji) else {
            return;
        };

        // Only previews the bot itself sent are valid action targets. Checking
        // via the gateway payload's `message_author_id` (present on ReactionAdd)
        // avoids fetching the message for the common case of a reaction on
        // someone else's message.
        if reaction.message_author_id != Some(bot_id) {
            return;
        }

        let (Some(guild_id), Some(reactor_id)) = (reaction.guild_id, reaction.user_id) else {
            return;
        };

        let span = tracing::info_span!(
            "reaction_add",
            guild_id = %guild_id,
            channel_id = %reaction.channel_id,
            message_id = %reaction.message_id,
            reactor = %reactor_id,
        );

        async {
            let preview = match reaction.message(&ctx.http).await {
                Ok(m) => m,
                Err(e) => {
                    tracing::error!(error = ?e, "failed to fetch reacted-to preview");
                    return;
                }
            };

            // Previews are sent as replies to their request (see
            // `send_expanded_contents`), so the requester is read back from
            // `referenced_message` rather than kept in separate state — this
            // is what lets a preview stay actionable across a bot restart.
            let requester = preview.referenced_message.as_deref().map(|m| m.author.id);

            // `member` is only needed for the MANAGE_MESSAGES check below, not
            // for the requester-self-delete path — so its absence must not
            // block a requester from deleting their own preview.
            let can_manage_messages = match reaction.member.as_ref() {
                Some(member) => match (CacheArgs {
                    guild_id,
                    channel_id: reaction.channel_id,
                })
                .get(&ctx)
                .await
                {
                    Ok(channel) => ctx.cache.guild(guild_id).is_some_and(|guild| {
                        guild
                            .user_permissions_in(&channel, member)
                            .contains(Permissions::MANAGE_MESSAGES)
                    }),
                    Err(e) => {
                        tracing::error!(error = %e, "failed to resolve channel for permission check");
                        false
                    }
                },
                None => false,
            };

            if !reaction::is_authorized(requester, reactor_id, can_manage_messages) {
                tracing::debug!("reactor is not authorized to trigger this action");
                return;
            }

            reaction::execute(&ctx, &preview, action).await;
        }
        .instrument(span)
        .await;
    }
}

/// Sends expanded contents as a reply to the original message.
async fn send_expanded_contents(ctx: &Context, request: &Message, results: Vec<ExpandedContent>) {
    let config = BabyriteConfig::get();
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

        match request
            .channel_id
            .send_message(&ctx.http, converted_message)
            .await
        {
            Ok(sent) => attach_delete_reaction(ctx, &sent, config).await,
            Err(e) => {
                tracing::error!(error = ?e, "failed to send preview");
                return;
            }
        }
    }

    // Code blocks are sent as replies rather than plain messages (as `.say`
    // would) so that, like the embed path above, a reaction on the sent
    // preview can look up its `referenced_message` to find who requested it
    // — see `EventHandler::reaction_add`. `allowed_mentions` is left empty so
    // this doesn't newly ping the requester on every reply.
    for block in code_blocks {
        let message = CreateMessage::new()
            .content(&block)
            .reference_message(request)
            .allowed_mentions(CreateAllowedMentions::new());

        match request.channel_id.send_message(&ctx.http, message).await {
            Ok(sent) => attach_delete_reaction(ctx, &sent, config).await,
            Err(e) => tracing::error!(error = ?e, "failed to send code block"),
        }
    }

    tracing::info!(
        embeds = embed_count,
        code_blocks = code_block_count,
        "preview sent"
    );
}

/// Attaches the delete-preview reaction to a freshly-sent preview message.
///
/// A no-op when the reactions feature is disabled, so a disabled feature
/// doesn't leave stray reactions that then do nothing when pressed.
async fn attach_delete_reaction(ctx: &Context, preview: &Message, config: &BabyriteConfig) {
    if !config.features.reactions {
        return;
    }

    if let Err(e) = preview
        .react(
            &ctx.http,
            ReactionType::Unicode(reaction::DELETE_PREVIEW_EMOJI.to_string()),
        )
        .await
    {
        tracing::error!(error = ?e, "failed to attach delete reaction to preview");
    }
}
