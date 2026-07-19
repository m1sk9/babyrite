//! Mention-prefixed lightweight command system.
//!
//! Slash commands and text commands both carry more ceremony (registration,
//! parsing conventions) than this bot needs. Instead, a message that *starts*
//! with a mention of the bot itself is treated as a command. Requiring the
//! mention to be the first thing in the message is what keeps an incidental
//! `@babyrite` buried inside a normal message from being misread as a command.

use crate::config::{BabyriteConfig, LogFormat};
use serenity::all::{
    Context, CreateMessage, EditMessage, Message, MessageFlags, ShardManager, UserId,
};
use serenity::prelude::TypeMapKey;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// TypeMap key for the shared shard manager.
///
/// Registered alongside [`crate::event::HttpClient`] so the `ping` command
/// can read the gateway heartbeat latency for the shard handling this event.
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

/// A recognized mention-prefixed command.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Show the running version, with a link to its GitHub release.
    Version,
    /// Show the current gateway and API latency.
    Ping,
    /// List the available commands.
    Help,
    /// Show the currently loaded configuration.
    Config,
    /// Echo a message back as a code block: the message this is a reply to,
    /// or (if it isn't a reply) the text after the first newline.
    Debug {
        /// The text after the first newline, used when this isn't a reply.
        payload: String,
    },
    /// A mention-prefixed message that didn't match any known command word.
    Unknown(String),
}

/// Parses a message into a [`Command`] if it starts with a mention of `bot_id`.
///
/// Returns `None` when the message doesn't start with the bot's own mention
/// (leading whitespace aside) — this is the threshold that keeps a mention
/// appearing mid-message from being treated as a command.
pub fn parse(content: &str, bot_id: UserId) -> Option<Command> {
    let rest = strip_bot_mention_prefix(content, bot_id)?.trim_start();
    if rest.is_empty() {
        // A bare mention with no command word is treated as a request for help.
        return Some(Command::Help);
    }

    let head_end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    let head = rest[..head_end].to_ascii_lowercase();

    match head.as_str() {
        "version" => Some(Command::Version),
        "ping" => Some(Command::Ping),
        "help" => Some(Command::Help),
        "config" => Some(Command::Config),
        "debug" => {
            // The payload is everything after the first newline, not just
            // after the "debug" word, so trailing text on the command line
            // itself (e.g. a stray space) isn't mistaken for the payload.
            let payload = rest.split_once('\n').map_or("", |(_, body)| body);
            Some(Command::Debug {
                payload: payload.to_string(),
            })
        }
        _ => Some(Command::Unknown(head)),
    }
}

/// Strips a leading `<@ID>` or `<@!ID>` mention of `bot_id` from `content`.
///
/// Returns `None` if the content (after leading whitespace) doesn't start
/// with either mention form.
fn strip_bot_mention_prefix(content: &str, bot_id: UserId) -> Option<&str> {
    let trimmed = content.trim_start();
    let nickname_mention = format!("<@!{bot_id}>");
    let plain_mention = format!("<@{bot_id}>");
    trimmed
        .strip_prefix(nickname_mention.as_str())
        .or_else(|| trimmed.strip_prefix(plain_mention.as_str()))
}

/// Builds the URL for a tagged GitHub release.
pub fn release_url(repository: &str, version: &str) -> String {
    format!("{repository}/releases/tag/v{version}")
}

/// Replaces code fences in `s` so they can't break out of a code block they're embedded in.
fn sanitize_code_block(s: &str) -> String {
    s.replace("```", "'''")
}

/// Executes a parsed command, replying on the channel the request came from.
pub async fn execute(ctx: &Context, request: &Message, command: Command) {
    match command {
        // Measures its own reply's round-trip and then edits the latency in,
        // so it needs the sent `Message` handle rather than a fixed string.
        Command::Ping => execute_ping(ctx, request).await,
        // Needs `request` itself to see whether it's a reply to another
        // message, which a plain string return from `render` can't carry.
        Command::Debug { payload } => {
            let content = render_debug(request, &payload);
            send_reply(ctx, request, &content).await;
        }
        other => {
            let content = render(other);
            send_reply(ctx, request, &content).await;
        }
    }
}

