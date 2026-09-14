//! Ralph Knowledge Layer tools (OB2H v1.3 Ф26–28).
//!
//! Exposes `ralph_start`, `ralph_iteration`, `ralph_verdict`, `ralph_context`, and `ralph_report`.

use async_trait::async_trait;
use chrono::Local;
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use omnesagent_api::tool::{Tool, ToolOutput, ToolResult};

fn new_id(prefix: &str) -> String {
    format!("{prefix}_{}", &Uuid::new_v4().to_string().replace('-', "")[..16])
}

fn auto_verdict(tests_summary: Option<&str>) -> (String, String) {
    match tests_summary.and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok()) {
        Some(v) => {
            let failed = v.get("failed").and_then(|x| x.as_i64()).unwrap_or(0);
            let passed = v.get("passed").and_then(|x| x.as_i64()).unwrap_or(0);
            if failed > 0 {
                ("failed".into(), "auto_tests".into())
            } else if passed > 0 {
                ("verified".into(), "auto_tests".into())
            } else {
                ("unconfirmed".into(), "auto_tests".into())
            }
        }
        None => ("unconfirmed".into(), "none".into()),
    }
}

/// Tool for starting a Ralph development session on a feature.
pub struct RalphStartTool {
    conn: Arc<Mutex<Connection>>,
}

impl RalphStartTool {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl Tool for RalphStartTool {
    fn name(&self) -> &str {
        "ralph_start"
    }

    fn description(&self) -> &str {
        "Start a Ralph iterative development run for a project feature. Ensures only one active run exists per (project_id, feature_slug)."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "project_id": {
                    "type": "string",
                    "description": "Identifier of the project"
                },
                "feature_slug": {
                    "type": "string",
                    "description": "Feature identifier (e.g. auth-jwt or search-filter)"
                },
                "goal": {
                    "type": "string",
                    "description": "High-level goal or specification for the feature run"
                },
                "autonomy": {
                    "type": "string",
                    "enum": ["L1", "L2", "L3"],
                    "description": "Autonomy level (default: L1)"
                },
                "max_iterations_per_task": {
                    "type": "integer",
                    "description": "Max iterations allowed per task (default: 5)"
                },
                "max_total_iterations": {
                    "type": "integer",
                    "description": "Max total iterations for the run (default: 60)"
                },
                "budget_tokens": {
                    "type": "integer",
                    "description": "Optional token budget cap"
                }
            },
            "required": ["project_id", "feature_slug", "goal"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let (Some(project_id), Some(feature_slug), Some(goal)) = (
            args.get("project_id").and_then(|v| v.as_str()),
            args.get("feature_slug").and_then(|v| v.as_str()),
            args.get("goal").and_then(|v| v.as_str()),
        ) else {
            return Ok(ToolResult {
                success: false,
                output: ToolOutput::default(),
                error: Some("project_id, feature_slug, and goal are required".into()),
            });
        };

        let autonomy = args.get("autonomy").and_then(|v| v.as_str()).unwrap_or("L1");
        let max_per_task = args.get("max_iterations_per_task").and_then(|v| v.as_i64()).unwrap_or(5);
        let max_total = args.get("max_total_iterations").and_then(|v| v.as_i64()).unwrap_or(60);
        let budget_tokens = args.get("budget_tokens").and_then(|v| v.as_i64());

        let conn = self.conn.lock();
        let active: Option<String> = conn
            .query_row(
                "SELECT id FROM ralph_runs WHERE project_id = ?1 AND feature_slug = ?2 \
                 AND status NOT IN ('archived', 'stopped', 'failed', 'completed')",
                params![project_id, feature_slug],
                |r| r.get(0),
            )
            .optional()?;

        if let Some(active_id) = active {
            return Ok(ToolResult {
                success: false,
                output: ToolOutput::default(),
                error: Some(format!(
                    "Active run {active_id} already exists for ({project_id}, {feature_slug})"
                )),
            });
        }

        let run_id = new_id("rrun");
        let now = Local::now().to_rfc3339();

        conn.execute(
            "INSERT INTO ralph_runs (id, project_id, feature_slug, goal, status, autonomy, \
             max_iterations_per_task, max_total_iterations, budget_tokens, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, 'applying', ?5, ?6, ?7, ?8, ?9, ?9)",
            params![
                run_id,
                project_id,
                feature_slug,
                goal,
                autonomy,
                max_per_task,
                max_total,
                budget_tokens,
                now
            ],
        )?;

        Ok(ToolResult {
            success: true,
            output: format!("run_id={run_id} status=applying").into(),
            error: None,
        })
    }
}

/// Tool for recording a Ralph iteration step with hypothesis, test summary and findings.
pub struct RalphIterationTool {
    conn: Arc<Mutex<Connection>>,
}

impl RalphIterationTool {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl Tool for RalphIterationTool {
    fn name(&self) -> &str {
        "ralph_iteration"
    }

