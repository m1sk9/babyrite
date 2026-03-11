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

        Ok(self.build_code_block(&body, max_lines))
    }

    /// Builds an `ExpandedContent::CodeBlock` from raw file content.
    fn build_code_block(&self, body: &str, max_lines: usize) -> ExpandedContent {
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
        let language = language_for_path(&self.path);

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

        ExpandedContent::CodeBlock {
            language: language.to_string(),
            code,
            metadata,
        }
    }
}

/// Extracts the filename from a path and returns the appropriate language identifier.
fn language_for_path(path: &str) -> &str {
    let filename = path.rsplit('/').next().unwrap_or(path);
    match filename.rsplit_once('.') {
        Some((_, ext)) => language_from_extension(ext),
        None => language_from_extension(filename),
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
            results[0].commit,
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
    fn parse_rejects_branch_name() {
        // Branch names (non-hex) should not match
        let text = "https://github.com/owner/repo/blob/main/src/lib.rs";
        let results = GitHubPermalink::parse_all(text);
        assert!(results.is_empty());
    }

    #[test]
    fn parse_rejects_short_sha() {
        // SHA must be at least 4 hex characters
        let text = "https://github.com/owner/repo/blob/abc/src/lib.rs";
        let results = GitHubPermalink::parse_all(text);
        assert!(results.is_empty());
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
        assert_eq!(results[0].commit, "abcd");
    }

    // --- language_for_path ---

    #[test]
    fn language_for_path_basic_extension() {
        assert_eq!(language_for_path("src/main.rs"), "rust");
    }

    #[test]
    fn language_for_path_dockerfile_in_subdir() {
        assert_eq!(language_for_path("docker/Dockerfile"), "dockerfile");
    }

    #[test]
    fn language_for_path_dotted_directory() {
        assert_eq!(language_for_path("some.config/Dockerfile"), "dockerfile");
    }

    #[test]
    fn language_for_path_makefile_in_subdir() {
        assert_eq!(language_for_path("build/Makefile"), "makefile");
    }

    #[test]
    fn language_for_path_multiple_dots() {
        assert_eq!(language_for_path("file.test.ts"), "typescript");
    }

    #[test]
    fn language_for_path_dotfile() {
        assert_eq!(language_for_path(".gitignore"), "gitignore");
    }

    // --- build_code_block ---

    fn make_permalink(path: &str, line_range: Option<LineRange>) -> GitHubPermalink {
        GitHubPermalink {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            commit: "abcdef1234567".to_string(),
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
            commit: "abcd".to_string(),
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
}
