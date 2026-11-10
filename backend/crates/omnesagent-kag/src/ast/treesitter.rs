//! Ф6.1 (ob2h v2.0, гибридный AST): tree-sitter extraction.
//!
//! Feature-gated behind `treesitter`. Парсит Rust/TS/JS/Python/Go грамматиками,
//! вендоренными через `inkjet` (одна версия tree-sitter в воркспейсе, без
//! отдельных crate-грамматик — бюджет грамматик удержан), и эмитит узлы/рёбра
//! графа с провенансом об2h v2.0 (`SYNTAX`/`RESOLVED`/`HEURISTIC`).
//!
//! Если доля ERROR/MISSING-узлов в дереве превышает [`FALLBACK_ERROR_RATIO`],
//! [`extract`] возвращает `None`, и вызывающий код отступает на regex-экстрактор.

use super::{AstEdge, AstNode};
use std::collections::HashSet;
use std::path::Path;
use tree_sitter::{Language, Node, Parser};

/// Провенанс ребра (ob2h v2.0): извлечено напрямую из дерева разбора.
pub const PROVENANCE_SYNTAX: &str = "SYNTAX";
/// Провенанс ребра (ob2h v2.0): подтверждено/добавлено type-resolution проходом.
pub const PROVENANCE_RESOLVED: &str = "RESOLVED";
/// Провенанс ребра (ob2h v2.0): framework/паттерн-эвристики.
pub const PROVENANCE_HEURISTIC: &str = "HEURISTIC";

/// Отступление на regex-экстрактор, если доля ERROR-узлов превышает 10%.
pub const FALLBACK_ERROR_RATIO: f64 = 0.10;

/// Языки, покрытые tree-sitter экстрактором.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsLanguage {
    Rust,
    Python,
    TypeScript,
    Tsx,
    JavaScript,
    Jsx,
    Go,
}

impl TsLanguage {
    /// Выбор грамматики по расширению файла.
    pub fn from_path(rel_path: &str) -> Option<Self> {
        let ext = Path::new(rel_path)
            .extension()?
            .to_str()?
            .to_ascii_lowercase();
        match ext.as_str() {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            "js" => Some(Self::JavaScript),
            "jsx" => Some(Self::Jsx),
            "go" => Some(Self::Go),
            _ => None,
        }
    }

    /// Исходная грамматика tree-sitter. Грамматики компилирует `inkjet`,
    /// публично экспонируя их через [`inkjet::Language`]; переиспользуем их,
    /// чтобы не тянуть отдельные crate-грамматики.
    fn grammar(self) -> Language {
        let lang = match self {
            Self::Rust => inkjet::Language::Rust,
            Self::Python => inkjet::Language::Python,
            Self::TypeScript => inkjet::Language::Typescript,
            Self::Tsx => inkjet::Language::Tsx,
            Self::JavaScript => inkjet::Language::Javascript,
            Self::Jsx => inkjet::Language::Jsx,
            Self::Go => inkjet::Language::Go,
        };
        lang.config().language.clone()
    }
}

/// Результат разбора одного файла.
pub struct TsExtraction {
    pub nodes: Vec<AstNode>,
    pub edges: Vec<AstEdge>,
}

/// Поддерживается ли файл tree-sitter экстрактором.
pub fn supports(rel_path: &str) -> bool {
    TsLanguage::from_path(rel_path).is_some()
}

/// Разобрать `content` и извлечь определения и импорты.
///
/// Возвращает `None`, когда язык не поддержан, разбор не удался, либо доля
/// ERROR/MISSING-узлов превышает [`FALLBACK_ERROR_RATIO`] — тогда вызывающий
/// код должен отступить на regex-экстрактор.
pub fn extract(rel_path: &str, content: &str) -> Option<TsExtraction> {
    let lang = TsLanguage::from_path(rel_path)?;
    let mut parser = Parser::new();
    parser.set_language(&lang.grammar()).ok()?;
    let tree = parser.parse(content, None)?;
    let root = tree.root_node();
    if should_fallback(root) {
        return None;
    }
    Some(collect(rel_path, content.as_bytes(), root, lang))
}

