//! Top-level `Config` struct loaded from `config.toml`, plus the
//! directly-embedded scalar/enum sub-configs it references
//! (`OnboardStateConfig`, `WireApi`, `ToolResultImagePolicy`, `AuthMode`,
//! `CacheTtl`, `ModelProviderConfig`, and the `ModelEndpoint` /
//! `FamilyEndpoint` traits).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::{Config, ModelProviderConfig, ...}` path is unchanged.

use super::*;

/// Top-level OmnesAgent configuration, loaded from `config.toml`.
///
/// Resolution order: `OMNESAGENT_CONFIG_DIR` env → `OMNESAGENT_WORKSPACE` env → `~/.omnesagent/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct Config {
    /// Shared instance data directory (databases, hygiene state, cost
    /// records, daemon state files). Computed from `OMNESAGENT_CONFIG_DIR`
    /// / `OMNESAGENT_DATA_DIR` / `OMNESAGENT_WORKSPACE` (deprecated) at
    /// load time, not serialized. Per-agent identity + markdown lives
    /// at `agent_workspace_dir(&alias)`, not here.
    #[serde(skip)]
    pub data_dir: PathBuf,
    /// Path to config.toml - computed from home, not serialized
    #[serde(skip)]
    pub config_path: PathBuf,
    /// Dotted prop-paths overridden by `OMNESAGENT_*` env vars at load time.
    /// Populated by `apply_env_overrides`; consulted by `save()` to mask the
    /// env-injected values back to disk-or-default before encryption, and by
    /// `prop_is_env_overridden` for O(1) display-layer lookup (config list,
    /// dashboard, quickstart).
    #[serde(skip)]
    pub env_overridden_paths: std::collections::HashSet<String>,
    /// Per-path snapshot of pre-override raw values, captured at apply time
    /// from the post-`decrypt_secrets` in-memory state (so secret entries
    /// hold plaintext, not the display mask). `save()` restores from this
    /// map so env-injected values never reach disk and the operator's
    /// original on-disk credentials survive any save cycle.
    #[serde(skip)]
    pub pre_override_snapshots: std::collections::HashMap<String, String>,
    /// Per-path snapshot of `op://` external secret references captured before
    /// `decrypt_secrets()` resolves them for runtime use. `save()` restores
    /// these references unless the same path was intentionally edited.
    #[serde(skip)]
    pub onepassword_reference_snapshots: std::collections::HashMap<String, String>,
    /// Dotted prop-paths mutated since the last persist; drives the
    /// per-path PATCH applied by `save_dirty()`.
    #[serde(skip)]
    pub dirty_paths: std::collections::HashSet<String>,
    /// Security-critical sections the resilient loader reset to `Default`
    /// because the on-disk block was malformed. Non-empty = posture may be
    /// weaker than intended; exposure gating should refuse to trust the
    /// instance until repaired. Never serialized — a load-time signal.
    #[serde(skip)]
    pub degraded_security: Vec<String>,
    /// Non-security sections the resilient loader reset to `Default`
    /// because the on-disk block was malformed (e.g. `[plugins.entries]`
    /// written where `[[plugins.entries]]` was meant). Never serialized;
    /// a load-time signal the CLI surfaces on stderr so a silently-dropped
    /// section is impossible to miss.
    #[serde(skip)]
    pub degraded_sections: Vec<String>,
    /// Retired WATI config section roots detected before migration and typed
    /// deserialization erase them. Never serialized; the CLI surfaces each
    /// path on stderr so an operator cannot miss the retired channel.
    #[serde(skip)]
    pub retired_wati_config_sections: Vec<String>,
    /// Whether a retired `[node_transport]` section was present before
    /// migration and typed deserialization erased it. Never serialized; the
    /// CLI surfaces upgrade guidance without retaining the retired secret.
    #[serde(skip)]
    pub retired_node_transport_config: bool,
    /// Config file schema version.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,

    /// All configured provider profiles, grouped by category under a
    /// single `[providers]` root. Categories today: `models`, `tts`,
    /// `transcription`. Shape: `[providers.<category>.<type>.<alias>]`,
    /// e.g. `[providers.models.anthropic.default]`,
    /// `[providers.tts.openai.default]`,
    /// `[providers.transcription.groq.default]`.
    #[serde(default)]
    #[nested]
    pub providers: crate::providers::Providers,

    /// Model-routing rules — route `hint:<name>` to specific
    /// model_provider + model combos.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[nested]
    #[natural_key = "hint"]
    #[group = "Foundation"]
    pub model_routes: Vec<ModelRouteConfig>,

    /// Embedding-routing rules — route `hint:<name>` to specific
    /// model_provider + model combos for embedding requests.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[nested]
    #[natural_key = "hint"]
    #[group = "Foundation"]
    pub embedding_routes: Vec<EmbeddingRouteConfig>,

    /// Observability backend configuration (`[observability]`).
    #[serde(default)]
    #[nested]
    #[group = "Operations"]
    pub observability: ObservabilityConfig,

    /// Trust scoring and regression detection configuration (`[trust]`).
    #[serde(default)]
    #[nested]
    #[group = "Operations"]
    pub trust: crate::scattered_types::TrustConfig,

    /// Security subsystem configuration (`[security]`).
    #[serde(default)]
    #[nested]
    #[group = "Operations"]
    pub security: SecurityConfig,

    /// Backup tool configuration (`[backup]`).
    #[serde(default)]
    #[nested]
    #[group = "Operations"]
    pub backup: BackupConfig,

    /// Data retention and purge configuration (`[data_retention]`).
    #[serde(default)]
    #[nested]
    #[group = "Operations"]
    pub data_retention: DataRetentionConfig,

    /// Cloud transformation accelerator configuration (`[cloud_ops]`).
    #[serde(default)]
    #[nested]
    #[group = "Operations"]
    pub cloud_ops: CloudOpsConfig,

    /// Conversational AI agent builder configuration (`[conversational_ai]`).
    ///
    /// Experimental / future feature — not yet wired into the agent runtime.
    /// Omitted from generated config files when disabled (the default).
    /// Existing configs that already contain this section will continue to
    /// deserialize correctly thanks to `#[serde(default)]`.
    #[serde(default, skip_serializing_if = "ConversationalAiConfig::is_disabled")]
    #[nested]
    #[group = "Operations"]
    pub conversational_ai: ConversationalAiConfig,

    /// Managed cybersecurity service configuration (`[security_ops]`).
    #[serde(default)]
    #[nested]
    #[group = "Operations"]
    pub security_ops: SecurityOpsConfig,

    /// Runtime adapter configuration (`[runtime]`). Controls native vs Docker execution.
    #[serde(default)]
    #[nested]
    #[group = "Agent"]
    pub runtime: RuntimeConfig,

    /// Reliability settings: retries, backoff, key rotation (`[reliability]`).
    #[serde(default)]
    #[nested]
    #[group = "Agent"]
    pub reliability: ReliabilityConfig,

    /// Scheduler configuration for periodic task execution (`[scheduler]`).
    #[serde(default)]
    #[nested]
    #[group = "Agent"]
    pub scheduler: SchedulerConfig,

    /// Agent evaluation harness (`[eval]`) — surfaced via `omnesagent eval`.
    /// Distinct from `[agent.eval]`, which is the in-loop response-quality scorer.
    #[serde(default)]
    #[nested]
    #[group = "Operations"]
    pub eval: crate::scattered_types::EvalHarnessConfig,

    /// Pacing controls for slow/local LLM workloads (`[pacing]`).
    #[serde(default)]
    #[nested]
    #[group = "Agent"]
    pub pacing: PacingConfig,

    /// Skills loading and community repository behavior (`[skills]`).
    #[serde(default)]
    #[nested]
    pub skills: SkillsConfig,

    /// Pipeline tool configuration (`[pipeline]`).
    #[serde(default)]
    #[nested]
    #[group = "Agent"]
    pub pipeline: PipelineConfig,

    /// Automatic query classification — maps user messages to model hints.
    #[serde(default)]
    #[nested]
    #[group = "Agent"]
    pub query_classification: QueryClassificationConfig,

    /// Heartbeat configuration for periodic health pings (`[heartbeat]`).
    #[serde(default)]
    #[nested]
    #[group = "Agent"]
    pub heartbeat: HeartbeatConfig,

    /// Ralph autonomous development cycle orchestrator (`[ralph]`).
    #[serde(default)]
    #[nested]
    #[group = "Agent"]
    pub ralph: crate::ralph::RalphConfig,

    /// External knowledge bridge to ob2h (`[ob2h_bridge]`).
    #[serde(default)]
    #[nested]
    #[group = "Integrations"]
    pub ob2h_bridge: crate::ralph::Ob2hBridgeConfig,

    /// Declarative cron jobs (`[cron.<alias>]`), alias-keyed.
    ///
    /// Each entry is a named scheduled job synced into the database at
    /// scheduler startup. Subsystem runtime knobs (enable/disable, catch-up,
    /// run-history retention) live on `[scheduler]`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub cron: HashMap<String, CronJobDecl>,

    /// ACP (Agent Client Protocol) server configuration (`[acp]`).
    #[serde(default)]
    #[nested]
    #[group = "Integrations"]
    pub acp: AcpConfig,

    /// Channel configurations: Telegram, Discord, Slack, etc. (`[channels]`).
    #[serde(default, alias = "channels_config")]
    #[nested]
    pub channels: ChannelsConfig,

    /// Memory backend configuration: sqlite, markdown, embeddings (`[memory]`).
    #[serde(default)]
    #[nested]
    pub memory: MemoryConfig,

    /// Persistent storage model_provider configuration (`[storage]`).
    #[serde(default)]
    #[nested]
    pub storage: StorageConfig,

    /// Tunnel configuration for exposing the gateway publicly (`[tunnel]`).
    #[serde(default)]
    #[nested]
    pub tunnel: TunnelConfig,

    /// Gateway server configuration: host, port, pairing, rate limits (`[gateway]`).
    #[serde(default)]
    #[nested]
    #[group = "Network"]
    pub gateway: GatewayConfig,

    /// Inbound A2A discovery server (`[a2a.server]`). Default-closed:
    /// serves the well-known catalog card and per-alias agent cards only
    /// when `enabled = true`. See `crate::multi_agent::A2aServerConfig`.
    #[serde(default)]
    #[nested]
    #[group = "Network"]
    pub a2a: crate::multi_agent::A2aServerSection,

    /// WebSocket Secure (WSS) transport for remote TUI connections (`[wss]`).
    #[serde(default)]
    #[nested]
    pub wss: WssConfig,

    /// Nominated-relay client for reaching this daemon through a relay (`[relay]`).
    #[serde(default)]
    #[nested]
    #[group = "Network"]
    pub relay: RelayConfig,

    /// Certificate enrollment endpoint for certless clients (`[enroll]`).
    #[serde(default)]
    #[nested]
    #[group = "Network"]
    pub enroll: EnrollConfig,

    /// Composio managed OAuth tools integration (`[composio]`).
    #[serde(default)]
    #[nested]
    #[group = "Integrations"]
    pub composio: ComposioConfig,

    /// Microsoft 365 Graph API integration (`[microsoft365]`).
    #[serde(default)]
    #[nested]
    pub microsoft365: Microsoft365Config,

    /// Secrets encryption configuration (`[secrets]`).
    #[serde(default)]
    #[nested]
    #[group = "Storage"]
    pub secrets: SecretsConfig,

    /// Browser automation configuration (`[browser]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub browser: BrowserConfig,

    /// Browser delegation configuration (`[browser_delegate]`).
    ///
    /// Delegates browser-based tasks to a browser-capable CLI subprocess (e.g.
    /// Claude Code with `claude-in-chrome` MCP tools). Useful for interacting
    /// with corporate web apps (Teams, Outlook, Jira, Confluence) that lack
    /// direct API access. A persistent Chrome profile can be configured so SSO
    /// sessions survive across invocations.
    ///
    /// Fields:
    /// - `enabled` (`bool`, default `false`) — enable the browser delegation tool.
    /// - `cli_binary` (`String`, default `"claude"`) — CLI binary to spawn for browser tasks.
    /// - `chrome_profile_dir` (`String`, default `""`) — Chrome user-data directory for
    ///   persistent SSO sessions. When empty, a fresh profile is used each invocation.
    /// - `allowed_domains` (`Vec<String>`, default `[]`) — allowlist of domains the browser
    ///   may navigate to. Empty means all non-blocked domains are permitted.
    /// - `blocked_domains` (`Vec<String>`, default `[]`) — denylist of domains. Blocked
    ///   domains take precedence over allowed domains.
    /// - `task_timeout_secs` (`u64`, default `120`) — per-task timeout in seconds.
    ///
    /// Compatibility: additive and disabled by default; existing configs remain valid when omitted.
    /// Rollback/migration: remove `[browser_delegate]` or keep `enabled = false` to disable.
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub browser_delegate: crate::scattered_types::BrowserDelegateConfig,

    /// HTTP request tool configuration (`[http_request]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub http_request: HttpRequestConfig,

    /// Multimodal (image) handling configuration (`[multimodal]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub multimodal: MultimodalConfig,

    /// Automatic media understanding pipeline (`[media_pipeline]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub media_pipeline: MediaPipelineConfig,

    /// Web fetch tool configuration (`[web_fetch]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub web_fetch: WebFetchConfig,

    /// Link enricher configuration (`[link_enricher]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub link_enricher: LinkEnricherConfig,

    /// Text browser tool configuration (`[text_browser]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub text_browser: TextBrowserConfig,

    /// Web search tool configuration (`[web_search]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub web_search: WebSearchConfig,

    /// Project delivery intelligence configuration (`[project_intel]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub project_intel: ProjectIntelConfig,

    /// Google Workspace CLI (`gws`) tool configuration (`[google_workspace]`).
    #[serde(default)]
    #[nested]
    #[group = "Integrations"]
    pub google_workspace: GoogleWorkspaceConfig,

    /// Proxy configuration for outbound HTTP/HTTPS/SOCKS5 traffic (`[proxy]`).
    #[serde(default)]
    #[nested]
    #[group = "Network"]
    pub proxy: ProxyConfig,

    /// Cost tracking and budget enforcement configuration (`[cost]`).
    /// Also hosts the operator-managed rate sheet at
    /// `[cost.rates.<type>.<model>]`.
    #[serde(default)]
    #[nested]
    #[group = "Operations"]
    pub cost: CostConfig,

    /// Peripheral board configuration for hardware integration (`[peripherals]`).
    #[serde(default)]
    #[nested]
    #[group = "Operations"]
    pub peripherals: PeripheralsConfig,

    /// Delegate tool global default configuration (`[delegate]`).
    #[serde(default)]
    #[nested]
    #[group = "Multi-agent"]
    pub delegate: DelegateToolConfig,

    /// Aliased agents in this install. Each entry under `[agents.<alias>]`
    /// is one user-facing agent with its own identity, channels, model
    /// provider, risk profile, workspace, and memory scope.
    /// `DelegateTool` consults this map when one agent delegates a
    /// subtask to another.
    #[serde(default)]
    #[nested]
    pub agents: HashMap<String, AliasedAgentConfig>,

    /// Named risk/autonomy profiles (`[risk_profiles.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub risk_profiles: HashMap<String, RiskProfileConfig>,

    /// Named runtime/LLM execution profiles (`[runtime_profiles.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub runtime_profiles: HashMap<String, RuntimeProfileConfig>,

    /// Named skill bundles (`[skill_bundles.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub skill_bundles: HashMap<String, SkillBundleConfig>,

    /// Named knowledge bundles (`[knowledge_bundles.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub knowledge_bundles: HashMap<String, KnowledgeBundleConfig>,

    /// Named MCP server bundles (`[mcp_bundles.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub mcp_bundles: HashMap<String, McpBundleConfig>,

    /// Named peer groups (`[peer_groups.<name>]`). Each entry binds a
    /// channel, a list of member agents, and optional non-agent
    /// (external) members and a per-group blocklist. Mutual opt-in:
    /// two agents become peers only when both appear in the same
    /// group's `agents`. Empty by default for single-agent installs.
    /// See `crate::multi_agent::PeerGroupConfig`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub peer_groups: HashMap<String, crate::multi_agent::PeerGroupConfig>,

    /// Hooks configuration (lifecycle hooks and built-in hook toggles).
    #[serde(default)]
    #[nested]
    #[group = "Agent"]
    pub hooks: HooksConfig,

    /// Hardware configuration (wizard-driven physical world setup).
    #[serde(default)]
    #[nested]
    pub hardware: HardwareConfig,

    /// Voice transcription configuration (Whisper API via Groq).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub transcription: TranscriptionConfig,

    /// Voice input and offline dictation configuration (`[voice]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub voice: VoiceConfig,

    /// Text-to-Speech configuration (`[tts]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub tts: TtsConfig,

    /// External MCP server connections (`[mcp]`).
    #[serde(default, alias = "mcpServers")]
    #[nested]
    pub mcp: McpConfig,

    /// Dynamic node discovery configuration (`[nodes]`).
    #[serde(default)]
    #[nested]
    #[group = "Network"]
    pub nodes: NodesConfig,

    /// Meta-state for the Quickstart flow (which sections the user has
    /// already walked through). Not user-facing config (`[onboard_state]`).
    #[serde(default)]
    #[nested]
    pub onboard_state: OnboardStateConfig,

    /// Notion integration configuration (`[notion]`).
    #[serde(default)]
    #[nested]
    #[group = "Integrations"]
    pub notion: NotionConfig,

    /// Jira integration configuration (`[jira]`).
    #[serde(default)]
    #[nested]
    #[group = "Integrations"]
    pub jira: JiraConfig,

    /// Knowledge graph configuration (`[knowledge]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub knowledge: KnowledgeConfig,

    /// LinkedIn integration configuration (`[linkedin]`).
    #[serde(default)]
    #[nested]
    #[group = "Integrations"]
    pub linkedin: LinkedInConfig,

    /// Standalone image generation tool configuration (`[image_gen]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub image_gen: ImageGenConfig,

    /// Standalone file upload tool configuration (`[file_upload]`).
    #[serde(default)]
    #[nested]
    pub file_upload: FileUploadConfig,

    /// Standalone multi-file bundle upload tool configuration
    /// (`[file_upload_bundle]`).
    #[serde(default)]
    #[nested]
    pub file_upload_bundle: FileUploadBundleConfig,

    /// Standalone file download tool configuration (`[file_download]`).
    #[serde(default)]
    #[nested]
    pub file_download: FileDownloadConfig,

    /// Plugin system configuration (`[plugins]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub plugins: PluginsConfig,

    /// Locale for tool descriptions (e.g. `"en"`, `"zh-CN"`).
    ///
    /// When set, tool descriptions shown in system prompts are loaded from
    /// Fluent `.ftl` locale files. Falls back to embedded English, then to
    /// hardcoded descriptions.
    ///
    /// If omitted or empty, the locale is auto-detected from the host
    /// system's locale (defaulting to `"en"` if that can't be determined).
    #[serde(default)]
    pub locale: Option<String>,

    /// Verifiable Intent (VI) credential issuance and constraint checking
    /// (`[verifiable_intent]`). No credential chain verifier exists yet, so the
    /// `vi_verify` tool is not registered for the model and this section does not
    /// currently enable verification of anything.
    #[serde(default)]
    #[nested]
    #[group = "Agent"]
    pub verifiable_intent: VerifiableIntentConfig,

    /// Claude Code tool configuration (`[claude_code]`).
    #[serde(default)]
    #[nested]
    #[group = "Integrations"]
    pub claude_code: ClaudeCodeConfig,

    /// Claude Code task runner with Slack progress and SSH session handoff (`[claude_code_runner]`).
    #[serde(default)]
    #[nested]
    #[group = "Integrations"]
    pub claude_code_runner: ClaudeCodeRunnerConfig,

    /// Codex CLI tool configuration (`[codex_cli]`).
    #[serde(default)]
    #[nested]
    #[group = "Integrations"]
    pub codex_cli: CodexCliConfig,

    /// Gemini CLI tool configuration (`[gemini_cli]`).
    #[serde(default)]
    #[nested]
    #[group = "Integrations"]
    pub gemini_cli: GeminiCliConfig,

    /// OpenCode CLI tool configuration (`[opencode_cli]`).
    #[serde(default)]
    #[nested]
    #[group = "Integrations"]
    pub opencode_cli: OpenCodeCliConfig,

    /// Standard Operating Procedures engine configuration (`[sop]`).
    #[serde(default)]
    #[nested]
    #[group = "Agent"]
    pub sop: SopConfig,

    /// Shell tool configuration (`[shell_tool]`).
    #[serde(default)]
    #[nested]
    #[group = "Tools"]
    pub shell_tool: ShellToolConfig,

    /// Escalation routing configuration (`[escalation]`).
    #[serde(default)]
    #[nested]
    pub escalation: EscalationConfig,
}

