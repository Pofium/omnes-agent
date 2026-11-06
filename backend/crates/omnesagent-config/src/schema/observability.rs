//! Observability configuration: the `[observability]` section - sink backend,
//! log persistence, tool-I/O / LLM payload capture, and OTel content knobs.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::<ObservabilityType>` path is unchanged.

use super::deserialize_enum_lenient;
use omnesagent_macros::Configurable;
use serde::{Deserialize, Serialize};

// ── Observability ─────────────────────────────────────────────────

/// Observability sink backend.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ObservabilityBackend {
    #[default]
    #[serde(alias = "noop")]
    None,
    Log,
    Verbose,
    Prometheus,
    #[serde(alias = "opentelemetry", alias = "otlp")]
    Otel,
}

impl ObservabilityBackend {
    #[must_use]
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Log => "log",
            Self::Verbose => "verbose",
            Self::Prometheus => "prometheus",
            Self::Otel => "otel",
        }
    }
}

/// JSONL log persistence mode.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum LogPersistence {
    None,
    #[default]
    Rolling,
    Full,
    /// Persist all events, but with OmnesAgent-managed archive rotation: the
    /// active file is rotated to a timestamped archive on a size and/or daily
    /// boundary, and old archives are pruned by count and/or age. Unlike
    /// `rolling` (entry-count trim of the active file), rotated events are
    /// preserved in archive files for later diagnostics. Rotation is tuned via
    /// the `log_persistence_max_bytes`, `log_persistence_rotate_daily`,
    /// `log_persistence_retention_max_files`, and
    /// `log_persistence_retention_max_age_days` settings.
    Rotating,
}

impl LogPersistence {
    #[must_use]
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Rolling => "rolling",
            Self::Full => "full",
            Self::Rotating => "rotating",
        }
    }
}

/// Tool I/O capture policy.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum LogToolIo {
    Off,
    #[default]
    Redacted,
    Full,
}

impl LogToolIo {
    #[must_use]
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Redacted => "redacted",
            Self::Full => "full",
        }
    }
}

/// LLM request payload capture policy. Mirrors [`LogToolIo`] but gates the
/// prompt/messages sent on each `llm_request`. Defaults to `Off` because the
/// full payload is the entire system prompt plus conversation every turn.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum LogLlmRequestPayload {
    #[default]
    Off,
    Redacted,
    Full,
}

impl LogLlmRequestPayload {
    #[must_use]
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Redacted => "redacted",
            Self::Full => "full",
        }
    }
}

/// OTel content capture policy. Mirrors [`LogToolIo`] but gates OTel span
/// attribute emission, not log persistence. Defaults to `Off` for privacy.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OtelContentPolicy {
    #[default]
    Off,
    Redacted,
    Full,
}

impl OtelContentPolicy {
    #[must_use]
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Redacted => "redacted",
            Self::Full => "full",
        }
    }
}

/// Observability backend configuration (`[observability]` section).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "observability"]
pub struct ObservabilityConfig {
    /// Observability sink: none | log | verbose | prometheus | otel.
    #[serde(default, deserialize_with = "deserialize_enum_lenient")]
    pub backend: ObservabilityBackend,

    /// OTLP endpoint (e.g. `"http://localhost:4318"`). Only used when backend = `"otel"`.
    #[serde(default)]
    pub otel_endpoint: Option<String>,

    /// Service name reported to the OTel collector. Defaults to "omnesagent".
    #[serde(default)]
    pub otel_service_name: Option<String>,

    /// Optional HTTP headers sent with every OTLP export request (e.g. authorization).
    /// Specified as key-value pairs in TOML:
    /// ```toml
    /// [observability.otel_headers]
    /// Authorization = "Bearer sk-..."
    /// ```
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub otel_headers: Option<std::collections::HashMap<String, String>>,

