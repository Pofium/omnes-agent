//! Ф6.1 (ob2h v2.0, гибридный AST): tree-sitter extraction.
//!
//! Feature-gated behind `treesitter`. Парсит Rust/TS/JS/Python/Go и
//! дополнительные языки грамматиками, вендоренными через `inkjet` (одна
//! версия tree-sitter в воркспейсе, без отдельных crate-грамматик — бюджет
//! грамматик удержан), и эмитит узлы/рёбра графа с провенансом об2h v2.0
//! (`SYNTAX`/`RESOLVED`/`HEURISTIC`).
//!
//! Помимо `DEFINES`/`IMPORTS`-рёбер экстрактор собирает места вызовов
//! ([`CallSite`]) и импорты с алиасами ([`ImportRef`]) для последующего
//! глобального type-pass'а ([`super::resolve_calls`]) — тот же контракт, что
//! использует regex-экстрактор.
//!
//! Если доля ERROR/MISSING-узлов в дереве превышает [`FALLBACK_ERROR_RATIO`],
//! [`extract`] возвращает `None`, и вызывающий код отступает на regex-экстрактор.

use super::{AstEdge, AstNode, CallSite, ImportRef};
use lru::LruCache;
use regex::Regex;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use tree_sitter::{Language, Node, Parser};

static TREE_CACHE: LazyLock<Mutex<LruCache<String, (String, tree_sitter::Tree)>>> = LazyLock::new(|| {
    Mutex::new(LruCache::new(NonZeroUsize::new(64).unwrap()))
});

static RUST_MACRO_CALL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*\("#).unwrap()
});

static SFC_SCRIPT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<script[^>]*>(.*?)</script>"#).unwrap()
});

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
    Java,
    Kotlin,
    CSharp,
    C,
    Cpp,
    Php,
    Ruby,
    Swift,
    Scala,
    Lua,
    Dart,
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
            "java" => Some(Self::Java),
            "kt" | "kts" => Some(Self::Kotlin),
            "cs" => Some(Self::CSharp),
            "c" | "h" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Self::Cpp),
            "php" => Some(Self::Php),
            "rb" => Some(Self::Ruby),
            "swift" => Some(Self::Swift),
            "scala" => Some(Self::Scala),
            "lua" => Some(Self::Lua),
            "dart" => Some(Self::Dart),
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
            Self::Java => inkjet::Language::Java,
            Self::Kotlin => inkjet::Language::Kotlin,
            Self::CSharp => inkjet::Language::CSharp,
            Self::C => inkjet::Language::C,
            Self::Cpp => inkjet::Language::Cpp,
            Self::Php => inkjet::Language::Php,
            Self::Ruby => inkjet::Language::Ruby,
            Self::Swift => inkjet::Language::Swift,
            Self::Scala => inkjet::Language::Scala,
            Self::Lua => inkjet::Language::Lua,
            Self::Dart => inkjet::Language::Dart,
        };
        lang.config().language.clone()
    }
}

/// Результат разбора одного файла.
pub struct TsExtraction {
    pub nodes: Vec<AstNode>,
    pub edges: Vec<AstEdge>,
    /// Места вызовов для type-pass ([`super::resolve_calls`]).
    pub call_sites: Vec<CallSite>,
    /// Импорты с алиасами для type-pass ([`super::resolve_calls`]).
    pub imports: Vec<ImportRef>,
}

/// Поддерживается ли файл tree-sitter экстрактором.
pub fn supports(rel_path: &str) -> bool {
    let lower = rel_path.to_ascii_lowercase();
    if lower.ends_with(".vue") || lower.ends_with(".svelte") {
        return true;
    }
    if super::is_generated_stub_file(rel_path, "") {
        return true;
    }
    TsLanguage::from_path(rel_path).is_some()
}

fn compute_input_edit(old_str: &str, new_str: &str) -> Option<tree_sitter::InputEdit> {
    if old_str == new_str {
        return None;
    }
    let prefix_len = old_str.bytes().zip(new_str.bytes()).take_while(|(a, b)| a == b).count();
    let old_rem = &old_str[prefix_len..];
    let new_rem = &new_str[prefix_len..];
    let suffix_len = old_rem.bytes().rev().zip(new_rem.bytes().rev()).take_while(|(a, b)| a == b).count();

    let start_byte = prefix_len;
    let old_end_byte = old_str.len().saturating_sub(suffix_len);
    let new_end_byte = new_str.len().saturating_sub(suffix_len);

    let start_position = byte_to_point(old_str, start_byte);
    let old_end_position = byte_to_point(old_str, old_end_byte);
    let new_end_position = byte_to_point(new_str, new_end_byte);

    Some(tree_sitter::InputEdit {
        start_byte,
        old_end_byte,
        new_end_byte,
        start_position,
        old_end_position,
        new_end_position,
    })
}

fn byte_to_point(s: &str, byte_idx: usize) -> tree_sitter::Point {
    let prefix = &s[..byte_idx.min(s.len())];
    let row = prefix.matches('\n').count();
    let col = prefix.rfind('\n').map(|pos| prefix.len() - pos - 1).unwrap_or(prefix.len());
    tree_sitter::Point { row, column: col }
}

/// Разобрать `content` и извлечь определения и импорты.
pub fn extract(rel_path: &str, content: &str) -> Option<TsExtraction> {
    if super::is_generated_stub_file(rel_path, content) {
        let file_node_id = format!("file:{}", rel_path);
        let label = rel_path.to_string();
        return Some(TsExtraction {
            nodes: vec![AstNode {
                node_id: file_node_id,
                label: label.clone(),
                node_type: "FileStub".to_string(),
                description: format!("Сгенерированный stub-файл: {}", rel_path),
                file_path: rel_path.to_string(),
                line_start: 1,
                line_end: content.lines().count().max(1),
                kind_detail: "generated_stub".to_string(),
                subtokens: super::split_subtokens(&label),
            }],
            edges: Vec::new(),
            call_sites: Vec::new(),
            imports: Vec::new(),
        });
    }

    let lower = rel_path.to_ascii_lowercase();
    if lower.ends_with(".vue") || lower.ends_with(".svelte") {
        return extract_sfc(rel_path, content);
    }

    let lang = TsLanguage::from_path(rel_path)?;
    extract_with_lang(rel_path, content, lang, 0)
}

fn extract_sfc(rel_path: &str, content: &str) -> Option<TsExtraction> {
    let mut combined = TsExtraction {
        nodes: Vec::new(),
        edges: Vec::new(),
        call_sites: Vec::new(),
        imports: Vec::new(),
    };
    let mut found = false;
    for cap in SFC_SCRIPT_RE.captures_iter(content) {
        let Some(m) = cap.get(1) else { continue };
        let offset = content[..m.start()].matches('\n').count();
        if let Some(sub) = extract_with_lang(rel_path, m.as_str(), TsLanguage::TypeScript, offset) {
            combined.nodes.extend(sub.nodes);
            combined.edges.extend(sub.edges);
            combined.call_sites.extend(sub.call_sites);
            combined.imports.extend(sub.imports);
            found = true;
        }
    }
    if found {
        Some(combined)
    } else {
        None
    }
}