/// Multi-client workspace isolation configuration.
///
/// When enabled, each client engagement gets an isolated workspace with
/// separate memory, audit, secrets, and tool restrictions.
#[allow(clippy::struct_excessive_bools)]
/// Opaque state the Quickstart flow writes so it can tell, on a
/// re-run, which sections the user has already walked through at least
/// once — which lets it offer "Reconfigure? [y/N]" skip gates instead of
/// forcing users through every field again.
///
/// This is meta-state about the Quickstart flow, not user-facing config.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "onboard_state"]
pub struct OnboardStateConfig {
    /// Section keys the user has completed at least once.
    /// Values are the lowercased Section variant names
    /// (`"workspace"`, `"model_providers"`, …).
    #[serde(default)]
    pub completed_sections: Vec<String>,
    /// `true` once the Quickstart has applied a `BuilderSubmission`
    /// successfully on this install. Web gateway and TUI auto-launch
    /// the Quickstart on startup iff this is `false` **and** no
    /// `agents.*` entries exist (the implicit-completion rule covers
    /// upgrades). The flag is flipped in the same atomic write that
    /// lands the Quickstart submission; re-entering the Quickstart
    /// later to add another agent does not flip it back to `false`.
    #[serde(default)]
    pub quickstart_completed: bool,
}

