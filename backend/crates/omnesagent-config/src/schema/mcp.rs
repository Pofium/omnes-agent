//! External MCP client configuration (the `[mcp]` section): the
//! `McpTransport` enum, per-server `McpServerConfig`, and `McpConfig`.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::{McpConfig, McpServerConfig, McpTransport}` path is
//! unchanged.

use super::*;

// ── MCP ────────────────────────────────────────────────────────

/// Transport type for MCP server connections.
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum McpTransport {
    /// Spawn a local process and communicate over stdin/stdout.
    #[default]
    Stdio,
    /// Connect via HTTP POST.
    Http,
    /// Connect via HTTP + Server-Sent Events.
    Sse,
}

impl McpTransport {
    /// Wire token for this transport, matching the `rename_all = "lowercase"`
    /// serde representation (`stdio`/`http`/`sse`). The exhaustive match makes
    /// a new variant a compile error here rather than a silent miss.
    pub fn wire_name(self) -> &'static str {
        match self {
            McpTransport::Stdio => "stdio",
            McpTransport::Http => "http",
            McpTransport::Sse => "sse",
        }
    }

    /// The `McpServerConfig` leaf this transport requires to be non-empty.
    /// Single source of truth shared by `validate_mcp_config` (runtime
    /// enforcement) and the config-form `x-required-by-transport` schema
    /// metadata the Operator Console reads to mark the right field Required
    /// per transport. Changing the relationship here updates both consumers.
    pub fn required_leaf(self) -> &'static str {
        match self {
            McpTransport::Stdio => "command",
            McpTransport::Http | McpTransport::Sse => "url",
        }
    }
}

/// Every transport, enumerated on-demand from the schema rather than a
/// hand-maintained list, so the set is derived from the enum itself and a new
/// variant shows up here automatically (its `required_leaf` arm then forces a
/// decision). schemars emits a fieldless enum as a top-level `enum` array, or
/// as `oneOf` of `const`s once the variants carry doc comments; handle both,
/// matching the existing `enum_variants` helper. The wire tokens honor the
/// serde rename, so they round-trip straight back to the enum.
#[cfg(feature = "schema-export")]
pub(super) fn mcp_transports() -> Vec<McpTransport> {
    let schema = serde_json::to_value(schemars::schema_for!(McpTransport)).unwrap_or_default();
    let wire_values: Vec<serde_json::Value> =
        if let Some(values) = schema.get("enum").and_then(serde_json::Value::as_array) {
            values.clone()
        } else if let Some(variants) = schema.get("oneOf").and_then(serde_json::Value::as_array) {
            variants
                .iter()
                .filter_map(|variant| variant.get("const").cloned())
                .collect()
        } else {
            Vec::new()
        };
    wire_values
        .into_iter()
        .filter_map(|value| serde_json::from_value::<McpTransport>(value).ok())
        .collect()
}

/// Per-transport required-leaf map, projected from [`McpTransport::required_leaf`]
/// onto the schema as the `x-required-by-transport` extension on
/// `McpServerConfig`. The Operator Console config form reads it so the Required
/// badge tracks the selected transport (`stdio` needs `command`, `http`/`sse`
/// need `url`) instead of hardcoding the relationship in the web surface.
#[cfg(feature = "schema-export")]
fn mcp_required_by_transport() -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = mcp_transports()
        .into_iter()
        .map(|transport| {
            (
                transport.wire_name().to_string(),
                serde_json::Value::String(transport.required_leaf().to_string()),
            )
        })
        .collect();
    serde_json::Value::Object(map)
}