    /// Log persistence mode: "none" | "rolling" | "full".
    /// Controls whether every event passing through `omnesagent_log::record!`
    /// is appended to the on-disk JSONL log.
    #[serde(
        default = "default_log_persistence",
        alias = "runtime_trace_mode",
        deserialize_with = "deserialize_enum_lenient"
    )]
    pub log_persistence: LogPersistence,

    /// Log persistence file path. Relative paths resolve under workspace_dir.
    #[serde(default = "default_log_persistence_path", alias = "runtime_trace_path")]
    pub log_persistence_path: String,

    /// Maximum entries retained when `log_persistence = "rolling"`.
    #[serde(
        default = "default_log_persistence_max_entries",
        alias = "runtime_trace_max_entries"
    )]
    pub log_persistence_max_entries: usize,

    /// Size threshold in bytes that triggers an archive rotation when
    /// `log_persistence = "rotating"`. The active file is rotated once an
    /// append leaves it at or above this size. `0` disables size-based
    /// rotation. Ignored unless `log_persistence = "rotating"`.
    #[serde(default = "default_log_persistence_max_bytes")]
    pub log_persistence_max_bytes: u64,

    /// Rotate the active file to an archive on a UTC day boundary when
    /// `log_persistence = "rotating"`: before the first event of a new day is
    /// appended, a file whose last write fell on an earlier day is archived.
    /// Ignored unless `log_persistence = "rotating"`.
    #[serde(default = "default_log_persistence_rotate_daily")]
    pub log_persistence_rotate_daily: bool,

    /// Retention cap on the number of rotated archive files kept alongside the
    /// active file when `log_persistence = "rotating"`. After a rotation the
    /// oldest archives beyond this count are deleted. `0` keeps all archives.
    /// Ignored unless `log_persistence = "rotating"`.
    #[serde(default = "default_log_persistence_retention_max_files")]
    pub log_persistence_retention_max_files: usize,

    /// Retention cap on the age (in days) of rotated archive files when
    /// `log_persistence = "rotating"`. After a rotation, archives older than
    /// this many days are deleted. `0` disables age-based cleanup. Ignored
    /// unless `log_persistence = "rotating"`.
    #[serde(default = "default_log_persistence_retention_max_age_days")]
    pub log_persistence_retention_max_age_days: u64,

    /// Tool I/O capture policy: "off" | "redacted" | "full".
    /// - `off`: only tool name + outcome + duration land in the log.
    /// - `redacted` (default): tool input + output are leak-scanned and
    ///   truncated at `log_tool_io_truncate_bytes` before persisting.
    /// - `full`: full input + output, still leak-scanned. For operators
    ///   who need replay fidelity and accept the disk cost.
    #[serde(
        default = "default_log_tool_io",
        deserialize_with = "deserialize_enum_lenient"
    )]
    pub log_tool_io: LogToolIo,

    /// Truncate the captured tool input and output at this many bytes when
    /// `log_tool_io = "redacted"`. Truncated events carry an explicit
    /// `tool_output_truncated: true` flag plus `tool_output_original_bytes`.
    #[serde(default = "default_log_tool_io_truncate_bytes")]
    pub log_tool_io_truncate_bytes: usize,

    /// Tool names whose I/O is never logged beyond name + outcome + duration
    /// regardless of `log_tool_io`. Use for tools whose I/O is intrinsically
    /// sensitive (e.g. memory recall against personal namespaces, agent
    /// secret reads). Empty by default.
    #[serde(default)]
    pub log_tool_io_denylist: Vec<String>,

    /// LLM request payload capture: "off" | "redacted" | "full".
    /// - `off` (default): only `messages_count` lands on the `llm_request`
    ///   event. No prompt or conversation content is persisted.
    /// - `redacted`: the full message history (role + content) is leak-scanned
    ///   and truncated at `log_tool_io_truncate_bytes` before persisting.
    /// - `full`: full message history, still leak-scanned, untruncated. For
    ///   operators who need replay fidelity and accept the disk cost.
    ///
    /// Opt-in (default off) because `full`/`redacted` capture the entire system
    /// prompt and conversation on every turn.
    #[serde(
        default = "default_log_llm_request_payload",
        deserialize_with = "deserialize_enum_lenient"
    )]
    pub log_llm_request_payload: LogLlmRequestPayload,

    /// OTel GenAI content capture: "off" | "redacted" | "full".
    /// Controls whether `gen_ai.system_instructions`, `gen_ai.input.messages`,
    /// and `gen_ai.output.messages` are emitted on OTel spans.
    /// - `off` (default): no content attributes, only metadata.
    /// - `redacted`: content is leak-scanned and truncated at `otel_genai_content_max_chars`.
    /// - `full`: content is leak-scanned but not truncated.
    #[serde(default, deserialize_with = "deserialize_enum_lenient")]
    pub otel_genai_content: OtelContentPolicy,

    /// Per-field character truncation limit for OTel GenAI content when
    /// `otel_genai_content = "redacted"`. Each string field is truncated
    /// independently. `0` is treated as `off`.
    #[serde(default = "default_otel_genai_content_max_chars")]
    pub otel_genai_content_max_chars: usize,

    /// OTel tool I/O capture: "off" | "redacted" | "full".
    /// Controls whether `gen_ai.tool.arguments`, `input.value`,
    /// `gen_ai.tool.result`, and `output.value` are emitted on OTel spans.
    /// - `off` (default): no content attributes, only tool name + outcome.
    /// - `redacted`: content is leak-scanned and truncated at `otel_tool_io_max_chars`.
    /// - `full`: content is leak-scanned but not truncated.
    #[serde(default, deserialize_with = "deserialize_enum_lenient")]
    pub otel_tool_io: OtelContentPolicy,

    /// Per-field character truncation limit for OTel tool I/O when
    /// `otel_tool_io = "redacted"`. Each string field is truncated
    /// independently. `0` is treated as `off`.
    #[serde(default = "default_otel_tool_io_max_chars")]
    pub otel_tool_io_max_chars: usize,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            backend: ObservabilityBackend::None,
            otel_endpoint: None,
            otel_service_name: None,
            otel_headers: None,
            log_persistence: default_log_persistence(),
            log_persistence_path: default_log_persistence_path(),
            log_persistence_max_entries: default_log_persistence_max_entries(),
            log_persistence_max_bytes: default_log_persistence_max_bytes(),
            log_persistence_rotate_daily: default_log_persistence_rotate_daily(),
            log_persistence_retention_max_files: default_log_persistence_retention_max_files(),
            log_persistence_retention_max_age_days: default_log_persistence_retention_max_age_days(
            ),
            log_tool_io: default_log_tool_io(),
            log_tool_io_truncate_bytes: default_log_tool_io_truncate_bytes(),
            log_tool_io_denylist: Vec::new(),
            log_llm_request_payload: default_log_llm_request_payload(),
            otel_genai_content: OtelContentPolicy::Off,
            otel_genai_content_max_chars: default_otel_genai_content_max_chars(),
            otel_tool_io: OtelContentPolicy::Off,
            otel_tool_io_max_chars: default_otel_tool_io_max_chars(),
        }
    }
}

fn default_log_persistence() -> LogPersistence {
    LogPersistence::Rolling
}

fn default_log_persistence_path() -> String {
    "state/runtime-trace.jsonl".to_string()
}

fn default_log_persistence_max_entries() -> usize {
    200
}

/// Size rotation off by default; operators opt in with an explicit byte budget.
fn default_log_persistence_max_bytes() -> u64 {
    0
}

/// Daily rotation is on by default so `rotating` mode produces day-scoped
/// archives out of the box even without a configured size budget.
fn default_log_persistence_rotate_daily() -> bool {
    true
}

/// Keep a week of rotated archives by default.
fn default_log_persistence_retention_max_files() -> usize {
    7
}

/// Age-based cleanup off by default; count-based retention governs unless set.
fn default_log_persistence_retention_max_age_days() -> u64 {
    0
}

fn default_log_tool_io() -> LogToolIo {
    LogToolIo::Redacted
}

fn default_log_llm_request_payload() -> LogLlmRequestPayload {
    LogLlmRequestPayload::Off
}

fn default_log_tool_io_truncate_bytes() -> usize {
    40960
}

fn default_otel_genai_content_max_chars() -> usize {
    1000
}

fn default_otel_tool_io_max_chars() -> usize {
    1000
}
