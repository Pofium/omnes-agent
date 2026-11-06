//! HMAC tool execution receipt config (`[agents.<alias>.tool_receipts]`).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::ToolReceiptsConfig` is unchanged.

use super::*;

// ── Tool receipts ────────────────────────────────────────────────

/// HMAC tool execution receipt configuration, per agent
/// (`[agents.<alias>.tool_receipts]`).
///
/// Receipts are short HMAC-SHA256 tags appended to tool results so the model
/// cannot claim it ran a tool that never actually executed. See
/// `docs/book/src/security/tool-receipts.md`.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "delegate_agent.tool_receipts"]
pub struct ToolReceiptsConfig {
    /// Generate HMAC receipts on every tool execution. Default: `false`.
    /// When false, the entire receipt subsystem is inert (no key, no
    /// generation, no append, no system-prompt addendum).
    #[serde(default)]
    pub enabled: bool,
    /// Append a trailing `Tool receipts:` block to user-visible replies so
    /// receipts are auditable from the channel surface, not just the
    /// internal history. Default: `false`.
    #[serde(default)]
    pub show_in_response: bool,
    /// Inject the receipt-echo instruction into the system prompt so the
    /// model carries receipts verbatim into its response. Default: `true`.
    /// No effect when `enabled = false`.
    #[serde(default = "default_inject_system_prompt")]
    pub inject_system_prompt: bool,
}

fn default_inject_system_prompt() -> bool {
    true
}

impl Default for ToolReceiptsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            show_in_response: false,
            inject_system_prompt: default_inject_system_prompt(),
        }
    }
}