    fn description(&self) -> &str {
        "Record an iteration step in a Ralph run. Automatically computes verdict from objective test signals (ADR-K4)."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "run_id": {
                    "type": "string",
                    "description": "Identifier of the active run"
                },
                "task_id": {
                    "type": "string",
                    "description": "Identifier of the task being worked on"
                },
                "n": {
                    "type": "integer",
                    "description": "Iteration number (1-based)"
                },
                "hypothesis": {
                    "type": "string",
                    "description": "Working hypothesis for this iteration"
                },
                "plan": {
                    "type": "string",
                    "description": "Step plan"
                },
                "result": {
                    "type": "string",
                    "description": "Result summary or observations"
                },
                "tests_summary": {
                    "type": "string",
                    "description": "JSON string containing test execution metrics, e.g. {\"passed\": 5, \"failed\": 0}"
                },
                "ladder_rung": {
                    "type": "string",
                    "description": "Ladder rung (1: prompt, 2: context, 3: agent, 4: human)"
                },
                "git_before": {
                    "type": "string",
                    "description": "Git commit SHA before iteration"
                },
                "git_after": {
                    "type": "string",
                    "description": "Git commit SHA after iteration"
                },
                "findings": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "kind": { "type": "string" },
                            "content": { "type": "string" },
                            "symbols": { "type": "array", "items": { "type": "string" } }
                        }
                    },
                    "description": "List of findings or lessons discovered during this iteration"
                }
            },
            "required": ["run_id", "task_id", "n"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let (Some(run_id), Some(task_id), Some(n)) = (
            args.get("run_id").and_then(|v| v.as_str()),
            args.get("task_id").and_then(|v| v.as_str()),
            args.get("n").and_then(|v| v.as_i64()),
        ) else {
            return Ok(ToolResult {
                success: false,
                output: ToolOutput::default(),
                error: Some("run_id, task_id, and n are required".into()),
            });
        };

        let hypothesis = args.get("hypothesis").and_then(|v| v.as_str());
        let plan = args.get("plan").and_then(|v| v.as_str());
        let result = args.get("result").and_then(|v| v.as_str());
        let tests_summary = args.get("tests_summary").and_then(|v| v.as_str());
        let ladder_rung = args.get("ladder_rung").and_then(|v| v.as_str());
        let git_before = args.get("git_before").and_then(|v| v.as_str());
        let git_after = args.get("git_after").and_then(|v| v.as_str());

        let (verdict, verdict_source) = auto_verdict(tests_summary);

        let conn = self.conn.lock();
        let project_id: Option<String> = conn
            .query_row(
                "SELECT project_id FROM ralph_runs WHERE id = ?1",
                params![run_id],
                |r| r.get(0),
            )
            .optional()?;

        let Some(project_id) = project_id else {
            return Ok(ToolResult {
                success: false,
                output: ToolOutput::default(),
                error: Some(format!("Run not found: {run_id}")),
            });
        };

        let iteration_id = new_id("rit");
        let now = Local::now().to_rfc3339();

        conn.execute(
            "INSERT INTO ralph_iterations (id, run_id, task_id, n, git_before, git_after, \
             hypothesis, plan, result, tests_summary, verdict, verdict_source, ladder_rung, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                iteration_id,
                run_id,
                task_id,
                n,
                git_before,
                git_after,
                hypothesis,
                plan,
                result,
                tests_summary,
                verdict,
                verdict_source,
                ladder_rung,
                now
            ],
        )?;

        conn.execute(
            "UPDATE ralph_runs SET status = 'verifying', updated_at = ?2 WHERE id = ?1",
            params![run_id, now],
        )?;

        let mut findings_count = 0usize;
        if let Some(findings) = args.get("findings").and_then(|v| v.as_array()) {
            for f in findings {
                let kind = f.get("kind").and_then(|x| x.as_str()).unwrap_or("gotcha");
                let Some(content) = f.get("content").and_then(|x| x.as_str()) else {
                    continue;
                };
                let symbols = f.get("symbols").map(|s| s.to_string()).unwrap_or_else(|| "[]".into());
                let finding_id = new_id("rfind");
                let _ = conn.execute(
                    "INSERT INTO ralph_findings (id, run_id, iteration_id, project_id, kind, content, symbols, verdict, verdict_source, created_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'unconfirmed', 'iteration', ?8)",
                    params![finding_id, run_id, iteration_id, project_id, kind, content, symbols, now],
                );
                findings_count += 1;
            }
        }

        Ok(ToolResult {
            success: true,
            output: format!(
                "iteration_id={iteration_id} project={project_id} verdict={verdict} (source={verdict_source}) findings={findings_count}"
            )
            .into(),
            error: None,
        })
    }
}