fn extract_with_lang(rel_path: &str, content: &str, lang: TsLanguage, line_offset: usize) -> Option<TsExtraction> {
    let mut parser = Parser::new();
    parser.set_language(&lang.grammar()).ok()?;

    let cache_key = format!("{}:{}", rel_path, line_offset);
    let mut old_tree_opt = None;
    if let Ok(mut lock) = TREE_CACHE.lock() {
        if let Some((old_content, mut tree)) = lock.pop(&cache_key) {
            if old_content == content {
                let root = tree.root_node();
                let res = if should_fallback(root) {
                    None
                } else {
                    Some(collect_with_offset(rel_path, content.as_bytes(), root, lang, line_offset))
                };
                lock.put(cache_key, (content.to_string(), tree));
                return res;
            } else if let Some(edit) = compute_input_edit(&old_content, content) {
                tree.edit(&edit);
                old_tree_opt = Some(tree);
            }
        }
    }

    let tree = parser.parse(content, old_tree_opt.as_ref())?;
    let root = tree.root_node();
    if should_fallback(root) {
        return None;
    }

    if let Ok(mut lock) = TREE_CACHE.lock() {
        lock.put(cache_key, (content.to_string(), tree.clone()));
    }

    Some(collect_with_offset(rel_path, content.as_bytes(), root, lang, line_offset))
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

fn is_rust_test_node(node: Node, src: &[u8]) -> bool {
    let mut cursor = node.prev_sibling();
    while let Some(prev) = cursor {
        if prev.kind() == "attribute_item" {
            let txt = text(prev, src);
            if txt.contains("test") {
                return true;
            }
            cursor = prev.prev_sibling();
        } else {
            break;
        }
    }
    false
}

/// Собрать узлы/рёбра из дерева разбора.
fn collect(rel_path: &str, src: &[u8], root: Node, lang: TsLanguage) -> TsExtraction {
    collect_with_offset(rel_path, src, root, lang, 0)
}

fn collect_with_offset(
    rel_path: &str,
    src: &[u8],
    root: Node,
    lang: TsLanguage,
    line_offset: usize,
) -> TsExtraction {
    let file_node_id = format!("file:{}", rel_path);
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut call_sites = Vec::new();
    let mut imports = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    walk(root, |node| {
        if let Some((prefix, node_type)) = definition_kind(node, lang) {
            if let Some(name) = node_name(node, lang, src) {
                let node_id = format!("{}:{}:{}", prefix, rel_path, name);
                if seen.insert(node_id.clone()) {
                    let is_test = super::is_test_symbol(name, rel_path)
                        || (lang == TsLanguage::Rust && is_rust_test_node(node, src));
                    let kind_detail = if is_test { "test".to_string() } else { String::new() };
                    let subtokens = super::split_subtokens(name);

                    nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: name.to_string(),
                        node_type: node_type.to_string(),
                        description: format!("{} `{}` в {}", node_type, name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: node.start_position().row + 1 + line_offset,
                        line_end: node.end_position().row + 1 + line_offset,
                        kind_detail,
                        subtokens,
                    });
                    edges.push(AstEdge {
                        source_node_id: file_node_id.clone(),
                        target_node_id: node_id.clone(),
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: snippet(node, src),
                        provenance: PROVENANCE_SYNTAX.to_string(),
                    });
                }
                // Ф6.1: рёбра наследования/реализации контейнеров (best-effort).
                for (label, base) in inherit_targets(node, lang, src) {
                    let target = if label == "IMPLEMENTS" {
                        format!("interface:{}", base)
                    } else {
                        format!("class:{}", base)
                    };
                    edges.push(AstEdge {
                        source_node_id: node_id.clone(),
                        target_node_id: target,
                        label: label.to_string(),
                        weight: 1.0,
                        context: format!("{} {}", label.to_ascii_lowercase(), base),
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

        // Ф6.1: места вызовов и импорты с алиасами для глобального type-pass.
        if let Some(name) = call_callee(node, lang, src) {
            call_sites.push(CallSite {
                rel_path: rel_path.to_string(),
                line: node.start_position().row + 1 + line_offset,
                name: name.to_string(),
            });
        }
        if node.kind() == "macro_invocation" && lang == TsLanguage::Rust {
            if let Some(token_tree) = node.child_by_field_name("token_tree") {
                let tree_bytes = &src[token_tree.start_byte()..token_tree.end_byte()];
                if let Ok(tree_str) = std::str::from_utf8(tree_bytes) {
                    for cap in RUST_MACRO_CALL_RE.captures_iter(tree_str) {
                        if let Some(m) = cap.get(1) {
                            let fn_name = m.as_str();
                            if !matches!(fn_name, "println" | "format" | "vec" | "info" | "error" | "warn" | "debug" | "trace" | "write" | "writeln" | "panic" | "assert" | "assert_eq") {
                                call_sites.push(CallSite {
                                    rel_path: rel_path.to_string(),
                                    line: token_tree.start_position().row + 1 + line_offset,
                                    name: fn_name.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        let refs = import_refs(node, lang, rel_path, src);
        if !refs.is_empty() {
            imports.extend(refs);
        }
        // Ф6.1: ROUTE-ребро (framework-эвристика) для Python-декораторов.
        if let Some((source, target)) = route_edge(node, lang, rel_path, src) {
            edges.push(AstEdge {
                source_node_id: source,
                target_node_id: target,
                label: "ROUTE".to_string(),
                weight: 1.0,
                context: "route".to_string(),
                provenance: PROVENANCE_HEURISTIC.to_string(),
            });
        }
    });

    TsExtraction {
        nodes,
        edges,
        call_sites,
        imports,
    }
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
                // `const f = () => {}` / `const f = function () {}`
                "variable_declarator" if ts_variable_is_function(node) => Some(("fn", "Function")),
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
        TsLanguage::Java => match node.kind() {
            "method_declaration" | "constructor_declaration" => Some(("fn", "Function")),
            "class_declaration" => Some(("class", "Class")),
            "interface_declaration" => Some(("interface", "Interface")),
            "enum_declaration" => Some(("struct", "Struct")),
            _ => None,
        },
        TsLanguage::Kotlin => match node.kind() {
            "function_declaration" => Some(("fn", "Function")),
            "class_declaration" | "object_declaration" => Some(("class", "Class")),
            _ => None,
        },
        TsLanguage::CSharp => match node.kind() {
            "method_declaration" | "constructor_declaration" => Some(("fn", "Function")),
            "class_declaration" => Some(("class", "Class")),
            "interface_declaration" => Some(("interface", "Interface")),
            "struct_declaration" | "record_declaration" => Some(("struct", "Struct")),
            _ => None,
        },
        TsLanguage::C | TsLanguage::Cpp => match node.kind() {
            "function_definition" => Some(("fn", "Function")),
            "class_specifier" => Some(("class", "Class")),
            "struct_specifier" => Some(("struct", "Struct")),
            _ => None,
        },
        TsLanguage::Php => match node.kind() {
            "function_definition" | "method_declaration" => Some(("fn", "Function")),
            "class_declaration" => Some(("class", "Class")),
            "interface_declaration" => Some(("interface", "Interface")),
            "trait_declaration" => Some(("trait", "Trait")),
            _ => None,
        },
        TsLanguage::Ruby => match node.kind() {
            "method" | "singleton_method" => Some(("fn", "Function")),
            "class" => Some(("class", "Class")),
            "module" => Some(("module", "Module")),
            _ => None,
        },
        TsLanguage::Swift => match node.kind() {
            "function_declaration" | "protocol_function_declaration" | "init_declaration"
            | "deinit_declaration" => Some(("fn", "Function")),
            "class_declaration" => {
                // Swift объединяет class/struct/enum в один узел; вид уточняется
                // ключевым словом в поле `declaration_kind`.
                match node.child_by_field_name("declaration_kind").map(|n| n.kind()) {
                    Some("struct") | Some("enum") => Some(("struct", "Struct")),
                    _ => Some(("class", "Class")),
                }
            }
            "protocol_declaration" => Some(("interface", "Interface")),
            _ => None,
        },
        TsLanguage::Scala => match node.kind() {
            "function_definition" | "function_declaration" => Some(("fn", "Function")),
            "class_definition" => Some(("class", "Class")),
            "object_definition" => Some(("module", "Module")),
            "trait_definition" => Some(("trait", "Trait")),
            _ => None,
        },
        TsLanguage::Lua => match node.kind() {
            "function_statement" => Some(("fn", "Function")),
            _ => None,
        },
        TsLanguage::Dart => match node.kind() {
            "function_signature" | "method_signature" => Some(("fn", "Function")),
            "class_definition" => Some(("class", "Class")),
            // Dart-миксин (`mixin RouteAware { ... }`) объявляет типаж.
            "mixin_declaration" => Some(("trait", "Trait")),
            _ => None,
        },
    }
}

/// Имя объявления: поле `name`, иначе язык-специфичный фолбэк.
fn node_name<'a>(node: Node, lang: TsLanguage, src: &'a [u8]) -> Option<&'a str> {
    // Lua: поле `name` может содержать `M.new`/`M:greet` — берём последний
    // сегмент, чтобы имя совпадало с regex-путём (`new`/`greet`).
    if lang == TsLanguage::Lua && node.kind() == "function_statement" {
        let n = node.child_by_field_name("name")?;
        let t = text_opt(n, src)?;
        return Some(t.rsplit(|c| c == '.' || c == ':').next().unwrap_or(t));
    }
    if let Some(n) = node.child_by_field_name("name") {
        if let Some(t) = text_opt(n, src) {
            return Some(t);
        }
    }
    match lang {
        // Kotlin: имя — прямой потомок `type_identifier`/`simple_identifier`
        // без поля `name`.
        TsLanguage::Kotlin => direct_child_of_kinds(node, &["type_identifier", "simple_identifier"], src),
        // C/C++: имя функции лежит в `declarator` (вложенные declarator-ы для
        // указателей/ссылок).
        TsLanguage::C | TsLanguage::Cpp => {
            if node.kind() == "function_definition" {
                node.child_by_field_name("declarator")
                    .and_then(|d| declarator_name(d, src))
            } else {
                None
            }
        }
        // Dart: `method_signature` оборачивает `function_signature`, у которой
        // есть поле `name`; у `mixin_declaration` поля `name` нет — берём
        // прямой `identifier`.
        TsLanguage::Dart => match node.kind() {
            "method_signature" => {
                let mut c = node.walk();
                node.children(&mut c)
                    .find(|n| n.kind() == "function_signature")
                    .and_then(|n| node_name(n, lang, src))
            }
            "mixin_declaration" => direct_child_of_kinds(node, &["identifier"], src),
            _ => None,
        },
        _ => None,
    }
}

/// Рекурсивный спуск по полю `declarator` до идентификатора (C/C++).
///
/// Обрабатывает указатели/ссылки (`pointer_declarator` и т.п.) и
/// out-of-line-методы с квалификатором (`UserService::getUser` →
/// `qualified_identifier`).
fn declarator_name<'a>(mut node: Node, src: &'a [u8]) -> Option<&'a str> {
    loop {
        match node.kind() {
            "identifier" | "field_identifier" | "type_identifier" => {
                return text_opt(node, src);
            }
            // `UserService::getUser` → берём последний сегмент (`name`).
            "qualified_identifier" => {
                return node
                    .child_by_field_name("name")
                    .and_then(|n| text_opt(n, src));
            }
            _ => {
                node = node.child_by_field_name("declarator")?;
            }
        }
    }
}

/// Первый прямой потомок одного из указанных видов.
fn direct_child_of_kinds<'a>(
    node: Node,
    kinds: &[&str],
    src: &'a [u8],
) -> Option<&'a str> {
    let mut c = node.walk();
    node.children(&mut c)
        .find(|n| kinds.contains(&n.kind()))
        .and_then(|n| text_opt(n, src))
}

/// Является ли TS/JS `variable_declarator` объявлением функции
/// (`const f = () => {}` / `const f = function () {}`).
fn ts_variable_is_function(node: Node) -> bool {
    node.child_by_field_name("value")
        .map(|v| {
            matches!(
                v.kind(),
                "arrow_function"
                    | "function"
                    | "function_expression"
                    | "generator_function"
                    | "generator_function_declaration"
            )
        })
        .unwrap_or(false)
}

/// Прямые потомки-узлы, виды которых входят в `kinds`.
fn direct_children_of_kinds<'a>(node: Node<'a>, kinds: &[&str]) -> Vec<Node<'a>> {
    let mut c = node.walk();
    node.children(&mut c)
        .filter(|n| kinds.contains(&n.kind()))
        .collect()
}

/// Узлы-потомки заданного вида в порядке пре-обхода (без самого `node`).
fn descendants_of_kind<'a>(node: Node<'a>, kind: &str) -> Vec<Node<'a>> {
    let root_id = node.id();
    let mut out = Vec::new();
    walk(node, |n| {
        if n.id() != root_id && n.kind() == kind {
            out.push(n);
        }
    });
    out
}

/// Тексты прямых потомков заданных видов → `(метка, текст)`.
fn push_child_texts<'a>(
    parent: Node,
    kinds: &[&str],
    label: &'static str,
    src: &'a [u8],
    out: &mut Vec<(&'static str, &'a str)>,
) {
    for n in direct_children_of_kinds(parent, kinds) {
        if let Some(t) = text_opt(n, src) {
            out.push((label, t));
        }
    }
}

/// Тексты узлов-потомков заданных видов → `(метка, текст)`.
fn push_desc_texts<'a>(
    parent: Node,
    kinds: &[&str],
    label: &'static str,
    src: &'a [u8],
    out: &mut Vec<(&'static str, &'a str)>,
) {
    let root_id = parent.id();
    walk(parent, |n| {
        if n.id() != root_id && kinds.contains(&n.kind()) {
            if let Some(t) = text_opt(n, src) {
                out.push((label, t));
            }
        }
    });
}

