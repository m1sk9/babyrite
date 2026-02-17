//! GitHub Permalink expansion.
//!
//! This module provides functionality for parsing GitHub permalink URLs
//! and fetching raw file content to display as code blocks.

use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

use super::{ExpandError, ExpandedContent};
use crate::config::BabyriteConfig;
use crate::utils::language_from_extension;

/// Regex pattern for matching GitHub permalink URLs.
///
/// Captures: owner, repo, commit, path, and optional line range fragment.
///
/// Supported patterns:
/// - `https://github.com/{owner}/{repo}/blob/{commit}/{path}`
/// - `https://github.com/{owner}/{repo}/blob/{commit}/{path}#L{line}`
/// - `https://github.com/{owner}/{repo}/blob/{commit}/{path}#L{start}-L{end}`
static GITHUB_PERMALINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"https://github\.com/([^/]+)/([^/]+)/blob/([0-9a-f]{4,40})/([^#\s]+)(?:#L(\d+)(?:-L(\d+))?)?"
    )
    .unwrap()
});

/// A parsed GitHub permalink.
#[derive(Debug)]
pub struct GitHubPermalink {
    /// Repository owner.
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// Commit SHA.
    pub commit: String,
    /// File path within the repository.
    pub path: String,
    /// Optional line range specification.
    pub line_range: Option<LineRange>,
}

/// A line range extracted from a GitHub permalink fragment.
#[derive(Debug, Clone, Copy)]
pub struct LineRange {
    /// Start line (1-indexed).
    pub start: usize,
    /// End line (1-indexed, inclusive). Same as `start` for single-line references.
    pub end: usize,
}

/// Errors that can occur when expanding a GitHub permalink.
#[derive(thiserror::Error, Debug)]
pub enum GitHubExpandError {
    /// Failed to fetch the raw file content.
    #[error("Failed to fetch raw content: {0}")]
    Fetch(String),
    /// The fetched content exceeds the maximum allowed size.
    #[error("Content exceeds size limit")]
    ContentTooLarge,
    /// An HTTP error occurred.
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

impl GitHubPermalink {
    /// Parses all GitHub permalink URLs from the given text.
    ///
    /// Only matches URLs with a commit SHA (not branch names), ensuring
    /// only true permalinks are expanded.
    ///
    /// Note: Duplicate URLs are ignored, and a maximum of 3 links are returned.
    pub fn parse_all(text: &str) -> Vec<GitHubPermalink> {
        let mut seen_urls = HashSet::new();
        GITHUB_PERMALINK_REGEX
            .captures_iter(text)
            .filter_map(|captures| {
                let full_url = captures.get(0)?.as_str();
                if !seen_urls.insert(full_url.to_string()) {
                    return None;
                }

                let owner = captures.get(1)?.as_str().to_string();
                let repo = captures.get(2)?.as_str().to_string();
                let commit = captures.get(3)?.as_str().to_string();
                let path = captures.get(4)?.as_str().to_string();

                let line_range = match (captures.get(5), captures.get(6)) {
                    (Some(start), Some(end)) => {
                        let s = start.as_str().parse().ok()?;
                        let e = end.as_str().parse().ok()?;
                        Some(LineRange { start: s, end: e })
                    }
                    (Some(start), None) => {
                        let s = start.as_str().parse().ok()?;
                        Some(LineRange { start: s, end: s })
                    }
                    _ => None,
                };

                Some(GitHubPermalink {
                    owner,
                    repo,
                    commit,
                    path,
                    line_range,
                })
            })
            .take(3) // Limit to maximum 3 links
            .collect()
    }

    /// Fetches the raw file content from GitHub and returns a code block.
    pub async fn fetch(
        &self,
        http_client: &reqwest::Client,
    ) -> Result<ExpandedContent, ExpandError> {
        let config = BabyriteConfig::get();
        let max_lines = config.github.max_lines;

        let raw_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            self.owner, self.repo, self.commit, self.path
        );

        let response = http_client
            .get(&raw_url)
            .send()
            .await
            .map_err(GitHubExpandError::Http)?;

        if !response.status().is_success() {
            return Err(GitHubExpandError::Fetch(format!(
                "HTTP {} for {}",
                response.status(),
                raw_url
            ))
            .into());
        }

        let content_length = response.content_length().unwrap_or(0);
        // 1 MB limit to avoid fetching huge files
        if content_length > 1_048_576 {
            return Err(GitHubExpandError::ContentTooLarge.into());
        }

        let body = response.text().await.map_err(GitHubExpandError::Http)?;

        let all_lines: Vec<&str> = body.lines().collect();
        let (code, line_info) = match self.line_range {
            Some(range) => {
                let start = range.start.saturating_sub(1); // 0-indexed
                let end = range.end.min(all_lines.len());
                let selected: Vec<&str> = all_lines.get(start..end).unwrap_or_default().to_vec();

                let (code, truncated) = truncate_lines(&selected, max_lines);
                let info = if truncated {
                    format!(
                        "L{}-L{}, truncated to {} lines",
                        range.start, range.end, max_lines
                    )
                } else {
                    format!("L{}-L{}", range.start, range.end)
                };
                (code, info)
            }
            None => {
                let (code, truncated) = truncate_lines(&all_lines, max_lines);
                let info = if truncated {
                    format!("truncated to {} lines", max_lines)
                } else {
                    String::new()
                };
                (code, info)
            }
        };

        let short_commit = &self.commit[..7.min(self.commit.len())];
        let language = self
            .path
            .rsplit('.')
            .next()
            .map(language_from_extension)
            .unwrap_or("");

        let metadata = if line_info.is_empty() {
            format!(
                "`{}` - {}/{}@{}",
                self.path, self.owner, self.repo, short_commit
            )
        } else {
            format!(
                "`{}` ({}) - {}/{}@{}",
                self.path, line_info, self.owner, self.repo, short_commit
            )
        };

        Ok(ExpandedContent::CodeBlock {
            language: language.to_string(),
            code,
            metadata,
        })
    }
}

/// Truncates lines to the given maximum, returning the joined string and whether truncation occurred.
fn truncate_lines(lines: &[&str], max: usize) -> (String, bool) {
    if lines.len() > max {
        let truncated: Vec<&str> = lines[..max].to_vec();
        (truncated.join("\n"), true)
    } else {
        (lines.join("\n"), false)
    }
}
