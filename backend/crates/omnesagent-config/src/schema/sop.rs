//! SOP engine configuration (the `[sop]` section): `SopConfig`, the
//! run-store/approval enums, and the approval policy sub-configs.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::{SopConfig, ...}` path is unchanged.

use super::*;

/// Standard Operating Procedures engine configuration (`[sop]`).
///
/// The `default_execution_mode` field uses the `SopExecutionMode` type from
/// `sop::types` (re-exported via `sop::SopExecutionMode`). To avoid circular
/// module references, config stores it using the same enum definition.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "sop"]
pub struct SopConfig {
    /// Directory containing SOP definitions (subdirs with SOP.toml + SOP.md).
    /// A relative value resolves against the install root (matching the
    /// `skill-bundles` convention), so the documented `shared/sops` loads from
    /// `<install>/shared/sops` — the same directory the web/RPC SOP author writes
    /// to. An absolute or `~`-prefixed value is used as-is. Unset by default;
    /// SOP runtime behavior activates only when this is set to a non-empty
    /// value, so leaving it unset (or an explicit empty value) keeps SOP
    /// loading disabled (unset/empty still falls back to `<install>/shared/sops`
    /// for offline CLI inspection).
    #[serde(default = "default_sop_sops_dir")]
    pub sops_dir: Option<String>,

    /// Default execution mode for SOPs that omit `execution_mode`.
    /// Values: `auto`, `supervised` (default), `step_by_step`,
    /// `priority_based`, `deterministic`.
    #[serde(default = "default_sop_execution_mode")]
    pub default_execution_mode: String,

    /// Maximum total concurrent SOP runs across all SOPs.
    #[serde(default = "default_sop_max_concurrent_total")]
    pub max_concurrent_total: usize,

    /// Approval timeout in seconds. When a run waits for approval longer than
    /// this, the configured `approval_timeout_action` is applied (default
    /// `escalate`: re-surface the gate to the out-of-band approver and never
    /// self-approve). Set to 0 to disable the timeout sweep.
    #[serde(default = "default_sop_approval_timeout_secs")]
    pub approval_timeout_secs: u64,

    /// Maximum number of finished runs kept in memory for status queries.
    /// Oldest runs are evicted when over capacity. 0 = unlimited.
    #[serde(default = "default_sop_max_finished_runs")]
    pub max_finished_runs: usize,

    /// How often (seconds) the daemon runs the SOP maintenance tick: fire
    /// fail-closed approval timeouts (per `approval_timeout_secs` /
    /// `approval_timeout_action`), reap expired concurrency-claim leases, and
    /// prune terminal runs past `max_finished_runs`. Default `60`; set to `0` to
    /// disable the tick entirely. The tick itself self-approves nothing - timeout
    /// handling is governed by `approval_timeout_action` (default `escalate`).
    #[serde(default = "default_sop_maintenance_interval_secs")]
    pub maintenance_interval_secs: u64,

    /// Persist run state durably across restarts. Default `true`: `build_sop_engine`
    /// selects the configured backend (`sqlite`) and in-flight runs - including runs
    /// parked at a HITL approval - survive a restart. This is the durable substrate
    /// the HITL admission model relies on so a pending approval is not lost when the
    /// daemon restarts. Set to `false` to opt back into ephemeral in-memory state.
    #[serde(default = "default_sop_persist_runs")]
    pub persist_runs: bool,

    /// Durable run-state backend when `persist_runs` is true: `sqlite` (default,
    /// durable) or `memory` (explicitly non-durable, for tests/degraded).
    #[serde(default)]
    pub run_store_backend: SopRunStoreBackend,

    /// Directory for the durable run store (created mode-0700). When omitted,
    /// `<data_dir>/sop`. Never OS-temp.
    #[serde(default)]
    pub run_state_dir: Option<String>,

    /// WHO may clear a SOP approval gate. Layered with execution_mode / priority /
    /// requires_confirmation (those still apply). Default `both` keeps today's
    /// behavior (the agent tool OR an out-of-band principal can clear a gate).
    #[serde(default)]
    pub approval_mode: ApprovalMode,

