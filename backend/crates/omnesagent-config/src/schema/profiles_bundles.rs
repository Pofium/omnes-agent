//! Risk/runtime profiles (`[risk_profiles.*]`, `[runtime_profiles.*]`), the
//! skill/knowledge/MCP bundles, and the legacy per-channel configs that are
//! not part of the `channels` module (DingTalk, WeCom, WeChat, QQ, Twitter,
//! Mochat, Reddit, Bluesky, Git, voice-wake, Nostr, Notion, Jira, CloudOps).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::{RiskProfileConfig, ...}` path is unchanged.

use super::*;

/// Named risk/autonomy profile (`[risk_profiles.<alias>]`).
///
/// Unified policy surface. Agents reference a profile by alias and the
/// runtime resolves through it for shell command allowlists, approval gates,
/// sandbox/resource limits, and delegation guardrails. The conventional
/// `risk_profiles["default"]` is the resolution target for non-agent
/// contexts (orchestrator init, cron worker startup); the `Default` impl
/// below mirrors the legacy safety-first defaults so a fresh install
/// behaves the same as a config from before the per-profile split.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "risk_profile"]
#[serde(default)]
pub struct RiskProfileConfig {
    /// Autonomy level applied to this profile. Default: `supervised`.
    pub level: AutonomyLevel,
    /// Restrict filesystem access to workspace-relative paths. Default: `false`.
    pub workspace_only: bool,
    /// Allowlist of executable names for shell execution.
    pub allowed_commands: Vec<String>,
    /// Explicit path denylist.
    pub forbidden_paths: Vec<String>,
    /// Require approval for medium-risk operations.
    pub require_approval_for_medium_risk: bool,
    /// Block high-risk commands even when allowlisted.
    pub block_high_risk_commands: bool,
    /// Environment variable names passed through to shell subprocesses.
    #[credential_class = "legacy_env_path"]
    pub shell_env_passthrough: Vec<String>,
    /// Tools that never require approval in this profile.
    pub auto_approve: Vec<String>,
    /// Tools that always require approval in this profile.
    pub always_ask: Vec<String>,
    /// Extra directory roots the agent may access.
    #[serde(alias = "allowed_path", alias = "allowed_paths")]
    pub allowed_roots: Vec<String>,
    /// Whether agents using this profile may initiate delegation. Defaults to
    /// `Forbidden`. Reachable delegate targets and bounded/independent mode
    /// are resolved from each caller agent's config.
    #[serde(default)]
    #[nested]
    pub delegation_policy: DelegationPolicy,
    /// Route this profile's tool approvals to a DISTINCT approver channel instead of the
    /// channel that triggered the run (closes the cross-channel-HITL gap). Absent ⇒ the
    /// originating channel approves (today's behavior). See [`crate::autonomy::ApprovalRoute`].
    ///
    /// Honored on both the interactive channel-driven path and the non-interactive turn path
    /// (gateway chat/webhook dispatch and agent-to-agent peer messages). On the non-interactive
    /// path the approver is resolved from the live daemon channel registry; if no registry is
    /// available or the approver is not live, the gate keeps the non-interactive default
    /// (fail-closed deny under the default `on_no_approver`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_route: Option<crate::autonomy::ApprovalRoute>,
    /// Tools the agent may call in agentic mode. Empty = inherit / no
    /// authorization constraint. Authorization decision: which tools is
    /// the agent permitted to invoke at all. See `excluded_tools` for
    /// the inverse denylist scoped to non-CLI channels.
    ///
    /// The TOML config does not distinguish an omitted field from
    /// `allowed_tools = []`; both deserialize to `Vec::new()` and
    /// `SecurityPolicy::from_profiles` maps that to "no authorization
    /// constraint" at this layer. If you need an explicit deny-all gate,
    /// apply it on the caller-supplied per-run `allowed_tools` (cron
    /// jobs and other narrowers pass that list in directly to
    /// `ToolAccessPolicy`, which honors `Some(vec![])` as deny-all) or
    /// via `excluded_tools` covering the specific tools you want blocked.
    ///
    /// MCP exception: when the list is non-empty, runtime-discovered MCP
    /// tools (any name containing `__`, which is the `<server>__<tool>`
    /// convention used by the MCP wrapper) are auto-admitted into the
    /// effective allow-list without needing to be listed here individually.
    /// This keeps the post-change eager-MCP default usable for agents with an
    /// explicit allow-list. Block individual MCP tools via `excluded_tools`.
    ///
    /// Scope of the exception: the `__` auto-admit applies only to this
    /// risk-profile allow-list, **not** to caller-supplied per-run
    /// `allowed_tools` (cron job `allowed_tools`, narrowed delegate
    /// invocations, etc.). Per-run lists are still strict explicit-list
    /// intersections, so a job that narrows `allowed_tools = ["cron_add"]`
    /// will not see runtime-discovered MCP tools unless it names them.
    ///
    pub allowed_tools: Vec<String>,
    /// Explicit deny-all for this profile: no tool may be invoked under it
    /// (built-ins, MCP tools, and skill-defined tools alike — there is no
    /// `__` auto-admit under deny-all). `allowed_tools = []` remains
    /// legacy-unrestricted; setting both `deny_all_tools = true` and a
    /// non-empty `allowed_tools` is a configuration error rejected by
    /// [`Config::validate`].
    #[serde(default, skip_serializing_if = "is_false")]
    pub deny_all_tools: bool,
    /// Tools excluded from non-CLI channels under this profile.
    ///
    /// Also subtracts from the agentic-delegate allow-list resolved at
    /// runtime, which is the only way to block individual
    /// `<server>__<tool>` MCP names that would otherwise be auto-admitted
    /// by the `allowed_tools` MCP exception described above.
    pub excluded_tools: Vec<String>,
    // ── Sandbox (from security.sandbox) ─────────────────────────────
    /// Whether the sandbox is enabled for this profile. `None` inherits global.
    pub sandbox_enabled: Option<bool>,
    /// Sandbox backend identifier (e.g. `"firejail"`, `"landlock"`). `None` inherits.
    pub sandbox_backend: Option<String>,
    /// Extra arguments forwarded to firejail when sandbox_backend = "firejail".
    pub firejail_args: Vec<String>,
}

