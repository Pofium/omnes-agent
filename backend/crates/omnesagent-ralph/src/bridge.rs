//! ob2h knowledge bridge (Ф2).
//!
//! Forwards archived Ralph findings into the external ob2h knowledge base over the
//! ob2h MCP stdio contract (`ob2h serve`). Publication is deliberately *fail-open*:
//! any failure to reach ob2h is returned to the caller, which logs it and lets the
//! Ralph cycle finish (Ф2.2, ADR-O3). The bridge is gated by the `[ob2h_bridge]`
//! config (`enabled`, `push_scope`, `projects`, `auto_push_after_archive`) and by a
//! credential guard that never forwards secret-classified content (Ф2.5, ADR-O5).

use crate::debt::DebtLedger;
use crate::journal::{FindingRecord, IterationRecord, Journal, JournalError};
use crate::openspec::OpenSpecWorkspace;
use async_trait::async_trait;
use omnesagent_config::ralph::{BridgePushScope, Ob2hBridgeConfig};
use serde_json::{json, Value};
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::timeout;

/// Default per-request timeout for a single MCP round-trip to ob2h.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Tokens that mark a finding as carrying classified credential content. Mirrors
/// the secret-bearing variants of `CredentialSurfaceClass` plus the
/// `#[credential_class]` attribute name, the PEM private-key envelope, and the
/// Ф69.1 / Ф11.1 redaction token prefixes (ADR-O11).
const CREDENTIAL_MARKERS: &[&str] = &[
    "credential_class",
    "encrypted_secret",
    "external_auth_store",
    "-----BEGIN ",
    "sk-",
    "ghp_",
    "gho_",
    "ghs_",
    "ghu_",
    "Bearer ",
    "OB2H_",
];

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("failed to spawn ob2h command '{command}': {source}")]
    Spawn {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("ob2h did not answer within {0:?}")]
    Timeout(Duration),
    #[error("ob2h stdio I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ob2h MCP protocol error: {0}")]
    Protocol(String),
    #[error("ob2h reported an error calling '{tool}': {message}")]
    Tool { tool: String, message: String },
    #[error("journal error: {0}")]
    Journal(#[from] JournalError),
}

/// Outcome of a single bridge publication attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BridgeReport {
    /// `memory_save` calls that ob2h acknowledged.
    pub published: usize,
    /// Findings excluded by `push_scope` or the credential guard.
    pub skipped: usize,
    /// `memory_save` calls that failed (counted per project target).
    pub failed: usize,
    /// `lesson_save` calls that ob2h acknowledged (Ф8.2, ADR-O8).
    pub lessons_published: usize,
    /// Findings excluded from lesson publication by scope or the credential guard.
    pub lessons_skipped: usize,
    /// `lesson_save` calls that failed (counted per project target).
    pub lessons_failed: usize,
}

/// Whether a finding may be forwarded under `scope` and the credential guard.
pub fn is_publishable(finding: &FindingRecord, scope: BridgePushScope) -> bool {
    let scope_allows = match scope {
        BridgePushScope::Verified => finding.verdict == "verified",
        BridgePushScope::All => true,
    };
    scope_allows && !carries_credential_class(&finding.content)
}

/// Whether `content` carries a credential-classified marker (Ф2.5, ADR-O5).
pub fn carries_credential_class(content: &str) -> bool {
    CREDENTIAL_MARKERS
        .iter()
        .any(|marker| content.contains(marker))
}

/// Minimal MCP client surface the bridge needs. Abstracted so tests can inject an
/// in-memory transport instead of spawning a real `ob2h serve` process.
#[async_trait]
pub trait Ob2hTransport: Send {
    /// Perform the MCP handshake (`initialize` + `notifications/initialized`).
    async fn initialize(&mut self) -> Result<(), BridgeError>;
    /// Invoke an MCP tool and return its textual result.
    async fn call_tool(&mut self, tool: &str, arguments: Value) -> Result<String, BridgeError>;
    /// Close the transport, letting the peer observe EOF.
    async fn shutdown(&mut self);
}

/// MCP stdio client speaking newline-delimited JSON-RPC 2.0 to `ob2h serve`.
pub struct McpStdioClient {
    command: String,
    args: Vec<String>,
    session: Option<StdioSession>,
}

struct StdioSession {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl McpStdioClient {
    /// Build a client for `command args` without spawning it yet.
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            command: command.into(),
            args,
            session: None,
        }
    }

    /// Spawn the process on first use and return the live session.
    fn session(&mut self) -> Result<&mut StdioSession, BridgeError> {
        if self.session.is_none() {
            let mut child = Command::new(&self.command)
                .args(&self.args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|source| BridgeError::Spawn {
                    command: self.command.clone(),
                    source,
                })?;
            let stdin = child.stdin.take();
            let stdout = child
                .stdout
                .take()
                .map(BufReader::new)
                .ok_or_else(|| BridgeError::Protocol("ob2h produced no stdout pipe".into()))?;
            self.session = Some(StdioSession {
                child,
                stdin,
                stdout,
                next_id: 1,
            });
        }
        self.session
            .as_mut()
            .ok_or_else(|| BridgeError::Protocol("ob2h session unavailable".into()))
    }

    async fn request(&mut self, method: &str, params: Value) -> Result<Value, BridgeError> {
        let session = self.session()?;
        let id = session.next_id;
        session.next_id += 1;
        let payload = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        write_line(session, &payload).await?;
        read_response(session, id).await
    }

    async fn notify(&mut self, method: &str, params: Value) -> Result<(), BridgeError> {
        let session = self.session()?;
        let payload = json!({ "jsonrpc": "2.0", "method": method, "params": params });
        write_line(session, &payload).await
    }
}

