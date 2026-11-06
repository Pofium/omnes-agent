//! ACP (Agent Client Protocol) server config: the `[acp]` section.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::AcpConfig` path is unchanged.

use super::*;

// ── ACP ──────────────────────────────────────────────────────────

/// ACP (Agent Client Protocol) server configuration (`[acp]` section).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "acp"]
pub struct AcpConfig {
    /// Agent alias to use when `session/new` omits `agentAlias` and more than
    /// one agent is configured. When exactly one agent exists it is
    /// auto-selected regardless of this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_agent: Option<String>,
    /// Maximum number of concurrent ACP sessions. Default: `10`.
    #[serde(default = "default_acp_max_sessions")]
    pub max_sessions: usize,
    /// Idle session timeout in seconds. Sessions with no activity for this
    /// duration are eligible for eviction. Default: `3600` (1 hour).
    #[serde(default = "default_acp_session_timeout_secs")]
    pub session_timeout_secs: u64,
}

fn default_acp_max_sessions() -> usize {
    10
}

fn default_acp_session_timeout_secs() -> u64 {
    3600
}

impl Default for AcpConfig {
    fn default() -> Self {
        Self {
            default_agent: None,
            max_sessions: default_acp_max_sessions(),
            session_timeout_secs: default_acp_session_timeout_secs(),
        }
    }
}