    /// What happens to a SOP gate that times out (after `approval_timeout_secs`).
    /// Default `escalate` is fail-closed: re-surface to the out-of-band approver and
    /// keep waiting, never self-approve. (The SOP-gate analog of the channel
    /// approval-routing fail-closed default; reconcile with that model if both land.)
    #[serde(default)]
    pub approval_timeout_action: ApprovalTimeoutAction,

    /// Approval broker policy config (`[sop.approval]`): named approver groups and
    /// per-name approval policies (required group + quorum + escalation route) the
    /// approval broker consumes for group-membership and quorum checks. Members are
    /// channel-provided identities (a gateway user, a forge login), so this is a
    /// permanent identity source, not a stopgap; a future auth system adds another
    /// resolver alongside it rather than replacing it. An empty block means no broker
    /// policy applies (`approval_mode` alone governs a gate, unchanged behavior).
    #[serde(default)]
    #[nested]
    pub approval: SopApprovalConfig,

    /// Enforce per-step tool scope. Default false keeps `tools:` advisory.
    #[serde(default)]
    pub step_scope_enforce: bool,

    /// Tool names that remain available while step scope is enforced.
    #[serde(default = "default_sop_step_mandatory_tools")]
    pub step_mandatory_tools: Vec<String>,

    /// Enforce per-step input/output schemas when a step declares them.
    #[serde(default = "default_sop_step_schema_enforce")]
    pub step_schema_enforce: bool,

    /// Maximum times a routed SOP run can visit one step.
    #[serde(default = "default_sop_max_step_visits")]
    pub max_step_visits: u32,

    /// Maximum retries allowed by a step failure policy.
    #[serde(default = "default_sop_max_step_retries")]
    pub max_step_retries: u32,

    /// Maximum bytes accepted from untrusted SOP trigger payload/topic content
    /// before char-boundary truncation. 0 disables the cap.
    #[serde(default = "default_sop_untrusted_payload_max_bytes")]
    pub untrusted_payload_max_bytes: usize,

    /// Prompt-guard action for untrusted SOP trigger input: warn, block, or sanitize.
    #[serde(default = "default_sop_untrusted_input_guard")]
    pub untrusted_input_guard: String,

    /// Prompt-guard and outbound-redaction sensitivity for untrusted SOP content.
    #[serde(default = "default_sop_untrusted_guard_sensitivity")]
    pub untrusted_guard_sensitivity: f64,

    /// Include the explanatory warning text inside untrusted-content frames.
    /// Boundary framing itself is always on once wired.
    #[serde(default = "default_sop_untrusted_frame_warning")]
    pub untrusted_frame_warning: bool,

    /// Redact outbound SOP content before persistence/audit consumers write it.
    /// Intended to converge with the shared B/F redaction switch once those consumers land.
    #[serde(default = "default_sop_untrusted_outbound_redact")]
    pub untrusted_outbound_redact: bool,

    /// Enable SOP procedural-memory proposal tooling. Default false keeps
    /// self-modifying SOP write-back opt-in while the SOP subsystem is
    /// Experimental.
    #[serde(default)]
    pub procedural_memory_enabled: bool,
}

impl SopConfig {
    /// Whether the SOP runtime (engine, tools, maintenance tick, run store) is
    /// active for this config. SOP loading is gated on a concrete definitions
    /// directory: a non-empty `sops_dir` enables it; an unset or explicit empty
    /// string keeps it disabled (the default is unset, so SOP runtime is off
    /// until an operator opts in). This is the single source of truth for
    /// activation, so callers must not re-derive it from `sops_dir.is_some()` —
    /// that treats an empty string as enabled and would break the disable path.
    #[must_use]
    pub fn runtime_enabled(&self) -> bool {
        self.sops_dir
            .as_deref()
            .is_some_and(|dir| !dir.trim().is_empty())
    }
}

fn default_sop_sops_dir() -> Option<String> {
    None
}

fn default_sop_execution_mode() -> String {
    "supervised".to_string()
}