/// Tool for explicitly setting or overriding an iteration or finding verdict.
pub struct RalphVerdictTool {
    conn: Arc<Mutex<Connection>>,
}

impl RalphVerdictTool {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl Tool for RalphVerdictTool {
    fn name(&self) -> &str {
        "ralph_verdict"
    }

    fn description(&self) -> &str {
        "Set or override the verdict on an iteration or finding ('verified', 'failed', 'unconfirmed', 'overturned', 'stale')."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "iteration_id": { "type": "string", "description": "Iteration identifier" },
                "finding_id": { "type": "string", "description": "Finding identifier" },
                "verdict": {
                    "type": "string",
                    "enum": ["verified", "failed", "unconfirmed", "overturned", "stale"],
                    "description": "Verdict to assign"
                },
                "verdict_source": { "type": "string", "description": "Source of verdict (default: manual)" }
            },
            "required": ["verdict"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let verdict = match args.get("verdict").and_then(|v| v.as_str()) {
            Some(v) => v,
            None => return Ok(ToolResult {
                success: false,
                output: ToolOutput::default(),
                error: Some("verdict is required".into()),
            }),
        };
        let source = args.get("verdict_source").and_then(|v| v.as_str()).unwrap_or("manual");

        let conn = self.conn.lock();
        if let Some(it_id) = args.get("iteration_id").and_then(|v| v.as_str()) {
            let updated = conn.execute(
                "UPDATE ralph_iterations SET verdict = ?1, verdict_source = ?2 WHERE id = ?3",
                params![verdict, source, it_id],
            )?;
            if updated > 0 {
                return Ok(ToolResult {
                    success: true,
                    output: format!("Updated iteration {it_id} verdict to {verdict} ({source})").into(),
                    error: None,
                });
            }
        }

        if let Some(f_id) = args.get("finding_id").and_then(|v| v.as_str()) {
            let updated = conn.execute(
                "UPDATE ralph_findings SET verdict = ?1, verdict_source = ?2 WHERE id = ?3",
                params![verdict, source, f_id],
            )?;
            if updated > 0 {
                return Ok(ToolResult {
                    success: true,
                    output: format!("Updated finding {f_id} verdict to {verdict} ({source})").into(),
                    error: None,
                });
            }
        }

        Ok(ToolResult {
            success: false,
            output: ToolOutput::default(),
            error: Some("Provide a valid iteration_id or finding_id to update".into()),
        })
    }
}

/// Tool for assembling task context pack (specs, negative experience, reuse candidates).
pub struct RalphContextTool {
    conn: Arc<Mutex<Connection>>,
}

impl RalphContextTool {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl Tool for RalphContextTool {
    fn name(&self) -> &str {
        "ralph_context"
    }