impl Default for RiskProfileConfig {
    fn default() -> Self {
        Self {
            level: AutonomyLevel::Supervised,
            workspace_only: true,
            allowed_commands: crate::policy::default_allowed_commands(),
            forbidden_paths: crate::policy::default_forbidden_paths(),
            require_approval_for_medium_risk: true,
            block_high_risk_commands: true,
            shell_env_passthrough: vec![],
            auto_approve: default_auto_approve(),
            always_ask: default_always_ask(),
            allowed_roots: Vec::new(),
            delegation_policy: DelegationPolicy::default(),
            approval_route: None,
            allowed_tools: Vec::new(),
            deny_all_tools: false,
            excluded_tools: Vec::new(),
            sandbox_enabled: None,
            sandbox_backend: None,
            firejail_args: Vec::new(),
        }
    }
}

/// Named runtime/LLM execution profile (`[runtime_profiles.<alias>]`).
///
/// Reusable operational tuning: agentic mode, iteration caps, context
/// budget, parallel dispatch, resource ceilings, recursion depth, and
/// the budget knobs that `SecurityPolicy` enforces with subagent
/// parent-subset discipline. Anything authorization-shaped (allowed
/// commands/tools/paths, approval gates, sandbox) lives on
/// `[risk_profiles.<alias>]`. Anything model-provider shaped (model,
/// temperature, max_tokens, timeout_secs) lives on
/// `[providers.models.<type>.<alias>]`.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "runtime_profile"]
#[serde(default)]
pub struct RuntimeProfileConfig {
    /// Enable agentic (multi-turn tool-call loop) mode.
    pub agentic: bool,
    /// Maximum tool-call iterations in agentic mode. `0` inherits the global default.
    pub max_tool_iterations: usize,
    // ── Budget caps (enforced with subagent parent-subset discipline) ──
    /// Maximum actions allowed per hour. `0` is a hard zero budget — the
    /// per-sender rate tracker treats a max of 0 as always exhausted
    /// (`PerSenderTracker::is_exhausted`), blocking every action. For an
    /// effectively-unlimited budget use a high value (e.g. `u32::MAX`), not 0.
    /// `SecurityPolicy::ensure_no_escalation_beyond` rejects subagents
    /// that try to raise this above the parent's value.
    pub max_actions_per_hour: u32,
    /// Maximum cost per day in cents. `0` inherits the global limit.
    /// Parent-subset enforced for subagents.
    pub max_cost_per_day_cents: u32,
    /// Shell subprocess timeout in seconds. `0` inherits the global timeout.
    /// Parent-subset enforced for subagents.
    pub shell_timeout_secs: u64,
    // ── Delegation tuning ──
    /// Maximum delegation recursion depth. `0` inherits the default.
    pub max_delegation_depth: u32,
    /// Delegate call timeout in seconds. `None` inherits global delegate timeout.
    pub delegation_timeout_secs: Option<u64>,
    /// Agentic delegate run timeout in seconds. `None` inherits global.
    pub agentic_timeout_secs: Option<u64>,
    // ── Per-agent runtime tunables (also live on AliasedAgentConfig) ─
    /// Maximum conversation history messages retained per session. `None` inherits.
    pub max_history_messages: Option<usize>,
    /// Fraction of `max_history_messages` that a whole-turn history trim
    /// drops the history to (the hysteresis low-water mark). Valid range is
    /// `(0.0, 1.0]`; `1.0` disables hysteresis and trims straight back to
    /// the cap. `None` inherits the default (0.7).
    pub history_trim_low_water: Option<f32>,
    /// Maximum estimated tokens for context before compaction. `None` inherits.
    pub max_context_tokens: Option<usize>,
    /// Use compact bootstrap (6000 chars / 2 RAG chunks). `None` inherits.
    pub compact_context: Option<bool>,
    /// Enable parallel tool execution per iteration. `None` inherits.
    pub parallel_tools: Option<bool>,
    /// Tool dispatch strategy (e.g. `"auto"`). `None` inherits.
    pub tool_dispatcher: Option<String>,
    /// Tools exempt from within-turn dedup check.
    pub tool_call_dedup_exempt: Vec<String>,
    /// Maximum characters for the assembled system prompt. `None` inherits.
    pub max_system_prompt_chars: Option<usize>,
    /// Maximum characters for a single tool result. `None` inherits.
    pub max_tool_result_chars: Option<usize>,
    /// Number of recent turns whose full tool context is preserved. `None` inherits.
    pub keep_tool_context_turns: Option<usize>,
    /// Maximum memory entries injected per turn. `None` inherits global default (5).
    /// Set to `0` for unlimited.
    pub memory_recall_limit: Option<usize>,
    /// How skills are injected into the system prompt for agents on this
    /// profile. `None` inherits the global `[skills] prompt_injection_mode`;
    /// `compact` inlines only compact skill metadata and registers the
    /// `read_skill` tool, while `full` inlines full skill instructions. Resolved
    /// through [`Config::effective_skills_prompt_mode`], the single point both
    /// the system-prompt builder and the `read_skill` tool-registration gate
    /// consult so they always agree on the effective mode.
    pub prompt_injection_mode: Option<SkillsPromptInjectionMode>,
    pub strict_tool_parsing: bool,
    #[nested]
    pub thinking: crate::scattered_types::ThinkingConfig,
    #[nested]
    pub history_pruning: crate::scattered_types::HistoryPrunerConfig,
    #[nested]
    pub eval: crate::scattered_types::EvalConfig,
    #[nested]
    pub auto_classify: Option<crate::scattered_types::AutoClassifyConfig>,
    #[nested]
    pub context_compression: crate::scattered_types::ContextCompressionConfig,
    #[nested]
    pub tool_receipts: ToolReceiptsConfig,
    pub tool_filter_groups: Vec<ToolFilterGroup>,
}

