//! Utility functions for babyrite.

/// Returns the language identifier for syntax highlighting based on file extension.
///
/// Used for code block language hints when expanding GitHub permalinks.
pub fn language_from_extension(extension: &str) -> &str {
    match extension.to_lowercase().as_str() {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "jsx" => "jsx",
        "tsx" => "tsx",
        "rb" => "ruby",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
        "cs" => "csharp",
        "swift" => "swift",
        "php" => "php",
        "scala" => "scala",
        "sh" | "bash" | "zsh" | "fish" => "bash",
        "ps1" => "powershell",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" => "scss",
        "sass" => "sass",
        "less" => "less",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "sql" => "sql",
        "md" | "markdown" => "markdown",
        "dockerfile" => "dockerfile",
        "lua" => "lua",
        "r" => "r",
        "dart" => "dart",
        "zig" => "zig",
        "nim" => "nim",
        "ex" | "exs" => "elixir",
        "erl" | "hrl" => "erlang",
        "hs" => "haskell",
        "ml" | "mli" => "ocaml",
        "clj" | "cljs" => "clojure",
        "tf" => "hcl",
        "vue" => "vue",
        "svelte" => "svelte",
        "graphql" | "gql" => "graphql",
        "proto" => "protobuf",
        "makefile" | "mk" => "makefile",
        _ => extension,
    }
}

/// Returns the language identifier for syntax highlighting based on a file path.
///
/// Uses the file extension when present; extensionless filenames
/// (e.g. `Dockerfile`, `Makefile`) are looked up by name.
pub fn language_for_path(path: &str) -> &str {
    let filename = path.rsplit('/').next().unwrap_or(path);
    match filename.rsplit_once('.') {
        Some((_, ext)) => language_from_extension(ext),
        None => language_from_extension(filename),
    }
}

/// Rewrites `text` so it cannot terminate a Discord code fence.
///
/// Discord closes a code block at the first ``` it encounters, so fetched file
/// content containing one would escape the fence and have the remainder
/// rendered as markdown. Runs of backticks are kept below three by wedging in a
/// zero-width space, which leaves the text visually unchanged. Single and double
/// backticks are untouched, so ordinary code (template literals, shell quoting)
/// still displays as written.
pub fn defuse_code_fences(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut backticks = 0;
    for c in text.chars() {
        if c == '`' {
            if backticks == 2 {
                out.push('\u{200b}');
                backticks = 0;
            }
            backticks += 1;
        } else {
            backticks = 0;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_extensions() {
        assert_eq!(language_from_extension("rs"), "rust");
        assert_eq!(language_from_extension("py"), "python");
        assert_eq!(language_from_extension("js"), "javascript");
        assert_eq!(language_from_extension("ts"), "typescript");
        assert_eq!(language_from_extension("go"), "go");
        assert_eq!(language_from_extension("java"), "java");
        assert_eq!(language_from_extension("cpp"), "cpp");
        assert_eq!(language_from_extension("cc"), "cpp");
        assert_eq!(language_from_extension("c"), "c");
        assert_eq!(language_from_extension("h"), "c");
        assert_eq!(language_from_extension("hpp"), "cpp");
        assert_eq!(language_from_extension("cs"), "csharp");
        assert_eq!(language_from_extension("rb"), "ruby");
        assert_eq!(language_from_extension("kt"), "kotlin");
        assert_eq!(language_from_extension("kts"), "kotlin");
        assert_eq!(language_from_extension("sh"), "bash");
        assert_eq!(language_from_extension("bash"), "bash");
        assert_eq!(language_from_extension("zsh"), "bash");
        assert_eq!(language_from_extension("fish"), "bash");
        assert_eq!(language_from_extension("yaml"), "yaml");
        assert_eq!(language_from_extension("yml"), "yaml");
        assert_eq!(language_from_extension("json"), "json");
        assert_eq!(language_from_extension("toml"), "toml");
        assert_eq!(language_from_extension("md"), "markdown");
        assert_eq!(language_from_extension("ex"), "elixir");
        assert_eq!(language_from_extension("exs"), "elixir");
        assert_eq!(language_from_extension("hs"), "haskell");
        assert_eq!(language_from_extension("tf"), "hcl");
        assert_eq!(language_from_extension("vue"), "vue");
        assert_eq!(language_from_extension("svelte"), "svelte");
        assert_eq!(language_from_extension("proto"), "protobuf");
        assert_eq!(language_from_extension("graphql"), "graphql");
        assert_eq!(language_from_extension("gql"), "graphql");
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(language_from_extension("RS"), "rust");
        assert_eq!(language_from_extension("Py"), "python");
        assert_eq!(language_from_extension("JS"), "javascript");
    }

    #[test]
    fn unknown_extension_returns_as_is() {
        assert_eq!(language_from_extension("xyz"), "xyz");
        assert_eq!(language_from_extension("foo"), "foo");
    }

    #[test]
    fn extensionless_filenames() {
        assert_eq!(language_from_extension("Dockerfile"), "dockerfile");
        assert_eq!(language_from_extension("dockerfile"), "dockerfile");
        assert_eq!(language_from_extension("Makefile"), "makefile");
        assert_eq!(language_from_extension("makefile"), "makefile");
    }

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

    /// The number of consecutive backticks a run of `n` is reduced to.
    fn longest_backtick_run(text: &str) -> usize {
        text.chars()
            .fold((0, 0), |(longest, run), c| {
                let run = if c == '`' { run + 1 } else { 0 };
                (longest.max(run), run)
            })
            .0
    }

    #[test]
    fn code_without_backticks_is_unchanged() {
        assert_eq!(defuse_code_fences("fn main() {}"), "fn main() {}");
    }

    #[test]
    fn short_backtick_runs_are_preserved() {
        // Inline code and shell quoting must survive verbatim.
        assert_eq!(defuse_code_fences("let s = `a`;"), "let s = `a`;");
        assert_eq!(defuse_code_fences("``double``"), "``double``");
    }

    #[test]
    fn no_backtick_run_survives_at_fence_length() {
        // Any run of three or more would close the fence Discord opened.
        for run in 3..=8 {
            let input = "`".repeat(run);
            assert!(
                longest_backtick_run(&defuse_code_fences(&input)) < 3,
                "a run of {run} backticks was not defused"
            );
        }
    }

    #[test]
    fn defusing_keeps_surrounding_text() {
        let defused = defuse_code_fences("before```after");
        assert!(longest_backtick_run(&defused) < 3);
        assert!(defused.starts_with("before"));
        assert!(defused.ends_with("after"));
        assert_eq!(defused.matches('`').count(), 3);
    }
}
