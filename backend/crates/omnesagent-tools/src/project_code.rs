//! Project Code Memory Tool (AST Graph, Blast Radius & Architecture Context).
//!
//! Exposes AST-driven code intelligence from `omnesagent-kag` to the agent.

use async_trait::async_trait;
use parking_lot::Mutex;
use rusqlite::Connection;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

use omnesagent_api::tool::{Tool, ToolResult};
use omnesagent_kag::project::ProjectService;

const PROJECT_CODE_ACTIONS: &[&str] = &[
    "project_init",
    "project_scan",
    "project_impact",
    "project_context",
    "project_graph_search",
    "project_report",
];

/// Tool for registering projects, scanning codebases via AST,
/// computing Blast Radius, and injecting project context.
pub struct ProjectCodeTool {
    project_service: ProjectService,
}

impl ProjectCodeTool {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self {
            project_service: ProjectService::new(conn),
        }
    }
}

#[async_trait]
impl Tool for ProjectCodeTool {
    fn name(&self) -> &str {
        "project_code"
    }

    fn description(&self) -> &str {
        "Manage project code memory: register projects, perform instant AST scans (10 languages: Rust, Python, TS/JS, Go, PHP, Dart, Java, C/C++, SQL), calculate Blast Radius impact for safe refactoring, search code symbols, and extract architectural context."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": PROJECT_CODE_ACTIONS,
                    "description": "The action to perform: project_init, project_scan, project_impact, project_context, project_graph_search, project_report"
                },
                "project_id": {
                    "type": "string",
                    "description": "Unique identifier for the project (e.g. 'omnes-agent')"
                },
                "name": {
                    "type": "string",
                    "description": "Human-readable project name (for project_init)"
                },
                "root_path": {
                    "type": "string",
                    "description": "Absolute path to the repository root directory"
                },
                "target_symbol": {
                    "type": "string",
                    "description": "Symbol name or function/class for impact analysis (for project_impact)"
                },
                "target_file": {
                    "type": "string",
                    "description": "Relative file path for impact analysis (for project_impact)"
                },
                "depth": {
                    "type": "integer",
                    "description": "Max traversal depth for impact calculation (default: 3)"
                },
                "query": {
                    "type": "string",
                    "description": "Search term for graph symbols (for project_graph_search or project_context)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Max results to return (default: 20)"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        match action {
            "project_init" => {
                let _name = args.get("name").and_then(|v| v.as_str());
                let root_path_str = args
                    .get("root_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("project_init requires 'root_path'"))?;
                let root_path = PathBuf::from(root_path_str);
                let project_id_opt = args.get("project_id").and_then(|v| v.as_str());

                let project = self.project_service.register_project_auto(&root_path, project_id_opt)?;

                Ok(ToolResult::ok(json!({
                    "status": "success",
                    "project": project,
                    "message": format!("Project '{}' registered successfully with ID '{}'", project.name, project.id)
                })))
            }
            "project_scan" => {
                let project_id = args
                    .get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("project_scan requires 'project_id'"))?;
                let custom_path = args.get("path").and_then(|v| v.as_str());
                let incremental = args.get("incremental").and_then(|v| v.as_bool()).unwrap_or(true);

                let stats = self.project_service.scan_project(project_id, custom_path, incremental)?;

                Ok(ToolResult::ok(json!({
                    "status": "success",
                    "scan_stats": {
                        "files_scanned": stats.files_scanned,
                        "nodes_found": stats.nodes.len(),
                        "edges_found": stats.edges.len()
                    },
                    "message": format!("Scanned {} files, extracted {} nodes, {} edges", stats.files_scanned, stats.nodes.len(), stats.edges.len())
                })))
            }
            "project_impact" => {
                let project_id = args
                    .get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("project_impact requires 'project_id'"))?;
                let target_symbol = args.get("target_symbol").and_then(|v| v.as_str());
                let target_file = args.get("target_file").and_then(|v| v.as_str());
                let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(3) as usize;

                let target = target_symbol.or(target_file).ok_or_else(|| {
                    anyhow::anyhow!("project_impact requires either 'target_symbol' or 'target_file'")
                })?;

                let impact = self.project_service.analyze_impact(project_id, target, depth)?;

                Ok(ToolResult::ok(json!({
                    "status": "success",
                    "impact": impact
                })))
            }
            "project_context" => {
                let project_id = args
                    .get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("project_context requires 'project_id'"))?;
                let query = args.get("query").and_then(|v| v.as_str());

                let context_str = self.project_service.build_context(project_id, query)?;

                Ok(ToolResult::ok(context_str))
            }
            "project_graph_search" => {
                let project_id = args
                    .get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("project_graph_search requires 'project_id'"))?;
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

                let nodes = self.project_service.search_symbols(project_id, query, limit).await?;

                Ok(ToolResult::ok(json!({
                    "status": "success",
                    "count": nodes.len(),
                    "nodes": nodes
                })))
            }
            "project_report" => {
                let project_id = args
                    .get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("project_report requires 'project_id'"))?;

                let report = self.project_service.generate_project_report(project_id)?;

                Ok(ToolResult::ok(json!({
                    "status": "success",
                    "report": report
                })))
            }
            other => anyhow::bail!("unknown action: {other}, expected one of {PROJECT_CODE_ACTIONS:?}"),
        }
    }
}
