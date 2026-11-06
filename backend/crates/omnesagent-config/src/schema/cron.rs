//! Cron subsystem configuration: the `[cron]` section - declarative job
//! declarations (`CronJobDecl`), schedule variants (`CronScheduleDecl`), the
//! shell output format, and delivery config.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::<CronType>` path is unchanged.

use super::*;

// ── Cron ────────────────────────────────────────────────────────

/// A declarative cron job definition (`[cron.<alias>]`).
///
/// Stored alias-keyed on `Config.cron`. The map key serves as the stable job id.
/// Synced into the database at scheduler startup with `source = "declarative"`,
/// distinguishing them from jobs created imperatively via CLI or API.
/// Declarative config takes precedence on each sync: if the config changes,
/// the DB is updated to match. Imperative jobs are never deleted by sync.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "cron"]
pub struct CronJobDecl {
    /// Human-readable name.
    #[serde(default)]
    pub name: Option<String>,
    /// Job type: `"shell"` (default) or `"agent"`.
    #[serde(default = "default_job_type_decl")]
    pub job_type: String,
    /// Schedule for the job.
    #[serde(default)]
    pub schedule: CronScheduleDecl,
    /// Shell command to run (required when `job_type = "shell"`).
    #[serde(default)]
    pub command: Option<String>,
    /// Agent prompt (required when `job_type = "agent"`).
    #[serde(default)]
    pub prompt: Option<String>,
    /// Whether the job is enabled. Default: `true`.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Model override for agent jobs.
    #[serde(default)]
    pub model: Option<String>,
    /// Optional allowlist of tool names for agent jobs. When omitted, scheduler
    /// defaults may still exclude scheduler mutation tools for cron agent jobs.
    #[serde(default)]
    pub allowed_tools: Option<Vec<String>>,
    /// Whether to recall and inject memory context before this agent job runs.
    /// Defaults to `true`; set to `false` for stateless digest jobs.
    #[serde(default = "default_true")]
    pub uses_memory: bool,
    /// Session target: `"isolated"` (default) or `"main"`.
    #[serde(default)]
    pub session_target: Option<String>,
    /// Delivery configuration.
    #[serde(default)]
    #[nested]
    pub delivery: Option<DeliveryConfigDecl>,
    /// Output format for shell jobs: `"wrapped"` (default) or `"raw"`.
    ///
    /// - `"wrapped"` (default): returns stdout and stderr wrapped in a
    ///   `status=... / stdout: / stderr:` envelope.
    /// - `"raw"`: returns trimmed stdout when the process exits zero. A
    ///   non-zero process exit still gets the wrapped status/stdout/stderr
    ///   envelope for diagnosis. Failures that never reach a process exit —
    ///   a security-policy denial, a shell setup/spawn failure, or a
    ///   timeout — return a plain error string in either format.
    #[serde(default)]
    pub shell_output_format: CronShellOutputFormat,
}

impl Default for CronJobDecl {
    fn default() -> Self {
        Self {
            name: None,
            job_type: default_job_type_decl(),
            schedule: CronScheduleDecl::default(),
            command: None,
            prompt: None,
            enabled: true,
            model: None,
            allowed_tools: None,
            uses_memory: true,
            session_target: None,
            delivery: None,
            shell_output_format: CronShellOutputFormat::default(),
        }
    }
}

/// Output format for shell cron job stdout.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, omnesagent_macros::ConfigEnum)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CronShellOutputFormat {
    /// Wrapped format: `status=...\nstdout:\n...\nstderr:\n...` (default).
    #[default]
    Wrapped,
    /// Raw stdout on a zero process exit; wrapped status/stdout/stderr on a
    /// non-zero exit; a plain error string for failures that never reach a
    /// process exit (security denial, spawn/setup failure, timeout).
    Raw,
}

/// Schedule variant for declarative cron jobs.
#[derive(Debug, Clone, Serialize, Deserialize, omnesagent_macros::ConfigEnum)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CronScheduleDecl {
    /// Classic cron expression.
    Cron {
        expr: String,
        #[serde(default)]
        tz: Option<String>,
    },
    /// Interval in milliseconds.
    Every { every_ms: u64 },
    /// One-shot at an RFC 3339 timestamp.
    At { at: String },
}

impl Default for CronScheduleDecl {
    fn default() -> Self {
        // Empty cron expression — `validate_decl` rejects it. Used only as
        // a placeholder when a fresh map entry is auto-created via the
        // schema's `create_map_key` path; the user fills it in immediately.
        Self::Cron {
            expr: String::new(),
            tz: None,
        }
    }
}

/// Delivery configuration for declarative cron jobs.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "cron_delivery"]
pub struct DeliveryConfigDecl {
    /// Delivery mode: `"none"` or `"announce"`.
    #[serde(default = "default_delivery_mode")]
    pub mode: String,
    /// Channel to deliver to, as `<type>.<alias>` (e.g.
    /// `"telegram.work"`, `"discord.ops"`). A bare type (`"telegram"`)
    /// resolves only while that type has exactly one configured instance,
    /// so prefer the aliased form.
    #[serde(default)]
    pub channel: Option<String>,
    /// Target/recipient identifier.
    #[serde(default)]
    pub to: Option<String>,
    /// Optional thread/conversation identifier carried into the outbound send.
    /// Required by channels that route on a separate `thread_id` field (e.g.
    /// webhook callbacks bridging into agent-chat platforms).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    /// Best-effort delivery. Default: `true`.
    #[serde(default = "default_true")]
    pub best_effort: bool,
}

impl Default for DeliveryConfigDecl {
    fn default() -> Self {
        Self {
            mode: default_delivery_mode(),
            channel: None,
            to: None,
            thread_id: None,
            best_effort: true,
        }
    }
}

fn default_job_type_decl() -> String {
    "shell".to_string()
}

fn default_delivery_mode() -> String {
    "none".to_string()
}
