//! `Config` construction and the config-file persistence machinery: the
//! inherent `impl Config` blocks, `impl Default for Config`, dirty-path
//! application, TOML editing, and OnePassword reference snapshots.
//!
//! Extracted from the `schema` module; the inherent impls stay effective
//! crate-wide and the public free functions are re-exported with `pub use`,
//! so every `crate::schema::...` path is unchanged.
//!
//! The directory-resolution helpers (`default_config_dir`,
//! `install_data_dir`, `resolve_config_dir_for_data`, ...) intentionally stay
//! in `schema` because `schema::v2` and the config tests reach them through
//! `super::` / module scope; this module reads them via `use super::*`.

use super::*;

impl Default for Config {
    fn default() -> Self {
        // `default_config_dir()` is the canonical resolution for "where does
        // an unspecified config live": it honors `OMNESAGENT_CONFIG_DIR`, then
        // a `HOME` env override, before falling back to `UserDirs`. Calling
        // it here, instead of duplicating a `UserDirs`-only computation,
        // means a `Config::default()` constructed under an isolated test or
        // deployment never resolves to the real machine's `~/.omnesagent`, and
        // so cannot become a save target pointing at an operator's populated
        // config.toml.
        let omnesagent_dir = default_config_dir().unwrap_or_else(|_| {
            let home =
                UserDirs::new().map_or_else(|| PathBuf::from("."), |u| u.home_dir().to_path_buf());
            home.join(".omnesagent")
        });

        Self {
            data_dir: install_data_dir(&omnesagent_dir),
            config_path: omnesagent_dir.join("config.toml"),
            env_overridden_paths: std::collections::HashSet::new(),
            pre_override_snapshots: std::collections::HashMap::new(),
            onepassword_reference_snapshots: std::collections::HashMap::new(),
            dirty_paths: std::collections::HashSet::new(),
            degraded_security: Vec::new(),
            degraded_sections: Vec::new(),
            retired_wati_config_sections: Vec::new(),
            retired_node_transport_config: false,
            schema_version: crate::migration::CURRENT_SCHEMA_VERSION,
            providers: crate::providers::Providers::default(),
            model_routes: Vec::new(),
            embedding_routes: Vec::new(),
            observability: ObservabilityConfig::default(),
            trust: crate::scattered_types::TrustConfig::default(),
            backup: BackupConfig::default(),
            data_retention: DataRetentionConfig::default(),
            cloud_ops: CloudOpsConfig::default(),
            conversational_ai: ConversationalAiConfig::default(),
            security: SecurityConfig::default(),
            security_ops: SecurityOpsConfig::default(),
            runtime: RuntimeConfig::default(),
            reliability: ReliabilityConfig::default(),
            scheduler: SchedulerConfig::default(),
            eval: crate::scattered_types::EvalHarnessConfig::default(),
            pacing: PacingConfig::default(),
            skills: SkillsConfig::default(),
            pipeline: PipelineConfig::default(),
            heartbeat: HeartbeatConfig::default(),
            ralph: crate::ralph::RalphConfig::default(),
            ob2h_bridge: crate::ralph::Ob2hBridgeConfig::default(),
            cron: HashMap::new(),
            acp: AcpConfig::default(),
            channels: ChannelsConfig::default(),
            memory: MemoryConfig::default(),
            storage: StorageConfig::default(),
            tunnel: TunnelConfig::default(),
            gateway: GatewayConfig::default(),
            a2a: crate::multi_agent::A2aServerSection::default(),
            wss: WssConfig::default(),
            relay: RelayConfig::default(),
            enroll: EnrollConfig::default(),
            composio: ComposioConfig::default(),
            microsoft365: Microsoft365Config::default(),
            secrets: SecretsConfig::default(),
            browser: BrowserConfig::default(),
            browser_delegate: crate::scattered_types::BrowserDelegateConfig::default(),
            http_request: HttpRequestConfig::default(),
            multimodal: MultimodalConfig::default(),
            media_pipeline: MediaPipelineConfig::default(),
            web_fetch: WebFetchConfig::default(),
            link_enricher: LinkEnricherConfig::default(),
            text_browser: TextBrowserConfig::default(),
            web_search: WebSearchConfig::default(),
            project_intel: ProjectIntelConfig::default(),
            google_workspace: GoogleWorkspaceConfig::default(),
            proxy: ProxyConfig::default(),
            cost: CostConfig::default(),
            peripherals: PeripheralsConfig::default(),
            delegate: DelegateToolConfig::default(),
            agents: HashMap::new(),
            risk_profiles: HashMap::new(),
            runtime_profiles: HashMap::new(),
            skill_bundles: HashMap::new(),
            knowledge_bundles: HashMap::new(),
            mcp_bundles: HashMap::new(),
            peer_groups: HashMap::new(),
            hooks: HooksConfig::default(),
            hardware: HardwareConfig::default(),
            query_classification: QueryClassificationConfig::default(),
            transcription: TranscriptionConfig::default(),
            tts: TtsConfig::default(),
            mcp: McpConfig::default(),
            nodes: NodesConfig::default(),
            onboard_state: OnboardStateConfig::default(),
            notion: NotionConfig::default(),
            jira: JiraConfig::default(),
            knowledge: KnowledgeConfig::default(),
            linkedin: LinkedInConfig::default(),
            image_gen: ImageGenConfig::default(),
            file_upload: FileUploadConfig::default(),
            file_upload_bundle: FileUploadBundleConfig::default(),
            file_download: FileDownloadConfig::default(),
            plugins: PluginsConfig::default(),
            locale: None,
            verifiable_intent: VerifiableIntentConfig::default(),
            claude_code: ClaudeCodeConfig::default(),
            claude_code_runner: ClaudeCodeRunnerConfig::default(),
            codex_cli: CodexCliConfig::default(),
            gemini_cli: GeminiCliConfig::default(),
            opencode_cli: OpenCodeCliConfig::default(),
            sop: SopConfig::default(),
            shell_tool: ShellToolConfig::default(),
            escalation: EscalationConfig::default(),
            voice: VoiceConfig::default(),
        }
    }
}

pub(super) fn config_dir_creation_error(path: &Path) -> String {
    format!(
        "Failed to create config directory: {}. If running as an OpenRC service, \
         ensure this path is writable by user 'omnesagent'.",
        path.display()
    )
}

/// Top-level keys that must always appear in the saved config even
/// when their value equals the default. `schema_version` is the
/// migration detector's anchor — dropping it from a freshly-saved
/// config would make the next load mis-detect the file as V1 (no
/// version key = V1).
const SAVE_PRESERVE_KEYS: &[&str] = &["schema_version"];

/// Insert a blank line before every `[section]` header that doesn't
/// already have one, so the serialized TOML reads as discrete blocks
/// instead of running every section header directly after the
/// previous line (`toml::to_string_pretty` doesn't gap between a
/// trailing scalar and the next section header).
fn ensure_blank_line_before_sections(toml: &str) -> String {
    let mut out = String::with_capacity(toml.len() + 64);
    let mut prev_line_blank = true; // start of file counts as blank
    for line in toml.lines() {
        let is_section_header = line.starts_with('[');
        if is_section_header && !prev_line_blank {
            out.push('\n');
        }
        out.push_str(line);
        out.push('\n');
        prev_line_blank = line.trim().is_empty();
    }
    out
}

/// Walk `actual` and drop every key whose value matches the same
/// key's value in `defaults`. Tables recurse; the recursion drops a
/// sub-table when every one of its keys was itself dropped (i.e. the
/// sub-table contained only defaults). Keys that don't appear in
/// `defaults` are operator-added and always survive.
///
/// HashMap-keyed sub-trees (e.g. `agents`, `providers.models.<family>`)
/// are not in the typed default tree, so their operator-added aliases
/// pass through this filter unchanged.
fn prune_default_values(actual: &mut toml::Table, defaults: &toml::Table) {
    let keys: Vec<String> = actual.keys().cloned().collect();
    for key in keys {
        if SAVE_PRESERVE_KEYS.contains(&key.as_str()) {
            continue;
        }
        let Some(default_value) = defaults.get(&key) else {
            // Operator added this key; not in the typed default tree.
            // Always keep — recursing in would either be a no-op or
            // strip operator content.
            continue;
        };
        let Some(child) = actual.remove(&key) else {
            continue;
        };
        let pruned = match (child, default_value) {
            (toml::Value::Table(mut child_table), toml::Value::Table(default_subtable)) => {
                prune_default_values(&mut child_table, default_subtable);
                if child_table.is_empty() {
                    None
                } else {
                    Some(toml::Value::Table(child_table))
                }
            }
            (child, default_value) => {
                if &child == default_value {
                    None
                } else {
                    Some(child)
                }
            }
        };
        if let Some(value) = pruned {
            actual.insert(key, value);
        }
    }
}

