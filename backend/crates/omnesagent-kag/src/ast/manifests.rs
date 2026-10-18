//! AST-экстрактор для манифестов пакетных менеджеров и зависимостей:
//! - Cargo.toml (Rust)
//! - package.json (Node.js/TypeScript)
//! - pyproject.toml / requirements.txt (Python)
//! - go.mod (Go)
//! - pubspec.yaml (Dart/Flutter)
//! - composer.json (PHP)

use super::{AstEdge, AstNode, AstScanResult};
use regex::Regex;
use serde_json::Value;

pub struct ManifestExtractor {
    // Cargo.toml
    cargo_pkg_name_re: Regex,
    cargo_dep_re: Regex,

    // Python requirements.txt / pyproject.toml
    req_line_re: Regex,
    pyproject_dep_re: Regex,

    // go.mod
    go_mod_module_re: Regex,
    go_mod_require_re: Regex,

    // pubspec.yaml
    pubspec_name_re: Regex,
    pubspec_dep_re: Regex,
}

impl Default for ManifestExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl ManifestExtractor {
    pub fn new() -> Self {
        Self {
            // Cargo
            cargo_pkg_name_re: Regex::new(r#"(?m)^\s*name\s*=\s*["']([^"']+)["']"#).unwrap(),
            cargo_dep_re: Regex::new(r#"(?m)^([a-zA-Z0-9_-]+)\s*=\s*(?:["']([^"']+)["']|\{([^}]+)\})"#).unwrap(),

            // Python
            req_line_re: Regex::new(r#"(?m)^\s*([a-zA-Z0-9_.-]+)\s*(?:([><!=~]=?\s*[a-zA-Z0-9_.*+-]+))?"#).unwrap(),
            pyproject_dep_re: Regex::new(r#"["']([a-zA-Z0-9_.-]+)\s*([><!=~]=?[^"']*)?["']"#).unwrap(),

            // go.mod
            go_mod_module_re: Regex::new(r#"(?m)^\s*module\s+([a-zA-Z0-9_./-]+)"#).unwrap(),
            go_mod_require_re: Regex::new(r#"(?m)^\s*([a-zA-Z0-9_./-]+)\s+(v[0-9a-zA-Z_.-]+)"#).unwrap(),

            // pubspec.yaml
            pubspec_name_re: Regex::new(r#"(?m)^\s*name:\s*([a-zA-Z0-9_]+)"#).unwrap(),
            pubspec_dep_re: Regex::new(r#"(?m)^(?:\s{2}|\t)([a-zA-Z0-9_]+):\s*(?:\^?([0-9a-zA-Z_.-]+)|$)"#).unwrap(),
        }
    }

    /// Парсит Cargo.toml.
    pub fn parse_cargo_toml(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let lines: Vec<&str> = content.lines().collect();

        // Package name
        if let Some(cap) = self.cargo_pkg_name_re.captures(content) {
            if let Some(pkg) = cap.get(1) {
                let pkg_name = pkg.as_str().trim();
                let node_id = format!("pkg:cargo:{}", pkg_name);
                out.nodes.push(AstNode {
                    node_id: node_id.clone(),
                    label: pkg_name.to_string(),
                    node_type: "Package".to_string(),
                    description: format!("Cargo пакет {} в {}", pkg_name, rel_path),
                    file_path: rel_path.to_string(),
                    line_start: 1,
                    line_end: 1,
                });

                out.edges.push(AstEdge {
                    source_node_id: file_node_id.to_string(),
                    target_node_id: node_id,
                    label: "DEFINES".to_string(),
                    weight: 1.0,
                    context: format!("package.name = \"{}\"", pkg_name),
                    provenance: "EXTRACTED".to_string(),
                });
            }
        }

        // Dependencies sections
        let mut in_deps = false;
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                in_deps = trimmed.starts_with("[dependencies")
                    || trimmed.starts_with("[dev-dependencies")
                    || trimmed.starts_with("[build-dependencies")
                    || trimmed.starts_with("[workspace.dependencies");
                continue;
            }

            if in_deps && !trimmed.is_empty() && !trimmed.starts_with('#') {
                if let Some(cap) = self.cargo_dep_re.captures(line) {
                    if let Some(dep_name_match) = cap.get(1) {
                        let dep_name = dep_name_match.as_str().trim();
                        let ver_str = cap.get(2).map(|v| v.as_str())
                            .or_else(|| cap.get(3).map(|v| v.as_str()))
                            .unwrap_or("");

                        let dep_node_id = format!("dep:cargo:{}", dep_name);
                        out.nodes.push(AstNode {
                            node_id: dep_node_id.clone(),
                            label: dep_name.to_string(),
                            node_type: "Dependency".to_string(),
                            description: format!("Cargo зависимость {} ({}) в {}", dep_name, ver_str, rel_path),
                            file_path: rel_path.to_string(),
                            line_start: idx + 1,
                            line_end: idx + 1,
                        });

                        out.edges.push(AstEdge {
                            source_node_id: file_node_id.to_string(),
                            target_node_id: dep_node_id,
                            label: "DEPENDS_ON".to_string(),
                            weight: 1.0,
                            context: format!("{} = {}", dep_name, ver_str),
                            provenance: "EXTRACTED".to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Парсит package.json.
    pub fn parse_package_json(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        if let Ok(json_val) = serde_json::from_str::<Value>(content) {
            if let Some(obj) = json_val.as_object() {
                // Package name
                if let Some(name_val) = obj.get("name").and_then(|n| n.as_str()) {
                    let node_id = format!("pkg:npm:{}", name_val);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: name_val.to_string(),
                        node_type: "Package".to_string(),
                        description: format!("NPM пакет {} в {}", name_val, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: 1,
                        line_end: 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("npm package: {}", name_val),
                        provenance: "EXTRACTED".to_string(),
                    });
                }

                // Dependencies
                for dep_section in ["dependencies", "devDependencies", "peerDependencies"] {
                    if let Some(deps) = obj.get(dep_section).and_then(|d| d.as_object()) {
                        for (dep_name, ver_val) in deps {
                            let ver_str = ver_val.as_str().unwrap_or("*");
                            let dep_node_id = format!("dep:npm:{}", dep_name);

                            out.nodes.push(AstNode {
                                node_id: dep_node_id.clone(),
                                label: dep_name.clone(),
                                node_type: "Dependency".to_string(),
                                description: format!("NPM зависимость {} ({}) в {}", dep_name, ver_str, rel_path),
                                file_path: rel_path.to_string(),
                                line_start: 1,
                                line_end: 1,
                            });

                            out.edges.push(AstEdge {
                                source_node_id: file_node_id.to_string(),
                                target_node_id: dep_node_id,
                                label: "DEPENDS_ON".to_string(),
                                weight: 1.0,
                                context: format!("{} ({}: {})", dep_name, dep_section, ver_str),
                                provenance: "EXTRACTED".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Парсит requirements.txt или pyproject.toml.
    pub fn parse_python_manifest(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let is_pyproject = rel_path.ends_with("pyproject.toml");
        let lines: Vec<&str> = content.lines().collect();

        if is_pyproject {
            let mut in_deps = false;
            for (idx, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with('[') {
                    in_deps = trimmed.contains("dependencies");
                    continue;
                }
                if in_deps {
                    for cap in self.pyproject_dep_re.captures_iter(line) {
                        if let Some(pkg) = cap.get(1) {
                            let pkg_name = pkg.as_str().trim();
                            let ver_str = cap.get(2).map(|v| v.as_str()).unwrap_or("");
                            let dep_node_id = format!("dep:pypi:{}", pkg_name);

                            out.nodes.push(AstNode {
                                node_id: dep_node_id.clone(),
                                label: pkg_name.to_string(),
                                node_type: "Dependency".to_string(),
                                description: format!("PyPI зависимость {} ({}) в {}", pkg_name, ver_str, rel_path),
                                file_path: rel_path.to_string(),
                                line_start: idx + 1,
                                line_end: idx + 1,
                            });

                            out.edges.push(AstEdge {
                                source_node_id: file_node_id.to_string(),
                                target_node_id: dep_node_id,
                                label: "DEPENDS_ON".to_string(),
                                weight: 1.0,
                                context: format!("{} {}", pkg_name, ver_str),
                                provenance: "EXTRACTED".to_string(),
                            });
                        }
                    }
                }
            }
        } else {
            // requirements.txt
            for (idx, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
                    continue;
                }

                if let Some(cap) = self.req_line_re.captures(trimmed) {
                    if let Some(pkg) = cap.get(1) {
                        let pkg_name = pkg.as_str().trim();
                        let ver_str = cap.get(2).map(|v| v.as_str()).unwrap_or("");
                        let dep_node_id = format!("dep:pypi:{}", pkg_name);

                        out.nodes.push(AstNode {
                            node_id: dep_node_id.clone(),
                            label: pkg_name.to_string(),
                            node_type: "Dependency".to_string(),
                            description: format!("PyPI зависимость {} ({}) в {}", pkg_name, ver_str, rel_path),
                            file_path: rel_path.to_string(),
                            line_start: idx + 1,
                            line_end: idx + 1,
                        });

                        out.edges.push(AstEdge {
                            source_node_id: file_node_id.to_string(),
                            target_node_id: dep_node_id,
                            label: "DEPENDS_ON".to_string(),
                            weight: 1.0,
                            context: format!("{} {}", pkg_name, ver_str),
                            provenance: "EXTRACTED".to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Парсит go.mod.
    pub fn parse_go_mod(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let lines: Vec<&str> = content.lines().collect();

        // Module
        if let Some(cap) = self.go_mod_module_re.captures(content) {
            if let Some(mod_m) = cap.get(1) {
                let mod_name = mod_m.as_str().trim();
                let node_id = format!("pkg:go:{}", mod_name);
                out.nodes.push(AstNode {
                    node_id: node_id.clone(),
                    label: mod_name.to_string(),
                    node_type: "Package".to_string(),
                    description: format!("Go модуль {} в {}", mod_name, rel_path),
                    file_path: rel_path.to_string(),
                    line_start: 1,
                    line_end: 1,
                });

                out.edges.push(AstEdge {
                    source_node_id: file_node_id.to_string(),
                    target_node_id: node_id,
                    label: "DEFINES".to_string(),
                    weight: 1.0,
                    context: format!("module {}", mod_name),
                    provenance: "EXTRACTED".to_string(),
                });
            }
        }

        // Requirements
        let mut in_require_block = false;
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("require (") {
                in_require_block = true;
                continue;
            } else if in_require_block && trimmed == ")" {
                in_require_block = false;
                continue;
            }

            if in_require_block || trimmed.starts_with("require ") {
                let target_line = if trimmed.starts_with("require ") {
                    trimmed.trim_start_matches("require ").trim()
                } else {
                    trimmed
                };

                if let Some(cap) = self.go_mod_require_re.captures(target_line) {
                    if let (Some(pkg), Some(ver)) = (cap.get(1), cap.get(2)) {
                        let pkg_name = pkg.as_str().trim();
                        let ver_str = ver.as_str().trim();
                        let dep_node_id = format!("dep:go:{}", pkg_name);

                        out.nodes.push(AstNode {
                            node_id: dep_node_id.clone(),
                            label: pkg_name.to_string(),
                            node_type: "Dependency".to_string(),
                            description: format!("Go зависимость {} ({}) в {}", pkg_name, ver_str, rel_path),
                            file_path: rel_path.to_string(),
                            line_start: idx + 1,
                            line_end: idx + 1,
                        });

                        out.edges.push(AstEdge {
                            source_node_id: file_node_id.to_string(),
                            target_node_id: dep_node_id,
                            label: "DEPENDS_ON".to_string(),
                            weight: 1.0,
                            context: format!("{} {}", pkg_name, ver_str),
                            provenance: "EXTRACTED".to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Парсит pubspec.yaml (Dart / Flutter).
    pub fn parse_pubspec_yaml(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let lines: Vec<&str> = content.lines().collect();

        // Name
        if let Some(cap) = self.pubspec_name_re.captures(content) {
            if let Some(name_m) = cap.get(1) {
                let pkg_name = name_m.as_str().trim();
                let node_id = format!("pkg:pub:{}", pkg_name);
                out.nodes.push(AstNode {
                    node_id: node_id.clone(),
                    label: pkg_name.to_string(),
                    node_type: "Package".to_string(),
                    description: format!("Dart/Flutter пакет {} в {}", pkg_name, rel_path),
                    file_path: rel_path.to_string(),
                    line_start: 1,
                    line_end: 1,
                });

                out.edges.push(AstEdge {
                    source_node_id: file_node_id.to_string(),
                    target_node_id: node_id,
                    label: "DEFINES".to_string(),
                    weight: 1.0,
                    context: format!("name: {}", pkg_name),
                    provenance: "EXTRACTED".to_string(),
                });
            }
        }

        // Dependencies
        let mut in_deps = false;
        for (idx, line) in lines.iter().enumerate() {
            if line.starts_with("dependencies:") || line.starts_with("dev_dependencies:") {
                in_deps = true;
                continue;
            } else if !line.starts_with(' ') && !line.starts_with('\t') && !line.is_empty() {
                in_deps = false;
            }

            if in_deps {
                if let Some(cap) = self.pubspec_dep_re.captures(line) {
                    if let Some(pkg) = cap.get(1) {
                        let pkg_name = pkg.as_str().trim();
                        if pkg_name == "flutter" || pkg_name == "sdk" {
                            continue;
                        }
                        let ver_str = cap.get(2).map(|v| v.as_str()).unwrap_or("*");
                        let dep_node_id = format!("dep:pub:{}", pkg_name);

                        out.nodes.push(AstNode {
                            node_id: dep_node_id.clone(),
                            label: pkg_name.to_string(),
                            node_type: "Dependency".to_string(),
                            description: format!("Pub зависимость {} ({}) в {}", pkg_name, ver_str, rel_path),
                            file_path: rel_path.to_string(),
                            line_start: idx + 1,
                            line_end: idx + 1,
                        });

                        out.edges.push(AstEdge {
                            source_node_id: file_node_id.to_string(),
                            target_node_id: dep_node_id,
                            label: "DEPENDS_ON".to_string(),
                            weight: 1.0,
                            context: format!("{} : {}", pkg_name, ver_str),
                            provenance: "EXTRACTED".to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Парсит composer.json (PHP).
    pub fn parse_composer_json(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        if let Ok(json_val) = serde_json::from_str::<Value>(content) {
            if let Some(obj) = json_val.as_object() {
                // Name
                if let Some(name_val) = obj.get("name").and_then(|n| n.as_str()) {
                    let node_id = format!("pkg:composer:{}", name_val);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: name_val.to_string(),
                        node_type: "Package".to_string(),
                        description: format!("Composer пакет {} в {}", name_val, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: 1,
                        line_end: 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("composer package: {}", name_val),
                        provenance: "EXTRACTED".to_string(),
                    });
                }

                // Dependencies
                for dep_section in ["require", "require-dev"] {
                    if let Some(deps) = obj.get(dep_section).and_then(|d| d.as_object()) {
                        for (dep_name, ver_val) in deps {
                            let ver_str = ver_val.as_str().unwrap_or("*");
                            let dep_node_id = format!("dep:composer:{}", dep_name);

                            out.nodes.push(AstNode {
                                node_id: dep_node_id.clone(),
                                label: dep_name.clone(),
                                node_type: "Dependency".to_string(),
                                description: format!("Composer зависимость {} ({}) в {}", dep_name, ver_str, rel_path),
                                file_path: rel_path.to_string(),
                                line_start: 1,
                                line_end: 1,
                            });

                            out.edges.push(AstEdge {
                                source_node_id: file_node_id.to_string(),
                                target_node_id: dep_node_id,
                                label: "DEPENDS_ON".to_string(),
                                weight: 1.0,
                                context: format!("{} ({}: {})", dep_name, dep_section, ver_str),
                                provenance: "EXTRACTED".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
}