/// Рёбра наследования/реализации контейнера (best-effort, SYNTAX-провенанс).
///
/// Первый базовый тип трактуется как наследование (`INHERITS`), последующие —
/// как реализация интерфейсов (`IMPLEMENTS`), где грамматика их различает;
/// в остальных языках используется явная семантика (extends/implements).
fn inherit_targets<'a>(node: Node, lang: TsLanguage, src: &'a [u8]) -> Vec<(&'static str, &'a str)> {
    let mut out: Vec<(&'static str, &'a str)> = Vec::new();
    match lang {
        TsLanguage::Python => {
            if node.kind() == "class_definition" {
                if let Some(sc) = node.child_by_field_name("superclasses") {
                    push_child_texts(sc, &["identifier", "attribute"], "INHERITS", src, &mut out);
                }
            }
        }
        TsLanguage::Php => {
            if matches!(node.kind(), "class_declaration" | "interface_declaration") {
                if let Some(bc) = direct_child_of_kind(node, "base_clause") {
                    push_child_texts(bc, &["name"], "INHERITS", src, &mut out);
                }
                if let Some(ic) = direct_child_of_kind(node, "class_interface_clause") {
                    push_child_texts(ic, &["name"], "IMPLEMENTS", src, &mut out);
                }
            }
        }
        TsLanguage::Java => {
            if node.kind() == "class_declaration" {
                if let Some(sc) = node.child_by_field_name("superclass") {
                    push_desc_texts(sc, &["type_identifier"], "INHERITS", src, &mut out);
                }
                if let Some(si) = node.child_by_field_name("interfaces") {
                    push_desc_texts(si, &["type_identifier"], "IMPLEMENTS", src, &mut out);
                }
            }
        }
        TsLanguage::Dart => {
            if node.kind() == "class_definition" {
                if let Some(sc) = node.child_by_field_name("superclass") {
                    push_desc_texts(sc, &["type_identifier"], "INHERITS", src, &mut out);
                }
                if let Some(ifaces) = node.child_by_field_name("interfaces") {
                    push_desc_texts(ifaces, &["type_identifier"], "IMPLEMENTS", src, &mut out);
                }
            }
        }
        TsLanguage::Scala => {
            if node.kind() == "class_definition" {
                let ec = node
                    .child_by_field_name("extend")
                    .or_else(|| direct_child_of_kind(node, "extends_clause"));
                if let Some(ec) = ec {
                    for (i, t) in descendants_of_kind(ec, "type_identifier").iter().enumerate() {
                        if let Some(txt) = text_opt(*t, src) {
                            let label = if i == 0 { "INHERITS" } else { "IMPLEMENTS" };
                            out.push((label, txt));
                        }
                    }
                }
            }
        }
        TsLanguage::CSharp => {
            if node.kind() == "class_declaration" {
                if let Some(bl) = direct_child_of_kind(node, "base_list") {
                    for (i, t) in direct_children_of_kinds(bl, &["identifier"]).iter().enumerate() {
                        if let Some(txt) = text_opt(*t, src) {
                            let label = if i == 0 { "INHERITS" } else { "IMPLEMENTS" };
                            out.push((label, txt));
                        }
                    }
                }
            }
        }
        TsLanguage::Kotlin => {
            if node.kind() == "class_declaration" {
                for (i, spec) in direct_children_of_kinds(node, &["delegation_specifier"])
                    .iter()
                    .enumerate()
                {
                    if let Some(t) = find_descendant(*spec, "type_identifier") {
                        if let Some(txt) = text_opt(t, src) {
                            let label = if i == 0 { "INHERITS" } else { "IMPLEMENTS" };
                            out.push((label, txt));
                        }
                    }
                }
            }
        }
        TsLanguage::Swift => {
            if node.kind() == "class_declaration" {
                for (i, spec) in direct_children_of_kinds(node, &["inheritance_specifier"])
                    .iter()
                    .enumerate()
                {
                    if let Some(t) = find_descendant(*spec, "type_identifier") {
                        if let Some(txt) = text_opt(t, src) {
                            let label = if i == 0 { "INHERITS" } else { "IMPLEMENTS" };
                            out.push((label, txt));
                        }
                    }
                }
            }
        }
        TsLanguage::C | TsLanguage::Cpp => {
            if node.kind() == "class_specifier" {
                if let Some(bc) = direct_child_of_kind(node, "base_class_clause") {
                    push_desc_texts(bc, &["type_identifier"], "INHERITS", src, &mut out);
                }
            }
        }
        TsLanguage::Ruby => {
            if node.kind() == "class" {
                if let Some(sc) = node.child_by_field_name("superclass") {
                    push_desc_texts(sc, &["constant"], "INHERITS", src, &mut out);
                }
            }
        }
        _ => {}
    }
    out
}