fn is_local_ollama_endpoint(api_url: Option<&str>) -> bool {
    let Some(raw) = api_url.map(str::trim).filter(|value| !value.is_empty()) else {
        return true;
    };

    reqwest::Url::parse(raw)
        .ok()
        .and_then(|url| url.host_str().map(|host| host.to_ascii_lowercase()))
        .is_some_and(|host| matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1" | "0.0.0.0"))
}

fn is_official_ollama_cloud_endpoint(api_url: Option<&str>) -> bool {
    let Some(raw) = api_url.map(str::trim).filter(|value| !value.is_empty()) else {
        return false;
    };

    reqwest::Url::parse(raw)
        .ok()
        .and_then(|url| {
            url.host_str().map(|host| {
                host.eq_ignore_ascii_case("ollama.com")
                    || host.eq_ignore_ascii_case("api.ollama.com")
            })
        })
        .unwrap_or(false)
}

fn has_ollama_cloud_credential(config_api_key: Option<&str>) -> bool {
    config_api_key
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
}

/// Ensure that essential bootstrap files exist in the workspace directory.
///
/// When the workspace is created outside of Quickstart (e.g., non-tty
/// daemon/cron sessions), these files would otherwise be missing. This function
/// creates sensible defaults that allow the agent to operate with a basic identity.
pub async fn ensure_bootstrap_files(workspace_dir: &Path) -> Result<()> {
    let defaults: &[(&str, &str)] = &[
        (
            "IDENTITY.md",
            "# IDENTITY.md — Who Am I?\n\n\
             I am OmnesAgent, an autonomous AI agent.\n\n\
             ## Traits\n\
             - Helpful, precise, and safety-conscious\n\
             - I prioritize clarity and correctness\n",
        ),
        (
            "SOUL.md",
            "# SOUL.md — Who You Are\n\n\
             You are OmnesAgent, an autonomous AI agent.\n\n\
             ## Core Principles\n\
             - Be helpful and accurate\n\
             - Respect user intent and boundaries\n\
             - Ask before taking destructive actions\n\
             - Prefer safe, reversible operations\n",
        ),
    ];

    for (filename, content) in defaults {
        let path = workspace_dir.join(filename);
        if !path.exists() {
            fs::write(&path, content)
                .await
                .with_context(|| format!("Failed to create default {filename} in workspace"))?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ExtraNestedModelProviderTable {
    pub(super) family: String,
    pub(super) alias: String,
    pub(super) nested: String,
}

/// Classification of a `peer_groups.<name>.channel` reference against the
/// configured `[channels.*]` blocks. This is the single source of truth for
/// resolving a raw `channel` string — `Config::validate()` consumes it for
/// its hard-error checks, and `Config::collect_peer_groups_warnings`
/// consumes it for the non-fatal dangling-alias warning. Do not duplicate
/// this resolution logic elsewhere; classify the ref once and match on the
/// result.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PeerGroupChannelRef {
    /// Empty or whitespace-only `channel` value.
    Empty,
    /// A bare channel type with no dotted alias (e.g. `"discord"`), which
    /// authorizes every alias of that type.
    BareType {
        channel_type: String,
        /// Whether `[channels.<channel_type>.*]` has at least one entry.
        configured: bool,
    },
    /// A dotted `type.alias` reference (e.g. `"discord.work"`).
    Dotted {
        channel_type: String,
        alias: String,
        /// Whether `[channels.<channel_type>.*]` is configured at all.
        type_configured: bool,
        /// Whether `alias` is among the configured aliases for
        /// `channel_type`. Always `false` when `type_configured` is `false`.
        exists: bool,
        /// The configured aliases for `channel_type` (empty when the type
        /// itself is unconfigured), sorted lexicographically so consumers
        /// that pick between candidates are deterministic.
        known_aliases: Vec<String>,
    },
}

impl Config {
    /// External-peer usernames authorized on `<channel_type>.<alias>`.
    ///
    /// A `[peer_groups.<name>]` contributes when its `channel` field either
    /// matches `channel_type` (type-wide group, applies to every alias of
    /// that type) or matches the full dotted `"<channel_type>.<alias>"`
    /// (instance-scoped group, applies to that one alias only).
    pub fn channel_external_peers(&self, channel_type: &str, alias: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for group in self.peer_groups.values() {
            let group_matches = match group.channel.split_once('.') {
                Some((ty, al)) => ty == channel_type && al == alias,
                None => group.channel == channel_type,
            };
            if !group_matches {
                continue;
            }
            for peer in &group.external_peers {
                let username = peer.as_str().to_string();
                if seen.insert(username.clone()) {
                    out.push(username);
                }
            }
        }
        out
    }

    /// Voice-peer usernames for `<channel_type>.<alias>` that should always
    /// receive TTS voice replies.
    ///
    /// A `[peer_groups.<name>]` contributes when its `channel` field matches
    /// (type-wide or dotted) **and** `output_modality = "voice"`.
    ///
    /// This is the live-resolve counterpart of `channel_external_peers`,
    /// filtered to voice-only peer groups. No cache — single source of truth
    /// is `self.peer_groups`.
    pub fn channel_voice_peers(&self, channel_type: &str, alias: &str) -> Vec<String> {
        use crate::multi_agent::OutputModality;

        let mut out: Vec<String> = Vec::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for group in self.peer_groups.values() {
            if group.output_modality != OutputModality::Voice {
                continue;
            }
            let group_matches = match group.channel.split_once('.') {
                Some((ty, al)) => ty == channel_type && al == alias,
                None => group.channel == channel_type,
            };
            if !group_matches {
                continue;
            }
            for peer in &group.external_peers {
                let username = peer.as_str().to_string();
                if seen.insert(username.clone()) {
                    out.push(username);
                }
            }
        }
        out
    }

    /// Sender usernames authorized to issue `/model --agent <model>` on
    /// `<channel_type>.<alias>` for agent `agent_alias`. A
    /// `[peer_groups.<name>]` contributes when its `channel` matches
    /// (type-wide or dotted), its `admin_for_agent_scope = true`, and its
    /// `agents` list (if non-empty) contains `agent_alias`. Returns
    /// deduped and sorted. Live-resolve against the `Config` this method
    /// is called on — no internal cache. Edits to `peer_groups` take
    /// effect at the next `Config::channel_agent_scope_admins` call; the
    /// orchestrator gate reads through a snapshot of this `Config`, so
    /// operator edits become visible after the runtime context is rebuilt
    /// (daemon restart).
    pub fn channel_agent_scope_admins(
        &self,
        channel_type: &str,
        alias: &str,
        agent_alias: &str,
    ) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for group in self.peer_groups.values() {
            if !group.admin_for_agent_scope {
                continue;
            }
            let group_matches = match group.channel.split_once('.') {
                Some((ty, al)) => ty == channel_type && al == alias,
                None => group.channel == channel_type,
            };
            if !group_matches {
                continue;
            }
            // Agent-bound: when the group's `agents` list is non-empty,
            // only grant the privilege if `agent_alias` appears in it.
            // An empty `agents` list is interpreted as "all agents"
            // (backward-compatible channel-wide default).
            if !group.agents.is_empty() && !group.agents.iter().any(|a| a.as_str() == agent_alias) {
                continue;
            }
            for peer in &group.external_peers {
                let username = peer.as_str().to_string();
                if seen.insert(username.clone()) {
                    out.push(username);
                }
            }
        }
        out.sort();
        out
    }

    /// Collect the `IntegrationDescriptor` from every nested config that
    /// declares one via `#[integration(...)]`. Adding a new toggleable
    /// integration is one struct-level attribute on the new config + one
    /// row in this method. The integrations registry consumes the result
    /// without per-vendor branches.
    pub fn integration_descriptors(&self) -> Vec<crate::config::IntegrationDescriptor> {
        // BrowserConfig and GoogleWorkspaceConfig carry
        // `#[integration(...)]` annotations on V3, so the macro emits
        // `integration_descriptor()` on each. Cron has been flattened
        // to `HashMap<String, CronJobDecl>` with no enable toggle, so
        // it gets a hand-crafted descriptor whose `active` reflects
        // whether any job is configured. Display copy lives next to
        // the field so the registry never branches on a category name.
        vec![
            self.browser.integration_descriptor(),
            self.google_workspace.integration_descriptor(),
            crate::config::IntegrationDescriptor {
                display_name: "Cron",
                description: "Scheduled tasks",
                category: "ToolsAutomation",
                active: !self.cron.is_empty(),
            },
        ]
    }

    /// Return top-level TOML keys in `raw_toml` that Config does not recognise.
    ///
    /// Keys present in `Config::default()` serialization pass immediately.
    /// Remaining keys are probed: the key is deserialized in isolation and
    /// the result compared to the default — a changed output means serde
    /// consumed it (covers `Option<T>` fields and `#[serde(alias)]` names).
    /// V1 legacy keys (consumed by migration) are also accepted.
    pub fn unknown_keys(raw_toml: &str) -> Vec<String> {
        let raw: toml::Table = match raw_toml.parse() {
            Ok(t) => t,
            Err(_) => return Vec::new(),
        };
        static DEFAULTS: OnceLock<toml::Table> = OnceLock::new();
        let defaults = DEFAULTS.get_or_init(|| {
            toml::to_string(&Config::default())
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_default()
        });
        raw.keys()
            .filter(|key| {
                if defaults.contains_key(key.as_str()) {
                    return false;
                }
                if crate::migration::V1_LEGACY_KEYS.contains(&key.as_str()) {
                    return false;
                }
                if key.as_str() == "node_transport" {
                    return false;
                }
                let mut t = toml::Table::new();
                t.insert((*key).clone(), raw[key.as_str()].clone());
                let consumed = toml::to_string(&t)
                    .ok()
                    .and_then(|s| toml::from_str::<Config>(&s).ok())
                    .and_then(|c| toml::to_string(&c).ok())
                    .and_then(|s| s.parse::<toml::Table>().ok())
                    .is_some_and(|t| t != *defaults);
                !consumed
            })
            .cloned()
            .collect()
    }

    /// Return retired WATI section roots before migration and typed
    /// deserialization erase them. V1 used `[channels_config.wati]`; V2/V3
    /// channel aliases use `[channels.wati.<alias>]`.
    pub(super) fn retired_wati_config_sections(raw_toml: &str) -> Vec<String> {
        let raw: toml::Table = match raw_toml.parse() {
            Ok(t) => t,
            Err(_) => return Vec::new(),
        };

        ["channels", "channels_config"]
            .into_iter()
            .filter(|root| {
                raw.get(*root)
                    .and_then(toml::Value::as_table)
                    .is_some_and(|channels| channels.contains_key("wati"))
            })
            .map(|root| format!("{root}.wati"))
            .collect()
    }

    /// Detect the retired top-level transport section without retaining any
    /// of its values, including `shared_secret`.
    pub(super) fn has_retired_node_transport_config(raw_toml: &str) -> bool {
        raw_toml
            .parse::<toml::Table>()
            .ok()
            .is_some_and(|raw| raw.contains_key("node_transport"))
    }

    /// Return `<kind>.<family>` entries under `[providers]` in `raw_toml`
    /// whose family is not a known typed slot (kinds: models, tts,
    /// transcription). Serde silently drops these sections at deserialize
    /// time, so a typo'd family (`[providers.models.antropic.x]`) or one
    /// from a newer binary vanishes on reload: the alias "works" in the
    /// session that created it, then disappears after restart.
    pub fn unknown_provider_families(raw_toml: &str) -> Vec<String> {
        let raw: toml::Table = match raw_toml.parse() {
            Ok(t) => t,
            Err(_) => return Vec::new(),
        };
        let Some(kinds) = raw.get("providers").and_then(toml::Value::as_table) else {
            return Vec::new();
        };
        let kind_slots: &[(&str, &[&str])] = &[
            ("models", crate::providers::ModelProviders::slot_names()),
            ("tts", crate::providers::TtsProviders::slot_names()),
            (
                "transcription",
                crate::providers::TranscriptionProviders::slot_names(),
            ),
        ];
        let mut out = Vec::new();
        for (kind, slots) in kind_slots {
            let Some(families) = kinds.get(*kind).and_then(toml::Value::as_table) else {
                continue;
            };
            out.extend(
                families
                    .keys()
                    .filter(|k| !slots.contains(&k.as_str()))
                    .map(|k| format!("{kind}.{k}")),
            );
        }
        out
    }

    /// Return extra-nested model-provider alias tables that look like a
    /// misplaced provider config, e.g.
    /// `[providers.models.zai.default.default]`.
    ///
    /// Serde treats the first `default` as the alias and silently ignores the
    /// second `default` child table. When that child table carries provider
    /// fields such as `model`, `api_key`, or family-specific fields, the
    /// operator's settings are dropped while the empty alias still validates as
    /// an in-progress provider. This raw-TOML detector runs before that shape
    /// is erased by typed deserialization.
    pub(super) fn extra_nested_model_provider_tables(
        raw_toml: &str,
    ) -> Vec<ExtraNestedModelProviderTable> {
        let raw: toml::Table = match raw_toml.parse() {
            Ok(t) => t,
            Err(_) => return Vec::new(),
        };
        let Some(models) = raw
            .get("providers")
            .and_then(toml::Value::as_table)
            .and_then(|providers| providers.get("models"))
            .and_then(toml::Value::as_table)
        else {
            return Vec::new();
        };

        let mut out = Vec::new();
        for (family, aliases_value) in models {
            if !crate::providers::ModelProviders::slot_names().contains(&family.as_str()) {
                continue;
            }
            let Some(aliases) = aliases_value.as_table() else {
                continue;
            };
            for (alias, alias_value) in aliases {
                let Some(alias_table) = alias_value.as_table() else {
                    continue;
                };
                for (nested_key, nested_value) in alias_table {
                    let Some(nested_table) = nested_value.as_table() else {
                        continue;
                    };
                    if nested_table.is_empty()
                        || Self::is_model_provider_table_valued_field(nested_key)
                    {
                        continue;
                    }
                    out.push(ExtraNestedModelProviderTable {
                        family: family.clone(),
                        alias: alias.clone(),
                        nested: nested_key.clone(),
                    });
                }
            }
        }
        out.sort();
        out.dedup();
        out
    }

    fn is_model_provider_table_valued_field(key: &str) -> bool {
        // These shared model-provider fields legitimately deserialize from
        // nested TOML tables under an alias. Every other table-valued child of
        // `[providers.models.<family>.<alias>]` is an ignored extra section,
        // regardless of whether its contents are shared or family-specific
        // provider fields.
        matches!(
            key,
            "chat_template_kwargs" | "extra_headers" | "pricing" | "provider_extra"
        )
    }

    /// Returns `true` if `path` was populated by a `OMNESAGENT_*` env-var
    /// override at load time. O(1) HashSet lookup; safe to call per row in
    /// list-rendering paths (`config list`, dashboard, quickstart).
    pub fn prop_is_env_overridden(&self, path: &str) -> bool {
        self.env_overridden_paths.contains(path)
    }

    pub async fn load_or_init() -> Result<Self> {
        let (default_omnesagent_dir, default_workspace_dir) = default_config_and_data_dirs()?;

        // Resolve env overrides FIRST so the migration runs against
        // the install root the operator actually uses. Running the
        // migration against `default_omnesagent_dir` would silently skip
        // any install reached via `OMNESAGENT_CONFIG_DIR` or
        // `OMNESAGENT_WORKSPACE`.
        let (omnesagent_dir, _data_dir, resolution_source) =
            resolve_runtime_config_dirs(&default_omnesagent_dir, &default_workspace_dir).await?;

        // One-time, V<3 → V3 ONLY move of `<install>/workspace/` into
        // `<install>/agents/default/workspace/`. The "default" alias is
        // the migration bridge — it must NEVER appear on a fresh install
        // or on a V3 install that already declared its own aliases.
        //
        // Gate strictly on the on-disk config's `schema_version`:
        // - missing config.toml → fresh install, skip.
        // - schema_version >= 3 → already V3, skip.
        // - schema_version 1 or 2 → upgrade in progress, run.
        // Anything else (parse failure, weird value) is treated as
        // "don't touch the filesystem"; the TOML migrator will surface
        // the real error.
        let config_toml_path = omnesagent_dir.join("config.toml");
        let needs_fs_migration = config_toml_path.is_file()
            && matches!(
                std::fs::read_to_string(&config_toml_path)
                    .ok()
                    .and_then(|raw| toml::from_str::<toml::Value>(&raw).ok())
                    .and_then(|v| crate::migration::detect_version(&v).ok()),
                Some(v) if v < crate::migration::CURRENT_SCHEMA_VERSION
            );
        if needs_fs_migration
            && let Err(e) = crate::schema::v2::migrate_v2_to_v3_install_filesystem(&omnesagent_dir)
        {
            ::omnesagent_log::record!(
                WARN,
                ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                    .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                    .with_attrs(::serde_json::json!({
                        "install": omnesagent_dir.display().to_string(),
                        "error": format!("{}", e),
                    })),
                "[system] filesystem migration failed; continuing with legacy layout"
            );
        } else if !needs_fs_migration
            && let Err(e) =
                crate::schema::v2::relocate_default_agent_skills_to_shared(&omnesagent_dir)
        {
            ::omnesagent_log::record!(
                WARN,
                ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                    .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                    .with_attrs(::serde_json::json!({
                        "install": omnesagent_dir.display().to_string(),
                        "error": format!("{}", e),
                    })),
                "[system] skills relocation to shared workspace failed; continuing"
            );
        }

        let config_path = omnesagent_dir.join("config.toml");

        // The install dir is the only directory `load_or_init` creates
        // unconditionally. Per-agent workspaces (`agents/<alias>/workspace/`)
        // are seeded lazily at agent-loop entry by
        // `Agent::from_config_with_session_cwd_and_mcp`, which runs
        // `ensure_bootstrap_files` for the agent it is starting. There
        // is no fresh-install "default" agent and therefore no
        // `agents/default/workspace/` synthesized at boot; the only
        // legitimate origin for that directory is the V1/V2→V3
        // legacy-workspace migration above, which fires only when a
        // pre-multi-agent install's `<install>/workspace/` is present
        // and needs to be moved into the new layout.
        //
        // `config.data_dir` resolves to `<install>/data/` — the shared
        // instance data directory holding databases (memory, sessions,
        // cost records) and hygiene/state files. Per-agent identity
        // and markdown (MEMORY.md, IDENTITY.md, SOUL.md) lives at
        // `Config::agent_workspace_dir(alias)` instead.
        let data_dir = install_data_dir(&omnesagent_dir);
        fs::create_dir_all(&data_dir).await.with_context(|| {
            format!(
                "Failed to create data directory: {}",
                data_dir.display().to_string()
            )
        })?;
        // Legacy alias retained for clarity in the struct initializer
        // and existing field assignments below.
        let workspace_dir = data_dir;

        // `<install>/shared/` — root workspace shared across every agent
        // on the host. Holds skills, skill bundles, and other content
        // not scoped to a single agent. Per-agent state still lives at
        // `<install>/agents/<alias>/workspace/`.
        let shared_dir = omnesagent_dir.join("shared");
        fs::create_dir_all(&shared_dir).await.with_context(|| {
            format!(
                "Failed to create shared workspace directory: {}",
                shared_dir.display()
            )
        })?;

        fs::create_dir_all(&omnesagent_dir)
            .await
            .with_context(|| config_dir_creation_error(&omnesagent_dir))?;

        if config_path.exists() {
            // Warn if config file is world-readable (may contain API keys)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = fs::metadata(&config_path).await
                    && meta.permissions().mode() & 0o004 != 0
                {
                    ::omnesagent_log::record!(
                        WARN,
                        ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                            .with_outcome(::omnesagent_log::EventOutcome::Unknown),
                        &format!(
                            "Config file {:?} is world-readable (mode {:o}). \
                             Consider restricting with: chmod 600 {:?}",
                            config_path,
                            meta.permissions().mode() & 0o777,
                            config_path
                        )
                    );
                }
            }

            let contents = fs::read_to_string(&config_path)
                .await
                .context("Failed to read config file")?;

            let retired_wati_config_sections = Self::retired_wati_config_sections(&contents);
            for path in &retired_wati_config_sections {
                ::omnesagent_log::record!(
                    WARN,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                        .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                        .with_attrs(::serde_json::json!({
                            "channel": "wati",
                            "retired_config": path,
                        })),
                    &format!(
                        "Retired WATI channel config section `{path}` is ignored because WATI \
                         support was removed. Migrate to `[channels.whatsapp.<alias>]` using \
                         the Cloud API or WhatsApp Web, then revoke the unused WATI API token."
                    )
                );
            }
            let retired_node_transport_config = Self::has_retired_node_transport_config(&contents);
            if retired_node_transport_config {
                ::omnesagent_log::record!(
                    WARN,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                        .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                        .with_attrs(::serde_json::json!({
                            "retired_config": "node_transport",
                        })),
                    "Retired `[node_transport]` config is ignored because the legacy HMAC node \
                     transport was removed. Delete the section from config.toml."
                );
            }

            // Deserialize the config with the standard TOML parser.
            //
            // Previously this used `serde_ignored::deserialize` for both
            // deserialization and unknown-key detection. However,
            // `serde_ignored` silently drops field values inside nested
            // structs that carry `#[serde(default)]` (e.g. the entire
            // `[autonomy]` table), causing user-supplied values to be
            // replaced by defaults.
            //
            // We now deserialize with `toml::from_str` (which is correct)
            // and run `serde_ignored` separately just for diagnostics.
            //
            // `migrate_to_current` parses the TOML, detects the schema
            // version, runs the typed V1→V2→V3 chain via `V1Config::migrate`
            // / `V2Config::migrate`, and deserializes the result into the
            // current `Config` shape.
            //
            // Detect the on-disk version up-front so we can emit one WARN
            // line when the daemon auto-migrates an older config in memory:
            // the disk file is left untouched and the user is advised to lock
            // the migration in with `omnesagent config migrate`.
            let stale_version = toml::from_str::<toml::Value>(&contents)
                .ok()
                .as_ref()
                .and_then(|v| crate::migration::detect_version(v).ok())
                .filter(|n| *n != crate::migration::CURRENT_SCHEMA_VERSION);
            // Daemon load must never hard-fail on a malformed config — the
            // operator needs the process up to repair it. The resilient path
            // degrades (dropping invalid blocks to defaults); security-critical
            // drops are recorded on `degraded_security` for exposure gating.
            // Strict validation lives in `omnesagent config migrate`.
            let salvage = crate::migration::migrate_to_current_salvaged(&contents);
            let mut config: Config = salvage.config;
            config.degraded_security = salvage.dropped_security;
            config.degraded_sections = salvage.dropped;
            config.retired_wati_config_sections = retired_wati_config_sections;
            config.retired_node_transport_config = retired_node_transport_config;
            if let Some(from_version) = stale_version {
                ::omnesagent_log::record!(
                    WARN,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                        .with_outcome(::omnesagent_log::EventOutcome::Unknown),
                    &format!(
                        "Config at {} is schema_version {from_version}; auto-migrated to {} in memory. \
                     Run `omnesagent config migrate` to commit the migration to disk. \
                     V0.8.0 also replaced the env-var override grammar; see \
                     https://github.com/omnesagent-labs/omnesagent/blob/master/docs/book/src/reference/env-vars.md \
                     for the migration recipes.",
                        config_path.display().to_string(),
                        crate::migration::CURRENT_SCHEMA_VERSION
                    )
                );
            }

            // Ensure the built-in default auto_approve entries are always
            // present on a `risk_profiles.default` entry that already
            // exists (typically post-V1/V2→V3 migration). When a user
            // specifies `auto_approve` in their TOML (e.g. to add a
            // custom tool), serde replaces the default list instead of
            // merging — this re-adds the framework defaults so safe
            // tools like `weather` and `calculator` keep their
            // auto-approve status.
            //
            // Users who want to require approval for a default tool can
            // add it to `always_ask`, which takes precedence over
            // `auto_approve` in the approval decision (see approval/mod.rs).
            //
            // Skipped when the loaded config has no `risk_profiles.default`
            // entry: we will not synthesize a `default` alias here.
            // `default` is a migration artifact (V1/V2→V3
            // single-instance bridge); a config that arrives without it
            // is a legitimate multi-aliased shape and must not have one
            // injected at load time.
            if let Some(default_profile) = config.risk_profiles.get_mut("default") {
                default_profile.ensure_default_auto_approve();
            }

            // Detect unknown top-level config keys by comparing the raw
            // TOML table keys against what Config actually deserializes.
            // This replaces the previous serde_ignored-based approach which
            // had false-positive issues with #[serde(default)] nested structs.
            for key in Self::unknown_keys(&contents) {
                ::omnesagent_log::record!(
                    WARN,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                        .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                        .with_attrs(::serde_json::json!({"key": key})),
                    "Unknown config key ignored: \"\". Check config.toml for typos or deprecated options."
                );
            }
            // Unknown provider families are dropped by serde, so an alias
            // created under a typo'd or unsupported family silently vanishes
            // on reload while agents.*.model_provider still references it.
            for entry in Self::unknown_provider_families(&contents) {
                let (kind, family) = entry.split_once('.').unwrap_or(("models", entry.as_str()));
                let reference = if kind == "models" {
                    "any agents.*.model_provider referencing them will fail to resolve; \
                     run `omnesagent providers` for valid family names"
                } else {
                    "references to its aliases will fail to resolve"
                };
                ::omnesagent_log::record!(
                    WARN,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                        .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                        .with_attrs(::serde_json::json!({"kind": kind, "family": family})),
                    &format!(
                        "[providers.{kind}.{family}] section dropped: not a known {kind} \
                        provider family. Its aliases will not load and {reference}."
                    )
                );
            }
            // Extra child tables under a known model-provider alias are also
            // erased by serde. If the child carries provider-shaped fields,
            // call out the likely mistaken nesting instead of letting the
            // parent alias look like an ordinary unfinished provider.
            for entry in Self::extra_nested_model_provider_tables(&contents) {
                let family = entry.family.as_str();
                let alias = entry.alias.as_str();
                let nested = entry.nested.as_str();
                ::omnesagent_log::record!(
                    WARN,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                        .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                        .with_attrs(::serde_json::json!({
                            "family": family,
                            "alias": alias,
                            "nested": nested,
                        })),
                    &format!(
                        "[providers.models.{}.{}.{}] section ignored: \
                         model-provider fields must live directly under \
                         [providers.models.{}.{}]. Move the nested keys up one level.",
                        family, alias, nested, family, alias
                    )
                );
            }
            // Set computed paths that are skipped during serialization
            config.config_path = config_path.clone();
            config.data_dir = workspace_dir;

            // Ensure each configured skill-bundle's resolved directory
            // exists on disk so the bundle has somewhere for skills to
            // land immediately. Idempotent.
            let install_root = config.install_root_dir();
            for alias in config.skill_bundles.keys().cloned().collect::<Vec<_>>() {
                if let Ok(dir) =
                    crate::skill_bundles::resolve_directory(&config, &install_root, &alias)
                    && let Err(e) = std::fs::create_dir_all(&dir)
                {
                    ::omnesagent_log::record!(
                        WARN,
                        ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                            .with_outcome(::omnesagent_log::EventOutcome::Unknown),
                        &format!(
                            "skill-bundle '{alias}' directory creation failed at {}: {e}",
                            dir.display().to_string()
                        )
                    );
                }
            }

            let store = crate::secrets::SecretStore::new(&omnesagent_dir, config.secrets.encrypt);
            config.onepassword_reference_snapshots =
                collect_onepassword_reference_snapshots(&config);
            // Decrypt all #[secret]-annotated fields via Configurable derive
            config = tokio::task::spawn_blocking(move || {
                config.decrypt_secrets(&store)?;
                Ok::<_, anyhow::Error>(config)
            })
            .await
            .context("Config secret decryption task failed")??;

            // Apply OMNESAGENT_<lowercase_path> env-var overrides. Hard-errors
            // on any unresolvable path — no silent ignores. Tracks overridden
            // paths and per-path pre-override snapshots so save() can mask
            // env-injected values back to the original on-disk state.
            let applied = crate::env_overrides::apply_env_overrides(&mut config)?;
            config.env_overridden_paths = applied.paths;
            config.pre_override_snapshots = applied.snapshots;

            // Validation must NOT prevent the daemon from booting. If
            // it did, a single broken agent reference would lock the
            // operator out of `/config` — the only place they can fix
            // it. Demote to a startup warning; the gateway and dashboard
            // still come up so the user can navigate to the bad section
            // and repair it.
            if let Err(e) = config.validate() {
                ::omnesagent_log::record!(
                    WARN,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                        .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                        .with_attrs(::serde_json::json!({"error": format!("{e:#}")})),
                    "[system] config has validation errors — booting anyway so you \
                     can fix them via /config or `omnesagent config set`"
                );
            }
            // Publish the effective post-decryption, post-env-override proxy
            // configuration before any runtime client or tool is constructed.
            // Interactive proxy_config updates use the same live state.
            set_runtime_proxy_config(config.proxy.clone());
            ::omnesagent_log::record!(INFO, ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note).with_attrs(::serde_json::json!({"path": config.config_path.display().to_string(), "workspace": config.data_dir.display().to_string(), "source": resolution_source.as_str(), "initialized": true})), "Config loaded");
            Ok(config)
        } else {
            let mut config = Config {
                config_path: config_path.clone(),
                data_dir: workspace_dir,
                ..Config::default()
            };
            // Save defaults FIRST so env-injected values never reach the
            // freshly-created config file. Env overrides apply post-save to
            // populate the in-memory Config for the running process.
            config.save().await?;

            // Restrict permissions on newly created config file (may contain API keys)
            #[cfg(unix)]
            {
                use std::{fs::Permissions, os::unix::fs::PermissionsExt};
                let _ = fs::set_permissions(&config_path, Permissions::from_mode(0o600)).await;
            }

            let applied = crate::env_overrides::apply_env_overrides(&mut config)?;
            config.env_overridden_paths = applied.paths;
            config.pre_override_snapshots = applied.snapshots;

            // Same boot-resilience as the load-existing branch above:
            // a fresh-init config can't realistically fail validation,
            // but if it does we still want the daemon up.
            if let Err(e) = config.validate() {
                ::omnesagent_log::record!(
                    WARN,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                        .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                        .with_attrs(::serde_json::json!({"error": format!("{e:#}")})),
                    "[system] freshly-initialized config has validation errors — \
                     booting anyway so you can fix them via /config"
                );
            }
            set_runtime_proxy_config(config.proxy.clone());
            ::omnesagent_log::record!(INFO, ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note).with_attrs(::serde_json::json!({"path": config.config_path.display().to_string(), "workspace": config.data_dir.display().to_string(), "source": resolution_source.as_str(), "initialized": true})), "Config loaded");
            Ok(config)
        }
    }

    /// Report that opting into `[verifiable_intent]` does not currently enable
    /// credential verification, because `vi_verify` is withheld from the
    /// model-visible registry until a chain verifier exists.
    ///
    /// The runtime already traces this at config load. That trace reaches a
    /// sink only when log persistence is on, so under
    /// `observability.log_persistence = "none"` it is delivered nowhere. This
    /// warning is the channel that survives: `omnesagent doctor` prints the
    /// structured list to stdout and the config API returns it in its
    /// response, neither of which depends on the log writer.
    ///
    /// Same class as `memory_config_knob_inert` — a knob that is set, accepted,
    /// and currently has no runtime consumer.
    ///
    /// The change that re-registers `vi_verify` must delete this check and the
    /// runtime trace together, or an operator is told the capability is
    /// unavailable while the model is calling it.
    fn collect_verifiable_intent_warnings(
        &self,
        warnings: &mut Vec<crate::validation_warnings::ValidationWarning>,
    ) {
        if !self.verifiable_intent.enabled {
            return;
        }
        warnings.push(crate::validation_warnings::ValidationWarning::new(
            crate::validation_warnings::VERIFIABLE_INTENT_TOOL_WITHHELD,
            "verifiable_intent.enabled is set, but the vi_verify tool is withheld from the \
             model-visible registry until a credential chain verifier exists. Enabling the \
             section does not enable credential verification on commerce tool calls. The \
             issuance and verification library paths are unaffected.",
            "verifiable_intent.enabled",
        ));
    }

    /// Collect non-fatal validation warnings — config that loads and
    /// validates successfully (`validate()` returns `Ok(())`) but will fail
    /// at runtime because of a logical inconsistency the schema cannot
    /// enforce structurally.
    ///
    /// Called by `validate()` (which emits each warning via `tracing::warn!`
    /// for log visibility) and by the gateway HTTP API (which returns the
    /// structured list in `PropResponse` / `PatchResponse` so dashboard
    /// callers see the same signal the CLI sees on stderr).
    ///
    /// Adding a new warning: append a check here, pick a stable `code`,
    /// and document the code in `validation_warnings.rs`.
    pub fn collect_warnings(&self) -> Vec<crate::validation_warnings::ValidationWarning> {
        let mut warnings = Vec::new();
        self.collect_codex_cli_extra_arg_warnings(&mut warnings);
        self.collect_fallback_warnings(&mut warnings);
        self.collect_cross_provider_summary_model_warnings(&mut warnings);
        self.collect_a2a_exposed_skills_warnings(&mut warnings);
        self.collect_memory_semantic_search_warnings(&mut warnings);
        self.collect_dns_pinned_proxy_warnings(&mut warnings);
        self.collect_peer_groups_warnings(&mut warnings);
        // Must run after `collect_cross_provider_summary_model_warnings`: it
        // scans `warnings` to suppress its generic inert `summary_model`
        // warning when the more specific cross-provider diagnostic already
        // covers the same path.
        self.collect_context_compression_ignored_warnings(&mut warnings);
        self.collect_verifiable_intent_warnings(&mut warnings);
        warnings.extend(validate_memory_semantics(&self.memory));
        for (alias, wa) in &self.channels.whatsapp {
            warnings.extend(validate_whatsapp_semantics(alias, wa));
        }
        // `wire_api` is only honored by bring-your-own-endpoint families; on a
        // branded family with a fixed wire protocol it is silently ignored.
        // Surface that so an operator who sets it on, e.g., `mistral` learns it
        // has no effect instead of debugging unexpected runtime behavior.
        for (family, alias, entry) in self.providers.models.iter_entries() {
            if entry.wire_api.is_some() && !crate::provider_aliases::family_honors_wire_api(family)
            {
                warnings.push(crate::validation_warnings::ValidationWarning::new(
                    "wire_api_not_supported_for_family",
                    format!(
                        "wire_api is set on `{family}.{alias}` but the `{family}` family has a \
                         fixed wire protocol and ignores it. wire_api only takes effect on the \
                         openai, llamacpp, and custom (openai-compatible) families."
                    ),
                    format!("providers.models.{family}.{alias}.wire_api"),
                ));
            }
        }
        warnings
    }

    fn collect_dns_pinned_proxy_warnings(
        &self,
        warnings: &mut Vec<crate::validation_warnings::ValidationWarning>,
    ) {
        if !self.proxy.enabled {
            return;
        }
        if self.proxy.scope != ProxyScope::Environment && !self.proxy.has_any_proxy_url() {
            return;
        }

        let file_download_enabled = self
            .file_download
            .url
            .as_deref()
            .is_some_and(|url| !url.trim().is_empty());
        let mut affected = Vec::new();
        match self.proxy.scope {
            ProxyScope::Environment | ProxyScope::Omnesagent => {
                affected.push("http_request");
                affected.push("web_fetch");
                if file_download_enabled {
                    affected.push("file_download");
                }
            }
            ProxyScope::Services => {
                if self.proxy.should_apply_to_service("tool.http_request") {
                    affected.push("http_request");
                }
                if self.proxy.should_apply_to_service("tool.web_fetch") {
                    affected.push("web_fetch");
                }
                if file_download_enabled && self.proxy.should_apply_to_service("tool.file_download")
                {
                    affected.push("file_download");
                }
            }
        }
        if affected.is_empty() {
            return;
        }
        let affected = match affected.as_slice() {
            [one] => (*one).to_string(),
            [first, second] => format!("{first} and {second}"),
            [first, second, third] => format!("{first}, {second}, and {third}"),
            _ => affected.join(", "),
        };
        warnings.push(crate::validation_warnings::ValidationWarning::new(
            "proxy_conflicts_with_dns_pinned_tools",
            format!(
                "The configured proxy scope applies to DNS-pinned tool calls ({affected}), so \
                 those calls will fail instead of using an unpinned proxy connection. Use \
                 proxy.scope = \"services\" and omit tool.http_request, tool.web_fetch, \
                 tool.file_download, and tool.* from proxy.services."
            ),
            if self.proxy.scope == ProxyScope::Services {
                "proxy.services"
            } else {
                "proxy.scope"
            },
        ));
    }

    fn collect_codex_cli_extra_arg_warnings(
        &self,
        warnings: &mut Vec<crate::validation_warnings::ValidationWarning>,
    ) {
        for risky_match in risky_codex_cli_arg_matches(&self.codex_cli.extra_args) {
            warnings.push(crate::validation_warnings::ValidationWarning::new(
                CODEX_CLI_EXTRA_ARGS_SECURITY_BOUNDARY_WARNING,
                format!(
                    "Codex CLI argument `{}` can {}. OmnesAgent allows this operator-controlled \
                     argument without blocking; verify that the resulting trust boundary is \
                     intentional.",
                    risky_match.flag.display, risky_match.flag.effect
                ),
                format!("codex_cli.extra_args[{}]", risky_match.index),
            ));
        }
    }

    /// Surface sqlite semantic/hybrid search with no effective embedder. The
    /// runtime treats `NoopEmbedding` as dimensions 0 and silently skips the
    /// vector path, so derive this from the canonical memory settings instead
    /// of storing another effective-mode field.
    fn collect_memory_semantic_search_warnings(
        &self,
        warnings: &mut Vec<crate::validation_warnings::ValidationWarning>,
    ) {
        if !Self::memory_backend_is_sqlite(&self.memory.backend) {
            return;
        }
        if !matches!(
            &self.memory.search_mode,
            SearchMode::Embedding | SearchMode::Hybrid
        ) {
            return;
        }
        if self.memory_has_effective_embedder() {
            return;
        }

        let search_mode = match &self.memory.search_mode {
            SearchMode::Bm25 => "bm25",
            SearchMode::Embedding => "embedding",
            SearchMode::Hybrid => "hybrid",
        };
        warnings.push(crate::validation_warnings::ValidationWarning::new(
            "memory_semantic_search_without_embedder",
            format!(
                "memory.search_mode is {search_mode:?} on sqlite memory, but no effective \
                 embedding provider is configured; vector search is skipped and \
                 recall falls back to keyword-only behavior. Configure \
                 memory.embedding_provider or a valid memory.embedding_model \
                 hint route, or set memory.search_mode = \"bm25\"."
            ),
            "memory.search_mode",
        ));
    }

    fn memory_backend_is_sqlite(backend: &str) -> bool {
        let trimmed = backend.trim();
        let family = trimmed
            .split_once('.')
            .map_or(trimmed, |(family, _)| family)
            .trim();
        family.eq_ignore_ascii_case("sqlite")
    }

    fn memory_has_effective_embedder(&self) -> bool {
        let fallback = || Self::memory_embedding_settings_have_effective_embedder(&self.memory);
        let Some(hint) = self
            .memory
            .embedding_model
            .strip_prefix("hint:")
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return fallback();
        };

        let Some(route) = self
            .embedding_routes
            .iter()
            .find(|route| route.hint.trim() == hint)
        else {
            return fallback();
        };

        if Self::embedding_route_has_effective_embedder(route, self.memory.embedding_dimensions) {
            return true;
        }
        fallback()
    }

    fn memory_embedding_settings_have_effective_embedder(memory: &MemoryConfig) -> bool {
        let provider = memory.embedding_provider.trim();
        !provider.is_empty()
            && !provider.eq_ignore_ascii_case("none")
            && memory.embedding_dimensions > 0
    }

    fn embedding_route_has_effective_embedder(
        route: &EmbeddingRouteConfig,
        fallback_dimensions: usize,
    ) -> bool {
        let provider = route.model_provider.trim();
        let model = route.model.trim();
        let dimensions = route.dimensions.unwrap_or(fallback_dimensions);
        !provider.is_empty()
            && !provider.eq_ignore_ascii_case("none")
            && !model.is_empty()
            && dimensions > 0
    }

    /// Surface the A2A `exposed_skills` filter that can never resolve a skill.
    /// `exposed_skills` is a narrowing filter over the agent's resolved skill
    /// set, which comes from `skill_bundles`. An agent that lists
    /// `a2a.exposed_skills` but declares no `skill_bundles` advertises an empty
    /// `skills: []` array on its agent card with no error, because every wanted
    /// id is dropped for belonging to a bundle the agent does not declare. The
    /// same silent emptiness happens when a wanted id names no skill in any
    /// declared bundle, or when the owning bundle's include/exclude filter
    /// excludes it; those need disk resolution, so this offline check covers the
    /// common structural case: exposed skills with zero declared bundles.
    fn collect_a2a_exposed_skills_warnings(
        &self,
        warnings: &mut Vec<crate::validation_warnings::ValidationWarning>,
    ) {
        for (agent_alias, agent) in &self.agents {
            if agent.a2a.exposed_skills.is_empty() {
                continue;
            }
            if !agent.skill_bundles.is_empty() {
                continue;
            }
            warnings.push(crate::validation_warnings::ValidationWarning::new(
                "a2a_exposed_skills_without_bundles",
                format!(
                    "agent `{agent_alias}` lists a2a.exposed_skills but declares no \
                     skill_bundles, so its agent card advertises no skills (skills: []). \
                     exposed_skills is a filter over the agent's resolved skill set; add \
                     the owning bundle(s) to agents.{agent_alias}.skill_bundles so the \
                     listed skills resolve."
                ),
                format!("agents.{agent_alias}.a2a.exposed_skills"),
            ));
        }
    }

    /// Surface cross-provider ambiguity in a legacy config while reporting
    /// the current contract: context compression has no runtime consumer, so
    /// this knob is inert like every other `context_compression` field (see
    /// `collect_context_compression_ignored_warnings`). This diagnostic adds
    /// config-shape detail as a more specific companion for the same line. The
    /// deprecated
    /// `runtime_profiles.<p>.context_compression.summary_model` is a bare
    /// model id that names no provider of its own — it would need to be
    /// resolved onto each consuming agent's OWN provider were the field ever
    /// read again — so when a single profile is shared by agents resolving
    /// to MORE THAN ONE distinct provider, that one bare id is ambiguous for
    /// at least one of them. A `summary_provider` supplies provider identity
    /// for this narrower diagnostic and excludes the corresponding value from
    /// the ambiguity count; it does not make context compression functional.
    ///
    /// The diagnostic is offline and deterministic: no schema bump, no
    /// network, and no model catalog. It names the profile, the affected
    /// agents, and their differing providers, then recommends removing the
    /// unsupported setting or waiting for an accepted compression design.
    fn collect_cross_provider_summary_model_warnings(
        &self,
        warnings: &mut Vec<crate::validation_warnings::ValidationWarning>,
    ) {
        for (profile_alias, profile) in &self.runtime_profiles {
            // Only the deprecated bare summary_model lacks provider identity.
            // A profile-level summary_provider excludes this narrower
            // ambiguity shape, but context compression remains inert.
            if !profile
                .context_compression
                .summary_provider
                .trim()
                .is_empty()
            {
                continue;
            }
            let Some(summary_model) = profile.context_compression.summary_model.as_deref() else {
                continue;
            };
            if summary_model.trim().is_empty() {
                continue;
            }

            // Gather agents that reference this profile and have no agent-level
            // summary_provider identity. An override excludes that agent from
            // this ambiguity diagnostic but does not make compression
            // functional. Resolve the provider that would be paired with the
            // bare model if a future implementation consumed this config.
            let mut affected: Vec<(String, String)> = Vec::new();
            for (agent_alias, agent) in &self.agents {
                if agent.runtime_profile.trim() != profile_alias {
                    continue;
                }
                if !agent.summary_provider.trim().is_empty() {
                    continue;
                }
                let provider_label = self.canonical_provider_label(agent.model_provider.trim());
                affected.push((agent_alias.clone(), provider_label));
            }

            // Cross-provider ambiguity requires distinct providers. A
            // same-provider bare id is still inert, but the generic
            // context-compression warning reports that fact without this
            // additional ambiguity detail.
            let distinct: std::collections::BTreeSet<&str> =
                affected.iter().map(|(_, p)| p.as_str()).collect();
            if distinct.len() < 2 {
                continue;
            }

            let mut agents_sorted: Vec<&(String, String)> = affected.iter().collect();
            agents_sorted.sort_by(|a, b| a.0.cmp(&b.0));
            let detail = agents_sorted
                .iter()
                .map(|(name, provider)| format!("{name} -> {provider}"))
                .collect::<Vec<_>>()
                .join(", ");

            warnings.push(crate::validation_warnings::ValidationWarning::new(
                "cross_provider_summary_model",
                format!(
                    "runtime_profiles.{profile_alias}.context_compression.summary_model \
                     ({summary_model:?}) is set, but context compression is not currently \
                     implemented in the runtime; this setting has no effect. It is also a \
                     bare model id reused by agents resolving to different providers \
                     ({detail}), which names no provider of its own and would be ambiguous \
                     for at least one of them if compression were read again. Remove the \
                     unsupported context_compression setting (every context_compression \
                     field, including summary_provider, is currently inert), or wait for a \
                     separately accepted compression design before configuring it."
                ),
                format!("runtime_profiles.{profile_alias}.context_compression.summary_model"),
            ));
        }
    }

    /// Surface every non-default `context_compression` knob as inert: the
    /// runtime context compressor was removed and nothing in the
    /// workspace reads `context_compression` at runtime anymore, so the whole
    /// struct — `enabled`, thresholds, protected counts, summarizer limits,
    /// provider selection, tool-result retrimming — has no effect. Mirrors
    /// `validate_memory_semantics`: one warning per non-default field, so a
    /// user sees exactly which of their authored knobs are dead. A field
    /// explicitly written at its default value is indistinguishable from an
    /// omitted one post-deserialization and stays silent (same limitation as
    /// `validate_memory_semantics`).
    ///
    /// Only `[runtime_profiles.<alias>.context_compression]` is checked —
    /// `AliasedAgentConfig` (`[agents.<alias>]`) has no `context_compression`
    /// field of its own, and the legacy pre-V3 `[agent.context_compression]`
    /// top-level table (folded into `[runtime_profiles.default]` by the V2→V3
    /// migration, see `schema/v2.rs`) has already collapsed into this same
    /// surface by the time `Config` exists, so a single pass over
    /// `runtime_profiles` covers both the historical and current authored
    /// forms without double-warning.
    fn collect_context_compression_ignored_warnings(
        &self,
        warnings: &mut Vec<crate::validation_warnings::ValidationWarning>,
    ) {
        let defaults = crate::scattered_types::ContextCompressionConfig::default();
        for (alias, profile) in &self.runtime_profiles {
            let cc = &profile.context_compression;

            // `enabled = true` gets its own message: it is the master switch
            // users flip expecting compression to happen at all.
            if cc.enabled {
                warnings.push(crate::validation_warnings::ValidationWarning::new(
                    "context_compression_unsupported",
                    format!(
                        "runtime_profiles.{alias}.context_compression.enabled is set but context \
                         compression is not currently implemented in the runtime (the compressor \
                         was removed in #8196); this setting has no effect."
                    ),
                    format!("runtime_profiles.{alias}.context_compression.enabled"),
                ));
            }

            // Every other knob: flag any value that differs from the default.
            let mut inert: Vec<&'static str> = Vec::new();
            if (cc.threshold_ratio - defaults.threshold_ratio).abs() > f64::EPSILON {
                inert.push("threshold_ratio");
            }
            if cc.protect_first_n != defaults.protect_first_n {
                inert.push("protect_first_n");
            }
            if cc.protect_last_n != defaults.protect_last_n {
                inert.push("protect_last_n");
            }
            if cc.max_passes != defaults.max_passes {
                inert.push("max_passes");
            }
            if cc.summary_max_chars != defaults.summary_max_chars {
                inert.push("summary_max_chars");
            }
            if cc.source_max_chars != defaults.source_max_chars {
                inert.push("source_max_chars");
            }
            if cc.timeout_secs != defaults.timeout_secs {
                inert.push("timeout_secs");
            }
            if cc.summary_provider != defaults.summary_provider {
                inert.push("summary_provider");
            }
            if cc.summary_model != defaults.summary_model {
                // Ordering dependency: `collect_warnings()` runs
                // `collect_cross_provider_summary_model_warnings` before this
                // helper, so a cross-provider `summary_model` diagnostic for
                // this same path is already in `warnings`. That diagnostic
                // reports the same "has no effect" fact plus the specific
                // cross-provider agents affected, so it wins and the generic
                // inert warning is skipped to avoid printing two warnings
                // for the same config line. All other `summary_model`
                // shapes (single-provider, unshared) still get the inert
                // warning; no other diagnostic covers them.
                let summary_model_path =
                    format!("runtime_profiles.{alias}.context_compression.summary_model");
                if !warnings.iter().any(|w| {
                    w.code == "cross_provider_summary_model" && w.path == summary_model_path
                }) {
                    inert.push("summary_model");
                }
            }
            if cc.identifier_policy != defaults.identifier_policy {
                inert.push("identifier_policy");
            }
            if cc.tool_result_retrim_chars != defaults.tool_result_retrim_chars {
                inert.push("tool_result_retrim_chars");
            }
            if cc.tool_result_trim_exempt != defaults.tool_result_trim_exempt {
                inert.push("tool_result_trim_exempt");
            }

            for field in inert {
                warnings.push(crate::validation_warnings::ValidationWarning::new(
                    "context_compression_unsupported",
                    format!(
                        "runtime_profiles.{alias}.context_compression.{field} is set to a \
                         non-default value but context compression is not currently implemented \
                         in the runtime (the compressor was removed in #8196); this setting has \
                         no effect."
                    ),
                    format!("runtime_profiles.{alias}.context_compression.{field}"),
                ));
            }
        }
    }

    /// Canonical label for an agent's resolved model provider, used to decide
    /// whether two agents sit on distinct providers. A non-empty ref that
    /// resolves through `[providers.models]` collapses to its canonical
    /// `<family>.<alias>` so equivalent spellings (bare vs dotted) compare
    /// equal; an empty or unresolved ref keeps its raw form (empty becomes a
    /// stable sentinel) so it still participates as a distinct bucket.
    fn canonical_provider_label(&self, provider_ref: &str) -> String {
        if provider_ref.is_empty() {
            return "<agent default provider>".to_string();
        }
        match self.providers.models.find_by_name(provider_ref) {
            Some((family, alias, _)) => format!("{family}.{alias}"),
            None => provider_ref.to_string(),
        }
    }

    /// Surface non-fatal issues in per-alias `fallback` chains: dangling refs
    /// (a fallback naming an alias that is not configured) and cycles (a
    /// fallback path that loops back onto itself). Both are warn-and-skip — the
    /// chain still loads and runs with the offending edge pruned at build time.
    fn collect_fallback_warnings(
        &self,
        warnings: &mut Vec<crate::validation_warnings::ValidationWarning>,
    ) {
        for (family, alias, cfg) in self.providers.models.iter_entries() {
            self.collect_fallback_model_warnings(family, alias, cfg, warnings);
            if cfg.fallback.is_empty() {
                continue;
            }
            let root = format!("{family}.{alias}");
            let mut visited: Vec<String> = vec![root.clone()];
            self.walk_fallback(&root, &cfg.fallback, &mut visited, 1, warnings);
        }
    }

    /// Surface `fallback_models` entries the build path silently skips: blank
    /// entries and entries that duplicate the alias's primary `model`. The skip
    /// itself is safe, but without a warning an operator never learns that a
    /// listed fallback model is doing nothing.
    fn collect_fallback_model_warnings(
        &self,
        family: &str,
        alias: &str,
        cfg: &ModelProviderConfig,
        warnings: &mut Vec<crate::validation_warnings::ValidationWarning>,
    ) {
        let Some(primary) = cfg.model.as_deref() else {
            return;
        };
        for (i, model) in cfg.fallback_models.iter().enumerate() {
            let path = format!("providers.models.{family}.{alias}.fallback_models[{i}]");
            if model.trim().is_empty() {
                warnings.push(crate::validation_warnings::ValidationWarning::new(
                    "empty_fallback_model",
                    format!(
                        "fallback_models entry {i} on {family}.{alias} is empty; \
                         it is skipped at runtime"
                    ),
                    path,
                ));
            } else if model == primary {
                warnings.push(crate::validation_warnings::ValidationWarning::new(
                    "fallback_model_duplicates_primary",
                    format!(
                        "fallback_models entry {model:?} on {family}.{alias} duplicates the \
                         primary model; it is skipped at runtime"
                    ),
                    path,
                ));
            }
        }
    }

    fn walk_fallback(
        &self,
        from: &str,
        refs: &[crate::providers::ModelProviderRef],
        visited: &mut Vec<String>,
        depth: usize,
        warnings: &mut Vec<crate::validation_warnings::ValidationWarning>,
    ) {
        if depth > crate::providers::MAX_FALLBACK_DEPTH {
            warnings.push(crate::validation_warnings::ValidationWarning::new(
                "max_fallback_depth_exceeded",
                format!(
                    "fallback chain from {from} exceeds the maximum depth of {}; \
                     deeper links are pruned at runtime",
                    crate::providers::MAX_FALLBACK_DEPTH
                ),
                format!("providers.models.{from}.fallback"),
            ));
            return;
        }
        for (i, fallback_ref) in refs.iter().enumerate() {
            let raw = fallback_ref.as_str().trim();
            if raw.is_empty() {
                continue;
            }
            let path = format!("providers.models.{from}.fallback[{i}]");
            let Some((family, alias, cfg)) = self.providers.models.find_by_name(raw) else {
                warnings.push(crate::validation_warnings::ValidationWarning::new(
                    "dangling_fallback_ref",
                    format!(
                        "fallback {raw:?} on {from} does not resolve to a configured \
                         providers.models entry; this fallback link is skipped at runtime"
                    ),
                    path,
                ));
                continue;
            };
            let resolved = format!("{family}.{alias}");
            if visited.iter().any(|v| v == &resolved) {
                warnings.push(crate::validation_warnings::ValidationWarning::new(
                    "fallback_cycle",
                    format!(
                        "fallback {raw:?} on {from} closes a cycle \
                         ({} -> {resolved}); the cycle edge is pruned at runtime",
                        visited.join(" -> ")
                    ),
                    path,
                ));
                continue;
            }
            visited.push(resolved.clone());
            self.walk_fallback(&resolved, &cfg.fallback, visited, depth + 1, warnings);
            visited.pop();
        }
    }

    /// Walk every channel HashMap that carries reply pacing fields and yield
    /// `(dotted-path-prefix, &dyn HasReplyPacing)` pairs. Used by validation
    /// and by anywhere that wants a single source of truth for which channel
    /// types participate in pacing.
    pub fn reply_pacing_entries(&self) -> Vec<(String, &dyn HasReplyPacing)> {
        let c = &self.channels;
        fn rows<'a, C: HasReplyPacing>(
            ch_type: &'static str,
            map: &'a std::collections::HashMap<String, C>,
        ) -> impl Iterator<Item = (String, &'a dyn HasReplyPacing)> + 'a {
            map.iter().map(move |(alias, cfg)| {
                (
                    format!("channels.{ch_type}.{alias}"),
                    cfg as &dyn HasReplyPacing,
                )
            })
        }
        rows("telegram", &c.telegram)
            .chain(rows("discord", &c.discord))
            .chain(rows("slack", &c.slack))
            .chain(rows("mattermost", &c.mattermost))
            .chain(rows("webhook", &c.webhook))
            .chain(rows("imessage", &c.imessage))
            .chain(rows("matrix", &c.matrix))
            .chain(rows("signal", &c.signal))
            .chain(rows("whatsapp", &c.whatsapp))
            .collect()
    }

    /// Classify a raw `peer_groups.<name>.channel` value against the
    /// configured `[channels.*]` blocks. See [`PeerGroupChannelRef`] for the
    /// meaning of each variant; this is the only place that resolves the
    /// channel-type/alias lookup for peer groups, via the same canonical
    /// `get_map_keys` enumeration every other typed-ref check uses (`None` =
    /// the channel type has no configured block at all; `Some(keys)` = the
    /// list of configured aliases for that type).
    fn classify_peer_group_channel_ref(&self, raw: &str) -> PeerGroupChannelRef {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return PeerGroupChannelRef::Empty;
        }
        // `get_map_keys` stores section names using the raw field ident
        // (snake); look up the channel type verbatim.
        let (channel_type, alias) = match trimmed.split_once('.') {
            Some((ty, al)) => (ty, Some(al)),
            None => (trimmed, None),
        };
        let channel_aliases = self.get_map_keys(&format!("channels.{channel_type}"));
        match alias {
            None => PeerGroupChannelRef::BareType {
                channel_type: channel_type.to_string(),
                configured: channel_aliases.is_some(),
            },
            Some(alias) => {
                let type_configured = channel_aliases.is_some();
                let exists = channel_aliases
                    .as_ref()
                    .is_some_and(|keys| keys.iter().any(|k| k == alias));
                // `get_map_keys` iterates a HashMap-backed section, so its
                // order is randomized per process; sort here (the canonical
                // resolution point) so downstream consumers — notably the
                // did-you-mean tie-break in `collect_peer_groups_warnings`
                // — are deterministic across runs.
                let mut known_aliases = channel_aliases.unwrap_or_default();
                known_aliases.sort();
                PeerGroupChannelRef::Dotted {
                    channel_type: channel_type.to_string(),
                    alias: alias.to_string(),
                    type_configured,
                    exists,
                    known_aliases,
                }
            }
        }
    }

    /// Surface a `peer_groups.<name>.channel` dotted reference that does not
    /// resolve to any configured `[channels.<type>.<alias>]` block —
    /// typically a typo (`telegram.alert` vs. configured `telegram.alerts`)
    /// that silently authorizes nobody instead of failing loudly. This is a
    /// non-fatal companion to the `DanglingReference` hard error
    /// `Config::validate()` already raises for the exact same condition (see
    /// `classify_peer_group_channel_ref`): it exists so config-load tracing,
    /// `channel doctor`, and the gateway config surface can flag the
    /// misconfiguration even when a caller chooses to log-and-continue past
    /// a failed `validate()` instead of refusing to boot.
    ///
    /// Bare type-wide refs (`channel = "discord"`) are an explicit non-goal:
    /// they intentionally authorize every alias of that type, so there is no
    /// "did you mean" ambiguity, and an unconfigured bare type is already a
    /// hard error at `validate()`.
    fn collect_peer_groups_warnings(
        &self,
        warnings: &mut Vec<crate::validation_warnings::ValidationWarning>,
    ) {
        let mut group_names: Vec<&String> = self.peer_groups.keys().collect();
        group_names.sort();
        for group_name in group_names {
            let group_channel = self.peer_groups[group_name].channel.trim();
            let PeerGroupChannelRef::Dotted {
                channel_type,
                alias,
                type_configured,
                exists,
                known_aliases,
            } = self.classify_peer_group_channel_ref(group_channel)
            else {
                continue;
            };
            if type_configured && exists {
                continue;
            }

            let suggestion = if type_configured {
                crate::validation_warnings::closest_match(&alias, &known_aliases)
                    .map(|closest| format!("; did you mean \"{channel_type}.{closest}\"?"))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            let reason = if type_configured {
                format!("[channels.{channel_type}.{alias}] is not configured")
            } else {
                format!("no [channels.{channel_type}.*] block is configured")
            };
            warnings.push(crate::validation_warnings::ValidationWarning::new(
                "peer_group_channel_dangling",
                format!(
                    "peer_groups.{group_name}.channel = {group_channel:?} but {reason}{suggestion}"
                ),
                format!("peer_groups.{group_name}.channel"),
            ));
        }
    }

    /// Validate configuration values that would cause runtime failures.
    ///
    /// Called after TOML deserialization and env-override application to catch
    /// obviously invalid values early instead of failing at arbitrary runtime points.
    pub fn validate(&self) -> Result<()> {
        validate_memory_rerank_config(&self.memory)?;
        self.cost.rates.validate()?;

        let websocket_ping_interval_secs = self.gateway.websocket_ping_interval_secs;
        if websocket_ping_interval_secs > GATEWAY_WEBSOCKET_PING_INTERVAL_MAX_SECS {
            let path = "gateway.websocket_ping_interval_secs";
            validation_bail!(
                InvalidNumericRange,
                path,
                "{path} = {websocket_ping_interval_secs} is out of range; must be 0..={GATEWAY_WEBSOCKET_PING_INTERVAL_MAX_SECS}"
            );
        }

        // Tunnel — OpenVPN
        if self.tunnel.tunnel_provider.trim() == "openvpn" {
            let openvpn = self.tunnel.openvpn.as_ref().ok_or_else(|| {
                ::omnesagent_log::record!(
                    WARN,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Reject)
                        .with_outcome(::omnesagent_log::EventOutcome::Failure),
                    "tunnel.tunnel_provider='openvpn' rejected: [tunnel.openvpn] block missing"
                );
                anyhow::Error::msg("tunnel.tunnel_provider='openvpn' requires [tunnel.openvpn]")
            })?;

            if openvpn.config_file.trim().is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    "tunnel.openvpn.config_file",
                    "tunnel.openvpn.config_file must not be empty"
                );
            }
            if openvpn.connect_timeout_secs == 0 {
                validation_bail!(
                    InvalidNumericRange,
                    "tunnel.openvpn.connect_timeout_secs",
                    "tunnel.openvpn.connect_timeout_secs must be greater than 0"
                );
            }
        }

        let mut lucid_aliases: Vec<&String> = self.storage.lucid.keys().collect();
        lucid_aliases.sort();
        for alias in lucid_aliases {
            let lucid = &self.storage.lucid[alias];
            if lucid
                .binary_path
                .as_ref()
                .is_some_and(|path| path.trim().is_empty())
            {
                validation_bail!(
                    RequiredFieldEmpty,
                    format!("storage.lucid.{alias}.binary_path"),
                    "storage.lucid.{alias}.binary_path must not be empty"
                );
            }
            if matches!(lucid.recall_timeout_ms, Some(0)) {
                validation_bail!(
                    InvalidNumericRange,
                    format!("storage.lucid.{alias}.recall_timeout_ms"),
                    "storage.lucid.{alias}.recall_timeout_ms must be greater than 0"
                );
            }
            if matches!(lucid.store_timeout_ms, Some(0)) {
                validation_bail!(
                    InvalidNumericRange,
                    format!("storage.lucid.{alias}.store_timeout_ms"),
                    "storage.lucid.{alias}.store_timeout_ms must be greater than 0"
                );
            }
        }

        // Reply-pacing bounds — both `reply_min_interval_secs` and
        // `reply_queue_depth_max` walk through one entry list so adding
        // a new paced channel only requires extending `reply_pacing_entries`.
        for (path_prefix, cfg) in self.reply_pacing_entries() {
            let secs = cfg.reply_min_interval_secs();
            if secs > REPLY_MIN_INTERVAL_MAX_SECS {
                let path = format!("{path_prefix}.reply_min_interval_secs");
                validation_bail!(
                    InvalidNumericRange,
                    path,
                    "{path} = {secs} is out of range; must be 0..={REPLY_MIN_INTERVAL_MAX_SECS}"
                );
            }
            let depth = cfg.reply_queue_depth_max();
            if depth > REPLY_QUEUE_DEPTH_CEILING {
                let path = format!("{path_prefix}.reply_queue_depth_max");
                validation_bail!(
                    InvalidNumericRange,
                    path,
                    "{path} = {depth} is out of range; must be 0..={REPLY_QUEUE_DEPTH_CEILING}"
                );
            }
        }

        for (alias, tg) in &self.channels.telegram {
            tg.validate_bot_token(alias)?;
            validate_http_base_url(
                &format!("channels.telegram.{alias}.api_base_url"),
                &tg.api_base_url,
            )?;
        }

        for (alias, matrix) in &self.channels.matrix {
            matrix.validate_stream_tool_arguments().with_context(|| {
                format!("invalid channels.matrix.{alias}.stream_tool_arguments")
            })?;
        }

        for (alias, slack) in &self.channels.slack {
            let max_messages = slack.effective_thread_context_max_messages();
            if max_messages > MAX_SLACK_THREAD_CONTEXT_MAX_MESSAGES {
                let path = format!("channels.slack.{alias}.thread_context_max_messages");
                validation_bail!(
                    InvalidNumericRange,
                    path,
                    "{path} = {max_messages} is out of range; must be 0..={MAX_SLACK_THREAD_CONTEXT_MAX_MESSAGES}"
                );
            }
        }

        for (alias, dc) in &self.channels.discord {
            dc.validate_bot_token(alias)?;
        }

        // Signal and Voice Call: like Telegram/Discord's bot_token, these
        // channels have unambiguous required plain-String credentials. An
        // enabled channel built with an empty credential never connects and
        // its per-channel supervisor restarts it forever (crashloop), so
        // reject at config-load time instead.
        for (alias, sig) in &self.channels.signal {
            sig.validate_required(alias)?;
        }

        for (alias, vc) in &self.channels.voice_call {
            let enabled_path = format!("channels.voice_call.{alias}.enabled");
            validate_required_field(
                &format!("channels.voice_call.{alias}.account_id"),
                &enabled_path,
                vc.enabled,
                &vc.account_id,
            )?;
            validate_required_field(
                &format!("channels.voice_call.{alias}.auth_token"),
                &enabled_path,
                vc.enabled,
                &vc.auth_token,
            )?;
            validate_required_field(
                &format!("channels.voice_call.{alias}.from_number"),
                &enabled_path,
                vc.enabled,
                &vc.from_number,
            )?;
        }

        // Git forge channel: a PAT-backed provider must name its API origin
        // explicitly. There is deliberately no default host - every request
        // carries `access_token` as a bearer credential, so guessing an
        // endpoint (e.g. gitea.com) would disclose the token to a host the
        // operator never configured. Only enabled aliases are checked so a
        // half-filled disabled block doesn't fail the whole config.
        for (alias, git) in &self.channels.git {
            let provider = git.provider.trim().to_ascii_lowercase();
            if git.enabled && matches!(provider.as_str(), "gitea" | "forgejo") {
                let path = format!("channels.git.{alias}.api_base_url");
                match git.api_base_url.as_deref() {
                    None => validation_bail!(
                        RequiredFieldEmpty,
                        path,
                        "{path} is required when provider = \"{provider}\": set the \
                         instance's API base URL including /api/v1 (e.g. \
                         https://git.example.org/api/v1); no default host is assumed \
                         because API requests carry the access token"
                    ),
                    Some(url) => validate_http_base_url(&path, url)?,
                }
            }
        }

        for name in self.http_request.secrets.keys() {
            if name.is_empty()
                || name.len() > 64
                || !name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                validation_bail!(
                    InvalidFormat,
                    format!("http_request.secrets.{name}"),
                    "http_request.secrets key {name:?} must contain 1..=64 ASCII letters, numbers, underscores, or hyphens"
                );
            }
        }

        // Gateway
        if self.gateway.host.trim().is_empty() {
            validation_bail!(
                RequiredFieldEmpty,
                "gateway.host",
                "gateway.host must not be empty"
            );
        }
        if self.nodes.mdns.max_peers == 0 {
            validation_bail!(
                InvalidNumericRange,
                "nodes.mdns.max_peers",
                "nodes.mdns.max_peers must be greater than 0"
            );
        }
        if self.nodes.mdns.announce_interval_secs == 0 {
            validation_bail!(
                InvalidNumericRange,
                "nodes.mdns.announce_interval_secs",
                "nodes.mdns.announce_interval_secs must be greater than 0"
            );
        }
        if self.nodes.mdns.peer_ttl_secs == 0 {
            validation_bail!(
                InvalidNumericRange,
                "nodes.mdns.peer_ttl_secs",
                "nodes.mdns.peer_ttl_secs must be greater than 0"
            );
        }
        if self.nodes.mdns.peer_ttl_secs <= self.nodes.mdns.announce_interval_secs {
            validation_bail!(
                InvalidNumericRange,
                "nodes.mdns.peer_ttl_secs",
                "nodes.mdns.peer_ttl_secs must be greater than nodes.mdns.announce_interval_secs"
            );
        }
        if matches!(self.transcription.max_audio_bytes, Some(0)) {
            validation_bail!(
                InvalidNumericRange,
                "transcription.max_audio_bytes",
                "transcription.max_audio_bytes must be greater than zero"
            );
        }
        if self.channels.max_concurrent_per_channel == 0 {
            validation_bail!(
                InvalidNumericRange,
                "channels.max_concurrent_per_channel",
                "channels.max_concurrent_per_channel must be greater than 0"
            );
        }
        // Typed-memory producers are a SQLite-only slice: the enabled write
        // path stores through `store_with_options_and_agent`, and only the
        // SQLite backend persists the full typed StoreOptions surface. Any
        // other backend would hit the trait default's explicit failure deep
        // inside spawned background consolidation, so reject the
        // configuration at load instead.
        if self.memory.types.enabled || self.memory.consolidation_extract_facts {
            let flag_path = if self.memory.types.enabled {
                "memory.types.enabled"
            } else {
                "memory.consolidation_extract_facts"
            };
            // Mirror the runtime's backend classification
            // (`backend_kind_from_dotted`): trim, take the prefix before the
            // first dot, compare case-insensitively.
            let backend_kind = self
                .memory
                .backend
                .trim()
                .split('.')
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase();
            if backend_kind != "sqlite" {
                validation_bail!(
                    InvalidFormat,
                    flag_path,
                    "{flag_path} = true requires memory.backend = \"sqlite\" (typed memory storage is SQLite-only), but memory.backend = {:?}",
                    self.memory.backend
                );
            }
            for (alias, agent) in &self.agents {
                if agent.memory.backend != crate::multi_agent::MemoryBackendKind::Sqlite {
                    let agent_backend = agent.memory.backend;
                    validation_bail!(
                        InvalidFormat,
                        format!("agents.{alias}.memory.backend"),
                        "{flag_path} = true requires every agent on the sqlite memory backend (typed memory storage is SQLite-only), but agents.{alias}.memory.backend = {agent_backend:?}",
                    );
                }
            }
        }
        for (alias, agent) in &self.agents {
            if agent.precheck.timeout_secs == 0 {
                validation_bail!(
                    InvalidNumericRange,
                    format!("agents.{alias}.precheck.timeout_secs"),
                    "agents.{alias}.precheck.timeout_secs must be greater than 0"
                );
            }
        }
        // Heartbeat agent: when heartbeat is enabled, the agent field
        // must name a configured agent.
        if self.heartbeat.enabled {
            let hb_agent = self.heartbeat.agent.trim();
            if hb_agent.is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    "heartbeat.agent",
                    "heartbeat.agent must reference a configured agent when heartbeat.enabled = true"
                );
            }
            if !self.agents.contains_key(hb_agent) {
                validation_bail!(
                    DanglingReference,
                    "heartbeat.agent",
                    "heartbeat.agent = {hb_agent:?} but no [agents.{hb_agent}] entry is configured"
                );
            }
        }
        if let Some(ref prefix) = self.gateway.path_prefix {
            // Validate the raw value — no silent trimming so the stored
            // value is exactly what was validated.
            if !prefix.is_empty() {
                if !prefix.starts_with('/') {
                    validation_bail!(
                        InvalidFormat,
                        "gateway.path_prefix",
                        "gateway.path_prefix must start with '/'"
                    );
                }
                if prefix.ends_with('/') {
                    validation_bail!(
                        InvalidFormat,
                        "gateway.path_prefix",
                        "gateway.path_prefix must not end with '/' (including bare '/')"
                    );
                }
                // Reject characters unsafe for URL paths or HTML/JS injection.
                // Whitespace is intentionally excluded from the allowed set.
                if let Some(bad) = prefix.chars().find(|c| {
                    !matches!(c, '/' | '-' | '_' | '.' | '~'
                        | 'a'..='z' | 'A'..='Z' | '0'..='9'
                        | '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '='
                        | ':' | '@')
                }) {
                    anyhow::bail!(
                        "gateway.path_prefix contains invalid character '{bad}'; \
                         only unreserved and sub-delim URI characters are allowed"
                    );
                }
            }
        }

        // Skill bundles — directories must stay inside `<install>/shared/`
        // and no two bundles may resolve to the same directory. Default
        // directory and the rules themselves live in
        // [`crate::skill_bundles`] so the runtime SkillsService and this
        // validator share one implementation.
        if !self.skill_bundles.is_empty() {
            let install_root = self.install_root_dir();
            for alias in self.skill_bundles.keys() {
                let dir = crate::skill_bundles::resolve_directory(self, &install_root, alias)
                    .map_err(|e| {
                        ::omnesagent_log::record!(
                            WARN,
                            ::omnesagent_log::Event::new(
                                module_path!(),
                                ::omnesagent_log::Action::Reject
                            )
                            .with_outcome(::omnesagent_log::EventOutcome::Failure)
                            .with_attrs(::serde_json::json!({
                                "skill_bundle": alias,
                                "error": format!("{}", e),
                            })),
                            "skill_bundles.<alias>.directory could not be resolved"
                        );
                        anyhow::Error::msg(e.to_string())
                    })?;
                if let Err(e) = crate::skill_bundles::validate_directory(&dir, &install_root) {
                    validation_bail!(
                        InvalidFormat,
                        format!("skill-bundles.{alias}.directory"),
                        "{e}"
                    );
                }
            }
            if let Err(e) = crate::skill_bundles::validate_uniqueness(self, &install_root) {
                validation_bail!(InvalidFormat, "skill_bundles", "{e}");
            }
        }

        // MCP bundles: resolution is secure by default, so a dangling
        // reference fails closed (the agent is granted fewer servers, never
        // more). Surface the misconfiguration as a warning so an operator is
        // not left wondering why an agent's MCP tools vanished, without
        // turning a typo into a hard startup failure.
        {
            // Operator UX: secure-by-default means an agent with no
            // `mcp_bundles` grant connects to ZERO MCP servers. When MCP is
            // enabled and `[[mcp.servers]]` is non-empty but no
            // `[mcp_bundles.*]` exists at all, every agent silently gets
            // nothing. That is the inverse of the original silent
            // no-op and surprises operators upgrading from <0.8.3. Warn
            // once at startup so this surfaces via `doctor` and the
            // standard startup warning stream.
            if self.mcp.enabled && !self.mcp.servers.is_empty() && self.mcp_bundles.is_empty() {
                ::omnesagent_log::record!(
                    WARN,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note,)
                        .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                        .with_attrs(::serde_json::json!({
                            "mcp_servers_configured": self.mcp.servers.len(),
                        })),
                    "[[mcp.servers]] is configured but no [mcp_bundles.*] bundles exist; no agent will receive any MCP tools. Add a [mcp_bundles.<alias>] entry and reference it from agents.<alias>.mcp_bundles to grant access."
                );
            }

            let known_servers: std::collections::HashSet<&str> =
                self.mcp.servers.iter().map(|s| s.name.as_str()).collect();
            for (bundle_alias, bundle) in &self.mcp_bundles {
                for server in bundle.servers.iter().chain(bundle.exclude.iter()) {
                    if !known_servers.contains(server.as_str()) {
                        ::omnesagent_log::record!(
                            WARN,
                            ::omnesagent_log::Event::new(
                                module_path!(),
                                ::omnesagent_log::Action::Note
                            )
                            .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                            .with_attrs(::serde_json::json!({
                                "mcp_bundle": bundle_alias,
                                "server": server,
                            })),
                            "mcp_bundles.<alias> references an MCP server name not in [mcp.servers]; it grants nothing"
                        );
                    }
                }
            }
            for agent_alias in self.agents.keys() {
                let Some(agent) = self.resolved_agent_config(agent_alias) else {
                    continue;
                };
                for bundle_alias in &agent.mcp_bundles {
                    if !self.mcp_bundles.contains_key(bundle_alias) {
                        ::omnesagent_log::record!(
                            WARN,
                            ::omnesagent_log::Event::new(
                                module_path!(),
                                ::omnesagent_log::Action::Note
                            )
                            .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                            .with_attrs(::serde_json::json!({
                                "agent": agent_alias,
                                "mcp_bundle": bundle_alias,
                            })),
                            "agents.<alias>.mcp_bundles references an undefined [mcp_bundles.<alias>]; the agent is granted no servers from it"
                        );
                    }
                }
            }
        }

        // Validate every configured risk profile. Each profile stands on
        // its own — there is no "active" or "default" risk profile concept;
        // an agent's `risk_profile` field names exactly which one applies.
        let mut profile_aliases: Vec<&String> = self.risk_profiles.keys().collect();
        profile_aliases.sort();
        for profile_alias in profile_aliases {
            let profile = &self.risk_profiles[profile_alias];
            for (i, env_name) in profile.shell_env_passthrough.iter().enumerate() {
                if !is_valid_env_var_name(env_name) {
                    anyhow::bail!(
                        "risk_profiles.{profile_alias}.shell_env_passthrough[{i}] is invalid ({env_name}); expected [A-Za-z_][A-Za-z0-9_]*"
                    );
                }
            }
            // `deny_all_tools` is the explicit deny-all gate; `allowed_tools = []`
            // stays legacy-unrestricted. Combining the flag with a non-empty
            // allowlist is contradictory — reject it instead of silently
            // preferring one side.
            if profile.deny_all_tools && !profile.allowed_tools.is_empty() {
                anyhow::bail!(
                    "risk_profiles.{profile_alias}.deny_all_tools cannot be combined with a non-empty allowed_tools list: deny_all_tools denies every tool, while allowed_tools = [] alone means unrestricted"
                );
            }
        }

        // Security OTP / estop
        if self.security.otp.challenge_max_attempts == 0 {
            validation_bail!(
                InvalidNumericRange,
                "security.otp.challenge_max_attempts",
                "security.otp.challenge_max_attempts must be greater than 0"
            );
        }
        if self.security.otp.token_ttl_secs == 0 {
            validation_bail!(
                InvalidNumericRange,
                "security.otp.token_ttl_secs",
                "security.otp.token_ttl_secs must be greater than 0"
            );
        }
        if self.security.otp.cache_valid_secs == 0 {
            validation_bail!(
                InvalidNumericRange,
                "security.otp.cache_valid_secs",
                "security.otp.cache_valid_secs must be greater than 0"
            );
        }
        if self.security.otp.cache_valid_secs < self.security.otp.token_ttl_secs {
            anyhow::bail!(
                "security.otp.cache_valid_secs must be greater than or equal to security.otp.token_ttl_secs"
            );
        }
        if self.security.otp.challenge_max_attempts == 0 {
            validation_bail!(
                InvalidNumericRange,
                "security.otp.challenge_max_attempts",
                "security.otp.challenge_max_attempts must be greater than 0"
            );
        }
        for (i, action) in self.security.otp.gated_actions.iter().enumerate() {
            let normalized = action.trim();
            if normalized.is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    format!("security.otp.gated_actions[{i}]"),
                    "security.otp.gated_actions[{i}] must not be empty"
                );
            }
            if !normalized
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                anyhow::bail!(
                    "security.otp.gated_actions[{i}] contains invalid characters: {normalized}"
                );
            }
            if !default_otp_gated_actions()
                .iter()
                .any(|known| known == normalized)
            {
                ::omnesagent_log::record!(
                    WARN,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                        .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                        .with_attrs(::serde_json::json!({
                            "action": normalized,
                            "known_actions": default_otp_gated_actions(),
                        })),
                    "security.otp.gated_actions entry does not match a known gated action and will not be enforced: "
                );
            }
        }
        DomainMatcher::new(
            &self.security.otp.gated_domains,
            &self.security.otp.gated_domain_categories,
        )
        .with_context(
            || "Invalid security.otp.gated_domains or security.otp.gated_domain_categories",
        )?;
        if self.security.estop.state_file.trim().is_empty() {
            validation_bail!(
                RequiredFieldEmpty,
                "security.estop.state_file",
                "security.estop.state_file must not be empty"
            );
        }
        if !(0.0..=1.0).contains(&self.security.leak_detection.sensitivity) {
            validation_bail!(
                InvalidNumericRange,
                "security.leak_detection.sensitivity",
                "security.leak_detection.sensitivity must be between 0.0 and 1.0"
            );
        }

        // Scheduler
        if self.scheduler.max_concurrent == 0 {
            validation_bail!(
                InvalidNumericRange,
                "scheduler.max_concurrent",
                "scheduler.max_concurrent must be greater than 0"
            );
        }
        if self.scheduler.max_tasks == 0 {
            validation_bail!(
                InvalidNumericRange,
                "scheduler.max_tasks",
                "scheduler.max_tasks must be greater than 0"
            );
        }

        // Model routes
        for (i, route) in self.model_routes.iter().enumerate() {
            if route.hint.trim().is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    format!("model_routes[{i}].hint"),
                    "model_routes[{i}].hint must not be empty"
                );
            }
            let mp = route.model_provider.trim();
            if mp.is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    format!("model_routes[{i}].model_provider"),
                    "model_routes[{i}].model_provider must not be empty"
                );
            }
            // Route refs are dotted `<type>.<alias>` and must resolve to a
            // configured `[providers.models.<type>.<alias>]` entry. Unresolved
            // routes are dropped at runtime construction; rejecting them here
            // keeps that drift visible at config-load time.
            match mp.split_once('.') {
                Some((ty, inner)) if !ty.is_empty() && !inner.is_empty() => {
                    if self.providers.models.find(ty, inner).is_none() {
                        validation_bail!(
                            DanglingReference,
                            format!("model_routes[{i}].model_provider"),
                            "model_routes[{i}].model_provider = {mp:?} but providers.models.{ty}.{inner} is not configured",
                        );
                    }
                }
                _ => validation_bail!(
                    InvalidFormat,
                    format!("model_routes[{i}].model_provider"),
                    "model_routes[{i}].model_provider must be dotted form `<type>.<alias>` (got {mp:?})",
                ),
            }
            if route.model.trim().is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    format!("model_routes[{i}].model"),
                    "model_routes[{i}].model must not be empty"
                );
            }
        }

        // Embedding routes
        for (i, route) in self.embedding_routes.iter().enumerate() {
            if route.hint.trim().is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    format!("embedding_routes[{i}].hint"),
                    "embedding_routes[{i}].hint must not be empty"
                );
            }
            let mp = route.model_provider.trim();
            if mp.is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    format!("embedding_routes[{i}].model_provider"),
                    "embedding_routes[{i}].model_provider must not be empty"
                );
            }
            // Embedding routes resolve against the same model-provider map;
            // there is no separate `providers.embeddings` typed section.
            match mp.split_once('.') {
                Some((ty, inner)) if !ty.is_empty() && !inner.is_empty() => {
                    if self.providers.models.find(ty, inner).is_none() {
                        validation_bail!(
                            DanglingReference,
                            format!("embedding_routes[{i}].model_provider"),
                            "embedding_routes[{i}].model_provider = {mp:?} but providers.models.{ty}.{inner} is not configured",
                        );
                    }
                }
                _ => validation_bail!(
                    InvalidFormat,
                    format!("embedding_routes[{i}].model_provider"),
                    "embedding_routes[{i}].model_provider must be dotted form `<type>.<alias>` (got {mp:?})",
                ),
            }
            if route.model.trim().is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    format!("embedding_routes[{i}].model"),
                    "embedding_routes[{i}].model must not be empty"
                );
            }
        }

        for (type_key, alias_key, profile) in self.providers.models.iter_entries() {
            let profile_name = format!("{type_key}.{alias_key}");

            let has_uri = profile
                .uri
                .as_deref()
                .map(str::trim)
                .is_some_and(|value| !value.is_empty());

            // Entries created by migration from top-level fields use the
            // model_provider type+alias as the map key and may not have
            // explicit `uri` (the model_provider factory resolves the
            // family's default endpoint via `ModelEndpoint`). An entry
            // with no identifying information at all is almost always an
            // in-progress quickstart state — the user picked the model
            // provider but hasn't filled anything in yet. Warn but don't
            // bail; the runtime falls back to family-default endpoint at
            // use time, and a chat against the unconfigured model
            // provider fails with a clear error then.
            let has_api_key = profile
                .api_key
                .as_deref()
                .is_some_and(|v| !v.trim().is_empty());
            let has_model = profile
                .model
                .as_deref()
                .is_some_and(|v| !v.trim().is_empty());
            if !has_uri && !has_api_key && !has_model {
                ::omnesagent_log::record!(WARN, ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note).with_outcome(::omnesagent_log::EventOutcome::Unknown).with_attrs(::serde_json::json!({"model_provider": profile_name, "profile_name": profile_name})), "providers.models. is empty (no uri / api_key / model). \
                     Skipping at runtime; run `omnesagent quickstart` (or use the dashboard) \
                     to make this model_provider usable.");
                continue;
            }

            if let Some(uri) = profile.uri.as_deref().map(str::trim)
                && !uri.is_empty()
            {
                let parsed = reqwest::Url::parse(uri).with_context(|| {
                    format!("providers.models.{profile_name}.uri is not a valid URL")
                })?;
                if !matches!(parsed.scheme(), "http" | "https") {
                    anyhow::bail!("providers.models.{profile_name}.uri must use http/https");
                }
            }

            if let Some(temp) = profile.temperature {
                validate_temperature(temp).map_err(|e| {
                    ::omnesagent_log::record!(
                        WARN,
                        ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Reject)
                            .with_outcome(::omnesagent_log::EventOutcome::Failure)
                            .with_attrs(::serde_json::json!({
                                "profile": profile_name,
                                "temperature": temp,
                                "error": format!("{}", e),
                            })),
                        "providers.models.<alias>.temperature rejected"
                    );
                    anyhow::Error::msg(format!("providers.models.{profile_name}.temperature: {e}"))
                })?;
            }

            for (key, value) in &profile.pricing {
                if value.is_nan() {
                    anyhow::bail!(
                        "providers.models.{profile_name}.pricing.{key}: value must not be NaN"
                    );
                }
                if *value < 0.0 {
                    anyhow::bail!(
                        "providers.models.{profile_name}.pricing.{key}: value must be >= 0.0 (got {value})"
                    );
                }
            }
        }

        // Non-fatal validation warnings: surfaced both via tracing (CLI sees
        // on stderr) and via Config::collect_warnings (gateway HTTP returns
        // structured to dashboard callers). Single source of truth lives in
        // collect_warnings; emit them to tracing here so the existing log
        // behavior is preserved, with the single documented exception below.
        for w in self.collect_warnings() {
            // One code is held back from this loop on purpose. The runtime
            // already records the withheld-capability notice itself, with an
            // explicit `System` category and the same code and path, once at
            // every config application. Records emitted here carry no category
            // and are stored as `internal`, which the dashboard Logs view hides
            // by default, so tracing it here as well leaves a hidden second copy
            // of a notice an operator is supposed to read. Every command that
            // loads config a second time inside a live process writes that pair.
            //
            // `collect_warnings` still returns it, so `omnesagent doctor` and the
            // config API report it exactly as before; only this tracing copy is
            // dropped. The change that re-registers the tool retires this skip
            // together with the runtime record it defers to.
            if w.code == crate::validation_warnings::VERIFIABLE_INTENT_TOOL_WITHHELD {
                continue;
            }
            ::omnesagent_log::record!(
                WARN,
                ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                    .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                    .with_attrs(::serde_json::json!({"path": w.path, "code": w.code})),
                &format!("{}", w.message)
            );
        }

        // Ollama cloud-routing safety checks
        for (alias, cfg) in &self.providers.models.ollama {
            let entry = &cfg.base;
            if !entry
                .model
                .as_deref()
                .is_some_and(|model| model.trim().ends_with(":cloud"))
            {
                continue;
            }

            if is_local_ollama_endpoint(entry.uri.as_deref()) {
                anyhow::bail!(
                    "providers.models.ollama.{alias}.model uses ':cloud', but uri is local or unset. Set uri to a remote Ollama endpoint (for example https://ollama.com)."
                );
            }
            if is_official_ollama_cloud_endpoint(entry.uri.as_deref())
                && !has_ollama_cloud_credential(entry.api_key.as_deref())
            {
                anyhow::bail!(
                    "providers.models.ollama.{alias}.model uses ':cloud', but no API key is configured. Set api_key on [providers.models.ollama.{alias}] (or via the schema-mirror grammar: OMNESAGENT_providers__models__ollama__{alias}__api_key=<value>)."
                );
            }
        }

        // Microsoft 365
        if self.microsoft365.enabled {
            let tenant = self
                .microsoft365
                .tenant_id
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty());
            if tenant.is_none() {
                anyhow::bail!(
                    "microsoft365.tenant_id must not be empty when microsoft365 is enabled"
                );
            }
            let client = self
                .microsoft365
                .client_id
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty());
            if client.is_none() {
                anyhow::bail!(
                    "microsoft365.client_id must not be empty when microsoft365 is enabled"
                );
            }
            let flow = self.microsoft365.auth_flow.trim();
            if flow != "client_credentials" && flow != "device_code" {
                anyhow::bail!(
                    "microsoft365.auth_flow must be 'client_credentials' or 'device_code'"
                );
            }
            if flow == "client_credentials"
                && self
                    .microsoft365
                    .client_secret
                    .as_deref()
                    .is_none_or(|s| s.trim().is_empty())
            {
                anyhow::bail!(
                    "microsoft365.client_secret must not be empty when auth_flow is 'client_credentials'"
                );
            }
        }

        // Microsoft 365
        if self.microsoft365.enabled {
            let tenant = self
                .microsoft365
                .tenant_id
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty());
            if tenant.is_none() {
                anyhow::bail!(
                    "microsoft365.tenant_id must not be empty when microsoft365 is enabled"
                );
            }
            let client = self
                .microsoft365
                .client_id
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty());
            if client.is_none() {
                anyhow::bail!(
                    "microsoft365.client_id must not be empty when microsoft365 is enabled"
                );
            }
            let flow = self.microsoft365.auth_flow.trim();
            if flow != "client_credentials" && flow != "device_code" {
                anyhow::bail!("microsoft365.auth_flow must be client_credentials or device_code");
            }
            if flow == "client_credentials"
                && self
                    .microsoft365
                    .client_secret
                    .as_deref()
                    .is_none_or(|s| s.trim().is_empty())
            {
                anyhow::bail!(
                    "microsoft365.client_secret must not be empty when auth_flow is client_credentials"
                );
            }
        }

        validate_plugin_entries(&self.plugins)?;
        validate_plugin_channel_instances(&self.channels)?;

        // MCP
        if self.mcp.enabled {
            validate_mcp_config(&self.mcp)?;
        }

        // Knowledge graph
        if self.knowledge.enabled {
            if self.knowledge.max_nodes == 0 {
                validation_bail!(
                    InvalidNumericRange,
                    "knowledge.max_nodes",
                    "knowledge.max_nodes must be greater than 0"
                );
            }
            if self.knowledge.db_path.trim().is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    "knowledge.db_path",
                    "knowledge.db_path must not be empty"
                );
            }
        }

        // Google Workspace allowed_services validation
        let mut seen_gws_services = std::collections::HashSet::new();
        for (i, service) in self.google_workspace.allowed_services.iter().enumerate() {
            let normalized = service.trim();
            if normalized.is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    format!("google_workspace.allowed_services[{i}]"),
                    "google_workspace.allowed_services[{i}] must not be empty"
                );
            }
            if !normalized
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
            {
                anyhow::bail!(
                    "google_workspace.allowed_services[{i}] contains invalid characters: {normalized}"
                );
            }
            if !seen_gws_services.insert(normalized.to_string()) {
                anyhow::bail!(
                    "google_workspace.allowed_services contains duplicate entry: {normalized}"
                );
            }
        }

        // Build the effective allowed-services set for cross-validation.
        // When the operator leaves allowed_services empty the tool falls back to
        // DEFAULT_GWS_SERVICES; use the same constant here so validation is
        // consistent in both cases.
        let effective_services: std::collections::HashSet<&str> =
            if self.google_workspace.allowed_services.is_empty() {
                DEFAULT_GWS_SERVICES.iter().copied().collect()
            } else {
                self.google_workspace
                    .allowed_services
                    .iter()
                    .map(|s| s.trim())
                    .collect()
            };

        let mut seen_gws_operations = std::collections::HashSet::new();
        for (i, operation) in self.google_workspace.allowed_operations.iter().enumerate() {
            let service = operation.service.trim();
            let resource = operation.resource.trim();

            if service.is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    format!("google_workspace.allowed_operations[{i}].service"),
                    "google_workspace.allowed_operations[{i}].service must not be empty"
                );
            }
            if resource.is_empty() {
                anyhow::bail!(
                    "google_workspace.allowed_operations[{i}].resource must not be empty"
                );
            }

            if !effective_services.contains(service) {
                anyhow::bail!(
                    "google_workspace.allowed_operations[{i}].service '{service}' is not in the \
                     effective allowed_services; this entry can never match at runtime"
                );
            }
            if !service
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
            {
                anyhow::bail!(
                    "google_workspace.allowed_operations[{i}].service contains invalid characters: {service}"
                );
            }
            // Unlike service IDs, resource/sub_resource/method names are camelCase
            // in the Google APIs (calendarList, quickAdd, batchUpdate), so
            // uppercase must be accepted here and in the runtime tool check.
            if !resource
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                anyhow::bail!(
                    "google_workspace.allowed_operations[{i}].resource contains invalid characters: {resource}"
                );
            }

            if let Some(ref sub_resource) = operation.sub_resource {
                let sub = sub_resource.trim();
                if sub.is_empty() {
                    anyhow::bail!(
                        "google_workspace.allowed_operations[{i}].sub_resource must not be empty when present"
                    );
                }
                if !sub
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                {
                    anyhow::bail!(
                        "google_workspace.allowed_operations[{i}].sub_resource contains invalid characters: {sub}"
                    );
                }
            }

            if operation.methods.is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    format!("google_workspace.allowed_operations[{i}].methods"),
                    "google_workspace.allowed_operations[{i}].methods must not be empty"
                );
            }

            let mut seen_methods = std::collections::HashSet::new();
            for (j, method) in operation.methods.iter().enumerate() {
                let normalized = method.trim();
                if normalized.is_empty() {
                    anyhow::bail!(
                        "google_workspace.allowed_operations[{i}].methods[{j}] must not be empty"
                    );
                }
                if !normalized
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                {
                    anyhow::bail!(
                        "google_workspace.allowed_operations[{i}].methods[{j}] contains invalid characters: {normalized}"
                    );
                }
                if !seen_methods.insert(normalized.to_string()) {
                    anyhow::bail!(
                        "google_workspace.allowed_operations[{i}].methods contains duplicate entry: {normalized}"
                    );
                }
            }

            let sub_key = operation
                .sub_resource
                .as_deref()
                .map(str::trim)
                .unwrap_or("");
            let operation_key = format!("{service}:{resource}:{sub_key}");
            if !seen_gws_operations.insert(operation_key.clone()) {
                anyhow::bail!(
                    "google_workspace.allowed_operations contains duplicate service/resource/sub_resource entry: {operation_key}"
                );
            }
        }

        // Project intelligence
        if self.project_intel.enabled {
            let lang = &self.project_intel.default_language;
            if !["en", "de", "fr", "it"].contains(&lang.as_str()) {
                anyhow::bail!(
                    "project_intel.default_language must be one of: en, de, fr, it (got '{lang}')"
                );
            }
            let sens = &self.project_intel.risk_sensitivity;
            if !["low", "medium", "high"].contains(&sens.as_str()) {
                anyhow::bail!(
                    "project_intel.risk_sensitivity must be one of: low, medium, high (got '{sens}')"
                );
            }
            if let Some(ref tpl_dir) = self.project_intel.templates_dir
                && !std::path::Path::new(tpl_dir).exists()
            {
                anyhow::bail!("project_intel.templates_dir path does not exist: {tpl_dir}");
            }
        }

        // Proxy (delegate to existing validation)
        self.proxy.validate()?;
        self.cloud_ops.validate()?;

        // Skills — extra registries
        {
            let mut seen = std::collections::HashSet::new();
            for (i, reg) in self.skills.extra_registries.iter().enumerate() {
                if reg.name.trim().is_empty() {
                    anyhow::bail!("skills.extra_registries[{i}].name must not be empty");
                }
                if !ExternalRegistry::is_valid_name(&reg.name) {
                    anyhow::bail!(
                        "skills.extra_registries[{i}].name '{}' is invalid; use only lowercase ASCII letters, numbers, '-' or '_' so it can be addressed as registry:<name>/<skill>",
                        reg.name
                    );
                }
                if !seen.insert(reg.name.as_str()) {
                    anyhow::bail!("skills.extra_registries has duplicate name '{}'", reg.name);
                }
                if reg.url.trim().is_empty() {
                    anyhow::bail!(
                        "skills.extra_registries[{}].url must not be empty",
                        reg.name
                    );
                }
                if reg.kind != ExternalRegistryKind::Git {
                    anyhow::bail!(
                        "skills.extra_registries[{}].kind must be 'git' (got '{}'); other protocols are not yet supported",
                        reg.name,
                        reg.kind
                    );
                }
                match reqwest::Url::parse(&reg.url) {
                    Ok(u) if matches!(u.scheme(), "http" | "https" | "file") => {}
                    Ok(u) => anyhow::bail!(
                        "skills.extra_registries[{}].url scheme '{}' is unsupported (use http, https, or file)",
                        reg.name,
                        u.scheme()
                    ),
                    Err(e) => anyhow::bail!(
                        "skills.extra_registries[{}].url is not a valid URL: {e}",
                        reg.name
                    ),
                }
            }
        }

        // Notion
        if self.notion.enabled {
            if self.notion.database_id.trim().is_empty() {
                anyhow::bail!("notion.database_id must not be empty when notion.enabled = true");
            }
            if self.notion.poll_interval_secs == 0 {
                validation_bail!(
                    InvalidNumericRange,
                    "notion.poll_interval_secs",
                    "notion.poll_interval_secs must be greater than 0"
                );
            }
            if self.notion.max_concurrent == 0 {
                validation_bail!(
                    InvalidNumericRange,
                    "notion.max_concurrent",
                    "notion.max_concurrent must be greater than 0"
                );
            }
            if self.notion.status_property.trim().is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    "notion.status_property",
                    "notion.status_property must not be empty"
                );
            }
            if self.notion.input_property.trim().is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    "notion.input_property",
                    "notion.input_property must not be empty"
                );
            }
            if self.notion.result_property.trim().is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    "notion.result_property",
                    "notion.result_property must not be empty"
                );
            }
        }

        // Pinggy tunnel region — validate allowed values (case-insensitive, auto-lowercased at runtime).
        if let Some(ref pinggy) = self.tunnel.pinggy
            && let Some(ref region) = pinggy.region
        {
            let r = region.trim().to_ascii_lowercase();
            if !r.is_empty() && !matches!(r.as_str(), "us" | "eu" | "ap" | "br" | "au") {
                anyhow::bail!(
                    "tunnel.pinggy.region must be one of: us, eu, ap, br, au (or omitted for auto)"
                );
            }
        }

        // Jira
        if self.jira.enabled {
            if self.jira.base_url.trim().is_empty() {
                anyhow::bail!("jira.base_url must not be empty when jira.enabled = true");
            }
            if self.jira.api_token.trim().is_empty()
                && std::env::var("JIRA_API_TOKEN")
                    .unwrap_or_default()
                    .trim()
                    .is_empty()
            {
                anyhow::bail!(
                    "jira.api_token must be set (or JIRA_API_TOKEN env var) when jira.enabled = true"
                );
            }
            let valid_actions = [
                "get_ticket",
                "search_tickets",
                "comment_ticket",
                "list_projects",
                "myself",
                "list_transitions",
                "transition_ticket",
                "create_ticket",
            ];
            for action in &self.jira.allowed_actions {
                if !valid_actions.contains(&action.as_str()) {
                    anyhow::bail!(
                        "jira.allowed_actions contains unknown action: '{}'. \
                         Valid: get_ticket, search_tickets, comment_ticket, list_projects, myself, list_transitions, transition_ticket, create_ticket",
                        action
                    );
                }
            }
        }

        // Nevis IAM — delegate to NevisConfig::validate() for field-level checks
        if let Err(msg) = self.security.nevis.validate() {
            anyhow::bail!("security.nevis: {msg}");
        }

        // Delegate tool global defaults
        if self.delegate.timeout_secs == 0 {
            validation_bail!(
                InvalidNumericRange,
                "delegate.timeout_secs",
                "delegate.timeout_secs must be greater than 0"
            );
        }
        if self.delegate.agentic_timeout_secs == 0 {
            validation_bail!(
                InvalidNumericRange,
                "delegate.agentic_timeout_secs",
                "delegate.agentic_timeout_secs must be greater than 0"
            );
        }

        // Per-profile validation: the whole-turn history-trim hysteresis
        // fraction must be in (0.0, 1.0]. Zero or negative would request an
        // empty refill target; anything above 1.0 would trim deeper than the
        // cap itself. Sorted iteration keeps error ordering stable.
        let mut profile_aliases: Vec<&String> = self.runtime_profiles.keys().collect();
        profile_aliases.sort();
        for palias in profile_aliases {
            let Some(low_water) = self.runtime_profiles[palias].history_trim_low_water else {
                continue;
            };
            if !low_water.is_finite() || low_water <= 0.0 || low_water > 1.0 {
                validation_bail!(
                    InvalidNumericRange,
                    format!("runtime_profiles.{palias}.history_trim_low_water"),
                    "runtime_profiles.{palias}.history_trim_low_water must be a finite number in (0.0, 1.0] (got {low_water})",
                );
            }
        }

        // Per-profile validation: the context-compression summarizer provider
        // ref must resolve to a configured `[providers.models.*]` alias.
        // Empty = inherit (valid). A shared profile fails loud at config time
        // instead of only when some agent using it compresses.
        let mut profile_aliases: Vec<&String> = self.runtime_profiles.keys().collect();
        profile_aliases.sort();
        for palias in profile_aliases {
            let value = self.runtime_profiles[palias]
                .context_compression
                .summary_provider
                .trim();
            if value.is_empty() {
                continue;
            }
            match value.split_once('.') {
                Some((ty, inner)) if !ty.is_empty() && !inner.is_empty() => {
                    let exists = self
                        .get_map_keys(&format!("providers.models.{ty}"))
                        .is_some_and(|keys| keys.iter().any(|k| k == inner));
                    if !exists {
                        validation_bail!(
                            DanglingReference,
                            format!(
                                "runtime_profiles.{palias}.context_compression.summary_provider"
                            ),
                            "runtime_profiles.{palias}.context_compression.summary_provider = {value:?} but providers.models.{ty}.{inner} is not configured",
                        );
                    }
                }
                _ => validation_bail!(
                    InvalidFormat,
                    format!("runtime_profiles.{palias}.context_compression.summary_provider"),
                    "runtime_profiles.{palias}.context_compression.summary_provider must be dotted form `<type>.<alias>` (got {value:?})",
                ),
            }
        }

        // Per-agent validation. Mandatory + alias-existence checks live
        // here so the gateway PATCH path returns structured per-field
        // errors and the frontend never owns this rule. Sorted iteration
        // keeps error ordering stable across runs.
        let mut agent_aliases: Vec<&String> = self.agents.keys().collect();
        agent_aliases.sort();
        for alias in agent_aliases {
            let agent = &self.agents[alias];

            // model_provider: mandatory, dotted `<type>.<inner>` ref into
            // model_providers.<type>.<inner>.
            let mp = agent.model_provider.trim();
            if mp.is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    format!("agents.{alias}.model_provider"),
                    "agents.{alias}.model_provider must reference a configured model model_provider (e.g. \"anthropic.default\")",
                );
            }
            match mp.split_once('.') {
                Some((ty, inner)) if !ty.is_empty() && !inner.is_empty() => {
                    if !crate::providers::ModelProviders::slot_names().contains(&ty) {
                        validation_bail!(
                            DanglingReference,
                            format!("agents.{alias}.model_provider"),
                            "agents.{alias}.model_provider = {mp:?} but {ty:?} is not a known provider family; check [providers.models.<family>.<alias>] in config.toml (valid families: `omnesagent providers`)",
                        );
                    }
                    let exists = self
                        .get_map_keys(&format!("providers.models.{ty}"))
                        .is_some_and(|keys| keys.iter().any(|k| k == inner));
                    if !exists {
                        validation_bail!(
                            DanglingReference,
                            format!("agents.{alias}.model_provider"),
                            "agents.{alias}.model_provider = {mp:?} but [providers.models.{ty}.{inner}] is not configured",
                        );
                    }
                }
                _ => validation_bail!(
                    InvalidFormat,
                    format!("agents.{alias}.model_provider"),
                    "agents.{alias}.model_provider must be dotted form `<type>.<alias>` (got {mp:?})",
                ),
            }

            // channels: each entry is a dotted `<type>.<inner>` ref into
            // channels.<type>.<inner>. Empty list is valid (delegate-only agent).
            // Uses the schema-derived `get_map_keys` so new channel types
            // surface here automatically — no per-type match arm.
            for (i, ch) in agent.channels.iter().enumerate() {
                let trimmed = ch.trim();
                match trimmed.split_once('.') {
                    Some((ty, inner)) if !ty.is_empty() && !inner.is_empty() => {
                        // `get_map_keys` stores section names using the raw
                        // field ident (snake), the same dotted form the
                        // operator sees in TOML (`gmail_push`, `voice_call`,
                        // `nextcloud_talk`). Look up verbatim.
                        let exists = self
                            .get_map_keys(&format!("channels.{ty}"))
                            .is_some_and(|keys| keys.iter().any(|k| k == inner));
                        if !exists {
                            validation_bail!(
                                DanglingReference,
                                format!("agents.{alias}.channels[{i}]"),
                                "agents.{alias}.channels[{i}] = {trimmed:?} but channels.{ty}.{inner} is not configured",
                            );
                        }
                    }
                    _ => validation_bail!(
                        InvalidFormat,
                        format!("agents.{alias}.channels[{i}]"),
                        "agents.{alias}.channels[{i}] must be dotted form `<type>.<alias>` (got {trimmed:?})",
                    ),
                }
            }

            // Per-agent provider refs that resolve into the typed provider
            // sections. Empty = no preference for that category (no TTS / no
            // STT for this agent), which is valid. Non-empty values must
            // match a configured `[providers.<category>.<type>.<alias>]`
            // entry, fail loud with the dangling ref otherwise.
            // there is no global default-X-provider concept — every consumer
            // either picks a configured alias or opts out entirely.
            let typed_provider_refs: &[(&str, &str, &str)] = &[
                ("providers.tts", "tts_provider", agent.tts_provider.trim()),
                (
                    "providers.transcription",
                    "transcription_provider",
                    agent.transcription_provider.trim(),
                ),
                // New field:
                (
                    "providers.models",
                    "classifier_provider",
                    agent.classifier_provider.trim(),
                ),
                // Agent-level context-compression summarizer override.
                (
                    "providers.models",
                    "summary_provider",
                    agent.summary_provider.trim(),
                ),
            ];
            for (section_prefix, field, value) in typed_provider_refs {
                if value.is_empty() {
                    continue;
                }
                match value.split_once('.') {
                    Some((ty, inner)) if !ty.is_empty() && !inner.is_empty() => {
                        let exists = self
                            .get_map_keys(&format!("{section_prefix}.{ty}"))
                            .is_some_and(|keys| keys.iter().any(|k| k == inner));
                        if !exists {
                            validation_bail!(
                                DanglingReference,
                                format!("agents.{alias}.{field}"),
                                "agents.{alias}.{field} = {value:?} but {section_prefix}.{ty}.{inner} is not configured",
                            );
                        }
                    }
                    _ => validation_bail!(
                        InvalidFormat,
                        format!("agents.{alias}.{field}"),
                        "agents.{alias}.{field} must be dotted form `<type>.<alias>` (got {value:?})",
                    ),
                }
            }

            // Bare-alias bundle refs. Tuple is (kebab section path, kebab
            // agent field name, value list). Both names use the schema's
            // kebab form: section name matches what `get_map_keys` expects
            // (macro converts snake→kebab via `snake_to_kebab` per
            // crates/omnesagent-macros/src/lib.rs:1056); field name matches
            // what `prop_fields()` emits, so DanglingReference paths bind
            // directly to the right inline error in the dashboard form.
            let bare_multi: &[(&str, &str, &[String])] = &[
                ("skill_bundles", "skill_bundles", &agent.skill_bundles),
                (
                    "knowledge_bundles",
                    "knowledge_bundles",
                    &agent.knowledge_bundles,
                ),
                ("mcp_bundles", "mcp_bundles", &agent.mcp_bundles),
            ];
            for (section, field, values) in bare_multi {
                for (i, key) in values.iter().enumerate() {
                    let trimmed = key.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let exists = self
                        .get_map_keys(section)
                        .is_some_and(|keys| keys.iter().any(|k| k == trimmed));
                    if !exists {
                        validation_bail!(
                            DanglingReference,
                            format!("agents.{alias}.{field}[{i}]"),
                            "agents.{alias}.{field}[{i}] = {trimmed:?} but {section}.{trimmed} is not configured",
                        );
                    }
                }
            }
            let bare_single: &[(&str, &str, &str)] = &[
                ("risk_profiles", "risk_profile", agent.risk_profile.as_str()),
                (
                    "runtime_profiles",
                    "runtime_profile",
                    agent.runtime_profile.as_str(),
                ),
            ];
            for (section, field, raw) in bare_single {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let exists = self
                    .get_map_keys(section)
                    .is_some_and(|keys| keys.iter().any(|k| k == trimmed));
                if !exists {
                    validation_bail!(
                        DanglingReference,
                        format!("agents.{alias}.{field}"),
                        "agents.{alias}.{field} = {trimmed:?} but {section}.{trimmed} is not configured",
                    );
                }
            }

            // risk_profile is mandatory for enabled agents — there is no
            // global fallback, so an enabled agent with no profile can't
            // gate its actions. Run this check last so the more specific
            // dangling/format errors above surface first.
            if agent.enabled && agent.risk_profile.trim().is_empty() {
                validation_bail!(
                    RequiredFieldEmpty,
                    format!("agents.{alias}.risk_profile"),
                    "agents.{alias}.risk_profile must reference a configured [risk_profiles.<alias>] entry",
                );
            }

            // delegates: explicit roster entries must point at OTHER
            // configured agents, never self. Cross-profile targets are
            // permitted (they run under the target's own policy), so no
            // profile match is required here.
            let mut seen_delegates: std::collections::BTreeSet<&str> =
                std::collections::BTreeSet::new();
            for (i, target) in agent.delegates.iter().enumerate() {
                let target_str = target.agent().trim();
                if target_str.is_empty() {
                    validation_bail!(
                        RequiredFieldEmpty,
                        format!("agents.{alias}.delegates[{i}].agent"),
                        "agents.{alias}.delegates[{i}].agent is empty; remove it or name a configured agent",
                    );
                }
                if target_str == alias.as_str() {
                    validation_bail!(
                        InvalidFormat,
                        format!("agents.{alias}.delegates[{i}].agent"),
                        "agents.{alias}.delegates[{i}].agent = {target_str:?} names this agent itself; an agent cannot delegate to itself",
                    );
                }
                if !self.agents.contains_key(target_str) {
                    validation_bail!(
                        DanglingReference,
                        format!("agents.{alias}.delegates[{i}].agent"),
                        "agents.{alias}.delegates[{i}].agent = {target_str:?} but agents.{target_str} is not configured",
                    );
                }
                if !seen_delegates.insert(target_str) {
                    validation_bail!(
                        InvalidFormat,
                        format!("agents.{alias}.delegates[{i}].agent"),
                        "agents.{alias}.delegates[{i}].agent = {target_str:?} duplicates an earlier delegate target",
                    );
                }
            }

            // workspace.access: keys must point at OTHER agents, never
            // self, and every target must be a configured agent.
            for (target, mode) in &agent.workspace.access {
                let target_str = target.as_str();
                if target_str == alias.as_str() {
                    validation_bail!(
                        InvalidFormat,
                        format!("agents.{alias}.workspace.access.{target_str}"),
                        "agents.{alias}.workspace.access.{target_str} = {mode:?} but {target_str} is this agent itself; an agent always has full access to its own workspace, so self-references in the cross-agent allowlist are not permitted",
                    );
                }
                if !self.agents.contains_key(target_str) {
                    validation_bail!(
                        DanglingReference,
                        format!("agents.{alias}.workspace.access.{target_str}"),
                        "agents.{alias}.workspace.access.{target_str} = {mode:?} but agents.{target_str} is not configured",
                    );
                }
            }

            // workspace.read_memory_from: every alias must exist as a
            // configured agent and must use the same MemoryBackendKind
            // as the declaring agent. Mismatched backends fail at
            // config load rather than producing a runtime error when
            // the per-agent memory plumbing consumes the allowlist.
            let agent_backend = agent.memory.backend;
            for (i, target) in agent.workspace.read_memory_from.iter().enumerate() {
                let target_str = target.as_str();
                if target_str == alias.as_str() {
                    validation_bail!(
                        InvalidFormat,
                        format!("agents.{alias}.workspace.read_memory_from[{i}]"),
                        "agents.{alias}.workspace.read_memory_from[{i}] = {target_str:?} but {target_str} is this agent itself; an agent always sees its own memory rows, so self-references in the cross-agent allowlist are not permitted",
                    );
                }
                let Some(target_agent) = self.agents.get(target_str) else {
                    validation_bail!(
                        DanglingReference,
                        format!("agents.{alias}.workspace.read_memory_from[{i}]"),
                        "agents.{alias}.workspace.read_memory_from[{i}] = {target_str:?} but agents.{target_str} is not configured",
                    );
                };
                if target_agent.memory.backend != agent_backend {
                    let target_backend = target_agent.memory.backend;
                    validation_bail!(
                        InvalidFormat,
                        format!("agents.{alias}.workspace.read_memory_from[{i}]"),
                        "agents.{alias}.workspace.read_memory_from[{i}] points at agents.{target_str} which uses memory backend {target_backend:?}, but agents.{alias} uses {agent_backend:?}; the allowlist must point at same-backend siblings only",
                    );
                }
            }
        }

        // Peer groups: every member alias must exist as a configured
        // agent, and the group's channel must be in each member's
        // channels list. Mutual opt-in resolution happens at runtime;
        // this cross-reference check keeps misconfigured group
        // members from looking like real peer relationships at load
        // time.
        let mut peer_group_names: Vec<&String> = self.peer_groups.keys().collect();
        peer_group_names.sort();
        for group_name in peer_group_names {
            let group = &self.peer_groups[group_name];
            let group_channel = group.channel.trim();
            // The channel ref is resolved once by the shared classifier; its
            // bound fields feed the error messages below. The split_once
            // locals exist only for the member-ref loop further down, which
            // is out of scope for the classifier.
            let (group_channel_type, group_channel_alias) = match group_channel.split_once('.') {
                Some((ty, al)) => (ty, Some(al)),
                None => (group_channel, None),
            };
            match self.classify_peer_group_channel_ref(group_channel) {
                PeerGroupChannelRef::Empty => {
                    validation_bail!(
                        RequiredFieldEmpty,
                        format!("peer_groups.{group_name}.channel"),
                        "peer_groups.{group_name}.channel must name a channel type (e.g. \"discord\") or dotted alias (e.g. \"discord.work\")",
                    );
                }
                PeerGroupChannelRef::BareType {
                    channel_type,
                    configured,
                } => {
                    if !configured {
                        validation_bail!(
                            DanglingReference,
                            format!("peer_groups.{group_name}.channel"),
                            "peer_groups.{group_name}.channel = {group_channel:?} but no [channels.{channel_type}.*] block is configured",
                        );
                    }
                }
                PeerGroupChannelRef::Dotted {
                    channel_type,
                    alias,
                    type_configured,
                    exists,
                    ..
                } => {
                    if !type_configured {
                        validation_bail!(
                            DanglingReference,
                            format!("peer_groups.{group_name}.channel"),
                            "peer_groups.{group_name}.channel = {group_channel:?} but no [channels.{channel_type}.*] block is configured",
                        );
                    }
                    if !exists {
                        validation_bail!(
                            DanglingReference,
                            format!("peer_groups.{group_name}.channel"),
                            "peer_groups.{group_name}.channel = {group_channel:?} but [channels.{channel_type}.{alias}] is not configured",
                        );
                    }
                }
            }
            for (i, member) in group.agents.iter().enumerate() {
                let member_str = member.as_str();
                let Some(member_agent) = self.agents.get(member_str) else {
                    validation_bail!(
                        DanglingReference,
                        format!("peer_groups.{group_name}.agents[{i}]"),
                        "peer_groups.{group_name}.agents[{i}] = {member_str:?} but agents.{member_str} is not configured",
                    );
                };
                let has_channel_match = member_agent.channels.iter().any(|ch| {
                    let ch_str = ch.as_str();
                    match group_channel_alias {
                        Some(alias) => ch_str == format!("{group_channel_type}.{alias}"),
                        None => ch_str.starts_with(&format!("{group_channel_type}.")),
                    }
                });
                if !has_channel_match {
                    let needs_msg = match group_channel_alias {
                        Some(alias) => format!("entry for {group_channel_type}.{alias}"),
                        None => format!("entry of type {group_channel_type:?}"),
                    };
                    validation_bail!(
                        InvalidFormat,
                        format!("peer_groups.{group_name}.agents[{i}]"),
                        "peer_groups.{group_name}.agents[{i}] = {member_str:?} but agents.{member_str}.channels has no {needs_msg}",
                    );
                }
            }
        }

        if self.plugins.max_active_instances == 0 {
            validation_bail!(
                InvalidNumericRange,
                "plugins.max_active_instances",
                "plugins.max_active_instances must be greater than 0; a zero ceiling rejects every logical plugin instance"
            );
        }
        if self.plugins.limits.call_fuel == 0 {
            validation_bail!(
                InvalidNumericRange,
                "plugins.limits.call_fuel",
                "plugins.limits.call_fuel must be greater than 0; a zero budget traps every plugin call before it runs"
            );
        }
        if self.plugins.limits.max_memory_mb == 0 {
            validation_bail!(
                InvalidNumericRange,
                "plugins.limits.max_memory_mb",
                "plugins.limits.max_memory_mb must be greater than 0; a zero cap rejects every plugin at instantiation"
            );
        }
        if self.plugins.limits.max_table_elements == 0 {
            validation_bail!(
                InvalidNumericRange,
                "plugins.limits.max_table_elements",
                "plugins.limits.max_table_elements must be greater than 0; a zero ceiling rejects every plugin that allocates a table"
            );
        }
        if self.plugins.limits.max_instances == 0 {
            validation_bail!(
                InvalidNumericRange,
                "plugins.limits.max_instances",
                "plugins.limits.max_instances must be greater than 0; a zero ceiling rejects every plugin at instantiation"
            );
        }
        if self.plugins.limits.call_timeout_ms == 0 {
            validation_bail!(
                InvalidNumericRange,
                "plugins.limits.call_timeout_ms",
                "plugins.limits.call_timeout_ms must be greater than 0; a zero deadline aborts every plugin call before it runs"
            );
        }
        if self.plugins.limits.max_connections_per_instance == 0 {
            validation_bail!(
                InvalidNumericRange,
                "plugins.limits.max_connections_per_instance",
                "plugins.limits.max_connections_per_instance must be greater than 0; a zero ceiling rejects every plugin network connection"
            );
        }

        // The granted egress allowlist is a security control, so a malformed
        // entry is a hard config error rather than a silently dropped line.
        // Both lists validate against the one shared strict grammar in
        // `omnesagent_infra::net_guard`, including containment of the private
        // address carveout by the host grant.
        for entry in &self.plugins.entries {
            let hosts = {
                let path = format!("plugins.entries.{}.egress_hosts", entry.name);
                match omnesagent_infra::net_guard::normalize_egress_patterns(
                    &entry.egress_hosts,
                    &path,
                ) {
                    Ok(patterns) => patterns,
                    Err(e) => validation_bail!(InvalidFormat, path, "{}", e),
                }
            };
            let private = {
                let path = format!("plugins.entries.{}.egress_allow_private", entry.name);
                match omnesagent_infra::net_guard::normalize_egress_patterns(
                    &entry.egress_allow_private,
                    &path,
                ) {
                    Ok(patterns) => patterns,
                    Err(e) => validation_bail!(InvalidFormat, path, "{}", e),
                }
            };

            // A carveout for a host that was never granted is almost always a
            // typo, and silently ignoring it leaves an operator believing they
            // opened a path they did not.
            for private in &private {
                if !hosts
                    .iter()
                    .any(|grant| omnesagent_infra::net_guard::egress_pattern_contains(grant, private))
                {
                    validation_bail!(
                        InvalidFormat,
                        format!("plugins.entries.{}.egress_allow_private", entry.name),
                        "plugins.entries.{}.egress_allow_private lists {private:?}, which is not granted by egress_hosts; the carveout relaxes an address class for a granted destination, it does not grant one. A wildcard carveout ('*.host') needs an equal-or-broader wildcard grant, not an exact one",
                        entry.name
                    );
                }
            }
        }

        Ok(())
    }

    pub fn mark_dirty(&mut self, path: &str) {
        self.dirty_paths.insert(path.to_string());
    }

    /// Auto-create the parent map key for a dotted set-prop `path` when it does
    /// not yet exist, so a `set_prop` on a brand-new alias's field materializes
    /// the entry first.
    ///
    /// Returns `true` iff it REFUSED to create the reserved `default` agent: a
    /// set-prop surface should then surface a reserved-name error rather than the
    /// generic "not configured" that the still-missing entry would otherwise
    /// produce. Returns `false` in every other case (created, already existed, or
    /// the path is not a map-keyed entry). The bool is advisory; statement-callers
    /// that do not distinguish the reserved case may ignore it.
    ///
    /// When it newly creates the alias, it also guarantees no phantom is left
    /// behind: if `path` still doesn't resolve afterward (e.g. an unknown tail
    /// field), the just-created alias is rolled back before returning, so the
    /// caller never observes a half-materialized entry from this probe alone.
    ///
    /// Failures that happen *after* this returns (value coercion, a masked
    /// secret, or a `set_prop` parse error) are the caller's to contain. The
    /// RPC `config_set` path does so by mutating a cloned `Config` and only
    /// committing it on full success, so a discarded attempt drops the alias
    /// with the clone — no tracked-tuple mirror of config transaction state is
    /// exposed to the runtime.
    ///
    /// `#[resource_key]` sections are excluded. Their keys are values drawn from
    /// another domain (model id, voice, tool name) and may themselves contain
    /// dots, so the first-dot split below would read
    /// `cost.rates.providers.models.openai.gpt-4.1.input_per_mtok` as the bogus
    /// alias `gpt-4` plus a nonsense tail, and `create_map_key` skips
    /// `validate_alias_key` for those sections so nothing else rejects it.
    /// Rate rows are created explicitly through `POST /api/config/map-key`
    /// instead.
    pub fn ensure_map_key_for_path(&mut self, path: &str) -> bool {
        use crate::traits::MapKeyKind;
        let mut best: Option<&'static str> = None;
        for s in Self::map_key_sections()
            .iter()
            .filter(|s| s.kind == MapKeyKind::Map)
            .filter(|s| !s.resource_key)
        {
            let prefix = format!("{}.", s.path);
            if path.starts_with(&prefix)
                && path.len() > prefix.len()
                && best.is_none_or(|b| s.path.len() > b.len())
            {
                best = Some(s.path);
            }
        }
        let Some(section) = best else {
            return false;
        };
        let rest = &path[section.len() + 1..];
        let Some(alias) = rest.split('.').next().filter(|a| !a.is_empty()) else {
            return false;
        };
        if self
            .get_map_keys(section)
            .is_some_and(|keys| keys.iter().any(|k| k == alias))
        {
            return false;
        }
        // Never auto-vivify the reserved `default` agent from a set-prop path: a
        // prop write under a nonexistent `agents.default` must not materialize the
        // reserved runtime fallback (the operator would get an agent the rename
        // guard then traps). This is the set-prop analogue of the create guard in
        // `create_map_key_checked`. The migration that legitimately synthesizes
        // `agents.default` calls `create_map_key` directly, not this path, and an
        // already-present `default` is left untouched by the existence check above.
        if section == "agents" && crate::alias_refs::is_reserved_agent_alias(alias) {
            // Make the refusal observable: callers that check the return surface a
            // reserved-name error, and this WARN with a stable `error_key` lets an
            // operator see why a set-prop on a fresh `agents.default` was dropped.
            ::omnesagent_log::record!(
                WARN,
                ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Reject)
                    .with_outcome(::omnesagent_log::EventOutcome::Failure)
                    .with_attrs(::serde_json::json!({
                        "error_key": "config.reserved_agent_vivify_refused",
                        "alias": alias,
                        "path": path,
                    })),
                "refused to auto-create the reserved `default` agent from a set-prop path"
            );
            return true;
        }
        // Roll back on the same `Ok(true)` == "newly created" signal the CLI
        // helper `ensure_map_key_for_prop_path` (src/main.rs) uses: never gate
        // this on the earlier `get_map_keys` pre-check, or a bogus tail field
        // on an already-existing alias would delete a legitimate config entry.
        if matches!(self.create_map_key(section, alias), Ok(true))
            && self.get_prop(path).is_err()
            // A missing entry under a dynamic secret map is a valid first
            // write, not an unknown schema tail. `get_prop` can only read
            // keys that already exist, while the generated `set_prop`
            // intentionally inserts them. The static secret classifier
            // recognizes the map prefix without requiring the key to exist.
            && !Self::prop_is_secret(path)
        {
            let _ = self.delete_map_key(section, alias);
        }
        false
    }

    pub fn clear_dirty(&mut self) {
        self.dirty_paths.clear();
    }

    /// Refuse paths whose bytes identify both a complete live map key and a
    /// field below a shorter live key. Mutation routes to the shorter key,
    /// while whole-entry persistence must treat the exact key as canonical;
    /// rejecting before either operation keeps memory and disk aligned.
    fn reject_ambiguous_persistent_map_key_path(&self, name: &str) -> Result<()> {
        let Some(section) = find_map_key_section_for_path(name) else {
            return Ok(());
        };
        let remainder = &name[section.path.len() + 1..];
        let Some(keys) = self.get_map_keys(section.path) else {
            return Ok(());
        };
        if !keys.iter().any(|key| key == remainder) {
            return Ok(());
        }

        let Some((field_key, inner_name)) = crate::helpers::route_hashmap_path(
            name,
            "",
            section.path,
            "",
            keys.iter().map(String::as_str),
        ) else {
            return Ok(());
        };

        anyhow::bail!(
            "Ambiguous config path `{name}`: it could mean the map entry `{}[{:?}]` or property `{}[{:?}].{inner_name}`. Refusing to choose; rename one of the colliding map keys",
            section.path,
            remainder,
            section.path,
            field_key,
        )
    }

    pub fn set_prop_persistent(&mut self, name: &str, value_str: &str) -> Result<()> {
        self.reject_ambiguous_persistent_map_key_path(name)?;
        self.set_prop(name, value_str)?;
        self.mark_dirty(name);
        Ok(())
    }

    pub fn set_secret_persistent(&mut self, name: &str, value: String) -> Result<()> {
        self.reject_ambiguous_persistent_map_key_path(name)?;
        self.set_secret(name, value)?;
        self.mark_dirty(name);
        Ok(())
    }

    async fn resolve_config_path_for_save(&self) -> Result<PathBuf> {
        if self
            .config_path
            .parent()
            .is_some_and(|parent| !parent.as_os_str().is_empty())
        {
            return Ok(self.config_path.clone());
        }

        let (default_omnesagent_dir, default_workspace_dir) = default_config_and_data_dirs()?;
        let (omnesagent_dir, _workspace_dir, source) =
            resolve_runtime_config_dirs(&default_omnesagent_dir, &default_workspace_dir).await?;
        let file_name = self
            .config_path
            .file_name()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| std::ffi::OsStr::new("config.toml"));
        let resolved = omnesagent_dir.join(file_name);
        if tokio::fs::try_exists(&resolved).await.with_context(|| {
            format!(
                "Failed to check resolved config path {}",
                resolved.display()
            )
        })? {
            anyhow::bail!(
                "Config path {} has no parent directory and resolves to {}; refusing to overwrite existing config",
                self.config_path.display(),
                resolved.display()
            );
        }
        ::omnesagent_log::record!(WARN, ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note).with_outcome(::omnesagent_log::EventOutcome::Unknown).with_attrs(::serde_json::json!({"path": self.config_path.display().to_string(), "resolved": resolved.display().to_string(), "source": source.as_str()})), "Config path missing parent directory; resolving from runtime environment");
        Ok(resolved)
    }

    pub async fn save(&self) -> Result<()> {
        // Encrypt secrets before serialization
        let mut config_to_save = self.clone();
        // Stamp the current schema version on every write. The in-memory
        // config is always at `CURRENT_SCHEMA_VERSION` (load-time migration
        // brings it forward), but pin it explicitly so a full save can never
        // emit a body-newer-than-label file. See `save_dirty` and.
        config_to_save.schema_version = crate::migration::CURRENT_SCHEMA_VERSION;
        let config_path = self.resolve_config_path_for_save().await?;
        let omnesagent_dir = config_path
            .parent()
            .context("Config path must have a parent directory")?;
        let store = crate::secrets::SecretStore::new(omnesagent_dir, self.secrets.encrypt);

        // Restore env-overridden paths to their pre-override snapshots before
        // encryption, so values supplied via `OMNESAGENT_*` env vars never reach
        // disk. Snapshots were captured at apply time from the post-decrypt
        // in-memory state, so secrets carry the original plaintext that
        // `encrypt_secrets()` will re-encrypt to fresh ciphertext that
        // decrypts back to the same value.
        if !self.pre_override_snapshots.is_empty() {
            crate::env_overrides::mask_env_overrides_for_save(
                &mut config_to_save,
                &self.pre_override_snapshots,
            )?;
        }
        restore_onepassword_references_for_save(
            &mut config_to_save,
            &self.onepassword_reference_snapshots,
            &self.dirty_paths,
        )?;

        // Encrypt all #[secret]-annotated fields via Configurable derive
        config_to_save.encrypt_secrets(&store)?;

        // Serialize, then prune fields whose values match
        // `Config::default()` so the on-disk config carries only the
        // operator's actual choices (no hundreds of lines of struct
        // defaults the operator never touched). The schema's
        // `#[serde(default = "...")]` annotations re-supply the
        // defaults on load, so the pruned file round-trips identically.
        let mut new_table: toml::Table = toml::Value::try_from(&config_to_save)
            .context("Failed to serialize config to TOML value")?
            .try_into()
            .context("Serialized config is not a TOML table")?;
        let default_table: toml::Table = toml::Value::try_from(Config::default())
            .ok()
            .and_then(|v| v.try_into().ok())
            .unwrap_or_default();
        prune_default_values(&mut new_table, &default_table);
        let new_toml = ensure_blank_line_before_sections(
            &toml::to_string_pretty(&new_table).context("Failed to serialize pruned config")?,
        );

        // If an existing config file is present, sync the new values onto it
        // to preserve comments and formatting. Otherwise, use the fresh serialization.
        let toml_str = if config_path.exists() {
            let existing = fs::read_to_string(&config_path).await.unwrap_or_default();
            if existing.is_empty() {
                new_toml
            } else {
                let mut doc: toml_edit::DocumentMut = existing
                    .parse()
                    .context("Failed to parse existing config for comment preservation")?;
                crate::migration::sync_table(doc.as_table_mut(), &new_table);
                // sync_table preserves existing decor verbatim, so newly
                // inserted sections lack the blank-line gap before their
                // header until the post-processor runs.
                ensure_blank_line_before_sections(&doc.to_string())
            }
        } else {
            new_toml
        };

        write_config_atomically(&config_path, &toml_str).await
    }

    /// Incremental save: only the paths in `self.dirty_paths` are written
    /// against the existing on-disk file. Non-dirty entries (including
    /// secret ciphertext) are left untouched; dirty paths whose value
    /// equals the schema default are removed from the doc instead of
    /// written. Falls back to a full `save()` when the file doesn't
    /// exist yet. Clears the dirty set on success.
    pub async fn save_dirty(&mut self) -> Result<()> {
        if self.dirty_paths.is_empty() {
            return Ok(());
        }

        let config_path = self.resolve_config_path_for_save().await?;
        if !config_path.exists() {
            let result = self.save().await;
            if result.is_ok() {
                self.clear_dirty();
            }
            return result;
        }

        let mut config_to_save = self.clone();
        let omnesagent_dir = config_path
            .parent()
            .context("Config path must have a parent directory")?;
        let store = crate::secrets::SecretStore::new(omnesagent_dir, self.secrets.encrypt);

        if !self.pre_override_snapshots.is_empty() {
            crate::env_overrides::mask_env_overrides_for_save(
                &mut config_to_save,
                &self.pre_override_snapshots,
            )?;
        }
        restore_onepassword_references_for_save(
            &mut config_to_save,
            &self.onepassword_reference_snapshots,
            &self.dirty_paths,
        )?;
        config_to_save.encrypt_secrets(&store)?;

        let full_table: toml::Table = toml::Value::try_from(&config_to_save)
            .context("Failed to serialize config to TOML value")?
            .try_into()
            .context("Serialized config is not a TOML table")?;
        let default_table: toml::Table = toml::Value::try_from(Config::default())
            .ok()
            .and_then(|v| v.try_into().ok())
            .unwrap_or_default();

        let existing = fs::read_to_string(&config_path).await.with_context(|| {
            format!(
                "Failed to read existing config for incremental save: {}",
                config_path.display()
            )
        })?;
        let mut doc: toml_edit::DocumentMut = existing
            .parse()
            .context("Failed to parse existing config for incremental save")?;

        for path in &self.dirty_paths {
            apply_dirty_path(doc.as_table_mut(), path, &full_table, &default_table)?;
        }

        // Stamp the current schema version. An incremental save writes
        // current-schema-shaped sections (e.g. the dashboard saving a single
        // `agents.<name>.model_provider`) but `schema_version` is never a
        // dirty path, so without this it keeps whatever an older binary first
        // wrote on disk. The resulting body-newer-than-label file then crashes
        // older binaries with an opaque `missing field ...` serde error.
        // `insert` updates the existing key in place (preserving its
        // position at the top of the file) or appends it when absent.
        doc.as_table_mut().insert(
            "schema_version",
            toml_edit::value(i64::from(crate::migration::CURRENT_SCHEMA_VERSION)),
        );

        let toml_str = ensure_blank_line_before_sections(&doc.to_string());

        write_config_atomically(&config_path, &toml_str).await?;
        self.clear_dirty();
        Ok(())
    }
}

