//! Aliased agents and delegation: `AliasedAgentConfig`, delegate targets and
//! execution modes, `ResolvedRuntime`, `ChannelAliasInfo`, the agent/channel/
//! delegation/storage resolution inherent `impl Config` blocks (`ActiveStorage`
//! included), the shared config default/lenient-deserialize helpers, the
//! `Config::validate` helper fns, and the runtime proxy client helpers.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::{AliasedAgentConfig, ...}` path is unchanged.

use super::*;

/// Default fraction of `max_history_messages` that a whole-turn history trim
/// drops the history to. Strictly below 1.0 so a trim leaves headroom and
/// the next turns do not immediately trigger another trim.
pub const DEFAULT_HISTORY_TRIM_LOW_WATER: f32 = 0.7;

/// Runtime tunables resolved from the agent's runtime profile. Populated
/// by `Config::resolved_agent_config`; never deserialized from the agent
/// table. The runtime profile is the sole config surface for these.
#[derive(Debug, Clone)]
pub struct ResolvedRuntime {
    pub compact_context: bool,
    pub max_tool_iterations: usize,
    pub max_history_messages: usize,
    /// Fraction of `max_history_messages` a whole-turn trim drops to
    /// (hysteresis low-water mark; 1.0 disables hysteresis).
    pub history_trim_low_water: f32,
    /// Token budget for preemptive context/history trimming (from runtime profile).
    /// NOT the provider `max_tokens` output limit.
    pub max_context_tokens: usize,
    /// Model's context window (max input tokens) — from provider config.
    pub model_context_window: usize,
    pub parallel_tools: bool,
    pub tool_dispatcher: String,
    pub strict_tool_parsing: bool,
    pub tool_call_dedup_exempt: Vec<String>,
    pub tool_filter_groups: Vec<ToolFilterGroup>,
    pub max_system_prompt_chars: usize,
    pub thinking: crate::scattered_types::ThinkingConfig,
    pub history_pruning: crate::scattered_types::HistoryPrunerConfig,
    pub eval: crate::scattered_types::EvalConfig,
    pub auto_classify: Option<crate::scattered_types::AutoClassifyConfig>,
    pub context_compression: crate::scattered_types::ContextCompressionConfig,
    pub max_tool_result_chars: usize,
    pub keep_tool_context_turns: usize,
    pub tool_receipts: ToolReceiptsConfig,
    pub prompt_injection_mode: SkillsPromptInjectionMode,
}

impl ResolvedRuntime {
    /// Effective token budget for preemptive whole-turn history trimming.
    /// When `history_pruning.enabled` is set, an explicit `max_tokens` floor
    /// trims earlier than the hard context ceiling; otherwise the ceiling is
    /// the only trigger. Reuses the existing `history_pruning.*` idents.
    pub fn effective_context_budget(&self) -> usize {
        if self.history_pruning.enabled && self.history_pruning.max_tokens > 0 {
            self.max_context_tokens.min(self.history_pruning.max_tokens)
        } else {
            self.max_context_tokens
        }
    }
}

impl Default for ResolvedRuntime {
    fn default() -> Self {
        Self {
            compact_context: true,
            max_tool_iterations: 10,
            max_history_messages: 50,
            history_trim_low_water: DEFAULT_HISTORY_TRIM_LOW_WATER,
            max_context_tokens: 32_000,
            model_context_window: 32_000,
            parallel_tools: false,
            tool_dispatcher: default_agent_tool_dispatcher(),
            strict_tool_parsing: false,
            tool_call_dedup_exempt: Vec::new(),
            tool_filter_groups: Vec::new(),
            max_system_prompt_chars: default_max_system_prompt_chars(),
            thinking: crate::scattered_types::ThinkingConfig::default(),
            history_pruning: crate::scattered_types::HistoryPrunerConfig::default(),
            eval: crate::scattered_types::EvalConfig::default(),
            auto_classify: None,
            context_compression: crate::scattered_types::ContextCompressionConfig::default(),
            max_tool_result_chars: default_max_tool_result_chars(),
            keep_tool_context_turns: default_keep_tool_context_turns(),
            tool_receipts: ToolReceiptsConfig::default(),
            prompt_injection_mode: SkillsPromptInjectionMode::default(),
        }
    }
}

/// Execution semantics for one explicit delegate target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum DelegateExecutionMode {
    /// Parent-bounded delegation: the target is reachable through the caller,
    /// shares caller budgets, and agentic tools are capped by the caller's
    /// tool envelope.
    #[default]
    Bounded,
    /// Independent delegation: the target runs under its own configured
    /// policy/tool envelope, like opening a new chat with that agent.
    Independent,
}

/// One explicit delegate target listed under `[agents.<alias>].delegates`.
///
/// String entries deserialize as `{ agent = "<string>", mode = "bounded" }`
/// for concise manual editing. Serialization always emits the object form so
/// the config surface is a pure object array.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Configurable, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "delegate_target"]
pub struct DelegateTargetConfig {
    /// Target agent alias.
    pub agent: String,
    /// Whether this target is parent-bounded or independent.
    #[serde(default)]
    pub mode: DelegateExecutionMode,
}

impl DelegateTargetConfig {
    /// Construct the legacy-equivalent target shape used for string entries.
    ///
    /// Most tests and migration paths call this instead of spelling out
    /// `mode = Bounded`, making it explicit that a bare delegate alias never
    /// opts into independent execution.
    #[must_use]
    pub fn bounded(agent: impl Into<String>) -> Self {
        Self {
            agent: agent.into(),
            mode: DelegateExecutionMode::Bounded,
        }
    }

    /// Return the raw configured alias.
    ///
    /// Callers that compare aliases should trim at the comparison boundary so
    /// diagnostics can still point at the stored value when validation fails.
    #[must_use]
    pub fn agent(&self) -> &str {
        self.agent.as_str()
    }

    /// Return the configured execution mode for this explicit target.
    #[must_use]
    pub fn mode(&self) -> DelegateExecutionMode {
        self.mode
    }
}

impl<'de> Deserialize<'de> for DelegateTargetConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DelegateTargetVisitor;

        impl<'de> Visitor<'de> for DelegateTargetVisitor {
            type Value = DelegateTargetConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an agent alias string or a delegate target object")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(DelegateTargetConfig::bounded(value))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(DelegateTargetConfig::bounded(value))
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                // Keep the object arm strict so misspelled mode/agent fields do
                // not silently round-trip as bounded targets. String entries
                // already provide the forgiving legacy shape.
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct DelegateTargetObject {
                    agent: String,
                    #[serde(default)]
                    mode: DelegateExecutionMode,
                }

                let target =
                    DelegateTargetObject::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(DelegateTargetConfig {
                    agent: target.agent,
                    mode: target.mode,
                })
            }
        }

        deserializer.deserialize_any(DelegateTargetVisitor)
    }
}

/// Configuration for an aliased agent. Each `[agents.<alias>]` TOML
/// block deserializes into one of these. The `DelegateTool` looks up
/// entries here to dispatch a subtask to a named sibling agent.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "delegate_agent"]
pub struct AliasedAgentConfig {
    /// Whether this agent is active. Set false to disable without removing the definition.
    #[tab(General)]
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Channel aliases this agent handles (e.g. `["telegram.<alias>", "discord.<alias>"]`).
    /// Each entry is a `ChannelRef` resolving through `[channels.<type>.<alias>]`;
    /// `Config::validate()` fails loud on dangling references.
    #[tab(Channels)]
    #[serde(default)]
    pub channels: Vec<crate::providers::ChannelRef>,
    /// Dotted model-provider alias (e.g. `"anthropic.<alias>"`).
    /// Resolves through `model_providers.<type>.<alias>` at runtime;
    /// `Config::validate()` fails loud on dangling references.
    #[tab(Providers)]
    #[serde(default)]
    pub model_provider: crate::providers::ModelProviderRef,
    /// Risk profile alias (e.g. `"default"`). Resolves delegation guardrails at runtime.
    #[tab(General)]
    #[serde(default)]
    pub risk_profile: crate::providers::RiskProfileRef,
    /// Runtime profile alias (e.g. `"default"`). Resolves agentic/iteration settings.
    #[tab(General)]
    #[serde(default)]
    pub runtime_profile: crate::providers::RuntimeProfileRef,
    /// Skill bundle aliases. Each entry resolves to
    /// `skill_bundles[key].directory` at runtime; the agent loads every
    /// listed bundle.
    #[tab(Bundles)]
    #[serde(default)]
    pub skill_bundles: Vec<String>,
    /// Knowledge bundle aliases. Additive: the agent loads every listed
    /// bundle.
    #[tab(Bundles)]
    #[serde(default)]
    pub knowledge_bundles: Vec<String>,
    /// MCP bundle aliases. Each entry references `mcp_bundles[key]`, a named
    /// group of MCP servers. Secure by default: an agent is granted only the
    /// servers named by its bundles. An agent with no `mcp_bundles` receives
    /// no MCP servers (omission is not a grant). See
    /// `Config::mcp_servers_for_agent`.
    #[tab(Bundles)]
    #[serde(default)]
    pub mcp_bundles: Vec<String>,
    /// Initialize this agent's `mcp_bundles` tools when it serves an ACP
    /// (`session/new`) session.
    ///
    /// Off by default: MCP servers are external processes/services that can
    /// block startup while they connect, and ACP `session/new` is expected to
    /// return promptly. Enable it when this agent must call its `mcp_bundles`
    /// tools over ACP; `session/new` then pays the one-time MCP connection cost
    /// (bounded and non-fatal per server). Set per agent so each ACP profile
    /// opts in independently; when this agent is the ACP default
    /// (`acp.default_agent`, or the sole configured agent), the flag is picked
    /// up automatically for sessions that omit `agentAlias`.
    #[tab(Bundles)]
    #[serde(default)]
    pub acp_enable_mcp: bool,
    /// Cron job aliases. Each entry references `cron[key]`, a declarative
    /// scheduled job invoked by the scheduler on its configured trigger.
    /// When the cron fires, this agent is the actor that executes the job.
    #[tab(Cron)]
    #[serde(default)]
    pub cron_jobs: Vec<String>,
    /// TTS provider as a dotted alias reference (`<type>.<alias>`,
    /// e.g. `"openai.<alias>"`). Resolves through `tts_providers.<type>.<alias>`.
    /// Empty = no TTS for this agent (there is no global default-provider concept;
    /// every agent that wants TTS sets its own `tts_provider`).
    #[tab(Providers)]
    #[serde(default)]
    pub tts_provider: crate::providers::TtsProviderRef,
    /// Transcription / STT provider as a dotted alias reference
    /// (`<type>.<alias>`, e.g. `"groq.<alias>"`). Resolves through
    /// `transcription_providers.<type>.<alias>`. Empty = agent has no
    /// transcription preference; channels that ingest voice still need a
    /// resolved provider (there is no global default), so an inbound voice
    /// flow into an agent with empty `transcription_provider` errors loudly
    /// at the channel boundary.
    #[tab(Providers)]
    #[serde(default)]
    pub transcription_provider: crate::providers::TranscriptionProviderRef,