/// Renders the plain-text reply body for a command that only needs its own data.
fn render(command: Command) -> String {
    match command {
        Command::Version => render_version(),
        Command::Help => render_help(),
        Command::Config => render_config(BabyriteConfig::get()),
        Command::Unknown(word) => render_unknown(&word),
        Command::Ping | Command::Debug { .. } => {
            unreachable!("ping and debug are handled in execute() before render() is called")
        }
    }
}

fn render_version() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let url = release_url(env!("CARGO_PKG_REPOSITORY"), version);
    format!("Running babyrite v{version}\n{url}")
}

fn render_help() -> String {
    "Available commands:\n\
     `version` — Show the running version.\n\
     `ping` — Show the current latency.\n\
     `help` — Show this help message.\n\
     `config` — Show the currently loaded configuration.\n\
     `debug` — Echo the following lines back as a code block."
        .to_string()
}

fn render_config(config: &BabyriteConfig) -> String {
    let log_format = match config.resolved_log_format() {
        LogFormat::Compact => "compact",
        LogFormat::Json => "json",
    };

    format!(
        "Current configuration:\n\
         log.level: `{}`\n\
         log.format: `{log_format}`\n\
         features.github_permalink: `{}`\n\
         features.commands: `{}`\n\
         github.max_lines: `{}`",
        config.log.level,
        config.features.github_permalink,
        config.features.commands,
        config.github.max_lines,
    )
}

/// Discord rejects message content longer than this many characters.
const MAX_MESSAGE_CONTENT_CHARS: usize = 2000;

/// Appended to a debug payload that had to be cut to fit `MAX_MESSAGE_CONTENT_CHARS`.
const TRUNCATION_NOTICE: &str = "\n… (truncated)";

/// Shown when `debug` has nothing to display.
const DEBUG_MISSING_SOURCE_ERROR: &str = "Error: `debug` needs something to show — reply to the message you want debugged, \
     or put text on the line(s) after the command.";

/// Renders the `debug` reply body.
///
/// When the command message is itself a reply, the referenced message's
/// content is what the user most likely wants echoed back, so it takes
/// priority over a typed `payload`. Falls back to `payload`, and finally to
/// an error if neither has anything to show.
fn render_debug(request: &Message, payload: &str) -> String {
    let referenced_content = request
        .referenced_message
        .as_deref()
        .map(|referenced| referenced.content.as_str())
        .filter(|content| !content.trim().is_empty());

    let Some(source) =
        referenced_content.or_else(|| Some(payload).filter(|p| !p.trim().is_empty()))
    else {
        return DEBUG_MISSING_SOURCE_ERROR.to_string();
    };

    let sanitized = sanitize_code_block(source);
    // The "```\n" / "\n```" fence around the payload is 8 characters of fixed overhead.
    let budget = MAX_MESSAGE_CONTENT_CHARS.saturating_sub(8);

    if sanitized.chars().count() <= budget {
        return format!("```\n{sanitized}\n```");
    }

    // `debug`'s entire purpose is pasting arbitrary text (log excerpts, stack
    // traces), which realistically exceeds Discord's message length limit.
    // Truncating and saying so beats send_reply's error path silently
    // dropping the whole reply.
    let keep = budget.saturating_sub(TRUNCATION_NOTICE.chars().count());
    let truncated: String = sanitized.chars().take(keep).collect();
    format!("```\n{truncated}{TRUNCATION_NOTICE}\n```")
}

fn render_unknown(word: &str) -> String {
    format!("Unknown command: `{word}`. Try `help`.")
}

/// Sends `content` as a reply to `request`, suppressing link-preview embeds.
///
/// Returns the sent message so callers (namely `ping`) can edit it afterwards.
/// Returns `None` (after logging) if the send fails.
async fn send_reply(ctx: &Context, request: &Message, content: &str) -> Option<Message> {
    let message = CreateMessage::new()
        .content(content)
        .reference_message(request)
        .flags(MessageFlags::SUPPRESS_EMBEDS);

    match request.channel_id.send_message(&ctx.http, message).await {
        Ok(sent) => Some(sent),
        Err(e) => {
            tracing::error!(error = ?e, "failed to send command reply");
            None
        }
    }
}