/// Durable SOP run-state backend selector. A closed, compile-time-known set, so it
/// is a serde enum rather than free text (mirrors `SandboxBackend` /
/// `ObservabilityBackend` in this file); unknown values are rejected at parse time.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum SopRunStoreBackend {
    /// Durable WAL-mode SQLite store (default).
    #[default]
    Sqlite,
    /// Ephemeral in-memory store: explicitly non-durable, for tests / degraded boot.
    Memory,
}

/// WHO may clear a SOP approval gate. Layered with execution_mode / priority /
/// requires_confirmation (all of which still apply). Closed set => serde enum.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ApprovalMode {
    /// Agent `sop_approve` OR an out-of-band principal both clear. Backward-compat default.
    #[default]
    Both,
    /// Only out-of-band principals (CLI / WS / HTTP / timeout) clear; `sop_approve`
    /// becomes a no-op. Lets an operator require a separate approver without bricking runs.
    OutOfBandRequired,
    /// Only the agent tool clears; out-of-band surfaces are read-only pending lists. Niche.
    AgentTool,
}

/// What happens to a SOP approval gate when it times out. Default is fail-closed:
/// the gate never self-approves. Closed set => serde enum.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ApprovalTimeoutAction {
    /// Re-surface to the out-of-band approver and keep waiting; never self-approve.
    /// Default, fail-closed.
    #[default]
    Escalate,
    /// Terminate the run (fail-safe cancel). Strict mode.
    Cancel,
    /// LEGACY: auto-approve on timeout. The ONLY path to the old fail-open behavior;
    /// opt-in only.
    AutoApprove,
}

/// `[sop.approval]` - approval broker policy config. A permanent identity source
/// for channel-provided approvers (not a stopgap): the approval broker consumes it
/// for group-membership and quorum checks. Empty = no broker policy applies.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "sop_approval"]
pub struct SopApprovalConfig {
    /// Named approver groups: `group name -> members`. A member is matched against
    /// the transport-derived (channel-authenticated) `ApprovalPrincipal` identity.
    /// A member may be source-qualified (`<source>:<identity>`, e.g. `http:OmnesAgentOperator`,
    /// `ws:<subject>`, `agent:<alias>`) to grant rights on one transport only, or a
    /// bare identity (`alice`) to grant from any non-channel source. Channel members
    /// must include the channel namespace (`channel:<channel-key>:<sender>`) so sender
    /// ids from different channel aliases cannot collide. A future auth system adds a
    /// second resolver alongside this one; it does not replace channel identities.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    #[nested]
    pub groups: std::collections::HashMap<String, ApprovalGroupConfig>,
    /// Named approval policies a SOP step may reference by name.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    #[nested]
    pub policies: std::collections::HashMap<String, ApprovalPolicyConfig>,
}

/// A named approver group (`[sop.approval.groups.<name>]`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "sop_approval_group"]
pub struct ApprovalGroupConfig {
    /// Identity labels that belong to this group.
    #[serde(default)]
    pub members: Vec<String>,
}

/// A named approval policy (`[sop.approval.policies.<name>]`) the broker enforces.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "sop_approval_policy"]
pub struct ApprovalPolicyConfig {
    /// Group whose members may satisfy this policy's gate. `None`/empty = any
    /// principal permitted by `approval_mode` (back-compat, no membership gate).
    #[serde(default)]
    pub required_group: Option<String>,
    /// Distinct approvers required before the gate clears. `0`/`1` both mean a
    /// single approval; `>= 2` requires a quorum of distinct approver identities.
    #[serde(default)]
    pub quorum: u32,
    /// Channel to deliver the INITIAL approval request to when a run parks at a
    /// gate this policy governs, formatted `channel:recipient` (e.g.
    /// `discord.ops:123456789012345678`). The `channel` names a configured channel
    /// (the `<channel>.<alias>` / bare-`<channel>` key from the channel map); the
    /// `recipient` is that channel's addressee (a Discord channel id, a chat id,
    /// etc.). Delivery is best-effort - the gate is the source of truth and a
    /// delivery failure never blocks or clears it; approvals still come back
    /// through the normal HTTP/WS/tool surfaces. `None`/empty = no out-of-band
    /// request notice (today's behavior: only the originating surface is notified).
    /// This is a DISTINCT lifecycle event from `escalation_route`: the request goes
    /// out when the run parks; the escalation goes out only if it later times out.
    /// When configured, the route must use the `channel:recipient` format.
    #[serde(default)]
    pub request_route: Option<String>,
    /// Route to escalate to on timeout (the distinct "second route"). `None`/empty
    /// re-surfaces to the same route (today's `Escalate` behavior). When configured,
    /// the route must use the `channel:recipient` format.
    #[serde(default)]
    pub escalation_route: Option<String>,
}