    /// Optional override for the per-message LLM reply-intent classifier
    /// (`classify_channel_reply_intent` in omnesagent-channels). When non-empty,
    /// the channel orchestrator routes the "should this message be replied to?"
    /// classification call to `[providers.models.<type>.<alias>]` referenced
    /// here, instead of reusing the main agent's `model_provider`.
    ///
    /// Source of truth for api_key / uri / model / temperature etc. is the
    /// referenced `[providers.models.<type>.<alias>]` entry. This field is
    /// a reference only (NEVER a copy), per AGENTS.md SINGLE SOURCE OF TRUTH.
    ///
    /// Empty (`Default`) = inherit the main agent's resolved provider+model
    /// (preserves pre-PR behavior; backward compatible).
    ///
    /// Use case: classification is a cheap REPLY/NO_REPLY decision, doesn't
    /// need a high-end model. Point this at a fast/free small model
    /// (e.g. `kimi-k2.5`, `qwen-turbo`) while `model_provider` stays on the
    /// expensive answering model (e.g. `qwen3.6-plus`).
    ///
    /// Note: TOML table names cannot contain `.`, so alias `kimi-k2.5`
    /// must be written as `[providers.models.custom.kimi-k2-5]`. The
    /// underlying `model = "kimi-k2.5"` string can still contain dots.
    ///
    /// ACP channels (IDE-direct) always reply and skip the classifier
    /// entirely, so this field has no effect on ACP traffic.
    #[tab(Providers)]
    #[serde(default)]
    pub classifier_provider: crate::providers::ModelProviderRef,

    /// Per-agent reply-intent precheck controls. The classifier call reads
    /// this block at message time; model/provider selection stays on
    /// `classifier_provider` so there is only one routing source of truth.
    #[tab(Channels)]
    #[serde(default)]
    #[nested]
    pub precheck: crate::scattered_types::ChannelPrecheckConfig,

    /// Per-agent override for the context-compression summarizer provider, as
    /// a `providers.models.<type>.<alias>` reference. Empty (Default) = inherit
    /// the runtime profile's `context_compression.summary_provider`, else the
    /// agent's own resolved provider+model. Reference only, never a copy;
    /// resolved by [`Config::effective_summary_provider`]. Validated in
    /// `Config::validate()`.
    #[tab(Providers)]
    #[serde(default)]
    pub summary_provider: crate::providers::ModelProviderRef,

    /// Auto-allow delegation to every agent sharing this agent's risk
    /// profile. Default `true` preserves the historical reach where any
    /// same-profile peer is a delegation target. Set `false` to opt this
    /// agent out so only the explicit `delegates` list is reachable.
    /// Gating (whether delegation is permitted at all) still lives on the
    /// risk profile's `delegation_policy.mode`; this only narrows reach.
    #[tab(General)]
    #[serde(default = "default_true")]
    pub delegate_same_risk_profile: bool,

    /// Explicit delegate roster: additional agent aliases this agent may
    /// delegate to, beyond same-profile peers. Possibly empty. String
    /// entries are accepted for concise manual editing and load as bounded
    /// delegates; saved config emits object entries with explicit modes.
    /// Entries may name agents on a different risk profile. `Config::validate()`
    /// fails loud on a dangling alias, duplicate alias, or self-reference.
    #[tab(General)]
    #[serde(default)]
    pub delegates: Vec<DelegateTargetConfig>,

    // ── Resolved runtime tunables (populated by `resolved_agent_config`
    // from the runtime profile; not config-settable on the agent). ──
    #[serde(skip)]
    pub resolved: ResolvedRuntime,

    /// Per-agent workspace block (`[agents.<alias>.workspace]`).
    /// Holds the agent's filesystem path, cross-agent access allowlist,
    /// filesystem-escape boolean, and cross-agent memory allowlist.
    /// Default is fully jailed (no cross-agent access). See
    /// `crate::multi_agent::AgentWorkspaceConfig`.
    #[tab(Workspace)]
    #[serde(default)]
    #[nested]
    pub workspace: crate::multi_agent::AgentWorkspaceConfig,

    /// Per-agent memory backend selection (`[agents.<alias>.memory]`).
    /// The `backend` field is locked at agent creation and immutable on
    /// subsequent loads. Defaults to `Sqlite`. See
    /// `crate::multi_agent::AgentMemoryConfig`.
    #[tab(Memory)]
    #[serde(default)]
    #[nested]
    pub memory: crate::multi_agent::AgentMemoryConfig,

    /// Per-agent identity format (`[agents.<alias>.identity]`). Each
    /// agent renders its own IDENTITY.md / SOUL.md inside its
    /// per-agent workspace; this block selects the format (OpenClaw or
    /// AIEOS) and optional inline/file source for the agent's identity
    /// document.
    #[tab(Tuning)]
    #[serde(default)]
    #[nested]
    pub identity: IdentityConfig,

    /// Per-agent A2A publication block (`[agents.<alias>.a2a]`). Gates
    /// whether this alias is discoverable as a spec-conforming A2A agent
    /// and which resolved skills appear on its card. Default-closed
    /// (`published = false`, no exposed skills). See
    /// `crate::multi_agent::AgentA2aConfig`.
    #[tab(General)]
    #[serde(default)]
    #[nested]
    pub a2a: crate::multi_agent::AgentA2aConfig,
}

impl Default for AliasedAgentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            channels: Vec::new(),
            model_provider: crate::providers::ModelProviderRef::default(),
            risk_profile: crate::providers::RiskProfileRef::default(),
            runtime_profile: crate::providers::RuntimeProfileRef::default(),
            skill_bundles: Vec::new(),
            knowledge_bundles: Vec::new(),
            mcp_bundles: Vec::new(),
            acp_enable_mcp: false,
            cron_jobs: Vec::new(),
            tts_provider: crate::providers::TtsProviderRef::default(),
            transcription_provider: crate::providers::TranscriptionProviderRef::default(),
            classifier_provider: crate::providers::ModelProviderRef::default(),
            precheck: crate::scattered_types::ChannelPrecheckConfig::default(),
            summary_provider: crate::providers::ModelProviderRef::default(),
            delegate_same_risk_profile: true,
            delegates: Vec::new(),
            resolved: ResolvedRuntime::default(),
            workspace: crate::multi_agent::AgentWorkspaceConfig::default(),
            memory: crate::multi_agent::AgentMemoryConfig::default(),
            identity: IdentityConfig::default(),
            a2a: crate::multi_agent::AgentA2aConfig::default(),
        }
    }
}

impl AliasedAgentConfig {
    /// True when this agent has the bindings required to dispatch a turn:
    /// enabled, non-empty `model_provider`, `risk_profile`, and
    /// `runtime_profile`. `Config::validate()` emits the per-field errors
    /// that, when all passed, mean this returns `true`.
    #[must_use]
    pub fn is_dispatchable(&self) -> bool {
        self.enabled
            && !self.model_provider.trim().is_empty()
            && !self.risk_profile.trim().is_empty()
            && !self.runtime_profile.trim().is_empty()
    }
}

/// One `[channels.<type>.<alias>]` block, with the owning agent (if any)
/// resolved via `agents.<agent>.channels`. Returned by
/// `Config::channels_by_alias()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct ChannelAliasInfo {
    /// Channel type as the schema emits it (kebab; e.g. `"discord"`,
    /// `"nextcloud-talk"`).
    pub channel_type: String,
    /// Per-alias HashMap key (e.g. `"loneliness"`).
    pub alias: String,
    /// The agent whose `channels` list contains `<type>.<alias>`. `None`
    /// when the block is orphaned (config error caught at startup).
    pub owning_agent: Option<String>,
    /// Resolved value of `[channels.<type>.<alias>].enabled` at scan time.
    /// `false` when the field is unset (matches the serde bool default).
    pub enabled: bool,
}

impl Config {
    /// Resolve the configured alias values valid for an [`crate::traits::AliasSource`].
    /// Two-tier sources return dotted `<type>.<alias>` keys; flat sources
    /// return bare alias keys. The single resolver every surface uses for
    /// `PropKind::AliasRef` pickers and validation.
    #[must_use]
    pub fn resolve_alias_source(&self, source: crate::traits::AliasSource) -> Vec<String> {
        let section = source.section_path();
        if source.is_two_tier() {
            let prefix = format!("{section}.");
            let mut out: Vec<String> = Vec::new();
            for f in self.prop_fields() {
                if let Some(rest) = f.name.strip_prefix(&prefix) {
                    let mut parts = rest.splitn(3, '.');
                    if let (Some(ty), Some(alias), Some(_)) =
                        (parts.next(), parts.next(), parts.next())
                    {
                        let dotted = format!("{ty}.{alias}");
                        if !out.contains(&dotted) {
                            out.push(dotted);
                        }
                    }
                }
            }
            out.sort();
            out
        } else {
            let mut out = self.get_map_keys(section).unwrap_or_default();
            out.sort();
            out
        }
    }

