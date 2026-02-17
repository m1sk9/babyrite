//! Link expansion module.
//!
//! This module provides common types for expanding various types of links
//! (Discord message links, GitHub permalinks, etc.) into rich preview content.

pub mod discord;

use serenity_builder::model::embed::SerenityEmbed;

/// Expanded content produced by a link expander.
///
/// Represents the different kinds of content that can result from
/// expanding a link.
pub enum ExpandedContent {
    /// A Discord message preview displayed as an embed.
    Embed(Box<SerenityEmbed>),
}

/// Errors that can occur during link expansion.
#[derive(thiserror::Error, Debug)]
pub enum ExpandError {
    /// An error from the Discord message link expander.
    #[error(transparent)]
    Discord(#[from] discord::PreviewError),
}