/// Used by `#[serde(skip_serializing_if)]` on plain `bool` fields to omit
/// them from TOML output when they carry their struct-level default (`false`).
/// Keeps fresh model_provider entries clean — a default-constructed
/// `ModelProviderConfig` for one model_provider family shouldn't write flag fields
/// that only apply to a different family.
pub(crate) fn is_false(value: &bool) -> bool {
    !*value
}

/// One trait per family-endpoint enum. Returns the URI template for the chosen
/// variant — a literal URL for fixed endpoints (`https://api.openai.com/v1`),
/// or a substitution template for computed endpoints (Azure's
/// `https://{resource}.openai.azure.com/...`). Substitution happens family-side
/// in the runtime constructor; for non-templated families the return value is
/// the final URL.
///
/// Resolution order at runtime is uniform across every model model_provider family:
/// operator's `cfg.uri` first; family endpoint enum's `uri()` second; loud
/// failure when neither is set.
pub trait ModelEndpoint {
    fn uri(&self) -> &'static str;
}

/// Implemented by every `*ModelProviderConfig`. Multi-region families
/// override to return `Some(self.endpoint.uri())`; single-endpoint families
/// inherit the `None` default. Drives `ModelProviders::resolved_endpoint_uri`,
/// which is itself driven by the `for_each_model_provider_slot!` macro — so
/// adding a new family without an impl is a compile error.
pub trait FamilyEndpoint {
    fn endpoint_uri(&self) -> Option<&'static str> {
        None
    }
}