async fn write_line(session: &mut StdioSession, value: &Value) -> Result<(), BridgeError> {
    let mut line =
        serde_json::to_string(value).map_err(|e| BridgeError::Protocol(e.to_string()))?;
    line.push('\n');
    let stdin = session
        .stdin
        .as_mut()
        .ok_or_else(|| BridgeError::Protocol("ob2h stdin closed".into()))?;
    stdin.write_all(line.as_bytes()).await?;
    stdin.flush().await?;
    Ok(())
}

async fn read_response(session: &mut StdioSession, id: u64) -> Result<Value, BridgeError> {
    loop {
        let mut line = String::new();
        let read = session.stdout.read_line(&mut line).await?;
        if read == 0 {
            return Err(BridgeError::Protocol("ob2h closed the connection".into()));
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(trimmed)
            .map_err(|e| BridgeError::Protocol(format!("invalid JSON from ob2h: {e}")))?;
        if value.get("id").and_then(Value::as_u64) == Some(id) {
            return Ok(value);
        }
        // Notifications and unrelated responses carry no matching id: ignore them.
    }
}

fn error_message(error: &Value) -> String {
    error
        .get("message")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| error.to_string())
}

#[async_trait]
impl Ob2hTransport for McpStdioClient {
    async fn initialize(&mut self) -> Result<(), BridgeError> {
        let response = self
            .request(
                "initialize",
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "omnesagent-ralph",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                }),
            )
            .await?;
        if let Some(error) = response.get("error") {
            return Err(BridgeError::Tool {
                tool: "initialize".into(),
                message: error_message(error),
            });
        }
        self.notify("notifications/initialized", json!({})).await
    }

    async fn call_tool(&mut self, tool: &str, arguments: Value) -> Result<String, BridgeError> {
        let response = self
            .request("tools/call", json!({ "name": tool, "arguments": arguments }))
            .await?;
        if let Some(error) = response.get("error") {
            return Err(BridgeError::Tool {
                tool: tool.to_string(),
                message: error_message(error),
            });
        }
        let result = response.get("result").cloned().unwrap_or(Value::Null);
        let text = result
            .get("content")
            .and_then(Value::as_array)
            .and_then(|items| {
                items
                    .iter()
                    .find_map(|item| item.get("text").and_then(Value::as_str))
            })
            .unwrap_or_default()
            .to_string();
        if result
            .get("isError")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return Err(BridgeError::Tool {
                tool: tool.to_string(),
                message: text,
            });
        }
        Ok(text)
    }

    async fn shutdown(&mut self) {
        if let Some(session) = self.session.as_mut() {
            // Closing stdin lets `ob2h serve` observe EOF and exit its loop.
            session.stdin.take();
            if timeout(Duration::from_secs(3), session.child.wait())
                .await
                .is_err()
            {
                let _ = session.child.kill().await;
                let _ = session.child.wait().await;
            }
        }
    }
}

type TransportFactory = Arc<dyn Fn() -> Box<dyn Ob2hTransport> + Send + Sync>;

/// Publishes archived Ralph findings into ob2h (Ф2.2).
pub struct Ob2hBridge {
    config: Ob2hBridgeConfig,
    request_timeout: Duration,
    factory: TransportFactory,
}

impl Ob2hBridge {
    /// Build a bridge that talks to `ob2h serve` using `config`.
    pub fn from_config(config: Ob2hBridgeConfig) -> Self {
        Self::with_timeout(config, DEFAULT_REQUEST_TIMEOUT)
    }

    /// Build a bridge with an explicit per-request timeout.
    pub fn with_timeout(config: Ob2hBridgeConfig, request_timeout: Duration) -> Self {
        let command = config.ob2h_command.clone();
        let args = config.ob2h_args.clone();
        let factory: TransportFactory = Arc::new(move || {
            Box::new(McpStdioClient::new(command.clone(), args.clone())) as Box<dyn Ob2hTransport>
        });
        Self {
            config,
            request_timeout,
            factory,
        }
    }

    #[cfg(test)]
    fn with_factory(
        config: Ob2hBridgeConfig,
        request_timeout: Duration,
        factory: TransportFactory,
    ) -> Self {
        Self {
            config,
            request_timeout,
            factory,
        }
    }

    /// Whether the bridge is configured to publish at all.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Read the run's findings from the journal and publish those selected by
    /// `push_scope`. A disabled bridge is a no-op.
    pub async fn publish_after_archive(
        &self,
        repo_root: &Path,
        slug: &str,
    ) -> Result<BridgeReport, BridgeError> {
        if !self.config.enabled {
            return Ok(BridgeReport::default());
        }
        let journal_path = OpenSpecWorkspace::new(repo_root, slug).journal_path();
        let findings = Journal::read_findings(&journal_path)?;
        self.publish_findings(&findings).await
    }