fn default_sop_max_concurrent_total() -> usize {
    4
}

fn default_sop_approval_timeout_secs() -> u64 {
    300
}

fn default_sop_max_finished_runs() -> usize {
    100
}

fn default_sop_persist_runs() -> bool {
    true
}

fn default_sop_step_mandatory_tools() -> Vec<String> {
    ["sop_advance", "sop_approve", "sop_status"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn default_sop_step_schema_enforce() -> bool {
    true
}

fn default_sop_max_step_visits() -> u32 {
    256
}

fn default_sop_max_step_retries() -> u32 {
    2
}

fn default_sop_untrusted_payload_max_bytes() -> usize {
    8192
}

fn default_sop_untrusted_input_guard() -> String {
    "warn".to_string()
}

fn default_sop_untrusted_guard_sensitivity() -> f64 {
    0.7
}

fn default_sop_untrusted_frame_warning() -> bool {
    true
}

fn default_sop_untrusted_outbound_redact() -> bool {
    true
}

fn default_sop_maintenance_interval_secs() -> u64 {
    60
}

impl Default for SopConfig {
    fn default() -> Self {
        Self {
            sops_dir: default_sop_sops_dir(),
            default_execution_mode: default_sop_execution_mode(),
            max_concurrent_total: default_sop_max_concurrent_total(),
            approval_timeout_secs: default_sop_approval_timeout_secs(),
            max_finished_runs: default_sop_max_finished_runs(),
            maintenance_interval_secs: default_sop_maintenance_interval_secs(),
            persist_runs: default_sop_persist_runs(),
            run_store_backend: SopRunStoreBackend::Sqlite,
            run_state_dir: None,
            approval_mode: ApprovalMode::Both,
            approval_timeout_action: ApprovalTimeoutAction::Escalate,
            approval: SopApprovalConfig::default(),
            step_scope_enforce: false,
            step_mandatory_tools: default_sop_step_mandatory_tools(),
            step_schema_enforce: default_sop_step_schema_enforce(),
            max_step_visits: default_sop_max_step_visits(),
            max_step_retries: default_sop_max_step_retries(),
            untrusted_payload_max_bytes: default_sop_untrusted_payload_max_bytes(),
            untrusted_input_guard: default_sop_untrusted_input_guard(),
            untrusted_guard_sensitivity: default_sop_untrusted_guard_sensitivity(),
            untrusted_frame_warning: default_sop_untrusted_frame_warning(),
            untrusted_outbound_redact: default_sop_untrusted_outbound_redact(),
            procedural_memory_enabled: false,
        }
    }
}

// Config enums self-declare via `#[derive(ConfigEnum)]` at their definition
// site. `DelegationPolicy` is a struct whose `mode` leaf surfaces as the enum,
// so it carries the impl directly.
impl HasPropKind for crate::autonomy::DelegationPolicy {
    const PROP_KIND: PropKind = PropKind::Enum;
}

impl HasPropKind for serde_json::Value {
    // `serde_json::Value` is an arbitrary JSON document, not an enum.
    // Classifying it as `Enum` previously made `enum_variants_for::<Value>()`
    // hand back the literal placeholder `"(unknown variants)"`, and the
    // dashboard form rendered fields like `model_providers.<key>.provider_extra`
    // as a single-option dropdown. `String` is the closest scalar kind —
    // the form renders a text input where the user pastes raw JSON.
    // Round-trip via `set_prop` stays correct: serde deserializes the TOML
    // string back into `Value::String(...)`. Power users editing complex
    // objects still use `omnesagent config set --json` or hand-edit the
    // `config.toml`.
    const PROP_KIND: PropKind = PropKind::String;
}
