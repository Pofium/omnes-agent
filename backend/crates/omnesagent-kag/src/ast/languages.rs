//! AST-экстрактор для специализированных и современных языков:
//! - Jupyter Notebooks (.ipynb)
//! - Solidity (.sol)
//! - Zig (.zig)

use super::{AstEdge, AstNode, AstScanResult};
use regex::Regex;
use serde_json::Value;

pub struct LanguageExtractor {
    // Solidity
    sol_import_re: Regex,
    sol_contract_re: Regex,
    sol_interface_re: Regex,
    sol_library_re: Regex,
    sol_fn_re: Regex,
    sol_event_re: Regex,
    sol_modifier_re: Regex,

    // Zig
    zig_import_re: Regex,
    zig_fn_re: Regex,
    zig_type_re: Regex,
}

impl Default for LanguageExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageExtractor {
    pub fn new() -> Self {
        Self {
            // Solidity patterns
            sol_import_re: Regex::new(r#"(?m)^\s*import\s+(?:(?:\{[^}]+\}|\*\s+as\s+([a-zA-Z0-9_]+))\s+from\s+)?["']([^"']+)["']\s*;"#).unwrap(),
            sol_contract_re: Regex::new(r#"(?m)^\s*(?:abstract\s+)?contract\s+([a-zA-Z0-9_]+)(?:\s+is\s+([^{]+))?\s*\{"#).unwrap(),
            sol_interface_re: Regex::new(r#"(?m)^\s*interface\s+([a-zA-Z0-9_]+)(?:\s+is\s+([^{]+))?\s*\{"#).unwrap(),
            sol_library_re: Regex::new(r#"(?m)^\s*library\s+([a-zA-Z0-9_]+)\s*\{"#).unwrap(),
            sol_fn_re: Regex::new(r#"(?m)^\s*(?:function|constructor|fallback|receive)\s*([a-zA-Z0-9_]*)\s*\(([^)]*)\)"#).unwrap(),
            sol_event_re: Regex::new(r#"(?m)^\s*event\s+([a-zA-Z0-9_]+)\s*\(([^)]*)\)"#).unwrap(),
            sol_modifier_re: Regex::new(r#"(?m)^\s*modifier\s+([a-zA-Z0-9_]+)\s*\(([^)]*)\)"#).unwrap(),

            // Zig patterns
            zig_import_re: Regex::new(r#"(?m)^\s*(?:pub\s+)?const\s+([a-zA-Z0-9_]+)\s*=\s*@import\(["']([^"']+)["']\)\s*;"#).unwrap(),
            zig_fn_re: Regex::new(r#"(?m)^\s*(?:pub\s+)?(?:export\s+)?(?:inline\s+)?fn\s+([a-zA-Z0-9_]+)\s*\(([^)]*)\)"#).unwrap(),
            zig_type_re: Regex::new(r#"(?m)^\s*(?:pub\s+)?const\s+([a-zA-Z0-9_]+)\s*=\s*(?:packed\s+|extern\s+)?(struct|enum|union|error)\b"#).unwrap(),
        }
    }

    /// Парсит Jupyter Notebooks (.ipynb) с извлечением кода ячеек.
    pub fn parse_ipynb<F>(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        mut parse_py_fn: F,
        out: &mut AstScanResult,
    ) where
        F: FnMut(&str, &str, &str, &mut AstScanResult),
    {
        if let Ok(json_val) = serde_json::from_str::<Value>(content) {
            if let Some(cells) = json_val.get("cells").and_then(|c| c.as_array()) {
                let mut cell_index = 0;
                for cell in cells {
                    let cell_type = cell.get("cell_type").and_then(|t| t.as_str()).unwrap_or("");
                    if cell_type == "code" {
                        cell_index += 1;
                        let mut code_buf = String::new();
                        if let Some(src_arr) = cell.get("source").and_then(|s| s.as_array()) {
                            for line in src_arr {
                                if let Some(line_str) = line.as_str() {
                                    code_buf.push_str(line_str);
                                }
                            }
                        } else if let Some(src_str) = cell.get("source").and_then(|s| s.as_str()) {
                            code_buf.push_str(src_str);
                        }

                        if !code_buf.trim().is_empty() {
                            let cell_node_id = format!("ipynb_cell:{}:{}", rel_path, cell_index);
                            out.nodes.push(AstNode {
                                node_id: cell_node_id.clone(),
                                label: format!("Cell [{}]", cell_index),
                                node_type: "Module".to_string(),
                                description: format!("Jupyter блокнот ячейка #{} в {}", cell_index, rel_path),
                                file_path: rel_path.to_string(),
                                line_start: cell_index,
                                line_end: cell_index,
                            });

                            out.edges.push(AstEdge {
                                source_node_id: file_node_id.to_string(),
                                target_node_id: cell_node_id.clone(),
                                label: "DEFINES".to_string(),
                                weight: 1.0,
                                context: format!("code cell #{}", cell_index),
                                provenance: "EXTRACTED".to_string(),
                            });

                            // Прогоняем Python код ячейки через основной Python парсер
                            parse_py_fn(rel_path, &cell_node_id, &code_buf, out);
                        }
                    }
                }
            }
        }
    }

    /// Парсит смарт-контракты Solidity (.sol).
    pub fn parse_solidity(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let lines: Vec<&str> = content.lines().collect();

        // 1. imports
        for line in lines.iter() {
            if let Some(cap) = self.sol_import_re.captures(line) {
                let target = cap.get(2).map(|m| m.as_str()).or_else(|| cap.get(1).map(|m| m.as_str())).unwrap_or("");
                if !target.is_empty() {
                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: format!("file:{}", target),
                        label: "IMPORTS".to_string(),
                        weight: 1.0,
                        context: format!("import \"{}\"", target),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }

        // 2. contracts
        for (idx, line) in lines.iter().enumerate() {
            if let Some(cap) = self.sol_contract_re.captures(line) {
                if let Some(c_name) = cap.get(1) {
                    let name = c_name.as_str().trim();
                    let node_id = format!("sol_contract:{}", name);

                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: name.to_string(),
                        node_type: "Contract".to_string(),
                        description: format!("Solidity контракт {} в {}", name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id.clone(),
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("contract {}", name),
                        provenance: "EXTRACTED".to_string(),
                    });

                    // Наследование (contract A is B, C)
                    if let Some(inherits) = cap.get(2) {
                        for parent in inherits.as_str().split(',') {
                            let clean_parent = parent.trim();
                            if !clean_parent.is_empty() {
                                out.edges.push(AstEdge {
                                    source_node_id: node_id.clone(),
                                    target_node_id: format!("sol_contract:{}", clean_parent),
                                    label: "INHERITS".to_string(),
                                    weight: 1.0,
                                    context: format!("is {}", clean_parent),
                                    provenance: "EXTRACTED".to_string(),
                                });
                            }
                        }
                    }
                }
            }

            // 3. interfaces
            if let Some(cap) = self.sol_interface_re.captures(line) {
                if let Some(i_name) = cap.get(1) {
                    let name = i_name.as_str().trim();
                    let node_id = format!("sol_iface:{}", name);

                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: name.to_string(),
                        node_type: "Interface".to_string(),
                        description: format!("Solidity интерфейс {} в {}", name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("interface {}", name),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }

            // 4. libraries
            if let Some(cap) = self.sol_library_re.captures(line) {
                if let Some(l_name) = cap.get(1) {
                    let name = l_name.as_str().trim();
                    let node_id = format!("sol_lib:{}", name);

                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: name.to_string(),
                        node_type: "Module".to_string(),
                        description: format!("Solidity библиотека {} в {}", name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("library {}", name),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }

            // 5. functions
            if let Some(cap) = self.sol_fn_re.captures(line) {
                let name = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let label = if name.is_empty() {
                    "constructor/fallback".to_string()
                } else {
                    name.to_string()
                };

                let node_id = format!("sol_fn:{}:{}", rel_path, idx + 1);
                out.nodes.push(AstNode {
                    node_id: node_id.clone(),
                    label: label.clone(),
                    node_type: "Function".to_string(),
                    description: format!("Solidity функция {} в {}", label, rel_path),
                    file_path: rel_path.to_string(),
                    line_start: idx + 1,
                    line_end: idx + 1,
                });

                out.edges.push(AstEdge {
                    source_node_id: file_node_id.to_string(),
                    target_node_id: node_id,
                    label: "DEFINES".to_string(),
                    weight: 1.0,
                    context: format!("function {}", label),
                    provenance: "EXTRACTED".to_string(),
                });
            }

            // 6. events
            if let Some(cap) = self.sol_event_re.captures(line) {
                if let Some(ev) = cap.get(1) {
                    let ev_name = ev.as_str().trim();
                    let node_id = format!("sol_event:{}:{}", rel_path, ev_name);

                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: ev_name.to_string(),
                        node_type: "Event".to_string(),
                        description: format!("Solidity событие {} в {}", ev_name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("event {}", ev_name),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }

            // 7. modifiers
            if let Some(cap) = self.sol_modifier_re.captures(line) {
                if let Some(m) = cap.get(1) {
                    let mod_name = m.as_str().trim();
                    let node_id = format!("sol_mod:{}:{}", rel_path, mod_name);

                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: mod_name.to_string(),
                        node_type: "Modifier".to_string(),
                        description: format!("Solidity модификатор {} в {}", mod_name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("modifier {}", mod_name),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }
    }

    /// Парсит исходный код Zig (.zig).
    pub fn parse_zig(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let lines: Vec<&str> = content.lines().collect();

        // 1. @import
        for line in lines.iter() {
            if let Some(cap) = self.zig_import_re.captures(line) {
                if let (Some(alias), Some(target)) = (cap.get(1), cap.get(2)) {
                    let target_str = target.as_str().trim();
                    let alias_str = alias.as_str().trim();

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: format!("module:{}", target_str),
                        label: "IMPORTS".to_string(),
                        weight: 1.0,
                        context: format!("const {} = @import(\"{}\")", alias_str, target_str),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }

        // 2. types (struct, enum, union, error)
        for (idx, line) in lines.iter().enumerate() {
            if let Some(cap) = self.zig_type_re.captures(line) {
                if let (Some(t_name), Some(kind)) = (cap.get(1), cap.get(2)) {
                    let name = t_name.as_str().trim();
                    let kind_str = kind.as_str().trim();
                    let node_type = match kind_str {
                        "struct" => "Struct",
                        "enum" => "Enum",
                        "union" => "Type",
                        _ => "Type",
                    };

                    let node_id = format!("zig_type:{}:{}", rel_path, name);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: name.to_string(),
                        node_type: node_type.to_string(),
                        description: format!("Zig {} {} в {}", kind_str, name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("const {} = {}", name, kind_str),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }

            // 3. functions
            if let Some(cap) = self.zig_fn_re.captures(line) {
                if let Some(fn_m) = cap.get(1) {
                    let fn_name = fn_m.as_str().trim();
                    let node_id = format!("zig_fn:{}:{}", rel_path, fn_name);

                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: fn_name.to_string(),
                        node_type: "Function".to_string(),
                        description: format!("Zig функция {} в {}", fn_name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("fn {}()", fn_name),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }
    }
}