    /// Publish `findings`, one `memory_save` per selected finding per project
    /// target. Returns a per-attempt report; individual failures are counted, not
    /// propagated, so one bad record cannot abort the batch.
    pub async fn publish_findings(
        &self,
        findings: &[FindingRecord],
    ) -> Result<BridgeReport, BridgeError> {
        let selected: Vec<&FindingRecord> = findings
            .iter()
            .filter(|finding| is_publishable(finding, self.config.push_scope))
            .collect();

        let mut report = BridgeReport {
            skipped: findings.len() - selected.len(),
            ..BridgeReport::default()
        };
        if selected.is_empty() {
            return Ok(report);
        }

        let targets: Vec<Option<&str>> = if self.config.projects.is_empty() {
            vec![None]
        } else {
            self.config
                .projects
                .iter()
                .map(|project| Some(project.as_str()))
                .collect()
        };

        let mut transport = (self.factory)();
        self.round_trip(transport.initialize()).await?;

        for finding in &selected {
            for target in &targets {
                let mut arguments = json!({
                    "content": finding.content,
                    "key": finding.finding_id,
                    "category": "ralph_finding",
                    "importance": 0.7,
                    "source": "ralph",
                });
                if let Some(project) = target {
                    arguments["project_id"] = json!(project);
                }
                match self
                    .round_trip(transport.call_tool("memory_save", arguments))
                    .await
                {
                    Ok(_) => report.published += 1,
                    Err(_) => report.failed += 1,
                }
            }
        }

        transport.shutdown().await;
        Ok(report)
    }

    /// Export the fork's harvested debt ledger into ob2h's Ralph Knowledge Layer
    /// (Ф3.1) so that `ob2h ralph debt` surfaces the same deferred markers.
    ///
    /// ob2h has no standalone "add finding" tool, so the bridge opens a run
    /// (`ralph_start`) and records the debt items as a single `deferred`
    /// iteration (`ralph_iteration`). The feature slug is suffixed with the fork
    /// run id, which keeps repeated archives from colliding with ob2h's
    /// "one active run per (project, feature)" guard. A disabled bridge or an
    /// empty project list is a no-op; individual failures are counted, never
    /// propagated (fail-open, ADR-O3).
    pub async fn publish_debt(
        &self,
        ledger: &DebtLedger,
        slug: &str,
        run_id: &str,
        goal: &str,
    ) -> Result<BridgeReport, BridgeError> {
        if !self.config.enabled || self.config.projects.is_empty() {
            return Ok(BridgeReport {
                skipped: ledger.items.len(),
                ..BridgeReport::default()
            });
        }

        // Credential guard (ADR-O5): never forward secret-classified debt text.
        let items: Vec<&crate::debt::DebtItem> = ledger
            .items
            .iter()
            .filter(|item| {
                !carries_credential_class(&item.raw_marker)
                    && !carries_credential_class(&item.ceiling)
            })
            .collect();

        let mut report = BridgeReport {
            skipped: ledger.items.len() - items.len(),
            ..BridgeReport::default()
        };
        if items.is_empty() {
            return Ok(report);
        }

        let findings: Vec<Value> = items
            .iter()
            .map(|item| {
                json!({
                    "kind": "deferred",
                    "content": item.raw_marker,
                    "meta": {
                        "ceiling": item.ceiling,
                        "upgrade_trigger": item.upgrade_trigger,
                        "file": item.file,
                        "line": item.line,
                        "no_trigger": item.tags.iter().any(|t| t == "no-trigger"),
                    },
                })
            })
            .collect();

        let feature_slug = format!("{slug}@{run_id}");

        let mut transport = (self.factory)();
        self.round_trip(transport.initialize()).await?;

        for project in &self.config.projects {
            let start = self
                .round_trip(transport.call_tool(
                    "ralph_start",
                    json!({
                        "project_id": project,
                        "feature_slug": feature_slug,
                        "goal": goal,
                    }),
                ))
                .await;
            let Some(ob2h_run_id) = start.ok().as_deref().and_then(parse_run_id) else {
                report.failed += 1;
                continue;
            };

            match self
                .round_trip(transport.call_tool(
                    "ralph_iteration",
                    json!({
                        "run_id": ob2h_run_id,
                        "task_id": "debt",
                        "n": 1,
                        "findings": findings,
                    }),
                ))
                .await
            {
                Ok(_) => report.published += items.len(),
                Err(_) => report.failed += 1,
            }
        }

        transport.shutdown().await;
        Ok(report)
    }

    /// Publish archived findings as procedural lessons (Ф8.2, ADR-O8).
    ///
    /// One `lesson_save` per selected finding per project target. A disabled
    /// bridge or a `push_lessons = false` config is a no-op; individual failures
    /// are counted, never propagated (fail-open, ADR-O3). ob2h deduplicates by
    /// fingerprint, so re-publishing an unchanged finding reinforces the lesson
    /// instead of creating a duplicate.
    pub async fn publish_lessons(
        &self,
        findings: &[FindingRecord],
        feature_slug: &str,
    ) -> Result<BridgeReport, BridgeError> {
        if !self.config.enabled || !self.config.push_lessons {
            return Ok(BridgeReport::default());
        }

        let selected: Vec<&FindingRecord> = findings
            .iter()
            .filter(|finding| is_publishable(finding, self.config.push_scope))
            .collect();

        let mut report = BridgeReport {
            lessons_skipped: findings.len() - selected.len(),
            ..BridgeReport::default()
        };
        if selected.is_empty() {
            return Ok(report);
        }

        let targets: Vec<Option<&str>> = if self.config.projects.is_empty() {
            vec![None]
        } else {
            self.config
                .projects
                .iter()
                .map(|project| Some(project.as_str()))
                .collect()
        };

        let mut transport = (self.factory)();
        self.round_trip(transport.initialize()).await?;

        for finding in &selected {
            for target in &targets {
                let mut arguments = json!({
                    "content": finding.content,
                    "context": lesson_context(finding),
                    "tags": ["ralph", feature_slug],
                    "source": "manual",
                });
                if let Some(project) = target {
                    arguments["project"] = json!(project);
                }
                match self
                    .round_trip(transport.call_tool("lesson_save", arguments))
                    .await
                {
                    Ok(_) => report.lessons_published += 1,
                    Err(_) => report.lessons_failed += 1,
                }
            }
        }

        transport.shutdown().await;
        Ok(report)
    }