/// Доля узлов дерева, являющихся ERROR или MISSING.
pub fn missing_error_ratio(root: Node) -> f64 {
    let mut total: u64 = 0;
    let mut bad: u64 = 0;
    walk(root, |node| {
        total += 1;
        if node.is_error() || node.is_missing() {
            bad += 1;
        }
    });
    if total == 0 { 0.0 } else { bad as f64 / total as f64 }
}

/// Превышает ли доля ошибок порог отступления.
fn should_fallback(root: Node) -> bool {
    missing_error_ratio(root) > FALLBACK_ERROR_RATIO
}

/// Собрать узлы/рёбра из дерева разбора.
fn collect(rel_path: &str, src: &[u8], root: Node, lang: TsLanguage) -> TsExtraction {
    let file_node_id = format!("file:{}", rel_path);
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    walk(root, |node| {
        if let Some((prefix, node_type)) = definition_kind(node, lang) {
            if let Some(name) = node_name(node, src) {
                let node_id = format!("{}:{}:{}", prefix, rel_path, name);
                if seen.insert(node_id.clone()) {
                    nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: name.to_string(),
                        node_type: node_type.to_string(),
                        description: format!("{} `{}` в {}", node_type, name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: node.start_position().row + 1,
                        line_end: node.end_position().row + 1,
                    });
                    edges.push(AstEdge {
                        source_node_id: file_node_id.clone(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: snippet(node, src),
                        provenance: PROVENANCE_SYNTAX.to_string(),
                    });
                }
            }
        } else if let Some(module) = import_target(node, lang, src) {
            edges.push(AstEdge {
                source_node_id: file_node_id.clone(),
                target_node_id: format!("module:{}", module),
                label: "IMPORTS".to_string(),
                weight: 1.0,
                context: snippet(node, src),
                provenance: PROVENANCE_SYNTAX.to_string(),
            });
        }
    });

    TsExtraction { nodes, edges }
}

/// Проекция узла на пару (префикс node_id, тип узла графа).
fn definition_kind(node: Node, lang: TsLanguage) -> Option<(&'static str, &'static str)> {
    match lang {
        TsLanguage::Rust => match node.kind() {
            "function_item" => Some(("fn", "Function")),
            "struct_item" | "enum_item" | "union_item" => Some(("struct", "Struct")),
            "trait_item" => Some(("trait", "Trait")),
            "mod_item" => Some(("module", "Module")),
            _ => None,
        },
        TsLanguage::Python => match node.kind() {
            "function_definition" => Some(("fn", "Function")),
            "class_definition" => Some(("class", "Class")),
            _ => None,
        },
        TsLanguage::TypeScript | TsLanguage::Tsx | TsLanguage::JavaScript | TsLanguage::Jsx => {
            match node.kind() {
                "function_declaration"
                | "generator_function_declaration"
                | "method_definition" => Some(("fn", "Function")),
                "class_declaration" => Some(("class", "Class")),
                "interface_declaration" => Some(("interface", "Interface")),
                _ => None,
            }
        }
        TsLanguage::Go => match node.kind() {
            "function_declaration" | "method_declaration" => Some(("fn", "Function")),
            "type_spec" if has_child_kind(node, "struct_type") => Some(("struct", "Struct")),
            _ => None,
        },
    }
}

/// Имя объявления из поля `name` узла.
fn node_name<'a>(node: Node, src: &'a [u8]) -> Option<&'a str> {
    let name = node.child_by_field_name("name")?.utf8_text(src).ok()?;
    (!name.is_empty()).then_some(name)
}