impl Default for RuntimeProfileConfig {
    fn default() -> Self {
        Self {
            agentic: false,
            max_tool_iterations: 0,
            max_actions_per_hour: 20,
            max_cost_per_day_cents: 500,
            shell_timeout_secs: 60,
            max_delegation_depth: 0,
            delegation_timeout_secs: None,
            agentic_timeout_secs: None,
            max_history_messages: None,
            history_trim_low_water: None,
            max_context_tokens: None,
            compact_context: None,
            parallel_tools: None,
            tool_dispatcher: None,
            tool_call_dedup_exempt: Vec::new(),
            max_system_prompt_chars: None,
            max_tool_result_chars: None,
            keep_tool_context_turns: None,
            memory_recall_limit: None,
            prompt_injection_mode: None,
            strict_tool_parsing: false,
            thinking: crate::scattered_types::ThinkingConfig::default(),
            history_pruning: crate::scattered_types::HistoryPrunerConfig::default(),
            eval: crate::scattered_types::EvalConfig::default(),
            auto_classify: None,
            context_compression: crate::scattered_types::ContextCompressionConfig::default(),
            tool_receipts: ToolReceiptsConfig::default(),
            tool_filter_groups: Vec::new(),
        }
    }
}

/// Named skill bundle (`[skill_bundles.<alias>]`).
///
/// A reusable group of skills that can be attached to an agent or channel
/// by alias, controlling which skills are loaded and from where.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "skill_bundle"]
#[serde(default)]
pub struct SkillBundleConfig {
    /// Directory path (relative to workspace root) to load skills from.
    pub directory: Option<String>,
    /// Skill names to include. Empty means include all skills in `directory`.
    pub include: Vec<String>,
    /// Skill names to exclude from this bundle.
    pub exclude: Vec<String>,
}

impl SkillBundleConfig {
    /// Whether a skill name survives this bundle's include/exclude filter.
    /// Empty `include` admits every name; `exclude` always wins.
    pub fn admits_skill(&self, name: &str) -> bool {
        if !self.include.is_empty() && !self.include.iter().any(|s| s == name) {
            return false;
        }
        !self.exclude.iter().any(|s| s == name)
    }
}

/// Named knowledge bundle (`[knowledge_bundles.<alias>]`).
///
/// A reusable set of knowledge sources (documents, URLs, or RAG corpus paths)
/// that can be attached to an agent by alias.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "knowledge_bundle"]
#[serde(default)]
pub struct KnowledgeBundleConfig {
    /// Paths or URLs to include in this knowledge bundle.
    pub sources: Vec<String>,
    /// Tags for filtering or categorising sources within the bundle.
    pub tags: Vec<String>,
}

/// Named MCP server bundle (`[mcp_bundles.<alias>]`).
///
/// A reusable group of MCP servers granted to an agent that references the
/// bundle by alias in `agents.<alias>.mcp_bundles`. Server IDs are matched
/// against `[mcp.servers]` by `name`. Resolution is secure by default (see
/// `Config::mcp_servers_for_bundles`): an ID with no matching server grants
/// nothing, and `exclude` wins over `servers` across every bundle an agent
/// references.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "mcp_bundle"]
#[serde(default)]
pub struct McpBundleConfig {
    /// MCP server IDs (`[mcp.servers].name`) granted by this bundle.
    pub servers: Vec<String>,
    /// MCP server IDs removed from the grant. Deny wins: a name listed here is
    /// excluded even if another referenced bundle includes it.
    pub exclude: Vec<String>,
}



