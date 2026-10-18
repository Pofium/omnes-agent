//! AST-экстрактор для схем API и межсервисных контрактов:
//! - Protocol Buffers (.proto)
//! - GraphQL (.graphql, .gql)
//! - OpenAPI / Swagger (.json, .yaml)

use super::{AstEdge, AstNode, AstScanResult};
use regex::Regex;
use serde_json::Value;
use std::collections::HashSet;

pub struct SchemaExtractor {
    // Protobuf
    proto_package_re: Regex,
    proto_import_re: Regex,
    proto_service_re: Regex,
    proto_rpc_re: Regex,
    proto_message_re: Regex,
    proto_enum_re: Regex,

    // GraphQL
    gql_type_re: Regex,
    gql_interface_re: Regex,
    gql_enum_re: Regex,
    gql_input_re: Regex,
    gql_union_re: Regex,
    gql_field_re: Regex,

    // YAML OpenAPI fallback patterns
    yaml_path_re: Regex,
    yaml_method_re: Regex,
    yaml_schema_re: Regex,
}

impl Default for SchemaExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaExtractor {
    pub fn new() -> Self {
        Self {
            // Protobuf patterns
            proto_package_re: Regex::new(r#"(?m)^\s*package\s+([a-zA-Z0-9_.]+)\s*;"#).unwrap(),
            proto_import_re: Regex::new(r#"(?m)^\s*import\s+(?:public\s+|weak\s+)?["']([^"']+)["']\s*;"#).unwrap(),
            proto_service_re: Regex::new(r#"(?m)^\s*service\s+([a-zA-Z0-9_]+)\s*\{"#).unwrap(),
            proto_rpc_re: Regex::new(
                r#"(?m)^\s*rpc\s+([a-zA-Z0-9_]+)\s*\(\s*(?:stream\s+)?([a-zA-Z0-9_.]+)\s*\)\s*returns\s*\(\s*(?:stream\s+)?([a-zA-Z0-9_.]+)\s*\)"#,
            )
            .unwrap(),
            proto_message_re: Regex::new(r#"(?m)^\s*message\s+([a-zA-Z0-9_]+)\s*\{"#).unwrap(),
            proto_enum_re: Regex::new(r#"(?m)^\s*enum\s+([a-zA-Z0-9_]+)\s*\{"#).unwrap(),

            // GraphQL patterns
            gql_type_re: Regex::new(r#"(?m)^\s*(?:type|extend\s+type)\s+([a-zA-Z0-9_]+)(?:\s+implements\s+([a-zA-Z0-9_&\s]+))?\s*\{"#).unwrap(),
            gql_interface_re: Regex::new(r#"(?m)^\s*interface\s+([a-zA-Z0-9_]+)\s*\{"#).unwrap(),
            gql_enum_re: Regex::new(r#"(?m)^\s*enum\s+([a-zA-Z0-9_]+)\s*\{"#).unwrap(),
            gql_input_re: Regex::new(r#"(?m)^\s*input\s+([a-zA-Z0-9_]+)\s*\{"#).unwrap(),
            gql_union_re: Regex::new(r#"(?m)^\s*union\s+([a-zA-Z0-9_]+)\s*=\s*([^{]+)"#).unwrap(),
            gql_field_re: Regex::new(r#"(?m)^\s*([a-zA-Z0-9_]+)(?:\([^)]*\))?\s*:\s*([a-zA-Z0-9_!\[\]]+)"#).unwrap(),

            // OpenAPI YAML fallback
            yaml_path_re: Regex::new(r#"(?m)^(?:\s{2}|\t)(/[a-zA-Z0-9_{}/.-]+):\s*$"#).unwrap(),
            yaml_method_re: Regex::new(r#"(?m)^(?:\s{4}|\t{2})(get|post|put|delete|patch|options|head):\s*$"#).unwrap(),
            yaml_schema_re: Regex::new(r#"(?m)^(?:\s{4}|\t{2})([a-zA-Z0-9_]+):\s*$"#).unwrap(),
        }
    }

    /// Парсит Protocol Buffers (.proto) файлы.
    pub fn parse_proto(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let lines: Vec<&str> = content.lines().collect();

        // 1. package
        for (idx, line) in lines.iter().enumerate() {
            if let Some(cap) = self.proto_package_re.captures(line) {
                if let Some(pkg) = cap.get(1) {
                    let pkg_name = pkg.as_str().trim();
                    let node_id = format!("proto_pkg:{}", pkg_name);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: format!("package {}", pkg_name),
                        node_type: "Module".to_string(),
                        description: format!("Protobuf пакет {} в {}", pkg_name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("package {}", pkg_name),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }

        // 2. imports
        for line in lines.iter() {
            if let Some(cap) = self.proto_import_re.captures(line) {
                if let Some(imp) = cap.get(1) {
                    let imp_path = imp.as_str().trim();
                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: format!("file:{}", imp_path),
                        label: "IMPORTS".to_string(),
                        weight: 1.0,
                        context: format!("import \"{}\"", imp_path),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }

        // 3. services и RPC-методы
        let mut current_service: Option<String> = None;
        for (idx, line) in lines.iter().enumerate() {
            if let Some(cap) = self.proto_service_re.captures(line) {
                if let Some(svc) = cap.get(1) {
                    let svc_name = svc.as_str().trim();
                    current_service = Some(svc_name.to_string());
                    let node_id = format!("service:{}:{}", rel_path, svc_name);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: svc_name.to_string(),
                        node_type: "Service".to_string(),
                        description: format!("gRPC сервис {} в {}", svc_name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("service {}", svc_name),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }

            if let Some(cap) = self.proto_rpc_re.captures(line) {
                if let (Some(rpc), Some(req), Some(resp)) = (cap.get(1), cap.get(2), cap.get(3)) {
                    let rpc_name = rpc.as_str().trim();
                    let req_type = req.as_str().trim();
                    let resp_type = resp.as_str().trim();
                    let rpc_node_id = format!("rpc:{}:{}", rel_path, rpc_name);

                    out.nodes.push(AstNode {
                        node_id: rpc_node_id.clone(),
                        label: rpc_name.to_string(),
                        node_type: "Function".to_string(),
                        description: format!("gRPC RPC {} ({} -> {}) в {}", rpc_name, req_type, resp_type, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    let parent_id = match &current_service {
                        Some(svc) => format!("service:{}:{}", rel_path, svc),
                        None => file_node_id.to_string(),
                    };

                    out.edges.push(AstEdge {
                        source_node_id: parent_id,
                        target_node_id: rpc_node_id.clone(),
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("rpc {}({}) returns ({})", rpc_name, req_type, resp_type),
                        provenance: "EXTRACTED".to_string(),
                    });

                    // Связи входного и выходного сообщений
                    out.edges.push(AstEdge {
                        source_node_id: rpc_node_id.clone(),
                        target_node_id: format!("proto_msg:{}", req_type),
                        label: "DEPENDS_ON".to_string(),
                        weight: 0.8,
                        context: format!("param: {}", req_type),
                        provenance: "EXTRACTED".to_string(),
                    });

                    out.edges.push(AstEdge {
                        source_node_id: rpc_node_id,
                        target_node_id: format!("proto_msg:{}", resp_type),
                        label: "DEPENDS_ON".to_string(),
                        weight: 0.8,
                        context: format!("returns: {}", resp_type),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }

        // 4. Messages
        for (idx, line) in lines.iter().enumerate() {
            if let Some(cap) = self.proto_message_re.captures(line) {
                if let Some(msg) = cap.get(1) {
                    let msg_name = msg.as_str().trim();
                    let node_id = format!("proto_msg:{}", msg_name);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: msg_name.to_string(),
                        node_type: "Struct".to_string(),
                        description: format!("Protobuf message {} в {}", msg_name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("message {}", msg_name),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }

        // 5. Enums
        for (idx, line) in lines.iter().enumerate() {
            if let Some(cap) = self.proto_enum_re.captures(line) {
                if let Some(enm) = cap.get(1) {
                    let enm_name = enm.as_str().trim();
                    let node_id = format!("proto_enum:{}", enm_name);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: enm_name.to_string(),
                        node_type: "Enum".to_string(),
                        description: format!("Protobuf enum {} в {}", enm_name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("enum {}", enm_name),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }
    }

    /// Парсит GraphQL (.graphql, .gql) схемы.
    pub fn parse_graphql(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        let lines: Vec<&str> = content.lines().collect();
        let mut current_type: Option<String> = None;

        // 1. types & interfaces
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('}') {
                current_type = None;
            }

            if let Some(cap) = self.gql_type_re.captures(line) {
                if let Some(t_name) = cap.get(1) {
                    let name = t_name.as_str().trim();
                    current_type = Some(name.to_string());
                    let node_id = format!("gql_type:{}", name);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: name.to_string(),
                        node_type: "Class".to_string(),
                        description: format!("GraphQL тип {} в {}", name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id.clone(),
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("type {}", name),
                        provenance: "EXTRACTED".to_string(),
                    });

                    // implements
                    if let Some(impls) = cap.get(2) {
                        for iface in impls.as_str().split('&') {
                            let clean_iface = iface.trim();
                            if !clean_iface.is_empty() {
                                out.edges.push(AstEdge {
                                    source_node_id: node_id.clone(),
                                    target_node_id: format!("gql_iface:{}", clean_iface),
                                    label: "IMPLEMENTS".to_string(),
                                    weight: 1.0,
                                    context: format!("implements {}", clean_iface),
                                    provenance: "EXTRACTED".to_string(),
                                });
                            }
                        }
                    }
                }
            } else if let Some(ct) = &current_type {
                if ct == "Query" || ct == "Mutation" || ct == "Subscription" {
                    if let Some(fcap) = self.gql_field_re.captures(line) {
                        if let (Some(f_name), Some(ret_type)) = (fcap.get(1), fcap.get(2)) {
                            let field_str = f_name.as_str().trim();
                            let ret_str = ret_type.as_str().trim();
                            let op_node_id = format!("gql_op:{}:{}", ct, field_str);

                            out.nodes.push(AstNode {
                                node_id: op_node_id.clone(),
                                label: format!("{}.{}", ct, field_str),
                                node_type: "Function".to_string(),
                                description: format!("GraphQL операция {}.{}: {}", ct, field_str, ret_str),
                                file_path: rel_path.to_string(),
                                line_start: idx + 1,
                                line_end: idx + 1,
                            });

                            out.edges.push(AstEdge {
                                source_node_id: format!("gql_type:{}", ct),
                                target_node_id: op_node_id,
                                label: "DEFINES".to_string(),
                                weight: 1.0,
                                context: format!("{}: {}", field_str, ret_str),
                                provenance: "EXTRACTED".to_string(),
                            });
                        }
                    }
                }
            }

            if let Some(cap) = self.gql_interface_re.captures(line) {
                if let Some(iface) = cap.get(1) {
                    let name = iface.as_str().trim();
                    let node_id = format!("gql_iface:{}", name);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: name.to_string(),
                        node_type: "Interface".to_string(),
                        description: format!("GraphQL интерфейс {} в {}", name, rel_path),
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

            if let Some(cap) = self.gql_enum_re.captures(line) {
                if let Some(enm) = cap.get(1) {
                    let name = enm.as_str().trim();
                    let node_id = format!("gql_enum:{}", name);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: name.to_string(),
                        node_type: "Enum".to_string(),
                        description: format!("GraphQL enum {} в {}", name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("enum {}", name),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }

            if let Some(cap) = self.gql_input_re.captures(line) {
                if let Some(inp) = cap.get(1) {
                    let name = inp.as_str().trim();
                    let node_id = format!("gql_input:{}", name);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: name.to_string(),
                        node_type: "Struct".to_string(),
                        description: format!("GraphQL input {} в {}", name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("input {}", name),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }

            if let Some(cap) = self.gql_union_re.captures(line) {
                if let Some(u) = cap.get(1) {
                    let name = u.as_str().trim();
                    let node_id = format!("gql_union:{}", name);
                    out.nodes.push(AstNode {
                        node_id: node_id.clone(),
                        label: name.to_string(),
                        node_type: "Type".to_string(),
                        description: format!("GraphQL union {} в {}", name, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: idx + 1,
                        line_end: idx + 1,
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: node_id,
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("union {}", name),
                        provenance: "EXTRACTED".to_string(),
                    });
                }
            }
        }
    }

    /// Парсит спецификации OpenAPI / Swagger (.json, .yaml).
    pub fn parse_openapi(
        &self,
        rel_path: &str,
        file_node_id: &str,
        content: &str,
        out: &mut AstScanResult,
    ) {
        // Попытка структурированного JSON разбора
        if let Ok(json_val) = serde_json::from_str::<Value>(content) {
            if let Some(obj) = json_val.as_object() {
                if obj.contains_key("openapi") || obj.contains_key("swagger") || obj.contains_key("paths") {
                    let title = obj.get("info")
                        .and_then(|i| i.get("title"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("API");

                    // 1. API Title Node
                    let api_node_id = format!("api:{}", rel_path);
                    out.nodes.push(AstNode {
                        node_id: api_node_id.clone(),
                        label: title.to_string(),
                        node_type: "Service".to_string(),
                        description: format!("OpenAPI/Swagger спецификация '{}' в {}", title, rel_path),
                        file_path: rel_path.to_string(),
                        line_start: 1,
                        line_end: content.lines().count().max(1),
                    });

                    out.edges.push(AstEdge {
                        source_node_id: file_node_id.to_string(),
                        target_node_id: api_node_id.clone(),
                        label: "DEFINES".to_string(),
                        weight: 1.0,
                        context: format!("API: {}", title),
                        provenance: "EXTRACTED".to_string(),
                    });

                    // 2. Paths / Endpoints
                    if let Some(paths) = obj.get("paths").and_then(|p| p.as_object()) {
                        for (path_str, methods_val) in paths {
                            if let Some(methods) = methods_val.as_object() {
                                for (method, endpoint_val) in methods {
                                    let method_upper = method.to_uppercase();
                                    if !matches!(method_upper.as_str(), "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "OPTIONS" | "HEAD") {
                                        continue;
                                    }

                                    let op_id = endpoint_val.get("operationId").and_then(|v| v.as_str()).unwrap_or("");
                                    let summary = endpoint_val.get("summary").and_then(|v| v.as_str()).unwrap_or("");
                                    let label = format!("{} {}", method_upper, path_str);
                                    let node_id = format!("endpoint:{}:{}", method_upper, path_str);

                                    out.nodes.push(AstNode {
                                        node_id: node_id.clone(),
                                        label: label.clone(),
                                        node_type: "Endpoint".to_string(),
                                        description: format!("{} (opId: '{}', summary: '{}')", label, op_id, summary),
                                        file_path: rel_path.to_string(),
                                        line_start: 1,
                                        line_end: 1,
                                    });

                                    out.edges.push(AstEdge {
                                        source_node_id: api_node_id.clone(),
                                        target_node_id: node_id,
                                        label: "EXPOSES".to_string(),
                                        weight: 1.0,
                                        context: label,
                                        provenance: "EXTRACTED".to_string(),
                                    });
                                }
                            }
                        }
                    }

                    // 3. Components / Schemas (DTOs)
                    let schemas_opt = obj.get("components")
                        .and_then(|c| c.get("schemas"))
                        .and_then(|s| s.as_object())
                        .or_else(|| obj.get("definitions").and_then(|d| d.as_object()));

                    if let Some(schemas) = schemas_opt {
                        for (schema_name, _) in schemas {
                            let node_id = format!("schema:{}", schema_name);
                            out.nodes.push(AstNode {
                                node_id: node_id.clone(),
                                label: schema_name.clone(),
                                node_type: "Struct".to_string(),
                                description: format!("OpenAPI DTO модель/схема {} в {}", schema_name, rel_path),
                                file_path: rel_path.to_string(),
                                line_start: 1,
                                line_end: 1,
                            });

                            out.edges.push(AstEdge {
                                source_node_id: api_node_id.clone(),
                                target_node_id: node_id,
                                label: "DEFINES".to_string(),
                                weight: 1.0,
                                context: format!("schema {}", schema_name),
                                provenance: "EXTRACTED".to_string(),
                            });
                        }
                    }
                    return;
                }
            }
        }

        // YAML эвристический разбор (для OpenAPI в формате YAML)
        if content.contains("openapi:") || content.contains("swagger:") || content.contains("paths:") {
            let lines: Vec<&str> = content.lines().collect();
            let mut current_path: Option<String> = None;
            let mut in_schemas = false;
            let mut seen_endpoints = HashSet::new();
            let mut seen_schemas = HashSet::new();

            for (idx, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed == "paths:" {
                    in_schemas = false;
                } else if trimmed == "components:" || trimmed == "schemas:" || trimmed == "definitions:" {
                    in_schemas = true;
                    current_path = None;
                }

                if in_schemas {
                    if let Some(cap) = self.yaml_schema_re.captures(line) {
                        if let Some(s_name) = cap.get(1) {
                            let schema_str = s_name.as_str().trim();
                            if !schema_str.is_empty() && schema_str != "type" && schema_str != "properties" && seen_schemas.insert(schema_str.to_string()) {
                                let node_id = format!("schema:{}", schema_str);
                                out.nodes.push(AstNode {
                                    node_id: node_id.clone(),
                                    label: schema_str.to_string(),
                                    node_type: "Struct".to_string(),
                                    description: format!("YAML OpenAPI DTO модель {} в {}", schema_str, rel_path),
                                    file_path: rel_path.to_string(),
                                    line_start: idx + 1,
                                    line_end: idx + 1,
                                });

                                out.edges.push(AstEdge {
                                    source_node_id: file_node_id.to_string(),
                                    target_node_id: node_id,
                                    label: "DEFINES".to_string(),
                                    weight: 1.0,
                                    context: format!("schema {}", schema_str),
                                    provenance: "EXTRACTED".to_string(),
                                });
                            }
                        }
                    }
                } else if let Some(cap) = self.yaml_path_re.captures(line) {
                    if let Some(p) = cap.get(1) {
                        current_path = Some(p.as_str().to_string());
                    }
                } else if let Some(cap) = self.yaml_method_re.captures(line) {
                    if let (Some(method), Some(path_str)) = (cap.get(1), &current_path) {
                        let method_upper = method.as_str().to_uppercase();
                        let label = format!("{} {}", method_upper, path_str);
                        if seen_endpoints.insert(label.clone()) {
                            let node_id = format!("endpoint:{}:{}", method_upper, path_str);
                            out.nodes.push(AstNode {
                                node_id: node_id.clone(),
                                label: label.clone(),
                                node_type: "Endpoint".to_string(),
                                description: format!("YAML OpenAPI эндпоинт {} в {}", label, rel_path),
                                file_path: rel_path.to_string(),
                                line_start: idx + 1,
                                line_end: idx + 1,
                            });

                            out.edges.push(AstEdge {
                                source_node_id: file_node_id.to_string(),
                                target_node_id: node_id,
                                label: "EXPOSES".to_string(),
                                weight: 1.0,
                                context: label,
                                provenance: "EXTRACTED".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
}