/// Целевой модуль импорта в нотации `module:{...}`, совместимой с regex-путём.
fn import_target<'a>(node: Node, lang: TsLanguage, src: &'a [u8]) -> Option<&'a str> {
    match lang {
        TsLanguage::Rust => {
            if node.kind() != "use_declaration" {
                return None;
            }
            let raw = text(node, src).trim();
            let raw = raw
                .strip_prefix("use ")
                .unwrap_or(raw)
                .trim_end_matches(';')
                .trim();
            (!raw.is_empty()).then_some(raw)
        }
        TsLanguage::Python => match node.kind() {
            "import_from_statement" => node
                .child_by_field_name("module_name")
                .map(|n| text(n, src))
                .filter(|s| !s.is_empty()),
            "import_statement" => node
                .child_by_field_name("name")
                .map(|n| text(n, src))
                .filter(|s| !s.is_empty()),
            _ => None,
        },
        TsLanguage::TypeScript | TsLanguage::Tsx | TsLanguage::JavaScript | TsLanguage::Jsx => {
            if node.kind() != "import_statement" {
                return None;
            }
            let raw = node.child_by_field_name("source").map(|n| text(n, src))?;
            let module = strip_quotes(raw);
            (!module.is_empty()).then_some(module)
        }
        TsLanguage::Go => {
            if node.kind() != "import_declaration" {
                return None;
            }
            let spec = find_descendant(node, "import_spec")?;
            let raw = spec.child_by_field_name("path").map(|n| text(n, src))?;
            let module = strip_quotes(raw);
            (!module.is_empty()).then_some(module)
        }
    }
}

/// Текст узла (пустая строка, если срез не UTF-8).
fn text<'a>(node: Node, src: &'a [u8]) -> &'a str {
    node.utf8_text(src).unwrap_or("")
}

/// Снять обрамляющие кавычки строкового литерала.
fn strip_quotes(raw: &str) -> &str {
    raw.trim()
        .trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .trim()
}

/// Компактный однострочный контекст (первая непустая строка, <= 200 символов).
fn snippet(node: Node, src: &[u8]) -> String {
    text(node, src)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .chars()
        .take(200)
        .collect()
}