/// Вербы HTTP-роутов, распознаваемые в декораторах (Python).
const ROUTE_VERBS: &[&str] = &[
    "get", "post", "put", "delete", "patch", "options", "head",
];

/// ROUTE-ребро для Python-декоратора: `(source_id, "route:{METHOD} {PATH}")`.
fn route_edge(
    node: Node,
    lang: TsLanguage,
    rel_path: &str,
    src: &[u8],
) -> Option<(String, String)> {
    if lang != TsLanguage::Python || node.kind() != "decorator" {
        return None;
    }
    let call = find_descendant(node, "call")?;
    let func = call.child_by_field_name("function")?;
    if func.kind() != "attribute" {
        return None;
    }
    let method = func
        .child_by_field_name("attribute")
        .and_then(|n| text_opt(n, src))?;
    if !ROUTE_VERBS.contains(&method) {
        return None;
    }
    let args = call.child_by_field_name("arguments")?;
    let path_node = find_descendant(args, "string")?;
    let path = strip_quotes(text(path_node, src));
    if path.is_empty() {
        return None;
    }
    let defn = node.parent()?.child_by_field_name("definition")?;
    let fname = defn.child_by_field_name("name").and_then(|n| text_opt(n, src))?;
    Some((
        format!("fn:{}:{}", rel_path, fname),
        format!("route:{} {}", method.to_ascii_uppercase(), path),
    ))
}

