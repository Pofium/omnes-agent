//! MCP tool filtering config: named groups of tool-name glob patterns with an
//! activation mode (`[runtime_profiles.<name>.tool_filter_groups]`).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::{ToolFilterGroup, ToolFilterGroupMode}` are unchanged.

use super::*;

/// Determines when a `ToolFilterGroup` is active.
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ToolFilterGroupMode {
    /// Tools in this group are always included in every turn.
    Always,
    /// Tools in this group are included only when the user message contains
    /// at least one of the configured `keywords` (case-insensitive substring match).
    #[default]
    Dynamic,
}

/// A named group of MCP tool patterns with an activation mode.
///
/// Each group lists glob patterns for MCP tool names and an optional set of
/// keywords that trigger inclusion in `dynamic` mode. MCP tools are named
/// `<server>__<tool>` (double-underscore separator, e.g. `filesystem__read_file`).
///
/// Classification is by origin, not name shape: only tools that actually came
/// from the MCP registry are subject to these groups. Built-in tools AND skill
/// tools — which share the `<x>__<y>` naming convention (`{skill}__{tool}`) —
/// always pass through and are never affected by `tool_filter_groups`.
///
/// Under `[mcp] deferred_loading = true`, `mode = "always"` entries are
/// pre-activated at assembly time so matching tools are live from the first
/// turn without a `tool_search` round-trip.
///
/// # Example
/// ```toml
/// [[runtime_profiles.default.tool_filter_groups]]
/// mode = "always"
/// tools = ["filesystem__*"]
/// keywords = []
///
/// [[runtime_profiles.default.tool_filter_groups]]
/// mode = "dynamic"
/// tools = ["browser__*"]
/// keywords = ["browse", "website", "url", "search"]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct ToolFilterGroup {
    /// Activation mode: `"always"` or `"dynamic"`.
    #[serde(default)]
    pub mode: ToolFilterGroupMode,
    /// Glob patterns matching MCP tool names (single `*` wildcard supported).
    #[serde(default)]
    pub tools: Vec<String>,
    /// Keywords that activate this group in `dynamic` mode (case-insensitive substring).
    /// Ignored when `mode = "always"`.
    #[serde(default)]
    pub keywords: Vec<String>,
}