    /// Recall procedural lessons relevant to the next iteration (Ф8.3, ADR-O8).
    ///
    /// Returns the lesson text wrapped in a `<lessons>` block, or `None` when the
    /// lessons loop is disabled or ob2h returned nothing. Fail-open by design:
    /// callers treat an `Err` (timeout, unreachable ob2h) as "no lessons this
    /// iteration" and continue the cycle.
    pub async fn recall_lessons(
        &self,
        query: &str,
        min_confidence: f64,
        limit: usize,
    ) -> Result<Option<String>, BridgeError> {
        if !self.config.enabled || !self.config.push_lessons {
            return Ok(None);
        }

        let mut arguments = json!({
            "query": query,
            "limit": limit,
            "min_confidence": min_confidence,
        });
        if let Some(project) = self.config.projects.first() {
            arguments["project"] = json!(project);
        }

        let mut transport = (self.factory)();
        self.round_trip(transport.initialize()).await?;
        let result = self
            .round_trip(transport.call_tool("lesson_recall", arguments))
            .await;
        transport.shutdown().await;

        let text = result?;
        let trimmed = text.trim();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(format!("<lessons>\n{trimmed}\n</lessons>")))
        }
    }

    /// Ingest the run journal and save a session recap so that published facts
    /// carry provenance (Ф10.1, ADR-O9).
    ///
    /// `session_ingest` is position-idempotent, so re-archiving a run does not
    /// duplicate messages; `session_recap(save = true)` then records the single
    /// fact that `memory_verify` can trace back to the daily log. A disabled
    /// bridge or a `recap_on_archive = false` config is a no-op; failures are
    /// counted, never propagated (fail-open).
    pub async fn recap_after_archive(
        &self,
        repo_root: &Path,
        slug: &str,
        run_id: &str,
    ) -> Result<BridgeReport, BridgeError> {
        if !self.config.enabled || !self.config.recap_on_archive {
            return Ok(BridgeReport::default());
        }

        let journal_path = OpenSpecWorkspace::new(repo_root, slug).journal_path();
        let iterations = Journal::read_iterations(&journal_path)?;
        // Credential guard (ADR-O5/O11): never forward secret-classified text to
        // ob2h, even transitively through journal messages.
        let messages: Vec<Value> = build_journal_messages(&iterations)
            .into_iter()
            .filter(|message| {
                message
                    .get("content")
                    .and_then(Value::as_str)
                    .map(|content| !carries_credential_class(content))
                    .unwrap_or(true)
            })
            .collect();
        if messages.is_empty() {
            return Ok(BridgeReport::default());
        }

        let session_id = format!("ralph-{run_id}");
        let targets: Vec<Option<&str>> = if self.config.projects.is_empty() {
            vec![None]
        } else {
            self.config
                .projects
                .iter()
                .map(|project| Some(project.as_str()))
                .collect()
        };

        let mut transport = (self.factory)();
        self.round_trip(transport.initialize()).await?;

        let mut report = BridgeReport::default();
        for target in &targets {
            let mut ingest = json!({
                "messages": messages,
                "session_id": session_id,
                "author": "omnesagent-ralph",
                "source": "ralph",
            });
            if let Some(project) = target {
                ingest["project_id"] = json!(project);
            }
            if self
                .round_trip(transport.call_tool("session_ingest", ingest))
                .await
                .is_err()
            {
                report.failed += 1;
                continue;
            }

            let mut recap = json!({ "session_id": session_id, "save": true });
            if let Some(project) = target {
                recap["project_id"] = json!(project);
            }
            match self
                .round_trip(transport.call_tool("session_recap", recap))
                .await
            {
                Ok(_) => report.published += 1,
                Err(_) => report.failed += 1,
            }
        }

        transport.shutdown().await;
        Ok(report)
    }

    /// Bound a single transport operation by `request_timeout`.
    async fn round_trip<F, T>(&self, future: F) -> Result<T, BridgeError>
    where
        F: std::future::Future<Output = Result<T, BridgeError>>,
    {
        match timeout(self.request_timeout, future).await {
            Ok(result) => result,
            Err(_) => Err(BridgeError::Timeout(self.request_timeout)),
        }
    }
}

