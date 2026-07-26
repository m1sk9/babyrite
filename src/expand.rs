//! Link expansion module.
//!
//! This module provides common types for expanding various types of links
//! (Discord message links, GitHub permalinks, etc.) into rich preview content.

pub mod discord;
pub mod github;

use crate::config::BabyriteConfig;
use regex::Regex;
use serenity::all::{Context, GuildId, Message};
use serenity_builder::model::embed::SerenityEmbed;
use std::collections::HashSet;

/// Shared inputs for expanding the links of one message.
pub struct ExpandContext<'a> {
    /// The serenity context.
    pub ctx: &'a Context,
    /// The message whose links are being expanded.
    pub message: &'a Message,
    /// The guild the message was sent in.
    pub guild_id: GuildId,
}

/// A link expander: parses its own link type out of a message and expands
/// each link into [`ExpandedContent`].
///
/// Adding a new link type means implementing this trait and registering the
/// expander in [`EXPANDERS`]; the event handler needs no changes.
#[serenity::async_trait]
pub trait LinkExpander: Send + Sync {
    /// Whether this expander is enabled under `config`.
    fn enabled(&self, config: &BabyriteConfig) -> bool;

    /// Parses and expands all links of this expander's type in the message.
    ///
    /// Failures are logged per link and never abort the other links or
    /// expanders, so this returns only the successful expansions.
    async fn expand_all(&self, cx: &ExpandContext<'_>) -> Vec<ExpandedContent>;
}

/// All registered link expanders, in the order their results appear in a reply.
pub static EXPANDERS: &[&dyn LinkExpander] = &[&discord::DiscordExpander, &github::GitHubExpander];

/// Maximum number of links expanded per message.
const MAX_LINKS_PER_MESSAGE: usize = 3;

/// Extracts links matching `regex` from `text`, applying the shared link policy:
/// URLs wrapped in angle brackets (e.g. `<https://...>`) are skipped, duplicate
/// URLs are ignored, and at most [`MAX_LINKS_PER_MESSAGE`] links are returned.
///
/// `parse` converts a regex match into the expander-specific link type;
/// returning `None` drops that match.
pub(crate) fn parse_links<T>(
    text: &str,
    regex: &Regex,
    parse: impl Fn(&regex::Captures) -> Option<T>,
) -> Vec<T> {
    let mut seen_urls = HashSet::new();
    regex
        .captures_iter(text)
        .filter_map(|captures| {
            let m = captures.get(0)?;
            if m.start() > 0 && text.as_bytes()[m.start() - 1] == b'<' {
                return None;
            }
            if !seen_urls.insert(m.as_str()) {
                return None;
            }
            parse(&captures)
        })
        .take(MAX_LINKS_PER_MESSAGE)
        .collect()
}

/// Expanded content produced by a link expander.
///
/// Represents the different kinds of content that can result from
/// expanding a link.
pub enum ExpandedContent {
    /// A Discord message preview displayed as an embed.
    Embed(Box<SerenityEmbed>),
    /// A code block with syntax highlighting (e.g. GitHub permalink).
    CodeBlock {
        /// The programming language for syntax highlighting.
        language: String,
        /// The code content.
        code: String,
        /// Metadata line displayed above the code block (e.g. file path, line range).
        metadata: String,
    },
}

/// Errors that can occur during link expansion.
#[derive(thiserror::Error, Debug)]
pub enum ExpandError {
    /// An error from the Discord message link expander.
    #[error(transparent)]
    Discord(#[from] discord::PreviewError),
    /// An error from the GitHub permalink expander.
    #[error(transparent)]
    GitHub(#[from] github::GitHubExpandError),
}
