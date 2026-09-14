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
    "project_call_path",
    "project_dead_code",
    "project_blast_hint",
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
        "Manage project code memory: register projects, perform instant AST scans (10 languages: Rust, Python, TS/JS, Go, PHP, Dart, Java, C/C++, SQL), calculate Blast Radius impact for safe refactoring, inspect call paths/dead code, generate token-budgeted repo-maps, and extract architectural context."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": PROJECT_CODE_ACTIONS,
                    "description": "The action to perform: project_init, project_scan, project_impact, project_context, project_graph_search, project_report, project_call_path, project_dead_code, project_blast_hint"
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
                "from_symbol": {
                    "type": "string",
                    "description": "Starting symbol for call path exploration (for project_call_path)"
                },
                "to_symbol": {
                    "type": "string",
                    "description": "Destination symbol for call path exploration (for project_call_path)"
                },
                "symbol": {
                    "type": "string",
                    "description": "Symbol to inspect callers/callees (for project_call_path)"
                },
                "direction": {
                    "type": "string",
                    "enum": ["callers", "callees"],
                    "description": "Direction for symbol inspection: 'callers' (who calls this) or 'callees' (who this calls)"
                },
                "mode": {
                    "type": "string",
                    "enum": ["default", "repo_map"],
                    "description": "Mode for project_context: 'default' (hierarchical context) or 'repo_map' (Aider-style token-budgeted signatures)"
                },
                "token_budget": {
                    "type": "integer",
                    "description": "Token budget for repo_map mode (e.g. 2048, 4096, 8192; default: 4096)"
                },
                "with_memory": {
                    "type": "boolean",
                    "description": "Include high-importance project memories in repo_map (default: true)"
                },
                "depth": {
                    "type": "integer",
                    "description": "Max traversal depth for impact calculation or call path (default: 3)"
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
                let mode = args.get("mode").and_then(|v| v.as_str()).unwrap_or("default");

                if mode == "repo_map" {
                    let max_tokens = args
                        .get("token_budget")
                        .or_else(|| args.get("max_tokens"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(4096) as usize;
                    let with_memory = args
                        .get("with_memory")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);

                    let repo_map = self.project_service.build_repo_map(project_id, query, max_tokens, with_memory)?;
                    Ok(ToolResult::ok(repo_map))
                } else {
                    let context_str = self.project_service.build_context(project_id, query)?;
                    Ok(ToolResult::ok(context_str))
                }
            }
            "project_call_path" => {
                let project_id = args
                    .get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("project_call_path requires 'project_id'"))?;
                let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

                let from = args.get("from_symbol").and_then(|v| v.as_str());
                let to = args.get("to_symbol").and_then(|v| v.as_str());

                if let (Some(f), Some(t)) = (from, to) {
                    match self.project_service.call_path(project_id, f, t, depth) {
                        Ok(Some(path_res)) => {
                            let md = omnesagent_kag::graph::format_path(&path_res);
                            Ok(ToolResult::ok(json!({
                                "status": "success",
                                "hops": path_res.hops,
                                "chain": path_res.chain,
                                "markdown": md
                            })))
                        }
                        Ok(None) => Ok(ToolResult::ok(json!({
                            "status": "not_found",
                            "found": false,
                            "message": format!("[project_call_path] Пути от `{f}` к `{t}` на глубине <= {depth} не найдено")
                        }))),
                        Err(e) => Ok(ToolResult::err(format!("[project_call_path] {e}"))),
                    }
                } else if let Some(sym) = args.get("symbol").and_then(|v| v.as_str()).or(from) {
                    let dir_str = args.get("direction").and_then(|v| v.as_str()).unwrap_or("callers");
                    let dir = if dir_str == "callees" {
                        omnesagent_kag::graph::Dir::Callees
                    } else {
                        omnesagent_kag::graph::Dir::Callers
                    };
                    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
                    match self.project_service.callers_callees(project_id, sym, dir, depth, limit) {
                        Ok(links) => {
                            let header = format!("{dir_str} для `{sym}`");
                            let md = omnesagent_kag::graph::format_links(&header, &links);
                            Ok(ToolResult::ok(json!({
                                "status": "success",
                                "count": links.len(),
                                "links": links,
                                "markdown": md
                            })))
                        }
                        Err(e) => Ok(ToolResult::err(format!("[project_call_path] {e}"))),
                    }
                } else {
                    anyhow::bail!("project_call_path requires either ('from_symbol' and 'to_symbol') or 'symbol'")
                }
            }
            "project_dead_code" => {
                let project_id = args
                    .get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("project_dead_code requires 'project_id'"))?;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(30) as usize;

                let dead = self.project_service.dead_code(project_id)?;
                let md = omnesagent_kag::graph::format_dead_code(&dead, limit);
                Ok(ToolResult::ok(json!({
                    "status": "success",
                    "count": dead.len(),
                    "dead_symbols": dead,
                    "markdown": md
                })))
            }
            "project_blast_hint" => {
                let project_id = args
                    .get("project_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("project_blast_hint requires 'project_id'"))?;

                let hint = self.project_service.blast_hint(project_id)?;
                Ok(ToolResult::ok(json!({
                    "status": "success",
                    "hint": hint
                })))
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_tool() -> ProjectCodeTool {
        let conn = Connection::open_in_memory().unwrap();
        omnesagent_kag::schema::migrate(&conn).unwrap();
        ProjectCodeTool::new(Arc::new(Mutex::new(conn)))
    }

    #[test]
    fn name_and_schema() {
        let tool = setup_tool();
        assert_eq!(tool.name(), "project_code");
        let schema = tool.parameters_schema();
        assert!(schema["properties"]["action"].is_object());
        assert!(schema["properties"]["project_id"].is_object());
    }

    #[tokio::test]
    async fn unknown_action_returns_error() {
        let tool = setup_tool();
        let res = tool.execute(json!({"action": "nonexistent"})).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn call_path_on_empty_project() {
        let tool = setup_tool();
        let res = tool.execute(json!({"action": "project_call_path"})).await;
        assert!(res.is_err());

        let res = tool
            .execute(json!({
                "action": "project_call_path",
                "project_id": "test_proj",
                "from_symbol": "foo",
                "to_symbol": "bar"
            }))
            .await
            .unwrap();
        assert!(!res.success);
        assert!(res.error.unwrap().contains("foo"));
    }

    #[tokio::test]
    async fn dead_code_on_empty_project() {
        let tool = setup_tool();
        let res = tool
            .execute(json!({
                "action": "project_dead_code",
                "project_id": "test_proj"
            }))
            .await
            .unwrap();
        assert!(res.success);
        let parsed: serde_json::Value = serde_json::from_str(&res.output).unwrap();
        assert_eq!(parsed["count"], 0);
    }

    #[tokio::test]
    async fn blast_hint_on_empty_project() {
        let tool = setup_tool();
        let res = tool
            .execute(json!({
                "action": "project_blast_hint",
                "project_id": "test_proj"
            }))
            .await
            .unwrap();
        assert!(res.success);
    }

    #[tokio::test]
    async fn repo_map_mode_context() {
        let tool = setup_tool();
        let res = tool
            .execute(json!({
                "action": "project_context",
                "project_id": "test_proj",
                "mode": "repo_map",
                "token_budget": 2048
            }))
            .await
            .unwrap();
        assert!(res.success);
    }
}