    /// Return the first concrete `model` string available for use as a
    /// default: the model declared by the first entry that has one, in the
    /// iteration order of
    /// `ModelProviders::first_entry_with_model`. Returns `None` only when no
    /// model-provider entry has any model configured at all (upstream #10638).
    #[must_use]
    pub fn resolve_default_model(&self) -> Option<String> {
        self.providers
            .models
            .first_entry_with_model()
            .and_then(|(_, _, base)| base.model.as_deref())
            .map(str::trim)
            .filter(|m| !m.is_empty())
            .map(ToString::to_string)
    }

    /// Resolve the risk profile for an explicit agent alias.
    ///
    /// Each agent's `risk_profile` field names a `[risk_profiles.<alias>]`
    /// entry that gates its actions. There is no "global" risk profile in
    /// every callsite must come through an agent. When the agent has
    /// no profile set or names a missing entry, returns `None` and the
    /// caller decides how to handle it (validation rejects this shape at
    /// load time; the runtime treating `None` as a config error).
    #[must_use]
    pub fn risk_profile_for_agent(&self, agent_alias: &str) -> Option<&RiskProfileConfig> {
        let agent = self.agents.get(agent_alias)?;
        let profile_alias = agent.risk_profile.trim();
        if profile_alias.is_empty() {
            return None;
        }
        self.risk_profiles.get(profile_alias)
    }

    /// Resolve the delegate targets `caller_alias` may reach:
    /// same-profile peers when `delegate_same_risk_profile` is set, unioned
    /// with the explicit `delegates` roster, minus the caller. Single
    /// source of truth for delegate reach and mode; gating
    /// (`delegation_policy`) is enforced separately by the caller.
    /// Deduped, sorted; unknown caller yields empty. Disabled targets are
    /// never reachable. Explicit entries override the bounded mode used for
    /// implicit same-profile peers.
    #[must_use]
    pub fn reachable_delegate_target_configs(
        &self,
        caller_alias: &str,
    ) -> Vec<DelegateTargetConfig> {
        let Some(caller) = self.agents.get(caller_alias) else {
            return Vec::new();
        };

        let mut targets: std::collections::BTreeMap<String, DelegateExecutionMode> =
            std::collections::BTreeMap::new();

        if caller.delegate_same_risk_profile {
            let caller_profile = caller.risk_profile.trim();
            if !caller_profile.is_empty() {
                for (alias, agent) in &self.agents {
                    if alias.as_str() != caller_alias
                        && agent.enabled
                        && agent.risk_profile.trim() == caller_profile
                    {
                        targets.insert(alias.clone(), DelegateExecutionMode::Bounded);
                    }
                }
            }
        }

        for explicit in &caller.delegates {
            let trimmed = explicit.agent().trim();
            if trimmed.is_empty() || trimmed == caller_alias {
                continue;
            }
            if self.agents.get(trimmed).is_some_and(|agent| agent.enabled) {
                targets.insert(trimmed.to_string(), explicit.mode());
            }
        }

        targets
            .into_iter()
            .map(|(agent, mode)| DelegateTargetConfig { agent, mode })
            .collect()
    }

    /// Resolve only the agent aliases reachable by `caller_alias`.
    /// Use [`Self::reachable_delegate_target_configs`] when the target mode
    /// matters.
    #[must_use]
    pub fn reachable_delegate_targets(&self, caller_alias: &str) -> Vec<String> {
        self.reachable_delegate_target_configs(caller_alias)
            .into_iter()
            .map(|target| target.agent)
            .collect()
    }

    /// Resolve the execution mode for one reachable delegate target.
    ///
    /// Target aliases are normalized the same way the reachable roster
    /// normalizes explicit `delegates` entries. This keeps callers that need
    /// one target mode aligned with the canonical roster resolver instead of
    /// creating a subtly different admission rule.
    #[must_use]
    pub fn delegate_target_mode(
        &self,
        caller_alias: &str,
        target_alias: &str,
    ) -> Option<DelegateExecutionMode> {
        let target_alias = target_alias.trim();
        if target_alias.is_empty() {
            return None;
        }

        self.reachable_delegate_target_configs(caller_alias)
            .into_iter()
            .find(|target| target.agent == target_alias)
            .map(|target| target.mode)
    }

    /// Resolve the `[runtime_profiles.<alias>]` entry owned by an agent
    /// (via `agents.<alias>.runtime_profile`). Returns `None` when the
    /// agent has no runtime profile set or names a missing entry. Unlike
    /// `risk_profile_for_agent`, the missing case is not a hard error
    /// because runtime budgets and tunables fall back to global defaults.
    #[must_use]
    pub fn runtime_profile_for_agent(&self, agent_alias: &str) -> Option<&RuntimeProfileConfig> {
        let agent = self.agents.get(agent_alias)?;
        let profile_alias = agent.runtime_profile.trim();
        if profile_alias.is_empty() {
            return None;
        }
        self.runtime_profiles.get(profile_alias)
    }

    /// Effective context-compression summarizer provider for an agent:
    /// agent-level `summary_provider` override → the runtime profile's
    /// `context_compression.summary_provider` → `None` (the caller then reuses
    /// the agent's own provider+model, optionally via the deprecated
    /// `summary_model` swap). Unlike the inert agent-inline tunables below, the
    /// agent-level override IS consulted — it's an explicit per-agent choice,
    /// mirroring `classifier_provider`'s "empty = inherit" semantics.
    #[must_use]
    pub fn effective_summary_provider(
        &self,
        agent_alias: &str,
    ) -> Option<crate::providers::ModelProviderRef> {
        if let Some(a) = self.agents.get(agent_alias)
            && !a.summary_provider.trim().is_empty()
        {
            return Some(a.summary_provider.clone());
        }
        self.runtime_profile_for_agent(agent_alias)
            .map(|p| &p.context_compression.summary_provider)
            .filter(|r| !r.trim().is_empty())
            .cloned()
    }

    // ── Effective per-agent runtime tunables ──────────────────────────
    //
    // Runtime tunables live on `[runtime_profiles.<profile>]`, referenced by an
    // agent via `agents.<alias>.runtime_profile`. A profile value that is
    // explicitly set (non-sentinel, i.e. `> 0` for `max_tool_iterations`) is
    // authoritative; otherwise the global default applies. Agent-inline copies
    // of these tunable keys are inert — superseded by runtime profiles (see the
    // `agent_level_tunable_keys_are_inert` test) — so they are deliberately not
    // consulted here as a fallback.

    #[must_use]
    pub fn effective_max_tool_iterations(&self, agent_alias: &str) -> usize {
        self.runtime_profile_for_agent(agent_alias)
            .map(|p| p.max_tool_iterations)
            .filter(|&v| v > 0)
            .unwrap_or(10)
    }

    #[must_use]
    pub fn effective_max_history_messages(&self, agent_alias: &str) -> usize {
        self.runtime_profile_for_agent(agent_alias)
            .and_then(|p| p.max_history_messages)
            .unwrap_or(50)
    }

    /// Resolve the fraction of the history cap that a whole-turn trim drops
    /// to (hysteresis low-water mark). Same resolution chain as
    /// `effective_max_history_messages`: the agent's runtime profile value
    /// when set, otherwise [`DEFAULT_HISTORY_TRIM_LOW_WATER`].
    #[must_use]
    pub fn effective_history_trim_low_water(&self, agent_alias: &str) -> f32 {
        self.runtime_profile_for_agent(agent_alias)
            .and_then(|p| p.history_trim_low_water)
            .unwrap_or(DEFAULT_HISTORY_TRIM_LOW_WATER)
    }

    /// Resolve the whole-turn history cap used by structured `Agent` sessions.
    ///
    /// An explicit runtime-profile cap remains authoritative. When omitted, the
    /// cap scales with the profile's tool-iteration limit while preserving the
    /// legacy floor of 50 messages.
    #[must_use]
    pub fn effective_structured_max_history_messages(&self, agent_alias: &str) -> usize {
        if let Some(max_history_messages) = self
            .runtime_profile_for_agent(agent_alias)
            .and_then(|p| p.max_history_messages)
        {
            return max_history_messages;
        }

        // Each tool iteration adds two structural messages; user/final assistant
        // add two more, while the floor preserves the structured default cap of 50.
        self.effective_max_tool_iterations(agent_alias)
            .saturating_mul(2)
            .saturating_add(2)
            .max(50)
    }

    #[must_use]
    pub fn effective_max_context_tokens(&self, agent_alias: &str) -> usize {
        // Token budget for preemptive context/history trimming (runtime profile override).
        // This is NOT the provider max_tokens output limit and NOT the model's context window.
        self.runtime_profile_for_agent(agent_alias)
            .and_then(|p| p.max_context_tokens)
            .unwrap_or(32_000)
    }

    /// The model's context window exactly as configured, or `None` when no
    /// provider profile declares one.
    ///
    /// `None` means *unknown*, not "small". Report it to an operator as unset
    /// rather than resolving it to [`UNCONFIGURED_CONTEXT_WINDOW_FALLBACK`],
    /// which would present a stub as though it were the model's real capacity.
    /// Use [`Config::effective_model_context_window`] when a number is required
    /// for budget arithmetic.
    #[must_use]
    pub fn configured_model_context_window(&self, agent_alias: &str) -> Option<usize> {
        // Provider config (config.toml) is the source of truth for this value.
        self.resolved_model_provider_for_agent(agent_alias)
            .and_then(|(_, _, provider_config)| provider_config.context_window)
    }

    /// Returns the model's context window size (max input tokens).
    /// Source: provider config `context_window` →
    /// [`UNCONFIGURED_CONTEXT_WINDOW_FALLBACK`].
    /// Does NOT check runtime profile (that's for output budget).
    #[must_use]
    pub fn effective_model_context_window(&self, agent_alias: &str) -> usize {
        self.configured_model_context_window(agent_alias)
            .unwrap_or(UNCONFIGURED_CONTEXT_WINDOW_FALLBACK)
    }

