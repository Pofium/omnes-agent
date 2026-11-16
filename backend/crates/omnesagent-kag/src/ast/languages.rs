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

    // Assembly (GAS / NASM / MASM)
    asm_include_re: Regex,
    asm_macro_re: Regex,
    asm_equ_re: Regex,
    asm_proc_re: Regex,
    asm_label_re: Regex,
    asm_call_re: Regex,
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

            // Assembly patterns (GAS / NASM / MASM)
            asm_include_re: Regex::new(r#"(?i)^\s*(?:#include|%include|include)\s+["<]?([^">\s]+)[">]?"#).unwrap(),
            asm_macro_re: Regex::new(r#"(?i)^\s*(?:\.macro|%macro)\s+([a-zA-Z_.$?@][a-zA-Z0-9_.$?@]*)|^\s*([a-zA-Z_.$?@][a-zA-Z0-9_.$?@]*)\s+MACRO\b"#).unwrap(),
            asm_equ_re: Regex::new(r#"(?i)^\s*(?:\.equ\s+([a-zA-Z_.$?@][a-zA-Z0-9_.$?@]*)\s*,|%define\s+([a-zA-Z_.$?@][a-zA-Z0-9_.$?@]*)|([a-zA-Z_.$?@][a-zA-Z0-9_.$?@]*)\s+EQU\b)"#).unwrap(),
            asm_proc_re: Regex::new(r#"(?i)^\s*([a-zA-Z_.$?@][a-zA-Z0-9_.$?@]*)\s+PROC\b"#).unwrap(),
            asm_label_re: Regex::new(r#"^\s*([a-zA-Z_.$?@][a-zA-Z0-9_.$?@]*):(?:\s|$|;|#|//)"#).unwrap(),
            asm_call_re: Regex::new(r#"(?i)^\s*(?:callq?|bl|jal)\s+([a-zA-Z_.$?@][a-zA-Z0-9_.$?@]*)"#).unwrap(),
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
                                ..Default::default()
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
                        ..Default::default()
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
                        ..Default::default()
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
                        ..Default::default()
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
                    ..Default::default()
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
                        ..Default::default()
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
                        ..Default::default()
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
                        ..Default::default()
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
                        ..Default::default()
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

    /// Парсит ассемблерные файлы (`.s`, `.S`, `.asm`) со сниффингом диалекта (GAS / NASM / MASM).
    pub fn parse_asm(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let dialect = if rel_path.to_ascii_lowercase().ends_with(".s") {
            "GAS"
        } else if content.contains("%macro") || content.contains("%include") || content.contains("%define") {
            "NASM"
        } else if content.to_ascii_uppercase().contains(" PROC") || content.to_ascii_uppercase().contains(" ENDP") {
            "MASM"
        } else {
            "ASM"
        };

        let mut current_fn_node_id: Option<String> = None;

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with(';')
                || trimmed.starts_with("//")
                || (trimmed.starts_with('#') && !trimmed.starts_with("#include"))
            {
                continue;
            }

            if let Some(cap) = self.asm_include_re.captures(trimmed)
                && let Some(inc) = cap.get(1)
            {
                out.edges.push(AstEdge {
                    source_node_id: file_node_id.to_string(),
                    target_node_id: format!("module:{}", inc.as_str().trim()),
                    label: "IMPORTS".to_string(),
                    weight: 1.0,
                    context: format!("include {}", inc.as_str().trim()),
                    provenance: "SYNTAX".to_string(),
                });
            }

            if let Some(cap) = self.asm_macro_re.captures(trimmed)
                && let Some(name_m) = cap.get(1).or_else(|| cap.get(2))
            {
                let name = name_m.as_str().trim();
                let node_id = format!("macro:{}:{}", rel_path, name);
                out.nodes.push(AstNode {
                    node_id: node_id.clone(),
                    label: name.to_string(),
                    node_type: "Macro".to_string(),
                    description: format!("{dialect} макрос {name} в {rel_path}"),
                    file_path: rel_path.to_string(),
                    line_start: idx + 1,
                    line_end: idx + 1,
                    ..Default::default()
                });
                out.edges.push(AstEdge {
                    source_node_id: file_node_id.to_string(),
                    target_node_id: node_id,
                    label: "DEFINES".to_string(),
                    weight: 1.0,
                    context: format!("{dialect} macro {name}"),
                    provenance: "SYNTAX".to_string(),
                });
                continue;
            }

            if let Some(cap) = self.asm_equ_re.captures(trimmed)
                && let Some(name_m) = cap.get(1).or_else(|| cap.get(2)).or_else(|| cap.get(3))
            {
                let name = name_m.as_str().trim();
                let node_id = format!("data:{}:{}", rel_path, name);
                out.nodes.push(AstNode {
                    node_id: node_id.clone(),
                    label: name.to_string(),
                    node_type: "Data".to_string(),
                    description: format!("{dialect} константа {name} в {rel_path}"),
                    file_path: rel_path.to_string(),
                    line_start: idx + 1,
                    line_end: idx + 1,
                    ..Default::default()
                });
                out.edges.push(AstEdge {
                    source_node_id: file_node_id.to_string(),
                    target_node_id: node_id,
                    label: "DEFINES".to_string(),
                    weight: 1.0,
                    context: format!("{dialect} const {name}"),
                    provenance: "SYNTAX".to_string(),
                });
                continue;
            }

            if let Some(name) = self
                .asm_proc_re
                .captures(trimmed)
                .and_then(|c| c.get(1))
                .or_else(|| self.asm_label_re.captures(trimmed).and_then(|c| c.get(1)))
                .map(|m| m.as_str().trim())
            {
                if is_skipped_asm_label(name) {
                    continue;
                }
                let node_id = format!("fn:{}:{}", rel_path, name);
                out.nodes.push(AstNode {
                    node_id: node_id.clone(),
                    label: name.to_string(),
                    node_type: "Function".to_string(),
                    description: format!("{dialect} процедура {name} в {rel_path}"),
                    file_path: rel_path.to_string(),
                    line_start: idx + 1,
                    line_end: idx + 1,
                    ..Default::default()
                });
                out.edges.push(AstEdge {
                    source_node_id: file_node_id.to_string(),
                    target_node_id: node_id.clone(),
                    label: "DEFINES".to_string(),
                    weight: 1.0,
                    context: format!("{dialect} fn {name}"),
                    provenance: "SYNTAX".to_string(),
                });
                current_fn_node_id = Some(node_id);
                continue;
            }

            if let Some(cap) = self.asm_call_re.captures(trimmed)
                && let Some(target_m) = cap.get(1)
            {
                let target = target_m.as_str().trim();
                if is_skipped_asm_call_target(target) {
                    continue;
                }
                let caller = current_fn_node_id.as_deref().unwrap_or(file_node_id);
                out.edges.push(AstEdge {
                    source_node_id: caller.to_string(),
                    target_node_id: format!("fn:{}", target),
                    label: "CALLS".to_string(),
                    weight: 1.0,
                    context: format!("call {target}"),
                    provenance: "HEURISTIC".to_string(),
                });
            }
        }
    }
}

fn is_skipped_asm_label(name: &str) -> bool {
    name.starts_with(".L")
        || name.ends_with(".cold")
        || name.starts_with("__x86.get_pc_thunk.")
        || name.chars().all(|c| c.is_ascii_digit())
}

fn is_skipped_asm_call_target(target: &str) -> bool {
    matches!(
        target.to_ascii_lowercase().as_str(),
        "rax" | "rbx" | "rcx" | "rdx" | "rsi" | "rdi" | "rbp" | "rsp"
            | "r8" | "r9" | "r10" | "r11" | "r12" | "r13" | "r14" | "r15"
            | "eax" | "ebx" | "ecx" | "edx" | "esi" | "edi" | "ebp" | "esp"
            | "dword" | "qword" | "ptr"
    ) || is_skipped_asm_label(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_asm_extracts_gas_nasm_masm_symbols_and_skips_local_labels() {
        let ext = LanguageExtractor::new();
        let mut res = AstScanResult::default();
        let gas = r#"
#include "arch_defs.h"
.macro SAVE_REGS
.endm
.equ STACK_SZ, 64
.Llocal_loop:
__memcpy_avx:
    call helper_fn
    call .Llocal_loop
"#;
        ext.parse_asm("arch/memcpy.S", "file:arch/memcpy.S", gas, &mut res);
        let labels: Vec<&str> = res.nodes.iter().map(|n| n.label.as_str()).collect();
        assert!(labels.contains(&"SAVE_REGS"));
        assert!(labels.contains(&"STACK_SZ"));
        assert!(labels.contains(&"__memcpy_avx"));
        assert!(!labels.contains(&".Llocal_loop"));
        assert!(res.edges.iter().any(|e| e.label == "IMPORTS" && e.target_node_id == "module:arch_defs.h"));
        assert!(res.edges.iter().any(|e| e.label == "CALLS" && e.target_node_id == "fn:helper_fn"));

        let mut masm_res = AstScanResult::default();
        let masm = "my_proc PROC\n  call fast_crc32\nmy_proc ENDP\n";
        ext.parse_asm("src/crc.asm", "file:src/crc.asm", masm, &mut masm_res);
        assert!(masm_res.nodes.iter().any(|n| n.label == "my_proc" && n.node_type == "Function"));
        assert!(masm_res.edges.iter().any(|e| e.label == "CALLS" && e.target_node_id == "fn:fast_crc32"));
    }
}
