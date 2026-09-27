//! Editor language detection by file path. A minimal table for F0; the full
//! tree-sitter registry with queries and language properties lands in phase F1
//! (`BACKEND_SPEC` §9.4).

use std::path::Path;

/// Stable language id used in `hello` frames and buffer metadata.
/// Matched case-insensitively against the path extension.
pub fn language_id_from_path(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match ext.as_str() {
        "rs" => "rust",
        "dart" => "dart",
        "json" => "json",
        "md" | "markdown" => "markdown",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "sh" | "bash" | "zsh" => "shell",
        "py" | "pyi" => "python",
        "ts" | "mts" | "cts" => "typescript",
        "tsx" => "tsx",
        "js" | "mjs" | "cjs" => "javascript",
        "jsx" => "jsx",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" => "scss",
        "c" | "h" => "c",
        "cpp" | "hpp" | "cc" | "hh" => "cpp",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "swift" => "swift",
        "rb" => "ruby",
        "php" => "php",
        "xml" | "svg" => "xml",
        "sql" => "sql",
        "lock" => "json",
        _ => "plaintext",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_common_languages() {
        assert_eq!(language_id_from_path(Path::new("lib/main.dart")), "dart");
        assert_eq!(language_id_from_path(Path::new("src/lib.rs")), "rust");
        assert_eq!(language_id_from_path(Path::new("Cargo.TOML")), "toml");
        assert_eq!(language_id_from_path(Path::new("README.md")), "markdown");
        assert_eq!(language_id_from_path(Path::new("run.sh")), "shell");
    }

    #[test]
    fn falls_back_to_plaintext() {
        assert_eq!(language_id_from_path(Path::new("Makefile")), "plaintext");
        assert_eq!(language_id_from_path(Path::new("no_ext")), "plaintext");
    }
}