/// Wire protocol flavor for the model_provider client. `responses` routes
/// through OpenAI's Responses API (`POST /v1/responses`);
/// `chat_completions` routes through the legacy `/v1/chat/completions` (or
/// the family's chat-completions-compatible endpoint). New OpenAI provider
/// slots default to `responses`; other families default to chat-completions
/// (or ignore the field) when unset.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum WireApi {
    Responses,
    ChatCompletions,
}

impl WireApi {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Responses => "responses",
            Self::ChatCompletions => "chat_completions",
        }
    }
}

/// Policy for image markers embedded in native tool-result content.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ToolResultImagePolicy {
    /// Preserve tool-result image markers as structured `image_url` parts.
    #[default]
    ImageUrl,
    /// Remove tool-result image payloads and leave a fixed notice for the model.
    Omit,
}

fn is_default_tool_result_image_policy(value: &ToolResultImagePolicy) -> bool {
    *value == ToolResultImagePolicy::default()
}

/// Authentication mode for model model_provider families that support more than one
/// (e.g. Qwen, Minimax can use API key OR OAuth). Families that only support a
/// single auth flow simply omit this field from their config struct.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    /// Standard API key authentication via the `api_key` field.
    #[default]
    ApiKey,
    /// OAuth flow — credential resolution defers to the family runtime impl
    /// (typically reading a vendor-specific token cache or env var).
    OAuth,
}