fn collect_onepassword_reference_snapshots(
    config: &Config,
) -> std::collections::HashMap<String, String> {
    config
        .prop_fields()
        .into_iter()
        .filter(|field| field.is_secret)
        .filter_map(|field| {
            let value = crate::env_overrides::raw_value_for_path(config, &field.name)?;
            crate::secrets::SecretStore::is_onepassword_ref(&value).then_some((field.name, value))
        })
        .collect()
}

fn restore_onepassword_references_for_save(
    config_to_save: &mut Config,
    snapshots: &std::collections::HashMap<String, String>,
    dirty_paths: &std::collections::HashSet<String>,
) -> Result<()> {
    for (path, value) in snapshots {
        if dirty_paths.contains(path) {
            continue;
        }
        if let Err(err) = config_to_save.set_prop(path, value) {
            ::omnesagent_log::record!(
                WARN,
                ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                    .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                    .with_attrs(::serde_json::json!({"path": path, "error": format!("{}", err)})),
                "1Password reference save-restore failed; field retains resolved value"
            );
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub(super) enum PostReplaceSync {
    Real,
    #[cfg(any(test, feature = "test-helpers"))]
    FailForTest,
}

/// Test-only fault injection: when armed for a specific config path, the
/// next atomic write to that path simulates a post-rename directory sync
/// failure. This lets downstream crates (e.g. the runtime's `config/set`
/// RPC tests) drive the injected fault through the full production save
/// path instead of calling the write helper directly. Keying by path keeps
/// concurrently running tests from consuming each other's fault.
/// Unreachable from production builds: the hook only exists under `test`
/// or the `test-helpers` feature, which no production dependency enables.
#[cfg(any(test, feature = "test-helpers"))]
static FAIL_POST_REPLACE_SYNC_FOR_PATHS: std::sync::Mutex<Vec<std::path::PathBuf>> =
    std::sync::Mutex::new(Vec::new());

/// Arm a one-shot post-rename sync failure for the next atomic write to
/// `config_path`.
#[cfg(any(test, feature = "test-helpers"))]
pub fn arm_post_replace_sync_failure_for_test(config_path: &Path) {
    FAIL_POST_REPLACE_SYNC_FOR_PATHS
        .lock()
        .unwrap()
        .push(config_path.to_path_buf());
}

/// Whether an armed post-rename sync failure is still pending for
/// `config_path`. Tests use this to prove the fault actually fired (the
/// entry is consumed) rather than being bypassed by a save path that never
/// reached the atomic writer.
#[cfg(any(test, feature = "test-helpers"))]
pub fn post_replace_sync_failure_armed(config_path: &Path) -> bool {
    FAIL_POST_REPLACE_SYNC_FOR_PATHS
        .lock()
        .unwrap()
        .iter()
        .any(|p| p == config_path)
}

#[cfg(any(test, feature = "test-helpers"))]
fn take_post_replace_sync_failure(config_path: &Path) -> bool {
    let mut armed = FAIL_POST_REPLACE_SYNC_FOR_PATHS.lock().unwrap();
    if let Some(idx) = armed.iter().position(|p| p == config_path) {
        armed.swap_remove(idx);
        true
    } else {
        false
    }
}

/// Atomic write shared by `save()` and `save_dirty()`.
async fn write_config_atomically(config_path: &Path, toml_str: &str) -> Result<()> {
    #[cfg(any(test, feature = "test-helpers"))]
    if take_post_replace_sync_failure(config_path) {
        return write_config_atomically_with_sync(
            config_path,
            toml_str,
            PostReplaceSync::FailForTest,
        )
        .await;
    }
    write_config_atomically_with_sync(config_path, toml_str, PostReplaceSync::Real).await
}

pub(super) async fn write_config_atomically_with_sync(
    config_path: &Path,
    toml_str: &str,
    post_replace_sync: PostReplaceSync,
) -> Result<()> {
    let parent_dir = config_path
        .parent()
        .context("Config path must have a parent directory")?;

    fs::create_dir_all(parent_dir).await.with_context(|| {
        format!(
            "Failed to create config directory: {}",
            parent_dir.display()
        )
    })?;

    // Open and sync the directory before replacement so permission, handle,
    // and platform failures are reported while disk and live config are still
    // unchanged. A second sync after rename establishes rename durability.
    sync_directory(parent_dir).await?;

    let file_name = config_path
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("config.toml");
    let temp_path = parent_dir.join(format!(".{file_name}.tmp-{}", uuid::Uuid::new_v4()));
    let backup_path = parent_dir.join(format!("{file_name}.bak"));

    let mut temp_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp_path)
        .await
        .with_context(|| {
            format!(
                "Failed to create temporary config file: {}",
                temp_path.display()
            )
        })?;
    temp_file
        .write_all(toml_str.as_bytes())
        .await
        .context("Failed to write temporary config contents")?;
    temp_file
        .sync_all()
        .await
        .context("Failed to fsync temporary config file")?;
    drop(temp_file);

    let had_existing_config = config_path.exists();
    if had_existing_config {
        fs::copy(config_path, &backup_path).await.with_context(|| {
            format!(
                "Failed to create config backup before atomic replace: {}",
                backup_path.display()
            )
        })?;
        // The post-rename uncertainty path retains this copy as recovery
        // material, so make that promise durable before the replacement can
        // become visible. `copy` alone only establishes an immediately
        // readable file; syncing both its contents and the parent directory
        // pins the backup data and directory entry across a crash. Any failure
        // here is still pre-commit and therefore returns with `config.toml`
        // unchanged.
        // The handle must carry write access: on Windows `sync_all` reaches
        // `FlushFileBuffers`, which rejects a handle opened without
        // `GENERIC_WRITE`. `write(true)` without `truncate`/`create_new`
        // keeps the freshly copied bytes intact and only widens the access
        // rights, so the fsync below is portable rather than Unix-only.
        let backup_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&backup_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to open config backup for fsync: {}",
                    backup_path.display()
                )
            })?;
        backup_file
            .sync_all()
            .await
            .with_context(|| format!("Failed to fsync config backup: {}", backup_path.display()))?;
        sync_directory(parent_dir)
            .await
            .context("Failed to fsync config backup directory entry before atomic replace")?;
    }

    if let Err(e) = fs::rename(&temp_path, config_path).await {
        let _ = fs::remove_file(&temp_path).await;
        if had_existing_config && backup_path.exists() {
            fs::copy(&backup_path, config_path)
                .await
                .context("Failed to restore config backup")?;
        }
        anyhow::bail!("Failed to atomically replace config file: {e}");
    }

    #[cfg(unix)]
    {
        use std::{fs::Permissions, os::unix::fs::PermissionsExt};
        if let Err(err) = fs::set_permissions(config_path, Permissions::from_mode(0o600)).await {
            ::omnesagent_log::record!(
                WARN,
                ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                    .with_outcome(::omnesagent_log::EventOutcome::Unknown),
                &format!(
                    "Failed to harden config permissions to 0600 at {}: {}",
                    config_path.display().to_string(),
                    err
                )
            );
        }
    }

    let post_replace_sync_result = match post_replace_sync {
        PostReplaceSync::Real => sync_directory(parent_dir).await,
        #[cfg(any(test, feature = "test-helpers"))]
        PostReplaceSync::FailForTest => Err(anyhow::Error::msg(
            "injected post-replace directory sync failure",
        )),
    };
    if let Err(err) = post_replace_sync_result {
        // The rename is already visible and the replacement file itself was
        // fsynced before it moved. Returning an error here would make callers
        // keep the old live snapshot even though disk contains the new one.
        // Report the durability uncertainty, keep the backup for recovery,
        // and classify the save as committed so save-then-swap callers install
        // the matching snapshot.
        ::omnesagent_log::record!(
            WARN,
            ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                .with_attrs(::serde_json::json!({
                    "error_key": "config.directory_sync_after_replace_failed",
                    "path": config_path.display().to_string(),
                    "backup_path": had_existing_config
                        .then(|| backup_path.display().to_string()),
                    "error": err.to_string(),
                })),
            "Config replacement committed but directory durability sync failed; keeping backup"
        );
    } else if had_existing_config {
        let _ = fs::remove_file(&backup_path).await;
    }

    Ok(())
}

