//! GitHub Permalink expansion.
//!
//! This module provides functionality for parsing GitHub permalink URLs
//! and fetching raw file content to display as code blocks.

use regex::Regex;
use serenity::futures::future::join_all;
use serenity::prelude::TypeMapKey;
use std::sync::LazyLock;

use super::{ExpandContext, ExpandError, ExpandedContent, LinkExpander};
use crate::config::BabyriteConfig;
use crate::utils::language_for_path;

/// TypeMap key for the shared reqwest HTTP client used to fetch raw content.
pub struct HttpClient;

impl TypeMapKey for HttpClient {
    type Value = reqwest::Client;
}

/// Regex pattern for matching GitHub blob URLs.
///
/// Captures: owner, repo, git_ref (commit SHA or branch name), path, and optional line range fragment.
///
/// Supported patterns:
/// - `https://github.com/{owner}/{repo}/blob/{ref}/{path}`
/// - `https://github.com/{owner}/{repo}/blob/{ref}/{path}#L{line}`
/// - `https://github.com/{owner}/{repo}/blob/{ref}/{path}#L{start}-L{end}`
///
/// The `{ref}` can be a commit SHA (e.g., `abcdef1234567`) or a branch/tag name (e.g., `main`, `feature/foo`).
///
/// An optional query string (e.g., `?plain=1`) is consumed but discarded — GitHub's blob query
/// parameters control browser rendering only and do not affect the raw content served by
/// `raw.githubusercontent.com`.
static GITHUB_PERMALINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"https://github\.com/([^/]+)/([^/]+)/blob/([^/]+)/([^#\s?]+)(?:\?[^#\s]*)?(?:#L(\d+)(?:-L(\d+))?)?",
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
    /// Git ref (commit SHA or branch/tag name).
    pub git_ref: String,
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

/// GitHub permalink expander.
pub struct GitHubExpander;

#[serenity::async_trait]
impl LinkExpander for GitHubExpander {
    fn enabled(&self, config: &BabyriteConfig) -> bool {
        config.features.github_permalink
    }

    /// Expands GitHub permalinks into code blocks.
    #[cfg_attr(coverage_nightly, coverage(off))]
    async fn expand_all(&self, cx: &ExpandContext<'_>) -> Vec<ExpandedContent> {
        let permalinks = GitHubPermalink::parse_all(&cx.message.content);
        if permalinks.is_empty() {
            return Vec::new();
        }
        tracing::debug!(count = permalinks.len(), "parsed GitHub permalinks");

        // `reqwest::Client` is internally reference-counted, so clone it out of
        // the TypeMap instead of holding the read guard across the fetches below.
        let http_client = {
            let data = cx.ctx.data.read().await;
            data.get::<HttpClient>().cloned()
        };
        let Some(http_client) = http_client else {
            tracing::error!("HTTP client not found in TypeMap");
            return Vec::new();
        };

        join_all(permalinks.iter().map(|p| p.fetch(&http_client)))
            .await
            .into_iter()
            .filter_map(|result| match result {
                Ok(content) => Some(content),
                Err(e) => {
                    tracing::error!(error = %e, "failed to expand GitHub permalink");
                    None
                }
            })
            .collect()
    }
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
    /// Matches URLs with commit SHAs, branch names, or tag names.
    /// The shared link policy applies (see [`super::parse_links`]): angle-bracket
    /// wrapped and duplicate URLs are ignored, and at most 3 links are returned.
    pub fn parse_all(text: &str) -> Vec<GitHubPermalink> {
        super::parse_links(text, &GITHUB_PERMALINK_REGEX, |captures| {
            let line_range = match captures.get(5) {
                Some(start) => {
                    let start = start.as_str().parse().ok()?;
                    // A missing end (e.g. `#L42`) means a single-line reference.
                    let end = match captures.get(6) {
                        Some(end) => end.as_str().parse().ok()?,
                        None => start,
                    };
                    Some(LineRange { start, end })
                }
                None => None,
            };

            Some(GitHubPermalink {
                owner: captures.get(1)?.as_str().to_string(),
                repo: captures.get(2)?.as_str().to_string(),
                git_ref: captures.get(3)?.as_str().to_string(),
                path: captures.get(4)?.as_str().to_string(),
                line_range,
            })
        })
    }

