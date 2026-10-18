//! AST-экстрактор для инфраструктуры как кода (IaC) и контейнеризации:
//! - Terraform / OpenTofu (.tf, .hcl)
//! - Dockerfile & Docker Compose
//! - Kubernetes manifests (.yaml)

use super::{AstEdge, AstNode, AstScanResult};
use regex::Regex;

pub struct IacExtractor {
    // Terraform
    tf_resource_re: Regex,
    tf_data_re: Regex,
    tf_module_re: Regex,
    tf_module_source_re: Regex,
    tf_var_re: Regex,
    tf_output_re: Regex,
    tf_provider_re: Regex,

    // Dockerfile
    docker_from_re: Regex,
    docker_expose_re: Regex,

    // Docker Compose
    compose_service_re: Regex,
    compose_image_re: Regex,
    compose_depends_item_re: Regex,

    // Kubernetes
    k8s_kind_re: Regex,
    k8s_name_re: Regex,
}

impl Default for IacExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl IacExtractor {
    pub fn new() -> Self {
        Self {
            // Terraform / HCL
            tf_resource_re: Regex::new(r#"(?m)^\s*resource\s+["']([^"']+)["']\s+["']([^"']+)["']\s*\{"#).unwrap(),
            tf_data_re: Regex::new(r#"(?m)^\s*data\s+["']([^"']+)["']\s+["']([^"']+)["']\s*\{"#).unwrap(),
            tf_module_re: Regex::new(r#"(?m)^\s*module\s+["']([^"']+)["']\s*\{"#).unwrap(),
            tf_module_source_re: Regex::new(r#"(?m)^\s*source\s*=\s*["']([^"']+)["']"#).unwrap(),
            tf_var_re: Regex::new(r#"(?m)^\s*variable\s+["']([^"']+)["']\s*\{"#).unwrap(),
            tf_output_re: Regex::new(r#"(?m)^\s*output\s+["']([^"']+)["']\s*\{"#).unwrap(),
            tf_provider_re: Regex::new(r#"(?m)^\s*provider\s+["']([^"']+)["']\s*\{"#).unwrap(),

            // Dockerfile
            docker_from_re: Regex::new(r#"(?mi)^\s*FROM\s+([a-zA-Z0-9_./:-]+)(?:\s+AS\s+([a-zA-Z0-9_-]+))?"#).unwrap(),
            docker_expose_re: Regex::new(r#"(?mi)^\s*EXPOSE\s+([0-9\s/a-zA-Z]+)"#).unwrap(),

            // Compose
            compose_service_re: Regex::new(r#"(?m)^(?:\s{2}|\t)([a-zA-Z0-9_-]+):\s*$"#).unwrap(),
            compose_image_re: Regex::new(r#"(?m)^(?:\s{4}|\t{2})image:\s*["']?([a-zA-Z0-9_./:-]+)["']?"#).unwrap(),
            compose_depends_item_re: Regex::new(r#"(?m)^\s*-\s*([a-zA-Z0-9_-]+)"#).unwrap(),

            // Kubernetes
            k8s_kind_re: Regex::new(r#"(?m)^kind:\s*([a-zA-Z0-9_]+)\s*$"#).unwrap(),
            k8s_name_re: Regex::new(r#"(?m)^\s*name:\s*["']?([a-zA-Z0-9_.-]+)["']?\s*$"#).unwrap(),
        }
    }

    /// Парсит файлы Terraform / OpenTofu (.tf, .hcl).
    pub fn parse_terraform(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let lines: Vec<&str> = content.lines().collect();

        // 1. Resources
        for (idx, line) in lines.iter().enumerate() {
            if let Some(cap) = self.tf_resource_re.captures(line) {
                if let (Some(r_type), Some(r_name)) = (cap.get(1), cap.get(2)) {
                    let label = format!("{}.{}", r_type.as_str(), r_name.as_str());
                    let node_id = format!("tf_resource:{}:{}", rel_path, label);

                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: label.clone(),
                        node_type: "Resource".to_string(),
                        description: format!("Terraform ресурс {} в {}", label, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("resource \"{}\" \"{}\"", r_type.as_str(), r_name.as_str()),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }

            // 2. Data sources
            if let Some(cap) = self.tf_data_re.captures(line) {
                if let (Some(d_type), Some(d_name)) = (cap.get(1), cap.get(2)) {
                    let label = format!("data.{}.{}", d_type.as_str(), d_name.as_str());
                    let node_id = format!("tf_data:{}:{}", rel_path, label);

                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: label.clone(),
                        node_type: "Data".to_string(),
                        description: format!("Terraform data source {} в {}", label, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("data \"{}\" \"{}\"", d_type.as_str(), d_name.as_str()),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }

            // 3. Modules
            if let Some(cap) = self.tf_module_re.captures(line) {
                if let Some(m_name) = cap.get(1) {
                    let label = format!("module.{}", m_name.as_str());
                    let node_id = format!("tf_module:{}:{}", rel_path, label);

                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: label.clone(),
                        node_type: "Module".to_string(),
                        description: format!("Terraform модуль {} в {}", label, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id.clone(),
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("module \"{}\"", m_name.as_str()),
                        provenance: "EXTRACTED".to_string(),
                    });

                    // Поиск source внутри следующих нескольких строк
                    for look_idx in (idx + 1)..(idx + 10).min(lines.len()) {
                        if let Some(scap) = self.tf_module_source_re.captures(lines[look_idx]) {
                            if let Some(src) = scap.get(1) {
                                out.edges.push(AstEdge {
                                    source_node_id: node_id.clone(),
                                    target_node_id: format!("module_source:{}", src.as_str()),
                                    label: "IMPORTS".to_string(),
                                    weight: 1.0,
                                    context: format!("source = \"{}\"", src.as_str()),
                                    provenance: "EXTRACTED".to_string(),
                                });
                                break;
                            }
                        }
                    }
                }
            }

            // 4. Variables & Outputs & Providers
            if let Some(cap) = self.tf_var_re.captures(line) {
                if let Some(v) = cap.get(1) {
                    let label = format!("var.{}", v.as_str());
                    let node_id = format!("tf_var:{}:{}", rel_path, label);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: label.clone(),
                        node_type: "Variable".to_string(),
                        description: format!("Terraform переменная {} в {}", label, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 0.5,
                        context: format!("variable \"{}\"", v.as_str()),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }

            if let Some(cap) = self.tf_output_re.captures(line) {
                if let Some(o) = cap.get(1) {
                    let label = format!("output.{}", o.as_str());
                    let node_id = format!("tf_output:{}:{}", rel_path, label);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: label.clone(),
                        node_type: "Output".to_string(),
                        description: format!("Terraform output {} в {}", label, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 0.5,
                        context: format!("output \"{}\"", o.as_str()),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }

            if let Some(cap) = self.tf_provider_re.captures(line) {
                if let Some(p) = cap.get(1) {
                    let label = format!("provider.{}", p.as_str());
                    let node_id = format!("tf_provider:{}", label);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: label.clone(),
                        node_type: "Provider".to_string(),
                        description: format!("Terraform провайдер {} в {}", label, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEPENDS_ON".to_string(),
                        weight: 1.0,
                        context: format!("provider \"{}\"", p.as_str()),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }
    }

    /// Парсит Dockerfile.
    pub fn parse_dockerfile(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let lines: Vec<&str> = content.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            if let Some(cap) = self.docker_from_re.captures(line) {
                if let Some(img) = cap.get(1) {
                    let img_str = img.as_str().trim();
                    let stage_opt = cap.get(2).map(|s| s.as_str().trim());

                    let label = match stage_opt {
                        Some(stage) => format!("FROM {} AS {}", img_str, stage),
                        None => format!("FROM {}", img_str),
                    };

                    let node_id = format!("docker_stage:{}:{}", rel_path, idx + 1);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: label.clone(),
                        node_type: "Image".to_string(),
                        description: format!("Базовый образ Docker {} в {}", label, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id.clone(),
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: label.clone(),
                        provenance: "EXTRACTED".to_string(),
                    });

                    // Зависимость от внешнего образа
                    out.edges.push(AstEdge {
                        source_node_id: node_id,
                        target_node_id: format!("image:{}", img_str),
                        label: "DEPENDS_ON".to_string(),
                        weight: 1.0,
                        context: format!("FROM {}", img_str),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            } else if let Some(cap) = self.docker_expose_re.captures(line) {
                if let Some(port) = cap.get(1) {
                    let port_str = port.as_str().trim();
                    let node_id = format!("docker_port:{}:{}", rel_path, port_str);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: format!("EXPOSE {}", port_str),
                        node_type: "Endpoint".to_string(),
                        description: format!("Docker порт EXPOSE {} в {}", port_str, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });
                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 0.5,
                        context: format!("EXPOSE {}", port_str),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }
    }

    /// Парсит файлы Docker Compose (docker-compose.yml, compose.yaml).
    pub fn parse_docker_compose(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let lines: Vec<&str> = content.lines().collect();
        let mut in_services = false;
        let mut in_depends_on = false;
        let mut current_service: Option<(String, usize)> = None;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }

            if line.starts_with("services:") {
                in_services = true;
                in_depends_on = false;
                continue;
            } else if !line.starts_with(' ') && !line.starts_with('\t') && !line.is_empty() {
                in_services = false;
                in_depends_on = false;
            }

            if in_services {
                if let Some(cap) = self.compose_service_re.captures(line) {
                    if let Some(svc) = cap.get(1) {
                        let svc_name = svc.as_str().trim();
                        current_service = Some((svc_name.to_string(), idx + 1));
                        in_depends_on = false;
                        let node_id = format!("compose_svc:{}:{}", rel_path, svc_name);

                        out.nodes.push(AstNode {
                            node_id: node_id.clone(),
                            label: svc_name.to_string(),
                            node_type: "Service".to_string(),
                            description: format!("Docker Compose сервис {} в {}", svc_name, rel_path),
                            file_path: rel_path.to_string(),
                            line_start: idx + 1,
                            line_end: idx + 1,
                        });

                        out.edges.push(AstEdge {
                            source_node_id: file_node_id.to_string(),
                            target_node_id: node_id,
                            label: "DEFINES".to_string(),
                            weight: 1.0,
                            context: format!("service: {}", svc_name),
                            provenance: "EXTRACTED".to_string(),
                        });
                    }
                } else if trimmed.starts_with("depends_on:") {
                    in_depends_on = true;
                } else if in_depends_on && trimmed.starts_with('-') {
                    if let Some(cap) = self.compose_depends_item_re.captures(line) {
                        if let (Some(dep), Some((svc_name, _))) = (cap.get(1), &current_service) {
                            let dep_name = dep.as_str().trim();
                            let src_id = format!("compose_svc:{}:{}", rel_path, svc_name);
                            out.edges.push(AstEdge {
                                source_node_id: src_id,
                                target_node_id: format!("compose_svc:{}:{}", rel_path, dep_name),
                                label: "DEPENDS_ON".to_string(),
                                weight: 1.0,
                                context: format!("depends_on: {}", dep_name),
                                provenance: "EXTRACTED".to_string(),
                            });
                        }
                    }
                } else if !trimmed.starts_with('-') {
                    in_depends_on = false;
                }

                if let Some(cap) = self.compose_image_re.captures(line) {
                    if let (Some(img), Some((svc_name, _))) = (cap.get(1), &current_service) {
                        let img_name = img.as_str().trim();
                        let src_id = format!("compose_svc:{}:{}", rel_path, svc_name);
                        out.edges.push(AstEdge {
                            source_node_id: src_id,
                            target_node_id: format!("image:{}", img_name),
                            label: "DEPENDS_ON".to_string(),
                            weight: 1.0,
                            context: format!("image: {}", img_name),
                            provenance: "EXTRACTED".to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Парсит манифесты Kubernetes (k8s/*.yaml, Helm templates).
    pub fn parse_kubernetes(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let lines: Vec<&str> = content.lines().collect();
        let mut kind_opt: Option<String> = None;
        let mut name_opt: Option<String> = None;
        let mut line_kind = 1;

        for (idx, line) in lines.iter().enumerate() {
            if let Some(cap) = self.k8s_kind_re.captures(line) {
                if let Some(k) = cap.get(1) {
                    kind_opt = Some(k.as_str().trim().to_string());
                    line_kind = idx + 1;
                }
            } else if let Some(cap) = self.k8s_name_re.captures(line) {
                if name_opt.is_none() {
                    if let Some(n) = cap.get(1) {
                        name_opt = Some(n.as_str().trim().to_string());
                    }
                }
            }

            // Документы в k8s разделяются `---`
            if line.trim() == "---" || idx == lines.len() - 1 {
                if let (Some(kind), Some(name)) = (&kind_opt, &name_opt) {
                    let label = format!("{}:{}", kind, name);
                    let node_id = format!("k8s:{}:{}", rel_path, label);

                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: label.clone(),
                        node_type: "Resource".to_string(),
                        description: format!("Kubernetes {} манифест в {}", label, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: line_kind,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: label,
                        provenance: "EXTRACTED".to_string(),
                    });
                }
                kind_opt = None;
                name_opt = None;
            }
        }
    }
}
