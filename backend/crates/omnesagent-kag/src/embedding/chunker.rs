//! AST-aware code chunking for neural vector embeddings.
//!
//! Chunks source code along AST node boundaries (functions, classes, structs, traits)
//! with contextual headers (`// File: {path} | Scope: {scope}`), ensuring semantic coherence
//! and improved retrieval recall compared to fixed-window chunking.

use serde::{Deserialize, Serialize};

/// Represents an AST-delimited code chunk with contextual metadata and blake3 content hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeChunk {
    pub file_path: String,
    pub scope: String,
    pub line_start: usize,
    pub line_end: usize,
    pub text: String,
    pub blake3_hash: String,
}

/// Chunks a source file using its extracted AST nodes.
///
/// Falls back to whole-file or windowed chunks if no AST nodes are provided.
pub fn chunk_file_ast(
    file_path: &str,
    content: &str,
    nodes: &[crate::ast::AstNode],
) -> Vec<CodeChunk> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();

    // Filter significant nodes that represent clear semantic blocks
    let mut significant_nodes: Vec<&crate::ast::AstNode> = nodes
        .iter()
        .filter(|n| {
            matches!(
                n.node_type.as_str(),
                "Function" | "Method" | "Class" | "Struct" | "Trait" | "Interface" | "Enum"
            ) && n.line_start > 0
                && n.line_end >= n.line_start
        })
        .collect();

    // Sort by line_start
    significant_nodes.sort_by_key(|n| n.line_start);

    if significant_nodes.is_empty() {
        // Fallback: whole file chunk or 100-line sliding window
        let chunk_lines = lines.len().min(120);
        let body = lines[..chunk_lines].join("\n");
        let header = format!("// File: {}\n", file_path);
        let full_text = format!("{}{}", header, body);
        let hash = blake3::hash(full_text.as_bytes()).to_hex().to_string();

        chunks.push(CodeChunk {
            file_path: file_path.to_string(),
            scope: "File".to_string(),
            line_start: 1,
            line_end: chunk_lines,
            text: full_text,
            blake3_hash: hash,
        });
        return chunks;
    }

    for node in significant_nodes {
        let start_idx = node.line_start.saturating_sub(1);
        let end_idx = node.line_end.min(lines.len());

        if start_idx >= lines.len() || start_idx >= end_idx {
            continue;
        }

        let body = lines[start_idx..end_idx].join("\n");
        let scope_str = format!("{} {}", node.node_type, node.label);
        let header = format!("// File: {} | Scope: {}\n", file_path, scope_str);
        let full_text = format!("{}{}", header, body);
        let hash = blake3::hash(full_text.as_bytes()).to_hex().to_string();

        chunks.push(CodeChunk {
            file_path: file_path.to_string(),
            scope: scope_str,
            line_start: node.line_start,
            line_end: node.line_end,
            text: full_text,
            blake3_hash: hash,
        });
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::AstNode;

    #[test]
    fn chunks_source_by_ast_nodes_with_headers() {
        let code = r#"
pub struct User {
    pub id: u64,
    pub name: String,
}

pub fn authenticate(user: &User) -> bool {
    !user.name.is_empty()
}
"#;

        let nodes = vec![
            AstNode {
                node_id: "struct:User".to_string(),
                label: "User".to_string(),
                node_type: "Struct".to_string(),
                description: String::new(),
                file_path: "src/user.rs".to_string(),
                line_start: 2,
                line_end: 5,
                ..Default::default()
            },
            AstNode {
                node_id: "fn:authenticate".to_string(),
                label: "authenticate".to_string(),
                node_type: "Function".to_string(),
                description: String::new(),
                file_path: "src/user.rs".to_string(),
                line_start: 7,
                line_end: 9,
                ..Default::default()
            },
        ];

        let chunks = chunk_file_ast("src/user.rs", code, &nodes);
        assert_eq!(chunks.len(), 2);

        assert_eq!(chunks[0].scope, "Struct User");
        assert!(chunks[0].text.starts_with("// File: src/user.rs | Scope: Struct User\n"));
        assert!(chunks[0].text.contains("pub struct User"));
        assert!(!chunks[0].blake3_hash.is_empty());

        assert_eq!(chunks[1].scope, "Function authenticate");
        assert!(chunks[1].text.starts_with("// File: src/user.rs | Scope: Function authenticate\n"));
        assert!(chunks[1].text.contains("pub fn authenticate"));
    }

    #[test]
    fn fallback_for_empty_nodes_creates_single_chunk() {
        let code = "const CONFIG: &str = \"default\";";
        let chunks = chunk_file_ast("config.rs", code, &[]);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].scope, "File");
        assert!(chunks[0].text.contains("const CONFIG"));
    }
}