    fn description(&self) -> &str {
        "Assemble comprehensive Ralph task context pack including goal, negative experience from failed iterations, and architectural candidates."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "run_id": { "type": "string", "description": "Run identifier" },
                "task_id": { "type": "string", "description": "Task identifier" },
                "mode": { "type": "string", "enum": ["full", "lite"], "description": "Detail mode (default: full)" }
            },
            "required": ["run_id", "task_id"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let (Some(run_id), Some(task_id)) = (
            args.get("run_id").and_then(|v| v.as_str()),
            args.get("task_id").and_then(|v| v.as_str()),
        ) else {
            return Ok(ToolResult {
                success: false,
                output: ToolOutput::default(),
                error: Some("run_id and task_id are required".into()),
            });
        };

        let conn = self.conn.lock();
        let run_info: Option<(String, String, String)> = conn
            .query_row(
                "SELECT project_id, feature_slug, goal FROM ralph_runs WHERE id = ?1",
                params![run_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;

        let Some((project_id, feature_slug, goal)) = run_info else {
            return Ok(ToolResult {
                success: false,
                output: ToolOutput::default(),
                error: Some(format!("Run {run_id} not found")),
            });
        };

        let mut out = format!("# Ralph Task Context: {task_id}\n\n");
        out.push_str(&format!("**Project:** {project_id}\n**Feature:** {feature_slug}\n**Goal:** {goal}\n\n"));

        // Negative experience: failed iterations
        let mut stmt = conn.prepare(
            "SELECT n, hypothesis, plan, tests_summary FROM ralph_iterations \
             WHERE run_id = ?1 AND (task_id = ?2 OR verdict = 'failed') ORDER BY n DESC LIMIT 5"
        )?;
        let rows = stmt.query_map(params![run_id, task_id], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, Option<String>>(1)?, r.get::<_, Option<String>>(2)?, r.get::<_, Option<String>>(3)?))
        })?;

        let mut fails = Vec::new();
        for r in rows.flatten() {
            fails.push(format!("- Iteration #{}: hyp: {:?}, summary: {:?}", r.0, r.1.unwrap_or_default(), r.3.unwrap_or_default()));
        }

        if !fails.is_empty() {
            out.push_str("### Past Iteration Experience\n");
            for f in fails {
                out.push_str(&format!("{f}\n"));
            }
            out.push('\n');
        }

        // Findings
        let mut f_stmt = conn.prepare(
            "SELECT kind, content, verdict FROM ralph_findings WHERE run_id = ?1 LIMIT 10"
        )?;
        let f_rows = f_stmt.query_map(params![run_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
        })?;
        let findings: Vec<_> = f_rows.flatten().collect();
        if !findings.is_empty() {
            out.push_str("### Findings & Gotchas\n");
            for (k, c, v) in findings {
                out.push_str(&format!("- [{k} | {v}] {c}\n"));
            }
        }

        Ok(ToolResult {
            success: true,
            output: out.into(),
            error: None,
        })
    }
}

/// Tool for generating analytic summary report of a Ralph run.
pub struct RalphReportTool {
    conn: Arc<Mutex<Connection>>,
}

impl RalphReportTool {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl Tool for RalphReportTool {
    fn name(&self) -> &str {
        "ralph_report"
    }