    #[must_use]
    pub fn effective_memory_recall_limit(&self, agent_alias: &str) -> usize {
        let raw = self
            .runtime_profile_for_agent(agent_alias)
            .and_then(|p| p.memory_recall_limit)
            .unwrap_or(5);
        if raw == 0 { usize::MAX } else { raw }
    }

    #[must_use]
    pub fn effective_compact_context(&self, agent_alias: &str) -> bool {
        self.runtime_profile_for_agent(agent_alias)
            .and_then(|p| p.compact_context)
            .unwrap_or(true)
    }

    #[must_use]
    pub fn effective_parallel_tools(&self, agent_alias: &str) -> bool {
        self.runtime_profile_for_agent(agent_alias)
            .and_then(|p| p.parallel_tools)
            .unwrap_or(false)
    }

    #[must_use]
    pub fn effective_tool_dispatcher(&self, agent_alias: &str) -> String {
        self.runtime_profile_for_agent(agent_alias)
            .and_then(|p| p.tool_dispatcher.as_ref())
            .filter(|s| !s.trim().is_empty())
            .map_or_else(default_agent_tool_dispatcher, Clone::clone)
    }

    #[must_use]
    pub fn effective_tool_call_dedup_exempt(&self, agent_alias: &str) -> Vec<String> {
        self.runtime_profile_for_agent(agent_alias)
            .map(|p| p.tool_call_dedup_exempt.clone())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn effective_max_system_prompt_chars(&self, agent_alias: &str) -> usize {
        self.runtime_profile_for_agent(agent_alias)
            .and_then(|p| p.max_system_prompt_chars)
            .unwrap_or_else(default_max_system_prompt_chars)
    }

    #[must_use]
    pub fn effective_max_tool_result_chars(&self, agent_alias: &str) -> usize {
        self.runtime_profile_for_agent(agent_alias)
            .and_then(|p| p.max_tool_result_chars)
            .unwrap_or_else(default_max_tool_result_chars)
    }

    #[must_use]
    pub fn effective_keep_tool_context_turns(&self, agent_alias: &str) -> usize {
        self.runtime_profile_for_agent(agent_alias)
            .and_then(|p| p.keep_tool_context_turns)
            .unwrap_or_else(default_keep_tool_context_turns)
    }

    /// Return a clone of the named agent's `AliasedAgentConfig` with all
    /// runtime-profile overrides baked in. Use this when an `Agent` (or
    /// any other struct) needs to own a self-contained, already-resolved
    /// view of the agent's runtime knobs without holding a reference to
    /// the full `Config`.
    ///
    /// Returns `None` when `agent_alias` is not present in `agents`.
    ///
    /// Semantics: every field touched here mirrors the matching
    /// `effective_*` helper above. If you add a new runtime_profile knob,
    /// add it both to an `effective_*` helper *and* to this function so
    /// downstream consumers see consistent values regardless of which
    /// surface they read from.
    #[must_use]
    pub fn resolved_agent_config(&self, agent_alias: &str) -> Option<AliasedAgentConfig> {
        let mut out = self.agents.get(agent_alias)?.clone();
        let mut resolved = ResolvedRuntime {
            max_tool_iterations: self.effective_max_tool_iterations(agent_alias),
            max_history_messages: self.effective_max_history_messages(agent_alias),
            history_trim_low_water: self.effective_history_trim_low_water(agent_alias),
            // Token budget for context/history trimming — from runtime profile
            max_context_tokens: self.effective_max_context_tokens(agent_alias),
            // Model's context window (max input tokens) — from provider config
            model_context_window: self.effective_model_context_window(agent_alias),
            compact_context: self.effective_compact_context(agent_alias),
            parallel_tools: self.effective_parallel_tools(agent_alias),
            tool_dispatcher: self.effective_tool_dispatcher(agent_alias),
            tool_call_dedup_exempt: self.effective_tool_call_dedup_exempt(agent_alias),
            max_system_prompt_chars: self.effective_max_system_prompt_chars(agent_alias),
            max_tool_result_chars: self.effective_max_tool_result_chars(agent_alias),
            keep_tool_context_turns: self.effective_keep_tool_context_turns(agent_alias),
            prompt_injection_mode: self.effective_skills_prompt_mode(agent_alias),
            ..ResolvedRuntime::default()
        };
        if let Some(profile) = self.runtime_profile_for_agent(agent_alias) {
            resolved.strict_tool_parsing = profile.strict_tool_parsing;
            resolved.thinking = profile.thinking.clone();
            resolved.history_pruning = profile.history_pruning.clone();
            resolved.eval = profile.eval.clone();
            resolved.auto_classify = profile.auto_classify.clone();
            resolved.context_compression = profile.context_compression.clone();
            resolved.tool_receipts = profile.tool_receipts.clone();
            resolved.tool_filter_groups = profile.tool_filter_groups.clone();
        }
        out.resolved = resolved;
        Some(out)
    }

    /// Resolve the effective skills prompt-injection mode for an agent: use an
    /// explicit runtime-profile override when set, otherwise inherit the global
    /// `[skills] prompt_injection_mode` value.
    ///
    /// Keyed on the resolved runtime profile — the sanctioned surface for
    /// per-agent runtime tunables — so agent-inline knobs stay inert.
    /// Centralizing the fallback here keeps the system-prompt builder and the
    /// `read_skill` tool-registration gate in lockstep on one effective mode,
    /// so a compact prompt is never paired with a missing `read_skill` tool
    /// (or a full prompt with a spurious one).
    #[must_use]
    pub fn effective_skills_prompt_mode(&self, agent_alias: &str) -> SkillsPromptInjectionMode {
        self.runtime_profile_for_agent(agent_alias)
            .and_then(|p| p.prompt_injection_mode)
            .unwrap_or(self.skills.prompt_injection_mode)
    }

    /// Resolve an agent's `model_provider` reference (`"<type>.<alias>"`) to
    /// its concrete `ModelProviderConfig` entry. Returns `None` when the
    /// agent doesn't exist, the reference is unparseable, or the
    /// `<type>.<alias>` pair doesn't resolve in `providers.models`.
    ///
    /// This is the lookup the orchestrator uses to build per-agent
    /// model_provider runtime options via explicit `<type>.<alias>`
    /// resolution — there is no concept of a "first" or "default"
    /// provider. The matching split logic lives in
    /// `crates/omnesagent-runtime/src/tools/delegate.rs::resolve_brain` for
    /// the delegation path; this helper exposes the same contract for the
    /// channel-server startup path.
    #[must_use]
    pub fn model_provider_for_agent(&self, agent_alias: &str) -> Option<&ModelProviderConfig> {
        let agent = self.agents.get(agent_alias)?;
        let (type_key, alias_key) = agent.model_provider.split_once('.')?;
        self.providers.models.find(type_key, alias_key)
    }

    /// Resolve `(provider_type, provider_alias, &ModelProviderConfig)` for an
    /// agent. Same lookup as `model_provider_for_agent` but also returns the
    /// `'static` type key that downstream provider factories
    /// (`create_routed_model_provider_with_options`, etc.) need. Returns
    /// `None` when the agent has no `model_provider` set, when the reference
    /// is unparseable, or when the resolved entry has been deleted from
    /// `providers.models`.
    #[must_use]
    pub fn resolved_model_provider_for_agent(
        &self,
        agent_alias: &str,
    ) -> Option<(&'static str, &str, &ModelProviderConfig)> {
        let agent = self.agents.get(agent_alias)?;
        let (type_key, alias_key) = agent.model_provider.split_once('.')?;
        self.providers
            .models
            .iter_entries()
            .find(|(ty, al, _)| *ty == type_key && *al == alias_key)
    }

    /// Reverse-lookup the agent alias that owns a configured channel
    /// (`<type>.<alias>`). Returns the first agent listing the channel in
    /// its `channels` field. `None` when no agent owns the channel —
    /// orphaned channels are a config error the orchestrator surfaces at
    /// startup.
    #[must_use]
    pub fn agent_for_channel(&self, channel_alias: &str) -> Option<&str> {
        self.agents
            .iter()
            .find(|(_, agent)| agent.enabled && agent.channels.iter().any(|c| c == channel_alias))
            .map(|(alias, _)| alias.as_str())
    }

    /// Workspace dir a channel's inbound-media handler writes into. Resolves
    /// the channel's owning agent and returns `<install>/agents/<alias>/workspace/`;
    /// falls back to `data_dir` for orphan channels (no owning agent enabled).
    #[must_use]
    pub fn channel_workspace_dir(&self, channel_ref: &str) -> PathBuf {
        self.agent_for_channel(channel_ref)
            .map_or_else(|| self.data_dir.clone(), |a| self.agent_workspace_dir(a))
    }

    /// Schema-walk: every populated `[channels.<type>.<alias>]` block.
    /// Type names come from the `prop_fields()` enumeration (kebab as the
    /// macro emits them) so adding a new channel type via the macro
    /// surfaces here without touching this code. Alias keys are HashMap
    /// keys; not kebab-converted.
    #[must_use]
    pub fn channels_by_alias(&self) -> Vec<ChannelAliasInfo> {
        use std::collections::BTreeMap;
        let mut seen: BTreeMap<(String, String), bool> = BTreeMap::new();
        for field in self.prop_fields() {
            let parts: Vec<&str> = field.name.split('.').collect();
            if parts.len() < 4 || parts[0] != "channels" {
                continue;
            }
            let key = (parts[1].to_string(), parts[2].to_string());
            let entry = seen.entry(key).or_insert(false);
            if parts.len() == 4 && parts[3] == "enabled" {
                *entry = field.display_value == "true";
            }
        }
        seen.into_iter()
            .map(|((channel_type, alias), enabled)| {
                let composite = format!("{channel_type}.{alias}");
                let owning_agent = self.agent_for_channel(&composite).map(str::to_string);
                ChannelAliasInfo {
                    channel_type,
                    alias,
                    owning_agent,
                    enabled,
                }
            })
            .collect()
    }

    /// Reverse-lookup the agent alias that owns a declaratively-configured
    /// cron job (`[cron.<alias>]`). Returns the first agent listing the
    /// alias in its `cron_jobs` field. `None` when no agent claims the
    /// job — orphaned cron jobs are skipped at scheduler time with a
    /// warning. Imperative jobs (created at runtime via `cron_add`) have
    /// UUID-shaped ids that won't match any agent's `cron_jobs`; the
    /// scheduler treats those separately (carrying their owning agent
    /// alongside the DB row is a follow-up).
    #[must_use]
    pub fn agent_for_cron_job(&self, cron_alias: &str) -> Option<&str> {
        self.agents
            .iter()
            .find(|(_, agent)| agent.enabled && agent.cron_jobs.iter().any(|c| c == cron_alias))
            .map(|(alias, _)| alias.as_str())
    }

    /// Resolve the per-agent workspace directory for `alias`.
    ///
    /// Returns the agent's `[agents.<alias>.workspace.path]` override
    /// when set (operator-explicit, e.g. for putting a workspace on a
    /// different disk), otherwise derives
    /// `<install>/agents/<alias>/workspace/` from the install root
    /// (the directory containing `config.toml`).
    ///
    /// Per-agent workspaces live under
    /// `<install>/agents/<alias>/workspace/` and hold the agent's
    /// markdown memory (MEMORY.md), identity files (IDENTITY.md,
    /// SOUL.md), and any other per-agent plaintext state. Shared
    /// databases (SQLite memory, sessions, cost records) live under
    /// `config.data_dir` instead and partition by agent at the row
    /// level. Per-agent overrides via `[agents.<alias>.workspace.path]`
    /// pin an arbitrary filesystem path (e.g. a different mount).
    #[must_use]
    pub fn agent_workspace_dir(&self, agent_alias: &str) -> std::path::PathBuf {
        if let Some(cfg) = self.agents.get(agent_alias)
            && let Some(custom) = cfg.workspace.path.as_ref()
        {
            return custom.clone();
        }
        self.install_root_dir()
            .join("agents")
            .join(agent_alias)
            .join("workspace")
    }

    /// Effective MCP servers granted to an agent by its `mcp_bundles`.
    ///
    /// Secure by default: omission is not a grant. An agent is reached via
    /// `resolved_agent_config`; an unknown alias or one with no `mcp_bundles`
    /// receives NO MCP servers. See [`Self::mcp_servers_for_bundles`] for how
    /// a non-empty bundle list resolves to servers.
    #[must_use]
    pub fn mcp_servers_for_agent(&self, agent_alias: &str) -> Vec<McpServerConfig> {
        match self.resolved_agent_config(agent_alias) {
            Some(agent) => self.mcp_servers_for_bundles(&agent.mcp_bundles),
            None => Vec::new(),
        }
    }

    /// Resolve a set of `[mcp_bundles.<alias>]` references to the concrete
    /// `[mcp.servers]` entries they grant.
    ///
    /// Secure by default: omission is not a grant.
    /// - An empty `bundle_aliases` grants NO servers (`Vec::new()`).
    /// - The grant is the union of each referenced bundle's `servers`, in
    ///   first-seen order with duplicates dropped.
    /// - Deny wins: a server name listed in ANY referenced bundle's `exclude`
    ///   is removed from the grant, regardless of which bundle included it.
    /// - An unknown bundle alias grants nothing (it is skipped, not an error)
    ///   and a server name with no matching `[mcp.servers]` entry grants
    ///   nothing. Both fail closed: a misconfiguration narrows access, never
    ///   widens it.
    #[must_use]
    pub fn mcp_servers_for_bundles(&self, bundle_aliases: &[String]) -> Vec<McpServerConfig> {
        if bundle_aliases.is_empty() {
            return Vec::new();
        }
        // Deny wins: collect every excluded name across the referenced bundles
        // first, so an include in one bundle cannot defeat an exclude in another.
        let mut excluded: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for alias in bundle_aliases {
            if let Some(bundle) = self.mcp_bundles.get(alias) {
                for name in &bundle.exclude {
                    excluded.insert(name.as_str());
                }
            }
        }
        // Union of granted names in first-seen order, skipping excludes.
        let mut granted: Vec<&str> = Vec::new();
        for alias in bundle_aliases {
            if let Some(bundle) = self.mcp_bundles.get(alias) {
                for name in &bundle.servers {
                    let n = name.as_str();
                    if !excluded.contains(n) && !granted.contains(&n) {
                        granted.push(n);
                    }
                }
            }
        }
        // Resolve names against the configured servers. A name that matches no
        // `[mcp.servers]` entry is not granted.
        granted
            .into_iter()
            .filter_map(|name| self.mcp.servers.iter().find(|s| s.name == name).cloned())
            .collect()
    }

    /// `<install>/shared/` — directory shared across every agent on this
    /// host. Holds skills, skill bundles, knowledge bundles, and any
    /// other content not scoped to a single agent's workspace. Distinct
    /// from `agent_workspace_dir(alias)` (per-agent state) and
    /// `data_dir` (databases + runtime state).
    #[must_use]
    pub fn shared_workspace_dir(&self) -> std::path::PathBuf {
        self.install_root_dir().join("shared")
    }

    /// Install root: `<install>/` derived from `config_path`'s parent. Used
    /// to compute `<install>/shared/`, `<install>/agents/`, and the
    /// skill-bundle directory defaults. Public so consumers (gateway, CLI,
    /// SkillsService) share the same anchor.
    #[must_use]
    pub fn install_root_dir(&self) -> std::path::PathBuf {
        self.config_path
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    }

    /// Resolve an aliased-agent config by alias. `None` when the alias
    /// isn't configured; callers should treat this as a config error
    /// rather than synthesizing a default.
    #[must_use]
    pub fn agent(&self, agent_alias: &str) -> Option<&AliasedAgentConfig> {
        self.agents.get(agent_alias)
    }

    /// Resolve the runtime-active agent alias the orchestrator binds
    /// channels to. Mirrors the same selection logic as
    /// `start_channels()` in omnesagent-channels: prefer the migration-
    /// synthesized `"default"` agent, otherwise fall back to the
    /// lexicographically-smallest enabled alias. Returns `None` only
    /// when no enabled agent is configured.
    ///
    /// Used by per-agent infrastructure (TtsManager, TranscriptionManager)
    /// to pick which agent's `tts_provider` / `transcription_provider`
    /// drives the manager's resolved alias. Until the per-channel
    /// dispatch refactor lands, the orchestrator runs in single-agent
    /// mode, so all manager instances share the same resolved agent.
    #[must_use]
    pub fn resolved_runtime_agent_alias(&self) -> Option<&str> {
        self.agents
            .keys()
            .find(|k| k.as_str() == "default")
            .map(String::as_str)
            .or_else(|| {
                self.agents
                    .iter()
                    .filter(|(_, a)| a.enabled)
                    .map(|(alias, _)| alias.as_str())
                    .min()
            })
    }

    /// Resolve the active storage backend for the memory subsystem.
    ///
    /// `MemoryConfig.backend` is a dotted reference (`<backend>.<alias>`) into
    /// `Config.storage.<backend>.<alias>`. Bare backend names are interpreted
    /// as `<backend>.default` for back-compat.
    ///
    /// Returns `ActiveStorage::None` when no backend is configured, when the
    /// backend is `"none"`, or when the dotted alias does not resolve to a
    /// configured entry.
    pub fn resolve_active_storage(&self) -> ActiveStorage<'_> {
        let backend = self.memory.backend.trim();
        if backend.is_empty() || backend.eq_ignore_ascii_case("none") {
            return ActiveStorage::None;
        }
        let (kind, alias) = backend.split_once('.').unwrap_or((backend, "default"));
        match kind {
            "sqlite" => self
                .storage
                .sqlite
                .get(alias)
                .map(ActiveStorage::Sqlite)
                .unwrap_or(ActiveStorage::None),
            "postgres" => self
                .storage
                .postgres
                .get(alias)
                .map(ActiveStorage::Postgres)
                .unwrap_or(ActiveStorage::None),
            "qdrant" => self
                .storage
                .qdrant
                .get(alias)
                .map(ActiveStorage::Qdrant)
                .unwrap_or(ActiveStorage::None),
            "markdown" => self
                .storage
                .markdown
                .get(alias)
                .map(ActiveStorage::Markdown)
                .unwrap_or(ActiveStorage::None),
            "lucid" => self
                .storage
                .lucid
                .get(alias)
                .map(ActiveStorage::Lucid)
                .unwrap_or(ActiveStorage::None),
            _ => ActiveStorage::None,
        }
    }
}