/// Write the in-memory value at `dotted` into the doc, or delete the leaf
/// when the value is absent or equals the schema default. Segments are
/// kebab→snake-translated; alias keys never carry hyphens (alias rule).
fn apply_dirty_path(
    root: &mut toml_edit::Table,
    dotted: &str,
    full_table: &toml::Table,
    default_table: &toml::Table,
) -> Result<()> {
    let raw: Vec<&str> = dotted.split('.').collect();
    if raw.is_empty() {
        return Ok(());
    }

    // Natural-key `Vec<T>` sections (`#[natural_key = "<f>"]`, currently
    // `mcp.servers`) serialize as `[[mcp.servers]]` — an
    // `Item::ArrayOfTables`, not a Table. The generic walker below
    // assumes every intermediate node is a Table (`as_table()` /
    // `as_table_mut()`), so without this branch a dirty path like
    // `mcp.servers.fs.command` would resolve to `None` in `full_table`,
    // get misclassified as `should_delete`, then bail at the
    // `ArrayOfTables` segment in `delete_path_in_doc` too. Net effect:
    // the per-field MCP editor's edits go nowhere on disk. See the
    // regression test `save_dirty_persists_mcp_server_field_via_natural_key`.
    if let Some(section) = find_natural_key_section_for_path(dotted) {
        apply_dirty_natural_key_path(root, &section, dotted, full_table, default_table);
        return Ok(());
    }

    // `HashMap<String, T>` sections whose key is a `#[resource_key]`
    // (drawn from another domain — a model id, tool name, …) may
    // themselves contain dots, e.g. `gpt-4.1` in
    // `cost.rates.providers.models.openai.gpt-4.1.input_per_mtok`. The
    // blind `raw.split('.')` above and the generic walker below both
    // assume one segment == one dot-delimited token, so a dotted key
    // fragments into bogus segments (`gpt-4`, `1`), the lookup below
    // finds nothing, and the write silently turns into a no-op delete.
    // Longest-match the key against the section's live keys instead —
    // same precedent as the natural-key branch above.
    if let Some(section) = find_map_key_section_for_path(dotted) {
        return apply_dirty_map_key_path(root, &section, dotted, full_table, default_table);
    }

    // Resolve each segment against the in-memory table: struct fields
    // serialize as snake_case (so `input-per-mtok` → `input_per_mtok`), but
    // HashMap keys are preserved verbatim and may legitimately carry hyphens
    // (`claude-opus-4-7`, `tts-1-hd`). Blind `s.replace('-', "_")` mangles
    // those keys and lookup returns None, which apply_dirty_path treats as
    // "delete this path" — silently dropping every cost.rates save.
    let segments: Vec<String> = resolve_dirty_segments(full_table, &raw);
    let segs: Vec<&str> = segments.iter().map(String::as_str).collect();
    write_or_delete_leaf(root, full_table, default_table, &segs);
    Ok(())
}

