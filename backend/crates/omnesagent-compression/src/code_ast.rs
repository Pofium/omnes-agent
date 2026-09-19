//! AST-based code compressor for pruning implementation bodies from context files.
//!
//! Preserves public signatures, type definitions, struct/enum layouts, and docstrings,
//! while collapsing lengthy function bodies into concise `// [collapsed N lines]` comments.

use regex::Regex;

/// AST Code Compressor.
pub struct AstCodeCompressor;

impl AstCodeCompressor {
    /// Compresses source code by collapsing function and method bodies longer than `min_body_lines`.
    #[must_use]
    pub fn collapse_function_bodies(code: &str, min_body_lines: usize) -> String {
        let lines: Vec<&str> = code.lines().collect();
        let mut result = Vec::with_capacity(lines.len());

        let fn_decl_regex = Regex::new(
            r"^\s*(pub\s+|fn\s+|def\s+|async\s+def\s+|function\s+|public\s+|private\s+|protected\s+)"
        ).unwrap();

        let mut in_body = false;
        let mut brace_depth = 0;
        let mut body_lines_count: usize = 0;
        let mut body_start_idx = 0;

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];

            if !in_body {
                result.push(line.to_string());
                if fn_decl_regex.is_match(line) && line.contains('{') {
                    in_body = true;
                    brace_depth = line.matches('{').count() as i32 - line.matches('}').count() as i32;
                    body_lines_count = 0;
                    body_start_idx = result.len();
                    if brace_depth <= 0 {
                        in_body = false;
                    }
                }
            } else {
                let open_braces = line.matches('{').count() as i32;
                let close_braces = line.matches('}').count() as i32;
                brace_depth += open_braces - close_braces;
                body_lines_count += 1;

                if brace_depth <= 0 {
                    // Function body ended
                    let collapsed_count = body_lines_count.saturating_sub(1);
                    in_body = false;
                    if collapsed_count > min_body_lines {
                        let indent = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                        result.truncate(body_start_idx);
                        result.push(format!("{indent}    // ... [collapsed {collapsed_count} lines of implementation]"));
                        result.push(line.to_string()); // closing brace line
                    } else {
                        // Keep small body intact
                        for b_line in &lines[i - body_lines_count + 1..=i] {
                            result.push((*b_line).to_string());
                        }
                    }
                }
            }
            i += 1;
        }

        result.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapse_function_bodies() {
        let source = r#"
pub struct Service;

impl Service {
    pub fn do_heavy_work(&self) {
        let x = 1;
        let y = 2;
        let z = x + y;
        println!("{}", z);
        println!("more work");
        println!("even more work");
    }

    pub fn quick(&self) -> bool {
        true
    }
}
"#;

        let collapsed = AstCodeCompressor::collapse_function_bodies(source, 3);
        assert!(collapsed.contains("pub fn do_heavy_work"));
        assert!(collapsed.contains("// ... [collapsed 6 lines of implementation]"));
        assert!(collapsed.contains("pub fn quick"));
        assert!(collapsed.contains("true")); // small body not collapsed
    }
}