    fn description(&self) -> &str {
        "Generate summary report on Ralph run(s): iteration counts, verified/failed rates, and findings."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "run_id": { "type": "string", "description": "Optional run identifier to inspect" }
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let run_id = args.get("run_id").and_then(|v| v.as_str());
        let conn = self.conn.lock();

        if let Some(rid) = run_id {
            let run: Option<(String, String, String, String)> = conn
                .query_row(
                    "SELECT project_id, feature_slug, goal, status FROM ralph_runs WHERE id = ?1",
                    params![rid],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
                .optional()?;
            let Some((project, feature, goal, status)) = run else {
                return Ok(ToolResult {
                    success: false,
                    output: ToolOutput::default(),
                    error: Some(format!("Run {rid} not found")),
                });
            };

            let iterations: i64 = conn.query_row(
                "SELECT count(*) FROM ralph_iterations WHERE run_id = ?1",
                params![rid],
                |r| r.get(0),
            )?;
            let verified: i64 = conn.query_row(
                "SELECT count(*) FROM ralph_iterations WHERE run_id = ?1 AND verdict = 'verified'",
                params![rid],
                |r| r.get(0),
            )?;
            let failed: i64 = conn.query_row(
                "SELECT count(*) FROM ralph_iterations WHERE run_id = ?1 AND verdict = 'failed'",
                params![rid],
                |r| r.get(0),
            )?;
            let findings: i64 = conn.query_row(
                "SELECT count(*) FROM ralph_findings WHERE run_id = ?1",
                params![rid],
                |r| r.get(0),
            )?;

            let report = format!(
                "## Ralph Report for {rid}\n\n\
                 - **Project**: {project}\n\
                 - **Feature**: {feature}\n\
                 - **Status**: {status}\n\
                 - **Goal**: {goal}\n\
                 - **Iterations**: {iterations} (verified: {verified}, failed: {failed})\n\
                 - **Findings**: {findings}\n"
            );
            Ok(ToolResult {
                success: true,
                output: report.into(),
                error: None,
            })
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, project_id, feature_slug, status FROM ralph_runs ORDER BY created_at DESC LIMIT 10"
            )?;
            let rows = stmt.query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?))
            })?;
            let mut list = String::from("## Recent Ralph Runs\n\n");
            for r in rows.flatten() {
                list.push_str(&format!("- **{}** (project: {}, feature: {}, status: {})\n", r.0, r.1, r.2, r.3));
            }
            Ok(ToolResult {
                success: true,
                output: list.into(),
                error: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        omnesagent_kag::schema::migrate(&conn).unwrap();
        Arc::new(Mutex::new(conn))
    }

    #[tokio::test]
    async fn test_ralph_lifecycle() {
        let conn = test_conn();
        {
            let c = conn.lock();
            c.execute(
                "INSERT INTO projects (id, name, root_path, created_at, updated_at) VALUES ('p1', 'proj', '/tmp', 't0', 't0')",
                [],
            ).unwrap();
        }

        let start_tool = RalphStartTool::new(conn.clone());
        let res = start_tool.execute(json!({
            "project_id": "p1",
            "feature_slug": "feat-login",
            "goal": "Add OAuth2 authentication"
        })).await.unwrap();
        assert!(res.success);
        let out = res.output.as_str();
        assert!(out.contains("run_id=rrun_"));

        let run_id = out.split('=').nth(1).unwrap().split(' ').next().unwrap();

        let it_tool = RalphIterationTool::new(conn.clone());
        let it_res = it_tool.execute(json!({
            "run_id": run_id,
            "task_id": "task-oauth",
            "n": 1,
            "hypothesis": "Test OAuth endpoint",
            "tests_summary": "{\"passed\": 3, \"failed\": 0}"
        })).await.unwrap();
        assert!(it_res.success);
        assert!(it_res.output.as_str().contains("verdict=verified"));

        let rep_tool = RalphReportTool::new(conn.clone());
        let rep_res = rep_tool.execute(json!({"run_id": run_id})).await.unwrap();
        assert!(rep_res.success);
        assert!(rep_res.output.as_str().contains("**Iterations**: 1"));
    }
}