/// DingTalk configuration for Stream Mode messaging
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.dingtalk"]
pub struct DingTalkConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Client ID (AppKey) from DingTalk developer console
    #[tab(Connection)]
    pub client_id: String,
    /// Client Secret (AppSecret) from DingTalk developer console
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub client_secret: String,
    /// Per-channel proxy URL (http, https, socks5, socks5h).
    /// Overrides the global `[proxy]` setting for this channel only.
    #[tab(Advanced)]
    #[serde(default)]
    pub proxy_url: Option<String>,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl ChannelConfig for DingTalkConfig {
    fn name() -> &'static str {
        "DingTalk"
    }
    fn desc() -> &'static str {
        "DingTalk Stream Mode"
    }
}

/// WeCom (WeChat Enterprise) Bot Webhook configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.wecom"]
pub struct WeComConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Webhook key from WeCom Bot configuration
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub webhook_key: String,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl ChannelConfig for WeComConfig {
    fn name() -> &'static str {
        "WeCom"
    }
    fn desc() -> &'static str {
        "WeCom Bot Webhook"
    }
}

fn default_wecom_ws_file_retention_days() -> u32 {
    7
}

fn default_wecom_ws_max_file_size_mb() -> u64 {
    20
}

fn default_wecom_ws_stream_mode() -> StreamMode {
    StreamMode::Partial
}

/// WeCom AI Bot WebSocket configuration.
///
/// This is distinct from webhook-based [`WeComConfig`] and uses the WeCom AI
/// Bot long-connection API for inbound messages and active-session replies.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.wecom_ws"]
pub struct WeComWsConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[serde(default)]
    pub enabled: bool,
    /// Bot ID for WeCom WebSocket subscription.
    pub bot_id: String,
    /// Secret for WeCom WebSocket subscription authentication.
    #[secret]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub secret: String,
    /// Allowed WeCom user IDs. Empty = deny all, "*" = allow all users.
    #[serde(default)]
    pub allowed_users: Vec<String>,
    /// Allowed WeCom group chat IDs. Empty = deny all groups, "*" = allow all groups.
    #[serde(default)]
    pub allowed_groups: Vec<String>,
    /// Display name or mention alias of the WeCom AI bot, for example `danya`.
    ///
    /// WeCom group text often arrives as plain text such as `@danya say hi`;
    /// passing this name through lets the generic reply-intent precheck
    /// recognize that a group message was addressed to the bot.
    #[serde(default)]
    pub bot_name: Option<String>,
    /// File retention days for downloaded WeCom attachments under the workspace cache.
    #[serde(default = "default_wecom_ws_file_retention_days")]
    pub file_retention_days: u32,
    /// Maximum accepted file size in MiB for WeCom attachment download attempts.
    #[serde(default = "default_wecom_ws_max_file_size_mb")]
    pub max_file_size_mb: u64,
    /// Streaming mode for progressive draft delivery over the WeCom long connection.
    #[serde(default = "default_wecom_ws_stream_mode")]
    pub stream_mode: StreamMode,
    /// Optional per-channel proxy override. Falls back to the global proxy config when empty.
    #[serde(default)]
    pub proxy_url: Option<String>,
    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl Default for WeComWsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bot_id: String::new(),
            secret: String::new(),
            allowed_users: Vec::new(),
            allowed_groups: Vec::new(),
            bot_name: None,
            file_retention_days: default_wecom_ws_file_retention_days(),
            max_file_size_mb: default_wecom_ws_max_file_size_mb(),
            stream_mode: default_wecom_ws_stream_mode(),
            proxy_url: None,
            excluded_tools: Vec::new(),
        }
    }
}

impl ChannelConfig for WeComWsConfig {
    fn name() -> &'static str {
        "WeCom WebSocket"
    }
    fn desc() -> &'static str {
        "WeCom AI Bot long connection"
    }
}

/// WeChat personal iLink Bot channel configuration.
///
/// Uses the iLink Bot API (`ilinkai.weixin.qq.com`) with QR-code login.
/// The bot token is obtained by scanning a QR code and persisted to disk
/// so subsequent restarts do not require re-scanning.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.wechat"]
pub struct WeChatConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Override the iLink API base URL. Default: `https://ilinkai.weixin.qq.com`.
    #[tab(Advanced)]
    #[serde(default)]
    pub api_base_url: Option<String>,
    /// Override the CDN base URL. Default: `https://novac2c.cdn.weixin.qq.com/c2c`.
    #[tab(Advanced)]
    #[serde(default)]
    pub cdn_base_url: Option<String>,
    /// Directory to persist bot token and sync cursor.
    /// Default: `~/.omnesagent/wechat/`.
    #[tab(Advanced)]
    #[serde(default)]
    pub state_dir: Option<String>,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl ChannelConfig for WeChatConfig {
    fn name() -> &'static str {
        "WeChat"
    }
    fn desc() -> &'static str {
        "WeChat iLink Bot"
    }
}

