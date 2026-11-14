//! Heartbeat config: the `[heartbeat]` section - periodic health pings, adaptive
//! intervals, dead-man's switch, and run-history retention.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::HeartbeatConfig` path is unchanged.

use super::*;

// ── Heartbeat ────────────────────────────────────────────────────

/// Heartbeat configuration for periodic health pings (`[heartbeat]` section).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "heartbeat"]
#[allow(clippy::struct_excessive_bools)]
pub struct HeartbeatConfig {
    /// Enable periodic heartbeat pings. Default: `false`. When enabled,
    /// `agent` must name a configured agent — there is no default agent
    /// for heartbeat to fall through to.
    #[serde(default)]
    pub enabled: bool,
    /// Configured agent alias the heartbeat worker runs as. Required
    /// when `enabled = true`; refers to a `[agents.<alias>]` entry.
    #[serde(default)]
    pub agent: String,
    /// Interval in minutes between heartbeat pings. Minimum: `1`. Default: `30`.
    #[serde(default = "default_heartbeat_interval")]
    pub interval_minutes: u32,
    /// Enable two-phase heartbeat: Phase 1 asks LLM whether to run, Phase 2
    /// executes only when the LLM decides there is work to do. Saves API cost
    /// during quiet periods. Default: `true`.
    #[serde(default = "default_two_phase")]
    pub two_phase: bool,
    /// Optional fallback task text when `HEARTBEAT.md` has no task entries.
    #[serde(default)]
    pub message: Option<String>,
    /// Optional delivery channel for heartbeat output (for example: `telegram`).
    /// When omitted, auto-selects the first configured channel.
    #[serde(default, alias = "channel")]
    pub target: Option<String>,
    /// Optional delivery recipient/chat identifier (required when `target` is
    /// explicitly set).
    #[serde(default, alias = "recipient")]
    pub to: Option<String>,
    /// Enable adaptive intervals that back off on failures and speed up for
    /// high-priority tasks. Default: `false`.
    #[serde(default)]
    pub adaptive: bool,
    /// Minimum interval in minutes when adaptive mode is enabled. Default: `5`.
    #[serde(default = "default_heartbeat_min_interval")]
    pub min_interval_minutes: u32,
    /// Maximum interval in minutes when adaptive mode backs off. Default: `120`.
    #[serde(default = "default_heartbeat_max_interval")]
    pub max_interval_minutes: u32,
    /// Dead-man's switch timeout in minutes. If the heartbeat has not ticked
    /// within this window, an alert is sent. `0` disables. Default: `0`.
    #[serde(default)]
    pub deadman_timeout_minutes: u32,
    /// Channel for dead-man's switch alerts (e.g. `telegram`). Falls back to
    /// the heartbeat delivery channel.
    #[serde(default)]
    pub deadman_channel: Option<String>,
    /// Recipient for dead-man's switch alerts. Falls back to `to`.
    #[serde(default)]
    pub deadman_to: Option<String>,
    /// Maximum number of heartbeat run history records to retain. Default: `100`.
    #[serde(default = "default_heartbeat_max_run_history")]
    pub max_run_history: u32,
    /// Load the channel session history before each heartbeat task execution so
    /// the LLM has conversational context. Default: `false`.
    ///
    /// When `true`, the session file for the configured `target`/`to` is passed
    /// to the agent as `session_state_file`, giving it access to the recent
    /// conversation history — just as if the user had sent a message.
    #[serde(default)]
    pub load_session_context: bool,
    /// Maximum wall-clock seconds allowed for a single agent invocation
    /// (Phase 1 decision or Phase 2 task execution). `0` disables.
    /// Default: `600` (10 minutes).
    #[serde(default = "default_heartbeat_task_timeout")]
    pub task_timeout_secs: u64,
}

fn default_heartbeat_interval() -> u32 {
    30
}

fn default_two_phase() -> bool {
    true
}

fn default_heartbeat_min_interval() -> u32 {
    5
}

fn default_heartbeat_max_interval() -> u32 {
    120
}

fn default_heartbeat_max_run_history() -> u32 {
    100
}

fn default_heartbeat_task_timeout() -> u64 {
    600
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            agent: String::new(),
            interval_minutes: default_heartbeat_interval(),
            two_phase: true,
            message: None,
            target: None,
            to: None,
            adaptive: false,
            min_interval_minutes: default_heartbeat_min_interval(),
            max_interval_minutes: default_heartbeat_max_interval(),
            deadman_timeout_minutes: 0,
            deadman_channel: None,
            deadman_to: None,
            max_run_history: default_heartbeat_max_run_history(),
            load_session_context: false,
            task_timeout_secs: default_heartbeat_task_timeout(),
        }
    }
}