    /// Fetches the raw file content from GitHub and returns a code block.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[tracing::instrument(
        skip(self, http_client),
        fields(
            owner = %self.owner,
            repo = %self.repo,
            git_ref = %self.git_ref,
            path = %self.path,
        )
    )]
    pub async fn fetch(
        &self,
        http_client: &reqwest::Client,
    ) -> Result<ExpandedContent, ExpandError> {
        let config = BabyriteConfig::get();
        let max_lines = config.github.max_lines;

        let raw_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            self.owner, self.repo, self.git_ref, self.path
        );

        tracing::debug!(url = %raw_url, "fetching raw content");
        let started = std::time::Instant::now();
        let response = http_client
            .get(&raw_url)
            .send()
            .await
            .map_err(GitHubExpandError::Http)?;

        let content_length = response.content_length().unwrap_or(0);
        tracing::debug!(
            status = %response.status(),
            content_length,
            elapsed_ms = started.elapsed().as_millis(),
            "received response"
        );

        if !response.status().is_success() {
            tracing::warn!(status = %response.status(), "non-success status fetching raw content");
            return Err(GitHubExpandError::Fetch(format!(
                "HTTP {} for {}",
                response.status(),
                raw_url
            ))
            .into());
        }

        // 1 MB limit to avoid fetching huge files
        if content_length > 1_048_576 {
            tracing::warn!(content_length, "content exceeds size limit");
            return Err(GitHubExpandError::ContentTooLarge.into());
        }

        let body = response.text().await.map_err(GitHubExpandError::Http)?;

        let content = self.build_code_block(&body, max_lines);
        tracing::debug!("code block built");
        Ok(content)
    }

    /// Builds an `ExpandedContent::CodeBlock` from raw file content.
    fn build_code_block(&self, body: &str, max_lines: usize) -> ExpandedContent {
        let all_lines: Vec<&str> = body.lines().collect();
        let (code, line_info) = match self.line_range {
            Some(range) => {
                let start = range.start.saturating_sub(1); // 0-indexed
                let end = range.end.min(all_lines.len());
                let selected = all_lines.get(start..end).unwrap_or_default();

                let (code, truncated) = truncate_lines(selected, max_lines);
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

        let display_ref = shorten_ref(&self.git_ref);
        let language = language_for_path(&self.path);

        let line_part = if line_info.is_empty() {
            String::new()
        } else {
            format!(" ({line_info})")
        };
        let metadata = format!(
            "`{}`{} - {}/{}@{}",
            self.path, line_part, self.owner, self.repo, display_ref
        );

        ExpandedContent::CodeBlock {
            language: language.to_string(),
            code,
            metadata,
        }
    }
}

/// Returns true if the given string looks like a commit SHA (4-40 hex characters).
fn is_commit_sha(s: &str) -> bool {
    (4..=40).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Shortens a git ref for display. Commit SHAs are truncated to 7 characters;
/// branch/tag names are returned as-is.
fn shorten_ref(git_ref: &str) -> &str {
    if is_commit_sha(git_ref) {
        &git_ref[..7.min(git_ref.len())]
    } else {
        git_ref
    }
}

/// Truncates lines to the given maximum, returning the joined string and whether truncation occurred.
fn truncate_lines(lines: &[&str], max: usize) -> (String, bool) {
    let kept = &lines[..lines.len().min(max)];
    (kept.join("\n"), lines.len() > max)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- truncate_lines ---

    #[test]
    fn truncate_lines_under_limit() {
        let lines = vec!["a", "b", "c"];
        let (result, truncated) = truncate_lines(&lines, 5);
        assert_eq!(result, "a\nb\nc");
        assert!(!truncated);
    }

    #[test]
    fn truncate_lines_at_limit() {
        let lines = vec!["a", "b", "c"];
        let (result, truncated) = truncate_lines(&lines, 3);
        assert_eq!(result, "a\nb\nc");
        assert!(!truncated);
    }

    #[test]
    fn truncate_lines_over_limit() {
        let lines = vec!["a", "b", "c", "d", "e"];
        let (result, truncated) = truncate_lines(&lines, 2);
        assert_eq!(result, "a\nb");
        assert!(truncated);
    }

    #[test]
    fn truncate_lines_empty() {
        let lines: Vec<&str> = vec![];
        let (result, truncated) = truncate_lines(&lines, 5);
        assert_eq!(result, "");
        assert!(!truncated);
    }

    // --- GitHubPermalink::parse_all ---

    #[test]
    fn parse_basic_permalink() {
        let text = "https://github.com/owner/repo/blob/abcdef1234567890abcdef1234567890abcdef12/src/main.rs";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].owner, "owner");
        assert_eq!(results[0].repo, "repo");
        assert_eq!(
            results[0].git_ref,
            "abcdef1234567890abcdef1234567890abcdef12"
        );
        assert_eq!(results[0].path, "src/main.rs");
        assert!(results[0].line_range.is_none());
    }

    #[test]
    fn parse_permalink_with_single_line() {
        let text = "https://github.com/owner/repo/blob/abcd1234/src/lib.rs#L42";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
        let range = results[0].line_range.unwrap();
        assert_eq!(range.start, 42);
        assert_eq!(range.end, 42);
    }

    #[test]
    fn parse_permalink_with_line_range() {
        let text = "https://github.com/owner/repo/blob/abcd1234/src/lib.rs#L10-L20";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
        let range = results[0].line_range.unwrap();
        assert_eq!(range.start, 10);
        assert_eq!(range.end, 20);
    }

    #[test]
    fn parse_branch_name() {
        let text = "https://github.com/owner/repo/blob/main/src/lib.rs";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].git_ref, "main");
        assert_eq!(results[0].path, "src/lib.rs");
    }

    #[test]
    fn parse_branch_name_with_line_range() {
        let text = "https://github.com/owner/repo/blob/develop/src/main.rs#L5-L10";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].git_ref, "develop");
        let range = results[0].line_range.unwrap();
        assert_eq!(range.start, 5);
        assert_eq!(range.end, 10);
    }

    #[test]
    fn parse_branch_name_with_single_line() {
        let text = "https://github.com/owner/repo/blob/main/src/lib.rs#L5";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].git_ref, "main");
        let range = results[0].line_range.unwrap();
        assert_eq!(range.start, 5);
        assert_eq!(range.end, 5);
    }

    #[test]
    fn parse_branch_with_special_characters() {
        let cases = [
            (
                "https://github.com/o/r/blob/release-v1.0/f.rs",
                "release-v1.0",
            ),
            (
                "https://github.com/o/r/blob/feat_something/f.rs",
                "feat_something",
            ),
            ("https://github.com/o/r/blob/v2.0.0/f.rs", "v2.0.0"),
        ];
        for (text, expected_ref) in cases {
            let results = GitHubPermalink::parse_all(text);
            assert_eq!(results.len(), 1, "failed for: {text}");
            assert_eq!(results[0].git_ref, expected_ref);
        }
    }

    #[test]
    fn parse_tag_name() {
        let text = "https://github.com/owner/repo/blob/v1.0.0/src/main.rs#L1-L10";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].git_ref, "v1.0.0");
        let range = results[0].line_range.unwrap();
        assert_eq!(range.start, 1);
        assert_eq!(range.end, 10);
    }

    #[test]
    fn parse_mixed_sha_and_branch() {
        let text = "https://github.com/o/r/blob/abcd1234/a.rs \
                    https://github.com/o/r/blob/main/b.rs";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].git_ref, "abcd1234");
        assert_eq!(results[1].git_ref, "main");
    }

    #[test]
    fn parse_accepts_short_ref() {
        // Short refs (e.g., short branch names) should still match
        let text = "https://github.com/owner/repo/blob/abc/src/lib.rs";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].git_ref, "abc");
    }

    #[test]
    fn parse_deduplicates_urls() {
        let text = "https://github.com/owner/repo/blob/abcd1234/src/lib.rs \
                    https://github.com/owner/repo/blob/abcd1234/src/lib.rs";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn parse_limits_to_three() {
        let text = "\
            https://github.com/o/r/blob/aaaa1111/a.rs \
            https://github.com/o/r/blob/bbbb2222/b.rs \
            https://github.com/o/r/blob/cccc3333/c.rs \
            https://github.com/o/r/blob/dddd4444/d.rs";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn parse_multiple_different_urls() {
        let text = "Check https://github.com/a/b/blob/1111aaaa/x.rs#L1 and \
                    https://github.com/c/d/blob/2222bbbb/y.py#L5-L10";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].owner, "a");
        assert_eq!(results[1].owner, "c");
        assert_eq!(results[1].path, "y.py");
    }

    #[test]
    fn parse_no_match() {
        let text = "Hello, no links here!";
        let results = GitHubPermalink::parse_all(text);
        assert!(results.is_empty());
    }

    #[test]
    fn parse_permalink_with_query() {
        let text = "https://github.com/owner/repo/blob/abcd1234/README.md?plain=1";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "README.md");
        assert!(results[0].line_range.is_none());
    }

    #[test]
    fn parse_permalink_with_query_and_line_range() {
        // Regression test for issue #593: the URL reported in the issue.
        let text = "https://github.com/m1sk9/dotfiles/blob/02962edfa2d9f5e1ed3f9a7cded1055b1a64b03d/private_dot_claude/CLAUDE.md?plain=1#L21-L46";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].owner, "m1sk9");
        assert_eq!(results[0].repo, "dotfiles");
        assert_eq!(
            results[0].git_ref,
            "02962edfa2d9f5e1ed3f9a7cded1055b1a64b03d"
        );
        assert_eq!(results[0].path, "private_dot_claude/CLAUDE.md");
        let range = results[0].line_range.unwrap();
        assert_eq!(range.start, 21);
        assert_eq!(range.end, 46);
    }

    #[test]
    fn parse_permalink_with_query_and_single_line() {
        let text = "https://github.com/owner/repo/blob/abcd1234/src/lib.rs?plain=1#L10";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "src/lib.rs");
        let range = results[0].line_range.unwrap();
        assert_eq!(range.start, 10);
        assert_eq!(range.end, 10);
    }

    #[test]
    fn parse_permalink_with_multiple_query_params() {
        let text = "https://github.com/owner/repo/blob/abcd1234/src/lib.rs?foo=bar&baz=qux#L1-L2";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "src/lib.rs");
        let range = results[0].line_range.unwrap();
        assert_eq!(range.start, 1);
        assert_eq!(range.end, 2);
    }

    #[test]
    fn parse_ignores_angle_bracket_link() {
        let text = "<https://github.com/owner/repo/blob/abcd1234/src/lib.rs#L10-L20>";
        let results = GitHubPermalink::parse_all(text);
        assert!(results.is_empty());
    }

    #[test]
    fn parse_nested_path() {
        let text = "https://github.com/owner/repo/blob/abcd1234/src/deeply/nested/path/file.rs";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "src/deeply/nested/path/file.rs");
    }

    #[test]
    fn parse_short_commit_sha() {
        // 4-character SHA is the minimum
        let text = "https://github.com/owner/repo/blob/abcd/file.rs";
        let results = GitHubPermalink::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].git_ref, "abcd");
    }

    // --- build_code_block ---

    fn make_permalink(path: &str, line_range: Option<LineRange>) -> GitHubPermalink {
        GitHubPermalink {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            git_ref: "abcdef1234567".to_string(),
            path: path.to_string(),
            line_range,
        }
    }

    #[test]
    fn build_code_block_full_file() {
        let permalink = make_permalink("src/main.rs", None);
        let body = "fn main() {\n    println!(\"hello\");\n}";
        let result = permalink.build_code_block(body, 50);

        match result {
            ExpandedContent::CodeBlock {
                language,
                code,
                metadata,
            } => {
                assert_eq!(language, "rust");
                assert_eq!(code, body);
                assert_eq!(metadata, "`src/main.rs` - owner/repo@abcdef1");
            }
            _ => panic!("expected CodeBlock"),
        }
    }

    #[test]
    fn build_code_block_with_line_range() {
        let permalink = make_permalink("src/lib.rs", Some(LineRange { start: 2, end: 3 }));
        let body = "line1\nline2\nline3\nline4";
        let result = permalink.build_code_block(body, 50);

        match result {
            ExpandedContent::CodeBlock {
                language,
                code,
                metadata,
            } => {
                assert_eq!(language, "rust");
                assert_eq!(code, "line2\nline3");
                assert!(metadata.contains("L2-L3"));
            }
            _ => panic!("expected CodeBlock"),
        }
    }

    #[test]
    fn build_code_block_truncated() {
        let permalink = make_permalink("app.py", None);
        let body = "a\nb\nc\nd\ne";
        let result = permalink.build_code_block(body, 2);

        match result {
            ExpandedContent::CodeBlock { code, metadata, .. } => {
                assert_eq!(code, "a\nb");
                assert!(metadata.contains("truncated to 2 lines"));
            }
            _ => panic!("expected CodeBlock"),
        }
    }

    #[test]
    fn build_code_block_line_range_truncated() {
        let permalink = make_permalink("app.py", Some(LineRange { start: 1, end: 5 }));
        let body = "a\nb\nc\nd\ne";
        let result = permalink.build_code_block(body, 3);

        match result {
            ExpandedContent::CodeBlock { code, metadata, .. } => {
                assert_eq!(code, "a\nb\nc");
                assert!(metadata.contains("L1-L5"));
                assert!(metadata.contains("truncated to 3 lines"));
            }
            _ => panic!("expected CodeBlock"),
        }
    }

    #[test]
    fn build_code_block_dockerfile_language() {
        let permalink = make_permalink("docker/Dockerfile", None);
        let body = "FROM rust:latest";
        let result = permalink.build_code_block(body, 50);

        match result {
            ExpandedContent::CodeBlock { language, .. } => {
                assert_eq!(language, "dockerfile");
            }
            _ => panic!("expected CodeBlock"),
        }
    }

    #[test]
    fn build_code_block_short_commit() {
        let permalink = GitHubPermalink {
            owner: "o".to_string(),
            repo: "r".to_string(),
            git_ref: "abcd".to_string(),
            path: "f.rs".to_string(),
            line_range: None,
        };
        let result = permalink.build_code_block("x", 50);

        match result {
            ExpandedContent::CodeBlock { metadata, .. } => {
                assert!(metadata.contains("o/r@abcd"));
            }
            _ => panic!("expected CodeBlock"),
        }
    }

    #[test]
    fn build_code_block_branch_ref() {
        let permalink = GitHubPermalink {
            owner: "o".to_string(),
            repo: "r".to_string(),
            git_ref: "main".to_string(),
            path: "f.rs".to_string(),
            line_range: None,
        };
        let result = permalink.build_code_block("x", 50);

        match result {
            ExpandedContent::CodeBlock { metadata, .. } => {
                // Branch names should not be truncated
                assert!(metadata.contains("o/r@main"));
            }
            _ => panic!("expected CodeBlock"),
        }
    }

    #[test]
    fn build_code_block_branch_ref_with_line_range() {
        let permalink = GitHubPermalink {
            owner: "o".to_string(),
            repo: "r".to_string(),
            git_ref: "develop".to_string(),
            path: "src/lib.rs".to_string(),
            line_range: Some(LineRange { start: 3, end: 5 }),
        };
        let body = "a\nb\nc\nd\ne\nf";
        let result = permalink.build_code_block(body, 50);

        match result {
            ExpandedContent::CodeBlock { code, metadata, .. } => {
                assert_eq!(code, "c\nd\ne");
                assert!(metadata.contains("L3-L5"));
                assert!(metadata.contains("o/r@develop"));
            }
            _ => panic!("expected CodeBlock"),
        }
    }

    // --- is_commit_sha / shorten_ref ---

    #[test]
    fn is_commit_sha_valid() {
        assert!(is_commit_sha("abcd1234"));
        assert!(is_commit_sha("abcdef1234567890abcdef1234567890abcdef12"));
    }

    #[test]
    fn is_commit_sha_boundary() {
        // Exactly 4 hex chars (minimum)
        assert!(is_commit_sha("abcd"));
        // Exactly 40 hex chars (full SHA-1)
        assert!(is_commit_sha("abcdef1234567890abcdef1234567890abcdef12"));
    }

    #[test]
    fn is_commit_sha_invalid() {
        assert!(!is_commit_sha("main"));
        assert!(!is_commit_sha("develop"));
        assert!(!is_commit_sha("abc")); // too short
        assert!(!is_commit_sha("abcdef1234567890abcdef1234567890abcdef123")); // too long (41)
        assert!(!is_commit_sha("ghijkl")); // non-hex
        assert!(!is_commit_sha("")); // empty
        assert!(is_commit_sha("ABCD1234")); // uppercase hex is still valid hex
    }

    #[test]
    fn shorten_ref_commit() {
        assert_eq!(shorten_ref("abcdef1234567890"), "abcdef1");
    }

    #[test]
    fn shorten_ref_short_sha() {
        // 4-char SHA should not be truncated further
        assert_eq!(shorten_ref("abcd"), "abcd");
    }

    #[test]
    fn shorten_ref_branch() {
        assert_eq!(shorten_ref("main"), "main");
        assert_eq!(shorten_ref("feature-branch"), "feature-branch");
        assert_eq!(shorten_ref("release-v1.0"), "release-v1.0");
        assert_eq!(shorten_ref("v2.0.0"), "v2.0.0");
    }
}