/// Resolved metadata for a `#[natural_key]` Vec section that matches a
/// given dirty path. `section_path` is the dotted section (e.g.
/// `"mcp.servers"`); `natural_key_field` is the snake_case field name
/// on the inner type that holds each element's alias (e.g. `"name"`).
struct NaturalKeySection {
    section_path: &'static str,
    natural_key_field: &'static str,
}

/// Return the longest natural-key `Kind::List` section whose path is a
/// strict prefix of `dotted` (i.e. `dotted` starts with `<path>.`).
/// Strict-prefix wins so a nested natural-key section would always
/// outscore its parent; today there's only `mcp.servers`, so the lookup
/// is effectively a single-entry match, but the longest-match rule
/// keeps the writer correct when a future `#[natural_key]` Vec lands
/// inside another section.
///
/// Returns the `<section_path>` (e.g. `"mcp.servers"`) and the
/// `<natural_key_field>` (e.g. `"name"`) the writer needs to locate
/// the matching `[[<section_path>]]` element on disk.
///
/// Section paths are exact-matched against the leading segments of
/// `dotted`; the function does **not** apply the kebab→snake fallback
/// that the inner segment resolver (`resolve_dirty_segments`) uses.
/// That is deliberate and mirrors the in-memory routing macro's
/// `section_path == expected` check in
/// `crates/omnesagent-macros/src/lib.rs` (the `get_map_keys` /
/// `route_vec_path` arms) — making either side dash-tolerant in
/// isolation would let the dispatcher and the writer disagree on
/// which call participates in this branch, which is exactly the
/// memory-vs-disk divergence this PR exists to eliminate. If a
/// future natural-key section needs an underscore in its path AND
/// must be addressable via a kebab wire path, change **both** sides
/// together (the macro arm and this match).
///
/// Plain `Kind::List` Vec sections without `#[natural_key]` (their
/// `natural_key` is `None`) are intentionally excluded — those use the
/// legacy whole-array `ObjectArray` save path and don't need the
/// array-of-tables walker.
fn find_natural_key_section_for_path(dotted: &str) -> Option<NaturalKeySection> {
    use crate::traits::MapKeyKind;

    let mut best: Option<NaturalKeySection> = None;
    for section in Config::map_key_sections().iter() {
        if section.kind != MapKeyKind::List {
            continue;
        }
        let Some(natural_key_field) = section.natural_key else {
            continue;
        };
        // strip_prefix avoids the allocation `format!("{}.", ...)`
        // would do per section per dirty path, and makes the "must
        // have *something* after the dot" guard a natural fallout of
        // the second `strip_prefix('.')` rather than a length check.
        let Some(after) = dotted
            .strip_prefix(section.path)
            .and_then(|s| s.strip_prefix('.'))
        else {
            continue;
        };
        // Empty alias segment ("mcp.servers." with nothing after) can't
        // be addressed by name; skip to avoid matching every otherwise-
        // unmatched path via the empty prefix. Same guard the in-memory
        // `route_vec_path` applies to empty natural keys.
        if after.is_empty() {
            continue;
        }
        let better = best
            .as_ref()
            .is_none_or(|b| section.path.len() > b.section_path.len());
        if better {
            best = Some(NaturalKeySection {
                section_path: section.path,
                natural_key_field,
            });
        }
    }
    best
}