async fn execute_ping(ctx: &Context, request: &Message) {
    let gateway_latency = current_gateway_latency(ctx).await;
    let gateway_text = format_latency(gateway_latency);

    let start = Instant::now();
    let Some(mut sent) = send_reply(
        ctx,
        request,
        &format!("🏓 Pong!\nGateway latency: {gateway_text}\nAPI latency: measuring…"),
    )
    .await
    else {
        return;
    };
    let api_latency = start.elapsed();

    if let Err(e) = sent
        .edit(
            ctx,
            EditMessage::new().content(format!(
                "🏓 Pong!\nGateway latency: {gateway_text}\nAPI latency: {}ms",
                api_latency.as_millis()
            )),
        )
        .await
    {
        tracing::error!(error = ?e, "failed to update ping latency");
    }
}

fn format_latency(latency: Option<Duration>) -> String {
    match latency {
        // No heartbeat ACK has been received yet (e.g. right after connecting).
        Some(d) => format!("{}ms", d.as_millis()),
        None => "measuring…".to_string(),
    }
}

/// Reads the gateway heartbeat latency for the shard handling this context.
///
/// `None` if the shard manager isn't registered in `ctx.data`, or if no
/// heartbeat acknowledgement has been received yet for this shard.
async fn current_gateway_latency(ctx: &Context) -> Option<Duration> {
    let data = ctx.data.read().await;
    let manager = data.get::<ShardManagerContainer>()?;
    let runners = manager.runners.lock().await;
    runners.get(&ctx.shard_id)?.latency
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bot_id() -> UserId {
        UserId::new(123)
    }

    #[test]
    fn parses_plain_mention_prefix() {
        assert_eq!(parse("<@123> ping", bot_id()), Some(Command::Ping));
    }

    #[test]
    fn parses_nickname_mention_prefix() {
        assert_eq!(parse("<@!123> ping", bot_id()), Some(Command::Ping));
    }

    #[test]
    fn ignores_mid_message_mention() {
        // The mention isn't at the start, so this must not be read as a command.
        assert_eq!(parse("hello <@123> ping", bot_id()), None);
    }

    #[test]
    fn ignores_mention_of_a_different_user() {
        assert_eq!(parse("<@999> ping", bot_id()), None);
    }

    #[test]
    fn bare_mention_shows_help() {
        assert_eq!(parse("<@123>", bot_id()), Some(Command::Help));
        assert_eq!(parse("<@123>   ", bot_id()), Some(Command::Help));
    }

    #[test]
    fn leading_whitespace_before_mention_is_allowed() {
        assert_eq!(parse("   <@123> ping", bot_id()), Some(Command::Ping));
    }

    #[test]
    fn command_word_is_case_insensitive() {
        assert_eq!(parse("<@123> PING", bot_id()), Some(Command::Ping));
    }

    #[test]
    fn parses_version_help_and_config() {
        assert_eq!(parse("<@123> version", bot_id()), Some(Command::Version));
        assert_eq!(parse("<@123> help", bot_id()), Some(Command::Help));
        assert_eq!(parse("<@123> config", bot_id()), Some(Command::Config));
    }

    #[test]
    fn debug_payload_is_the_text_after_the_first_newline() {
        assert_eq!(
            parse("<@123> debug\nsome body", bot_id()),
            Some(Command::Debug {
                payload: "some body".to_string()
            })
        );
    }

    #[test]
    fn debug_without_a_newline_has_no_payload() {
        assert_eq!(
            parse("<@123> debug", bot_id()),
            Some(Command::Debug {
                payload: String::new()
            })
        );
    }

    #[test]
    fn unknown_command_word_is_preserved() {
        assert_eq!(
            parse("<@123> foobar", bot_id()),
            Some(Command::Unknown("foobar".to_string()))
        );
    }

    #[test]
    fn render_version_puts_the_release_url_on_its_own_line() {
        let expected = format!(
            "Running babyrite v{}\n{}/releases/tag/v{}",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_REPOSITORY"),
            env!("CARGO_PKG_VERSION")
        );
        assert_eq!(render_version(), expected);
    }

    #[test]
    fn render_help_lists_commands_without_a_mention_prefix() {
        let help = render_help();
        assert!(help.contains("`version`"));
        assert!(help.contains("`ping`"));
        assert!(help.contains("`help`"));
        assert!(help.contains("`config`"));
        assert!(help.contains("`debug`"));
        assert!(!help.contains("<@"));
    }

    #[test]
    fn render_unknown_hint_has_no_mention_prefix() {
        let text = render_unknown("foobar");
        assert!(text.contains("foobar"));
        assert!(text.contains("`help`"));
        assert!(!text.contains("<@"));
    }

    #[test]
    fn render_config_reports_the_effective_settings() {
        let config = BabyriteConfig::default();
        let rendered = render_config(&config);

        assert!(!rendered.contains("```"));
        assert!(rendered.contains("log.level: `babyrite=info`"));
        assert!(rendered.contains("log.format: `compact`"));
        assert!(rendered.contains("features.github_permalink: `true`"));
        assert!(rendered.contains("features.commands: `true`"));
        assert!(rendered.contains("github.max_lines: `50`"));
    }

    fn message_without_reference() -> Message {
        Message::default()
    }

    // `Message` is `#[non_exhaustive]`, so it can't be built with struct-literal
    // syntax here (even with `..Default::default()`) — only field assignment
    // on an already-constructed instance is allowed outside its crate.
    fn message_replying_to(content: &str) -> Message {
        let mut referenced = Message::default();
        referenced.content = content.to_string();

        let mut request = Message::default();
        request.referenced_message = Some(Box::new(referenced));
        request
    }

    #[test]
    fn render_debug_errors_when_theres_no_reply_and_no_payload() {
        let request = message_without_reference();
        assert!(render_debug(&request, "").starts_with("Error:"));
        assert!(render_debug(&request, "   ").starts_with("Error:"));
    }

    #[test]
    fn render_debug_wraps_payload_in_a_sanitized_code_block() {
        let request = message_without_reference();
        assert_eq!(
            render_debug(&request, "```rust\ncode\n```"),
            "```\n'''rust\ncode\n'''\n```"
        );
    }

    #[test]
    fn render_debug_truncates_oversized_payloads_to_fit_discords_limit() {
        let request = message_without_reference();
        let payload = "a".repeat(3000);
        let rendered = render_debug(&request, &payload);

        assert!(rendered.chars().count() <= MAX_MESSAGE_CONTENT_CHARS);
        assert!(rendered.contains(TRUNCATION_NOTICE.trim()));
    }

    #[test]
    fn render_debug_shows_the_replied_to_message_over_a_typed_payload() {
        let request = message_replying_to("target content");
        assert_eq!(
            render_debug(&request, "ignored payload"),
            "```\ntarget content\n```"
        );
    }

    #[test]
    fn render_debug_falls_back_to_payload_when_the_reply_has_no_content() {
        let request = message_replying_to("");
        assert_eq!(
            render_debug(&request, "typed payload"),
            "```\ntyped payload\n```"
        );
    }

    #[test]
    fn sanitize_code_block_replaces_all_fences() {
        assert_eq!(
            sanitize_code_block("```rust\ncode\n```"),
            "'''rust\ncode\n'''"
        );
    }

    #[test]
    fn release_url_points_to_the_tagged_release() {
        assert_eq!(
            release_url("https://github.com/m1sk9/babyrite", "1.2.5"),
            "https://github.com/m1sk9/babyrite/releases/tag/v1.2.5"
        );
    }

    #[test]
    fn format_latency_reports_measuring_when_unknown() {
        assert_eq!(format_latency(None), "measuring…");
        assert_eq!(format_latency(Some(Duration::from_millis(42))), "42ms");
    }
}