/// Configuration for a single external MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "schema-export",
    schemars(extend("x-required-by-transport" = mcp_required_by_transport()))
)]
#[prefix = "mcp.servers"]
pub struct McpServerConfig {
    /// Display name used as a tool prefix (`<server>__<tool>`). Filled in
    /// from the supplied `map_key` when the entry is created via
    /// `create_map_key("mcp.servers", "<name>")`; `#[serde(default)]` lets
    /// the macro default-construct from `{}` before the name gets injected.
    #[serde(default)]
    pub name: String,
    /// Transport type (default: stdio).
    #[serde(default)]
    pub transport: McpTransport,
    /// URL for HTTP/SSE transports.
    #[serde(default)]
    pub url: Option<String>,
    /// Executable to spawn for stdio transport.
    #[serde(default)]
    pub command: String,
    /// Command arguments for stdio transport.
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional environment variables for stdio transport.
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub env: HashMap<String, String>,
    /// Optional HTTP headers for HTTP/SSE transports. Treated as secret:
    /// the values commonly carry Bearer tokens for the upstream MCP server.
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub headers: HashMap<String, String>,
    /// Optional per-call timeout in seconds (hard capped in validation).
    #[serde(default)]
    pub tool_timeout_secs: Option<u64>,
    /// Maximum bytes accepted for a single HTTP/SSE JSON-RPC response body,
    /// enforced at read time before the body is parsed or any embedded resource
    /// is materialized. A compromised or misbehaving server cannot force an
    /// unbounded response-body allocation. This is the *encoded wire* cap, not
    /// the decoded materialization budget: `None`/`0` uses the built-in default
    /// of 16,078,168 bytes, sized so a full 10 MiB decoded embedded-resource blob
    /// (its base64 expansion plus JSON-RPC envelope headroom) still fits. The
    /// separate 10 MiB figure is the decoded aggregate blob budget applied after
    /// parsing, not this wire cap.
    #[serde(default)]
    pub max_response_bytes: Option<u64>,
    /// Resource URIs to read once at agent startup and inject into the system
    /// prompt as untrusted, server-origin context. Each is read via
    /// `resources/read` on this server; pins on a server that does not advertise
    /// resources, or that the agent's tool policy denies, are skipped with a
    /// warning. Read once per run (not refreshed; no subscriptions).
    #[serde(default)]
    pub pinned_resources: Vec<String>,
    /// Absolute path to a PEM-encoded CA certificate or bundle to trust in
    /// addition to the default roots for this server's HTTP/SSE transport.
    ///
    /// Certificate and hostname verification remain enabled. The path must
    /// name a regular file no larger than 1 MiB; a missing, unreadable, empty,
    /// oversized, non-regular, or invalid file is a hard connection error
    /// rather than a fallback to the default trust store. When set, the
    /// configured URL and any advertised SSE message endpoint must use HTTPS;
    /// plaintext URLs and downgrade redirects are rejected. Ignored by stdio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_ca_cert_path: Option<String>,
}

/// External MCP client configuration (`[mcp]` section).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "mcp"]
pub struct McpConfig {
    /// Enable MCP tool loading.
    #[tab(Settings)]
    #[serde(default = "default_mcp_enabled")]
    pub enabled: bool,
    /// Load MCP tool schemas on-demand via `tool_search` instead of eagerly
    /// including them in the LLM context window. When enabled, only tool names
    /// are listed in the system prompt; the LLM must call `tool_search` to fetch
    /// full schemas before invoking a deferred tool.
    #[tab(Settings)]
    #[serde(default = "default_deferred_loading")]
    pub deferred_loading: bool,
    /// Configured MCP servers. The `#[nested]` annotation makes the macro
    /// expose this as a List section in `map_key_sections()`, so the
    /// dashboard's `+ Add MCP server` affordance and the `POST
    /// /api/config/map-key?path=mcp.servers&key=<name>` endpoint pick it
    /// up automatically (no hand-table on the gateway side).
    ///
    /// `#[natural_key = "name"]` opts the Vec into per-element property
    /// routing (see `route_vec_path` and the `Configurable` derive's
    /// `#[natural_key]` arm). With it, `set_prop("mcp.servers.<name>.url",
    /// ...)` and `get_prop("mcp.servers.<name>.transport")` resolve to
    /// the matching element's own `set_prop` / `get_prop`, and the
    /// `mcp.servers` section behaves like a `HashMap<String,
    /// McpServerConfig>` from the dashboard / TUI's point of view.
    /// `name` itself becomes read-only via `set_prop`; rename goes
    /// through `rename_map_key` which mutates the `name` field in place.
    #[tab(Servers)]
    #[serde(default, alias = "mcpServers")]
    #[nested]
    #[natural_key = "name"]
    pub servers: Vec<McpServerConfig>,
}

fn default_mcp_enabled() -> bool {
    true
}

fn default_deferred_loading() -> bool {
    false
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: default_mcp_enabled(),
            deferred_loading: default_deferred_loading(),
            servers: Vec::new(),
        }
    }
}