/// Apply a dirty path of the form
/// `<section_path>.<alias>[.<inner_suffix>...]` against an
/// `[[<section_path>]]` array of tables.
///
/// Three shapes show up in practice:
///
///   1. `<section>.<alias>.<inner>` — per-field edit (the dashboard /
///      TUI editor). Locate the in-memory element whose natural-key
///      field equals `<alias>` and reconcile its `<inner>` sub-path
///      against the doc's matching `[[<section>]]` table.
///
///   2. `<section>.<alias>` with the element still in memory — the
///      whole-element write the rename path emits for the *new* alias.
///      Reseed the matching doc entry from the in-memory serialization
///      so a rename `fs` → `filesystem` updates the entire
///      `[[mcp.servers]]` table (in particular its `name` field).
///
///   3. `<section>.<alias>` with the element gone from memory — the
///      delete path the rename emits for the *old* alias, and what
///      `delete_map_key` plus an explicit `mark_dirty` would produce.
///      Drop the matching doc entry.
///
/// All three are handled by walking the doc through the section's
/// non-array parents, locating the `ArrayOfTables` at the section's
/// last path segment, then either editing or removing the entry whose
/// natural-key field matches `<alias>`. The `<inner>` recursion reuses
/// the existing Table-shaped `set_path_in_doc` / `delete_path_in_doc`
/// helpers against the located entry's table.
///
/// **Order-independence invariant.** `Config::dirty_paths` is a
/// `HashSet<String>`, so the three shapes above can apply in any
/// interleaving within a single `save_dirty` batch — in particular,
/// a rename emits both the old and new aliases dirty without an
/// ordering guarantee, and a create-then-edit sequence may walk the
/// per-field path before the whole-element write. Convergence under
/// every ordering is load-bearing here and works only because every
/// branch derives its write from the current in-memory state and the
/// case-1 delete fires only on mem-absence (not on default
/// equivalence). Future edits — e.g. an early-exit that assumes
/// whole-element writes run before per-field ones, or a "skip if
/// already written" optimization keyed on the doc shape — would
/// break this. Preserve commutativity across the three shapes for
/// any single (section, alias) tuple.
fn apply_dirty_natural_key_path(
    root: &mut toml_edit::Table,
    section: &NaturalKeySection,
    dotted: &str,
    full_table: &toml::Table,
    _default_table: &toml::Table,
) {
    // `dotted` is known to start with `<section_path>.` thanks to
    // `find_natural_key_section_for_path`; split the alias and inner
    // suffix off the tail.
    let rest = match dotted
        .strip_prefix(section.section_path)
        .and_then(|s| s.strip_prefix('.'))
    {
        Some(r) => r,
        None => return,
    };
    let (alias, inner_suffix) = match rest.split_once('.') {
        Some((a, i)) => (a, Some(i)),
        None => (rest, None),
    };
    if alias.is_empty() {
        return;
    }

    // Walk the in-memory serialized array at `section_path`. Each
    // element is a `toml::Value::Table`; the alias lives in the
    // `natural_key_field` column.
    let section_segs: Vec<&str> = section.section_path.split('.').collect();
    let mem_array = lookup_path_in_table(full_table, &section_segs).and_then(|v| v.as_array());
    let mem_entry = mem_array.and_then(|arr| {
        arr.iter().find_map(|item| {
            let table = item.as_table()?;
            let alias_value = table.get(section.natural_key_field)?.as_str()?;
            (alias_value == alias).then_some(table)
        })
    });

    // The doc cursor walks the section's parent Tables (e.g. `mcp`) and
    // arrives at the array node (`servers`). The natural-key Vec is
    // always at the deepest segment of `section_path`; the leading
    // segments are plain struct fields and therefore Tables on disk.
    let Some((array_key, parent_segs)) = section_segs.split_last() else {
        return;
    };

    // Inner-suffix path: per-field edit (case 1). Reconcile the inner
    // sub-path on the matched doc table the same way the generic
    // Table-shaped walker does — mem-vs-default comparison decides
    // delete vs write — and bail out before touching the array spine
    // if the entry doesn't exist in either memory or the doc.
    if let Some(inner) = inner_suffix {
        // Look up the same inner sub-path on the default-shaped element.
        // `Config::default()` always has an empty `mcp.servers` (and no
        // default natural-key Vec entry is non-empty), so we synthesize
        // the default by going through the inner type's own
        // `Default` impl via the existing `default_table` plus an
        // alias-keyed lookup. The Vec being empty in defaults means
        // `default_array_entry` is always `None`, which mirrors the
        // existing HashMap behaviour: `default_val` for an alias's
        // child is None too, because the alias key doesn't exist in
        // `Config::default()`. Keep the comparison shape identical so
        // the delete-on-default rule degrades the same way the HashMap
        // path already does (i.e. it doesn't catch "field is set to
        // its serde default"; that limitation is shared and not a
        // regression from this fix). The `_default_table` parameter
        // is intentionally unused for this reason — kept on the
        // signature so the call site in `save_dirty` stays uniform
        // with `apply_dirty_path`, which does consume it.

        let inner_raw: Vec<&str> = inner.split('.').collect();
        // Same dash-aware segment resolution the top-level Table-shaped
        // walker uses (`apply_dirty_path` → `resolve_dirty_segments`), but
        // rooted at the natural-key entry's own serialized table so kebab
        // inner segments like `mcp.servers.fs.tool-timeout-secs` resolve
        // to the snake `tool_timeout_secs` field on disk. Sharing the
        // helper instead of forking is load-bearing: this PR exists
        // because two parallel walks (in-memory vs on-disk) drifted apart;
        // a forked copy here would recreate exactly that hazard, so any
        // future fix to dash handling lands in both code paths at once.
        let inner_segments: Vec<String> = match mem_entry {
            Some(table) => resolve_dirty_segments(table, &inner_raw),
            None => inner_raw.iter().map(|s| (*s).to_string()).collect(),
        };
        let inner_segs: Vec<&str> = inner_segments.iter().map(String::as_str).collect();

        let mem_inner_val = mem_entry.and_then(|t| lookup_path_in_table(t, &inner_segs));

        // mem missing → delete. mem present → write.
        match mem_inner_val {
            Some(value) => {
                let entry_table = ensure_array_of_tables_entry(
                    root,
                    parent_segs,
                    array_key,
                    section.natural_key_field,
                    alias,
                );
                let Some(entry_table) = entry_table else {
                    return;
                };
                let mut pruned = value.clone();
                prune_empty_leaves(&mut pruned);
                set_path_in_doc(entry_table, &inner_segs, &pruned);
            }
            None => {
                // Only walk the doc if we'd actually find something to
                // delete; otherwise an "ambient" delete pass on a fresh
                // alias would scribble an empty `[[mcp.servers]]` entry
                // via `ensure_array_of_tables_entry`'s create path.
                let Some(entry_table) = find_array_of_tables_entry_mut(
                    root,
                    parent_segs,
                    array_key,
                    section.natural_key_field,
                    alias,
                ) else {
                    return;
                };
                delete_path_in_doc(entry_table, &inner_segs);
            }
        }
        return;
    }

    // Bare-alias path (cases 2 and 3). No inner suffix → reconcile the
    // whole element.
    match mem_entry {
        Some(entry) => {
            // Case 2: rewrite the entire array entry from the in-memory
            // serialization. Prunes empties so default fields don't
            // sprawl, matching the per-field path's `prune_empty_leaves`
            // pass.
            let mut value = toml::Value::Table(entry.clone());
            prune_empty_leaves(&mut value);
            let entry_table = ensure_array_of_tables_entry(
                root,
                parent_segs,
                array_key,
                section.natural_key_field,
                alias,
            );
            let Some(entry_table) = entry_table else {
                return;
            };
            // Replace the table's contents wholesale so removed inner
            // fields actually disappear from disk.
            entry_table.clear();
            if let toml::Value::Table(t) = value {
                for (k, v) in t.into_iter() {
                    entry_table.insert(&k, crate::migration::toml_value_to_edit_item(&v));
                }
            }
        }
        None => {
            // Case 3: alias no longer in memory → drop the matching
            // doc entry. If no doc entry matches, no-op.
            delete_array_of_tables_entry(
                root,
                parent_segs,
                array_key,
                section.natural_key_field,
                alias,
            );
        }
    }
}

