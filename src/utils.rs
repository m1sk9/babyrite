//! Utility functions for babyrite.

/// Returns the language identifier for syntax highlighting based on file extension.
///
/// Used for code block language hints when expanding GitHub permalinks.
pub fn language_from_extension(extension: &str) -> &str {
    match extension.to_lowercase().as_str() {
        "rs" => "rust",
        "py" | "pyi" | "pyw" => "python",
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
        "ps1" | "psm1" => "powershell",
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
        "clj" | "cljs" | "cljc" => "clojure",
        "tf" => "hcl",
        "vue" => "vue",
        "svelte" => "svelte",
        "graphql" | "gql" => "graphql",
        "proto" => "protobuf",
        "makefile" | "mk" => "makefile",
        "jl" => "julia",
        // `.m` is shared with Matlab; Objective-C is far more common on GitHub.
        "m" | "mm" => "objectivec",
        "f" | "for" | "f77" => "fortran",
        "fs" | "fsx" | "fsi" => "fsharp",
        "adb" | "ads" => "ada",
        "asm" | "s" => "x86asm",
        "lisp" | "el" | "cl" => "lisp",
        "scm" | "ss" | "rkt" => "scheme",
        "ll" => "llvm",
        "vhd" => "vhdl",
        "vert" | "frag" => "glsl",
        "j2" => "django",
        "conf" => "nginx",
        "htaccess" => "apache",
        "sty" => "latex",
        "wat" => "wasm",
        _ => extension,
    }
}

/// Returns the language identifier for syntax highlighting based on a file path.
///
/// Filenames whose extension alone points to the wrong language
/// (e.g. `CMakeLists.txt`) are looked up by name first. Otherwise the file
/// extension is used when present; extensionless filenames
/// (e.g. `Dockerfile`, `Makefile`) are looked up by name.
pub fn language_for_path(path: &str) -> &str {
    let filename = path.rsplit('/').next().unwrap_or(path);
    if let Some(language) = language_from_filename(filename) {
        return language;
    }
    match filename.rsplit_once('.') {
        Some((_, ext)) => language_from_extension(ext),
        None => language_from_extension(filename),
    }
}

fn language_from_filename(filename: &str) -> Option<&'static str> {
    match filename.to_lowercase().as_str() {
        "cmakelists.txt" => Some("cmake"),
        "httpd.conf" | "apache2.conf" => Some("apache"),
        _ => None,
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

/// Returns `language` for use as a code fence's info string, or an empty info
/// string when `language` contains a backtick.
///
/// Unknown extensions are passed through verbatim, and a backtick there would
/// close the fence it is meant to open.
pub fn fence_info(language: &str) -> &str {
    if language.contains('`') { "" } else { language }
}

/// Escapes `text` so Discord renders it literally, on a single line.
///
/// Why not escape every punctuation character: `-` and `.` only act as markup
/// at the start of a line and followed by a space, so leaving them alone keeps
/// names such as `rust-lang/rust.vim@release-v1.0` unchanged. `:` is escaped
/// because it is what turns a URL into a link.
pub fn escape_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if c.is_whitespace() {
            out.push(' ');
            continue;
        }
        if matches!(
            c,
            '\\' | '`' | '*' | '_' | '~' | '|' | '[' | ']' | '<' | '>' | '#' | ':'
        ) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Formats `text` as inline code, or as escaped plain text when it contains a
/// backtick.
///
/// Why not inline code throughout: a backtick closes the span early, and
/// backslash escapes have no effect inside it.
pub fn inline_code(text: &str) -> String {
    if text.contains('`') {
        escape_markdown(text)
    } else {
        format!("`{text}`")
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
    fn extensions_highlight_js_does_not_recognize() {
        let cases = [
            ("jl", "julia"),
            ("m", "objectivec"),
            ("mm", "objectivec"),
            ("f", "fortran"),
            ("for", "fortran"),
            ("f77", "fortran"),
            ("fsx", "fsharp"),
            ("fsi", "fsharp"),
            ("adb", "ada"),
            ("ads", "ada"),
            ("asm", "x86asm"),
            ("s", "x86asm"),
            ("el", "lisp"),
            ("cl", "lisp"),
            ("ss", "scheme"),
            ("rkt", "scheme"),
            ("ll", "llvm"),
            ("vhd", "vhdl"),
            ("vert", "glsl"),
            ("frag", "glsl"),
            ("j2", "django"),
            ("conf", "nginx"),
            ("cljc", "clojure"),
            ("pyi", "python"),
            ("pyw", "python"),
            ("psm1", "powershell"),
            ("sty", "latex"),
            ("wat", "wasm"),
        ];
        for (extension, language) in cases {
            assert_eq!(language_from_extension(extension), language, "{extension}");
        }
    }

    #[test]
    fn language_for_path_htaccess() {
        assert_eq!(language_for_path("public/.htaccess"), "apache");
    }

    #[test]
    fn language_for_path_apache_config() {
        assert_eq!(language_for_path("conf/httpd.conf"), "apache");
        assert_eq!(language_for_path("apache2/apache2.conf"), "apache");
    }

    #[test]
    fn language_for_path_cmakelists_is_not_plaintext() {
        assert_eq!(language_for_path("CMakeLists.txt"), "cmake");
        assert_eq!(language_for_path("src/CMakeLists.txt"), "cmake");
    }

    #[test]
    fn language_for_path_other_txt_is_unchanged() {
        assert_eq!(language_for_path("docs/notes.txt"), "txt");
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

    #[test]
    fn fence_info_keeps_languages_without_backticks() {
        for language in ["rust", "c~", "%E8%A8%AD", "gitignore", ""] {
            assert_eq!(fence_info(language), language);
        }
    }

    #[test]
    fn fence_info_is_empty_for_a_language_containing_a_backtick() {
        assert_eq!(fence_info("a```"), "");
        assert_eq!(fence_info("a`"), "");
    }

    #[test]
    fn escape_markdown_leaves_github_names_unchanged() {
        for name in ["rust-lang", "rust.vim", "release-v1.0", "v1.0.0", "abcdef1"] {
            assert_eq!(escape_markdown(name), name);
        }
    }

    #[test]
    fn escape_markdown_escapes_markup_characters() {
        assert_eq!(
            escape_markdown("[x](https://evil.example)"),
            r"\[x\](https\://evil.example)"
        );
        assert_eq!(
            escape_markdown("**a** _b_ ~~c~~ ||d||"),
            r"\*\*a\*\* \_b\_ \~\~c\~\~ \|\|d\|\|"
        );
        assert_eq!(escape_markdown("`x` <#1>"), r"\`x\` \<\#1\>");
    }

    #[test]
    fn escape_markdown_escapes_backslashes_so_they_cannot_cancel_an_escape() {
        assert_eq!(escape_markdown(r"\*"), r"\\\*");
    }

    #[test]
    fn escape_markdown_keeps_text_on_one_line() {
        assert_eq!(escape_markdown("a\n# b\r\n> c"), r"a \# b  \> c");
    }

    #[test]
    fn inline_code_wraps_text_without_backticks() {
        assert_eq!(inline_code("src/main.rs"), "`src/main.rs`");
    }

    #[test]
    fn inline_code_falls_back_to_escaped_text_for_text_with_backticks() {
        assert_eq!(inline_code("x.a```"), r"x.a\`\`\`");
        assert_eq!(inline_code("x`[y](https://e)"), r"x\`\[y\](https\://e)");
    }
}
