//! Google Workspace CLI (`gws`) tool config (the `[google_workspace]`
//! section): the built-in default service allowlist, the per-operation
//! allowlist entry, and the section struct itself.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::{DEFAULT_GWS_SERVICES, GoogleWorkspaceAllowedOperation,
//! GoogleWorkspaceConfig}` paths are unchanged.

use super::*;

// ── Google Workspace ─────────────────────────────────────────────

/// Built-in default service allowlist for the `google_workspace` tool.
///
/// Applied when `allowed_services` is empty. Defined here (not in the tool layer)
/// so that config validation can cross-check `allowed_operations` entries against
/// the effective service set in all cases, including when the operator relies on
/// the default.
pub const DEFAULT_GWS_SERVICES: &[&str] = &[
    "drive",
    "sheets",
    "gmail",
    "calendar",
    "docs",
    "slides",
    "tasks",
    "people",
    "chat",
    "classroom",
    "forms",
    "keep",
    "meet",
    "events",
];

/// Google Workspace CLI (`gws`) tool configuration (`[google_workspace]` section).
///
/// ## Defaults
/// - `enabled`: `false` (tool is not registered unless explicitly opted-in).
/// - `allowed_services`: empty vector, which grants access to the full default
///   service set: `drive`, `sheets`, `gmail`, `calendar`, `docs`, `slides`,
///   `tasks`, `people`, `chat`, `classroom`, `forms`, `keep`, `meet`, `events`.
/// - `credentials_path`: `None` (uses default `gws` credential discovery).
/// - `default_account`: `None` (uses the `gws` active account).
/// - `rate_limit_per_minute`: `60`.
/// - `timeout_secs`: `30`.
/// - `audit_log`: `false`.
/// - `credentials_path`: `None` (uses default `gws` credential discovery).
/// - `default_account`: `None` (uses the `gws` active account).
/// - `rate_limit_per_minute`: `60`.
/// - `timeout_secs`: `30`.
/// - `audit_log`: `false`.
///
/// ## Compatibility
/// Configs that omit the `[google_workspace]` section entirely are treated as
/// `GoogleWorkspaceConfig::default()` (disabled, all defaults allowed). Adding
/// the section is purely opt-in and does not affect other config sections.
///
/// ## Rollback / Migration
/// To revert, remove the `[google_workspace]` section from the config file (or
/// set `enabled = false`). No data migration is required; the tool simply stops
/// being registered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct GoogleWorkspaceAllowedOperation {
    /// Google Workspace service ID (for example `gmail` or `drive`).
    pub service: String,
    /// Top-level resource name for the service (for example `users` for Gmail or `files` for Drive).
    pub resource: String,
    /// Optional sub-resource for 4-segment gws commands
    /// (for example `messages` or `drafts` under `gmail users`).
    /// When present, the entry only matches calls that include this exact sub_resource.
    /// When absent, the entry only matches calls with no sub_resource.
    #[serde(default)]
    pub sub_resource: Option<String>,
    /// Allowed methods for the service/resource/sub_resource combination.
    #[serde(default)]
    pub methods: Vec<String>,
}

/// Google Workspace CLI (`gws`) tool configuration (`[google_workspace]` section).
///
/// ## Defaults
/// - `enabled`: `false` (tool is not registered unless explicitly opted-in).
/// - `allowed_services`: empty vector, which grants access to the full default
///   service set: `drive`, `sheets`, `gmail`, `calendar`, `docs`, `slides`,
///   `tasks`, `people`, `chat`, `classroom`, `forms`, `keep`, `meet`, `events`.
/// - `allowed_operations`: empty vector, which preserves the legacy behavior of
///   allowing any resource/method under the allowed service set.
/// - `credentials_path`: `None` (uses default `gws` credential discovery).
/// - `default_account`: `None` (uses the `gws` active account).
/// - `rate_limit_per_minute`: `60`.
/// - `timeout_secs`: `30`.
/// - `audit_log`: `false`.
///
/// ## Compatibility
/// Configs that omit the `[google_workspace]` section entirely are treated as
/// `GoogleWorkspaceConfig::default()` (disabled, all defaults allowed). Adding
/// the section is purely opt-in and does not affect other config sections.
///
/// ## Rollback / Migration
/// To revert, remove the `[google_workspace]` section from the config file (or
/// set `enabled = false`). No data migration is required; the tool simply stops
/// being registered.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "google_workspace"]
#[integration(
    category = "ToolsAutomation",
    display_name = "Google Workspace",
    description = "Drive, Gmail, Calendar, Sheets, Docs via gws CLI",
    status_field = "enabled"
)]
pub struct GoogleWorkspaceConfig {
    /// Enable the `google_workspace` tool. Default: `false`.
    #[serde(default)]
    pub enabled: bool,
    /// Restrict which Google Workspace services the agent can access.
    ///
    /// When empty (the default), the full default service set is allowed (see
    /// struct-level docs). When non-empty, only the listed service IDs are
    /// permitted. Each entry must be non-empty, lowercase alphanumeric with
    /// optional underscores/hyphens, and unique.
    #[serde(default)]
    pub allowed_services: Vec<String>,
    /// Restrict which resource/method combinations the agent can access.
    ///
    /// When empty (the default), all methods under `allowed_services` remain
    /// available for backward compatibility. When non-empty, the runtime denies
    /// any `(service, resource, sub_resource, method)` combination that is not
    /// explicitly listed. `sub_resource` is optional per entry: an entry without
    /// it matches only 3-segment `gws` calls; an entry with it matches only calls
    /// that supply that exact sub_resource value.
    ///
    /// Each entry's `service` must appear in `allowed_services` when that list is
    /// non-empty; config validation rejects entries that would never match at
    /// runtime.
    #[serde(default)]
    pub allowed_operations: Vec<GoogleWorkspaceAllowedOperation>,
    /// Path to service account JSON or OAuth client credentials file.
    ///
    /// When `None`, the tool relies on the default `gws` credential discovery
    /// (`gws auth login`). Set this to point at a service-account key or an
    /// OAuth client-secrets JSON for headless / CI environments.
    #[serde(default)]
    pub credentials_path: Option<String>,
    /// Default Google account email to pass to `gws --account`.
    ///
    /// When `None`, the currently active `gws` account is used.
    #[serde(default)]
    pub default_account: Option<String>,
    /// Maximum number of `gws` API calls allowed per minute. Default: `60`.
    #[serde(default = "default_gws_rate_limit")]
    pub rate_limit_per_minute: u32,
    /// Command execution timeout in seconds. Default: `30`.
    #[serde(default = "default_gws_timeout_secs")]
    pub timeout_secs: u64,
    /// Enable audit logging of every `gws` invocation (service, resource,
    /// method, timestamp). Default: `false`.
    #[serde(default)]
    pub audit_log: bool,
}

fn default_gws_rate_limit() -> u32 {
    60
}

fn default_gws_timeout_secs() -> u64 {
    30
}

impl Default for GoogleWorkspaceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_services: Vec::new(),
            allowed_operations: Vec::new(),
            credentials_path: None,
            default_account: None,
            rate_limit_per_minute: default_gws_rate_limit(),
            timeout_secs: default_gws_timeout_secs(),
            audit_log: false,
        }
    }
}