/// Return the longest `MapKeyKind::Map` section whose path is a strict
/// prefix of `dotted` (i.e. `dotted` starts with `<path>.`). Longest-match
/// mirrors `find_natural_key_section_for_path`: a nested `HashMap` field's
/// own section always outscores a shorter ancestor section.
fn find_map_key_section_for_path(dotted: &str) -> Option<crate::traits::MapKeySection> {
    use crate::traits::MapKeyKind;

    let mut best: Option<crate::traits::MapKeySection> = None;
    for section in Config::map_key_sections() {
        if section.kind != MapKeyKind::Map {
            continue;
        }
        let Some(after) = dotted
            .strip_prefix(section.path)
            .and_then(|s| s.strip_prefix('.'))
        else {
            continue;
        };
        if after.is_empty() {
            continue;
        }
        let better = best
            .as_ref()
            .is_none_or(|b| section.path.len() > b.path.len());
        if better {
            best = Some(section);
        }
    }
    best
}

/// Apply a dirty path of the form `<section_path>.<key>[.<inner_suffix>...]`
/// against a `HashMap<String, T>` section. `<key>` is the literal map key
/// and, for `#[resource_key]` sections (a model id, tool name, … rather
/// than an operator-chosen alias), may itself contain dots — e.g. `gpt-4.1`
/// in `cost.rates.providers.models.openai.gpt-4.1.input_per_mtok`.
///
/// Longest-match `<key>` against the union of keys live in `full_table`
/// (the in-memory state) and in the doc (`root`), so a key that only
/// survives on one side — the delete half of a rename, or a fresh
/// alias not yet flushed — still resolves. This mirrors
/// `apply_dirty_natural_key_path`'s two-sided lookup for `mcp.servers`.
///
/// A whole-entry write (no inner suffix — `<section_path>.<key>` addresses
/// the map value itself, e.g. the create-map-key path) is the case where
/// `<key>` consumes the entire remainder; that always wins over any
/// shorter prefix-plus-dot match into the same key space, so it's checked
/// first.
///
/// Once `<key>` is resolved, the rest of the path reuses the exact
/// mem-vs-default comparison and doc write/delete the generic Table-shaped
/// walker in `apply_dirty_path` uses — the dotted key is just folded into
/// one opaque segment instead of being split on '.' with everything else.
fn apply_dirty_map_key_path(
    root: &mut toml_edit::Table,
    section: &crate::traits::MapKeySection,
    dotted: &str,
    full_table: &toml::Table,
    default_table: &toml::Table,
) -> Result<()> {
    let section_segs: Vec<&str> = section.path.split('.').collect();
    let remainder = &dotted[section.path.len() + 1..];

    let mem_table = lookup_path_in_table(full_table, &section_segs).and_then(|v| v.as_table());
    let doc_table = lookup_table_in_doc(root, &section_segs);

    let keys: Vec<&str> = mem_table
        .into_iter()
        .flat_map(|t| t.keys().map(String::as_str))
        .chain(doc_table.into_iter().flat_map(|t| t.iter().map(|(k, _)| k)))
        .collect();

    // Exact match first: a whole-entry path for key `gpt-4.1` must not be
    // swallowed by `route_hashmap_path` matching the shorter key `gpt-4`
    // plus a bogus inner suffix `1...`. The cost is the inverse shadow: a
    // pathological key literally named `<other-key>.<field>` wins over a
    // field write addressed to `<other-key>` — acceptable because the
    // exact key is the more specific interpretation of the same bytes.
    let key_match: Option<(&str, Option<String>)> = if keys.contains(&remainder) {
        Some((remainder, None))
    } else {
        crate::helpers::route_hashmap_path(dotted, "", section.path, "", keys.iter().copied())
            .map(|(key, inner)| (key, Some(inner)))
    };

    let Some((key, inner)) = key_match else {
        anyhow::bail!(
            "save_dirty: dirty path `{dotted}` addresses map-key section `{}` but its key \
             resolves in neither the in-memory config nor the on-disk file; refusing to \
             silently drop the write",
            section.path
        );
    };

    let mut segments: Vec<String> = section_segs.iter().map(|s| (*s).to_string()).collect();
    segments.push(key.to_string());

    if let Some(inner) = inner {
        // Same dash-aware segment resolution `apply_dirty_natural_key_path`
        // uses for its inner suffix, rooted at the matched key's own
        // serialized table so kebab inner segments (`tool-timeout-secs`)
        // resolve to the snake struct field on disk. Consult the matching
        // on-disk table too: when a dotted dynamic-map key is being deleted,
        // it is absent from memory by definition but still needs to resolve
        // as one opaque segment in the document.
        let key_table = mem_table
            .and_then(|t| t.get(key))
            .and_then(|v| v.as_table());
        let doc_key_table = doc_table
            .and_then(|t| t.get(key))
            .and_then(|item| item.as_table_like());
        let inner_raw: Vec<&str> = inner.split('.').collect();
        let inner_segments =
            resolve_dirty_segments_from_sources(key_table, doc_key_table, &inner_raw);
        segments.extend(inner_segments);
    }

    let segs: Vec<&str> = segments.iter().map(String::as_str).collect();
    write_or_delete_leaf(root, full_table, default_table, &segs);
    Ok(())
}