/// QQ Official Bot configuration (Tencent QQ Bot SDK)
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.qq"]
pub struct QQConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// App ID from QQ Bot developer console
    #[tab(Connection)]
    pub app_id: String,
    /// App Secret from QQ Bot developer console
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub app_secret: String,
    /// Per-channel proxy URL (http, https, socks5, socks5h).
    /// Overrides the global `[proxy]` setting for this channel only.
    #[tab(Advanced)]
    #[serde(default)]
    pub proxy_url: Option<String>,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl ChannelConfig for QQConfig {
    fn name() -> &'static str {
        "QQ Official"
    }
    fn desc() -> &'static str {
        "Tencent QQ Bot"
    }
}

/// X/Twitter channel configuration (Twitter API v2)
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.twitter"]
pub struct TwitterConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Twitter API v2 Bearer Token (OAuth 2.0)
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub bearer_token: String,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl ChannelConfig for TwitterConfig {
    fn name() -> &'static str {
        "X/Twitter"
    }
    fn desc() -> &'static str {
        "X/Twitter Bot via API v2"
    }
}

/// Mochat channel configuration (Mochat customer service API)
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.mochat"]
pub struct MochatConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Mochat API base URL
    #[tab(Advanced)]
    pub api_url: String,
    /// Mochat API token
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub api_token: String,
    /// Poll interval in seconds for new messages. Default: 5
    #[tab(Advanced)]
    #[serde(default = "default_mochat_poll_interval")]
    pub poll_interval_secs: u64,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

fn default_mochat_poll_interval() -> u64 {
    5
}

impl ChannelConfig for MochatConfig {
    fn name() -> &'static str {
        "Mochat"
    }
    fn desc() -> &'static str {
        "Mochat Customer Service"
    }
}

/// Reddit channel configuration (OAuth2 bot).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.reddit"]
pub struct RedditConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Reddit OAuth2 client ID.
    #[tab(Connection)]
    pub client_id: String,
    /// Reddit OAuth2 client secret.
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub client_secret: String,
    /// Reddit OAuth2 refresh token for persistent access.
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub refresh_token: String,
    /// Reddit bot username (without `u/` prefix).
    #[tab(Advanced)]
    pub username: String,
    /// Subreddits to filter messages (without `r/` prefix). Empty = accept
    /// from any subreddit the bot has access to. Migrated from the legacy
    /// `subreddit` singular field.
    #[tab(Advanced)]
    #[serde(default)]
    pub subreddits: Vec<String>,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl ChannelConfig for RedditConfig {
    fn name() -> &'static str {
        "Reddit"
    }
    fn desc() -> &'static str {
        "Reddit bot (OAuth2)"
    }
}

/// Bluesky channel configuration (AT Protocol).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.bluesky"]
pub struct BlueskyConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Bluesky handle (e.g. `"mybot.bsky.social"`).
    #[tab(Connection)]
    pub handle: String,
    /// App-specific password (from Bluesky settings).
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub app_password: String,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl ChannelConfig for BlueskyConfig {
    fn name() -> &'static str {
        "Bluesky"
    }
    fn desc() -> &'static str {
        "AT Protocol"
    }
}

