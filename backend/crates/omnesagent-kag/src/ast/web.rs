//! AST-экстрактор для веб-разметки и стилей (HTML, CSS, SCSS).
//! Извлекает DOM-элементы, классы, идентификаторы, импорты скриптов/стилей и CSS custom properties.

use super::{AstEdge, AstNode, AstScanResult};
use regex::Regex;
use std::collections::HashSet;

pub struct WebExtractor {
    html_script_re: Regex,
    html_link_re: Regex,
    html_href_re: Regex,
    html_id_re: Regex,
    html_class_re: Regex,
    html_custom_elem_re: Regex,

    css_import_re: Regex,
    css_class_re: Regex,
    css_id_re: Regex,
    css_custom_prop_re: Regex,
    css_mixin_re: Regex,
    css_keyframes_re: Regex,
}

impl Default for WebExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl WebExtractor {
    pub fn new() -> Self {
        Self {
            // HTML patterns
            html_script_re: Regex::new(r#"(?i)<script\b[^>]*\bsrc\s*=\s*["']([^"']+)["']"#).unwrap(),
            html_link_re: Regex::new(r#"(?i)<link\b([^>]+)>"#).unwrap(),
            html_href_re: Regex::new(r#"(?i)\bhref\s*=\s*["']([^"']+)["']"#).unwrap(),
            html_id_re: Regex::new(r#"(?i)\bid\s*=\s*["']([a-zA-Z0-9_-]+)["']"#).unwrap(),
            html_class_re: Regex::new(r#"(?i)\bclass\s*=\s*["']([^"']+)["']"#).unwrap(),
            html_custom_elem_re: Regex::new(r#"(?i)<([a-z][a-z0-9]*-[a-z0-9-]+)\b"#).unwrap(),

            // CSS / SCSS patterns
            css_import_re: Regex::new(r#"(?i)@import\s+(?:url\()?['"]([^'"]+)['"]\)?\s*;"#).unwrap(),
            css_class_re: Regex::new(r#"(?m)^\s*(?:\.|[a-zA-Z0-9_-]*\.)([a-zA-Z0-9_-]{2,})\s*(?:[,\{]|::|:|\s)"#).unwrap(),
            css_id_re: Regex::new(r#"(?m)^\s*(?:#|[a-zA-Z0-9_-]*#)([a-zA-Z0-9_-]{2,})\s*(?:[,\{]|::|:|\s)"#).unwrap(),
            css_custom_prop_re: Regex::new(r#"(?m)^\s*(--[a-zA-Z0-9_-]+)\s*:"#).unwrap(),
            css_mixin_re: Regex::new(r#"(?m)^\s*@(mixin|function)\s+([a-zA-Z0-9_-]+)"#).unwrap(),
            css_keyframes_re: Regex::new(r#"(?i)@keyframes\s+([a-zA-Z0-9_-]+)"#).unwrap(),
        }
    }

    /// Парсит HTML документы.
    pub fn parse_html(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let lines: Vec<&str> = content.lines().collect();

        // 1. Скрипты <script src="..."> -> IMPORTS
        for (_idx, line) in lines.iter().enumerate() {
            for cap in self.html_script_re.captures_iter(line) {
                if let Some(src) = cap.get(1) {
                    let src_path = src.as_str().trim();
                    let target_node_id = if src_path.starts_with("http://") || src_path.starts_with("https://") || src_path.starts_with("//") {
                        format!("external:{}", src_path)
                    } else {
                        format!("file:{}", src_path)
                    };

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id,
                        label: "IMPORTS".to_string(),
                        weight: 1.0,
                        context: format!("script src: {}", src_path),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }

        // 2. Стили <link rel="stylesheet" href="..."> -> STYLES
        for cap in self.html_link_re.captures_iter(content) {
            if let Some(attrs) = cap.get(1) {
                let attrs_str = attrs.as_str();
                if attrs_str.to_lowercase().contains("stylesheet") {
                    if let Some(href_cap) = self.html_href_re.captures(attrs_str) {
                        if let Some(href) = href_cap.get(1) {
                            let href_path = href.as_str().trim();
                            let target_node_id = if href_path.starts_with("http://") || href_path.starts_with("https://") {
                                format!("external:{}", href_path)
                            } else {
                                format!("file:{}", href_path)
                            };

                            out.edges.push(AstEdge {
                                source_node_id: file_node_id.to_string(),
                                target_node_id,
                                label: "STYLES".to_string(),
                                weight: 1.0,
                                context: format!("link stylesheet: {}", href_path),
                                provenance: "EXTRACTED".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // 3. ID элементов -> DEFINES (DOM Node)
        let mut seen_ids = HashSet::new();
        for (idx, line) in lines.iter().enumerate() {
            for cap in self.html_id_re.captures_iter(line) {
                if let Some(id_match) = cap.get(1) {
                    let id_val = id_match.as_str().trim();
                    if seen_ids.insert(id_val.to_string()) {
                        let node_id = format!("dom_id:{}:{}", rel_path, id_val);
                        out.nodes.push(AstNode {
                            node_id: node_id.clone(),
                            label: format!("#{}", id_val),
                            node_type: "Element".to_string(),
                            description: format!("HTML DOM элемент с id='{}' в {}", id_val, rel_path),
                            file_path: rel_path.to_string(),
                            line_start: idx + 1,
                            line_end: idx + 1,
                        });

                        out.edges.push(AstEdge {
                            source_node_id: file_node_id.to_string(),
                            target_node_id: node_id,
                            label: "DEFINES".to_string(),
                            weight: 1.0,
                            context: format!("id=\"{}\"", id_val),
                            provenance: "EXTRACTED".to_string(),
                        });
                    }
                }
            }
        }

        // 4. CSS Классы -> BINDS_CLASS
        let mut seen_classes = HashSet::new();
        for line in lines.iter() {
            for cap in self.html_class_re.captures_iter(line) {
                if let Some(classes_match) = cap.get(1) {
                    for cls in classes_match.as_str().split_whitespace() {
                        let clean_cls = cls.trim();
                        if !clean_cls.is_empty() && seen_classes.insert(clean_cls.to_string()) {
                            out.edges.push(AstEdge {
                                source_node_id: file_node_id.to_string(),
                                target_node_id: format!("class:{}", clean_cls),
                                label: "BINDS_CLASS".to_string(),
                                weight: 0.5,
                                context: format!("class=\"{}\"", clean_cls),
                                provenance: "EXTRACTED".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // 5. Custom Elements (web components) -> Component
        let mut seen_components = HashSet::new();
        for (idx, line) in lines.iter().enumerate() {
            for cap in self.html_custom_elem_re.captures_iter(line) {
                if let Some(tag_match) = cap.get(1) {
                    let tag_name = tag_match.as_str().to_lowercase();
                    if seen_components.insert(tag_name.clone()) {
                        let node_id = format!("component:{}:{}", rel_path, tag_name);
                        out.nodes.push(AstNode {
                            node_id: node_id.clone(),
                            label: format!("<{}>", tag_name),
                            node_type: "Component".to_string(),
                            description: format!("Веб-компонент <{}> в {}", tag_name, rel_path),
                            file_path: rel_path.to_string(),
                            line_start: idx + 1,
                            line_end: idx + 1,
                        });

                        out.edges.push(AstEdge {
                            source_node_id: file_node_id.to_string(),
                            target_node_id: node_id,
                            label: "DEFINES".to_string(),
                            weight: 1.0,
                            context: format!("<{}>", tag_name),
                            provenance: "EXTRACTED".to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Парсит файлы стилей CSS и SCSS.
    pub fn parse_css_scss(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let lines: Vec<&str> = content.lines().collect();

        // 1. @import -> IMPORTS
        for cap in self.css_import_re.captures_iter(content) {
            if let Some(target) = cap.get(1) {
                let target_path = target.as_str().trim();
                out.edges.push(AstEdge {
                    source_node_id: file_node_id.to_string(),
                    target_node_id: format!("file:{}", target_path),
                    label: "IMPORTS".to_string(),
                    weight: 1.0,
                    context: format!("@import \"{}\"", target_path),
                    provenance: "EXTRACTED".to_string(),
                });
            }
        }

        // 2. Селекторы классов .name -> Class
        let mut seen_classes = HashSet::new();
        for (idx, line) in lines.iter().enumerate() {
            for cap in self.css_class_re.captures_iter(line) {
                if let Some(cls) = cap.get(1) {
                    let class_name = cls.as_str().trim();
                    if !class_name.is_empty() && seen_classes.insert(class_name.to_string()) {
                        let node_id = format!("class:{}", class_name);
                        out.nodes.push(AstNode {
                            node_id: node_id.clone(),
                            label: format!(".{}", class_name),
                            node_type: "Class".to_string(),
                            description: format!("CSS класс .{} в {}", class_name, rel_path),
                            file_path: rel_path.to_string(),
                            line_start: idx + 1,
                            line_end: idx + 1,
                        });

                        out.edges.push(AstEdge {
                            source_node_id: file_node_id.to_string(),
                            target_node_id: node_id,
                            label: "DEFINES".to_string(),
                            weight: 1.0,
                            context: format!(".{}", class_name),
                            provenance: "EXTRACTED".to_string(),
                        });
                    }
                }
            }
        }

        // 3. Селекторы ID #name -> Element
        let mut seen_ids = HashSet::new();
        for (idx, line) in lines.iter().enumerate() {
            for cap in self.css_id_re.captures_iter(line) {
                if let Some(id_match) = cap.get(1) {
                    let id_name = id_match.as_str().trim();
                    if !id_name.is_empty() && seen_ids.insert(id_name.to_string()) {
                        let node_id = format!("dom_id:{}", id_name);
                        out.nodes.push(AstNode {
                            node_id: node_id.clone(),
                            label: format!("#{}", id_name),
                            node_type: "Element".to_string(),
                            description: format!("CSS ID селектор #{} в {}", id_name, rel_path),
                            file_path: rel_path.to_string(),
                            line_start: idx + 1,
                            line_end: idx + 1,
                        });

                        out.edges.push(AstEdge {
                            source_node_id: file_node_id.to_string(),
                            target_node_id: node_id,
                            label: "DEFINES".to_string(),
                            weight: 1.0,
                            context: format!("#{}", id_name),
                            provenance: "EXTRACTED".to_string(),
                        });
                    }
                }
            }
        }

        // 4. CSS переменные (custom properties) --var-name -> Property
        let mut seen_props = HashSet::new();
        for (idx, line) in lines.iter().enumerate() {
            for cap in self.css_custom_prop_re.captures_iter(line) {
                if let Some(prop) = cap.get(1) {
                    let prop_name = prop.as_str().trim();
                    if seen_props.insert(prop_name.to_string()) {
                        let node_id = format!("css_var:{}:{}", rel_path, prop_name);
                        out.nodes.push(AstNode {
                            node_id: node_id.clone(),
                            label: prop_name.to_string(),
                            node_type: "Property".to_string(),
                            description: format!("CSS Custom Property {} в {}", prop_name, rel_path),
                            file_path: rel_path.to_string(),
                            line_start: idx + 1,
                            line_end: idx + 1,
                        });

                        out.edges.push(AstEdge {
                            source_node_id: file_node_id.to_string(),
                            target_node_id: node_id,
                            label: "DEFINES".to_string(),
                            weight: 0.8,
                            context: prop_name.to_string(),
                            provenance: "EXTRACTED".to_string(),
                        });
                    }
                }
            }
        }

        // 5. SCSS @mixin и @function -> Function
        for (idx, line) in lines.iter().enumerate() {
            for cap in self.css_mixin_re.captures_iter(line) {
                if let (Some(kind), Some(name)) = (cap.get(1), cap.get(2)) {
                    let kind_str = kind.as_str();
                    let name_str = name.as_str();
                    let node_id = format!("scss_{}:{}:{}", kind_str, rel_path, name_str);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: format!("@{} {}", kind_str, name_str),
                        node_type: "Function".to_string(),
                        description: format!("SCSS {} {} в {}", kind_str, name_str, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("@{} {}", kind_str, name_str),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }

        // 6. @keyframes animation-name -> Animation
        for (idx, line) in lines.iter().enumerate() {
            for cap in self.css_keyframes_re.captures_iter(line) {
                if let Some(anim) = cap.get(1) {
                    let anim_name = anim.as_str().trim();
                    let node_id = format!("keyframes:{}:{}", rel_path, anim_name);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: format!("@keyframes {}", anim_name),
                        node_type: "Animation".to_string(),
                        description: format!("CSS анимация @keyframes {} в {}", anim_name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("@keyframes {}", anim_name),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }
    }
}
