//! Link expansion module.
//!
//! This module provides common types for expanding various types of links
//! (Discord message links, GitHub permalinks, etc.) into rich preview content.

pub mod discord;
pub mod github;

use serenity_builder::model::embed::SerenityEmbed;

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
