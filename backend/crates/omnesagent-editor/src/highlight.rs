//! Server-side syntax highlighting (phase F1): tree-sitter via `inkjet`,
//! the shared style table (one theme for gateway and client, `BACKEND_SPEC`
//! §9.4) and run-length row runs for the render plan.
//!
//! Design notes:
//! - The plan's `lapce-core/syntax` port (§11.11) is deferred; `inkjet` is the
//!   ready-made Apache/MIT layer that keeps ONE tree-sitter version per
//!   workspace (§11.5). This module is the swappable seam — callers only see
//!   `style_table()` and `row_runs()`.
//! - Runs are split at UTF-16 code-unit lengths (the client is a Dart string);
//!   spans are merged so that consecutive segments always cover the whole row.
//! - Markdown has no grammar in inkjet 0.11 — falls back to plaintext (no
//!   runs); add a dedicated grammar later without changing the contract.

use inkjet::tree_sitter_highlight::{HighlightEvent, Highlighter as TsHighlighter};
use serde::Serialize;

use crate::language::language_id_from_path;
use std::path::Path;
use std::sync::OnceLock;

/// One entry of the shared style table; `id` equals the index in
/// [`style_table`]. Delivered to the client once per session (`hello`).
#[derive(Debug, Clone, Serialize)]
pub struct StyleDef {
    pub id: u16,
    pub name: String,
    /// `#RRGGBB`.
    pub fg: String,
    pub bold: bool,
    pub italic: bool,
}

/// Fixed style classes (One-Dark-like default theme). Ids are wire-stable:
/// append-only changes allowed.
const STYLE_CLASSES: &[(&str, &str, bool, bool)] = &[
    // (name, fg, bold, italic)
    ("text", "#ABB2BF", false, false),        // 0
    ("comment", "#5C6370", false, true),      // 1
    ("keyword", "#C678DD", false, false),     // 2
    ("string", "#98C379", false, false),      // 3
    ("number", "#D19A66", false, false),      // 4
    ("constant", "#D19A66", false, false),    // 5
    ("function", "#61AFEF", false, false),    // 6
    ("type", "#E5C07B", false, false),        // 7
    ("property", "#E06C75", false, false),    // 8
    ("operator", "#56B6C2", false, false),    // 9
    ("punctuation", "#ABB2BF", false, false), // 10
    ("tag", "#E06C75", false, false),         // 11
    ("attribute", "#D19A66", false, true),    // 12
    ("escape", "#56B6C2", false, false),      // 13
    ("embedded", "#98C379", false, false),    // 14
    ("title", "#E5C07B", true, false),        // 15 (markdown headings etc.)
];

/// The shared style table (`hello.styles` payload).
///
/// Загружается из JSON-темы (один файл для шлюза и клиента, §9.4): путь берётся
/// из `OMNESAGENT_EDITOR_THEME`; без него — встроенная тема по умолчанию
/// (идентичная `frontend/shared/lib/editor/themes/default.json`).
pub fn style_table() -> &'static [StyleDef] {
    static TABLE: OnceLock<Vec<StyleDef>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let base: Vec<StyleDef> = STYLE_CLASSES
            .iter()
            .enumerate()
            .map(|(id, (name, fg, bold, italic))| StyleDef {
                id: id as u16,
                name: (*name).to_string(),
                fg: (*fg).to_string(),
                bold: *bold,
                italic: *italic,
            })
            .collect();
        match std::env::var("OMNESAGENT_EDITOR_THEME").ok() {
            Some(path) => match std::fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|raw| theme_from_json(&raw))
            {
                Ok(overridden) => overridden,
                Err(e) => {
                    eprintln!(
                        "[omnesagent-editor] theme `{path}` ignored: {e}"
                    );
                    base
                }
            },
            None => base,
        }
    })
}

