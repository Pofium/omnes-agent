//! OpenCode CLI tool config (the `[opencode_cli]` section): `opencode run`
//! delegation with timeout and env passthrough.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::OpenCodeCliConfig` is unchanged.

use super::*;

// ── OpenCode CLI ───────────────────────────────────────────────

/// OpenCode CLI tool configuration (`[opencode_cli]` section).
///
/// Delegates coding tasks to the `opencode run` CLI. Authentication uses the
/// binary's own session by default — no API key needed unless
/// `env_passthrough` includes provider-specific keys.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "opencode_cli"]
pub struct OpenCodeCliConfig {
    /// Enable the `opencode_cli` tool
    #[serde(default)]
    pub enabled: bool,
    /// Maximum execution time in seconds (coding tasks can be long)
    #[serde(default = "default_opencode_cli_timeout_secs")]
    pub timeout_secs: u64,
    /// Maximum output size in bytes (2MB default)
    #[serde(default = "default_opencode_cli_max_output_bytes")]
    pub max_output_bytes: usize,
    /// Extra env vars passed to the opencode subprocess
    #[serde(default)]
    #[credential_class = "legacy_env_path"]
    pub env_passthrough: Vec<String>,
}

fn default_opencode_cli_timeout_secs() -> u64 {
    600
}

fn default_opencode_cli_max_output_bytes() -> usize {
    2_097_152
}

impl Default for OpenCodeCliConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            timeout_secs: default_opencode_cli_timeout_secs(),
            max_output_bytes: default_opencode_cli_max_output_bytes(),
            env_passthrough: Vec::new(),
        }
    }
}
