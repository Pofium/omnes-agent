//! ob2h knowledge bridge (Ф2).
//!
//! Forwards archived Ralph findings into the external ob2h knowledge base over the
//! ob2h MCP stdio contract (`ob2h serve`). Publication is deliberately *fail-open*:
//! any failure to reach ob2h is returned to the caller, which logs it and lets the
//! Ralph cycle finish (Ф2.2, ADR-O3). The bridge is gated by the `[ob2h_bridge]`
//! config (`enabled`, `push_scope`, `projects`, `auto_push_after_archive`) and by a
//! credential guard that never forwards secret-classified content (Ф2.5, ADR-O5).

use crate::journal::{FindingRecord, Journal, JournalError};
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
/// `#[credential_class]` attribute name and the PEM private-key envelope.
const CREDENTIAL_MARKERS: &[&str] = &[
    "credential_class",
    "encrypted_secret",
    "external_auth_store",
    "-----BEGIN ",
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::sync::Mutex;

    fn finding(id: &str, verdict: &str, content: &str) -> FindingRecord {
        FindingRecord {
            finding_id: id.to_string(),
            task: "T-1".to_string(),
            content: content.to_string(),
            verdict: verdict.to_string(),
            symbols: Vec::new(),
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

        async fn call_tool(&mut self, _tool: &str, arguments: Value) -> Result<String, BridgeError> {
            if self.hang {
                return std::future::pending().await;
            }
            self.saved.lock().unwrap().push(arguments);
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
}