/// Git-forge channel configuration (polling-based; no webhook required).
///
/// A `provider` selects the forge. GitHub authenticates as a GitHub App:
/// a short-lived RS256 JWT signed with the app's private key is exchanged
/// for per-installation access tokens. Gitea and Forgejo use a personal
/// access token against the instance's `/api/v1` endpoint, named by the
/// required `api_base_url`. Inbound issue/PR comments are polled from the
/// forge REST API, so the daemon needs no inbound network exposure.
#[derive(Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.git"]
pub struct GitConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Git forge provider. Supported: `"github"`, `"gitea"`, and
    /// `"forgejo"` (Forgejo uses the Gitea-compatible REST provider).
    /// Default: `"github"`.
    #[tab(Connection)]
    #[serde(default = "default_git_provider")]
    pub provider: String,
    /// GitHub App ID (shown on the app's settings page). GitHub provider only.
    #[tab(Connection)]
    #[serde(default)]
    pub app_id: u64,
    /// RS256 private key PEM, the contents of the `.pem` file GitHub
    /// generates on the app's settings page, inline and encrypted at rest.
    /// Include the BEGIN/END lines. GitHub provider only.
    #[secret]
    #[multiline]
    #[credential_class = "encrypted_secret"]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    /// Filesystem path to the RS256 private key `.pem` file, read at startup
    /// when `private_key` (inline PEM) is unset. Backward-compatible fallback
    /// for configs that predate the inline-PEM field. GitHub provider only.
    #[tab(Connection)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_key_path: Option<String>,
    /// Installation ID to act as. When unset, the app's installations are
    /// listed on first use and a sole installation is auto-selected;
    /// startup fails if the app has zero or multiple installations.
    #[tab(Connection)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installation_id: Option<u64>,
    /// Gitea/Forgejo API base URL, including `/api/v1`, for example
    /// `https://git.example.org/api/v1` (for the public Gitea service:
    /// `https://gitea.com/api/v1`). Required when `provider` is `"gitea"`
    /// or `"forgejo"` - there is no default host, because every API
    /// request carries `access_token`; the channel fails closed at
    /// startup rather than send the token to an endpoint the operator
    /// never named. Gitea/Forgejo provider only.
    #[tab(Connection)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_base_url: Option<String>,
    /// Personal access token for Gitea/Forgejo API requests. The token needs
    /// repository read access plus issue/PR comment write access for replies
    /// and reactions. Gitea/Forgejo provider only.
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    #[serde(default)]
    pub access_token: String,
    /// Repositories to poll, as `owner/repo`. Empty = every repository
    /// visible to the installation.
    #[tab(Advanced)]
    #[serde(default)]
    pub repos: Vec<String>,
    /// Poll interval in seconds for new issues and comments. Values below
    /// 15 are clamped to 15. Default: `30`.
    #[tab(Advanced)]
    #[serde(default = "default_github_poll_interval_secs")]
    pub poll_interval_secs: u64,
    /// Only respond to comments that @-mention the app's bot login.
    /// Default: `true`.
    #[tab(Behavior)]
    #[serde(default = "default_true")]
    pub mention_only: bool,
    /// Process comments authored by other bot accounts. The app's own
    /// comments are always ignored. Default: `false`.
    #[tab(Behavior)]
    #[serde(default)]
    pub listen_to_bots: bool,
    /// Per-channel proxy override for GitHub API requests.
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    /// Per-event routing table, keyed by normalized event type
    /// (`"issue_comment.created"`, `"pull_request.opened"`,
    /// `"workflow_run.failed"`, …). Event types absent from the table fall
    /// back to the conversational default: `issue_comment.created`,
    /// `issues.opened`, and `pull_request.opened` are delivered as
    /// messages (mention-gated); everything else is ignored. Which API
    /// endpoints are polled is derived from this table; routing an event
    /// type is also subscribing to it.
    #[tab(Behavior)]
    #[serde(default)]
    #[nested]
    pub events: HashMap<String, GitEventRoute>,
    /// Also poll the repository Events API (`/repos/{owner}/{repo}/events`)
    /// as a broad backbone transport: one conditional (ETag) request per
    /// repository per tick, so an idle repo costs almost nothing. Caveats:
    /// events arrive with a delay of up to ~5 minutes and the feed carries
    /// no Actions/check events (workflow runs always use their dedicated
    /// endpoint). Items also surfaced by a targeted endpoint are
    /// de-duplicated. Default: `false`.
    #[tab(Advanced)]
    #[serde(default)]
    pub events_backbone: bool,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl std::fmt::Debug for GitConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitConfig")
            .field("enabled", &self.enabled)
            .field("provider", &self.provider)
            .field("app_id", &self.app_id)
            .field("private_key", &self.private_key.as_ref().map(|_| "***"))
            .field("private_key_path", &self.private_key_path)
            .field("installation_id", &self.installation_id)
            .field("api_base_url", &self.api_base_url)
            .field(
                "access_token",
                &if self.access_token.is_empty() {
                    ""
                } else {
                    "***"
                },
            )
            .field("repos", &self.repos)
            .field("poll_interval_secs", &self.poll_interval_secs)
            .field("mention_only", &self.mention_only)
            .field("listen_to_bots", &self.listen_to_bots)
            .field("proxy_url", &self.proxy_url)
            .field("events", &self.events)
            .field("events_backbone", &self.events_backbone)
            .field("excluded_tools", &self.excluded_tools)
            .finish()
    }
}

fn default_github_poll_interval_secs() -> u64 {
    30
}

fn default_git_provider() -> String {
    "github".to_string()
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_git_provider(),
            app_id: 0,
            private_key: None,
            private_key_path: None,
            installation_id: None,
            api_base_url: None,
            access_token: String::new(),
            repos: Vec::new(),
            poll_interval_secs: default_github_poll_interval_secs(),
            mention_only: true,
            listen_to_bots: false,
            proxy_url: None,
            events: HashMap::new(),
            events_backbone: false,
            excluded_tools: Vec::new(),
        }
    }
}

impl ChannelConfig for GitConfig {
    fn name() -> &'static str {
        "Git"
    }
    fn desc() -> &'static str {
        "Git forge (GitHub, Gitea, Forgejo): issues, PRs & events"
    }
}

/// Routing for one normalized git-forge event type, keyed in
/// `[channels.git.<alias>.events]` by the event type string
/// (`"pull_request.opened" = { sop = "pr-triage" }`).
///
/// An entry with neither `message = true` nor a `sop` explicitly ignores the
/// event type (overriding the conversational default).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.git.events"]
pub struct GitEventRoute {
    /// Deliver the event into the agent loop as a normal channel message.
    #[serde(default)]
    pub message: bool,
    /// Route the event to the named SOP through channel-sourced SOP ingress.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sop: Option<String>,
}

/// Voice duplex configuration (`[channels.voice_duplex]`).
///
/// Enables full-duplex voice event handling over WebSocket.
/// When disabled (default), voice events are rejected as unknown types.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct VoiceDuplexConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[serde(default)]
    pub enabled: bool,
    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