/// Prompt-cache entry lifetime to request for this provider's Anthropic
/// cache markers. `"5m"` is the API default; `"1h"` extends the cache
/// entry lifetime to one hour so a pause longer than five minutes does
/// not force a full-price rewrite of the cached prefix. Meaningful only
/// where Anthropic-shaped `cache_control` markers reach the API: the
/// native Anthropic provider always places them, compatible providers
/// only behind `cache_passthrough` (with passthrough off the setting is
/// inert). One TTL applies to every marker this implementation places;
/// operator-supplied `cache_control` (via `provider_extra` or raw tool
/// JSON) sits outside that guarantee and must order 1h before 5m when
/// mixing lifetimes in one request.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub enum CacheTtl {
    /// Standard 5-minute cache lifetime (the API default).
    #[default]
    #[serde(rename = "5m")]
    FiveMinutes,
    /// 1-hour cache lifetime; cache writes bill at a premium write rate.
    #[serde(rename = "1h")]
    OneHour,
}

/// Named model_provider profile definition.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models"]
pub struct ModelProviderConfig {
    /// Secret API token for this model_provider. Grab it from the model_provider's dashboard (OpenAI platform, Anthropic console, OpenRouter keys page, etc.). Stored via the OS keyring when possible; never commit it to config.toml directly.
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Provider implementation to instantiate for this profile. Use this
    /// when a canonical typed slot should run through a compatible
    /// implementation, e.g. `[providers.models.openai.proxy] kind =
    /// "openai-compatible"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Endpoint URI the client hits. Override the family's default endpoint when pointing at a self-hosted gateway (LiteLLM, vLLM, Ollama), a custom proxy, or any non-standard URL. Leave unset to use the family's default URI from its `ModelEndpoint` impl. Set this to the FULL endpoint URL; there is no separate path-suffix field.
    #[tab(Connection)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Model identifier to send with each request: the ID string from the model_provider's catalog (e.g. `gpt-4o`, `claude-sonnet-4-5`, `llama-3.3-70b`). Must match a model the model_provider actually serves on this account.
    #[tab(Model)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Ordered list of other provider aliases to try when every model on this
    /// alias has failed. Each entry is a dotted `<type>.<alias>` reference into
    /// `providers.models` and resolves with its own credentials, endpoint, and
    /// model. A fallback never inherits this alias's key. The walk is
    /// depth-first: this alias's models are exhausted first, then each fallback
    /// alias is descended in turn (applying its own `fallback_models` and
    /// `fallback`). Empty means no provider-level fallback.
    #[tab(Model)]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallback: Vec<crate::providers::ModelProviderRef>,
    /// Ordered alternate models to try on THIS provider before falling over to
    /// the `fallback` aliases. Same endpoint, key, and headers as the primary
    /// `model`. Only the model identifier changes. Use this when a provider
    /// serves a backup model (e.g. a smaller or older variant) that should be
    /// tried before leaving the provider entirely. Empty means only `model` is
    /// tried.
    #[tab(Model)]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallback_models: Vec<String>,
    /// Sampling temperature passed to the model. Lower values (0.0–0.3) give
    /// deterministic, near-verbatim output, which fits code, routing, summarization.
    /// Higher values (0.7–1.2) give more varied output, which fits open-ended chat.
    #[tab(Model)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// HTTP request timeout in seconds. Bump this for slow local model_providers (Ollama on CPU, big local models) or high-latency networks; leave unset otherwise. When set above 300 it also raises the provider's streaming idle bound (default 300 s, the maximum gap between stream reads) on OpenAI-compatible and OpenAI Responses providers.
    #[tab(Model)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    /// Extra HTTP headers sent with every request. Niche: used for auth bridges, corporate proxies, or custom gateways that demand a tracing header. Most users never touch this; edit `config.toml` directly if you need it.
    #[tab(Connection)]
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub extra_headers: HashMap<String, String>,
    /// Wire protocol flavor: `responses` for OpenAI's Responses API (`POST /v1/responses`), `chat_completions` for the legacy chat wire and most OpenAI-compatible gateways. New OpenAI provider slots default to `responses`; other families default to chat-completions (or ignore the field). Only override if you're forcing an unusual combination.
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wire_api: Option<WireApi>,
    /// When true, the client pulls credentials from OmnesAgent's stored `openai-codex` auth profile instead of the `api_key` field above. Import an existing Codex CLI login with `omnesagent auth login --model-provider openai-codex --import ~/.codex/auth.json`, or run `omnesagent auth login --model-provider openai-codex`. Turn on only for the OpenAI Codex model_provider; leave off for standard API-key model_providers.
    #[tab(Connection)]
    #[serde(default, skip_serializing_if = "is_false")]
    #[credential_class = "external_auth_store"]
    pub requires_openai_auth: bool,
    /// Hard cap on response length in tokens. Most models enforce sensible built-in limits already; leave unset unless you specifically need to clip long outputs for cost or latency reasons.
    #[tab(Model)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// ModelProvider-specific quirk: fold the system prompt into the first user message instead of sending a separate system role. Only needed for models that reject (or mishandle) a standalone system role, e.g. certain older Mistral variants.
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "is_false")]
    pub merge_system_into_user: bool,
    /// Extra JSON parameters to include in API requests.
    /// Merged at the top level of the request body, allowing provider-specific
    /// features (routing, transforms, etc.) without code changes.
    /// Example: `provider_extra = { model_provider = { only = ["Anthropic"] } }`
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_extra: Option<serde_json::Value>,
    /// Per-model pricing for cost tracking, USD per 1M tokens.
    ///
    /// Free-form key/value map. Keys are user-defined model identifiers; an
    /// optional `.input` / `.output` suffix encodes pricing dimension when
    /// the operator wants to split rates. A bare key without a suffix is
    /// used as a flat per-token rate when neither dimension is specified.
    /// Default is empty: cost tracking falls back to "unknown" rates and
    /// only token usage is recorded.
    ///
    /// Example: `pricing = { opus = 15.0, sonnet = 3.0 }`
    /// Or split: `pricing = { "opus.input" = 15.0, "opus.output" = 75.0 }`
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub pricing: HashMap<String, f64>,
    /// Whether stored assistant reasoning should be replayed on outbound
    /// assistant history messages. `Some(false)` strips `reasoning_content`
    /// and `reasoning` before sending. `None` (default) honours the provider's
    /// built-in default (true for most compat providers, false for Groq).
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_assistant_reasoning: Option<bool>,
    /// Forward Anthropic prompt caching through this OpenAI-compatible
    /// provider. When true, request bodies on the structured paths (agent
    /// turns, tool calls, structured streaming) gain an Anthropic-shaped
    /// `cache_control` breakpoint on the system prompt and on the last
    /// message once the conversation has more than one non-system message,
    /// mirroring the native Anthropic provider's placement strategy, and
    /// gateway-reported cache usage populates the cached-token counters.
    /// With `merge_system_into_user`, the merged first user message carries
    /// the system breakpoint instead. The text-only helpers (`chat_with_system`,
    /// `chat_with_history`, the legacy chunk-stream APIs) deliberately emit
    /// no breakpoints: their responses drop usage, so a premium cache write
    /// they triggered could never be accounted for.
    /// Only gateways that translate between OpenAI Chat Completions and the
    /// Anthropic Messages API forward these breakpoints (e.g. LiteLLM).
    /// Default `false`: request bodies and response handling are unchanged.
    ///
    /// Before relying on it, verify the configured route serves cache reads:
    /// an immediate repeat of a cache-creating request must report
    /// `cache_read_input_tokens > 0`. Some gateway routes accept and bill
    /// cache writes without ever serving reads.
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "is_false")]
    pub cache_passthrough: bool,
    /// Cache entry lifetime requested for this provider's Anthropic
    /// prompt-cache markers. `"5m"` (default) keeps the standard
    /// 5-minute lifetime; `"1h"` requests a 1-hour lifetime so a pause
    /// longer than five minutes does not force a full-price rewrite of
    /// the cached prefix. The 1h lifetime bills cache writes at a
    /// premium write rate (nominal planning figure: twice the input
    /// price), so it pays off only when turns regularly resume more
    /// than five minutes after the last request.
    ///
    /// The native Anthropic provider applies this to every cache marker
    /// it places in a request. Compatible providers apply it only when
    /// `cache_passthrough` is enabled; without passthrough no markers
    /// are placed at all and this field is inert (no parse-time warning:
    /// an operator may stage the key before switching passthrough on).
    /// Providers that emit their own cache markers by other means
    /// (openrouter) ignore this setting.
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_ttl: Option<CacheTtl>,
    /// Pull live token prices for this provider's models from its own
    /// OpenAI-compatible `/models` listing (the gateway is the source of truth
    /// for its prices), filling cost-tracking rates for models the operator
    /// has NOT priced under `[cost.rates]` / `pricing`. Models the gateway does
    /// not price (or providers with no HTTP `/models` listing at all, such as
    /// a subprocess gateway like `kilocli`) fall back to the public models.dev
    /// catalog. Configured rates always win; live prices only fill gaps. A
    /// background task refreshes the price snapshot hourly; the cost-recording
    /// path reads the cached snapshot and never blocks on the network. Default
    /// `false`: off means no fetching and behavior identical to a build
    /// without the feature.
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "is_false")]
    pub live_pricing: bool,
    /// Override the provider's default for native tool calling.
    /// `None` (default) honors the provider's built-in choice. `Some(true)`
    /// forces native tool calls on, `Some(false)` forces text-fallback.
    /// For OpenAI-compatible chat-completions families, native calls avoid
    /// injecting the prompt-guided tool-use protocol into the system prompt.
    /// Set this only after confirming the configured endpoint and model
    /// accept the native tool schema. Providers routed through
    /// `wire_api = "responses"` are inherently native and ignore this setting.
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_tools: Option<bool>,
    /// Enable or disable chain-of-thought thinking for models that support it
    /// (e.g. Qwen3, GLM-4). `true` turns thinking on, `false` turns it off.
    /// `None` (default) lets the model decide. Forwarded as `enable_thinking`
    /// in the request body; mirrors the Ollama provider's `think` field.
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub think: Option<bool>,
    /// Override the provider's vision (image input) capability.
    /// `None` (default) uses the provider family's built-in default. Several
    /// families (llama.cpp, the generic OpenAI-compatible endpoint, etc.)
    /// assume vision-capable because they can serve multimodal models. Set
    /// `vision = false` for a text-only model served by such a family (e.g. a
    /// text LLM behind llama.cpp) so image messages are routed to a configured
    /// `[multimodal] vision_model_provider` instead of being sent to a model
    /// that rejects them. `Some(true)` forces vision on.
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vision: Option<bool>,
    /// How native compatible chat-completions providers handle image markers in
    /// role=`tool` results. `image_url` preserves structured image parts;
    /// `omit` removes their payloads and appends a fixed notice. This does not
    /// affect direct user image content or OpenAI Responses providers.
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "is_default_tool_result_image_policy")]
    pub tool_result_image_policy: ToolResultImagePolicy,
    /// Arbitrary key/value pairs forwarded verbatim as a top-level
    /// `chat_template_kwargs` object in the request body of OpenAI-compatible
    /// providers. Consumed by chat-template-aware backends such as vLLM,
    /// SGLang, and llama.cpp to pass model-family template variables that
    /// control behaviour not exposed by other fields. Must be a JSON object
    /// (TOML inline table); non-object values are ignored with a warning.
    /// Example (Qwen3 thinking suppression):
    ///   `chat_template_kwargs = { enable_thinking = false }`
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_template_kwargs: Option<serde_json::Value>,
    /// Context window size (max input tokens) for this model.
    /// Auto-populated on setup from provider's /models endpoint if available.
    /// Override manually for custom endpoints or when auto-detection fails.
    #[tab(Advanced)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window: Option<usize>,
    /// Path to a PEM-encoded CA certificate for TLS connections to this provider.
    /// Must be an absolute path; shell expansion (e.g. `~`) is not performed.
    /// Leave unset to use the system's default trust store.
    #[tab(Connection)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_ca_cert_path: Option<String>,
}