/// Есть ли среди дочерних узлов узел заданного вида.
fn has_child_kind(node: Node, kind: &str) -> bool {
    let mut cursor = node.walk();
    let mut found = false;
    if cursor.goto_first_child() {
        loop {
            if cursor.node().kind() == kind {
                found = true;
                break;
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    found
}

/// Первый узел-потомок заданного вида (обход в глубину).
fn find_descendant<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut result = None;
    walk(node, |candidate| {
        if result.is_none() && candidate.kind() == kind {
            result = Some(candidate);
        }
    });
    result
}

/// Итеративный пре-обход дерева без рекурсии по стеку.
fn walk<'a>(root: Node<'a>, mut f: impl FnMut(Node<'a>)) {
    let mut cursor = root.walk();
    loop {
        f(cursor.node());
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_selection_by_extension() {
        assert_eq!(TsLanguage::from_path("a.rs"), Some(TsLanguage::Rust));
        assert_eq!(TsLanguage::from_path("a.py"), Some(TsLanguage::Python));
        assert_eq!(TsLanguage::from_path("a.ts"), Some(TsLanguage::TypeScript));
        assert_eq!(TsLanguage::from_path("a.tsx"), Some(TsLanguage::Tsx));
        assert_eq!(TsLanguage::from_path("a.js"), Some(TsLanguage::JavaScript));
        assert_eq!(TsLanguage::from_path("a.jsx"), Some(TsLanguage::Jsx));
        assert_eq!(TsLanguage::from_path("a.go"), Some(TsLanguage::Go));
        assert_eq!(TsLanguage::from_path("a.md"), None);
        assert!(!supports("README.md"));
        assert!(supports("src/lib.rs"));
    }

    fn parse_root<'a>(
        src: &str,
        lang: TsLanguage,
        tree: &'a mut Option<tree_sitter::Tree>,
    ) -> tree_sitter::Node<'a> {
        let mut parser = Parser::new();
        parser.set_language(&lang.grammar()).expect("grammar loads");
        *tree = parser.parse(src, None);
        tree.as_ref().expect("parses").root_node()
    }

    #[test]
    fn valid_source_has_no_errors() {
        let mut tree = None;
        let root = parse_root("fn main() { let x = 1; }\n", TsLanguage::Rust, &mut tree);
        assert_eq!(missing_error_ratio(root), 0.0);
        assert!(!should_fallback(root));
    }

    #[test]
    fn broken_source_triggers_fallback() {
        // Незакрытые скобки/выражение дают ERROR-узлы выше порога.
        let broken = "fn main( { let x = = ; struct { impl }";
        let mut tree = None;
        let root = parse_root(broken, TsLanguage::Rust, &mut tree);
        assert!(missing_error_ratio(root) > FALLBACK_ERROR_RATIO);
        assert!(should_fallback(root));
        assert!(extract("broken.rs", broken).is_none());
    }

    #[test]
    fn extracts_rust_definitions_and_imports_with_syntax_provenance() {
        let src = "use std::collections::HashMap;\n\npub struct Config { pub n: usize }\n\npub fn run(c: &Config) -> usize { c.n }\n";
        let extraction = extract("src/lib.rs", src).expect("extraction succeeds");

        let fn_id = "fn:src/lib.rs:run".to_string();
        let struct_id = "struct:src/lib.rs:Config".to_string();
        assert!(extraction.nodes.iter().any(|n| n.node_id == fn_id));
        assert!(extraction.nodes.iter().any(|n| n.node_id == struct_id));

        let import = extraction
            .edges
            .iter()
            .find(|e| e.label == "IMPORTS")
            .expect("import edge");
        assert_eq!(import.target_node_id, "module:std::collections::HashMap");
        assert_eq!(import.provenance, PROVENANCE_SYNTAX);

        assert!(extraction
            .edges
            .iter()
            .any(|e| e.label == "DEFINES" && e.target_node_id == fn_id && e.provenance == "SYNTAX"));
    }

    #[test]
    fn extracts_python_class_and_function() {
        let src = "from a.b import c\n\nclass Greeter:\n    def hello(self):\n        return c\n";
        let extraction = extract("app/greet.py", src).expect("extraction succeeds");
        assert!(extraction
            .nodes
            .iter()
            .any(|n| n.node_id == "class:app/greet.py:Greeter" && n.node_type == "Class"));
        assert!(extraction
            .nodes
            .iter()
            .any(|n| n.node_id == "fn:app/greet.py:hello"));
        assert!(extraction
            .edges
            .iter()
            .any(|e| e.label == "IMPORTS" && e.target_node_id == "module:a.b"));
    }

    #[test]
    fn extracts_typescript_and_go() {
        let ts = "import { x } from \"./dep\";\ninterface I { a: number }\nclass Impl implements I { a = 1; }\nfunction go(): void {}\n";
        let ts_ext = extract("web/app.ts", ts).expect("ts extraction succeeds");
        assert!(ts_ext
            .nodes
            .iter()
            .any(|n| n.node_id == "interface:web/app.ts:I"));
        assert!(ts_ext
            .nodes
            .iter()
            .any(|n| n.node_id == "class:web/app.ts:Impl"));
        assert!(ts_ext
            .nodes
            .iter()
            .any(|n| n.node_id == "fn:web/app.ts:go"));
        assert!(ts_ext
            .edges
            .iter()
            .any(|e| e.label == "IMPORTS" && e.target_node_id == "module:./dep"));

        let go = "package svc\n\nimport \"fmt\"\n\ntype Server struct { Name string }\n\nfunc Run() { fmt.Println(\"x\") }\n";
        let go_ext = extract("svc/main.go", go).expect("go extraction succeeds");
        assert!(go_ext
            .nodes
            .iter()
            .any(|n| n.node_id == "struct:svc/main.go:Server"));
        assert!(go_ext
            .nodes
            .iter()
            .any(|n| n.node_id == "fn:svc/main.go:Run"));
        assert!(go_ext
            .edges
            .iter()
            .any(|e| e.label == "IMPORTS" && e.target_node_id == "module:fmt"));
    }
}