/// Resolved storage backend variant.
///
/// Returned from [`Config::resolve_active_storage`]. Each variant carries a
/// borrow of the typed config from the corresponding `Config.storage` map.
#[derive(Debug, Clone, Copy)]
pub enum ActiveStorage<'a> {
    /// No storage configured (`memory.backend = "none"` or unresolved alias).
    None,
    /// SQLite storage instance.
    Sqlite(&'a SqliteStorageConfig),
    /// PostgreSQL storage instance.
    Postgres(&'a PostgresStorageConfig),
    /// Qdrant storage instance.
    Qdrant(&'a QdrantStorageConfig),
    /// Markdown directory storage instance.
    Markdown(&'a MarkdownStorageConfig),
    /// Lucid CLI sync instance.
    Lucid(&'a LucidStorageConfig),
}

impl ActiveStorage<'_> {
    /// Backend type name (`"sqlite"`, `"postgres"`, etc.); `"none"` for unconfigured.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            ActiveStorage::None => "none",
            ActiveStorage::Sqlite(_) => "sqlite",
            ActiveStorage::Postgres(_) => "postgres",
            ActiveStorage::Qdrant(_) => "qdrant",
            ActiveStorage::Markdown(_) => "markdown",
            ActiveStorage::Lucid(_) => "lucid",
        }
    }
}

pub(crate) fn default_delegate_timeout_secs() -> u64 {
    DEFAULT_DELEGATE_TIMEOUT_SECS
}

pub(crate) fn default_delegate_agentic_timeout_secs() -> u64 {
    DEFAULT_DELEGATE_AGENTIC_TIMEOUT_SECS
}

/// Valid temperature range for all paths (config, CLI, env override).
pub const TEMPERATURE_RANGE: std::ops::RangeInclusive<f64> = 0.0..=2.0;

/// Context window assumed when no provider profile declares one.
///
/// Deliberately conservative: it has to be safe for any model, which means it
/// is almost always *wrong* for the model actually in use — a 1M-context model
/// on a profile that omits `context_window` runs against this number. It exists
/// so budget arithmetic has an operand, not because it describes any model.
///
/// Anything reporting to an operator must call
/// [`Config::configured_model_context_window`] and say "not configured" when it
/// returns `None`, rather than echoing this value as the model's real capacity.
pub const UNCONFIGURED_CONTEXT_WINDOW_FALLBACK: usize = 32_000;

/// Defaults to 0 so configs without an explicit `schema_version` are recognized
/// as pre-versioning and get migrated.
pub(crate) fn default_schema_version() -> u32 {
    0
}

/// Default delegate tool timeout for non-agentic calls: 120 seconds.
pub const DEFAULT_DELEGATE_TIMEOUT_SECS: u64 = 120;

/// Default delegate tool timeout for agentic runs: 300 seconds.
pub const DEFAULT_DELEGATE_AGENTIC_TIMEOUT_SECS: u64 = 300;

/// Per-channel reply-pacing accessor. Implemented by every `*Config`
/// struct that participates in outbound pacing so validation and
/// wrapper construction can walk all of them through a single
/// abstraction rather than nine duplicated method calls.
pub trait HasReplyPacing {
    fn reply_min_interval_secs(&self) -> u64;
    fn reply_queue_depth_max(&self) -> u16;
}

/// Inclusive upper bound (seconds) for per-channel `reply_min_interval_secs`.
pub const REPLY_MIN_INTERVAL_MAX_SECS: u64 = 3600;

/// Inclusive upper bound for per-channel `reply_queue_depth_max`. The lower
/// bound is `0`, where `0` means "use [`DEFAULT_REPLY_QUEUE_DEPTH`] at the
/// pacing-wrapper construction site." A non-zero value pins the bound
/// explicitly. Validator rejects values above this ceiling.
pub const REPLY_QUEUE_DEPTH_CEILING: u16 = 1024;

/// Fallback queue depth applied at the pacing-wrapper construction site
/// when a channel's `reply_queue_depth_max` is left at `0`. Sized for the
/// AI-pacing use case: a paced channel buffering more than this is a sign
/// the agent is producing replies faster than the floor will ever drain
/// them and the overflow log is the right signal.
pub const DEFAULT_REPLY_QUEUE_DEPTH: u16 = 16;

/// Idle-state LRU cap on the pacing wrapper's per-recipient rows.
/// Bounds growth when a bot legitimately serves many thousands of distinct
/// peers. Eviction only reclaims idle rows (no queued sends, no running
/// worker, no in-flight dispatch), so under a pathological all-active burst
/// the row count can temporarily exceed this target until rows become idle —
/// it is not an unconditional hard bound. Each recipient's queue depth stays
/// bounded regardless. Not exposed in config — promote to a schema field if
/// an operator reports hitting it.
pub const PACING_RECIPIENT_CAP: usize = 1024;

/// Validate that a temperature value is within the allowed range.
pub fn validate_temperature(value: f64) -> std::result::Result<f64, String> {
    if TEMPERATURE_RANGE.contains(&value) {
        Ok(value)
    } else {
        Err(format!(
            "temperature {value} is out of range (expected {}..={})",
            TEMPERATURE_RANGE.start(),
            TEMPERATURE_RANGE.end()
        ))
    }
}

fn normalize_reasoning_effort(value: &str) -> std::result::Result<String, String> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "minimal" | "low" | "medium" | "high" | "xhigh" => Ok(normalized),
        _ => Err(format!(
            "reasoning_effort {value:?} is invalid (expected one of: minimal, low, medium, high, xhigh)"
        )),
    }
}