/// Целевой модуль импорта в нотации `module:{...}`, совместимой с regex-путём.
fn import_target(node: Node, lang: TsLanguage, src: &[u8]) -> Option<String> {
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
            (!raw.is_empty()).then(|| raw.to_string())
        }
        TsLanguage::Python => match node.kind() {
            "import_from_statement" => node
                .child_by_field_name("module_name")
                .map(|n| text(n, src).to_string())
                .filter(|s| !s.is_empty()),
            "import_statement" => node
                .child_by_field_name("name")
                .map(|n| text(n, src).to_string())
                .filter(|s| !s.is_empty()),
            _ => None,
        },
        TsLanguage::TypeScript | TsLanguage::Tsx | TsLanguage::JavaScript | TsLanguage::Jsx => {
            if node.kind() != "import_statement" {
                return None;
            }
            let raw = node.child_by_field_name("source").map(|n| text(n, src))?;
            let module = strip_quotes(raw);
            (!module.is_empty()).then(|| module.to_string())
        }
        TsLanguage::Go => {
            if node.kind() != "import_declaration" {
                return None;
            }
            let spec = find_descendant(node, "import_spec")?;
            let raw = spec.child_by_field_name("path").map(|n| text(n, src))?;
            let module = strip_quotes(raw);
            (!module.is_empty()).then(|| module.to_string())
        }
        // --- Ф6.1: расширение языков — импорт-рёбра (best-effort) ---
        TsLanguage::Java => {
            if node.kind() != "import_declaration" {
                return None;
            }
            let mut c = node.walk();
            node.children(&mut c)
                .find(|n| n.kind() == "scoped_identifier")
                .map(|n| text(n, src).to_string())
                .filter(|s| !s.is_empty())
        }
        TsLanguage::Kotlin => {
            if node.kind() != "import_header" {
                return None;
            }
            direct_child_of_kinds(node, &["identifier"], src).map(str::to_string)
        }
        TsLanguage::CSharp => {
            if node.kind() != "using_directive" {
                return None;
            }
            let mut c = node.walk();
            node.children(&mut c)
                .find(|n| matches!(n.kind(), "identifier" | "qualified_name"))
                .map(|n| text(n, src).to_string())
                .filter(|s| !s.is_empty())
        }
        TsLanguage::Php => {
            if node.kind() != "namespace_use_declaration" {
                return None;
            }
            find_descendant(node, "qualified_name")
                .map(|n| text(n, src).to_string())
                .filter(|s| !s.is_empty())
        }
        TsLanguage::Scala => {
            if node.kind() != "import_declaration" {
                return None;
            }
            let mut c = node.walk();
            let parts: Vec<&str> = node
                .children(&mut c)
                .filter(|n| n.kind() == "identifier")
                .map(|n| text(n, src))
                .collect();
            (!parts.is_empty()).then(|| parts.join("."))
        }
        TsLanguage::Swift => {
            if node.kind() != "import_declaration" {
                return None;
            }
            let mut c = node.walk();
            node.children(&mut c)
                .find(|n| n.kind() == "identifier")
                .map(|n| text(n, src).to_string())
                .filter(|s| !s.is_empty())
        }
        TsLanguage::Dart => {
            if node.kind() != "library_import" {
                return None;
            }
            let lit = find_descendant(node, "string_literal")?;
            let module = strip_quotes(text(lit, src));
            (!module.is_empty()).then(|| module.to_string())
        }
        TsLanguage::Ruby => {
            if node.kind() != "call" {
                return None;
            }
            let method = node.child_by_field_name("method").and_then(|n| text_opt(n, src))?;
            if method != "require" && method != "require_relative" {
                return None;
            }
            let args = node.child_by_field_name("arguments")?;
            let lit = find_descendant(args, "string")?;
            let module = strip_quotes(text(lit, src));
            (!module.is_empty()).then(|| module.to_string())
        }
        TsLanguage::Lua => {
            if node.kind() != "function_call" {
                return None;
            }
            let prefix = node.child_by_field_name("prefix").and_then(|n| text_opt(n, src))?;
            if prefix != "require" {
                return None;
            }
            let args = node.child_by_field_name("args")?;
            let lit = find_descendant(args, "string")?;
            let module = find_descendant(lit, "string_content")
                .map(|n| text(n, src))
                .unwrap_or_else(|| strip_quotes(text(lit, src)));
            (!module.is_empty()).then(|| module.to_string())
        }
        TsLanguage::C | TsLanguage::Cpp => {
            if node.kind() != "preproc_include" {
                return None;
            }
            let raw = node.child_by_field_name("path").map(|n| text(n, src))?;
            let module = raw
                .trim()
                .trim_matches(|c| c == '<' || c == '>' || c == '"' || c == '\'')
                .trim();
            (!module.is_empty()).then(|| module.to_string())
        }
    }
}

