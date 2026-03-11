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
        "c" => "c",
        "h" => "c",
        "cpp" | "cc" | "cxx" => "cpp",
        "hpp" | "hxx" => "cpp",
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
}