pub(crate) fn deserialize_reasoning_effort_opt<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<String> = Option::deserialize(deserializer)?;
    value
        .map(|raw| normalize_reasoning_effort(&raw).map_err(serde::de::Error::custom))
        .transpose()
}

pub(crate) fn deserialize_enum_lenient<'de, D, T>(deserializer: D) -> std::result::Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned + Default,
{
    let raw = serde_json::Value::deserialize(deserializer)?;
    Ok(T::deserialize(raw).unwrap_or_default())
}

/// Deserialize an `Option<String>` that maps an empty literal `""` to
/// `None`. Used by `JiraConfig::email` so a config that round-tripped
/// `email = ""` to disk (the legacy `email: String` had no
/// `skip_serializing_if`) doesn't deserialize as `Some("")` and silently
/// break Basic auth — the email-required validation was removed when
/// Server/DC Bearer-token support landed, so this is the last line of
/// defense.
pub(crate) fn deserialize_optional_email_skip_empty<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<String> = Option::deserialize(deserializer)?;
    Ok(value.filter(|s| !s.trim().is_empty()))
}

fn default_max_tool_result_chars() -> usize {
    50_000
}

fn default_keep_tool_context_turns() -> usize {
    2
}

fn default_agent_tool_dispatcher() -> String {
    "auto".into()
}

fn default_max_system_prompt_chars() -> usize {
    0
}

pub(crate) const MCP_MAX_TOOL_TIMEOUT_SECS: u64 = 600;

pub(crate) fn validate_plugin_entries(config: &PluginsConfig) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    for (index, entry) in config.entries.iter().enumerate() {
        let instance_key = entry.name.trim();
        if instance_key.is_empty() {
            anyhow::bail!("plugins.entries[{index}].name must not be empty");
        }
        if instance_key != entry.name {
            anyhow::bail!(
                "plugins.entries[{index}].name must not have leading or trailing whitespace"
            );
        }
        if !seen.insert(instance_key) {
            anyhow::bail!("plugins.entries contains a duplicate instance key");
        }
    }
    Ok(())
}

/// Reject a `[channels.plugin.<alias>]` declaration the activation loader could
/// never resolve to an installed package.
///
/// The alias uses the shared config alias grammar because it becomes the
/// channel's registry alias (`plugin.<alias>`); the package uses the shared
/// plugin package-name grammar so config and manifest admission agree on one
/// spelling. Aliases are visited in sorted order so the reported failure is
/// stable across runs regardless of map iteration order.
pub(crate) fn validate_plugin_channel_instances(config: &ChannelsConfig) -> Result<()> {
    let mut aliases: Vec<_> = config.plugin.keys().collect();
    aliases.sort_unstable();
    for alias in aliases {
        crate::helpers::validate_alias_key(alias)
            .map_err(|error| anyhow::Error::msg(format!("channels.plugin.{alias}: {error}")))?;
        let declaration = &config.plugin[alias];
        if declaration.package.trim() != declaration.package {
            anyhow::bail!(
                "channels.plugin.{alias}.package must not have leading or trailing whitespace"
            );
        }
        omnesagent_api::plugin::validate_plugin_package_name(&declaration.package).map_err(
            |error| anyhow::Error::msg(format!("channels.plugin.{alias}.package: {error}")),
        )?;
    }
    Ok(())
}