/// Extract the `run_id=...` token from an ob2h `ralph_start` reply.
fn parse_run_id(reply: &str) -> Option<String> {
    let rest = reply.split("run_id=").nth(1)?;
    let id = rest.split_whitespace().next()?;
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

/// Compose the `context` anchor for a lesson derived from `finding` (ADR-O8).
///
/// The lesson `content` is the finding itself (what to do / what to avoid); the
/// context anchors it to the originating task and the symbols the finding
/// touched, so `lesson_recall` can rank it against a future task's scope.
fn lesson_context(finding: &FindingRecord) -> String {
    if finding.symbols.is_empty() {
        format!("task={}", finding.task)
    } else {
        format!("task={} symbols={}", finding.task, finding.symbols.join(","))
    }
}

/// Flatten journal iterations into `session_ingest` user/assistant pairs (Ф10.1).
///
/// Each iteration contributes a `user` message (the hypothesis and plan Ralph
/// set out to execute) followed by an `assistant` message (what was done, any
/// errors, and the resulting verdict). The mapping is deterministic, so the same
/// journal yields the same message positions on every archive — which is what
/// makes `session_ingest` position-idempotent.
fn build_journal_messages(iterations: &[IterationRecord]) -> Vec<Value> {
    let mut messages = Vec::with_capacity(iterations.len() * 2);
    for iteration in iterations {
        let mut request = format!(
            "[{} #{}] {}",
            iteration.task, iteration.n, iteration.hypothesis
        );
        if !iteration.plan.is_empty() {
            request.push_str("\nplan:\n");
            for step in &iteration.plan {
                request.push_str("- ");
                request.push_str(step);
                request.push('\n');
            }
        }
        let mut outcome = iteration.result.what_done.clone();
        if !iteration.result.errors.is_empty() {
            outcome.push_str("\nerrors: ");
            outcome.push_str(&iteration.result.errors.join("; "));
        }
        outcome.push_str(&format!(
            "\nverdict: {} (tests {}/{} passed)",
            iteration.verdict,
            iteration.tests_summary.passed,
            iteration.tests_summary.passed + iteration.tests_summary.failed
        ));

        if carries_credential_class(&request) || carries_credential_class(&outcome) {
            continue;
        }
        messages.push(json!({ "role": "user", "content": request }));
        messages.push(json!({ "role": "assistant", "content": outcome }));
    }
    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journal::{ExecutionResult, JournalEntry, TestsSummary};
    use chrono::Utc;
    use std::sync::Mutex;

    fn finding(id: &str, verdict: &str, content: &str) -> FindingRecord {
        FindingRecord {
            finding_id: id.to_string(),
            task: "T-1".to_string(),
            content: content.to_string(),
            verdict: verdict.to_string(),
            symbols: Vec::new(),
            git_head_sha: None,
            at: Utc::now(),
        }
    }

    /// In-memory transport recording every `memory_save`; optionally never answers.
    struct MockTransport {
        saved: Arc<Mutex<Vec<Value>>>,
        hang: bool,
    }

    impl MockTransport {
        fn recording() -> (Arc<Mutex<Vec<Value>>>, Self) {
            let saved = Arc::new(Mutex::new(Vec::new()));
            (
                saved.clone(),
                Self {
                    saved,
                    hang: false,
                },
            )
        }

        fn hanging() -> Self {
            Self {
                saved: Arc::new(Mutex::new(Vec::new())),
                hang: true,
            }
        }
    }

    #[async_trait]
    impl Ob2hTransport for MockTransport {
        async fn initialize(&mut self) -> Result<(), BridgeError> {
            if self.hang {
                return std::future::pending().await;
            }
            Ok(())
        }

        async fn call_tool(&mut self, tool: &str, arguments: Value) -> Result<String, BridgeError> {
            if self.hang {
                return std::future::pending().await;
            }
            self.saved.lock().unwrap().push(arguments);
            if tool == "ralph_start" {
                return Ok("run_id=rrun_test status=applying".to_string());
            }
            Ok("saved".to_string())
        }

        async fn shutdown(&mut self) {}
    }

    fn factory_for(mock: MockTransport) -> TransportFactory {
        Arc::new(move || {
            let saved = mock.saved.clone();
            let hang = mock.hang;
            Box::new(MockTransport { saved, hang }) as Box<dyn Ob2hTransport>
        })
    }

    #[tokio::test]
    async fn publishes_only_verified_findings_under_verified_scope() {
        let (saved, mock) = MockTransport::recording();
        let config = Ob2hBridgeConfig {
            enabled: true,
            projects: vec!["omnes-agent".to_string()],
            push_scope: BridgePushScope::Verified,
            ..Ob2hBridgeConfig::default()
        };
        let bridge =
            Ob2hBridge::with_factory(config, Duration::from_secs(1), factory_for(mock));

        let report = bridge
            .publish_findings(&[
                finding("f1", "verified", "good"),
                finding("f2", "failed", "bad"),
            ])
            .await
            .unwrap();

        assert_eq!(report.published, 1);
        assert_eq!(report.skipped, 1);
        assert_eq!(report.failed, 0);

        let saved = saved.lock().unwrap();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0]["key"], "f1");
        assert_eq!(saved[0]["project_id"], "omnes-agent");
    }

    #[tokio::test]
    async fn all_scope_publishes_every_finding() {
        let (saved, mock) = MockTransport::recording();
        let config = Ob2hBridgeConfig {
            enabled: true,
            push_scope: BridgePushScope::All,
            ..Ob2hBridgeConfig::default()
        };
        let bridge =
            Ob2hBridge::with_factory(config, Duration::from_secs(1), factory_for(mock));

        let report = bridge
            .publish_findings(&[
                finding("f1", "verified", "good"),
                finding("f2", "unconfirmed", "maybe"),
            ])
            .await
            .unwrap();

        assert_eq!(report.published, 2);
        assert_eq!(report.skipped, 0);
        assert_eq!(saved.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn times_out_when_ob2h_never_answers() {
        let config = Ob2hBridgeConfig {
            enabled: true,
            ..Ob2hBridgeConfig::default()
        };
        let bridge = Ob2hBridge::with_factory(
            config,
            Duration::from_millis(50),
            factory_for(MockTransport::hanging()),
        );

        let err = bridge
            .publish_findings(&[finding("f1", "verified", "x")])
            .await
            .unwrap_err();

        assert!(matches!(err, BridgeError::Timeout(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn reports_missing_ob2h_binary() {
        let config = Ob2hBridgeConfig {
            enabled: true,
            ob2h_command: "omnesagent-definitely-missing-ob2h-binary".to_string(),
            ..Ob2hBridgeConfig::default()
        };
        let bridge = Ob2hBridge::from_config(config);

        let err = bridge
            .publish_findings(&[finding("f1", "verified", "x")])
            .await
            .unwrap_err();

        assert!(matches!(err, BridgeError::Spawn { .. }), "got {err:?}");
    }

    #[tokio::test]
    async fn disabled_bridge_is_a_noop() {
        let bridge = Ob2hBridge::from_config(Ob2hBridgeConfig::default());
        assert!(!bridge.is_enabled());

        let report = bridge
            .publish_after_archive(Path::new("."), "no-such-slug")
            .await
            .unwrap();

        assert_eq!(report, BridgeReport::default());
    }

    #[test]
    fn credential_class_content_is_never_published() {
        let record = finding("f1", "verified", "token = encrypted_secret: abc123");
        assert!(carries_credential_class(&record.content));
        assert!(!is_publishable(&record, BridgePushScope::All));
        assert!(!is_publishable(&record, BridgePushScope::Verified));
    }

    #[test]
    fn plain_content_is_publishable() {
        let verified = finding("f1", "verified", "Refactored the parser entry point");
        assert!(is_publishable(&verified, BridgePushScope::Verified));
        assert!(!is_publishable(
            &finding("f2", "failed", "whatever"),
            BridgePushScope::Verified
        ));
    }

    fn debt_item(marker: &str) -> crate::debt::DebtItem {
        crate::debt::DebtItem {
            file: "src/cache.rs".to_string(),
            line: 12,
            raw_marker: marker.to_string(),
            ceiling: "in-memory buffer up to 100".to_string(),
            upgrade_trigger: Some("migrate to sqlite".to_string()),
            tags: Vec::new(),
        }
    }

    #[tokio::test]
    async fn publishes_debt_as_deferred_findings() {
        let (saved, mock) = MockTransport::recording();
        let config = Ob2hBridgeConfig {
            enabled: true,
            projects: vec!["omnes-agent".to_string()],
            ..Ob2hBridgeConfig::default()
        };
        let bridge = Ob2hBridge::with_factory(config, Duration::from_secs(1), factory_for(mock));

        let ledger = DebtLedger {
            items: vec![debt_item("ponytail: in-memory buffer up to 100, migrate to sqlite")],
        };
        let report = bridge
            .publish_debt(&ledger, "test-feature", "run_1", "ship it")
            .await
            .unwrap();

        assert_eq!(report.published, 1);
        assert_eq!(report.failed, 0);

        let saved = saved.lock().unwrap();
        assert_eq!(saved.len(), 2, "expected ralph_start + ralph_iteration");

        let start = saved.iter().find(|a| a.get("feature_slug").is_some()).unwrap();
        assert_eq!(start["project_id"], "omnes-agent");
        assert_eq!(start["feature_slug"], "test-feature@run_1");

        let iteration = saved
            .iter()
            .find(|a| a.get("findings").is_some())
            .unwrap();
        assert_eq!(iteration["run_id"], "rrun_test");
        assert_eq!(iteration["findings"][0]["kind"], "deferred");
        assert_eq!(
            iteration["findings"][0]["meta"]["upgrade_trigger"],
            "migrate to sqlite"
        );
    }

    #[tokio::test]
    async fn debt_with_credentials_is_skipped() {
        let (saved, mock) = MockTransport::recording();
        let config = Ob2hBridgeConfig {
            enabled: true,
            projects: vec!["omnes-agent".to_string()],
            ..Ob2hBridgeConfig::default()
        };
        let bridge = Ob2hBridge::with_factory(config, Duration::from_secs(1), factory_for(mock));

        let ledger = DebtLedger {
            items: vec![debt_item("ponytail: pin stored as encrypted_secret, rotate later")],
        };
        let report = bridge
            .publish_debt(&ledger, "test-feature", "run_1", "ship it")
            .await
            .unwrap();

        assert_eq!(report.published, 0);
        assert_eq!(report.skipped, 1);
        assert!(saved.lock().unwrap().is_empty(), "nothing may reach ob2h");
    }

    #[tokio::test]
    async fn disabled_bridge_skips_debt_export() {
        let bridge = Ob2hBridge::from_config(Ob2hBridgeConfig::default());
        let ledger = DebtLedger {
            items: vec![debt_item("ponytail: temporary shim, revisit after v2")],
        };
        let report = bridge
            .publish_debt(&ledger, "test-feature", "run_1", "ship it")
            .await
            .unwrap();

        assert_eq!(report, BridgeReport {
            skipped: 1,
            ..BridgeReport::default()
        });
    }

    // --- Ф8.5 / Ф10.5: lessons and recap mock coverage -----------------------

    fn iteration(n: usize, hypothesis: &str, what_done: &str, verdict: &str) -> IterationRecord {
        IterationRecord {
            task: "T-1".to_string(),
            n,
            hypothesis: hypothesis.to_string(),
            plan: vec!["do the thing".to_string()],
            result: ExecutionResult {
                what_done: what_done.to_string(),
                errors: Vec::new(),
                self_assessment: String::new(),
            },
            tests_summary: TestsSummary {
                passed: 1,
                failed: 0,
                failed_tests: Vec::new(),
                fingerprint: "sha256:x".to_string(),
                exit_code: 0,
            },
            verdict: verdict.to_string(),
            verdict_source: "auto_tests".to_string(),
            ladder_rung: "stdlib".to_string(),
            markers: Vec::new(),
            ast_changes: Vec::new(),
            git_before: None,
            git_after: None,
            tokens_used: 0,
            duration_ms: 0,
            at: Utc::now(),
        }
    }

    fn lesson_config() -> Ob2hBridgeConfig {
        Ob2hBridgeConfig {
            enabled: true,
            projects: vec!["omnes-agent".to_string()],
            push_scope: BridgePushScope::Verified,
            push_lessons: true,
            ..Ob2hBridgeConfig::default()
        }
    }

    #[test]
    fn lesson_context_anchors_task_and_symbols() {
        let mut record = finding("f1", "verified", "prefer ok_or_else over unwrap");
        assert_eq!(lesson_context(&record), "task=T-1");
        record.symbols = vec!["fn:parse".to_string(), "fn:write".to_string()];
        assert_eq!(lesson_context(&record), "task=T-1 symbols=fn:parse,fn:write");
    }

    #[tokio::test]
    async fn publish_lessons_sends_one_lesson_save_per_verified_finding() {
        let (saved, mock) = MockTransport::recording();
        let bridge =
            Ob2hBridge::with_factory(lesson_config(), Duration::from_secs(1), factory_for(mock));

        let report = bridge
            .publish_lessons(
                &[
                    finding("f1", "verified", "use ok_or_else"),
                    finding("f2", "failed", "bad idea"),
                ],
                "feat",
            )
            .await
            .unwrap();

        assert_eq!(report.lessons_published, 1);
        assert_eq!(report.lessons_skipped, 1);
        assert_eq!(report.lessons_failed, 0);

        let saved = saved.lock().unwrap();
        assert_eq!(saved.len(), 1, "one lesson_save per verified finding");
        assert_eq!(saved[0]["content"], "use ok_or_else");
        assert_eq!(saved[0]["context"], "task=T-1");
        assert_eq!(saved[0]["tags"][0], "ralph");
        assert_eq!(saved[0]["tags"][1], "feat");
        assert_eq!(saved[0]["source"], "manual");
        assert_eq!(saved[0]["project"], "omnes-agent");
    }

    #[tokio::test]
    async fn publish_lessons_respects_credential_guard() {
        let (saved, mock) = MockTransport::recording();
        let bridge =
            Ob2hBridge::with_factory(lesson_config(), Duration::from_secs(1), factory_for(mock));

        let report = bridge
            .publish_lessons(
                &[finding("f1", "verified", "token = encrypted_secret: abc")],
                "feat",
            )
            .await
            .unwrap();

        assert_eq!(report.lessons_published, 0);
        assert_eq!(report.lessons_skipped, 1);
        assert!(saved.lock().unwrap().is_empty(), "nothing may reach ob2h");
    }

    #[tokio::test]
    async fn publish_lessons_drops_api_key_and_token_patterns() {
        let (saved, mock) = MockTransport::recording();
        let bridge =
            Ob2hBridge::with_factory(lesson_config(), Duration::from_secs(1), factory_for(mock));

        let report = bridge
            .publish_lessons(
                &[
                    finding("f1", "verified", "OpenAI key sk-1234567890abcdef1234567890"),
                    finding("f2", "verified", "GitHub token ghp_1234567890abcdef1234567890"),
                    finding("f3", "verified", "Header Bearer supersecrettoken"),
                    finding("f4", "verified", "Env OB2H_AUTH_TOKEN=xyz"),
                    finding("f5", "verified", "safe pattern ok"),
                ],
                "feat",
            )
            .await
            .unwrap();

        assert_eq!(report.lessons_published, 1);
        assert_eq!(report.lessons_skipped, 4);
        let saved = saved.lock().unwrap();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0]["content"], "safe pattern ok");
    }

    #[tokio::test]
    async fn publish_lessons_is_noop_when_push_lessons_disabled() {
        let (saved, mock) = MockTransport::recording();
        let config = Ob2hBridgeConfig {
            enabled: true,
            projects: vec!["omnes-agent".to_string()],
            ..Ob2hBridgeConfig::default()
        };
        let bridge = Ob2hBridge::with_factory(config, Duration::from_secs(1), factory_for(mock));

        let report = bridge
            .publish_lessons(&[finding("f1", "verified", "x")], "feat")
            .await
            .unwrap();

        assert_eq!(report, BridgeReport::default());
        assert!(saved.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn recall_lessons_wraps_text_in_lessons_block() {
        let (_, mock) = MockTransport::recording();
        let bridge =
            Ob2hBridge::with_factory(lesson_config(), Duration::from_secs(1), factory_for(mock));

        let block = bridge.recall_lessons("task T-1", 0.2, 5).await.unwrap();

        assert_eq!(block, Some("<lessons>\nsaved\n</lessons>".to_string()));
    }

    #[tokio::test]
    async fn recall_lessons_is_noop_when_disabled() {
        let bridge = Ob2hBridge::from_config(Ob2hBridgeConfig::default());
        assert_eq!(bridge.recall_lessons("q", 0.2, 5).await.unwrap(), None);
    }

    #[tokio::test]
    async fn recall_lessons_fails_open_on_timeout() {
        let bridge = Ob2hBridge::with_factory(
            lesson_config(),
            Duration::from_millis(50),
            factory_for(MockTransport::hanging()),
        );

        let err = bridge.recall_lessons("q", 0.2, 5).await.unwrap_err();
        assert!(matches!(err, BridgeError::Timeout(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn recap_after_archive_ingests_then_recaps() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = OpenSpecWorkspace::new(dir.path(), "feat");
        workspace.init().unwrap();
        Journal::append(
            &workspace.journal_path(),
            &JournalEntry::Iteration(iteration(1, "try approach A", "applied A", "verified")),
        )
        .unwrap();

        let (saved, mock) = MockTransport::recording();
        let config = Ob2hBridgeConfig {
            enabled: true,
            projects: vec!["omnes-agent".to_string()],
            recap_on_archive: true,
            ..Ob2hBridgeConfig::default()
        };
        let bridge = Ob2hBridge::with_factory(config, Duration::from_secs(1), factory_for(mock));

        let report = bridge
            .recap_after_archive(dir.path(), "feat", "run_1")
            .await
            .unwrap();

        assert_eq!(report.published, 1);
        assert_eq!(report.failed, 0);

        let saved = saved.lock().unwrap();
        let ingest = saved
            .iter()
            .find(|a| a.get("messages").is_some())
            .expect("session_ingest called");
        assert_eq!(ingest["session_id"], "ralph-run_1");
        assert_eq!(ingest["author"], "omnesagent-ralph");
        assert_eq!(ingest["source"], "ralph");
        assert_eq!(ingest["project_id"], "omnes-agent");
        assert_eq!(ingest["messages"][0]["role"], "user");
        assert_eq!(ingest["messages"][1]["role"], "assistant");

        let recap = saved
            .iter()
            .find(|a| a.get("save").is_some())
            .expect("session_recap called");
        assert_eq!(recap["session_id"], "ralph-run_1");
        assert_eq!(recap["save"], true);
    }

    #[tokio::test]
    async fn recap_after_archive_drops_credential_messages() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = OpenSpecWorkspace::new(dir.path(), "feat");
        workspace.init().unwrap();
        Journal::append(
            &workspace.journal_path(),
            &JournalEntry::Iteration(iteration(
                1,
                "rotate pin encrypted_secret: deadbeef",
                "done",
                "verified",
            )),
        )
        .unwrap();

        let (saved, mock) = MockTransport::recording();
        let config = Ob2hBridgeConfig {
            enabled: true,
            projects: vec!["omnes-agent".to_string()],
            recap_on_archive: true,
            ..Ob2hBridgeConfig::default()
        };
        let bridge = Ob2hBridge::with_factory(config, Duration::from_secs(1), factory_for(mock));

        let report = bridge
            .recap_after_archive(dir.path(), "feat", "run_1")
            .await
            .unwrap();

        assert_eq!(report, BridgeReport::default());
        assert!(saved.lock().unwrap().is_empty(), "nothing may reach ob2h");
    }

    #[tokio::test]
    async fn recap_after_archive_drops_api_keys_and_bearer_tokens() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = OpenSpecWorkspace::new(dir.path(), "feat");
        workspace.init().unwrap();
        Journal::append(
            &workspace.journal_path(),
            &JournalEntry::Iteration(iteration(
                1,
                "set Authorization: Bearer secret_live_token",
                "attempt with sk-proj-1234567890abcdef",
                "verified",
            )),
        )
        .unwrap();

        let (saved, mock) = MockTransport::recording();
        let config = Ob2hBridgeConfig {
            enabled: true,
            projects: vec!["omnes-agent".to_string()],
            recap_on_archive: true,
            ..Ob2hBridgeConfig::default()
        };
        let bridge = Ob2hBridge::with_factory(config, Duration::from_secs(1), factory_for(mock));

        let report = bridge
            .recap_after_archive(dir.path(), "feat", "run_1")
            .await
            .unwrap();

        assert_eq!(report, BridgeReport::default());
        assert!(saved.lock().unwrap().is_empty(), "no secret-bearing messages forwarded");
    }

    #[tokio::test]
    async fn recap_after_archive_is_noop_by_default() {
        let bridge = Ob2hBridge::from_config(Ob2hBridgeConfig::default());
        let report = bridge
            .recap_after_archive(Path::new("."), "no-such-slug", "run_1")
            .await
            .unwrap();
        assert_eq!(report, BridgeReport::default());
    }
}