/// Простое имя вызываемого (AMBIGUOUS-safe): только однозначные callee.
///
/// Возвращает `None` для языков без реализованного извлечения вызовов и для
/// неоднозначных форм (произвольный receiver, атрибуты, макросы и т.п.).
fn call_callee<'a>(node: Node, lang: TsLanguage, src: &'a [u8]) -> Option<&'a str> {
    match lang {
        TsLanguage::Rust => {
            if node.kind() != "call_expression" {
                return None;
            }
            node.child_by_field_name("function")
                .and_then(|f| rust_callee(f, src))
        }
        TsLanguage::Python => {
            if node.kind() != "call" {
                return None;
            }
            let f = node.child_by_field_name("function")?;
            match f.kind() {
                "identifier" => text_opt(f, src),
                // self.m()/cls.m() → метод того же файла; chained .m() → метод; произвольный receiver — AMBIGUOUS.
                "attribute" => {
                    let obj = f.child_by_field_name("object")?;
                    let obj_text = text(obj, src);
                    if obj_text == "self" || obj_text == "cls" || obj.kind() == "call" || obj.kind() == "await" {
                        f.child_by_field_name("attribute").and_then(|n| text_opt(n, src))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        TsLanguage::TypeScript | TsLanguage::Tsx | TsLanguage::JavaScript | TsLanguage::Jsx => {
            if node.kind() != "call_expression" {
                return None;
            }
            let f = node.child_by_field_name("function")?;
            match f.kind() {
                "identifier" => text_opt(f, src),
                // this.m() → метод того же класса; chained .m() → метод; прочее — AMBIGUOUS.
                "member_expression" => {
                    let obj = f.child_by_field_name("object")?;
                    let obj_t = text(obj, src);
                    if obj_t == "this" || obj.kind() == "call_expression" || obj.kind() == "await_expression" {
                        f.child_by_field_name("property").and_then(|n| text_opt(n, src))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        TsLanguage::Go => {
            if node.kind() != "call_expression" {
                return None;
            }
            let f = node.child_by_field_name("function")?;
            match f.kind() {
                "identifier" => text_opt(f, src),
                // pkg.Fn() — берём правую часть селектора.
                "selector_expression" => {
                    f.child_by_field_name("field").and_then(|n| text_opt(n, src))
                }
                _ => None,
            }
        }
        // Для остальных языков вызовы не извлекаем (только определения).
        _ => None,
    }
}

/// Rust: callee call_expression — идентификатор, scoped/generic функция либо
/// `self./Self.`-метод или chained вызов (`builder.foo().bar()`).
fn rust_callee<'a>(f: Node, src: &'a [u8]) -> Option<&'a str> {
    match f.kind() {
        "identifier" => text_opt(f, src),
        "scoped_identifier" => f.child_by_field_name("name").and_then(|n| text_opt(n, src)),
        "generic_function" => f
            .child_by_field_name("function")
            .and_then(|n| rust_callee(n, src)),
        "field_expression" => {
            let val = f.child_by_field_name("value")?;
            let val_text = text(val, src);
            if val_text == "self" || val_text == "Self" || val.kind() == "call_expression" || val.kind() == "await_expression" {
                f.child_by_field_name("field").and_then(|n| text_opt(n, src))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Импорты с алиасами для type-pass (общий диспетчер).
fn import_refs(node: Node, lang: TsLanguage, rel_path: &str, src: &[u8]) -> Vec<ImportRef> {
    let mut out = Vec::new();
    match lang {
        TsLanguage::Rust => {
            if node.kind() == "use_declaration" {
                rust_use_refs(node, rel_path, src, &mut out);
            }
        }
        TsLanguage::Python => python_import_refs(node, rel_path, src, &mut out),
        TsLanguage::TypeScript | TsLanguage::Tsx | TsLanguage::JavaScript | TsLanguage::Jsx => {
            ts_import_refs(node, rel_path, src, &mut out);
        }
        TsLanguage::Go => go_import_refs(node, rel_path, src, &mut out),
        _ => {}
    }
    out
}

/// Rust `use`-декларация → набор [`ImportRef`] (best-effort).
fn rust_use_refs(node: Node, rel_path: &str, src: &[u8], out: &mut Vec<ImportRef>) {
    let Some(arg) = node.child_by_field_name("argument") else {
        return;
    };
    match arg.kind() {
        "scoped_identifier" => {
            let module = arg
                .child_by_field_name("path")
                .map(|n| text(n, src))
                .unwrap_or("");
            let item = arg
                .child_by_field_name("name")
                .and_then(|n| text_opt(n, src))
                .map(str::to_string);
            out.push(ImportRef {
                rel_path: rel_path.to_string(),
                module: module.to_string(),
                item,
                alias: None,
            });
        }
        "identifier" => {
            out.push(ImportRef {
                rel_path: rel_path.to_string(),
                module: String::new(),
                item: text_opt(arg, src).map(str::to_string),
                alias: None,
            });
        }
        "use_as_clause" => {
            let alias = arg
                .child_by_field_name("alias")
                .and_then(|n| text_opt(n, src))
                .map(str::to_string);
            if let Some(p) = arg.child_by_field_name("path") {
                let (module, item) = scoped_parts(p, src);
                out.push(ImportRef {
                    rel_path: rel_path.to_string(),
                    module,
                    item,
                    alias,
                });
            }
        }
        "scoped_use_list" => {
            let module = arg
                .child_by_field_name("path")
                .map(|n| text(n, src))
                .unwrap_or("");
            if let Some(list) = arg.child_by_field_name("list") {
                rust_use_list_refs(list, module, rel_path, src, out);
            }
        }
        "use_list" => {
            rust_use_list_refs(arg, "", rel_path, src, out);
        }
        _ => {}
    }
}

/// Элементы Rust `use_list` (`use a::b::{c as d, e};`).
fn rust_use_list_refs(
    list: Node,
    module: &str,
    rel_path: &str,
    src: &[u8],
    out: &mut Vec<ImportRef>,
) {
    let mut c = list.walk();
    for item in list.children(&mut c) {
        match item.kind() {
            "use_as_clause" => {
                let alias = item
                    .child_by_field_name("alias")
                    .and_then(|n| text_opt(n, src))
                    .map(str::to_string);
                if let Some(p) = item.child_by_field_name("path") {
                    let (m, it) = scoped_parts(p, src);
                    out.push(ImportRef {
                        rel_path: rel_path.to_string(),
                        module: if m.is_empty() {
                            module.to_string()
                        } else {
                            m
                        },
                        item: it,
                        alias,
                    });
                }
            }
            "identifier" => {
                out.push(ImportRef {
                    rel_path: rel_path.to_string(),
                    module: module.to_string(),
                    item: text_opt(item, src).map(str::to_string),
                    alias: None,
                });
            }
            _ => {}
        }
    }
}

/// Разложить scoped-путь (`a::b::c`) на (module, item).
fn scoped_parts(p: Node, src: &[u8]) -> (String, Option<String>) {
    if p.kind() == "scoped_identifier" {
        let module = p
            .child_by_field_name("path")
            .map(|n| text(n, src))
            .unwrap_or("")
            .to_string();
        let item = p
            .child_by_field_name("name")
            .and_then(|n| text_opt(n, src))
            .map(str::to_string);
        (module, item)
    } else {
        (String::new(), text_opt(p, src).map(str::to_string))
    }
}

/// Python `import`/`from ... import` → [`ImportRef`].
fn python_import_refs(node: Node, rel_path: &str, src: &[u8], out: &mut Vec<ImportRef>) {
    match node.kind() {
        "import_from_statement" => {
            let Some(mn) = node.child_by_field_name("module_name") else {
                return;
            };
            let module = text(mn, src).to_string();
            if module.is_empty() {
                return;
            }
            let mut c = node.walk();
            if !c.goto_first_child() {
                return;
            }
            loop {
                // `name`-поле исключает сам `module_name` (его поле — module_name).
                if c.field_name() == Some("name") {
                    let child = c.node();
                    match child.kind() {
                        "dotted_name" => out.push(ImportRef {
                            rel_path: rel_path.to_string(),
                            module: module.clone(),
                            item: text_opt(child, src).map(str::to_string),
                            alias: None,
                        }),
                        "aliased_import" => out.push(ImportRef {
                            rel_path: rel_path.to_string(),
                            module: module.clone(),
                            item: child
                                .child_by_field_name("name")
                                .and_then(|n| text_opt(n, src))
                                .map(str::to_string),
                            alias: child
                                .child_by_field_name("alias")
                                .and_then(|n| text_opt(n, src))
                                .map(str::to_string),
                        }),
                        _ => {}
                    }
                }
                if !c.goto_next_sibling() {
                    break;
                }
            }
        }
        "import_statement" => {
            let mut c = node.walk();
            if !c.goto_first_child() {
                return;
            }
            loop {
                if c.field_name() == Some("name") {
                    let child = c.node();
                    match child.kind() {
                        "dotted_name" => {
                            if let Some(m) = text_opt(child, src) {
                                out.push(ImportRef {
                                    rel_path: rel_path.to_string(),
                                    module: m.to_string(),
                                    item: None,
                                    alias: None,
                                });
                            }
                        }
                        "aliased_import" => {
                            let module = child
                                .child_by_field_name("name")
                                .and_then(|n| text_opt(n, src))
                                .unwrap_or("")
                                .to_string();
                            if !module.is_empty() {
                                out.push(ImportRef {
                                    rel_path: rel_path.to_string(),
                                    module,
                                    item: None,
                                    alias: child
                                        .child_by_field_name("alias")
                                        .and_then(|n| text_opt(n, src))
                                        .map(str::to_string),
                                });
                            }
                        }
                        _ => {}
                    }
                }
                if !c.goto_next_sibling() {
                    break;
                }
            }
        }
        _ => {}
    }
}

/// TS/JS `import ... from "..."` → [`ImportRef`] на каждую связываемую сущность.
fn ts_import_refs(node: Node, rel_path: &str, src: &[u8], out: &mut Vec<ImportRef>) {
    if node.kind() != "import_statement" {
        return;
    }
    let Some(source) = node.child_by_field_name("source") else {
        return;
    };
    let module = strip_quotes(text(source, src)).to_string();
    if module.is_empty() {
        return;
    }
    let Some(clause) = direct_child_of_kind(node, "import_clause") else {
        return;
    };
    let mut c = clause.walk();
    for child in clause.children(&mut c) {
        match child.kind() {
            // default import: `import X from ...`
            "identifier" => {
                if let Some(name) = text_opt(child, src) {
                    out.push(ImportRef {
                        rel_path: rel_path.to_string(),
                        module: module.clone(),
                        item: Some(name.to_string()),
                        alias: None,
                    });
                }
            }
            // `import { a, b as c } from ...`
            "named_imports" => {
                let mut c2 = child.walk();
                for spec in child.children(&mut c2) {
                    if spec.kind() == "import_specifier" {
                        out.push(ImportRef {
                            rel_path: rel_path.to_string(),
                            module: module.clone(),
                            item: spec
                                .child_by_field_name("name")
                                .and_then(|n| text_opt(n, src))
                                .map(str::to_string),
                            alias: spec
                                .child_by_field_name("alias")
                                .and_then(|n| text_opt(n, src))
                                .map(str::to_string),
                        });
                    }
                }
            }
            // `import * as ns from ...`
            "namespace_import" => {
                if let Some(ns) = direct_child_of_kind(child, "identifier") {
                    if let Some(name) = text_opt(ns, src) {
                        out.push(ImportRef {
                            rel_path: rel_path.to_string(),
                            module: module.clone(),
                            item: Some(name.to_string()),
                            alias: None,
                        });
                    }
                }
            }
            _ => {}
        }
    }
}

/// Go `import` → [`ImportRef`] на каждый import_spec.
fn go_import_refs(node: Node, rel_path: &str, src: &[u8], out: &mut Vec<ImportRef>) {
    if node.kind() != "import_declaration" {
        return;
    }
    let mut specs = Vec::new();
    walk(node, |n| {
        if n.kind() == "import_spec" {
            specs.push(n);
        }
    });
    for spec in specs {
        let Some(path) = spec.child_by_field_name("path") else {
            continue;
        };
        let module = strip_quotes(text(path, src));
        if !module.is_empty() {
            out.push(ImportRef {
                rel_path: rel_path.to_string(),
                module: module.to_string(),
                item: None,
                alias: None,
            });
        }
    }
}

/// Текст узла (пустая строка, если срез не UTF-8).
fn text<'a>(node: Node, src: &'a [u8]) -> &'a str {
    node.utf8_text(src).unwrap_or("")
}

/// Текст узла как `Option` (None для пустого или не-UTF-8 среза).
fn text_opt<'a>(node: Node, src: &'a [u8]) -> Option<&'a str> {
    node.utf8_text(src).ok().filter(|s| !s.is_empty())
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

/// Первый прямой потомок заданного вида.
fn direct_child_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find(|n| n.kind() == kind)
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
        assert_eq!(TsLanguage::from_path("a.java"), Some(TsLanguage::Java));
        assert_eq!(TsLanguage::from_path("a.kt"), Some(TsLanguage::Kotlin));
        assert_eq!(TsLanguage::from_path("a.kts"), Some(TsLanguage::Kotlin));
        assert_eq!(TsLanguage::from_path("a.cs"), Some(TsLanguage::CSharp));
        assert_eq!(TsLanguage::from_path("a.c"), Some(TsLanguage::C));
        assert_eq!(TsLanguage::from_path("a.h"), Some(TsLanguage::C));
        assert_eq!(TsLanguage::from_path("a.cpp"), Some(TsLanguage::Cpp));
        assert_eq!(TsLanguage::from_path("a.hpp"), Some(TsLanguage::Cpp));
        assert_eq!(TsLanguage::from_path("a.php"), Some(TsLanguage::Php));
        assert_eq!(TsLanguage::from_path("a.rb"), Some(TsLanguage::Ruby));
        assert_eq!(TsLanguage::from_path("a.swift"), Some(TsLanguage::Swift));
        assert_eq!(TsLanguage::from_path("a.scala"), Some(TsLanguage::Scala));
        assert_eq!(TsLanguage::from_path("a.lua"), Some(TsLanguage::Lua));
        assert_eq!(TsLanguage::from_path("a.dart"), Some(TsLanguage::Dart));
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

    // --- Ф6.1 remainder: call sites + import refs + type-pass integration ---

    #[test]
    fn rust_call_sites_resolve_same_file() {
        let src = "fn a() {\n    b();\n}\n\nfn b() {}\n";
        let extraction = extract("src/lib.rs", src).expect("extraction succeeds");
        assert!(extraction
            .call_sites
            .iter()
            .any(|c| c.name == "b" && c.line == 2));
        // self.m()/Self.m() и macro_invocation не попадают в call sites.
        let src2 = "struct S;\nimpl S {\n    fn m(&self) {\n        self.helper();\n        other.x();\n        m!();\n    }\n    fn helper(&self) {}\n}\n";
        let e2 = extract("src/s.rs", src2).expect("extraction succeeds");
        let names: Vec<&str> = e2.call_sites.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"helper"));
        assert!(!names.contains(&"x"), "arbitrary receiver must be skipped");
        assert!(!names.contains(&"m"), "macro invocation must be skipped");

        // End-to-end: type-pass добавляет same-file CALLS с провенансом RESOLVED.
        let scan = crate::ast::AstScanResult {
            nodes: extraction.nodes,
            edges: extraction.edges,
            call_sites: extraction.call_sites,
            imports: extraction.imports,
            ..Default::default()
        };
        let mut scan = scan;
        let added = crate::ast::resolve_calls(&mut scan);
        assert!(added >= 1, "type-pass must add a CALLS edge");
        let edge = scan
            .edges
            .iter()
            .find(|e| e.label == "CALLS")
            .expect("CALLS edge");
        assert_eq!(edge.source_node_id, "fn:src/lib.rs:a");
        assert_eq!(edge.target_node_id, "fn:src/lib.rs:b");
        assert_eq!(edge.provenance, "RESOLVED");
    }

    #[test]
    fn rust_import_alias_resolves_via_imports() {
        let main = "use a::b::c as d;\n\nfn run() {\n    d();\n}\n";
        let callee = "pub fn c() {}\n";
        let m = extract("src/main.rs", main).expect("main parses");
        let c = extract("a/b.rs", callee).expect("callee parses");

        let mut scan = crate::ast::AstScanResult {
            nodes: m.nodes.into_iter().chain(c.nodes).collect(),
            edges: m.edges.into_iter().chain(c.edges).collect(),
            call_sites: m.call_sites,
            imports: m.imports,
            ..Default::default()
        };
        let added = crate::ast::resolve_calls(&mut scan);
        assert_eq!(added, 1, "alias import must resolve to the target symbol");
        let edge = scan
            .edges
            .iter()
            .find(|e| e.label == "CALLS")
            .expect("CALLS edge");
        assert_eq!(edge.source_node_id, "fn:src/main.rs:run");
        assert_eq!(edge.target_node_id, "fn:a/b.rs:c");
        assert_eq!(edge.provenance, "RESOLVED");
    }

    #[test]
    fn python_call_sites_and_alias_import() {
        let src = "from a.b import c as d\n\nclass K:\n    def run(self):\n        d()\n        self.helper()\n";
        let e = extract("app/m.py", src).expect("extraction succeeds");
        let names: Vec<&str> = e.call_sites.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"d"));
        assert!(names.contains(&"helper"), "self.helper() must be recorded");
        assert!(e
            .imports
            .iter()
            .any(|i| i.module == "a.b" && i.item.as_deref() == Some("c") && i.alias.as_deref() == Some("d")));

        let callee = "def c():\n    pass\n";
        let cc = extract("a/b.py", callee).expect("callee parses");
        let mut scan = crate::ast::AstScanResult {
            nodes: e.nodes.into_iter().chain(cc.nodes).collect(),
            edges: e.edges.into_iter().chain(cc.edges).collect(),
            call_sites: e.call_sites,
            imports: e.imports,
            ..Default::default()
        };
        let added = crate::ast::resolve_calls(&mut scan);
        assert!(added >= 1, "alias import must resolve");
        assert!(scan
            .edges
            .iter()
            .any(|e| e.label == "CALLS" && e.target_node_id == "fn:a/b.py:c"));
    }

    #[test]
    fn typescript_call_sites_and_relative_import() {
        let ts = "import { x } from \"./dep\";\nfunction f() {\n    x();\n    this.m();\n}\n";
        let e = extract("web/app.ts", ts).expect("extraction succeeds");
        let names: Vec<&str> = e.call_sites.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"x"));
        assert!(names.contains(&"m"), "this.m() must be recorded");
        assert!(e
            .imports
            .iter()
            .any(|i| i.module == "./dep" && i.item.as_deref() == Some("x")));

        let dep = "export function x(): void {}\n";
        let d = extract("web/dep.ts", dep).expect("dep parses");
        let mut scan = crate::ast::AstScanResult {
            nodes: e.nodes.into_iter().chain(d.nodes).collect(),
            edges: e.edges.into_iter().chain(d.edges).collect(),
            call_sites: e.call_sites,
            imports: e.imports,
            ..Default::default()
        };
        let added = crate::ast::resolve_calls(&mut scan);
        assert!(added >= 1, "relative TS import must resolve");
        assert!(scan
            .edges
            .iter()
            .any(|e| e.label == "CALLS" && e.target_node_id == "fn:web/dep.ts:x"));
    }

    #[test]
    fn go_call_sites_and_import_ref() {
        let go = "package svc\n\nimport \"fmt\"\n\nfunc Run() {\n    helper()\n    fmt.Println(\"x\")\n}\n";
        let e = extract("svc/main.go", go).expect("extraction succeeds");
        assert!(e.call_sites.iter().any(|c| c.name == "helper"));
        assert!(e.call_sites.iter().any(|c| c.name == "Println"));
        assert!(e.imports.iter().any(|i| i.module == "fmt" && i.item.is_none()));
    }

    #[test]
    fn extracts_java_definitions() {
        let src = "package a;\n\npublic class Foo {\n    void bar() { return 1; }\n}\n\ninterface I {\n    void m();\n}\n";
        let e = extract("src/Foo.java", src).expect("extraction succeeds");
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "class:src/Foo.java:Foo" && n.node_type == "Class"));
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "fn:src/Foo.java:bar" && n.node_type == "Function"));
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "interface:src/Foo.java:I" && n.node_type == "Interface"));
    }

    #[test]
    fn extracts_csharp_definitions() {
        let src = "namespace N {\n    class Foo {\n        int Bar() { return 1; }\n    }\n    interface I { void M(); }\n    struct S { }\n}\n";
        let e = extract("src/Foo.cs", src).expect("extraction succeeds");
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "class:src/Foo.cs:Foo" && n.node_type == "Class"));
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "fn:src/Foo.cs:Bar" && n.node_type == "Function"));
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "interface:src/Foo.cs:I" && n.node_type == "Interface"));
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "struct:src/Foo.cs:S" && n.node_type == "Struct"));
    }

    #[test]
    fn extracts_ruby_definitions() {
        let src = "module M\n  class Foo\n    def bar; end\n  end\nend\n\ndef top; end\n";
        let e = extract("lib/foo.rb", src).expect("extraction succeeds");
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "module:lib/foo.rb:M" && n.node_type == "Module"));
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "class:lib/foo.rb:Foo" && n.node_type == "Class"));
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "fn:lib/foo.rb:bar" && n.node_type == "Function"));
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "fn:lib/foo.rb:top" && n.node_type == "Function"));
    }

    #[test]
    fn extracts_php_definitions() {
        let src = "<?php\nclass Foo {\n    function bar() {}\n}\ninterface I { public function m(); }\ntrait T {}\nfunction top() {}\n";
        let e = extract("src/Foo.php", src).expect("extraction succeeds");
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "class:src/Foo.php:Foo" && n.node_type == "Class"));
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "fn:src/Foo.php:bar" && n.node_type == "Function"));
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "interface:src/Foo.php:I" && n.node_type == "Interface"));
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "trait:src/Foo.php:T" && n.node_type == "Trait"));
        assert!(e
            .nodes
            .iter()
            .any(|n| n.node_id == "fn:src/Foo.php:top" && n.node_type == "Function"));
    }

    #[test]
    fn extracts_c_cpp_kotlin_swift_scala_lua_dart_definitions() {
        let c = extract("src/a.c", "struct P { int x; };\nint add(int a) { return a; }\n")
            .expect("c parses");
        assert!(c.nodes.iter().any(|n| n.node_id == "struct:src/a.c:P"));
        assert!(c.nodes.iter().any(|n| n.node_id == "fn:src/a.c:add"));

        let cpp = extract(
            "src/a.cpp",
            "class Foo { public: int bar() { return 1; } };\nint add(int a) { return a; }\n",
        )
        .expect("cpp parses");
        assert!(cpp
            .nodes
            .iter()
            .any(|n| n.node_id == "class:src/a.cpp:Foo" && n.node_type == "Class"));
        assert!(cpp.nodes.iter().any(|n| n.node_id == "fn:src/a.cpp:bar"));
        assert!(cpp.nodes.iter().any(|n| n.node_id == "fn:src/a.cpp:add"));

        let kt = extract(
            "src/A.kt",
            "package a.b\nclass Foo { fun bar() {} }\nfun top() {}\nobject Obj {}\n",
        )
        .expect("kotlin parses");
        assert!(kt
            .nodes
            .iter()
            .any(|n| n.node_id == "class:src/A.kt:Foo" && n.node_type == "Class"));
        assert!(kt.nodes.iter().any(|n| n.node_id == "fn:src/A.kt:bar"));
        assert!(kt.nodes.iter().any(|n| n.node_id == "fn:src/A.kt:top"));
        assert!(kt.nodes.iter().any(|n| n.node_id == "class:src/A.kt:Obj"));

        let swift = extract(
            "src/A.swift",
            "class Foo { func bar() {} }\nstruct S {}\nprotocol P { func m() }\nfunc top() {}\n",
        )
        .expect("swift parses");
        assert!(swift
            .nodes
            .iter()
            .any(|n| n.node_id == "class:src/A.swift:Foo" && n.node_type == "Class"));
        assert!(swift
            .nodes
            .iter()
            .any(|n| n.node_id == "struct:src/A.swift:S" && n.node_type == "Struct"));
        assert!(swift
            .nodes
            .iter()
            .any(|n| n.node_id == "interface:src/A.swift:P" && n.node_type == "Interface"));
        assert!(swift.nodes.iter().any(|n| n.node_id == "fn:src/A.swift:top"));

        let scala = extract(
            "src/A.scala",
            "class Foo { def bar(): Int = 1 }\nobject Obj {}\ntrait T { def m(): Unit }\ndef top(): Int = 1\n",
        )
        .expect("scala parses");
        assert!(scala.nodes.iter().any(|n| n.node_id == "class:src/A.scala:Foo"));
        assert!(scala
            .nodes
            .iter()
            .any(|n| n.node_id == "module:src/A.scala:Obj" && n.node_type == "Module"));
        assert!(scala
            .nodes
            .iter()
            .any(|n| n.node_id == "trait:src/A.scala:T" && n.node_type == "Trait"));
        assert!(scala.nodes.iter().any(|n| n.node_id == "fn:src/A.scala:top"));

        let lua = extract("src/a.lua", "function bar() end\nlocal function baz() end\n")
            .expect("lua parses");
        assert!(lua.nodes.iter().any(|n| n.node_id == "fn:src/a.lua:bar"));
        assert!(lua.nodes.iter().any(|n| n.node_id == "fn:src/a.lua:baz"));

        let dart = extract("src/a.dart", "class Foo { void bar() {} }\nint top() => 1;\n")
            .expect("dart parses");
        assert!(dart
            .nodes
            .iter()
            .any(|n| n.node_id == "class:src/a.dart:Foo" && n.node_type == "Class"));
        assert!(dart.nodes.iter().any(|n| n.node_id == "fn:src/a.dart:bar"));
        assert!(dart.nodes.iter().any(|n| n.node_id == "fn:src/a.dart:top"));
    }
}