pub(crate) fn validate_mcp_config(config: &McpConfig) -> Result<()> {
    let mut seen_names = std::collections::HashSet::new();
    for (i, server) in config.servers.iter().enumerate() {
        let name = server.name.trim();
        if name.is_empty() {
            validation_bail!(
                RequiredFieldEmpty,
                format!("mcp.servers[{i}].name"),
                "mcp.servers[{i}].name must not be empty"
            );
        }
        if !seen_names.insert(name.to_ascii_lowercase()) {
            anyhow::bail!("mcp.servers contains duplicate name: {name}");
        }

        if let Some(timeout) = server.tool_timeout_secs {
            if timeout == 0 {
                validation_bail!(
                    InvalidNumericRange,
                    format!("mcp.servers[{i}].tool_timeout_secs"),
                    "mcp.servers[{i}].tool_timeout_secs must be greater than 0"
                );
            }
            if timeout > MCP_MAX_TOOL_TIMEOUT_SECS {
                anyhow::bail!(
                    "mcp.servers[{i}].tool_timeout_secs exceeds max {MCP_MAX_TOOL_TIMEOUT_SECS}"
                );
            }
        }

        // The transport -> required-leaf relationship is owned by
        // `McpTransport::required_leaf`, which feeds the `x-required-by-transport`
        // schema metadata. The validator matches on the `McpTransport` enum so a
        // new variant is a compile error here rather than a runtime fall-through.
        match server.transport {
            McpTransport::Stdio => {
                if server.command.trim().is_empty() {
                    anyhow::bail!(
                        "mcp.servers[{i}] with transport=stdio requires non-empty command"
                    );
                }
            }
            McpTransport::Http | McpTransport::Sse => {
                let url = server
                    .url
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        let transport_str = server.transport.wire_name();
                        ::omnesagent_log::record!(
                            WARN,
                            ::omnesagent_log::Event::new(
                                module_path!(),
                                ::omnesagent_log::Action::Reject
                            )
                            .with_outcome(::omnesagent_log::EventOutcome::Failure)
                            .with_attrs(::serde_json::json!({
                                "index": i,
                                "transport": transport_str,
                            })),
                            "mcp.servers entry rejected: transport requires url"
                        );
                        anyhow::Error::msg(format!(
                            "mcp.servers[{i}] with transport={transport_str} requires url"
                        ))
                    })?;
                let parsed = reqwest::Url::parse(url)
                    .with_context(|| format!("mcp.servers[{i}].url is not a valid URL"))?;
                if !matches!(parsed.scheme(), "http" | "https") {
                    anyhow::bail!("mcp.servers[{i}].url must use http/https");
                }
                if let Some(ca_path) = server.tls_ca_cert_path.as_deref() {
                    if ca_path.trim().is_empty() {
                        validation_bail!(
                            RequiredFieldEmpty,
                            format!("mcp.servers[{i}].tls_ca_cert_path"),
                            "mcp.servers[{i}].tls_ca_cert_path must not be empty"
                        );
                    }
                    if !std::path::Path::new(ca_path).is_absolute() {
                        anyhow::bail!("mcp.servers[{i}].tls_ca_cert_path must be an absolute path");
                    }
                    if parsed.scheme() != "https" {
                        anyhow::bail!(
                            "mcp.servers[{i}].url must use https when tls_ca_cert_path is set"
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_proxy_url(field: &str, url: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url)
        .with_context(|| format!("Invalid {field} URL: '{url}' is not a valid URL"))?;

    match parsed.scheme() {
        "http" | "https" | "socks5" | "socks5h" | "socks" => {}
        scheme => {
            anyhow::bail!(
                "Invalid {field} URL scheme '{scheme}'. Allowed: http, https, socks5, socks5h, socks"
            );
        }
    }

    if parsed.host_str().is_none() {
        anyhow::bail!("Invalid {field} URL: host is required");
    }

    Ok(())
}

pub(crate) fn validate_http_base_url(field: &str, url: &str) -> Result<()> {
    if url.trim().is_empty() {
        validation_bail!(
            RequiredFieldEmpty,
            field.to_string(),
            "{field} must not be empty"
        );
    }

    let parsed = match reqwest::Url::parse(url) {
        Ok(parsed) => parsed,
        Err(err) => {
            validation_bail!(
                InvalidFormat,
                field.to_string(),
                "{field} must be a valid URL: {err}"
            );
        }
    };

    if !matches!(parsed.scheme(), "http" | "https") {
        validation_bail!(
            InvalidFormat,
            field.to_string(),
            "{field} must use http:// or https://"
        );
    }

    if parsed.host_str().is_none() {
        validation_bail!(
            InvalidFormat,
            field.to_string(),
            "{field} must include a host"
        );
    }

    Ok(())
}

/// Shared bot-token rule for channel structs whose `bot_token` is required
/// once the alias is enabled (Telegram, Discord). `field_path` must be the
/// `channels.<type>.<alias>.bot_token` leaf; the enabled-state message
/// derives the sibling `.enabled` path from it.
pub(crate) fn validate_required_bot_token(field_path: &str, enabled: bool, token: &str) -> Result<()> {
    if token.trim() == crate::traits::UNSET_DISPLAY {
        validation_bail!(
            RequiredFieldEmpty,
            field_path.to_string(),
            "{field_path} must not contain the unset display placeholder",
        );
    }
    if enabled && crate::traits::is_unset_display_value(token) {
        let enabled_path = field_path.strip_suffix("bot_token").map_or_else(
            || field_path.to_string(),
            |prefix| format!("{prefix}enabled"),
        );
        validation_bail!(
            RequiredFieldEmpty,
            field_path.to_string(),
            "{field_path} is required when {enabled_path} = true",
        );
    }
    Ok(())
}

/// Shared "required once enabled" rule for channel-credential fields whose
/// name doesn't end in `bot_token` (Signal's `http_url`/`account`, Voice
/// Call's `account_id`/`auth_token`/`from_number`), so the sibling
/// `.enabled` path can't be derived by suffix-stripping like
/// `validate_required_bot_token` does. An enabled channel built with an
/// empty required credential never connects, so its per-channel supervisor
/// restarts it forever; rejecting at config-load time turns that crashloop
/// into a fail-fast startup error instead.
pub(crate) fn validate_required_field(
    field_path: &str,
    enabled_path: &str,
    enabled: bool,
    value: &str,
) -> Result<()> {
    if value.trim() == crate::traits::UNSET_DISPLAY {
        validation_bail!(
            RequiredFieldEmpty,
            field_path.to_string(),
            "{field_path} must not contain the unset display placeholder",
        );
    }
    if enabled && crate::traits::is_unset_display_value(value) {
        validation_bail!(
            RequiredFieldEmpty,
            field_path.to_string(),
            "{field_path} is required when {enabled_path} = true",
        );
    }
    Ok(())
}

pub(crate) fn set_proxy_env_pair(key: &str, value: Option<&str>) {
    let lowercase_key = key.to_ascii_lowercase();
    if let Some(value) = value.and_then(|candidate| normalize_proxy_url_option(Some(candidate))) {
        // SAFETY: called during single-threaded config init before async runtime starts.
        unsafe {
            std::env::set_var(key, &value);
            std::env::set_var(lowercase_key, value);
        }
    } else {
        // SAFETY: called during single-threaded config init before async runtime starts.
        unsafe {
            std::env::remove_var(key);
            std::env::remove_var(lowercase_key);
        }
    }
}

pub(crate) fn clear_proxy_env_pair(key: &str) {
    // SAFETY: called during single-threaded config init before async runtime starts.
    unsafe {
        std::env::remove_var(key);
        std::env::remove_var(key.to_ascii_lowercase());
    }
}

/// Return the process proxy variable that reqwest would use for `raw_url`,
/// excluding destinations covered by `NO_PROXY`/`no_proxy`.
///
/// DNS-pinned callers use this to surface an intentional environment-proxy
/// bypass instead of silently changing the operator's egress path.
pub fn environment_proxy_for_url(raw_url: &str) -> Option<&'static str> {
    environment_proxy_for_url_with(raw_url, |key| std::env::var(key).ok())
}

pub(crate) fn environment_proxy_for_url_with(
    raw_url: &str,
    mut read_env: impl FnMut(&str) -> Option<String>,
) -> Option<&'static str> {
    fn first_nonempty(
        read_env: &mut impl FnMut(&str) -> Option<String>,
        keys: &[&'static str],
    ) -> Option<&'static str> {
        keys.iter().copied().find(|key| {
            read_env(key)
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
        })
    }

    let parsed = reqwest::Url::parse(raw_url).ok()?;
    let scheme_keys: &[&str] = match parsed.scheme() {
        "http" => &["HTTP_PROXY", "http_proxy"],
        "https" => &["HTTPS_PROXY", "https_proxy"],
        _ => return None,
    };
    let proxy_var = first_nonempty(&mut read_env, scheme_keys)
        .or_else(|| first_nonempty(&mut read_env, &["ALL_PROXY", "all_proxy"]))?;
    let host = parsed.host_str()?;
    let no_proxy = read_env("NO_PROXY")
        .filter(|value| !value.trim().is_empty())
        .or_else(|| read_env("no_proxy").filter(|value| !value.trim().is_empty()));

    if no_proxy
        .as_deref()
        .is_some_and(|entries| no_proxy_matches_host(entries, host))
    {
        None
    } else {
        Some(proxy_var)
    }
}

fn no_proxy_matches_host(entries: &str, host: &str) -> bool {
    let host = host
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim_end_matches('.')
        .to_ascii_lowercase();
    let host_ip = host.parse::<std::net::IpAddr>().ok();

    entries.split(',').map(str::trim).any(|entry| {
        if entry == "*" {
            return true;
        }
        let entry = entry.trim_end_matches('.');
        if entry.is_empty() {
            return false;
        }

        if let Some(host_ip) = host_ip
            && let Some((network, prefix)) = entry.split_once('/')
            && let (Ok(network), Ok(prefix)) =
                (network.parse::<std::net::IpAddr>(), prefix.parse::<u32>())
        {
            return ip_in_prefix(host_ip, network, prefix);
        }

        if let Ok(entry_ip) = entry.parse::<std::net::IpAddr>() {
            return host_ip == Some(entry_ip);
        }

        let domain = entry.trim_start_matches('.').to_ascii_lowercase();
        host == domain
            || host
                .strip_suffix(&domain)
                .is_some_and(|prefix| prefix.ends_with('.'))
    })
}

fn ip_in_prefix(host: std::net::IpAddr, network: std::net::IpAddr, prefix: u32) -> bool {
    match (host, network) {
        (std::net::IpAddr::V4(host), std::net::IpAddr::V4(network)) if prefix <= 32 => {
            let mask = if prefix == 0 {
                0
            } else {
                u32::MAX << (32 - prefix)
            };
            u32::from(host) & mask == u32::from(network) & mask
        }
        (std::net::IpAddr::V6(host), std::net::IpAddr::V6(network)) if prefix <= 128 => {
            let mask = if prefix == 0 {
                0
            } else {
                u128::MAX << (128 - prefix)
            };
            u128::from(host) & mask == u128::from(network) & mask
        }
        _ => false,
    }
}

fn runtime_proxy_state() -> &'static RwLock<ProxyConfig> {
    RUNTIME_PROXY_CONFIG.get_or_init(|| RwLock::new(ProxyConfig::default()))
}

pub(crate) fn runtime_proxy_config_generation() -> u64 {
    RUNTIME_PROXY_CONFIG_GENERATION.load(Ordering::Acquire)
}

fn bump_runtime_proxy_config_generation() {
    let _ = RUNTIME_PROXY_CONFIG_GENERATION.fetch_add(1, Ordering::AcqRel);
}

fn runtime_proxy_current_generation() -> u64 {
    match runtime_proxy_state().read() {
        Ok(_guard) => runtime_proxy_config_generation(),
        Err(poisoned) => {
            let _guard = poisoned.into_inner();
            runtime_proxy_config_generation()
        }
    }
}

pub(crate) fn runtime_proxy_client_cache() -> &'static RwLock<HashMap<String, RuntimeProxyCachedClient>> {
    RUNTIME_PROXY_CLIENT_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

pub(crate) fn clear_runtime_proxy_client_cache() {
    match runtime_proxy_client_cache().write() {
        Ok(mut guard) => {
            guard.clear();
        }
        Err(poisoned) => {
            poisoned.into_inner().clear();
        }
    }
}

pub(crate) fn runtime_proxy_cache_key(
    service_key: &str,
    timeout_secs: Option<u64>,
    connect_timeout_secs: Option<u64>,
) -> String {
    format!(
        "{}|timeout={}|connect_timeout={}",
        service_key.trim().to_ascii_lowercase(),
        timeout_secs
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        connect_timeout_secs
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    )
}

pub(crate) fn runtime_proxy_cached_client(cache_key: &str) -> Option<reqwest::Client> {
    let generation = runtime_proxy_current_generation();
    match runtime_proxy_client_cache().read() {
        Ok(guard) => guard
            .get(cache_key)
            .filter(|entry| entry.generation == generation)
            .map(|entry| entry.client.clone()),
        Err(poisoned) => poisoned
            .into_inner()
            .get(cache_key)
            .filter(|entry| entry.generation == generation)
            .map(|entry| entry.client.clone()),
    }
}

pub(crate) fn set_runtime_proxy_cached_client(cache_key: String, generation: u64, client: reqwest::Client) {
    if generation != runtime_proxy_current_generation() {
        return;
    }

    match runtime_proxy_client_cache().write() {
        Ok(mut guard) => {
            if generation == runtime_proxy_config_generation() {
                guard.insert(cache_key, RuntimeProxyCachedClient { generation, client });
            }
        }
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            if generation == runtime_proxy_config_generation() {
                guard.insert(cache_key, RuntimeProxyCachedClient { generation, client });
            }
        }
    }
}