/// Parse a theme JSON (`{"styles": [{"name", "fg", "bold", "italic"}, ...]}`).
/// Идентификаторы стилей wire-stable: имена маппятся на фиксированные классы
/// `STYLE_CLASSES` по порядку, неизвестные имена игнорируются.
pub fn theme_from_json(raw: &str) -> Result<Vec<StyleDef>, String> {
    #[derive(serde::Deserialize)]
    struct ThemeFile {
        #[serde(default)]
        styles: Vec<ThemeStyle>,
    }
    #[derive(serde::Deserialize)]
    struct ThemeStyle {
        name: String,
        fg: Option<String>,
        #[serde(default)]
        bold: bool,
        #[serde(default)]
        italic: bool,
    }

    let file: ThemeFile =
        serde_json::from_str(raw).map_err(|e| format!("invalid theme json: {e}"))?;

    let mut table: Vec<StyleDef> = STYLE_CLASSES
        .iter()
        .enumerate()
        .map(|(id, (name, fg, bold, italic))| StyleDef {
            id: id as u16,
            name: (*name).to_string(),
            fg: (*fg).to_string(),
            bold: *bold,
            italic: *italic,
        })
        .collect();
    for style in file.styles {
        if let Some(slot) = table.iter_mut().find(|s| s.name == style.name) {
            if let Some(fg) = style.fg {
                slot.fg = fg;
            }
            slot.bold = style.bold;
            slot.italic = style.italic;
        }
    }
    Ok(table)
}

/// Map a tree-sitter capture name (possibly dotted, e.g. `variable.builtin`)
/// to a style class id. Falls back through the dot hierarchy.
fn class_id_for_capture(name: &str) -> u16 {
    let mut head = name;
    loop {
        let class = match head {
            // Markdown (nvim-style captures from tree-sitter-md).
            "text.title" => 15,
            "text.literal" => 3,
            "text.quote" => 1,
            "text.uri" | "link_text" | "link_label" | "label" => 8,
            "text.emphasis" | "text.strong" | "text.strike" => 0,
            // Generic classes.
            "comment" | "line_comment" | "block_comment" | "documentation" => 1,
            "keyword" | "include" | "module" | "keyword.function" => 2,
            "string" => 3,
            "number" | "integer" | "float" => 4,
            "constant" | "boolean" | "character" => 5,
            "function" | "method" | "constructor" | "call" => 6,
            "type" | "class" | "enum" | "struct" | "interface" | "namespace"
            | "record" | "trait" => 7,
            "property" | "field" | "parameter" | "argument" => 8,
            "operator" => 9,
            "punctuation" | "delimiter" | "bracket" => 10,
            "tag" => 11,
            "attribute" => 12,
            "escape" => 13,
            "embedded" | "spell" | "none" => 14,
            "title" => 15,
            _ => {
                match head.split_once('.') {
                    Some((prefix, _)) => {
                        head = prefix;
                        continue;
                    }
                    None => 0,
                }
            }
        };
        return class;
    }
}

/// inkjet language for our language id (see `language.rs`); `None` means
/// "no highlighting available" — the client paints plain text.
fn inkjet_language(language_id: &str) -> Option<inkjet::Language> {
    use inkjet::Language as L;
    Some(match language_id {
        "rust" => L::Rust,
        "dart" => L::Dart,
        "json" => L::Json,
        "yaml" => L::Yaml,
        "toml" => L::Toml,
        "shell" => L::Bash,
        "python" => L::Python,
        "typescript" => L::Typescript,
        "tsx" => L::Tsx,
        "javascript" | "jsx" => L::Javascript,
        "html" => L::Html,
        "css" => L::Css,
        "scss" => L::Scss,
        "c" => L::C,
        "cpp" => L::Cpp,
        "go" => L::Go,
        "dockerfile" => L::Dockerfile,
        "diff" => L::Diff,
        "sql" => L::Sql,
        "hcl" => L::Hcl,
        "java" => L::Java,
        "kotlin" => L::Kotlin,
        _ => return None,
    })
}

/// Расширенный список recognized-имён для markdown: имена inkjet (индексы
/// инъекций не смещаются — append-only) + nvim-style capture'ы markdown.
const MARKDOWN_EXTRA_NAMES: &[&str] = &[
    "text.title",
    "text.literal",
    "text.quote",
    "text.emphasis",
    "text.strong",
    "text.strike",
    "text.uri",
    "link_text",
    "link_label",
];

fn recognized_names() -> &'static Vec<&'static str> {
    static NAMES: OnceLock<Vec<&'static str>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let mut all = inkjet::constants::HIGHLIGHT_NAMES.to_vec();
        for name in MARKDOWN_EXTRA_NAMES {
            if !all.contains(name) {
                all.push(name);
            }
        }
        all
    })
}