/// Shared tail of the generic and map-key dirty-path walkers: reconcile
/// the resolved `segs` against the doc — delete the leaf when the
/// in-memory value is absent or equals the schema default, otherwise
/// write the pruned in-memory value. One copy so a future fix to the
/// delete-vs-write rule cannot land in one walker only.
fn write_or_delete_leaf(
    root: &mut toml_edit::Table,
    full_table: &toml::Table,
    default_table: &toml::Table,
    segs: &[&str],
) {
    let mem_val = lookup_path_in_table(full_table, segs);
    let default_val = lookup_path_in_table(default_table, segs);

    let should_delete = match (mem_val, default_val) {
        (None, _) => true,
        (Some(m), Some(d)) if m == d => true,
        _ => false,
    };

    if should_delete {
        delete_path_in_doc(root, segs);
    } else if let Some(value) = mem_val {
        let mut pruned = value.clone();
        prune_empty_leaves(&mut pruned);
        set_path_in_doc(root, segs, &pruned);
    }
}

/// Emit a single `WARN` event when a natural-key writer (ensure /
/// find / delete) bails because the on-disk node has the wrong shape
/// — for example an operator hand-edited `mcp.servers = "foo"`, or
/// wrote `servers = [{ name = "fs", ... }]` (an inline array of
/// tables, which `toml_edit` parses as `Item::Value(Value::Array)`
/// rather than the `Item::ArrayOfTables` this writer produces).
///
/// Without this, `config/set` returns success, the in-memory mutation
/// and the dashboard/TUI both reflect the new value, and the on-disk
/// file silently stays stale — the exact symptom the underlying
/// natural-key persistence bug ( fix) exists to eliminate. The
/// `WARN` shape mirrors the 0600-permissions fallback below: same
/// `Action::Note` / `EventOutcome::Unknown` / module path, so log
/// scrapers that already understand the schema warnings will surface
/// these the same way.
///
/// `op` is `"ensure"`, `"find"`, or `"delete"` — which writer bailed.
/// `kind` is `"parent_segment"` or `"array_key"` — which node was
/// wrong-shaped. `which` is the specific seg/key name involved.
fn warn_natural_key_doc_kind_mismatch(
    parent_segs: &[&str],
    array_key: &str,
    alias: &str,
    kind: &str,
    which: &str,
    op: &str,
) {
    let section_path = if parent_segs.is_empty() {
        array_key.to_string()
    } else {
        format!("{}.{array_key}", parent_segs.join("."))
    };
    ::omnesagent_log::record!(
        WARN,
        ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
            .with_outcome(::omnesagent_log::EventOutcome::Unknown)
            .with_attrs(::serde_json::json!({
                "section_path": section_path,
                "alias": alias,
                "node_kind": kind,
                "node_name": which,
                "operation": op,
            })),
        "natural-key writer: refusing to clobber hand-edited config.toml node; \
         on-disk file may diverge from in-memory state"
    );
}

/// Locate or create the `[[<section_path>]]` entry whose natural-key
/// field equals `alias`, returning a mutable reference to its inner
/// `toml_edit::Table`.
///
/// Three shapes are reconciled here:
///
///   * Parent Tables along `parent_segs` are auto-created if missing
///     (mirrors `set_path_in_doc`).
///   * The `array_key` slot may not yet exist; insert an
///     `ArrayOfTables` if so.
///   * The array may not yet contain an entry whose natural-key field
///     matches `alias`; push a new table with `<natural_key_field> =
///     "<alias>"` seeded.
///
/// Returns `None` only when an existing doc node at `parent_segs` or
/// `array_key` has the wrong kind (e.g. an operator hand-edited
/// `mcp.servers = "foo"`, or wrote an inline `servers = [{ ... }]`
/// array-of-tables that parses as `Item::Value(Value::Array)` rather
/// than the `Item::ArrayOfTables` shape this writer expects); in that
/// case we refuse to clobber the hand-edit and let the save bail
/// rather than data-loss the user's file. A `WARN`-level event is
/// emitted on every wrong-kind bail so the divergence between memory
/// and disk is observable in logs — silent bails were what let the
/// underlying bug ship.
fn ensure_array_of_tables_entry<'a>(
    root: &'a mut toml_edit::Table,
    parent_segs: &[&str],
    array_key: &str,
    natural_key_field: &str,
    alias: &str,
) -> Option<&'a mut toml_edit::Table> {
    let mut cursor: &mut toml_edit::Table = root;
    for seg in parent_segs {
        if !cursor.contains_key(seg) {
            cursor.insert(seg, toml_edit::Item::Table(toml_edit::Table::new()));
        }
        let next = cursor.get_mut(seg).and_then(|i| i.as_table_mut());
        if next.is_none() {
            warn_natural_key_doc_kind_mismatch(
                parent_segs,
                array_key,
                alias,
                "parent_segment",
                seg,
                "ensure",
            );
            return None;
        }
        cursor = next?;
    }
    if !cursor.contains_key(array_key) {
        cursor.insert(
            array_key,
            toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new()),
        );
    }
    let aot = cursor
        .get_mut(array_key)
        .and_then(|i| i.as_array_of_tables_mut());
    let Some(aot) = aot else {
        warn_natural_key_doc_kind_mismatch(
            parent_segs,
            array_key,
            alias,
            "array_key",
            array_key,
            "ensure",
        );
        return None;
    };

    let pos = aot
        .iter()
        .position(|t| t.get(natural_key_field).and_then(|i| i.as_str()) == Some(alias));
    let idx = match pos {
        Some(i) => i,
        None => {
            let mut new_table = toml_edit::Table::new();
            new_table.insert(natural_key_field, toml_edit::value(alias.to_string()));
            aot.push(new_table);
            aot.len() - 1
        }
    };
    aot.get_mut(idx)
}

/// Read-only counterpart to [`ensure_array_of_tables_entry`] used by
/// the delete path: never creates a missing array or entry, just
/// locates one if present. Returning `None` on a miss lets the caller
/// skip the doc-mutation step entirely. Distinguishes "node doesn't
/// exist" (silent no-op — the legitimate fresh-alias case) from "node
/// exists but has the wrong kind" (a `WARN`-logged bail — the
/// hand-edited inline-array hazard).
fn find_array_of_tables_entry_mut<'a>(
    root: &'a mut toml_edit::Table,
    parent_segs: &[&str],
    array_key: &str,
    natural_key_field: &str,
    alias: &str,
) -> Option<&'a mut toml_edit::Table> {
    let mut cursor: &mut toml_edit::Table = root;
    for seg in parent_segs {
        let item = cursor.get_mut(seg)?;
        let Some(next) = item.as_table_mut() else {
            warn_natural_key_doc_kind_mismatch(
                parent_segs,
                array_key,
                alias,
                "parent_segment",
                seg,
                "find",
            );
            return None;
        };
        cursor = next;
    }
    let item = cursor.get_mut(array_key)?;
    let Some(aot) = item.as_array_of_tables_mut() else {
        warn_natural_key_doc_kind_mismatch(
            parent_segs,
            array_key,
            alias,
            "array_key",
            array_key,
            "find",
        );
        return None;
    };
    let pos = aot
        .iter()
        .position(|t| t.get(natural_key_field).and_then(|i| i.as_str()) == Some(alias))?;
    aot.get_mut(pos)
}

/// Remove the `[[<section_path>]]` entry whose natural-key field
/// equals `alias`. No-op if the array or entry isn't present. When the
/// removal empties the `ArrayOfTables`, drop the array slot too so the
/// on-disk file doesn't carry a vestigial `[[mcp.servers]]`-shaped
/// blank section header.
fn delete_array_of_tables_entry(
    root: &mut toml_edit::Table,
    parent_segs: &[&str],
    array_key: &str,
    natural_key_field: &str,
    alias: &str,
) {
    let mut cursor: &mut toml_edit::Table = root;
    for seg in parent_segs {
        let Some(item) = cursor.get_mut(seg) else {
            return;
        };
        match item.as_table_mut() {
            Some(next) => cursor = next,
            None => {
                warn_natural_key_doc_kind_mismatch(
                    parent_segs,
                    array_key,
                    alias,
                    "parent_segment",
                    seg,
                    "delete",
                );
                return;
            }
        }
    }
    let Some(item) = cursor.get_mut(array_key) else {
        return;
    };
    let aot = match item.as_array_of_tables_mut() {
        Some(a) => a,
        None => {
            warn_natural_key_doc_kind_mismatch(
                parent_segs,
                array_key,
                alias,
                "array_key",
                array_key,
                "delete",
            );
            return;
        }
    };
    let pos = aot
        .iter()
        .position(|t| t.get(natural_key_field).and_then(|i| i.as_str()) == Some(alias));
    if let Some(idx) = pos {
        aot.remove(idx);
    }
    if aot.is_empty() {
        cursor.remove(array_key);
    }
}

/// Drop empty arrays / tables / strings from a value before writing it
/// to the doc. HashMap entries serialize every default field (no
/// `skip_serializing_if` on individual `Vec<String>` fields), so without
/// this pass an `mcp_bundles.<alias>` write produces `servers = []`,
/// `exclude = []`, etc. The pruned form round-trips identically because
/// each dropped field's serde default IS the dropped value.
fn prune_empty_leaves(value: &mut toml::Value) {
    match value {
        toml::Value::Table(t) => {
            let keys: Vec<String> = t.keys().cloned().collect();
            for key in keys {
                if let Some(inner) = t.get_mut(&key) {
                    prune_empty_leaves(inner);
                }
                let drop = match t.get(&key) {
                    Some(toml::Value::Array(arr)) => arr.is_empty(),
                    Some(toml::Value::Table(inner)) => inner.is_empty(),
                    Some(toml::Value::String(s)) => s.is_empty(),
                    _ => false,
                };
                if drop {
                    t.remove(&key);
                }
            }
        }
        toml::Value::Array(arr) => {
            for item in arr.iter_mut() {
                prune_empty_leaves(item);
            }
        }
        _ => {}
    }
}

fn resolve_dirty_segments(root: &toml::Table, raw: &[&str]) -> Vec<String> {
    resolve_dirty_segments_from_sources(Some(root), None, raw)
}

/// Resolve a dotted dirty-path suffix against the canonical serialized
/// in-memory table and, when supplied, its on-disk counterpart.
///
/// Struct fields consume one segment (with the existing kebab-to-snake
/// fallback). Map keys are different: the serialized table owns their exact
/// spelling, and a key may itself contain dots. Longest-match the remaining
/// suffix against keys present in either source so `X-Foo.bar` remains one
/// segment for both writes (present in memory) and deletes (present only on
/// disk). No parallel key registry is introduced; both views are derived from
/// the two states `save_dirty` is already reconciling.
fn resolve_dirty_segments_from_sources(
    mut memory_table: Option<&toml::Table>,
    mut document_table: Option<&dyn toml_edit::TableLike>,
    raw: &[&str],
) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(raw.len());
    let mut index = 0;
    while index < raw.len() {
        // Prefer the longest exact key from the remaining suffix. This is
        // load-bearing for opaque HashMap keys containing dots; an ordinary
        // struct table has no such key, so it naturally falls through to the
        // one-segment field lookup below.
        let exact = (index + 1..=raw.len()).rev().find_map(|end| {
            let candidate = raw[index..end].join(".");
            let exists_in_memory = memory_table.is_some_and(|t| t.contains_key(&candidate));
            let exists_on_disk = document_table.is_some_and(|t| t.contains_key(&candidate));
            (exists_in_memory || exists_on_disk).then_some((candidate, end - index))
        });

        let (resolved, consumed) = exact.unwrap_or_else(|| {
            let segment = raw[index];
            let snake = segment.replace('-', "_");
            let snake_exists = memory_table.is_some_and(|t| t.contains_key(&snake))
                || document_table.is_some_and(|t| t.contains_key(&snake));
            (
                if snake_exists {
                    snake
                } else {
                    segment.to_string()
                },
                1,
            )
        });

        memory_table = memory_table
            .and_then(|t| t.get(&resolved))
            .and_then(|v| v.as_table());
        document_table = document_table
            .and_then(|t| t.get(&resolved))
            .and_then(|item| item.as_table_like());
        out.push(resolved);
        index += consumed;
    }
    out
}

fn lookup_path_in_table<'a>(root: &'a toml::Table, segs: &[&str]) -> Option<&'a toml::Value> {
    let mut current: Option<&toml::Value> = None;
    for (i, seg) in segs.iter().enumerate() {
        let table = if i == 0 { root } else { current?.as_table()? };
        current = table.get(*seg);
    }
    current
}

/// Read-only walk to the table-like node at `segs`, or `None` if any
/// segment is missing or not table-shaped on disk. Used to read a map-key
/// section's live on-disk keys without mutating the doc. `TableLike`
/// rather than `Table` because OmnesAgent loads (though never writes)
/// hand-edited inline tables — `openai = { "gpt-4.1" = { ... } }` parses
/// as `Item::Value(Value::InlineTable)`, which `as_table()` rejects; a
/// key living only in such a section must still resolve here or the
/// unresolvable-key bail in `apply_dirty_map_key_path` aborts the whole
/// `save_dirty` batch.
fn lookup_table_in_doc<'a>(
    root: &'a toml_edit::Table,
    segs: &[&str],
) -> Option<&'a dyn toml_edit::TableLike> {
    let mut cursor: &dyn toml_edit::TableLike = root;
    for seg in segs {
        cursor = cursor.get(seg)?.as_table_like()?;
    }
    Some(cursor)
}

/// `TableLike` rather than `Table` for the traversal cursor — same reason
/// as `lookup_table_in_doc` above: a map-key section may live inside a
/// hand-edited inline table (`openai = { "gpt-4.1" = { ... } }`), which
/// parses as `Item::Value(Value::InlineTable)`. `as_table_mut()` returns
/// `None` for that shape, so a `Table`-only cursor would silently return
/// without removing anything — `save_dirty` reports success while the key
/// stays on disk. `as_table_like_mut()` descends into both `Table` and
/// `InlineTable`, and `TableLike::remove` deletes from either.
fn delete_path_in_doc(root: &mut toml_edit::Table, segs: &[&str]) {
    let Some((last, parents)) = segs.split_last() else {
        return;
    };
    let mut cursor: &mut dyn toml_edit::TableLike = root;
    for seg in parents {
        cursor = match cursor.get_mut(seg).and_then(|i| i.as_table_like_mut()) {
            Some(t) => t,
            None => return,
        };
    }
    cursor.remove(last);
}

/// Same `TableLike` traversal as `delete_path_in_doc`, for the same
/// reason: a write into a key that already lives inside a hand-edited
/// inline table must land in that inline table rather than silently
/// no-op-ing because the cursor only understood standard `Table` nodes.
fn set_path_in_doc(root: &mut toml_edit::Table, segs: &[&str], value: &toml::Value) {
    let Some((last, parents)) = segs.split_last() else {
        return;
    };
    let mut cursor: &mut dyn toml_edit::TableLike = root;
    for seg in parents {
        if !cursor.contains_key(seg) {
            cursor.insert(seg, toml_edit::Item::Table(toml_edit::Table::new()));
        }
        cursor = match cursor.get_mut(seg).and_then(|i| i.as_table_like_mut()) {
            Some(t) => t,
            None => return,
        };
    }
    let new_item = crate::migration::toml_value_to_edit_item(value);
    cursor.insert(last, new_item);
}

#[allow(clippy::unused_async)] // async needed on unix for tokio File I/O; no-op on other platforms
pub(super) async fn sync_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let dir = File::open(path).await.with_context(|| {
            format!(
                "Failed to open directory for fsync: {}",
                path.display().to_string()
            )
        })?;
        dir.sync_all().await.with_context(|| {
            format!(
                "Failed to fsync directory metadata: {}",
                path.display().to_string()
            )
        })?;
        Ok(())
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x02000000;
        let dir = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)
            .with_context(|| {
                format!(
                    "Failed to open directory for fsync: {}",
                    path.display().to_string()
                )
            })?;
        // FlushFileBuffers on directory handles returns ERROR_ACCESS_DENIED on
        // Windows (OS Error 5). This is expected — NTFS does not support
        // flushing directory metadata the same way Unix does. The individual
        // files have already been synced, so it is safe to ignore this error.
        if let Err(e) = dir.sync_all() {
            if e.raw_os_error() == Some(5) {
                ::omnesagent_log::record!(
                    TRACE,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note),
                    &format!(
                        "Ignoring expected ACCESS_DENIED when fsyncing directory on Windows: {}",
                        path.display().to_string()
                    )
                );
            } else {
                return Err(e).with_context(|| {
                    format!(
                        "Failed to fsync directory metadata: {}",
                        path.display().to_string()
                    )
                });
            }
        }
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        Ok(())
    }
}