pub fn set_runtime_proxy_config(config: ProxyConfig) {
    match runtime_proxy_state().write() {
        Ok(mut guard) => {
            *guard = config;
            bump_runtime_proxy_config_generation();
        }
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            *guard = config;
            bump_runtime_proxy_config_generation();
        }
    }

    clear_runtime_proxy_client_cache();
}

fn runtime_proxy_config_snapshot() -> (u64, ProxyConfig) {
    match runtime_proxy_state().read() {
        Ok(guard) => (runtime_proxy_config_generation(), guard.clone()),
        Err(poisoned) => {
            let guard = poisoned.into_inner();
            (runtime_proxy_config_generation(), guard.clone())
        }
    }
}

pub fn runtime_proxy_config() -> ProxyConfig {
    runtime_proxy_config_snapshot().1
}

pub fn apply_runtime_proxy_to_builder(
    builder: reqwest::ClientBuilder,
    service_key: &str,
) -> reqwest::ClientBuilder {
    runtime_proxy_config().apply_to_reqwest_builder(builder, service_key)
}

pub fn build_runtime_proxy_client(service_key: &str) -> reqwest::Client {
    let cache_key = runtime_proxy_cache_key(service_key, None, None);
    if let Some(client) = runtime_proxy_cached_client(&cache_key) {
        return client;
    }

    let (generation, config) = runtime_proxy_config_snapshot();
    let builder = config.apply_to_reqwest_builder(reqwest::Client::builder(), service_key);
    let client = builder.build().unwrap_or_else(|error| {
        ::omnesagent_log::record!(
            WARN,
            ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                .with_attrs(
                    ::serde_json::json!({"service_key": service_key, "error": format!("{}", error)})
                ),
            "Failed to build proxied client: "
        );
        reqwest::Client::new()
    });
    set_runtime_proxy_cached_client(cache_key, generation, client.clone());
    client
}

pub fn build_runtime_proxy_client_with_timeouts(
    service_key: &str,
    timeout_secs: u64,
    connect_timeout_secs: u64,
) -> reqwest::Client {
    let cache_key =
        runtime_proxy_cache_key(service_key, Some(timeout_secs), Some(connect_timeout_secs));
    if let Some(client) = runtime_proxy_cached_client(&cache_key) {
        return client;
    }

    let (generation, config) = runtime_proxy_config_snapshot();
    let builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .connect_timeout(std::time::Duration::from_secs(connect_timeout_secs));
    let builder = config.apply_to_reqwest_builder(builder, service_key);
    let client = builder.build().unwrap_or_else(|error| {
        ::omnesagent_log::record!(
            WARN,
            ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                .with_attrs(
                    ::serde_json::json!({"service_key": service_key, "error": format!("{}", error)})
                ),
            "Failed to build proxied timeout client: "
        );
        reqwest::Client::new()
    });
    set_runtime_proxy_cached_client(cache_key, generation, client.clone());
    client
}

pub fn build_runtime_proxy_client_with_read_timeout(
    service_key: &str,
    read_timeout_secs: u64,
    connect_timeout_secs: u64,
) -> reqwest::Client {
    let cache_key = format!(
        "{}|timeout=none|connect_timeout={}|read_timeout={}",
        service_key.trim().to_ascii_lowercase(),
        connect_timeout_secs,
        read_timeout_secs,
    );
    if let Some(client) = runtime_proxy_cached_client(&cache_key) {
        return client;
    }

    let (generation, config) = runtime_proxy_config_snapshot();
    let builder = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(connect_timeout_secs))
        .read_timeout(std::time::Duration::from_secs(read_timeout_secs));
    let builder = config.apply_to_reqwest_builder(builder, service_key);
    let client = builder.build().unwrap_or_else(|error| {
        ::omnesagent_log::record!(
            WARN,
            ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                .with_attrs(
                    ::serde_json::json!({"service_key": service_key, "error": format!("{}", error)})
                ),
            "Failed to build proxied read-timeout client: "
        );
        reqwest::Client::new()
    });
    set_runtime_proxy_cached_client(cache_key, generation, client.clone());
    client
}

/// Build an HTTP client for a channel, using an explicit per-channel proxy URL
/// when configured. Falls back to the global runtime proxy when `proxy_url` is
/// `None` or empty.
pub fn build_channel_proxy_client(service_key: &str, proxy_url: Option<&str>) -> reqwest::Client {
    match normalize_proxy_url_option(proxy_url) {
        Some(url) => build_explicit_proxy_client(service_key, &url, None, None),
        None => build_runtime_proxy_client(service_key),
    }
}

/// Build an HTTP client for a channel with custom timeouts, using an explicit
/// per-channel proxy URL when configured. Falls back to the global runtime
/// proxy when `proxy_url` is `None` or empty.
pub fn build_channel_proxy_client_with_timeouts(
    service_key: &str,
    proxy_url: Option<&str>,
    timeout_secs: u64,
    connect_timeout_secs: u64,
) -> reqwest::Client {
    match normalize_proxy_url_option(proxy_url) {
        Some(url) => build_explicit_proxy_client(
            service_key,
            &url,
            Some(timeout_secs),
            Some(connect_timeout_secs),
        ),
        None => build_runtime_proxy_client_with_timeouts(
            service_key,
            timeout_secs,
            connect_timeout_secs,
        ),
    }
}

/// Apply an explicit proxy URL to a `reqwest::ClientBuilder`, returning the
/// modified builder. Used by channels that specify a per-channel `proxy_url`.
pub fn apply_channel_proxy_to_builder(
    builder: reqwest::ClientBuilder,
    service_key: &str,
    proxy_url: Option<&str>,
) -> reqwest::ClientBuilder {
    match normalize_proxy_url_option(proxy_url) {
        Some(url) => apply_explicit_proxy_to_builder(builder, service_key, &url),
        None => apply_runtime_proxy_to_builder(builder, service_key),
    }
}

/// Build a client with a single explicit proxy URL (http+https via `Proxy::all`).
fn build_explicit_proxy_client(
    service_key: &str,
    proxy_url: &str,
    timeout_secs: Option<u64>,
    connect_timeout_secs: Option<u64>,
) -> reqwest::Client {
    let cache_key = format!(
        "explicit|{}|{}|timeout={}|connect_timeout={}",
        service_key.trim().to_ascii_lowercase(),
        proxy_url,
        timeout_secs
            .map(|v| v.to_string())
            .unwrap_or_else(|| "none".to_string()),
        connect_timeout_secs
            .map(|v| v.to_string())
            .unwrap_or_else(|| "none".to_string()),
    );
    if let Some(client) = runtime_proxy_cached_client(&cache_key) {
        return client;
    }

    let generation = runtime_proxy_current_generation();
    let mut builder = reqwest::Client::builder();
    if let Some(t) = timeout_secs {
        builder = builder.timeout(std::time::Duration::from_secs(t));
    }
    if let Some(ct) = connect_timeout_secs {
        builder = builder.connect_timeout(std::time::Duration::from_secs(ct));
    }
    builder = apply_explicit_proxy_to_builder(builder, service_key, proxy_url);
    let client = builder.build().unwrap_or_else(|error| {
        ::omnesagent_log::record!(WARN, ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note).with_outcome(::omnesagent_log::EventOutcome::Unknown).with_attrs(::serde_json::json!({"service_key": service_key, "proxy_url": proxy_url, "error": format!("{}", error)})), "Failed to build channel proxy client: ");
        reqwest::Client::new()
    });
    set_runtime_proxy_cached_client(cache_key, generation, client.clone());
    client
}

/// Apply a single explicit proxy URL to a builder via `Proxy::all`.
fn apply_explicit_proxy_to_builder(
    mut builder: reqwest::ClientBuilder,
    service_key: &str,
    proxy_url: &str,
) -> reqwest::ClientBuilder {
    match reqwest::Proxy::all(proxy_url) {
        Ok(proxy) => {
            builder = builder.proxy(proxy);
        }
        Err(error) => {
            ::omnesagent_log::record!(WARN, ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note).with_outcome(::omnesagent_log::EventOutcome::Unknown).with_attrs(::serde_json::json!({"proxy_url": proxy_url, "service_key": service_key, "error": format!("{}", error)})), "Ignoring invalid channel proxy_url: ");
        }
    }
    builder
}