/// Markdown uses the block grammar of `tree-sitter-md` (0.3.2, tree-sitter
/// ^0.23 — та же версия, что и в воркспейсе); code fences are injected into
/// inkjet languages through the standard injection callback.
fn markdown_config() -> &'static inkjet::tree_sitter_highlight::HighlightConfiguration {
    static CONFIG: OnceLock<inkjet::tree_sitter_highlight::HighlightConfiguration> =
        OnceLock::new();
    CONFIG.get_or_init(|| {
        let lang = tree_sitter::Language::new(tree_sitter_md::LANGUAGE);
        let mut config = inkjet::tree_sitter_highlight::HighlightConfiguration::new(
            lang,
            "markdown",
            tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
            tree_sitter_md::INJECTION_QUERY_BLOCK,
            "",
        )
        .expect("markdown highlight configuration must compile");
        config.configure(recognized_names());
        config
    })
}

fn config_for(language_id: &str) -> Option<&'static inkjet::tree_sitter_highlight::HighlightConfiguration> {
    if language_id == "markdown" {
        return Some(markdown_config());
    }
    inkjet_language(language_id).map(|l| l.config())
}

/// Whether the language has a highlighter (drives `runs` presence).
pub fn supports_language(language_id: &str) -> bool {
    config_for(language_id).is_some()
}

/// Run-length runs for every row of `text` (row content, terminator
/// excluded): `runs[row]` is a list of `(len_utf16, style_id)` segments whose
/// lengths sum to the row's UTF-16 length. `None` for unsupported languages.
pub fn row_runs(text: &str, language_id: &str) -> Option<Vec<Vec<(u32, u16)>>> {
    let config = config_for(language_id)?;

    let mut highlighter = TsHighlighter::new();
    let events = highlighter
        .highlight(
            config,
            text.as_bytes(),
            None,
            |token| inkjet::Language::from_token(token).map(|l| l.config()),
        )
        .ok()?;

    // Flatten events into byte spans with a style class. `Highlight.0`
    // indexes the recognized-names list the config was `configure`d with —
    // inkjet configs use HIGHLIGHT_NAMES, markdown uses the same list with
    // markdown names appended (append-only keeps injected indices stable).
    let mut spans: Vec<(usize, usize, u16)> = Vec::new();
    let mut class_stack: Vec<u16> = Vec::new();
    for event in events {
        match event.ok()? {
            HighlightEvent::HighlightStart(h) => {
                let name = recognized_names().get(h.0).copied().unwrap_or("");
                class_stack.push(class_id_for_capture(name));
            }
            HighlightEvent::HighlightEnd => {
                class_stack.pop();
            }
            HighlightEvent::Source { start, end } => {
                let class = class_stack.last().copied().unwrap_or(0);
                spans.push((start, end, class));
            }
        }
    }

    // Row byte ranges (content without the terminator).
    let mut row_ranges: Vec<(usize, usize)> = Vec::new();
    let mut cursor = 0usize;
    for line in text.split('\n') {
        let content_end = cursor + line.len();
        let trimmed_end = content_end - (line.ends_with('\r') as usize);
        row_ranges.push((cursor, trimmed_end));
        cursor = content_end + 1;
    }

    let mut result: Vec<Vec<(u32, u16)>> = Vec::with_capacity(row_ranges.len());
    let mut span_pos = 0usize;
    for (row_start, row_end) in row_ranges {
        let mut runs: Vec<(u32, u16)> = Vec::new();
        let mut pos = row_start;
        // Skip spans fully before this row.
        while span_pos < spans.len() && spans[span_pos].1 <= row_start {
            span_pos += 1;
        }
        let mut idx = span_pos;
        while idx < spans.len() {
            let (s, e, class) = spans[idx];
            if s >= row_end {
                break;
            }
            let seg_start = s.max(pos);
            let seg_end = e.min(row_end);
            if seg_start >= seg_end {
                idx += 1;
                continue;
            }
            if seg_start > pos {
                // Gap covered by an outer class — fill with the previous style.
                let prev = runs.last().map(|&(_, c)| c).unwrap_or(0);
                runs.push((utf16_len(&text[pos..seg_start]) as u32, prev));
            }
            runs.push((utf16_len(&text[seg_start..seg_end]) as u32, class));
            pos = seg_end;
            if pos >= row_end {
                break;
            }
            idx += 1;
        }
        if pos < row_end {
            let prev = runs.last().map(|&(_, c)| c).unwrap_or(0);
            runs.push((utf16_len(&text[pos..row_end]) as u32, prev));
        }
        result.push(runs);
    }
    Some(result)
}