/// Voice wake word detection channel configuration.
///
/// Listens on the default microphone for a configurable wake word,
/// then captures the following utterance and transcribes it via the
/// existing transcription API.
#[derive(Debug, Clone, Serialize, Deserialize, omnesagent_macros::Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "voice_wake"]
pub struct VoiceWakeConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[serde(default)]
    pub enabled: bool,
    /// Wake word phrase to listen for (case-insensitive substring match).
    /// Default: `"hey omnesagent"`.
    #[serde(default = "default_voice_wake_word")]
    pub wake_word: String,
    /// Silence timeout in milliseconds: how long to wait after the last
    /// energy spike before finalizing a capture window. Default: `2000`.
    #[serde(default = "default_voice_wake_silence_timeout_ms")]
    pub silence_timeout_ms: u32,
    /// RMS energy threshold for voice activity detection. Samples below
    /// this level are treated as silence. Default: `0.01`.
    #[serde(default = "default_voice_wake_energy_threshold")]
    pub energy_threshold: f32,
    /// Maximum capture duration in seconds before forcing transcription.
    /// Default: `30`.
    #[serde(default = "default_voice_wake_max_capture_secs")]
    pub max_capture_secs: u32,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

fn default_voice_wake_word() -> String {
    "hey omnesagent".into()
}

fn default_voice_wake_silence_timeout_ms() -> u32 {
    2000
}

fn default_voice_wake_energy_threshold() -> f32 {
    0.01
}

fn default_voice_wake_max_capture_secs() -> u32 {
    30
}

impl Default for VoiceWakeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            wake_word: default_voice_wake_word(),
            silence_timeout_ms: default_voice_wake_silence_timeout_ms(),
            energy_threshold: default_voice_wake_energy_threshold(),
            max_capture_secs: default_voice_wake_max_capture_secs(),
            excluded_tools: Vec::new(),
        }
    }
}

impl ChannelConfig for VoiceWakeConfig {
    fn name() -> &'static str {
        "VoiceWake"
    }
    fn desc() -> &'static str {
        "voice wake word detection"
    }
}

/// Nostr channel configuration (NIP-04 + NIP-17 private messages)
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.nostr"]
pub struct NostrConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Private key in hex or nsec bech32 format
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub private_key: String,
    /// Relay URLs (wss://). Defaults to popular public relays if omitted.
    #[tab(Advanced)]
    #[serde(default = "default_nostr_relays")]
    pub relays: Vec<String>,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl ChannelConfig for NostrConfig {
    fn name() -> &'static str {
        "Nostr"
    }
    fn desc() -> &'static str {
        "Nostr DMs"
    }
}

pub fn default_nostr_relays() -> Vec<String> {
    vec![
        "wss://relay.damus.io".to_string(),
        "wss://nos.lol".to_string(),
        "wss://relay.primal.net".to_string(),
        "wss://relay.snort.social".to_string(),
    ]
}

// -- Notion --

/// Notion integration configuration (`[notion]`).
///
/// When `enabled = true`, the agent polls a Notion database for pending tasks
/// and exposes a `notion` tool for querying, reading, creating, and updating pages.
/// Requires `api_key` (or the `NOTION_API_KEY` env var) and `database_id`.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "notion"]
pub struct NotionConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub api_key: String,
    #[serde(default)]
    pub database_id: String,
    #[serde(default = "default_notion_poll_interval")]
    pub poll_interval_secs: u64,
    #[serde(default = "default_notion_status_prop")]
    pub status_property: String,
    #[serde(default = "default_notion_input_prop")]
    pub input_property: String,
    #[serde(default = "default_notion_result_prop")]
    pub result_property: String,
    #[serde(default = "default_notion_max_concurrent")]
    pub max_concurrent: usize,
    #[serde(default = "default_notion_recover_stale")]
    pub recover_stale: bool,
}

fn default_notion_poll_interval() -> u64 {
    5
}
fn default_notion_status_prop() -> String {
    "Status".into()
}
fn default_notion_input_prop() -> String {
    "Input".into()
}
fn default_notion_result_prop() -> String {
    "Result".into()
}
fn default_notion_max_concurrent() -> usize {
    4
}
fn default_notion_recover_stale() -> bool {
    true
}

impl Default for NotionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: String::new(),
            database_id: String::new(),
            poll_interval_secs: default_notion_poll_interval(),
            status_property: default_notion_status_prop(),
            input_property: default_notion_input_prop(),
            result_property: default_notion_result_prop(),
            max_concurrent: default_notion_max_concurrent(),
            recover_stale: default_notion_recover_stale(),
        }
    }
}

