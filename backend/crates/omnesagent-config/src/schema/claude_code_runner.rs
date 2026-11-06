//! Claude Code task runner config (the `[claude_code_runner]` section): tmux
//! session orchestration with gateway progress hooks.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::ClaudeCodeRunnerConfig` is unchanged.

use super::*;

// ── Claude Code Runner ──────────────────────────────────────────

/// Claude Code task runner configuration (`[claude_code_runner]` section).
///
/// Spawns Claude Code in a tmux session with HTTP hooks that POST tool
/// execution events back to OmnesAgent's gateway, updating a Slack message
/// in-place with progress plus an SSH handoff link.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "claude_code_runner"]
pub struct ClaudeCodeRunnerConfig {
    /// Enable the `claude_code_runner` tool
    #[serde(default)]
    pub enabled: bool,
    /// SSH host for session handoff links (e.g. "myhost.example.com")
    #[serde(default)]
    pub ssh_host: Option<String>,
    /// Prefix for tmux session names (default: "oc-claude-")
    #[serde(default = "default_claude_code_runner_tmux_prefix")]
    pub tmux_prefix: String,
    /// Session time-to-live in seconds before auto-cleanup (default: 3600)
    #[serde(default = "default_claude_code_runner_session_ttl")]
    pub session_ttl: u64,
}

fn default_claude_code_runner_tmux_prefix() -> String {
    "oc-claude-".into()
}

fn default_claude_code_runner_session_ttl() -> u64 {
    3600
}

impl Default for ClaudeCodeRunnerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ssh_host: None,
            tmux_prefix: default_claude_code_runner_tmux_prefix(),
            session_ttl: default_claude_code_runner_session_ttl(),
        }
    }
}