fn utf16_len(s: &str) -> usize {
    s.chars().map(char::len_utf16).sum()
}

/// Convenience: language id from a path (re-exported for the gateway).
pub fn language_id_for_path(path: &Path) -> &'static str {
    language_id_from_path(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_table_ids_match_order() {
        let table = style_table();
        assert_eq!(table[0].name, "text");
        assert_eq!(table[1].name, "comment");
        for (i, style) in table.iter().enumerate() {
            assert_eq!(style.id, i as u16);
            assert!(style.fg.starts_with('#') && style.fg.len() == 7);
        }
    }

    #[test]
    fn rust_snippet_produces_keyword_and_comment_runs() {
        let src = "// привет\nfn main() { let x = 42; }\n";
        let runs = row_runs(src, "rust").expect("rust must be supported");
        assert_eq!(runs.len(), 3);

        // Comment row: whole row is one comment-colored run.
        assert_eq!(runs[0], vec![(9, 1)]);

        // Second row contains a keyword run with a non-default class.
        let second: &Vec<(u32, u16)> = &runs[1];
        let total: u32 = second.iter().map(|(l, _)| l).sum();
        assert_eq!(total as usize, "fn main() { let x = 42; }".len());
        assert!(second.iter().any(|&(_, c)| c == 2), "keyword run present");
        // Numbers carry `constant.numeric.*` → class 4 or the `constant`
        // fallthrough class 5 (both are the same theme color).
        assert!(
            second.iter().any(|&(_, c)| c == 4 || c == 5),
            "number run present"
        );
    }

    #[test]
    fn runs_cover_every_row_exactly() {
        let src = "let s = \"привет мир\"; // строка с кириллицей\nlet n = 1;\n\n";
        let runs = row_runs(src, "rust").unwrap();
        for (row, line) in src.split('\n').enumerate() {
            let total: u32 = runs[row].iter().map(|(l, _)| l).sum();
            assert_eq!(total as usize, line.chars().map(char::len_utf16).sum::<usize>());
            assert!(runs[row].iter().all(|(l, _)| *l > 0));
        }
    }

    #[test]
    fn unsupported_language_returns_none() {
        assert!(row_runs("hello\n", "plaintext").is_none());
        assert!(row_runs("hello\n", "made-up-lang").is_none());
    }

    #[test]
    fn markdown_headings_and_code_blocks_are_colored() {
        let src = "# Заголовок\n\nОбычный текст.\n\n```rust\nfn main() {}\n```\n";
        let runs = row_runs(src, "markdown").expect("markdown must be supported");
        assert_eq!(runs.len(), 8);

        // Заголовок — title-класс (15).
        assert!(
            runs[0].iter().any(|&(_, c)| c == 15),
            "heading must be title-styled: runs={:?}",
            runs[0]
        );
        // Код-блок: строка с `fn main() {}` содержит keyword-класс (инъекция rust).
        let code_row = &runs[5];
        assert!(
            code_row.iter().any(|&(_, c)| c == 2),
            "injected rust code must be highlighted: runs={:?}",
            code_row
        );
    }

    #[test]
    fn language_id_from_path_matches_table() {
        assert_eq!(language_id_for_path(Path::new("a/b/main.rs")), "rust");
        assert!(supports_language("dart"));
        assert!(supports_language("shell"));
        assert!(supports_language("markdown"));
    }

    #[test]
    fn theme_json_overrides_by_name() {
        let raw = r##"{"styles":[{"name":"keyword","fg":"#FF0000","bold":true},{"name":"unknown-one","fg":"#000000"}]}"##;
        let table = theme_from_json(raw).unwrap();
        assert_eq!(table.len(), STYLE_CLASSES.len());
        let keyword = table.iter().find(|s| s.name == "keyword").unwrap();
        assert_eq!(keyword.fg, "#FF0000");
        assert!(keyword.bold);
        // Порядок и id wire-stable.
        assert_eq!(table[0].name, "text");
        assert_eq!(table[0].fg, "#ABB2BF");
        let broken = theme_from_json("не json");
        assert!(broken.is_err());
    }

    #[test]
    fn capture_mapping_falls_through_dots() {
        assert_eq!(class_id_for_capture("keyword.modifier"), 2);
        assert_eq!(class_id_for_capture("variable.builtin"), 0);
        assert_eq!(class_id_for_capture("function.macro"), 6);
        assert_eq!(class_id_for_capture("totally_unknown"), 0);
    }

    /// Budget §7: после правки пересчитывается только затронутый чанк
    /// (200 строк), а не файл. Печатает фактическую длительность.
    #[test]
    fn edit_recompute_is_within_budget() {
        use crate::buffer::{EditorBuffer, EditOp, EditorPos};

        let mut src = String::with_capacity(160 * 5000);
        for i in 0..5000 {
            src.push_str(&format!(
                "fn process_{i}(input: &str, limit: usize) -> Option<String> {{\n\
                 \x20   // Шаг {i}: нормализуем вход и считаем длину.\n\
                 \x20   let trimmed = input.trim();\n\
                 \x20   if trimmed.len() > limit {{ return None; }}\n\
                 \x20   Some(format!(\"{{}}-{}\", trimmed))\n}}\n",
                i % 7
            ));
        }
        let mut b = EditorBuffer::from_text(
            1,
            std::path::PathBuf::from("bench.rs"),
            src,
            false,
        );

        // Прогрев: клиент запросил весь файл — все чанки посчитаны.
        let page = b.rows(0, 5000);
        assert_eq!(page.rows.len(), 5000);
        assert!(page.rows.iter().any(|r| !r.runs.is_empty()));
        let before = b.runs_recompute_count;

        // Правка в середине файла.
        let base = b.rev();
        b.apply_ops(
            base,
            &[EditOp {
                start: EditorPos { line: 2500, col: 0 },
                end: EditorPos { line: 2500, col: 0 },
                text: "// вставка правки\n".into(),
            }],
        )
        .unwrap();

        let start = std::time::Instant::now();
        let page = b.rows(2490, 20);
        let elapsed = start.elapsed();
        let recomputed = b.runs_recompute_count - before;
        println!(
            "per-edit chunk recompute: {elapsed:?} ({:.2} ms), chunks recomputed: {recomputed}",
            elapsed.as_secs_f64() * 1000.0
        );
        assert!(recomputed <= 2, "an edit must not recompute the file");
        assert!(!page.rows.iter().all(|r| r.runs.is_empty()));
        // Щедрый порог для debug-профиля; фактическое время — в лог.
        assert!(elapsed.as_millis() < 250, "per-edit recompute too slow: {elapsed:?}");
    }

    /// Комментарий, открытый в контексте перед чанком, красит строки чанка.
    #[test]
    fn comment_opened_before_chunk_colors_into_it() {
        use crate::buffer::EditorBuffer;

        // Строки 0..199 — обычный код; строка 198 открывает блочный
        // комментарий, строки 200+ (следующий чанк) остаются в нём.
        let mut src = String::new();
        for i in 0..198 {
            src.push_str(&format!("let v{i} = {i};\n"));
        }
        src.push_str("/* открытый комментарий\n");
        for i in 0..50 {
            src.push_str(&format!("ещё строка комментария {i}\n"));
        }
        src.push_str("*/\nlet после = 1;\n");

        let mut b = EditorBuffer::from_text(
            2,
            std::path::PathBuf::from("seam.rs"),
            src,
            false,
        );
        // Прогрев обоих чанков (198 кода + 52 строки комментария/кода).
        let page = b.rows(0, 260);
        assert_eq!(page.rows.len(), 252);
        // Строка 205 (внутри комментария, во втором чанке) — comment-класс.
        let row = &page.rows[205];
        assert!(
            row.runs.iter().any(|&(_, c)| c == 1),
            "comment context must color the next chunk: runs={:?}",
            row.runs
        );
        // Строка 250 (после закрытия) — не комментарий.
        let row = &page.rows[250];
        assert!(
            !row.runs.iter().any(|&(_, c)| c == 1),
            "comment must end: runs={:?}",
            row.runs
        );
    }
}