/// Jira integration configuration (`[jira]`).
///
/// When `enabled = true`, registers the `jira` tool which can get tickets,
/// search with JQL, and add comments. Requires `base_url` and `api_token`
/// (or the `JIRA_API_TOKEN` env var).
///
/// ## Defaults
/// - `enabled`: `false`
/// - `allowed_actions`: `["get_ticket"]` — read-only by default.
///   Add `"search_tickets"` or `"comment_ticket"` to unlock them.
/// - `timeout_secs`: `30`
///
/// ## Auth
/// Jira Cloud uses HTTP Basic auth: `email` + `api_token`.
/// Jira Server/Data Center uses Bearer token auth: omit `email` and set
/// `api_token` to a personal access token.
/// `api_token` is stored encrypted at rest; set it here or via `JIRA_API_TOKEN`.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "jira"]
pub struct JiraConfig {
    /// Enable the `jira` tool. Default: `false`.
    #[serde(default)]
    pub enabled: bool,
    /// Atlassian instance base URL, e.g. `https://yourco.atlassian.net`.
    #[serde(default)]
    pub base_url: String,
    /// Jira account email used for Basic auth (Cloud).
    /// Omit for Server/DC deployments using Bearer token auth.
    /// An empty string (`email = ""`) deserializes as `None`. Configs
    /// that round-tripped the empty default to disk would otherwise
    /// silently regress to Basic auth with empty username, since the
    /// email-required validation was dropped when Server/DC Bearer-token
    /// support landed.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_email_skip_empty"
    )]
    pub email: Option<String>,
    /// Jira API token. Encrypted at rest. Falls back to `JIRA_API_TOKEN` env var.
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub api_token: String,
    /// Actions the agent is permitted to call.
    /// Valid values: `"get_ticket"`, `"search_tickets"`, `"comment_ticket"`,
    /// `"list_projects"`, `"myself"`, `"list_transitions"`,
    /// `"transition_ticket"`, `"create_ticket"`.
    /// Defaults to `["get_ticket"]` (read-only).
    #[serde(default = "default_jira_allowed_actions")]
    pub allowed_actions: Vec<String>,
    /// Request timeout in seconds. Default: `30`.
    #[serde(default = "default_jira_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_jira_allowed_actions() -> Vec<String> {
    vec!["get_ticket".to_string()]
}

fn default_jira_timeout_secs() -> u64 {
    30
}

impl Default for JiraConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: String::new(),
            email: None,
            api_token: String::new(),
            allowed_actions: default_jira_allowed_actions(),
            timeout_secs: default_jira_timeout_secs(),
        }
    }
}

///
/// Controls the read-only cloud transformation analysis tools:
/// IaC review, migration assessment, cost analysis, and architecture review.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "cloud_ops"]
pub struct CloudOpsConfig {
    /// Enable cloud operations tools. Default: false.
    #[serde(default)]
    pub enabled: bool,
    /// Default cloud model_provider for analysis context. Default: "aws".
    #[serde(default = "default_cloud_ops_cloud")]
    pub default_cloud: String,
    /// Supported cloud model_providers. Default: [`aws`, `azure`, `gcp`].
    #[serde(default = "default_cloud_ops_supported_clouds")]
    pub supported_clouds: Vec<String>,
    /// Supported IaC tools for review. Default: \[`terraform`\].
    #[serde(default = "default_cloud_ops_iac_tools")]
    pub iac_tools: Vec<String>,
    /// Monthly USD threshold to flag cost items. Default: 100.0.
    #[serde(default = "default_cloud_ops_cost_threshold")]
    pub cost_threshold_monthly_usd: f64,
    /// Well-Architected Frameworks to check against. Default: \[`aws-waf`\].
    #[serde(default = "default_cloud_ops_waf")]
    pub well_architected_frameworks: Vec<String>,
}

impl Default for CloudOpsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_cloud: default_cloud_ops_cloud(),
            supported_clouds: default_cloud_ops_supported_clouds(),
            iac_tools: default_cloud_ops_iac_tools(),
            cost_threshold_monthly_usd: default_cloud_ops_cost_threshold(),
            well_architected_frameworks: default_cloud_ops_waf(),
        }
    }
}

impl CloudOpsConfig {
    pub fn validate(&self) -> Result<()> {
        if self.enabled {
            if self.default_cloud.trim().is_empty() {
                anyhow::bail!(
                    "cloud_ops.default_cloud must not be empty when cloud_ops is enabled"
                );
            }
            if self.supported_clouds.is_empty() {
                anyhow::bail!(
                    "cloud_ops.supported_clouds must not be empty when cloud_ops is enabled"
                );
            }
            for (i, cloud) in self.supported_clouds.iter().enumerate() {
                if cloud.trim().is_empty() {
                    validation_bail!(
                        RequiredFieldEmpty,
                        format!("cloud_ops.supported_clouds[{i}]"),
                        "cloud_ops.supported_clouds[{i}] must not be empty"
                    );
                }
            }
            if !self.supported_clouds.contains(&self.default_cloud) {
                anyhow::bail!(
                    "cloud_ops.default_cloud '{}' is not in cloud_ops.supported_clouds {:?}",
                    self.default_cloud,
                    self.supported_clouds
                );
            }
            if self.cost_threshold_monthly_usd < 0.0 {
                anyhow::bail!(
                    "cloud_ops.cost_threshold_monthly_usd must be non-negative, got {}",
                    self.cost_threshold_monthly_usd
                );
            }
            if self.iac_tools.is_empty() {
                anyhow::bail!("cloud_ops.iac_tools must not be empty when cloud_ops is enabled");
            }
        }
        Ok(())
    }
}

fn default_cloud_ops_cloud() -> String {
    "aws".into()
}

fn default_cloud_ops_supported_clouds() -> Vec<String> {
    vec!["aws".into(), "azure".into(), "gcp".into()]
}

fn default_cloud_ops_iac_tools() -> Vec<String> {
    vec!["terraform".into()]
}

fn default_cloud_ops_cost_threshold() -> f64 {
    100.0
}

fn default_cloud_ops_waf() -> Vec<String> {
    vec!["aws-waf".into()]
}

