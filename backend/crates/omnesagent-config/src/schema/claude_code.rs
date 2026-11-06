//! Claude Code CLI tool config (the `[claude_code]` section): `claude -p`
//! delegation with tool allowlist, timeout, and env passthrough.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::ClaudeCodeConfig` is unchanged.

use super::*;

// ── Claude Code ─────────────────────────────────────────────────

/// Claude Code CLI tool configuration (`[claude_code]` section).
///
/// Delegates coding tasks to the `claude -p` CLI. Authentication uses the
/// binary's own OAuth session (Max subscription) by default — no API key
/// needed unless `env_passthrough` includes `ANTHROPIC_API_KEY`.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "claude_code"]
pub struct ClaudeCodeConfig {
    /// Enable the `claude_code` tool
    #[serde(default)]
    pub enabled: bool,
    /// Maximum execution time in seconds (coding tasks can be long)
    #[serde(default = "default_claude_code_timeout_secs")]
    pub timeout_secs: u64,
    /// Claude Code tools the subprocess is allowed to use
    #[serde(default = "default_claude_code_allowed_tools")]
    pub allowed_tools: Vec<String>,
    /// Optional system prompt appended to Claude Code invocations
    #[serde(default)]
    pub system_prompt: Option<String>,
    /// Maximum output size in bytes (2MB default)
    #[serde(default = "default_claude_code_max_output_bytes")]
    pub max_output_bytes: usize,
    /// Extra env vars passed to the claude subprocess (e.g. ANTHROPIC_API_KEY for API-key billing)
    #[serde(default)]
    #[credential_class = "legacy_env_path"]
    pub env_passthrough: Vec<String>,
}

fn default_claude_code_timeout_secs() -> u64 {
    600
}

fn default_claude_code_allowed_tools() -> Vec<String> {
    vec!["Read".into(), "Edit".into(), "Bash".into(), "Write".into()]
}

fn default_claude_code_max_output_bytes() -> usize {
    2_097_152
}

impl Default for ClaudeCodeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            timeout_secs: default_claude_code_timeout_secs(),
            allowed_tools: default_claude_code_allowed_tools(),
            system_prompt: None,
            max_output_bytes: default_claude_code_max_output_bytes(),
            env_passthrough: Vec::new(),
        }
    }
}
