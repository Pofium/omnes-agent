// Historical schema typed lenses for migration. Each module is frozen after
// its corresponding version ships; only their `migrate(self) -> ...` methods
// are referenced at runtime by `crate::migration`.
pub mod v1;
pub mod v2;

// Per-family model provider configs (typed family slots + endpoint enums)
// live in their own module; re-exported here so every
// `crate::schema::<Family>ModelProviderConfig` path is unchanged.
mod model_providers;
pub use model_providers::*;

// TTS + transcription typed-family provider slots (endpoint enums + per-family
// configs) live in their own module; re-exported here so every
// `crate::schema::<Family>{Tts,Transcription}ProviderConfig` path is unchanged.
mod media_providers;
pub use media_providers::*;

// Cost tracking & budget enforcement config (the `[cost]` section) lives in
// its own module; re-exported here so `crate::schema::{CostConfig, ...}`
// paths are unchanged.
mod cost;
pub use cost::*;

// Skills config (the `[skills]` section: registries, prompt injection,
// creation/improvement/install suggestions) lives in its own module;
// re-exported here so `crate::schema::{SkillsConfig, ...}` paths are unchanged.
mod skills;
pub use skills::*;

// Observability config (the `[observability]` section) lives in its own
// module; re-exported here so `crate::schema::{ObservabilityConfig, ...}`
// paths are unchanged.
mod observability;
pub use observability::*;

// Memory subsystem config (the `[memory]` section) lives in its own module;
// re-exported here so `crate::schema::{MemoryConfig, ...}` paths are unchanged.
mod memory;
pub use memory::*;

// Lifecycle hooks config (the `[hooks]` section) lives in its own module;
// re-exported here so `crate::schema::{HooksConfig, ...}` paths are unchanged.
mod hooks;
pub use hooks::*;

// Backup tool config (the `[backup]` section) lives in its own module;
// re-exported here so `crate::schema::BackupConfig` is unchanged.
mod backup;
pub use backup::*;

// Data retention config (the `[data_retention]` section) lives in its own
// module; re-exported here so `crate::schema::DataRetentionConfig` is unchanged.
mod data_retention;
pub use data_retention::*;

// Scheduler loop config (the `[scheduler]` section) lives in its own module;
// re-exported here so `crate::schema::SchedulerConfig` is unchanged.
mod scheduler;
pub use scheduler::*;

// Tunnel config (the `[tunnel]` section and per-provider sub-configs) lives in
// its own module; re-exported here so `crate::schema::<TunnelType>` is unchanged.
mod tunnel;
pub use tunnel::*;

// Cron subsystem config (the `[cron]` section) lives in its own module;
// re-exported here so `crate::schema::<CronType>` is unchanged.
mod cron;
pub use cron::*;

// Runtime adapter config (the `[runtime]` section) lives in its own module;
// re-exported here so `crate::schema::<RuntimeType>` is unchanged.
mod runtime;
pub use runtime::*;

// Reliability & supervision config (the `[reliability]` section) lives in its
// own module; re-exported here so `crate::schema::ReliabilityConfig` is unchanged.
mod reliability;
pub use reliability::*;

// Model routing config (the `[[model_routes]]` table) lives in its own module;
// re-exported here so `crate::schema::ModelRouteConfig` is unchanged.
mod model_routing;
pub use model_routing::*;

// On-disk model cache schema lives in its own module; re-exported here so
// `crate::schema::ModelCacheState` is unchanged.
mod model_cache;
pub use model_cache::*;

// Embedding routing config (the `[[embedding_routes]]` table) lives in its own
// module; re-exported here so `crate::schema::EmbeddingRouteConfig` is unchanged.
mod embedding_routing;
pub use embedding_routing::*;

// Query classification config (the `[query_classification]` section) lives in
// its own module; re-exported here so `crate::schema::<QueryClassificationType>`
// is unchanged.
mod query_classification;
pub use query_classification::*;

// Heartbeat config (the `[heartbeat]` section) lives in its own module;
// re-exported here so `crate::schema::HeartbeatConfig` is unchanged.
mod heartbeat;
pub use heartbeat::*;

// ACP server config (the `[acp]` section) lives in its own module;
// re-exported here so `crate::schema::AcpConfig` is unchanged.
mod acp;
pub use acp::*;

// Channels config (the `[channels]` section: per-instance tables and the
// per-channel typed configs) lives in its own module; re-exported here so
// every `crate::schema::<ChannelsType>` path is unchanged.
mod channels;
pub use channels::*;

// Security config (the `[security]` section: audit, OTP, e-stop, leak
// detection, WebAuthn, Nevis IAM/SSO, sandbox) lives in its own module;
// re-exported here so every `crate::schema::<SecurityType>` path is unchanged.
mod security;
pub use security::*;

// `impl Config` / `impl Default for Config` and the config-file persistence
// machinery (dirty-path application, TOML editing, OnePassword snapshots)
// live in their own module; re-exported here so every crate-visible
// `crate::schema::<pub-fn>` path (e.g. `ftl_locale_dir`) is unchanged.
mod config_impl;
pub use config_impl::*;

// Conversational AI config (the `[conversational_ai]` section) lives in its
// own module; re-exported here so `crate::schema::ConversationalAiConfig` is
// unchanged.
mod conversational_ai;
pub use conversational_ai::*;

// Security ops config (the `[security_ops]` section) lives in its own module;
// re-exported here so `crate::schema::SecurityOpsConfig` is unchanged.
mod security_ops;
pub use security_ops::*;

// Gateway server security & remote transport config (`[gateway]`, `[wss]`,
// `[relay]`, `[enroll]` and their nested sub-configs) lives in its own
// module; re-exported here so every `crate::schema::<Type>` path is
// unchanged.
mod gateway;
pub use gateway::*;

// Composio managed OAuth tools config (the `[composio]` section) lives in
// its own module; re-exported here so `crate::schema::ComposioConfig` is
// unchanged.
mod composio;
pub use composio::*;

// Microsoft 365 (Graph API) integration config (the `[microsoft365]`
// section) lives in its own module; re-exported here so
// `crate::schema::Microsoft365Config` is unchanged.
mod microsoft365;
pub use microsoft365::*;

// Secrets encryption config (the `[secrets]` section) lives in its own
// module; re-exported here so `crate::schema::SecretsConfig` is unchanged.
mod secrets;
pub use secrets::*;

// Browser tool & computer-use sidecar config (`[browser]` and
// `[browser.computer_use]`) lives in its own module; re-exported here so
// every `crate::schema::<BrowserType>` path is unchanged.
mod browser;
pub use browser::*;

// HTTP request tool config (the `[http_request]` section) lives in its own
// module; re-exported here so `crate::schema::HttpRequestConfig` is
// unchanged.
mod http_request;
pub use http_request::*;

// Web fetch tool config (`[web_fetch]` + nested `[web_fetch.firecrawl]`)
// lives in its own module; re-exported here so
// `crate::schema::{WebFetchConfig, FirecrawlConfig, FirecrawlMode}` is
// unchanged.
mod web_fetch;
pub use web_fetch::*;

// Link enricher config (the `[link_enricher]` section) lives in its own
// module; re-exported here so `crate::schema::LinkEnricherConfig` is
// unchanged.
mod link_enricher;
pub use link_enricher::*;

// Text browser tool config (the `[text_browser]` section) lives in its own
// module; re-exported here so `crate::schema::TextBrowserConfig` is
// unchanged.
mod text_browser;
pub use text_browser::*;

// Shell tool config (the `[shell_tool]` section) lives in its own module;
// re-exported here so `crate::schema::ShellToolConfig` is unchanged.
mod shell_tool;
pub use shell_tool::*;

// Escalation routing config (the `[escalation]` section) lives in its own
// module; re-exported here so `crate::schema::EscalationConfig` is unchanged.
mod escalation;
pub use escalation::*;

// Web search tool config (the `[web_search]` section) lives in its own module;
// re-exported here so `crate::schema::WebSearchConfig` is unchanged.
mod web_search;
pub use web_search::*;

// Project intelligence config (the `[project_intel]` section) lives in its own
// module; re-exported here so `crate::schema::ProjectIntelConfig` is unchanged.
mod project_intel;
pub use project_intel::*;

// Google Workspace tool config (the `[google_workspace]` section) lives in its
// own module; re-exported here so `crate::schema::<GoogleWorkspaceType>` paths
// are unchanged.
mod google_workspace;
pub use google_workspace::*;

// Knowledge graph config (the `[knowledge]` section) lives in its own module;
// re-exported here so `crate::schema::KnowledgeConfig` is unchanged.
mod knowledge;
pub use knowledge::*;

// Claude Code CLI tool config (the `[claude_code]` section) lives in its own
// module; re-exported here so `crate::schema::ClaudeCodeConfig` is unchanged.
mod claude_code;
pub use claude_code::*;

// Claude Code task runner config (the `[claude_code_runner]` section) lives in
// its own module; re-exported here so `crate::schema::ClaudeCodeRunnerConfig`
// is unchanged.
mod claude_code_runner;
pub use claude_code_runner::*;

// Codex CLI tool config (the `[codex_cli]` section) and its argument-safety
// warning machinery live in their own module; re-exported here so
// `crate::schema::CodexCliConfig` is unchanged.
mod codex_cli;
pub use codex_cli::*;

// Gemini CLI tool config (the `[gemini_cli]` section) lives in its own module;
// re-exported here so `crate::schema::GeminiCliConfig` is unchanged.
mod gemini_cli;
pub use gemini_cli::*;

// OpenCode CLI tool config (the `[opencode_cli]` section) lives in its own
// module; re-exported here so `crate::schema::OpenCodeCliConfig` is unchanged.
mod opencode_cli;
pub use opencode_cli::*;

// Standalone image generation tool config (the `[image_gen]` section) lives in
// its own module; re-exported here so `crate::schema::ImageGenConfig` is
// unchanged.
mod image_gen;
pub use image_gen::*;

// Standalone file upload tool config (the `[file_upload]` section) lives in its
// own module; re-exported here so `crate::schema::FileUploadConfig` is
// unchanged.
mod file_upload;
pub use file_upload::*;

// Standalone multi-file bundle upload tool config (the `[file_upload_bundle]`
// section) lives in its own module; re-exported here so
// `crate::schema::FileUploadBundleConfig` is unchanged.
mod file_upload_bundle;
pub use file_upload_bundle::*;

// Standalone file download tool config (the `[file_download]` section) lives in
// its own module; re-exported here so `crate::schema::FileDownloadConfig` is
// unchanged.
mod file_download;
pub use file_download::*;

// Proxy config (the `[proxy]` section: `ProxyScope`/`ProxyConfig` + scope
// selection and `reqwest` application) lives in its own module; re-exported
// here so `crate::schema::{ProxyScope, ProxyConfig}` are unchanged.
mod proxy;
pub use proxy::*;

// Automatic media understanding pipeline (the `[media_pipeline]` section) lives
// in its own module; re-exported here so
// `crate::schema::MediaPipelineConfig` is unchanged.
mod media_pipeline;
pub use media_pipeline::*;

// Identity format config (the `[identity]` section) lives in its own module;
// re-exported here so `crate::schema::IdentityConfig` is unchanged.
mod identity;
pub use identity::*;

// Peripheral board integration (the `[peripherals]` section) lives in its own
// module; re-exported here so `crate::schema::{PeripheralsConfig,
// PeripheralBoardConfig}` are unchanged.
mod peripherals;
pub use peripherals::*;

// Hardware control config (the `[hardware]` section) lives in its own module;
// re-exported here so `crate::schema::{HardwareConfig, HardwareTransport}` are
// unchanged.
mod hardware;
pub use hardware::*;

// Voice transcription config (the `[transcription]` section) lives in its own
// module; re-exported here so `crate::schema::TranscriptionConfig` is unchanged.
mod transcription;
pub use transcription::*;

// Speech-to-text provider configs (the `[transcription.<provider>]` sections)
// live in their own module; re-exported here so every
// `crate::schema::<Provider>SttConfig` / `crate::schema::LocalWhisperConfig`
// path is unchanged.
mod stt_providers;
pub use stt_providers::*;

// Voice input config (the `[voice]` section) lives in its own module;
// re-exported here so `crate::schema::{VoiceConfig, HandyConfig}` are unchanged.
mod voice;
pub use voice::*;

// Dynamic node discovery config (the `[nodes]` section) lives in its own module;
// re-exported here so `crate::schema::{NodesConfig, MdnsConfig}` are unchanged.
mod nodes;
pub use nodes::*;

// Text-to-Speech config (the `[tts]` section) lives in its own module;
// re-exported here so `crate::schema::{TtsConfig, TtsProviderConfig}` are
// unchanged.
mod tts;
pub use tts::*;

// MCP tool filtering groups (`[...tool_filter_groups]`) live in their own
// module; re-exported here so `crate::schema::{ToolFilterGroup,
// ToolFilterGroupMode}` are unchanged.
mod tool_filter;
pub use tool_filter::*;

// External MCP client config (the `[mcp]` section) lives in its own module;
// re-exported here so `crate::schema::{McpConfig, McpServerConfig,
// McpTransport}` are unchanged.
mod mcp;
pub use mcp::*;

// Verifiable Intent (VI) credential config (the `[verifiable_intent]` section)
// lives in its own module; re-exported here so
// `crate::schema::VerifiableIntentConfig` is unchanged.
mod verifiable_intent;
pub use verifiable_intent::*;

// HMAC tool execution receipt config (`[agents.<alias>.tool_receipts]`) lives in
// its own module; re-exported here so `crate::schema::ToolReceiptsConfig` is
// unchanged.
mod tool_receipts;
pub use tool_receipts::*;

// Pacing controls (the `[pacing]` section: per-step timeouts, loop detection)
// live in their own module; re-exported here so `crate::schema::PacingConfig`
// is unchanged.
mod pacing;
pub use pacing::*;

// Pipeline meta-tool config (the `[pipeline]` section) lives in its own module;
// re-exported here so `crate::schema::PipelineConfig` is unchanged.
mod pipeline;
pub use pipeline::*;

// Multimodal (image) handling config (the `[multimodal]` section) lives in its
// own module; re-exported here so `crate::schema::MultimodalConfig` is
// unchanged.
mod multimodal;
pub use multimodal::*;

// Plugin system config (the `[plugins]` section: `PluginsConfig`, security and
// execution limits, per-instance entries) lives in its own module; re-exported
// here so `crate::schema::{PluginsConfig, PluginEntryConfig, ...}` are unchanged.
mod plugins;
pub use plugins::*;

// LinkedIn integration config (the `[linkedin]` section plus its content and
// image-generator sub-configs) lives in its own module; re-exported here so
// `crate::schema::LinkedInConfig` is unchanged.
mod linkedin;
pub use linkedin::*;

// Proxy-aware WebSocket connect helpers (`ws_connect_with_proxy`, `BoxedIo`,
// `ProxiedWsStream`) live in their own module; re-exported here so every
// `crate::schema::<item>` path is unchanged.
mod ws_proxy;
pub use ws_proxy::*;

// Top-level `Config` struct and its directly-embedded scalar/enum sub-configs
// live in their own module; re-exported here so every `crate::schema::Config`
// and `crate::schema::ModelProviderConfig` path is unchanged.
mod top_level;
pub use top_level::*;

// Delegate tool config (the `[delegate]` section) lives in its own module;
// re-exported here so `crate::schema::DelegateToolConfig` is unchanged.
mod delegate;
pub use delegate::*;

// Aliased agents, delegation targets and the agent/channel/runtime resolution
// `impl Config` blocks live in their own module; re-exported here so every
// `crate::schema::{AliasedAgentConfig, ...}` path is unchanged.
mod aliased_agents;
pub use aliased_agents::*;

// Autonomy/security policy defaults and the `impl RiskProfileConfig` helpers
// live in their own module; re-exported here so
// `crate::schema::default_auto_approve` etc. are unchanged.
mod autonomy_security;
pub use autonomy_security::*;

// Risk/runtime profiles, skill/knowledge/MCP bundles and the legacy per-channel
// configs live in their own module; re-exported here so every
// `crate::schema::{RiskProfileConfig, ...}` path is unchanged.
mod profiles_bundles;
pub use profiles_bundles::*;

// SOP engine config (the `[sop]` section) lives in its own module; re-exported
// here so every `crate::schema::{SopConfig, ...}` path is unchanged.
mod sop;
pub use sop::*;

// Used by `Config::validate` and its tests, which live outside the security
// module; kept as a schema-level helper rather than moving with the section.
fn default_otp_gated_actions() -> Vec<String> {
    vec![
        "shell".to_string(),
        "file_write".to_string(),
        "browser_open".to_string(),
        "browser".to_string(),
        "memory_forget".to_string(),
    ]
}

// These small `default_*` helpers are referenced both by the channels
// module (via `use super::*`) and by later config code in this module, so
// they are kept as schema-level helpers rather than moving with the
// channels section.
fn default_channel_max_concurrent_per_channel() -> usize {
    4
}

fn default_session_backend() -> String {
    "sqlite".into()
}

fn default_draft_update_interval_ms() -> u64 {
    1000
}

fn default_telegram_approval_timeout_secs() -> u64 {
    120
}

fn default_telegram_api_base_url() -> String {
    TELEGRAM_OFFICIAL_API_BASE_URL.to_string()
}

fn default_channel_approval_timeout_secs() -> u64 {
    300
}

// Shared `#[serde(default = "...")]` helpers referenced from many config
// sections (gateway, wss, memory, tunnel, channels, ...) and by their
// submodules via `use super::*`; kept at schema level.
fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_memory_backend() -> String {
    "sqlite".into()
}

fn default_tunnel_provider() -> String {
    "none".into()
}

// Used by `SchedulerConfig::default`; shared with the cron module, so it is
// kept as a schema-level helper rather than moving with the cron section.
fn default_max_run_history() -> u32 {
    50
}

// Called from `Config::validate`; kept private to this module so it does not
// move with the rest of the memory section.
fn validate_memory_rerank_config(memory: &MemoryConfig) -> Result<()> {
    if !(1..=MAX_MEMORY_RERANK_CANDIDATE_MULTIPLIER).contains(&memory.candidate_multiplier) {
        anyhow::bail!(
            "memory.candidate_multiplier must be in 1..={MAX_MEMORY_RERANK_CANDIDATE_MULTIPLIER} (got {})",
            memory.candidate_multiplier
        );
    }

    for (path, value) in [
        ("memory.min_relevance_score", memory.min_relevance_score),
        ("memory.mmr_lambda", memory.mmr_lambda),
        ("memory.importance_weight", memory.importance_weight),
        ("memory.recency_weight", memory.recency_weight),
    ] {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            anyhow::bail!("{path} must be a finite number in 0.0..=1.0 (got {value})");
        }
    }

    Ok(())
}

use crate::autonomy::AutonomyLevel;
use crate::autonomy::DelegationPolicy;
use crate::domain_matcher::DomainMatcher;
use crate::traits::{ChannelConfig, HasPropKind, PropKind};
use crate::validation_bail;
use anyhow::{Context, Result};
use directories::UserDirs;
use omnesagent_api::runtime_status::RuntimeConfigKind;
use omnesagent_macros::Configurable;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{OnceLock, RwLock};
#[cfg(unix)]
use tokio::fs::File;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;

const SUPPORTED_PROXY_SERVICE_KEYS: &[&str] = &[
    "model_provider.anthropic",
    "model_provider.compatible",
    "model_provider.copilot",
    "model_provider.gemini",
    "model_provider.glm",
    "model_provider.ollama",
    "model_provider.openai",
    "model_provider.openrouter",
    "channel.dingtalk",
    "channel.discord",
    "channel.lark",
    "channel.matrix",
    "channel.mattermost",
    "channel.nextcloud_talk",
    "channel.qq",
    "channel.signal",
    "channel.slack",
    "channel.telegram",
    "channel.wechat",
    "channel.whatsapp",
    "tool.browser",
    "tool.composio",
    "tool.file_download",
    "tool.http_request",
    "tool.pushover",
    "tool.web_search",
    "memory.embeddings",
    "tunnel.custom",
    "transcription.groq",
    "transcription.openai",
    "transcription.deepgram",
    "transcription.assemblyai",
    "transcription.google",
    "transcription.local_whisper",
];

const SUPPORTED_PROXY_SERVICE_SELECTORS: &[&str] = &[
    "model_provider.*",
    "channel.*",
    "tool.*",
    "memory.*",
    "tunnel.*",
    "transcription.*",
];

static RUNTIME_PROXY_CONFIG: OnceLock<RwLock<ProxyConfig>> = OnceLock::new();
static RUNTIME_PROXY_CONFIG_GENERATION: AtomicU64 = AtomicU64::new(0);
static RUNTIME_PROXY_CLIENT_CACHE: OnceLock<RwLock<HashMap<String, RuntimeProxyCachedClient>>> =
    OnceLock::new();

pub(crate) struct RuntimeProxyCachedClient {
    generation: u64,
    client: reqwest::Client,
}

// ── Config dir resolution ──────────────────────────────────────────

// `impl Config`, `impl Default for Config`, and the config-file persistence
// helpers now live in the `config_impl` module. The directory-resolution
// helpers below stay here because `schema::v2` and the config tests reach
// them through `super::` / module scope; `config_impl` reads them via
// `use super::*`.

fn default_config_and_data_dirs() -> Result<(PathBuf, PathBuf)> {
    let config_dir = default_config_dir()?;
    let data_dir = install_data_dir(&config_dir);
    Ok((config_dir, data_dir))
}

/// The shared instance data directory (databases and state files) of the
/// install rooted at `config_dir`. Per-agent identity and markdown live at
/// `<config-dir>/agents/<alias>/workspace/`, resolved separately via
/// `Config::agent_workspace_dir`.
///
/// Every path to the data directory goes through here: the runtime
/// resolution a daemon uses to lock its state before loading the config,
/// `Config::load_or_init`, which sets `config.data_dir`, and the V2-to-V3
/// filesystem migration's targets. A daemon refuses to start when the first
/// two disagree. Outside this crate, `omnescode` derives the same
/// `<config-dir>/data` for its daemon socket; keep them in step.
pub(crate) fn install_data_dir(config_dir: &Path) -> PathBuf {
    config_dir.join("data")
}

fn default_config_dir() -> Result<PathBuf> {
    if let Ok(custom) = std::env::var("OMNESAGENT_CONFIG_DIR") {
        let custom = custom.trim();
        if !custom.is_empty() {
            return Ok(expand_tilde_path(custom));
        }
    }

    if let Ok(home) = std::env::var("HOME")
        && !home.is_empty()
    {
        return Ok(PathBuf::from(home).join(".omnesagent"));
    }

    let home = UserDirs::new()
        .map(|u| u.home_dir().to_path_buf())
        .context("Could not find home directory")?;
    Ok(home.join(".omnesagent"))
}

/// Canonical on-disk directory for a locale's runtime/omnescode FTL catalogues:
/// `<config_dir>/data/ftl/<locale>/`. This is where `omnesagent locales fetch`
/// writes downloaded translations and where the runtime i18n loader reads them.
/// `<config_dir>` honors `OMNESAGENT_CONFIG_DIR` and otherwise defaults to
/// `~/.omnesagent`. The omnescode binary mirrors this path inline (it carries no
/// `omnesagent-*` dependency).
pub fn ftl_locale_dir(locale: &str) -> Result<PathBuf> {
    Ok(install_data_dir(&default_config_dir()?)
        .join("ftl")
        .join(locale))
}

/// The FTL catalogues that `omnesagent locales fetch` / the daemon's
/// `locales/fetch` RPC can download, as `(name, upstream-path-template,
/// output-filename)`. `{locale}` is substituted per request. This is the single
/// source of truth — a caller supplies only a catalog *name* matched against
/// this table, never a path.
pub const FTL_CATALOGS: &[(&str, &str, &str)] = &[
    (
        "cli",
        "crates/omnesagent-runtime/locales/{locale}/cli.ftl",
        "cli.ftl",
    ),
    (
        "tools",
        "crates/omnesagent-runtime/locales/{locale}/tools.ftl",
        "tools.ftl",
    ),
    (
        "omnescode",
        "apps/omnescode/locales/{locale}/omnescode.ftl",
        "omnescode.ftl",
    ),
];

/// Build a default path string by joining `relative` onto the resolved
/// platform config dir. The form sees the resolved absolute path
/// (`/home/<user>/.omnesagent/<relative>` on Linux,
/// `C:\Users\<user>\.omnesagent\<relative>` on Windows, etc.) instead of a
/// literal `~/...` token that doesn't expand on Windows. Falls back to
/// `~/.omnesagent/<relative>` if the platform dir can't be resolved (rare —
/// e.g. no HOME and `directories::UserDirs` returns None); the runtime's
/// `expand_tilde_path()` handles that literal at use-time.
///
/// Switching to platform-native config locations (`~/Library/Application
/// Support/omnesagent/` on macOS, `%APPDATA%\omnesagent\` on Windows) is the
/// schema-v3 follow-up tracked in — that needs a migration to move
/// existing users' configs.
fn default_path_under_config_dir(relative: &str) -> String {
    match default_config_dir() {
        Ok(dir) => dir.join(relative).to_string_lossy().into_owned(),
        Err(_) => format!("~/.omnesagent/{relative}"),
    }
}

/// Resolve the install a `OMNESAGENT_DATA_DIR` (or deprecated
/// `OMNESAGENT_WORKSPACE`) value points at: its config directory and that
/// install's data directory.
///
/// The value locates the install. It is the install itself when it holds a
/// `config.toml`, or the data directory beside a `.omnesagent` install (as in
/// the container images). The data directory returned is always the
/// install's own `<config-dir>/data`, which is where `Config::load_or_init`
/// keeps the databases.
pub fn resolve_config_dir_for_data(data_dir: &Path) -> (PathBuf, PathBuf) {
    let config_dir = config_dir_for_data(data_dir);
    let data_dir = install_data_dir(&config_dir);
    (config_dir, data_dir)
}

fn config_dir_for_data(data_dir: &Path) -> PathBuf {
    if data_dir.join("config.toml").exists() {
        return data_dir.to_path_buf();
    }

    if let Some(legacy_dir) = data_dir.parent().map(|parent| parent.join(".omnesagent")) {
        if legacy_dir.join("config.toml").exists() {
            return legacy_dir;
        }

        // Accept either the new "data" suffix or the legacy "workspace"
        // suffix; the V2->V3 filesystem migration renames the on-disk
        // dir but operator-set env-var paths from before the rename
        // still resolve correctly.
        if data_dir.file_name().is_some_and(|name| {
            name == std::ffi::OsStr::new("data") || name == std::ffi::OsStr::new("workspace")
        }) {
            return legacy_dir;
        }
    }

    data_dir.to_path_buf()
}

pub async fn classify_runtime_config_kind(config_path: &Path) -> RuntimeConfigKind {
    if path_is_under(config_path, &std::env::temp_dir()) {
        return RuntimeConfigKind::Temporary;
    }

    let Ok((default_config_dir, default_data_dir)) = default_config_and_data_dirs() else {
        return RuntimeConfigKind::Custom;
    };
    let Ok((resolved_config_dir, _data_dir, source)) =
        resolve_runtime_config_dirs(&default_config_dir, &default_data_dir).await
    else {
        return RuntimeConfigKind::Custom;
    };

    if !paths_equal_lexically(config_path, &resolved_config_dir.join("config.toml")) {
        return RuntimeConfigKind::Custom;
    }

    match source {
        ConfigResolutionSource::DefaultConfigDir | ConfigResolutionSource::HomebrewConfigDir => {
            RuntimeConfigKind::Default
        }
        ConfigResolutionSource::EnvConfigDir
        | ConfigResolutionSource::EnvDataDir
        | ConfigResolutionSource::EnvWorkspaceLegacy => RuntimeConfigKind::Custom,
    }
}

/// Resolve the current runtime config/data directories.
///
/// This mirrors the same precedence used by `Config::load_or_init()`:
/// `OMNESAGENT_CONFIG_DIR` > `OMNESAGENT_DATA_DIR` > `OMNESAGENT_WORKSPACE`
/// (deprecated) > defaults.
pub async fn resolve_runtime_dirs() -> Result<(PathBuf, PathBuf)> {
    let (default_omnesagent_dir, default_data_dir) = default_config_and_data_dirs()?;
    let (config_dir, data_dir, _) =
        resolve_runtime_config_dirs(&default_omnesagent_dir, &default_data_dir).await?;
    Ok((config_dir, data_dir))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConfigResolutionSource {
    EnvConfigDir,
    EnvDataDir,
    EnvWorkspaceLegacy,
    DefaultConfigDir,
    HomebrewConfigDir,
}

impl ConfigResolutionSource {
    const fn as_str(self) -> &'static str {
        match self {
            Self::EnvConfigDir => "OMNESAGENT_CONFIG_DIR",
            Self::EnvDataDir => "OMNESAGENT_DATA_DIR",
            Self::EnvWorkspaceLegacy => "OMNESAGENT_WORKSPACE",
            Self::DefaultConfigDir => "default",
            Self::HomebrewConfigDir => "homebrew",
        }
    }
}

/// Expand tilde in paths, falling back to `UserDirs` when HOME is unset.
///
/// In non-TTY environments (e.g. cron), HOME may not be set, causing
/// `shellexpand::tilde` to return the literal `~` unexpanded. This helper
/// detects that case and uses `directories::UserDirs` as a fallback.
fn expand_tilde_path(path: &str) -> PathBuf {
    let expanded = shellexpand::tilde(path);
    let expanded_str = expanded.as_ref();

    // If the path still starts with '~', tilde expansion failed (HOME unset)
    if expanded_str.starts_with('~') {
        if let Some(user_dirs) = UserDirs::new() {
            let home = user_dirs.home_dir();
            // Replace leading ~ with home directory
            if let Some(rest) = expanded_str.strip_prefix('~') {
                return home.join(rest.trim_start_matches(['/', '\\']));
            }
        }
        // If UserDirs also fails, log a warning and use the literal path
        ::omnesagent_log::record!(
            WARN,
            ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                .with_outcome(::omnesagent_log::EventOutcome::Unknown)
                .with_attrs(::serde_json::json!({"path": path})),
            "Failed to expand tilde: HOME environment variable is not set and UserDirs failed. \
             In cron/non-TTY environments, use absolute paths or set HOME explicitly."
        );
    }

    PathBuf::from(expanded_str)
}

fn path_is_under(path: &Path, root: &Path) -> bool {
    path.components()
        .zip(root.components())
        .all(|(a, b)| a == b)
        && path.components().count() >= root.components().count()
}

fn paths_equal_lexically(a: &Path, b: &Path) -> bool {
    a.components().eq(b.components())
}

/// Returns the legacy plugin directories that still hold installed plugins not
/// visible to the runtime, which now scans [`PluginsConfig::resolved_plugins_dir`].
///
/// Historically `omnesagent plugin install` wrote to `<data_dir>/plugins` (and,
/// before the data-dir rename, `<install_root>/workspace/plugins`). New V2
/// splits route packages to host data. An older split's live agent workspace
/// is not an automatic recovery source: compatible packages there require
/// explicit operator adoption through `omnesagent plugin install <path>`.
/// Supported legacy root aliases are followed, but roots overlapping a known
/// agent workspace in either direction are excluded. Target aliases are
/// excluded and duplicate roots occur once. This selection does not bind the
/// packages a later mover rescans or authenticate their historical origin.
/// Used to surface a migration hint and to drive `omnesagent plugin migrate`.
#[must_use]
pub fn legacy_plugin_dirs_with_entries(config: &Config) -> Vec<PathBuf> {
    let target = config.plugins.resolved_plugins_dir();
    let canonical_target = std::fs::canonicalize(&target).ok();
    let workspaces: Vec<PathBuf> = std::iter::once(config.agent_workspace_dir("default"))
        .chain(
            config
                .agents
                .keys()
                .map(|alias| config.agent_workspace_dir(alias)),
        )
        .chain(std::iter::once(
            config.install_root_dir().join("agents/default/workspace"),
        ))
        .collect();
    let mut seen = std::collections::HashSet::new();
    [
        config.data_dir.join("plugins"),
        config.install_root_dir().join("workspace").join("plugins"),
        v2::workspace_toplevel_v3_path(&config.install_root_dir(), "plugins"),
    ]
    .into_iter()
    .filter(|legacy| {
        if paths_equal_lexically(legacy, &target) || !dir_has_plugin(legacy) {
            return false;
        }
        let Ok(canonical) = std::fs::canonicalize(legacy) else {
            return false;
        };
        if workspaces
            .iter()
            .any(|workspace| legacy_root_overlaps_workspace(legacy, &canonical, workspace))
        {
            return false;
        }
        canonical_target.as_ref() != Some(&canonical) && seen.insert(canonical)
    })
    .collect()
}

fn legacy_root_overlaps_workspace(legacy: &Path, canonical: &Path, workspace: &Path) -> bool {
    let overlaps = |a: &Path, b: &Path| a.starts_with(b) || b.starts_with(a);
    if let Ok(workspace) = std::fs::canonicalize(workspace) {
        return overlaps(canonical, &workspace);
    }
    // An absent declared workspace is still a known role. Compare normalized
    // lexical paths conservatively; do not create it or attest it as present.
    let Some(lexical_workspace) = legacy_lexical_absolute_path(workspace) else {
        return true;
    };
    let Some(legacy) = legacy_lexical_absolute_path(legacy) else {
        return true;
    };
    if overlaps(&legacy, &lexical_workspace) || overlaps(canonical, &lexical_workspace) {
        return true;
    }
    // A declared absent child may be reached through an existing parent alias.
    // Resolve only that existing ancestor, then compare the remaining declared
    // path lexically. This does not attest the missing workspace as present.
    for ancestor in workspace.ancestors().skip(1) {
        if let Ok(parent) = std::fs::canonicalize(ancestor) {
            let Ok(suffix) = workspace.strip_prefix(ancestor) else {
                return true;
            };
            let Some(projected) = legacy_lexical_absolute_path(&parent.join(suffix)) else {
                return true;
            };
            return overlaps(canonical, &projected);
        }
    }
    false
}

fn legacy_lexical_absolute_path(path: &Path) -> Option<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    if !absolute.is_absolute() {
        return None;
    }
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    Some(normalized)
}

/// Returns `true` if `dir` holds at least one ordinary child directory containing
/// a `manifest.toml`, matching the runtime's package-directory eligibility.
fn dir_has_plugin(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        entry.file_type().is_ok_and(|kind| kind.is_dir())
            && entry.path().join("manifest.toml").exists()
    })
}

/// Detect if an executable path lives under a macOS Homebrew prefix and return
/// the Homebrew-managed config directory.
///
/// Homebrew can execute OmnesAgent from `<prefix>/Cellar/omnesagent/<version>/bin/`,
/// `<prefix>/bin/`, or `<prefix>/opt/omnesagent/bin/`.
async fn try_resolve_macos_homebrew_config_dir(exe: &Path) -> Option<PathBuf> {
    let parts = exe.iter().collect::<Vec<_>>();
    let prefix = match parts.as_slice() {
        [prefix @ .., cellar, formula, _version, bin, exe_name]
            if *cellar == std::ffi::OsStr::new("Cellar")
                && *formula == std::ffi::OsStr::new("omnesagent")
                && *bin == std::ffi::OsStr::new("bin")
                && *exe_name == std::ffi::OsStr::new("omnesagent") =>
        {
            prefix.iter().collect::<PathBuf>()
        }
        [prefix @ .., opt, formula, bin, exe_name]
            if *opt == std::ffi::OsStr::new("opt")
                && *formula == std::ffi::OsStr::new("omnesagent")
                && *bin == std::ffi::OsStr::new("bin")
                && *exe_name == std::ffi::OsStr::new("omnesagent") =>
        {
            let prefix = prefix.iter().collect::<PathBuf>();
            if !prefix.as_os_str().is_empty()
                && fs::metadata(prefix.join("Cellar"))
                    .await
                    .is_ok_and(|metadata| metadata.is_dir())
            {
                prefix
            } else {
                return None;
            }
        }
        [prefix @ .., bin, exe_name]
            if *bin == std::ffi::OsStr::new("bin")
                && *exe_name == std::ffi::OsStr::new("omnesagent") =>
        {
            let prefix = prefix.iter().collect::<PathBuf>();
            if !prefix.as_os_str().is_empty()
                && fs::metadata(prefix.join("Cellar"))
                    .await
                    .is_ok_and(|metadata| metadata.is_dir())
            {
                prefix
            } else {
                return None;
            }
        }
        _ => return None,
    };
    Some(prefix.join("var").join("omnesagent"))
}

async fn resolve_runtime_config_dirs(
    default_omnesagent_dir: &Path,
    default_data_dir: &Path,
) -> Result<(PathBuf, PathBuf, ConfigResolutionSource)> {
    if let Ok(custom_config_dir) = std::env::var("OMNESAGENT_CONFIG_DIR") {
        let custom_config_dir = custom_config_dir.trim();
        if !custom_config_dir.is_empty() {
            // If the operator ALSO set OMNESAGENT_DATA_DIR or
            // OMNESAGENT_WORKSPACE, CONFIG_DIR wins; surface the
            // collision so they know which one took effect.
            if std::env::var("OMNESAGENT_DATA_DIR")
                .ok()
                .filter(|v| !v.trim().is_empty())
                .is_some()
            {
                ::omnesagent_log::record!(
                    WARN,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                        .with_outcome(::omnesagent_log::EventOutcome::Unknown),
                    "OMNESAGENT_CONFIG_DIR is set; OMNESAGENT_DATA_DIR is ignored \
                     (CONFIG_DIR pins both the config directory and the data \
                     directory under it)."
                );
            }
            if std::env::var("OMNESAGENT_WORKSPACE")
                .ok()
                .filter(|v| !v.is_empty())
                .is_some()
            {
                ::omnesagent_log::record!(
                    WARN,
                    ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                        .with_outcome(::omnesagent_log::EventOutcome::Unknown),
                    "OMNESAGENT_CONFIG_DIR is set; OMNESAGENT_WORKSPACE (deprecated) \
                     is ignored. OMNESAGENT_WORKSPACE will be removed in a future \
                     release; switch any remaining references to OMNESAGENT_DATA_DIR."
                );
            }
            let omnesagent_dir = expand_tilde_path(custom_config_dir);
            let data_dir = install_data_dir(&omnesagent_dir);
            return Ok((
                omnesagent_dir,
                data_dir,
                ConfigResolutionSource::EnvConfigDir,
            ));
        }
    }

    if let Ok(custom_data) = std::env::var("OMNESAGENT_DATA_DIR")
        && !custom_data.trim().is_empty()
    {
        if std::env::var("OMNESAGENT_WORKSPACE")
            .ok()
            .filter(|v| !v.is_empty())
            .is_some()
        {
            ::omnesagent_log::record!(
                WARN,
                ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                    .with_outcome(::omnesagent_log::EventOutcome::Unknown),
                "OMNESAGENT_DATA_DIR and OMNESAGENT_WORKSPACE are both set; \
                 OMNESAGENT_WORKSPACE (deprecated) is ignored. \
                 OMNESAGENT_WORKSPACE will be removed in a future release."
            );
        }
        let expanded = expand_tilde_path(&custom_data);
        let (omnesagent_dir, data_dir) = resolve_config_dir_for_data(&expanded);
        return Ok((omnesagent_dir, data_dir, ConfigResolutionSource::EnvDataDir));
    }

    if let Ok(custom_workspace) = std::env::var("OMNESAGENT_WORKSPACE")
        && !custom_workspace.is_empty()
    {
        ::omnesagent_log::record!(
            WARN,
            ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note)
                .with_outcome(::omnesagent_log::EventOutcome::Unknown),
            "OMNESAGENT_WORKSPACE is deprecated; use OMNESAGENT_DATA_DIR instead. \
             OMNESAGENT_WORKSPACE will be removed in a future release."
        );
        let expanded = expand_tilde_path(&custom_workspace);
        let (omnesagent_dir, data_dir) = resolve_config_dir_for_data(&expanded);
        return Ok((
            omnesagent_dir,
            data_dir,
            ConfigResolutionSource::EnvWorkspaceLegacy,
        ));
    }

    if cfg!(target_os = "macos")
        && let Ok(exe) = std::env::current_exe()
        && let Some(homebrew_config_dir) = try_resolve_macos_homebrew_config_dir(&exe).await
    {
        let data_dir = install_data_dir(&homebrew_config_dir);
        return Ok((
            homebrew_config_dir,
            data_dir,
            ConfigResolutionSource::HomebrewConfigDir,
        ));
    }

    Ok((
        default_omnesagent_dir.to_path_buf(),
        default_data_dir.to_path_buf(),
        ConfigResolutionSource::DefaultConfigDir,
    ))
}

#[cfg(test)]
mod tests {
    #[::core::prelude::v1::test]
    fn cache_passthrough_deserializes_and_defaults_to_omitted() {
        let enabled: ModelProviderConfig = toml::from_str("cache_passthrough = true").unwrap();
        assert!(enabled.cache_passthrough);

        let serialized = toml::to_string(&ModelProviderConfig::default()).unwrap();
        assert!(
            !serialized.contains("cache_passthrough"),
            "default cache_passthrough must be omitted from serialized config"
        );
    }

    #[::core::prelude::v1::test]
    fn cache_ttl_deserializes_and_defaults_to_omitted() {
        let one_hour: ModelProviderConfig = toml::from_str("cache_ttl = \"1h\"").unwrap();
        assert_eq!(one_hour.cache_ttl, Some(CacheTtl::OneHour));
        assert_eq!(
            toml::to_string(&one_hour).unwrap(),
            "cache_ttl = \"1h\"\n",
            "an explicitly configured cache_ttl must round-trip its wire string"
        );

        let five_minutes: ModelProviderConfig = toml::from_str("cache_ttl = \"5m\"").unwrap();
        assert_eq!(five_minutes.cache_ttl, Some(CacheTtl::FiveMinutes));

        let serialized = toml::to_string(&ModelProviderConfig::default()).unwrap();
        assert!(
            !serialized.contains("cache_ttl"),
            "absent cache_ttl must be omitted from serialized config"
        );
    }

    #[::core::prelude::v1::test]
    fn cache_ttl_rejects_unknown_lifetime() {
        let parsed = toml::from_str::<ModelProviderConfig>("cache_ttl = \"2h\"");
        assert!(
            parsed.is_err(),
            "cache_ttl is a closed enum; unknown lifetimes must not parse into a silent default"
        );
    }

    // ── Nextcloud Talk: one normalized bot secret for both directions ──
    //
    // Nextcloud installs ONE secret per bot and uses it to verify inbound webhook
    // signatures AND to sign outbound bot-API requests. Before this, outbound used a
    // raw `bot_token.or_else(webhook_secret)` (no trim, no non-empty check) while
    // inbound read only a trimmed non-empty `webhook_secret`, so the two directions
    // could resolve different secrets -- or outbound could sign with whitespace.

    fn nc_cfg(webhook_secret: Option<&str>, bot_token: Option<&str>) -> NextcloudTalkConfig {
        NextcloudTalkConfig {
            webhook_secret: webhook_secret.map(ToOwned::to_owned),
            bot_token: bot_token.map(ToOwned::to_owned),
            ..NextcloudTalkConfig::default()
        }
    }

    /// Alias-only configuration must resolve, so a `bot_token`-only install keeps
    /// working through the migration window -- and, critically, now resolves for the
    /// INBOUND direction too, where it previously produced no secret at all.
    #[::core::prelude::v1::test]
    fn nextcloud_alias_only_configuration_resolves_the_secret() {
        let cfg = nc_cfg(None, Some("shared-bot-secret"));
        assert_eq!(
            cfg.resolve_bot_secret().unwrap(),
            Some("shared-bot-secret".to_string())
        );
    }

    /// The canonical field wins when both carry the SAME value, and agreeing
    /// duplicates are not treated as a conflict.
    #[::core::prelude::v1::test]
    fn nextcloud_agreeing_alias_and_canonical_resolve_to_one_secret() {
        let cfg = nc_cfg(Some("same"), Some("same"));
        assert_eq!(cfg.resolve_bot_secret().unwrap(), Some("same".to_string()));
    }

    /// Two conflicting non-empty values must be REJECTED rather than silently split
    /// into two authorities: whichever direction loses cannot match the installed bot.
    #[::core::prelude::v1::test]
    fn nextcloud_conflicting_alias_is_rejected() {
        let cfg = nc_cfg(Some("canonical"), Some("different"));
        assert!(
            cfg.resolve_bot_secret().is_err(),
            "conflicting non-empty secrets must not silently pick one"
        );
    }

    /// A blank or whitespace alias must not mask a real secret. Previously the raw
    /// `bot_token.or_else(...)` would hand whitespace to the outbound signer.
    #[::core::prelude::v1::test]
    fn nextcloud_blank_alias_does_not_mask_the_real_secret() {
        for blank in ["", "   ", "\t", "\n  "] {
            let cfg = nc_cfg(Some("real-secret"), Some(blank));
            assert_eq!(
                cfg.resolve_bot_secret().unwrap(),
                Some("real-secret".to_string()),
                "blank alias {blank:?} must not mask the canonical secret"
            );
        }
        // ...and a blank canonical falls back to a real alias rather than resolving blank.
        let cfg = nc_cfg(Some("  "), Some("real-secret"));
        assert_eq!(
            cfg.resolve_bot_secret().unwrap(),
            Some("real-secret".to_string())
        );
    }

    /// Values are trimmed, so trailing whitespace in config cannot produce a secret
    /// that differs between the two directions.
    #[::core::prelude::v1::test]
    fn nextcloud_secret_is_trimmed() {
        let cfg = nc_cfg(Some("  padded  "), None);
        assert_eq!(
            cfg.resolve_bot_secret().unwrap(),
            Some("padded".to_string())
        );
    }

    /// With nothing configured the secret is unresolved. Callers must fail closed:
    /// no outbound request is signed, and inbound verification is NOT skipped.
    /// (The gateway rejects with 401 in that case; see the inbound handler.)
    #[::core::prelude::v1::test]
    fn nextcloud_unresolved_secret_yields_none_so_callers_fail_closed() {
        assert_eq!(nc_cfg(None, None).resolve_bot_secret().unwrap(), None);
        assert_eq!(
            nc_cfg(Some(""), Some("   ")).resolve_bot_secret().unwrap(),
            None
        );
    }

    #[::core::prelude::v1::test]
    /// The whole point of splitting the accessor: an operator-facing caller
    /// must be able to tell "unconfigured" from a real 32,000, which a bare
    /// `usize` cannot express.
    #[::core::prelude::v1::test]
    fn configured_context_window_reports_unset_distinctly_from_the_fallback() {
        let mut cfg = super::Config::default();
        cfg.providers
            .models
            .ensure("ollama", "local")
            .expect("known model provider type")
            .model = Some("qwen3".to_string());
        cfg.agents.insert(
            "coder".to_string(),
            super::AliasedAgentConfig {
                model_provider: "ollama.local".into(),
                ..Default::default()
            },
        );

        // A real referenced profile without a declaration is honestly
        // unknown, while budget arithmetic retains its historical operand.
        assert_eq!(cfg.configured_model_context_window("coder"), None);
        // Budget arithmetic still gets an operand, unchanged from before.
        assert_eq!(
            cfg.effective_model_context_window("coder"),
            super::UNCONFIGURED_CONTEXT_WINDOW_FALLBACK
        );

        // Explicitly configuring the same numeric value remains distinguishable
        // from the fallback.
        cfg.providers
            .models
            .ensure("ollama", "local")
            .expect("known model provider type")
            .context_window = Some(super::UNCONFIGURED_CONTEXT_WINDOW_FALLBACK);
        assert_eq!(
            cfg.configured_model_context_window("coder"),
            Some(super::UNCONFIGURED_CONTEXT_WINDOW_FALLBACK)
        );
    }

    #[::core::prelude::v1::test]
    fn unconfigured_context_window_fallback_is_documented_stub_value() {
        // Pinned so a change to the stub is a deliberate, reviewed edit — the
        // value is load-bearing for trim budgets on unconfigured profiles.
        assert_eq!(super::UNCONFIGURED_CONTEXT_WINDOW_FALLBACK, 32_000);
    }

    #[::core::prelude::v1::test]
    fn mcp_server_config_pinned_resources_defaults_empty_and_round_trips() {
        // Absent field defaults to empty.
        let cfg: McpServerConfig = serde_json::from_str(r#"{"name":"s","command":"x"}"#).unwrap();
        assert!(cfg.pinned_resources.is_empty());

        // Present field round-trips.
        let cfg: McpServerConfig = serde_json::from_str(
            r#"{"name":"s","command":"x","pinned_resources":["file:///a","file:///b"]}"#,
        )
        .unwrap();
        assert_eq!(cfg.pinned_resources, vec!["file:///a", "file:///b"]);
    }

    #[::core::prelude::v1::test]
    fn mcp_server_config_tls_ca_cert_path_defaults_none_and_round_trips() {
        let cfg: McpServerConfig = serde_json::from_str(r#"{"name":"s","command":"x"}"#).unwrap();
        assert!(cfg.tls_ca_cert_path.is_none());
        assert!(
            !serde_json::to_value(&cfg)
                .unwrap()
                .as_object()
                .unwrap()
                .contains_key("tls_ca_cert_path")
        );

        let cfg: McpServerConfig = serde_json::from_str(
            r#"{"name":"s","transport":"http","url":"https://example.invalid/mcp","tls_ca_cert_path":"/etc/omnesagent/internal-ca.pem"}"#,
        )
        .unwrap();
        assert_eq!(
            cfg.tls_ca_cert_path.as_deref(),
            Some("/etc/omnesagent/internal-ca.pem")
        );
        assert_eq!(
            serde_json::to_value(&cfg).unwrap()["tls_ca_cert_path"],
            "/etc/omnesagent/internal-ca.pem"
        );
    }

    #[::core::prelude::v1::test]
    fn tool_filter_group_legacy_filter_builtins_key_still_parses() {
        // `filter_builtins` was declared-but-never-read and is removed.
        // `ToolFilterGroup` has no `deny_unknown_fields`, so configs
        // still carrying the key must keep deserializing (silently ignored).
        let group: super::ToolFilterGroup = toml::from_str(
            r#"
            mode = "always"
            tools = ["filesystem__*"]
            filter_builtins = true
            "#,
        )
        .expect("legacy filter_builtins key must not break deserialization");
        assert!(matches!(group.mode, super::ToolFilterGroupMode::Always));
        assert_eq!(group.tools, vec!["filesystem__*".to_string()]);
    }

    #[::core::prelude::v1::test]
    fn memory_config_rerank_stage_defaults() {
        // An empty [memory] block resolves the rerank-stage keys to their
        // inert defaults (stage off, "none" strategy).
        let cfg: super::MemoryConfig = serde_json::from_str("{}").unwrap();
        assert!(!cfg.rerank_enabled);
        assert_eq!(cfg.candidate_multiplier, 4);
        assert_eq!(cfg.rerank_threshold, 5);
        assert_eq!(cfg.rerank_strategy, "none");
        assert!((cfg.mmr_lambda - 0.7).abs() < f64::EPSILON);
        assert!((cfg.importance_weight - 0.2).abs() < f64::EPSILON);
        assert!((cfg.recency_weight - 0.1).abs() < f64::EPSILON);

        // The Default impl agrees with the serde defaults.
        let def = super::MemoryConfig::default();
        assert_eq!(def.candidate_multiplier, cfg.candidate_multiplier);
        assert_eq!(def.rerank_strategy, cfg.rerank_strategy);
        assert!((def.mmr_lambda - cfg.mmr_lambda).abs() < f64::EPSILON);
        assert!((def.importance_weight - cfg.importance_weight).abs() < f64::EPSILON);
        assert!((def.recency_weight - cfg.recency_weight).abs() < f64::EPSILON);
    }

    #[::core::prelude::v1::test]
    fn config_validate_rejects_invalid_memory_rerank_values() {
        // A NaN in any of the blend/floor floats must be rejected outright: it
        // survives `clamp` and would silently drop valid memories downstream.
        let reject_nan = |field: &str| {
            let mut config = super::Config::default();
            match field {
                "memory.min_relevance_score" => config.memory.min_relevance_score = f64::NAN,
                "memory.mmr_lambda" => config.memory.mmr_lambda = f64::NAN,
                "memory.importance_weight" => config.memory.importance_weight = f64::NAN,
                "memory.recency_weight" => config.memory.recency_weight = f64::NAN,
                other => panic!("unhandled field {other}"),
            }
            let err = config
                .validate()
                .expect_err("non-finite value must fail validation");
            assert!(
                err.to_string().contains(field),
                "expected {field}, got {err}"
            );
        };
        reject_nan("memory.min_relevance_score");
        reject_nan("memory.mmr_lambda");
        reject_nan("memory.importance_weight");
        reject_nan("memory.recency_weight");

        for multiplier in [0, super::MAX_MEMORY_RERANK_CANDIDATE_MULTIPLIER + 1] {
            let mut config = super::Config::default();
            config.memory.candidate_multiplier = multiplier;
            let err = config
                .validate()
                .expect_err("out-of-range multiplier must fail validation");
            assert!(
                err.to_string().contains("memory.candidate_multiplier"),
                "unexpected error: {err}"
            );
        }

        let mut config = super::Config::default();
        config.memory.mmr_lambda = 1.1;
        let err = config
            .validate()
            .expect_err("out-of-range MMR lambda must fail validation");
        assert!(err.to_string().contains("memory.mmr_lambda"));
    }

    #[::core::prelude::v1::test]
    fn skill_bundle_admits_skill_honors_include_and_exclude() {
        let mut bundle = super::SkillBundleConfig::default();
        assert!(bundle.admits_skill("anything"));

        bundle.include = vec!["widget".into()];
        assert!(bundle.admits_skill("widget"));
        assert!(!bundle.admits_skill("gadget"));

        bundle.exclude = vec!["widget".into()];
        assert!(!bundle.admits_skill("widget"));
    }

    #[::core::prelude::v1::test]
    fn provider_cost_categories_match_rate_struct_fields() {
        let json = serde_json::to_value(super::ProviderCostRates::default()).unwrap();
        let mut fields: Vec<String> = json
            .as_object()
            .expect("ProviderCostRates serializes to a map")
            .keys()
            .cloned()
            .collect();
        fields.sort();
        let mut registry: Vec<String> = super::PROVIDER_COST_CATEGORIES
            .iter()
            .map(|s| s.to_string())
            .collect();
        registry.sort();
        assert_eq!(
            fields, registry,
            "PROVIDER_COST_CATEGORIES must list exactly the ProviderCostRates rate-sheet fields"
        );
    }

    #[::core::prelude::v1::test]
    fn cost_category_resolves_only_rate_bearing_sections() {
        assert_eq!(
            super::cost_category_for_provider_section("providers.models"),
            Some("models")
        );
        assert_eq!(
            super::cost_category_for_provider_section("providers.tts"),
            Some("tts")
        );
        assert_eq!(
            super::cost_category_for_provider_section("providers.transcription"),
            Some("transcription")
        );
        assert_eq!(super::cost_category_for_provider_section("channels"), None);
        assert_eq!(
            super::cost_category_for_provider_section("providers.unknown"),
            None
        );
        assert_eq!(super::cost_category_for_provider_section("models"), None);
    }

    #[test]
    async fn plugin_entry_config_resolves_own_section_and_isolates_others() {
        let mut plugins = super::PluginsConfig::default();
        plugins.entries.push(super::PluginEntryConfig {
            name: "image_gen_fal".into(),
            config: std::collections::HashMap::from([("api_key".into(), "secret-a".into())]),
            ..Default::default()
        });
        plugins.entries.push(super::PluginEntryConfig {
            name: "sd_webui".into(),
            config: std::collections::HashMap::from([("base_url".into(), "http://host".into())]),
            ..Default::default()
        });

        let fal = plugins.entry_config("image_gen_fal").unwrap().unwrap();
        assert_eq!(fal.get("api_key").map(String::as_str), Some("secret-a"));
        assert!(fal.get("base_url").is_none());

        let sd = plugins.entry_config("sd_webui").unwrap().unwrap();
        assert_eq!(sd.get("base_url").map(String::as_str), Some("http://host"));
        assert!(sd.get("api_key").is_none());

        assert!(plugins.entry_config("unknown").unwrap().is_none());

        plugins.entries.push(super::PluginEntryConfig {
            name: "image_gen_fal".into(),
            config: std::collections::HashMap::new(),
            ..Default::default()
        });
        assert_eq!(
            plugins.entry_config("image_gen_fal"),
            Err(super::DuplicatePluginConfigEntry),
            "duplicate canonical rows must fail closed"
        );
    }

    #[test]
    async fn config_validation_rejects_duplicate_plugin_instance_keys() {
        let mut config = Config::default();
        config.plugins.entries = vec![
            super::PluginEntryConfig {
                name: "zpi1_same".into(),
                config: std::collections::HashMap::new(),
                ..Default::default()
            },
            super::PluginEntryConfig {
                name: "zpi1_same".into(),
                config: std::collections::HashMap::new(),
                ..Default::default()
            },
        ];

        let error = config
            .validate()
            .expect_err("duplicate plugin instance keys must invalidate config");
        assert!(error.to_string().contains("duplicate instance key"));
    }

    #[test]
    async fn git_events_routing_table_parses_dotted_keys_and_defaults() {
        let cfg: GitConfig = ::toml::from_str(
            r#"
            enabled = true
            app_id = 12345
            events_backbone = true

            [events]
            "pull_request.opened" = { sop = "pr-triage" }
            "issues.opened" = { sop = "issue-triage" }
            "issue_comment.created" = { message = true }
            "workflow_run.failed" = { sop = "ci-failure" }
            "#,
        )
        .unwrap();
        // Provider defaults to github when the field is omitted.
        assert_eq!(cfg.provider, "github");
        assert!(cfg.events_backbone);
        assert_eq!(cfg.events.len(), 4);
        let pr = &cfg.events["pull_request.opened"];
        assert_eq!(pr.sop.as_deref(), Some("pr-triage"));
        assert!(!pr.message);
        assert!(cfg.events["issue_comment.created"].message);
        assert!(cfg.events["issue_comment.created"].sop.is_none());

        // An explicit provider round-trips.
        let gitlab: GitConfig = ::toml::from_str("enabled = true\nprovider = \"gitlab\"").unwrap();
        assert_eq!(gitlab.provider, "gitlab");

        // Absent table: empty map, backbone off — the conversational
        // defaults live in the channel's router, not here.
        let bare: GitConfig = ::toml::from_str("enabled = true").unwrap();
        assert!(bare.events.is_empty());
        assert!(!bare.events_backbone);
    }

    #[test]
    async fn git_gitea_provider_fields_parse() {
        let cfg: GitConfig = ::toml::from_str(
            r#"
            enabled = true
            provider = "forgejo"
            api_base_url = "https://git.example.org/api/v1"
            access_token = "token-value"
            repos = ["team/project"]
            "#,
        )
        .unwrap();
        assert_eq!(cfg.provider, "forgejo");
        assert_eq!(
            cfg.api_base_url.as_deref(),
            Some("https://git.example.org/api/v1")
        );
        assert_eq!(cfg.access_token, "token-value");
        assert_eq!(cfg.repos, vec!["team/project"]);
    }

    #[test]
    async fn git_gitea_access_token_is_secret() {
        assert!(GitConfig::prop_is_secret("channels.git.access_token"));

        let cfg = GitConfig {
            access_token: "token-value".to_string(),
            ..Default::default()
        };
        let fields = cfg.secret_fields();
        assert!(
            fields
                .iter()
                .any(|field| field.name == "channels.git.access_token" && field.is_set),
            "Gitea/Forgejo access_token must be secret-classified and set"
        );
    }

    #[test]
    async fn amqp_validate_requires_paired_client_cert_and_key() {
        let base = AmqpConfig {
            enabled: true,
            amqp_url: "amqps://broker.example.org:5671/%2Fpublic".into(),
            exchange: "amq.topic".into(),
            routing_keys: vec!["org.example.release".into()],
            ca_cert: Some(std::path::PathBuf::from("/etc/ssl/ca.pem")),
            ..AmqpConfig::default()
        };

        // Both absent: server-auth only, valid.
        assert!(base.validate().is_ok());

        // Cert without key: invalid.
        let cert_only = AmqpConfig {
            client_cert: Some(std::path::PathBuf::from("/etc/ssl/client.pem")),
            ..base.clone()
        };
        assert!(cert_only.validate().is_err());

        // Key without cert: invalid.
        let key_only = AmqpConfig {
            client_key: Some(std::path::PathBuf::from("/etc/ssl/client.key")),
            ..base.clone()
        };
        assert!(key_only.validate().is_err());

        // Both present: valid.
        let both = AmqpConfig {
            client_cert: Some(std::path::PathBuf::from("/etc/ssl/client.pem")),
            client_key: Some(std::path::PathBuf::from("/etc/ssl/client.key")),
            ..base
        };
        assert!(both.validate().is_ok());
    }

    #[test]
    async fn amqp_dispatch_defaults_to_agent_loop() {
        assert_eq!(AmqpConfig::default().dispatch, SopDispatch::AgentLoop);
    }

    #[test]
    async fn filesystem_validate_requires_path() {
        let cfg = FilesystemConfig {
            enabled: true,
            ..FilesystemConfig::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.to_string().contains("at least one path"));
    }

    #[test]
    async fn filesystem_validate_rejects_broad_root_by_default() {
        for root in ["/etc", "/proc", "/sys", "/dev", "/tmp"] {
            let cfg = FilesystemConfig {
                enabled: true,
                paths: vec![root.into()],
                ..FilesystemConfig::default()
            };
            let err = cfg.validate().unwrap_err();
            assert!(
                err.to_string().contains("broad system root"),
                "root {root} must be rejected by default"
            );
        }
    }

    #[test]
    async fn filesystem_validate_allows_broad_root_with_escape_hatch() {
        let cfg = FilesystemConfig {
            enabled: true,
            paths: vec!["/var".into()],
            allow_broad_roots: true,
            ..FilesystemConfig::default()
        };
        assert!(cfg.validate().is_ok());
    }

    #[test]
    async fn filesystem_validate_rejects_unknown_event() {
        let cfg = FilesystemConfig {
            enabled: true,
            paths: vec!["/srv/inbox".into()],
            events: vec!["created".into(), "exploded".into()],
            ..FilesystemConfig::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.to_string().contains("exploded"));
    }

    #[test]
    async fn filesystem_validate_accepts_scoped_path() {
        let cfg = FilesystemConfig {
            enabled: true,
            paths: vec!["/srv/inbox".into()],
            ..FilesystemConfig::default()
        };
        assert!(cfg.validate().is_ok());
    }

    #[test]
    async fn filesystem_defaults_are_safe() {
        let cfg = FilesystemConfig::default();
        assert!(!cfg.enabled);
        assert!(cfg.recursive);
        assert!(!cfg.read_content);
        assert!(!cfg.follow_symlinks);
        assert!(!cfg.allow_broad_roots);
        assert_eq!(cfg.debounce_ms, 500);
        assert_eq!(cfg.settle_ms, 250);
        assert_eq!(cfg.max_content_bytes, Some(65536));
        assert_eq!(cfg.events.len(), 4);
    }
    use super::*;
    use std::ffi::OsString;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    #[cfg(unix)]
    use std::path::Path;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use tokio::sync::MutexGuard;
    use tokio::test;

    struct EnvValueGuard {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvValueGuard {
        fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
            let previous = std::env::var_os(key);
            // SAFETY: tests that mutate env vars serialize on env_override_lock().
            unsafe { std::env::set_var(key, value) };
            Self { key, previous }
        }

        fn remove(key: &'static str) -> Self {
            let previous = std::env::var_os(key);
            // SAFETY: tests that mutate env vars serialize on env_override_lock().
            unsafe { std::env::remove_var(key) };
            Self { key, previous }
        }
    }

    impl Drop for EnvValueGuard {
        fn drop(&mut self) {
            // SAFETY: tests that mutate env vars serialize on env_override_lock().
            unsafe {
                if let Some(previous) = &self.previous {
                    std::env::set_var(self.key, previous);
                } else {
                    std::env::remove_var(self.key);
                }
            }
        }
    }

    #[cfg(unix)]
    fn write_fake_op(bin_dir: &Path, script: &str) -> PathBuf {
        let op_path = bin_dir.join("op");
        std::fs::write(&op_path, script).expect("write fake op");
        let mut perms = std::fs::metadata(&op_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&op_path, perms).unwrap();
        op_path
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
    #[prefix = "test.object_array.entries"]
    struct ObjectArraySecretEntry {
        pub name: String,
        #[secret]
        pub token: Option<String>,
        #[secret]
        pub headers: HashMap<String, String>,
    }

    impl crate::config::HasPropKind for Vec<ObjectArraySecretEntry> {
        const PROP_KIND: crate::config::PropKind = crate::config::PropKind::ObjectArray;

        fn display_secret_terminals() -> Vec<&'static str> {
            ObjectArraySecretEntry::secret_field_terminals()
        }
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
    #[prefix = "test.object_array"]
    struct ObjectArraySecretFixture {
        pub entries: Vec<ObjectArraySecretEntry>,
    }

    // ── Tilde expansion ───────────────────────────────────────

    #[test]
    async fn expand_tilde_path_handles_absolute_path() {
        let path = expand_tilde_path("/absolute/path");
        assert_eq!(path, PathBuf::from("/absolute/path"));
    }

    #[test]
    async fn expand_tilde_path_handles_relative_path() {
        let path = expand_tilde_path("relative/path");
        assert_eq!(path, PathBuf::from("relative/path"));
    }

    #[test]
    async fn expand_tilde_path_expands_tilde_when_home_set() {
        // This test verifies that tilde expansion works when HOME is set.
        // In normal environments, HOME is set, so ~ should expand.
        let path = expand_tilde_path("~/.omnesagent");
        // The path should not literally start with '~' if HOME is set
        // (it should be expanded to the actual home directory)
        if std::env::var("HOME").is_ok() {
            assert!(
                !path.to_string_lossy().starts_with('~'),
                "Tilde should be expanded when HOME is set"
            );
        }
    }

    // ── Plugins dir resolution ────────────────────────────────

    #[test]
    async fn resolved_plugins_dir_passes_absolute_path_through() {
        let cfg = PluginsConfig {
            plugins_dir: "/srv/plugins".to_string(),
            ..PluginsConfig::default()
        };
        assert_eq!(cfg.resolved_plugins_dir(), PathBuf::from("/srv/plugins"));
    }

    #[test]
    async fn resolved_plugins_dir_expands_leading_tilde() {
        let cfg = PluginsConfig {
            plugins_dir: "~/.omnesagent/plugins".to_string(),
            ..PluginsConfig::default()
        };
        let resolved = cfg.resolved_plugins_dir();
        if std::env::var("HOME").is_ok() {
            assert!(!resolved.to_string_lossy().starts_with('~'));
            assert!(resolved.ends_with(".omnesagent/plugins"));
        }
    }

    /// Build a `Config` whose data dir, install root, and configured plugins dir
    /// live under `root`, and create a plugin at `<parent>/<name>/manifest.toml`.
    fn config_with_dirs(root: &Path) -> Config {
        Config {
            data_dir: root.join("data"),
            config_path: root.join("install").join("config.toml"),
            plugins: PluginsConfig {
                plugins_dir: root.join("plugins").to_string_lossy().into_owned(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn write_plugin(parent: &Path, name: &str) {
        std::fs::create_dir_all(parent.join(name)).unwrap();
        std::fs::write(parent.join(name).join("manifest.toml"), "name = \"x\"\n").unwrap();
    }

    #[test]
    async fn legacy_plugin_dirs_detects_data_and_workspace_locations() {
        let tmp = TempDir::new().unwrap();
        let config = config_with_dirs(tmp.path());
        write_plugin(&config.data_dir.join("plugins"), "fromdata");
        write_plugin(
            &config.install_root_dir().join("workspace").join("plugins"),
            "fromworkspace",
        );

        let dirs = legacy_plugin_dirs_with_entries(&config);
        assert_eq!(dirs.len(), 2, "both legacy locations should be reported");
        assert!(dirs.contains(&config.data_dir.join("plugins")));
        assert!(dirs.contains(&config.install_root_dir().join("workspace").join("plugins")));
    }

    #[test]
    async fn legacy_plugin_dirs_detects_data_destination_after_workspace_override() {
        let tmp = TempDir::new().unwrap();
        let mut config = config_with_dirs(tmp.path());
        let migrated = v2::workspace_toplevel_v3_path(&config.install_root_dir(), "plugins");
        assert_eq!(migrated, config.install_root_dir().join("data/plugins"));
        assert_ne!(migrated, config.data_dir.join("plugins"));
        write_plugin(&migrated, "resumed");
        config
            .agents
            .entry("default".into())
            .or_default()
            .workspace
            .path = Some(tmp.path().join("custom-workspace"));
        assert_ne!(
            config.agent_workspace_dir("default").join("plugins"),
            migrated
        );

        assert_eq!(legacy_plugin_dirs_with_entries(&config), vec![migrated]);
    }

    #[test]
    async fn legacy_plugin_dirs_skips_resumed_destination_when_it_is_the_target() {
        let tmp = TempDir::new().unwrap();
        let mut config = config_with_dirs(tmp.path());
        let migrated = v2::workspace_toplevel_v3_path(&config.install_root_dir(), "plugins");
        config.plugins.plugins_dir = migrated.to_string_lossy().into_owned();
        write_plugin(&migrated, "current");

        assert!(legacy_plugin_dirs_with_entries(&config).is_empty());
    }

    #[test]
    async fn legacy_plugin_dirs_empty_when_no_legacy_plugins() {
        let tmp = TempDir::new().unwrap();
        let config = config_with_dirs(tmp.path());
        // Plugin lives in the configured dir, not a legacy one.
        write_plugin(&config.plugins.resolved_plugins_dir(), "current");

        assert!(legacy_plugin_dirs_with_entries(&config).is_empty());
    }

    #[test]
    async fn legacy_plugin_dirs_skips_dir_equal_to_target() {
        let tmp = TempDir::new().unwrap();
        let mut config = config_with_dirs(tmp.path());
        // Point the configured plugins dir AT the legacy data dir.
        let data_plugins = config.data_dir.join("plugins");
        config.plugins.plugins_dir = data_plugins.to_string_lossy().into_owned();
        write_plugin(&data_plugins, "same");

        // The data-dir candidate now equals the target → not a "legacy" dir.
        assert!(legacy_plugin_dirs_with_entries(&config).is_empty());
    }

    #[test]
    async fn legacy_plugin_dirs_excludes_historical_and_live_agent_workspaces() {
        let tmp = TempDir::new().unwrap();
        let mut config = config_with_dirs(tmp.path());
        let historical = config.agent_workspace_dir("default").join("plugins");
        write_plugin(&historical, "historical");
        config
            .agents
            .entry("default".into())
            .or_default()
            .workspace
            .path = Some(tmp.path().join("custom-workspace"));
        write_plugin(
            &config.agent_workspace_dir("default").join("plugins"),
            "live",
        );
        assert!(legacy_plugin_dirs_with_entries(&config).is_empty());
    }

    #[test]
    async fn legacy_plugin_dirs_skips_canonical_target_and_duplicate_candidates() {
        let tmp = TempDir::new().unwrap();
        let mut config = config_with_dirs(tmp.path());
        let data = config.data_dir.join("plugins");
        write_plugin(&data, "current");
        config.plugins.plugins_dir = data
            .join("..")
            .join("plugins")
            .to_string_lossy()
            .into_owned();
        assert!(legacy_plugin_dirs_with_entries(&config).is_empty());

        config.plugins.plugins_dir = tmp.path().join("managed").to_string_lossy().into_owned();
        config.data_dir = config.install_root_dir().join("workspace");
        let legacy = config.data_dir.join("plugins");
        write_plugin(&legacy, "legacy");
        assert_eq!(legacy_plugin_dirs_with_entries(&config), vec![legacy]);
    }

    #[cfg(unix)]
    #[test]
    async fn legacy_plugin_dirs_follows_supported_alias_candidates() {
        use std::os::unix::fs::symlink;

        let tmp = TempDir::new().unwrap();
        let config = config_with_dirs(tmp.path());
        let outside = tmp.path().join("outside");
        write_plugin(&outside, "unselected");
        std::fs::create_dir_all(&config.data_dir).unwrap();
        let data_alias = config.data_dir.join("plugins");
        symlink(&outside, &data_alias).unwrap();
        let legacy = config.install_root_dir().join("workspace/plugins");
        std::fs::create_dir_all(&legacy).unwrap();
        symlink(outside.join("unselected"), legacy.join("linked-package")).unwrap();
        assert_eq!(
            legacy_plugin_dirs_with_entries(&config),
            vec![data_alias.clone()]
        );
        // An ordinary package directory can still use a manifest alias under
        // the existing runtime manifest-loading contract.
        std::fs::create_dir_all(legacy.join("linked-manifest")).unwrap();
        symlink(
            outside.join("unselected/manifest.toml"),
            legacy.join("linked-manifest/manifest.toml"),
        )
        .unwrap();
        assert_eq!(
            legacy_plugin_dirs_with_entries(&config),
            vec![data_alias, legacy]
        );
    }

    // ── Defaults ─────────────────────────────────────────────

    fn has_test_table(raw: &str, table: &str) -> bool {
        let exact = format!("[{table}]");
        let nested = format!("[{table}.");
        raw.lines()
            .map(str::trim)
            .any(|line| line == exact || line.starts_with(&nested))
    }

    fn mcp_server(name: &str) -> McpServerConfig {
        McpServerConfig {
            name: name.to_string(),
            ..McpServerConfig::default()
        }
    }

    /// A config with three configured servers (`a`, `b`, `c`) plus the given
    /// `[mcp_bundles.<alias>]` entries. Uses `push`/`insert` (not field
    /// assignment) to avoid `clippy::field_reassign_with_default`.
    fn config_with_mcp_bundles(bundles: Vec<(&str, McpBundleConfig)>) -> Config {
        let mut config = Config::default();
        config.mcp.servers.push(mcp_server("a"));
        config.mcp.servers.push(mcp_server("b"));
        config.mcp.servers.push(mcp_server("c"));
        for (alias, bundle) in bundles {
            config.mcp_bundles.insert(alias.to_string(), bundle);
        }
        config
    }

    #[test]
    async fn sop_untrusted_payload_defaults_are_back_compat() {
        let config: SopConfig = toml::from_str("").expect("empty SOP config should deserialize");

        assert_eq!(config.untrusted_payload_max_bytes, 8192);
        assert_eq!(config.untrusted_input_guard, "warn");
        assert_eq!(config.untrusted_guard_sensitivity, 0.7);
        assert!(config.untrusted_frame_warning);
        assert!(config.untrusted_outbound_redact);
    }

    #[test]
    async fn sop_sops_dir_defaults_to_unset_and_disables_runtime() {
        // A fresh install (no [sop] sops_dir) leaves the field unset, which keeps
        // the SOP runtime off. Operators opt in by setting a directory; the daemon
        // and CLI gate on this via `runtime_enabled()`.
        let config: SopConfig = toml::from_str("").expect("empty SOP config should deserialize");

        assert_eq!(config.sops_dir, None);
        assert!(!config.runtime_enabled());

        // SopConfig::default() must agree with the deserialized default.
        assert_eq!(SopConfig::default().sops_dir, None);
        assert!(!SopConfig::default().runtime_enabled());
    }

    #[test]
    async fn sop_empty_sops_dir_disables_runtime() {
        // An explicit empty string keeps the SOP runtime off, same as unset.
        // Whitespace-only is treated the same so a stray space cannot silently
        // enable it.
        let disabled: SopConfig =
            toml::from_str(r#"sops_dir = """#).expect("empty sops_dir should deserialize");
        assert_eq!(disabled.sops_dir.as_deref(), Some(""));
        assert!(!disabled.runtime_enabled());

        let whitespace: SopConfig =
            toml::from_str(r#"sops_dir = "   ""#).expect("whitespace sops_dir should deserialize");
        assert!(!whitespace.runtime_enabled());

        // A non-empty explicit path enables the runtime.
        let custom: SopConfig =
            toml::from_str(r#"sops_dir = "my-sops""#).expect("custom sops_dir should deserialize");
        assert!(custom.runtime_enabled());
    }

    // Regression: disabling the SOP runtime by setting an empty `sops_dir`
    // through the config API must survive save_dirty + reload. Because the
    // default is now unset (runtime off), even if the incremental writer drops
    // the empty field, a reload deserializes back to the disabled default —
    // the operator's opt-out cannot silently flip back on.
    #[test]
    async fn sop_empty_sops_dir_disable_survives_persist_and_reload() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Seed an on-disk file with the SOP runtime explicitly enabled so the
        // incremental save path runs against an existing [sop] section.
        let seed = format!(
            "schema_version = {}\n\n[sop]\nsops_dir = \"sops\"\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.sop.sops_dir = Some("sops".to_string());
        assert!(config.sop.runtime_enabled(), "seed must start enabled");

        // Operator opts out via the same call site the dashboard/TUI use.
        config
            .set_prop_persistent("sop.sops_dir", "")
            .expect("set_prop_persistent must accept an empty sops_dir");
        assert!(
            !config.sop.runtime_enabled(),
            "empty sops_dir must disable the runtime in memory"
        );

        config.save_dirty().await.unwrap();

        // Reload from disk and confirm the disable stuck.
        let written = std::fs::read_to_string(&config_path).unwrap();
        let reloaded: Config = toml::from_str(&written).expect("persisted config should reload");
        assert!(
            !reloaded.sop.runtime_enabled(),
            "empty/unset sops_dir must keep the SOP runtime off after reload; \
             on-disk file reads:\n{written}"
        );
    }

    #[test]
    async fn sop_untrusted_payload_config_overrides_deserialize() {
        let config: SopConfig = toml::from_str(
            r#"
untrusted_payload_max_bytes = 4096
untrusted_input_guard = "block"
untrusted_guard_sensitivity = 0.9
untrusted_frame_warning = false
untrusted_outbound_redact = false
"#,
        )
        .expect("SOP untrusted payload overrides should deserialize");

        assert_eq!(config.untrusted_payload_max_bytes, 4096);
        assert_eq!(config.untrusted_input_guard, "block");
        assert_eq!(config.untrusted_guard_sensitivity, 0.9);
        assert!(!config.untrusted_frame_warning);
        assert!(!config.untrusted_outbound_redact);
    }

    #[test]
    async fn mcp_bundles_empty_grants_no_servers() {
        // Secure by default: omission is not a grant.
        let config = config_with_mcp_bundles(vec![]);
        assert!(config.mcp_servers_for_bundles(&[]).is_empty());
    }

    #[test]
    async fn mcp_bundles_union_resolves_and_dedups() {
        let config = config_with_mcp_bundles(vec![
            (
                "x",
                McpBundleConfig {
                    servers: vec!["a".into(), "b".into()],
                    exclude: vec![],
                },
            ),
            (
                "y",
                McpBundleConfig {
                    servers: vec!["b".into(), "c".into()],
                    exclude: vec![],
                },
            ),
        ]);
        let names: Vec<String> = config
            .mcp_servers_for_bundles(&["x".to_string(), "y".to_string()])
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(
            names,
            vec!["a", "b", "c"],
            "union across bundles, first-seen order, deduplicated"
        );
    }

    #[test]
    async fn mcp_bundles_exclude_wins_across_bundles() {
        // `b` is included by bundle `x` but excluded by bundle `y`; deny wins.
        let config = config_with_mcp_bundles(vec![
            (
                "x",
                McpBundleConfig {
                    servers: vec!["a".into(), "b".into()],
                    exclude: vec![],
                },
            ),
            (
                "y",
                McpBundleConfig {
                    servers: vec!["c".into()],
                    exclude: vec!["b".into()],
                },
            ),
        ]);
        let names: Vec<String> = config
            .mcp_servers_for_bundles(&["x".to_string(), "y".to_string()])
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(
            names,
            vec!["a", "c"],
            "an excluded server is denied even when another referenced bundle includes it"
        );
    }

    #[test]
    async fn mcp_bundles_unknown_bundle_and_dangling_name_grant_nothing() {
        // An unknown bundle alias and a server name with no `[mcp.servers]`
        // entry both fail closed (grant nothing).
        let config = config_with_mcp_bundles(vec![(
            "x",
            McpBundleConfig {
                servers: vec!["a".into(), "ghost".into()],
                exclude: vec![],
            },
        )]);
        let names: Vec<String> = config
            .mcp_servers_for_bundles(&["x".to_string(), "missing".to_string()])
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(
            names,
            vec!["a"],
            "a dangling server name and an unknown bundle alias grant nothing"
        );
    }

    #[test]
    async fn mcp_servers_for_agent_grants_only_via_agent_bundles() {
        let mut config = config_with_mcp_bundles(vec![(
            "aa",
            McpBundleConfig {
                servers: vec!["a".into()],
                exclude: vec![],
            },
        )]);
        config.agents.insert(
            "aaatools".to_string(),
            AliasedAgentConfig {
                mcp_bundles: vec!["aa".to_string()],
                ..AliasedAgentConfig::default()
            },
        );
        config
            .agents
            .insert("defzc".to_string(), AliasedAgentConfig::default());

        let granted: Vec<String> = config
            .mcp_servers_for_agent("aaatools")
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(granted, vec!["a"], "agent is granted its bundle's servers");
        assert!(
            config.mcp_servers_for_agent("defzc").is_empty(),
            "an agent with no mcp_bundles is granted no MCP servers (omission is not a grant)"
        );
        assert!(
            config.mcp_servers_for_agent("ghost").is_empty(),
            "an unknown agent is granted no MCP servers"
        );
    }

    /// Regression test for the operator-UX warning added alongside:
    /// when MCP is enabled and `[[mcp.servers]]` is non-empty but no
    /// `[mcp_bundles.*]` exists, validate() must still succeed (warnings
    /// are non-fatal) AND every agent must resolve to zero servers
    /// (proving the secure-by-default semantics that motivate the warning
    /// are still in force).
    #[test]
    async fn validate_warns_when_servers_configured_but_no_bundles() {
        use crate::schema::{McpServerConfig, McpTransport};
        let mut config = Config::default();
        config.mcp.enabled = true;
        config.mcp.servers = vec![McpServerConfig {
            name: "fs".into(),
            transport: McpTransport::Stdio,
            command: "/usr/bin/mcp-fs".into(),
            ..Default::default()
        }];
        assert!(
            config.mcp_bundles.is_empty(),
            "test precondition: no bundles configured"
        );

        // validate() must succeed (warnings are non-fatal).
        assert!(config.validate().is_ok());

        // Behavioral assertion that motivates the warning: every agent
        // resolves to zero servers under these conditions.
        for alias in config.agents.keys() {
            assert!(
                config.mcp_servers_for_agent(alias).is_empty(),
                "every agent must get zero servers when no bundles exist"
            );
        }
    }

    /// Counterpart to `validate_warns_when_servers_configured_but_no_bundles`:
    /// once at least one `[mcp_bundles.*]` exists, the warning's
    /// precondition no longer holds. validate() still succeeds and the
    /// granted agent resolves to its bundled server.
    #[test]
    async fn validate_does_not_warn_when_a_bundle_exists() {
        use crate::schema::{McpBundleConfig, McpServerConfig, McpTransport};
        let mut config = Config::default();
        config.mcp.enabled = true;
        config.mcp.servers = vec![McpServerConfig {
            name: "fs".into(),
            transport: McpTransport::Stdio,
            command: "/usr/bin/mcp-fs".into(),
            ..Default::default()
        }];
        config.mcp_bundles.insert(
            "default".into(),
            McpBundleConfig {
                servers: vec!["fs".into()],
                exclude: vec![],
            },
        );

        assert!(config.validate().is_ok());
        // Precondition check: the warning's trigger condition is now false.
        assert!(!config.mcp_bundles.is_empty());
    }

    fn parse_test_config(raw: &str) -> Config {
        let mut merged = raw.trim().to_string();
        for table in [
            "data_retention",
            "cloud_ops",
            "conversational_ai",
            "security",
            "security_ops",
        ] {
            if has_test_table(&merged, table) {
                continue;
            }
            if !merged.is_empty() {
                merged.push_str("\n\n");
            }
            merged.push('[');
            merged.push_str(table);
            merged.push(']');
        }
        merged.push('\n');
        // Schema-deserialization helper: parses TOML directly into Config
        // WITHOUT running migration transforms. Tests that need migration
        // behavior should use `migrate_to_current` directly. This helper
        // exists so V2-shaped inputs (e.g. flat `[autonomy]` blocks) can
        // be exercised against the typed deserializer without losing
        // sections that V2→V3 strips.
        let mut config: Config = toml::from_str(&merged).unwrap();
        config
            .risk_profiles
            .entry("default".to_string())
            .or_default()
            .ensure_default_auto_approve();
        config
    }

    #[test]
    async fn multimodal_defaults_sit_at_the_effective_ceiling() {
        // The default is deliberately the clamp ceiling: an operator who never
        // configures `[multimodal]` should be able to send an ordinary photo.
        // If either number moves, move it here too rather than incidentally.
        let cfg = MultimodalConfig::default();
        assert_eq!(cfg.max_image_size_mb, 20);
        assert_eq!(cfg.effective_limits(), (4, 20));
    }

    #[test]
    async fn multimodal_effective_limits_clamp_out_of_range_values() {
        let cfg = MultimodalConfig {
            max_images: 99,
            max_image_size_mb: 512,
            ..MultimodalConfig::default()
        };
        assert_eq!(cfg.effective_limits(), (16, 20));

        let cfg = MultimodalConfig {
            max_images: 0,
            max_image_size_mb: 0,
            ..MultimodalConfig::default()
        };
        assert_eq!(cfg.effective_limits(), (1, 1));
    }

    #[test]
    async fn http_request_config_default_has_correct_values() {
        let cfg = HttpRequestConfig::default();
        assert_eq!(cfg.timeout_secs, 30);
        assert_eq!(cfg.max_response_size, 1_000_000);
        assert!(cfg.enabled);
        assert_eq!(cfg.allowed_domains, vec!["*".to_string()]);
        assert!(!cfg.allow_private_hosts);
        assert!(cfg.allowed_private_hosts.is_empty());
        assert!(cfg.secrets.is_empty());
    }

    #[test]
    async fn http_request_config_deserializes_allowed_private_hosts() {
        let c = parse_test_config(
            r#"
[http_request]
allowed_domains = ["example.com"]
allowed_private_hosts = ["localhost", "10.0.0.1"]
"#,
        );

        assert_eq!(
            c.http_request.allowed_private_hosts,
            vec!["localhost".to_string(), "10.0.0.1".to_string()]
        );
    }

    #[test]
    async fn http_request_config_deserializes_auth_secrets() {
        let c = parse_test_config(
            r#"
[http_request.secrets]
api_token = "Bearer test-token"
"#,
        );

        assert_eq!(
            c.http_request.secrets.get("api_token").map(String::as_str),
            Some("Bearer test-token")
        );
    }

    #[test]
    async fn http_request_auth_secret_names_are_validated() {
        let mut config = Config::default();
        config
            .http_request
            .secrets
            .insert("bad.name".to_string(), "Bearer test-token".to_string());

        let err = config.validate().expect_err("invalid secret name");
        assert!(
            err.to_string().contains("http_request.secrets.bad.name"),
            "validation error must name the bad auth secret path: {err}"
        );
    }

    #[test]
    async fn config_default_has_sane_values() {
        let c = Config::default();
        // No model_provider configured by default — set during Quickstart.
        assert!(c.providers.models.is_empty());
        assert!(c.providers.models.iter_entries().next().is_none());
        assert!(!c.skills.open_skills_enabled);
        assert!(!c.skills.allow_scripts);
        assert!(!c.skills.install_suggestions.enabled);
        assert_eq!(
            SkillsPromptInjectionMode::default(),
            SkillsPromptInjectionMode::Full
        );
        assert_eq!(
            c.skills.prompt_injection_mode,
            SkillsPromptInjectionMode::Full
        );
        assert_eq!(
            c.effective_skills_prompt_mode("missing"),
            SkillsPromptInjectionMode::Full
        );
        assert!(c.data_dir.to_string_lossy().contains("data"));
        assert!(c.config_path.to_string_lossy().contains("config.toml"));
    }

    #[test]
    async fn skills_table_without_prompt_mode_keeps_full_default() {
        let config = parse_test_config(
            r#"
[skills]
open_skills_enabled = false
"#,
        );

        assert_eq!(
            config.skills.prompt_injection_mode,
            SkillsPromptInjectionMode::Full
        );
    }

    #[test]
    async fn runtime_profile_prompt_injection_mode_overrides_global() {
        let mut config = Config::default();
        // A runtime profile that pins compact, and an agent pointing at it.
        config.runtime_profiles.insert(
            "compact_profile".to_string(),
            RuntimeProfileConfig {
                prompt_injection_mode: Some(SkillsPromptInjectionMode::Compact),
                ..RuntimeProfileConfig::default()
            },
        );
        // A runtime profile that leaves the mode unset (inherits global).
        config
            .runtime_profiles
            .insert("unset_profile".to_string(), RuntimeProfileConfig::default());
        config.agents.insert(
            "override".to_string(),
            AliasedAgentConfig {
                runtime_profile: "compact_profile".into(),
                ..AliasedAgentConfig::default()
            },
        );
        config.agents.insert(
            "unset".to_string(),
            AliasedAgentConfig {
                runtime_profile: "unset_profile".into(),
                ..AliasedAgentConfig::default()
            },
        );
        // An agent with no runtime profile inherits the global.
        config
            .agents
            .insert("inherit".to_string(), AliasedAgentConfig::default());

        // Profile override to Compact beats the global value.
        assert_eq!(
            config.effective_skills_prompt_mode("override"),
            SkillsPromptInjectionMode::Compact
        );
        // An unset profile, an agent with no profile, and an unknown alias all
        // inherit the global Full value.
        assert_eq!(
            config.effective_skills_prompt_mode("unset"),
            SkillsPromptInjectionMode::Full
        );
        assert_eq!(
            config.effective_skills_prompt_mode("inherit"),
            SkillsPromptInjectionMode::Full
        );
        assert_eq!(
            config.effective_skills_prompt_mode("missing"),
            SkillsPromptInjectionMode::Full
        );

        // A runtime profile that explicitly pins Full is also honored.
        config.runtime_profiles.insert(
            "full_profile".to_string(),
            RuntimeProfileConfig {
                prompt_injection_mode: Some(SkillsPromptInjectionMode::Full),
                ..RuntimeProfileConfig::default()
            },
        );
        config.agents.insert(
            "pinned_full".to_string(),
            AliasedAgentConfig {
                runtime_profile: "full_profile".into(),
                ..AliasedAgentConfig::default()
            },
        );
        assert_eq!(
            config.effective_skills_prompt_mode("pinned_full"),
            SkillsPromptInjectionMode::Full
        );

        // Flipping the global to Compact updates inheriting agents while both
        // explicit profile overrides remain authoritative.
        config.skills.prompt_injection_mode = SkillsPromptInjectionMode::Compact;
        assert_eq!(
            config.effective_skills_prompt_mode("unset"),
            SkillsPromptInjectionMode::Compact
        );
        assert_eq!(
            config.effective_skills_prompt_mode("inherit"),
            SkillsPromptInjectionMode::Compact
        );
        assert_eq!(
            config.effective_skills_prompt_mode("missing"),
            SkillsPromptInjectionMode::Compact
        );
        assert_eq!(
            config.effective_skills_prompt_mode("override"),
            SkillsPromptInjectionMode::Compact
        );
        assert_eq!(
            config.effective_skills_prompt_mode("pinned_full"),
            SkillsPromptInjectionMode::Full
        );
    }

    #[test]
    async fn runtime_profile_prompt_injection_mode_deserializes() {
        // Absent → None, so a profile that omits the key inherits the global
        // mode (no migration for existing global-only configs).
        let inherited: RuntimeProfileConfig = toml::from_str("").unwrap();
        assert_eq!(inherited.prompt_injection_mode, None);

        // Explicit wire spellings parse to their variants.
        let compact: RuntimeProfileConfig =
            toml::from_str("prompt_injection_mode = \"compact\"").unwrap();
        assert_eq!(
            compact.prompt_injection_mode,
            Some(SkillsPromptInjectionMode::Compact)
        );
        let full: RuntimeProfileConfig =
            toml::from_str("prompt_injection_mode = \"full\"").unwrap();
        assert_eq!(
            full.prompt_injection_mode,
            Some(SkillsPromptInjectionMode::Full)
        );
    }

    #[test]
    async fn resolved_agent_config_bakes_prompt_injection_mode_from_profile() {
        // The documented invariant: a runtime_profile knob must be threaded
        // through `resolved_agent_config` into `ResolvedRuntime`, consistent
        // with the `effective_*` helper.
        let raw = r#"
[skills]
prompt_injection_mode = "full"

[runtime_profiles.fast]
prompt_injection_mode = "compact"

[agents.default]
runtime_profile = "fast"

[agents.plain]
"#;
        let parsed = parse_test_config(raw);

        // Profiled agent: resolved value + effective helper both see compact.
        let resolved = parsed
            .resolved_agent_config("default")
            .expect("agent default resolves");
        assert_eq!(
            resolved.resolved.prompt_injection_mode,
            SkillsPromptInjectionMode::Compact
        );
        assert_eq!(
            parsed.effective_skills_prompt_mode("default"),
            SkillsPromptInjectionMode::Compact
        );

        // Profile-less agent: explicit global `full` remains effective.
        let plain = parsed
            .resolved_agent_config("plain")
            .expect("agent plain resolves");
        assert_eq!(
            plain.resolved.prompt_injection_mode,
            SkillsPromptInjectionMode::Full
        );
        assert_eq!(
            parsed.effective_skills_prompt_mode("plain"),
            SkillsPromptInjectionMode::Full
        );
    }

    #[test]
    async fn full_modes_do_not_emit_deprecation_warning_before_v09() {
        let mut config = Config::default();
        config.skills.prompt_injection_mode = SkillsPromptInjectionMode::Full;
        config.runtime_profiles.insert(
            "full".to_string(),
            RuntimeProfileConfig {
                prompt_injection_mode: Some(SkillsPromptInjectionMode::Full),
                ..RuntimeProfileConfig::default()
            },
        );

        assert!(
            config
                .collect_warnings()
                .into_iter()
                .all(|warning| warning.code != "skills_prompt_injection_mode_full_deprecated")
        );
    }

    #[test]
    async fn skills_install_suggestions_config_deserializes_enabled() {
        let c = parse_test_config(
            r#"
[skills.install_suggestions]
enabled = true
"#,
        );

        assert!(c.skills.install_suggestions.enabled);
    }

    #[test]
    async fn skills_install_suggestions_config_accepts_hyphen_alias() {
        let c = parse_test_config(
            r#"
[skills.install-suggestions]
enabled = true
"#,
        );

        assert!(c.skills.install_suggestions.enabled);
    }

    fn capture_log_events() -> tokio::sync::broadcast::Receiver<serde_json::Value> {
        ::omnesagent_log::try_install_capture_subscriber();
        ::omnesagent_log::subscribe_or_install()
    }

    fn drain_captured(rx: &mut tokio::sync::broadcast::Receiver<serde_json::Value>) -> String {
        let mut buf = String::new();
        while let Ok(value) = rx.try_recv() {
            buf.push_str(&serde_json::to_string(&value).unwrap_or_default());
            buf.push('\n');
        }
        buf
    }

    #[test]
    async fn config_dir_creation_error_mentions_openrc_and_path() {
        let msg = config_dir_creation_error(Path::new("/etc/omnesagent"));
        assert!(msg.contains("/etc/omnesagent"));
        assert!(msg.contains("OpenRC"));
        assert!(msg.contains("omnesagent"));
    }

    #[test]
    async fn config_schema_export_contains_expected_contract_shape() {
        #[cfg(feature = "schema-export")]
        let schema = schemars::schema_for!(Config);
        let schema_json = serde_json::to_value(&schema).expect("schema should serialize to json");

        assert_eq!(
            schema_json
                .get("$schema")
                .and_then(serde_json::Value::as_str),
            Some("https://json-schema.org/draft/2020-12/schema")
        );

        let properties = schema_json
            .get("properties")
            .and_then(serde_json::Value::as_object)
            .expect("schema should expose top-level properties");

        assert!(properties.contains_key("providers"));
        assert!(properties.contains_key("skills"));
        assert!(properties.contains_key("gateway"));
        assert!(properties.contains_key("channels"));
        assert!(!properties.contains_key("workspace_dir"));
        assert!(!properties.contains_key("config_path"));
        assert!(!properties.contains_key("model_providers"));
        assert!(!properties.contains_key("tts_providers"));
        assert!(!properties.contains_key("transcription_providers"));
        // These fields are now #[serde(skip)] cache fields, not in schema.
        assert!(!properties.contains_key("default_model_provider"));
        assert!(!properties.contains_key("api_key"));
        assert!(!properties.contains_key("default_model"));

        assert!(
            schema_json
                .get("$defs")
                .and_then(serde_json::Value::as_object)
                .is_some(),
            "schema should include reusable type definitions"
        );
    }

    #[cfg(unix)]
    #[test]
    async fn save_sets_config_permissions_on_new_file() {
        let temp = TempDir::new().expect("temp dir");
        let config_path = temp.path().join("config.toml");
        let workspace_dir = temp.path().join("workspace");

        let config = Config {
            config_path: config_path.clone(),
            data_dir: workspace_dir,
            ..Default::default()
        };

        config.save().await.expect("save config");

        let mode = std::fs::metadata(&config_path)
            .expect("config metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    async fn validate_rejects_reply_min_interval_above_upper_bound() {
        let mut config = Config::default();
        let mut tg = TelegramConfig {
            bot_token: "tok".into(),
            ..Default::default()
        };
        tg.reply_min_interval_secs = REPLY_MIN_INTERVAL_MAX_SECS + 1;
        config.channels.telegram.insert("default".to_string(), tg);
        let err = config.validate().expect_err("over-bound must be rejected");
        let msg = err.to_string();
        assert!(
            msg.contains("channels.telegram.default.reply_min_interval_secs"),
            "error must name the offending path; got: {msg}"
        );
    }

    #[test]
    async fn validate_rejects_websocket_ping_interval_above_upper_bound() {
        let mut config = Config::default();
        config.gateway.websocket_ping_interval_secs = GATEWAY_WEBSOCKET_PING_INTERVAL_MAX_SECS + 1;

        let err = config
            .validate()
            .expect_err("over-bound WebSocket ping interval must be rejected");

        assert!(
            err.to_string()
                .contains("gateway.websocket_ping_interval_secs"),
            "error must name the offending path; got: {err}"
        );
    }

    #[test]
    async fn validate_accepts_websocket_ping_interval_upper_bound() {
        let mut config = Config::default();
        config.gateway.websocket_ping_interval_secs = GATEWAY_WEBSOCKET_PING_INTERVAL_MAX_SECS;

        config
            .validate()
            .expect("WebSocket ping interval upper bound must validate");
    }

    fn plugin_entry_with_egress(hosts: &[&str], private: &[&str]) -> super::PluginEntryConfig {
        super::PluginEntryConfig {
            name: "gitea".to_string(),
            config: HashMap::new(),
            egress_hosts: hosts.iter().map(|h| (*h).to_string()).collect(),
            egress_allow_private: private.iter().map(|h| (*h).to_string()).collect(),
        }
    }

    #[test]
    async fn validate_accepts_a_well_formed_egress_grant() {
        let mut config = Config::default();
        config.plugins.entries.push(plugin_entry_with_egress(
            &["git.internal.example.com", "*.cdn.example.com", "10.0.0.5"],
            &["git.internal.example.com", "10.0.0.5"],
        ));
        config.validate().expect("a canonical grant must validate");
    }

    #[test]
    async fn validate_accepts_canonical_equivalent_ipv6_and_wildcard_carveouts() {
        let mut config = Config::default();
        config.plugins.entries.push(plugin_entry_with_egress(
            &["[::1]", "*.example.com"],
            &["[::1]", "*.sub.example.com"],
        ));

        config
            .validate()
            .expect("equivalent IPv6 and narrower wildcard grants must validate");
    }

    #[test]
    async fn validate_rejects_an_allow_all_egress_entry() {
        let mut config = Config::default();
        config
            .plugins
            .entries
            .push(plugin_entry_with_egress(&["*"], &[]));
        let err = config
            .validate()
            .expect_err("a bare '*' egress grant must be rejected");
        let text = err.to_string();
        assert!(
            text.contains("plugins.entries.gitea.egress_hosts"),
            "error must name the offending path; got: {text}"
        );
        assert!(text.contains("allow-all"), "got: {text}");
    }

    #[test]
    async fn validate_rejects_malformed_egress_entries() {
        for bad in [
            "https://api.example.com",
            "api.example.com:8443",
            "*.com",
            "",
        ] {
            let mut config = Config::default();
            config
                .plugins
                .entries
                .push(plugin_entry_with_egress(&[bad], &[]));
            assert!(
                config.validate().is_err(),
                "egress_hosts entry {bad:?} must be rejected"
            );
        }
    }

    #[test]
    async fn validate_rejects_a_carveout_for_an_ungranted_destination() {
        let mut config = Config::default();
        config.plugins.entries.push(plugin_entry_with_egress(
            &["api.example.com"],
            &["127.0.0.1"],
        ));
        let err = config
            .validate()
            .expect_err("a carveout must not stand in for a grant");
        let text = err.to_string();
        assert!(
            text.contains("egress_allow_private"),
            "error must name the offending path; got: {text}"
        );
        assert!(text.contains("not granted by egress_hosts"), "got: {text}");
    }

    #[test]
    async fn validate_rejects_a_wildcard_carveout_for_an_exact_apex_grant() {
        let mut config = Config::default();
        config.plugins.entries.push(plugin_entry_with_egress(
            &["example.com"],
            &["*.example.com"],
        ));
        let err = config
            .validate()
            .expect_err("a wildcard carveout must not widen an exact grant");
        let text = err.to_string();
        assert!(text.contains("egress_allow_private"), "got: {text}");
        assert!(text.contains("not granted by egress_hosts"), "got: {text}");
    }

    /// The allowlist must stay in the plaintext half of the entry: an operator
    /// audits the file, and an encrypted allowlist is not auditable.
    #[test]
    async fn egress_allowlist_is_not_part_of_the_secret_config_map() {
        let entry = plugin_entry_with_egress(&["api.example.com"], &[]);
        let rendered = ::toml::to_string(&entry).expect("entry serializes");
        assert!(
            rendered.contains("egress_hosts"),
            "allowlist must serialize as a plaintext field; got:\n{rendered}"
        );
        assert!(
            entry.config.is_empty(),
            "the allowlist must not have landed inside the secret config map"
        );
    }

    /// An entry authored before this field parses unchanged, and grants nothing.
    #[test]
    async fn entry_without_egress_fields_grants_nothing() {
        let entry: super::PluginEntryConfig =
            ::toml::from_str("name = \"legacy\"\n").expect("legacy entry parses");
        assert!(entry.egress_hosts.is_empty());
        assert!(entry.egress_allow_private.is_empty());

        let mut config = Config::default();
        config.plugins.entries.push(entry);
        config.validate().expect("a legacy entry stays valid");
        assert_eq!(
            config.plugins.entry_egress("legacy"),
            (Vec::new(), Vec::new())
        );
    }

    #[test]
    async fn entry_egress_returns_deny_all_for_an_unknown_alias() {
        let config = Config::default();
        assert_eq!(
            config.plugins.entry_egress("never-configured"),
            (Vec::new(), Vec::new()),
            "an unconfigured instance must resolve to no reach"
        );
    }

    #[test]
    async fn entry_egress_returns_the_authored_grant_for_a_configured_alias() {
        let mut config = Config::default();
        config.plugins.entries.push(plugin_entry_with_egress(
            &["git.internal.example.com"],
            &["git.internal.example.com"],
        ));
        assert_eq!(
            config.plugins.entry_egress("gitea"),
            (
                vec!["git.internal.example.com".to_string()],
                vec!["git.internal.example.com".to_string()]
            )
        );
    }

    #[test]
    async fn validate_rejects_zero_plugin_max_connections_per_instance() {
        let mut config = Config::default();
        config.plugins.limits.max_connections_per_instance = 0;
        let err = config
            .validate()
            .expect_err("zero max_connections_per_instance must be rejected");
        assert!(
            err.to_string()
                .contains("plugins.limits.max_connections_per_instance"),
            "error must name the offending path; got: {err}"
        );
    }

    #[test]
    async fn validate_rejects_zero_plugin_call_fuel() {
        let mut config = Config::default();
        config.plugins.limits.call_fuel = 0;
        let err = config
            .validate()
            .expect_err("zero call_fuel must be rejected");
        assert!(
            err.to_string().contains("plugins.limits.call_fuel"),
            "error must name the offending path; got: {err}"
        );
    }

    #[test]
    async fn plugin_call_timeout_defaults_and_deserializes_compatibly() {
        assert_eq!(PluginLimitsConfig::default().call_timeout_ms, 30_000);

        let limits: PluginLimitsConfig =
            toml::from_str("call_fuel = 42").expect("legacy limits deserialize");
        assert_eq!(limits.call_fuel, 42);
        assert_eq!(
            limits.call_timeout_ms, 30_000,
            "omitting the additive field keeps the host-safe default"
        );
    }

    #[test]
    async fn validate_rejects_zero_plugin_call_timeout() {
        let mut config = Config::default();
        config.plugins.limits.call_timeout_ms = 0;
        let err = config
            .validate()
            .expect_err("zero call_timeout_ms must be rejected");
        assert!(
            err.to_string().contains("plugins.limits.call_timeout_ms"),
            "error must name the offending path; got: {err}"
        );
    }

    #[test]
    async fn validate_rejects_zero_plugin_max_memory() {
        let mut config = Config::default();
        config.plugins.limits.max_memory_mb = 0;
        let err = config
            .validate()
            .expect_err("zero max_memory_mb must be rejected");
        assert!(
            err.to_string().contains("plugins.limits.max_memory_mb"),
            "error must name the offending path; got: {err}"
        );
    }

    #[test]
    async fn validate_rejects_zero_plugin_max_table_elements() {
        let mut config = Config::default();
        config.plugins.limits.max_table_elements = 0;
        let err = config
            .validate()
            .expect_err("zero max_table_elements must be rejected");
        assert!(
            err.to_string()
                .contains("plugins.limits.max_table_elements"),
            "error must name the offending path; got: {err}"
        );
    }

    #[test]
    async fn validate_rejects_zero_plugin_max_instances() {
        let mut config = Config::default();
        config.plugins.limits.max_instances = 0;
        let err = config
            .validate()
            .expect_err("zero max_instances must be rejected");
        assert!(
            err.to_string().contains("plugins.limits.max_instances"),
            "error must name the offending path; got: {err}"
        );
    }

    fn ext_reg(name: &str, url: &str, kind: &str) -> ExternalRegistry {
        ExternalRegistry {
            name: name.to_string(),
            url: url.to_string(),
            kind: kind.to_string().into(),
            enabled: true,
        }
    }

    #[test]
    async fn validate_accepts_git_extra_registry() {
        let mut config = Config::default();
        config.skills.extra_registries =
            vec![ext_reg("team", "https://github.com/acme/skills", "git")];
        assert!(config.validate().is_ok(), "valid git registry must pass");
    }

    #[test]
    async fn validate_rejects_extra_registry_non_git_kind() {
        let mut config = Config::default();
        config.skills.extra_registries =
            vec![ext_reg("team", "https://github.com/acme/skills", "zip-api")];
        let err = config
            .validate()
            .expect_err("non-git kind must be rejected");
        assert!(err.to_string().contains("kind must be 'git'"), "got: {err}");
    }

    #[test]
    async fn validate_rejects_extra_registry_duplicate_names() {
        let mut config = Config::default();
        config.skills.extra_registries = vec![
            ext_reg("team", "https://github.com/acme/a", "git"),
            ext_reg("team", "https://github.com/acme/b", "git"),
        ];
        let err = config
            .validate()
            .expect_err("duplicate names must be rejected");
        assert!(
            err.to_string().contains("duplicate name 'team'"),
            "got: {err}"
        );
    }

    #[test]
    async fn validate_rejects_extra_registry_unaddressable_names() {
        for name in [
            "team.prod",
            "team prod",
            "team/prod",
            "..",
            " team",
            "team ",
            "Team",
            "teamProd",
        ] {
            let mut config = Config::default();
            config.skills.extra_registries =
                vec![ext_reg(name, "https://github.com/acme/skills", "git")];
            let err = config
                .validate()
                .expect_err("unaddressable extra-registry name must be rejected");
            assert!(
                err.to_string().contains("registry:<name>/<skill>"),
                "name {name:?} produced unexpected error: {err}"
            );
        }
    }

    #[test]
    async fn validate_rejects_extra_registry_empty_name_or_url() {
        let mut config = Config::default();
        config.skills.extra_registries = vec![ext_reg("", "https://github.com/acme/a", "git")];
        assert!(config.validate().is_err(), "empty name must be rejected");

        let mut config = Config::default();
        config.skills.extra_registries = vec![ext_reg("team", "   ", "git")];
        assert!(config.validate().is_err(), "empty url must be rejected");
    }

    #[test]
    async fn validate_rejects_extra_registry_bad_url_scheme() {
        let mut config = Config::default();
        config.skills.extra_registries = vec![ext_reg("team", "ftp://example.com/x", "git")];
        let err = config
            .validate()
            .expect_err("non-http(s)/file scheme must be rejected");
        assert!(err.to_string().contains("scheme"), "got: {err}");
    }

    #[test]
    async fn validate_accepts_reply_min_interval_at_upper_bound() {
        let mut config = Config::default();
        let mut tg = TelegramConfig {
            bot_token: "tok".into(),
            ..Default::default()
        };
        tg.reply_min_interval_secs = REPLY_MIN_INTERVAL_MAX_SECS;
        config.channels.telegram.insert("default".to_string(), tg);
        config.validate().expect("documented upper bound must pass");
    }

    #[test]
    async fn validate_rejects_reply_queue_depth_above_ceiling() {
        let mut config = Config::default();
        let mut tg = TelegramConfig {
            bot_token: "tok".into(),
            ..Default::default()
        };
        tg.reply_min_interval_secs = 1;
        tg.reply_queue_depth_max = REPLY_QUEUE_DEPTH_CEILING + 1;
        config.channels.telegram.insert("default".to_string(), tg);
        let err = config
            .validate()
            .expect_err("over-ceiling depth must be rejected");
        let msg = err.to_string();
        assert!(
            msg.contains("channels.telegram.default.reply_queue_depth_max"),
            "error must name the offending path; got: {msg}"
        );
    }

    #[test]
    async fn validate_accepts_reply_queue_depth_at_ceiling() {
        let mut config = Config::default();
        let mut tg = TelegramConfig {
            bot_token: "tok".into(),
            ..Default::default()
        };
        tg.reply_min_interval_secs = 1;
        tg.reply_queue_depth_max = REPLY_QUEUE_DEPTH_CEILING;
        config.channels.telegram.insert("default".to_string(), tg);
        config.validate().expect("documented ceiling must pass");
    }

    #[test]
    async fn validate_accepts_reply_queue_depth_zero_meaning_default() {
        // depth=0 means "fall back to DEFAULT_REPLY_QUEUE_DEPTH at the
        // pacing-wrapper construction site." Validator must accept it.
        let mut config = Config::default();
        let mut tg = TelegramConfig {
            bot_token: "tok".into(),
            ..Default::default()
        };
        tg.reply_min_interval_secs = 1;
        tg.reply_queue_depth_max = 0;
        config.channels.telegram.insert("default".to_string(), tg);
        config
            .validate()
            .expect("zero depth means default; must pass");
    }

    #[test]
    async fn telegram_api_base_url_default_uses_official_endpoint() {
        assert_eq!(
            TelegramConfig::default().api_base_url,
            "https://api.telegram.org"
        );
    }

    #[test]
    async fn telegram_api_base_url_missing_toml_defaults_to_official_endpoint() {
        let parsed: TelegramConfig = toml::from_str(r#"bot_token = "tok""#).unwrap();

        assert_eq!(parsed.api_base_url, "https://api.telegram.org");
    }

    #[test]
    async fn telegram_api_base_url_parses_custom_endpoint() {
        let parsed: TelegramConfig = toml::from_str(
            r#"
bot_token = "tok"
api_base_url = "http://127.0.0.1:8081"
"#,
        )
        .unwrap();

        assert_eq!(parsed.api_base_url, "http://127.0.0.1:8081");
    }

    #[test]
    async fn validate_rejects_empty_telegram_api_base_url() {
        let mut config = Config::default();
        config.channels.telegram.insert(
            "default".to_string(),
            TelegramConfig {
                bot_token: "tok".into(),
                api_base_url: "   ".into(),
                ..Default::default()
            },
        );

        let err = config
            .validate()
            .expect_err("empty Telegram API base URL must be rejected");
        let msg = err.to_string();
        assert!(msg.contains("channels.telegram.default.api_base_url"));
    }

    #[test]
    async fn validate_rejects_malformed_telegram_api_base_url() {
        let mut config = Config::default();
        config.channels.telegram.insert(
            "default".to_string(),
            TelegramConfig {
                bot_token: "tok".into(),
                api_base_url: "not a url".into(),
                ..Default::default()
            },
        );

        let err = config
            .validate()
            .expect_err("malformed Telegram API base URL must be rejected");
        let msg = err.to_string();
        assert!(msg.contains("channels.telegram.default.api_base_url"));
    }

    #[test]
    async fn validate_rejects_enabled_telegram_without_bot_token() {
        let mut config = Config::default();
        config.channels.telegram.insert(
            "telegram".to_string(),
            TelegramConfig {
                enabled: true,
                bot_token: "   ".into(),
                ..Default::default()
            },
        );

        let err = config
            .validate()
            .expect_err("enabled Telegram channel must require a bot token");
        assert!(
            err.to_string()
                .contains("channels.telegram.telegram.bot_token")
        );
    }

    #[test]
    async fn validate_allows_disabled_telegram_without_bot_token() {
        let mut config = Config::default();
        config.channels.telegram.insert(
            "telegram".to_string(),
            TelegramConfig {
                enabled: false,
                bot_token: "   ".into(),
                ..Default::default()
            },
        );

        config
            .validate()
            .expect("disabled Telegram channel may be staged without a bot token");
    }

    #[test]
    async fn validate_rejects_enabled_telegram_with_unset_display_token() {
        let mut config = Config::default();
        config.channels.telegram.insert(
            "telegram".to_string(),
            TelegramConfig {
                enabled: true,
                bot_token: crate::traits::UNSET_DISPLAY.into(),
                ..Default::default()
            },
        );

        config
            .validate()
            .expect_err("enabled Telegram channel must reject the display sentinel");
    }

    #[test]
    async fn validate_rejects_disabled_telegram_with_unset_display_token() {
        let mut config = Config::default();
        config.channels.telegram.insert(
            "telegram".to_string(),
            TelegramConfig {
                enabled: false,
                bot_token: crate::traits::UNSET_DISPLAY.into(),
                ..Default::default()
            },
        );

        config
            .validate()
            .expect_err("the unset display sentinel must never become persisted config");
    }

    #[test]
    async fn validate_rejects_enabled_discord_without_bot_token() {
        let mut config = Config::default();
        config.channels.discord.insert(
            "discord".to_string(),
            DiscordConfig {
                enabled: true,
                bot_token: "   ".into(),
                ..Default::default()
            },
        );

        let err = config
            .validate()
            .expect_err("enabled Discord channel must require a bot token");
        assert!(
            err.to_string()
                .contains("channels.discord.discord.bot_token")
        );
    }

    #[test]
    async fn validate_allows_disabled_discord_without_bot_token() {
        let mut config = Config::default();
        config.channels.discord.insert(
            "discord".to_string(),
            DiscordConfig {
                enabled: false,
                bot_token: "   ".into(),
                ..Default::default()
            },
        );

        config
            .validate()
            .expect("disabled Discord channel may be staged without a bot token");
    }

    #[test]
    async fn validate_rejects_enabled_discord_with_unset_display_token() {
        let mut config = Config::default();
        config.channels.discord.insert(
            "discord".to_string(),
            DiscordConfig {
                enabled: true,
                bot_token: crate::traits::UNSET_DISPLAY.into(),
                ..Default::default()
            },
        );

        config
            .validate()
            .expect_err("enabled Discord channel must reject the display sentinel");
    }

    #[test]
    async fn validate_rejects_disabled_discord_with_unset_display_token() {
        let mut config = Config::default();
        config.channels.discord.insert(
            "discord".to_string(),
            DiscordConfig {
                enabled: false,
                bot_token: crate::traits::UNSET_DISPLAY.into(),
                ..Default::default()
            },
        );

        config
            .validate()
            .expect_err("the unset display sentinel must never become persisted config");
    }

    // Regression: an enabled Signal or Voice Call channel with empty
    // required credentials was built anyway, then its listener failed to
    // connect and the per-channel supervisor restarted it forever
    // (crashloop). Reject at config-load time instead, mirroring how
    // Telegram/Discord require `bot_token`.
    #[test]
    async fn validate_rejects_enabled_signal_without_http_url() {
        let mut config = Config::default();
        config.channels.signal.insert(
            "signal".to_string(),
            SignalConfig {
                enabled: true,
                http_url: "   ".into(),
                account: "+15551234567".into(),
                ..Default::default()
            },
        );

        let err = config
            .validate()
            .expect_err("enabled Signal channel must require an http_url");
        assert!(err.to_string().contains("channels.signal.signal.http_url"));
    }

    #[test]
    async fn validate_rejects_enabled_signal_without_account() {
        let mut config = Config::default();
        config.channels.signal.insert(
            "signal".to_string(),
            SignalConfig {
                enabled: true,
                http_url: "http://127.0.0.1:8686".into(),
                account: "   ".into(),
                ..Default::default()
            },
        );

        let err = config
            .validate()
            .expect_err("enabled Signal channel must require an account");
        assert!(err.to_string().contains("channels.signal.signal.account"));
    }

    #[test]
    async fn validate_allows_disabled_signal_without_creds() {
        let mut config = Config::default();
        config.channels.signal.insert(
            "signal".to_string(),
            SignalConfig {
                enabled: false,
                http_url: "   ".into(),
                account: "   ".into(),
                ..Default::default()
            },
        );

        config
            .validate()
            .expect("disabled Signal channel may be staged without credentials");
    }

    #[test]
    async fn validate_rejects_disabled_signal_with_unset_sentinel() {
        let mut config = Config::default();
        config.channels.signal.insert(
            "signal".to_string(),
            SignalConfig {
                enabled: false,
                http_url: crate::traits::UNSET_DISPLAY.to_string(),
                account: "+15551234567".into(),
                ..Default::default()
            },
        );

        let err = config
            .validate()
            .expect_err("the unset display sentinel must never become persisted config, even for a disabled channel");
        assert!(err.to_string().contains("channels.signal.signal.http_url"));
    }

    #[test]
    async fn signal_has_required_credentials_true_when_both_set() {
        let sig = SignalConfig {
            http_url: "http://127.0.0.1:8686".into(),
            account: "+15551234567".into(),
            ..Default::default()
        };
        assert!(sig.has_required_credentials());
    }

    #[test]
    async fn signal_has_required_credentials_false_when_either_blank() {
        let missing_http_url = SignalConfig {
            http_url: "   ".into(),
            account: "+15551234567".into(),
            ..Default::default()
        };
        assert!(!missing_http_url.has_required_credentials());

        let missing_account = SignalConfig {
            http_url: "http://127.0.0.1:8686".into(),
            account: "   ".into(),
            ..Default::default()
        };
        assert!(!missing_account.has_required_credentials());

        assert!(!SignalConfig::default().has_required_credentials());
    }

    #[test]
    async fn validate_rejects_enabled_voice_call_without_account_id() {
        let mut config = Config::default();
        config.channels.voice_call.insert(
            "voice".to_string(),
            crate::scattered_types::VoiceCallConfig {
                enabled: true,
                account_id: "   ".into(),
                auth_token: "tok".into(),
                from_number: "+15551234567".into(),
                ..Default::default()
            },
        );

        let err = config
            .validate()
            .expect_err("enabled Voice Call channel must require an account_id");
        assert!(
            err.to_string()
                .contains("channels.voice_call.voice.account_id")
        );
    }

    #[test]
    async fn validate_rejects_enabled_voice_call_without_auth_token() {
        let mut config = Config::default();
        config.channels.voice_call.insert(
            "voice".to_string(),
            crate::scattered_types::VoiceCallConfig {
                enabled: true,
                account_id: "AC123".into(),
                auth_token: "   ".into(),
                from_number: "+15551234567".into(),
                ..Default::default()
            },
        );

        let err = config
            .validate()
            .expect_err("enabled Voice Call channel must require an auth_token");
        assert!(
            err.to_string()
                .contains("channels.voice_call.voice.auth_token")
        );
    }

    #[test]
    async fn validate_rejects_enabled_voice_call_without_from_number() {
        let mut config = Config::default();
        config.channels.voice_call.insert(
            "voice".to_string(),
            crate::scattered_types::VoiceCallConfig {
                enabled: true,
                account_id: "AC123".into(),
                auth_token: "tok".into(),
                from_number: "   ".into(),
                ..Default::default()
            },
        );

        let err = config
            .validate()
            .expect_err("enabled Voice Call channel must require a from_number");
        assert!(
            err.to_string()
                .contains("channels.voice_call.voice.from_number")
        );
    }

    #[test]
    async fn validate_allows_disabled_voice_call_without_creds() {
        let mut config = Config::default();
        config.channels.voice_call.insert(
            "voice".to_string(),
            crate::scattered_types::VoiceCallConfig {
                enabled: false,
                account_id: "   ".into(),
                auth_token: "   ".into(),
                from_number: "   ".into(),
                ..Default::default()
            },
        );

        config
            .validate()
            .expect("disabled Voice Call channel may be staged without credentials");
    }

    // Regression (fail closed, both PAT-backed forge providers): a Gitea or
    // Forgejo alias with an access token but no api_base_url must be rejected
    // at config-validation time. The old behavior silently defaulted to
    // https://gitea.com/api/v1 and sent the configured token there.
    #[test]
    async fn validate_rejects_gitea_forgejo_without_api_base_url() {
        for provider in ["gitea", "forgejo"] {
            let mut config = Config::default();
            config.channels.git.insert(
                "default".to_string(),
                GitConfig {
                    enabled: true,
                    provider: provider.to_string(),
                    access_token: "tok".into(),
                    ..Default::default()
                },
            );

            let err = config
                .validate()
                .expect_err("a PAT-backed forge provider without api_base_url must be rejected");
            let msg = err.to_string();
            assert!(msg.contains("channels.git.default.api_base_url"), "{msg}");
            assert!(msg.contains(provider), "{msg}");
        }
    }

    #[test]
    async fn validate_rejects_blank_or_malformed_gitea_api_base_url() {
        for bad in ["   ", "not a url", "ftp://git.example.org/api/v1"] {
            let mut config = Config::default();
            config.channels.git.insert(
                "default".to_string(),
                GitConfig {
                    enabled: true,
                    provider: "gitea".to_string(),
                    access_token: "tok".into(),
                    api_base_url: Some(bad.to_string()),
                    ..Default::default()
                },
            );

            let err = config
                .validate()
                .expect_err("blank or malformed Gitea API base URL must be rejected");
            let msg = err.to_string();
            assert!(msg.contains("channels.git.default.api_base_url"), "{msg}");
        }
    }

    // The requirement is scoped to enabled PAT-backed providers: the GitHub
    // provider has no api_base_url, and a disabled half-filled Gitea block
    // must not fail the whole config.
    #[test]
    async fn validate_git_api_base_url_scope() {
        let mut config = Config::default();
        config.channels.git.insert(
            "default".to_string(),
            GitConfig {
                enabled: true,
                ..Default::default()
            },
        );
        config.channels.git.insert(
            "disabled_gitea".to_string(),
            GitConfig {
                enabled: false,
                provider: "gitea".to_string(),
                access_token: "tok".into(),
                ..Default::default()
            },
        );
        config
            .validate()
            .expect("github provider and disabled gitea alias must validate");

        let mut config = Config::default();
        config.channels.git.insert(
            "default".to_string(),
            GitConfig {
                enabled: true,
                provider: "forgejo".to_string(),
                access_token: "tok".into(),
                api_base_url: Some("https://git.example.org/api/v1".to_string()),
                ..Default::default()
            },
        );
        config
            .validate()
            .expect("forgejo with an explicit api_base_url must validate");
    }

    #[test]
    async fn observability_enums_deserialize_legacy_string_values() {
        // Backward compat: TOML configs written before the enum conversion
        // stored these as bare strings. They must still parse.
        let toml = r#"
backend = "otel"
log_persistence = "full"
log_tool_io = "off"
"#;
        let o: ObservabilityConfig = toml::from_str(toml).unwrap();
        assert_eq!(o.backend, ObservabilityBackend::Otel);
        assert_eq!(o.log_persistence, LogPersistence::Full);
        assert_eq!(o.log_tool_io, LogToolIo::Off);

        // Legacy alias key `runtime_trace_mode` still maps to log_persistence.
        let aliased: ObservabilityConfig = toml::from_str("runtime_trace_mode = \"none\"").unwrap();
        assert_eq!(aliased.log_persistence, LogPersistence::None);

        // Round-trip: serialize back to the same wire strings the runtime
        // boundary (`to_log_config`) and downstream `from_raw` expect.
        assert_eq!(ObservabilityBackend::Otel.as_wire(), "otel");
        assert_eq!(LogPersistence::Full.as_wire(), "full");
        assert_eq!(LogPersistence::Rotating.as_wire(), "rotating");
        assert_eq!(LogToolIo::Off.as_wire(), "off");

        // The `rotating` mode and its knobs parse from a TOML config.
        let rot: ObservabilityConfig = toml::from_str(
            "log_persistence = \"rotating\"\nlog_persistence_max_bytes = 1048576\nlog_persistence_rotate_daily = false\nlog_persistence_retention_max_files = 3\nlog_persistence_retention_max_age_days = 14\n",
        )
        .unwrap();
        assert_eq!(rot.log_persistence, LogPersistence::Rotating);
        assert_eq!(rot.log_persistence_max_bytes, 1_048_576);
        assert!(!rot.log_persistence_rotate_daily);
        assert_eq!(rot.log_persistence_retention_max_files, 3);
        assert_eq!(rot.log_persistence_retention_max_age_days, 14);

        // Rotation knobs are serde-defaulted, so existing configs that omit them
        // still parse and pick up the documented defaults.
        let defaults: ObservabilityConfig = toml::from_str("backend = \"none\"").unwrap();
        assert_eq!(defaults.log_persistence_max_bytes, 0);
        assert!(defaults.log_persistence_rotate_daily);
        assert_eq!(defaults.log_persistence_retention_max_files, 7);
        assert_eq!(defaults.log_persistence_retention_max_age_days, 0);

        // log_llm_request_payload parses leniently and round-trips its wire form.
        let payload: ObservabilityConfig =
            toml::from_str("log_llm_request_payload = \"full\"").unwrap();
        assert_eq!(payload.log_llm_request_payload, LogLlmRequestPayload::Full);
        assert_eq!(LogLlmRequestPayload::Off.as_wire(), "off");
    }

    #[test]
    async fn observability_backend_unknown_falls_back_to_default() {
        let parsed: ObservabilityConfig = toml::from_str("backend = \"bogus\"").unwrap();
        assert_eq!(parsed.backend, ObservabilityBackend::None);
        let noop: ObservabilityConfig = toml::from_str("backend = \"noop\"").unwrap();
        assert_eq!(noop.backend, ObservabilityBackend::None);
        let otlp: ObservabilityConfig = toml::from_str("backend = \"otlp\"").unwrap();
        assert_eq!(otlp.backend, ObservabilityBackend::Otel);
        let otel_alias: ObservabilityConfig =
            toml::from_str("backend = \"opentelemetry\"").unwrap();
        assert_eq!(otel_alias.backend, ObservabilityBackend::Otel);
    }

    #[test]
    async fn runtime_kind_unknown_falls_back_to_default() {
        let docker: RuntimeConfig = toml::from_str("kind = \"docker\"").unwrap();
        assert_eq!(docker.kind, RuntimeKind::Docker);
        let cf: RuntimeConfig = toml::from_str("kind = \"cloudflare\"").unwrap();
        assert_eq!(cf.kind, RuntimeKind::Cloudflare);
        let bogus: RuntimeConfig = toml::from_str("kind = \"bogus\"").unwrap();
        assert_eq!(bogus.kind, RuntimeKind::Native);
        let empty: RuntimeConfig = toml::from_str("kind = \"\"").unwrap();
        assert_eq!(empty.kind, RuntimeKind::Native);
        assert_eq!(RuntimeKind::default(), RuntimeKind::Native);
    }

    #[test]
    async fn observability_config_default() {
        let o = ObservabilityConfig::default();
        assert_eq!(o.backend, ObservabilityBackend::None);
        assert_eq!(o.log_persistence, LogPersistence::Rolling);
        assert_eq!(o.log_persistence_path, "state/runtime-trace.jsonl");
        assert_eq!(o.log_persistence_max_entries, 200);
        assert_eq!(o.log_tool_io, LogToolIo::Redacted);
        assert_eq!(o.log_tool_io_truncate_bytes, 40960);
        assert!(o.log_tool_io_denylist.is_empty());
        assert_eq!(o.log_llm_request_payload, LogLlmRequestPayload::Off);
    }

    #[test]
    async fn risk_profile_default_mirrors_v2_autonomy_safety_defaults() {
        let a = RiskProfileConfig::default();
        assert_eq!(a.level, AutonomyLevel::Supervised);
        assert!(a.workspace_only);
        assert!(a.allowed_commands.contains(&"git".to_string()));
        assert!(a.allowed_commands.contains(&"cargo".to_string()));
        assert!(
            !a.forbidden_paths.is_empty(),
            "default forbidden_paths must not be empty"
        );
        #[cfg(not(target_os = "windows"))]
        assert!(
            a.forbidden_paths.iter().any(|p| p == "/etc"),
            "Default forbidden_paths must include /etc on Unix"
        );
        #[cfg(target_os = "windows")]
        assert!(
            a.forbidden_paths.iter().any(|p| p == "C:\\Windows"),
            "Default forbidden_paths must include C:\\Windows on Windows"
        );
        assert!(
            a.forbidden_paths.contains(&"~/.ssh".to_string()),
            "Default forbidden_paths must include ~/.ssh"
        );
        assert!(a.require_approval_for_medium_risk);
        assert!(a.block_high_risk_commands);
        assert!(a.shell_env_passthrough.is_empty());
        assert!(a.allowed_tools.is_empty());
        assert!(!a.deny_all_tools);
    }

    /// `allowed_tools = []` keeps its legacy meaning (unrestricted): it does
    /// not flip the profile to deny-all, and it is not a validation error.
    #[test]
    async fn risk_profile_empty_allowed_tools_stays_unrestricted() {
        let raw = r#"
default_temperature = 0.7

[risk_profiles.default]
allowed_tools = []
"#;
        let parsed = parse_test_config(raw);
        let profile = parsed.risk_profiles.get("default").unwrap();
        assert!(
            profile.allowed_tools.is_empty(),
            "explicit [] must deserialize as the legacy unrestricted state"
        );
        assert!(!profile.deny_all_tools);
        parsed.validate().expect("allowed_tools = [] must validate");
    }

    #[test]
    async fn risk_profile_effective_allowed_tools_normalizes_legacy_deny_all_sentinel() {
        let mut profile = RiskProfileConfig::default();
        assert_eq!(profile.effective_allowed_tools(), None);

        profile.allowed_tools = vec![RiskProfileConfig::LEGACY_DENY_ALL_TOOLS_SENTINEL.into()];
        assert_eq!(profile.effective_allowed_tools(), Some(vec![]));

        profile.allowed_tools = vec![
            RiskProfileConfig::LEGACY_DENY_ALL_TOOLS_SENTINEL.into(),
            RiskProfileConfig::LEGACY_DENY_ALL_TOOLS_SENTINEL.into(),
        ];
        assert_eq!(profile.effective_allowed_tools(), Some(vec![]));

        profile.allowed_tools = vec![
            RiskProfileConfig::LEGACY_DENY_ALL_TOOLS_SENTINEL.into(),
            "shell".into(),
        ];
        assert_eq!(
            profile.effective_allowed_tools(),
            Some(vec!["shell".into()])
        );
    }

    /// `deny_all_tools = true` is the explicit deny-all representation and is
    /// valid on its own.
    #[test]
    async fn risk_profile_deny_all_tools_flag_parses_and_validates() {
        let raw = r#"
default_temperature = 0.7

[risk_profiles.default]
deny_all_tools = true
"#;
        let parsed = parse_test_config(raw);
        let profile = parsed.risk_profiles.get("default").unwrap();
        assert!(profile.deny_all_tools);
        parsed
            .validate()
            .expect("deny_all_tools = true alone must validate");
    }

    /// `deny_all_tools = true` combined with a non-empty `allowed_tools` is a
    /// configuration error and must fail validation loudly instead of one
    /// side silently winning.
    #[test]
    async fn risk_profile_deny_all_tools_with_nonempty_allowed_tools_is_rejected() {
        let raw = r#"
default_temperature = 0.7

[risk_profiles.default]
deny_all_tools = true
allowed_tools = ["shell"]
"#;
        let parsed = parse_test_config(raw);
        let err = parsed
            .validate()
            .expect_err("deny_all_tools + non-empty allowed_tools must be rejected");
        assert!(
            err.to_string()
                .contains("risk_profiles.default.deny_all_tools"),
            "error must name the offending path, got: {err}"
        );
    }

    #[test]
    async fn runtime_config_default() {
        let r = RuntimeConfig::default();
        assert_eq!(r.kind, RuntimeKind::Native);
        assert_eq!(r.docker.image, "alpine:3.20");
        assert_eq!(r.docker.network, "none");
        assert_eq!(r.docker.memory_limit_mb, Some(512));
        assert_eq!(r.docker.cpu_limit, Some(1.0));
        assert!(r.docker.read_only_rootfs);
        assert!(r.docker.mount_workspace);
    }

    #[test]
    async fn heartbeat_config_default() {
        let h = HeartbeatConfig::default();
        // Heartbeat defaults to disabled. Enabling requires the user to
        // also bind it to a configured agent — there is no default agent
        // for heartbeat to fall through to.
        assert!(!h.enabled);
        assert!(h.agent.is_empty());
        assert_eq!(h.interval_minutes, 30);
        assert!(h.message.is_none());
        assert!(h.target.is_none());
        assert!(h.to.is_none());
    }

    #[test]
    async fn heartbeat_config_parses_delivery_aliases() {
        let raw = r#"
enabled = true
interval_minutes = 10
message = "Ping"
channel = "telegram"
recipient = "42"
"#;
        let parsed: HeartbeatConfig = toml::from_str(raw).unwrap();
        assert!(parsed.enabled);
        assert_eq!(parsed.interval_minutes, 10);
        assert_eq!(parsed.message.as_deref(), Some("Ping"));
        assert_eq!(parsed.target.as_deref(), Some("telegram"));
        assert_eq!(parsed.to.as_deref(), Some("42"));
    }

    #[test]
    async fn scheduler_config_default() {
        let s = SchedulerConfig::default();
        assert!(s.enabled);
        assert!(s.catch_up_on_startup);
        assert_eq!(s.max_run_history, 50);
    }

    #[test]
    async fn scheduler_config_serde_roundtrip() {
        let s = SchedulerConfig {
            enabled: false,
            max_tasks: 16,
            max_concurrent: 2,
            catch_up_on_startup: false,
            max_run_history: 100,
        };
        let json = serde_json::to_string(&s).unwrap();
        let parsed: SchedulerConfig = serde_json::from_str(&json).unwrap();
        assert!(!parsed.enabled);
        assert!(!parsed.catch_up_on_startup);
        assert_eq!(parsed.max_run_history, 100);
    }

    #[test]
    async fn config_defaults_scheduler_when_section_missing() {
        let toml_str = r#"
workspace_dir = "/tmp/workspace"
config_path = "/tmp/config.toml"
default_temperature = 0.7
"#;

        let parsed = parse_test_config(toml_str);
        assert!(parsed.scheduler.enabled);
        assert!(parsed.scheduler.catch_up_on_startup);
        assert_eq!(parsed.scheduler.max_run_history, 50);
        assert!(parsed.cron.is_empty());
    }

    #[test]
    async fn memory_config_default_hygiene_settings() {
        let m = MemoryConfig::default();
        assert_eq!(m.backend, "sqlite");
        assert!(m.auto_save);
        assert!(m.hygiene_enabled);
        assert_eq!(m.archive_after_days, 7);
        assert_eq!(m.purge_after_days, 30);
        assert_eq!(m.conversation_retention_days, 30);
        assert_eq!(m.search_mode, SearchMode::Hybrid);
    }

    #[test]
    async fn memory_types_and_extract_facts_default_off() {
        let m = MemoryConfig::default();
        assert!(!m.consolidation_extract_facts);
        assert!(!m.types.enabled);
    }

    #[test]
    async fn memory_config_without_types_keys_deserializes_off() {
        // Back-compat: configs written before [memory.types] and
        // consolidation_extract_facts existed must still parse, with both off.
        let toml_str = r#"
workspace_dir = "/tmp/workspace"
config_path = "/tmp/config.toml"
default_temperature = 0.7

[memory]
backend = "sqlite"
auto_save = true
"#;
        let parsed = parse_test_config(toml_str);
        assert!(!parsed.memory.consolidation_extract_facts);
        assert!(!parsed.memory.types.enabled);
    }

    #[test]
    async fn memory_types_keys_parse_when_set() {
        let toml_str = r#"
workspace_dir = "/tmp/workspace"
config_path = "/tmp/config.toml"
default_temperature = 0.7

[memory]
backend = "sqlite"
consolidation_extract_facts = true

[memory.types]
enabled = true
"#;
        let parsed = parse_test_config(toml_str);
        assert!(parsed.memory.consolidation_extract_facts);
        assert!(parsed.memory.types.enabled);
    }

    #[test]
    async fn search_mode_config_deserialization() {
        let toml_str = r#"
workspace_dir = "/tmp/workspace"
config_path = "/tmp/config.toml"
default_temperature = 0.7

[memory]
backend = "sqlite"
auto_save = true
search_mode = "bm25"
"#;
        let parsed = parse_test_config(toml_str);
        assert_eq!(parsed.memory.search_mode, SearchMode::Bm25);

        let toml_str_embedding = r#"
workspace_dir = "/tmp/workspace"
config_path = "/tmp/config.toml"
default_temperature = 0.7

[memory]
backend = "sqlite"
auto_save = true
search_mode = "embedding"
"#;
        let parsed = parse_test_config(toml_str_embedding);
        assert_eq!(parsed.memory.search_mode, SearchMode::Embedding);

        let toml_str_hybrid = r#"
workspace_dir = "/tmp/workspace"
config_path = "/tmp/config.toml"
default_temperature = 0.7

[memory]
backend = "sqlite"
auto_save = true
search_mode = "hybrid"
"#;
        let parsed = parse_test_config(toml_str_hybrid);
        assert_eq!(parsed.memory.search_mode, SearchMode::Hybrid);
    }

    #[test]
    async fn search_mode_defaults_to_hybrid_when_omitted() {
        let toml_str = r#"
workspace_dir = "/tmp/workspace"
config_path = "/tmp/config.toml"
default_temperature = 0.7

[memory]
backend = "sqlite"
auto_save = true
"#;
        let parsed = parse_test_config(toml_str);
        assert_eq!(parsed.memory.search_mode, SearchMode::Hybrid);
    }

    #[test]
    async fn search_mode_serde_roundtrip() {
        let json_bm25 = serde_json::to_string(&SearchMode::Bm25).unwrap();
        assert_eq!(json_bm25, "\"bm25\"");
        let parsed: SearchMode = serde_json::from_str(&json_bm25).unwrap();
        assert_eq!(parsed, SearchMode::Bm25);

        let json_embedding = serde_json::to_string(&SearchMode::Embedding).unwrap();
        assert_eq!(json_embedding, "\"embedding\"");
        let parsed: SearchMode = serde_json::from_str(&json_embedding).unwrap();
        assert_eq!(parsed, SearchMode::Embedding);

        let json_hybrid = serde_json::to_string(&SearchMode::Hybrid).unwrap();
        assert_eq!(json_hybrid, "\"hybrid\"");
        let parsed: SearchMode = serde_json::from_str(&json_hybrid).unwrap();
        assert_eq!(parsed, SearchMode::Hybrid);
    }

    #[test]
    async fn storage_two_tier_defaults_empty() {
        let storage = StorageConfig::default();
        assert!(storage.sqlite.is_empty());
        assert!(storage.postgres.is_empty());
        assert!(storage.qdrant.is_empty());
        assert!(storage.markdown.is_empty());
        assert!(storage.lucid.is_empty());
    }

    #[test]
    async fn storage_postgres_alias_pgvector_roundtrip() {
        let toml = r#"
            [postgres.default]
            db_url = "postgres://user:pw@host/db"
            vector_enabled = true
            vector_dimensions = 768
        "#;
        let parsed: StorageConfig = toml::from_str(toml).unwrap();
        let pg = parsed.postgres.get("default").expect("alias present");
        assert_eq!(pg.db_url.as_deref(), Some("postgres://user:pw@host/db"));
        assert!(pg.vector_enabled);
        assert_eq!(pg.vector_dimensions, 768);
    }

    #[test]
    async fn storage_postgres_pgvector_defaults_when_omitted() {
        let toml = r#"
            [postgres.default]
        "#;
        let parsed: StorageConfig = toml::from_str(toml).unwrap();
        let pg = parsed.postgres.get("default").expect("alias present");
        assert!(!pg.vector_enabled);
        assert_eq!(pg.vector_dimensions, 1536);
        assert_eq!(pg.schema, "public");
        assert_eq!(pg.table, "memories");
    }

    #[test]
    async fn ollama_alias_tuning_fields_roundtrip() {
        // Ollama-specific tuning lives on `OllamaModelProviderConfig`,
        // not on the generic `ModelProviderConfig` base. These knobs
        // ride alongside the flattened `base` so a TOML alias like
        // `[providers.models.ollama.local]` accepts them at the same
        // level as `model`, `api_key`, etc.
        let toml = r#"
            num_ctx = 16384
            num_predict = 4096
            temperature_override = 0.5
        "#;
        let parsed: OllamaModelProviderConfig = toml::from_str(toml).unwrap();
        assert_eq!(parsed.num_ctx, Some(16384));
        assert_eq!(parsed.num_predict, Some(4096));
        assert_eq!(parsed.temperature_override, Some(0.5));

        let serialized = toml::to_string(&parsed).unwrap();
        let reparsed: OllamaModelProviderConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(reparsed.num_ctx, Some(16384));
        assert_eq!(reparsed.num_predict, Some(4096));
        assert_eq!(reparsed.temperature_override, Some(0.5));
    }

    #[test]
    async fn ollama_alias_tuning_fields_default_to_none() {
        let toml = r#"
            api_key = "sk-test"
        "#;
        let parsed: OllamaModelProviderConfig = toml::from_str(toml).unwrap();
        assert!(parsed.num_ctx.is_none());
        assert!(parsed.num_predict.is_none());
        assert!(parsed.temperature_override.is_none());
    }

    #[::core::prelude::v1::test]
    fn tool_result_image_policy_defaults_and_round_trips() {
        let parsed: ModelProviderConfig = toml::from_str("").unwrap();
        assert_eq!(
            parsed.tool_result_image_policy,
            ToolResultImagePolicy::ImageUrl
        );
        let serialized = toml::to_string(&parsed).unwrap();
        assert!(
            !serialized.contains("tool_result_image_policy"),
            "the default policy should remain omitted from serialized config"
        );

        let parsed: ModelProviderConfig =
            toml::from_str("tool_result_image_policy = \"omit\"").unwrap();
        assert_eq!(parsed.tool_result_image_policy, ToolResultImagePolicy::Omit);
        let serialized = toml::to_string(&parsed).unwrap();
        assert!(serialized.contains("tool_result_image_policy = \"omit\""));
    }

    #[::core::prelude::v1::test]
    fn grok_cli_alias_requires_working_directory() {
        let error = toml::from_str::<GrokCliModelProviderConfig>("model = \"grok-4.5\"")
            .expect_err("missing ACP session boundary must fail");
        assert!(error.to_string().contains("working_directory"));
    }

    #[::core::prelude::v1::test]
    fn grok_cli_alias_deserializes_explicit_working_directory() {
        let parsed: GrokCliModelProviderConfig = toml::from_str(
            r#"
                model = "grok-4.5"
                working_directory = "/srv/omnesagent/workspace"
                env_passthrough = ["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"]
                max_acp_stdout_bytes = 8388608
            "#,
        )
        .expect("explicit Grok ACP cwd");
        assert_eq!(parsed.working_directory, "/srv/omnesagent/workspace");
        assert_eq!(
            parsed.env_passthrough,
            ["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"]
        );
        assert_eq!(parsed.max_acp_stdout_bytes, Some(8_388_608));
    }

    #[test]
    async fn channels_default() {
        let c = ChannelsConfig::default();
        assert!(c.cli);
        assert!(c.telegram.is_empty());
        assert!(c.discord.is_empty());
        assert!(c.wecom_ws.is_empty());
        assert!(!c.show_tool_calls);
        assert_eq!(
            c.max_concurrent_per_channel,
            default_channel_max_concurrent_per_channel()
        );
    }

    #[test]
    async fn channels_max_concurrent_per_channel_defaults_and_round_trips() {
        let parsed: ChannelsConfig = toml::from_str("cli = true").unwrap();
        assert_eq!(
            parsed.max_concurrent_per_channel,
            default_channel_max_concurrent_per_channel()
        );

        let parsed: ChannelsConfig =
            toml::from_str("cli = true\nmax_concurrent_per_channel = 2").unwrap();
        assert_eq!(parsed.max_concurrent_per_channel, 2);

        let toml_str = toml::to_string_pretty(&parsed).unwrap();
        let reparsed: ChannelsConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(reparsed.max_concurrent_per_channel, 2);
    }

    #[test]
    async fn validate_rejects_zero_channel_max_concurrent_per_channel() {
        let mut config = Config::default();
        config.channels.max_concurrent_per_channel = 0;

        let err = config
            .validate()
            .expect_err("zero channel concurrency budget must fail validate");
        assert!(
            err.to_string()
                .contains("channels.max_concurrent_per_channel must be greater than 0"),
            "got: {err}"
        );
    }

    #[test]
    async fn wecom_ws_config_serde_defaults_and_secret_metadata() {
        let toml = r#"
            enabled = true
            bot_id = "bot-123"
            secret = "sk-test"
            allowed_users = ["omnesagent_user"]
            allowed_groups = ["omnesagent_group"]
            bot_name = "danya"
            proxy_url = "http://127.0.0.1:7890"
        "#;
        let parsed: WeComWsConfig = toml::from_str(toml).unwrap();

        assert!(parsed.enabled);
        assert_eq!(parsed.bot_id, "bot-123");
        assert_eq!(parsed.secret, "sk-test");
        assert_eq!(parsed.allowed_users, vec!["omnesagent_user"]);
        assert_eq!(parsed.allowed_groups, vec!["omnesagent_group"]);
        assert_eq!(parsed.bot_name.as_deref(), Some("danya"));
        assert_eq!(parsed.file_retention_days, 7);
        assert_eq!(parsed.max_file_size_mb, 20);
        assert_eq!(parsed.stream_mode, StreamMode::Partial);
        assert_eq!(parsed.proxy_url.as_deref(), Some("http://127.0.0.1:7890"));
        assert!(parsed.excluded_tools.is_empty());
        assert_eq!(WeComWsConfig::default().file_retention_days, 7);
        assert_eq!(WeComWsConfig::default().max_file_size_mb, 20);
        assert_eq!(WeComWsConfig::default().stream_mode, StreamMode::Partial);
        assert!(WeComWsConfig::default().bot_name.is_none());
        assert!(WeComWsConfig::default().proxy_url.is_none());
        assert!(WeComWsConfig::prop_is_secret("channels.wecom_ws.secret"));
    }

    #[test]
    async fn config_parses_wecom_ws_separate_from_wecom_webhook() {
        let toml = r#"
            [channels.wecom.default]
            enabled = true
            webhook_key = "webhook-key"

            [channels.wecom_ws.default]
            enabled = true
            bot_id = "bot-123"
            secret = "sk-test"
            allowed_users = ["omnesagent_user"]
        "#;
        let parsed: Config = toml::from_str(toml).unwrap();

        assert_eq!(
            parsed.channels.wecom.get("default").unwrap().webhook_key,
            "webhook-key"
        );
        let ws = parsed.channels.wecom_ws.get("default").unwrap();
        assert_eq!(ws.bot_id, "bot-123");
        assert_eq!(ws.allowed_users, vec!["omnesagent_user"]);
        assert_eq!(ws.stream_mode, StreamMode::Partial);
    }

    // ── Serde round-trip ─────────────────────────────────────

    #[test]
    async fn config_toml_roundtrip() {
        let config = Config {
            eval: crate::scattered_types::EvalHarnessConfig::default(),
            degraded_security: Vec::new(),
            degraded_sections: Vec::new(),
            retired_wati_config_sections: Vec::new(),
            retired_node_transport_config: false,
            schema_version: crate::migration::CURRENT_SCHEMA_VERSION,
            providers: {
                let mut p = crate::providers::Providers::default();
                p.models.openrouter.insert(
                    "default".to_string(),
                    OpenRouterModelProviderConfig {
                        base: ModelProviderConfig {
                            api_key: Some("sk-test-key".into()),
                            model: Some("gpt-4o".into()),
                            temperature: Some(0.5),
                            timeout_secs: Some(120),
                            ..Default::default()
                        },
                    },
                );
                p
            },
            model_routes: Vec::new(),
            embedding_routes: Vec::new(),
            data_dir: PathBuf::from("/tmp/test/workspace"),
            config_path: PathBuf::from("/tmp/test/config.toml"),
            observability: ObservabilityConfig {
                backend: ObservabilityBackend::Log,
                ..ObservabilityConfig::default()
            },
            risk_profiles: {
                let mut m = HashMap::new();
                m.insert(
                    "default".into(),
                    RiskProfileConfig {
                        level: AutonomyLevel::Full,
                        workspace_only: false,
                        allowed_commands: vec!["docker".into()],
                        forbidden_paths: vec!["/secret".into()],
                        require_approval_for_medium_risk: false,
                        block_high_risk_commands: true,
                        shell_env_passthrough: vec!["DATABASE_URL".into()],
                        auto_approve: vec!["file_read".into()],
                        always_ask: vec![],
                        allowed_roots: vec![],
                        allowed_tools: vec![],
                        excluded_tools: vec![],
                        ..RiskProfileConfig::default()
                    },
                );
                m
            },
            trust: crate::scattered_types::TrustConfig::default(),
            backup: BackupConfig::default(),
            data_retention: DataRetentionConfig::default(),
            cloud_ops: CloudOpsConfig::default(),
            conversational_ai: ConversationalAiConfig::default(),
            security: SecurityConfig::default(),
            security_ops: SecurityOpsConfig::default(),
            runtime: RuntimeConfig {
                kind: RuntimeKind::Docker,
                ..RuntimeConfig::default()
            },
            reliability: ReliabilityConfig::default(),
            scheduler: SchedulerConfig::default(),
            skills: SkillsConfig::default(),
            pipeline: PipelineConfig::default(),
            query_classification: QueryClassificationConfig::default(),
            heartbeat: HeartbeatConfig {
                enabled: true,
                interval_minutes: 15,
                two_phase: true,
                message: Some("Check London time".into()),
                target: Some("telegram".into()),
                to: Some("123456".into()),
                ..HeartbeatConfig::default()
            },
            cron: HashMap::new(),
            acp: AcpConfig::default(),
            channels: ChannelsConfig {
                cli: true,
                telegram: HashMap::from([(
                    "default".to_string(),
                    TelegramConfig {
                        enabled: true,
                        bot_token: "123:ABC".into(),
                        api_base_url: default_telegram_api_base_url(),
                        stream_mode: StreamMode::default(),
                        draft_update_interval_ms: default_draft_update_interval_ms(),
                        debounce_ms: None,
                        interrupt_on_new_message: false,
                        mention_only: false,
                        ack_reactions: None,
                        proxy_url: None,
                        approval_timeout_secs: default_telegram_approval_timeout_secs(),
                        excluded_tools: vec![],
                        reply_min_interval_secs: 0,
                        reply_queue_depth_max: 0,
                    },
                )]),
                discord: HashMap::new(),
                slack: HashMap::new(),
                mattermost: HashMap::new(),
                webhook: HashMap::new(),
                imessage: HashMap::new(),
                matrix: HashMap::new(),
                signal: HashMap::new(),
                whatsapp: HashMap::new(),
                linq: HashMap::new(),
                nextcloud_talk: HashMap::new(),
                email: HashMap::new(),
                gmail_push: HashMap::new(),
                irc: HashMap::new(),
                twitch: HashMap::new(),
                lark: HashMap::new(),
                line: HashMap::new(),
                dingtalk: HashMap::new(),
                wecom: HashMap::new(),
                wecom_ws: HashMap::new(),
                wechat: HashMap::new(),
                qq: HashMap::new(),
                twitter: HashMap::new(),
                mochat: HashMap::new(),
                nostr: HashMap::new(),
                clawdtalk: HashMap::new(),
                reddit: HashMap::new(),
                bluesky: HashMap::new(),
                git: HashMap::new(),
                voice_call: HashMap::new(),
                voice_duplex: HashMap::new(),
                voice_wake: HashMap::new(),
                mqtt: HashMap::new(),
                amqp: HashMap::new(),
                filesystem: HashMap::new(),
                plugin: HashMap::new(),
                message_timeout_secs: 300,
                max_concurrent_per_channel: default_channel_max_concurrent_per_channel(),
                ack_reactions: true,
                show_tool_calls: true,
                session_persistence: true,
                session_backend: default_session_backend(),
                session_ttl_hours: 0,
                debounce_ms: 0,
            },
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
            pacing: PacingConfig::default(),
            cost: CostConfig::default(),
            peripherals: PeripheralsConfig::default(),
            delegate: DelegateToolConfig::default(),
            agents: HashMap::new(),
            runtime_profiles: HashMap::new(),
            skill_bundles: HashMap::new(),
            knowledge_bundles: HashMap::new(),
            mcp_bundles: HashMap::new(),
            peer_groups: HashMap::new(),
            hooks: HooksConfig::default(),
            hardware: HardwareConfig::default(),
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
            ralph: crate::ralph::RalphConfig::default(),
            ob2h_bridge: crate::ralph::Ob2hBridgeConfig::default(),
            voice: VoiceConfig::default(),
            env_overridden_paths: std::collections::HashSet::new(),
            pre_override_snapshots: std::collections::HashMap::new(),
            onepassword_reference_snapshots: std::collections::HashMap::new(),
            dirty_paths: std::collections::HashSet::new(),
        };
        // ModelProvider fields are now resolved directly — no cache needed.

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed = parse_test_config(&toml_str);

        assert_eq!(parsed.providers.models.len(), config.providers.models.len());
        assert_eq!(parsed.observability.backend, ObservabilityBackend::Log);
        assert_eq!(
            parsed.observability.log_persistence,
            LogPersistence::Rolling
        );
        let default_profile = parsed.risk_profiles.get("default").unwrap();
        assert_eq!(default_profile.level, AutonomyLevel::Full);
        assert!(!default_profile.workspace_only);
        assert_eq!(parsed.runtime.kind, RuntimeKind::Docker);
        assert!(parsed.heartbeat.enabled);
        assert_eq!(parsed.heartbeat.interval_minutes, 15);
        assert_eq!(
            parsed.heartbeat.message.as_deref(),
            Some("Check London time")
        );
        assert_eq!(parsed.heartbeat.target.as_deref(), Some("telegram"));
        assert_eq!(parsed.heartbeat.to.as_deref(), Some("123456"));
        assert!(!parsed.channels.telegram.is_empty());
        assert_eq!(
            parsed.channels.telegram.get("default").unwrap().bot_token,
            "123:ABC"
        );
    }

    #[test]
    async fn config_minimal_toml_uses_defaults() {
        let minimal = r#"
workspace_dir = "/tmp/ws"
config_path = "/tmp/config.toml"
default_temperature = 0.7
"#;
        let parsed = parse_test_config(minimal);
        assert!(
            parsed
                .providers
                .models
                .iter_entries()
                .next()
                .map(|(_, _, e)| e)
                .and_then(|e| e.api_key.as_deref())
                .is_none()
        );
        assert_eq!(parsed.observability.backend, ObservabilityBackend::None);
        assert_eq!(
            parsed.observability.log_persistence,
            LogPersistence::Rolling
        );
        // Migration synthesizes risk_profiles.default from the legacy
        // [autonomy] block; assert against the named entry rather than a
        // global "active" profile (no such concept exists).
        assert_eq!(
            parsed
                .risk_profiles
                .get("default")
                .expect("migration synthesized risk_profiles.default")
                .level,
            AutonomyLevel::Supervised
        );
        assert_eq!(parsed.runtime.kind, RuntimeKind::Native);
        // Heartbeat defaults to disabled.
        assert!(!parsed.heartbeat.enabled);
        assert!(parsed.channels.cli);
        assert!(parsed.memory.hygiene_enabled);
        assert_eq!(parsed.memory.archive_after_days, 7);
        assert_eq!(parsed.memory.purge_after_days, 30);
        assert_eq!(parsed.memory.conversation_retention_days, 30);
        // Temperature migrated onto the primary model_provider entry
        assert!(
            (parsed
                .providers
                .models
                .iter_entries()
                .next()
                .map(|(_, _, e)| e)
                .and_then(|e| e.temperature)
                .unwrap_or(0.7)
                - 0.7)
                .abs()
                < f64::EPSILON
        );
        assert_eq!(
            parsed
                .providers
                .models
                .iter_entries()
                .next()
                .map(|(_, _, e)| e)
                .and_then(|e| e.timeout_secs)
                .unwrap_or(120),
            DEFAULT_DELEGATE_TIMEOUT_SECS
        );
    }

    /// `[autonomy]` migrates onto `[risk_profiles.default]` via the V2→V3
    /// migration. The fields must round-trip without being silently dropped.
    #[test]
    async fn v2_autonomy_section_migrates_onto_risk_profiles_default() {
        let raw = r#"
schema_version = 2
default_temperature = 0.7

[autonomy]
level = "full"
max_actions_per_hour = 99
auto_approve = ["file_read", "memory_recall", "http_request"]
"#;
        let parsed = crate::migration::migrate_to_current(raw).unwrap();
        let profile = parsed
            .risk_profiles
            .get("default")
            .expect("default profile");
        assert_eq!(profile.level, AutonomyLevel::Full);
        assert!(profile.auto_approve.contains(&"http_request".to_string()));
        let runtime = parsed
            .runtime_profiles
            .get("default")
            .expect("default runtime profile");
        assert_eq!(runtime.max_actions_per_hour, 99);
    }

    /// Regression test for: when a user provides a custom auto_approve
    /// list, the built-in defaults must still be present.
    #[test]
    async fn auto_approve_merges_user_entries_with_defaults() {
        let raw = r#"
default_temperature = 0.7

[risk_profiles.default]
auto_approve = ["my_custom_tool", "another_tool"]
"#;
        let parsed = parse_test_config(raw);
        let profile = parsed.risk_profiles.get("default").unwrap();
        assert!(profile.auto_approve.contains(&"my_custom_tool".to_string()));
        assert!(profile.auto_approve.contains(&"another_tool".to_string()));
        for default_tool in &[
            "file_read",
            "memory_recall",
            "weather",
            "calculator",
            "web_fetch",
        ] {
            assert!(
                profile.auto_approve.contains(&String::from(*default_tool)),
                "default tool '{default_tool}' must be present"
            );
        }
    }

    #[test]
    async fn default_auto_approve_includes_tool_search() {
        let defaults = default_auto_approve();
        assert!(defaults.contains(&"tool_search".to_string()));
    }

    /// Regression test: empty auto_approve still gets defaults merged.
    #[test]
    async fn auto_approve_empty_list_gets_defaults() {
        let raw = r#"
default_temperature = 0.7

[risk_profiles.default]
auto_approve = []
"#;
        let parsed = parse_test_config(raw);
        let profile = parsed.risk_profiles.get("default").unwrap();
        for tool in &default_auto_approve() {
            assert!(
                profile.auto_approve.contains(tool),
                "default tool '{tool}' must be present"
            );
        }
    }

    /// When no risk_profiles section is provided, defaults are applied to the
    /// synthesized "default" profile.
    #[test]
    async fn auto_approve_defaults_when_no_risk_profile_section() {
        let raw = r#"
default_temperature = 0.7
"#;
        let parsed = parse_test_config(raw);
        let profile = parsed.risk_profiles.get("default").unwrap();
        for tool in &default_auto_approve() {
            assert!(
                profile.auto_approve.contains(tool),
                "default tool '{tool}' must be present"
            );
        }
    }

    /// Duplicates are not introduced when ensure_default_auto_approve runs
    /// on a list that already contains the defaults.
    #[test]
    async fn auto_approve_no_duplicates() {
        let raw = r#"
default_temperature = 0.7

[risk_profiles.default]
auto_approve = ["weather", "file_read"]
"#;
        let parsed = parse_test_config(raw);
        let profile = parsed.risk_profiles.get("default").unwrap();
        assert_eq!(
            profile
                .auto_approve
                .iter()
                .filter(|t| *t == "weather")
                .count(),
            1
        );
        assert_eq!(
            profile
                .auto_approve
                .iter()
                .filter(|t| *t == "file_read")
                .count(),
            1
        );
    }

    #[test]
    async fn provider_timeout_secs_parses_from_toml() {
        // V1 top-level `provider_timeout_secs` is folded into the
        // synthesized model_provider entry's `timeout_secs`.
        let raw = r#"
default_temperature = 0.7
provider_timeout_secs = 300
"#;
        let parsed = crate::migration::migrate_to_current(raw).expect("migration succeeds");
        assert_eq!(
            parsed
                .providers
                .models
                .find("openrouter", "default")
                .and_then(|e| e.timeout_secs)
                .unwrap_or(120),
            300
        );
    }

    #[test]
    async fn extra_headers_parses_from_toml() {
        // V1 top-level `[extra_headers]` is folded into the synthesized
        // default model_provider entry's `extra_headers` map.
        let raw = r#"
default_temperature = 0.7

[extra_headers]
User-Agent = "MyApp/1.0"
X-Title = "omnesagent"
"#;
        let parsed = crate::migration::migrate_to_current(raw).expect("migration succeeds");
        let headers = &parsed
            .providers
            .models
            .find("openrouter", "default")
            .expect("synthesized openrouter.default model_provider")
            .extra_headers;
        assert_eq!(headers.len(), 2);
        assert_eq!(headers.get("User-Agent").unwrap(), "MyApp/1.0");
        assert_eq!(headers.get("X-Title").unwrap(), "omnesagent");
    }

    #[test]
    async fn extra_headers_defaults_to_empty() {
        let raw = r#"
default_temperature = 0.7
"#;
        let parsed = parse_test_config(raw);
        assert!(
            parsed
                .providers
                .models
                .iter_entries()
                .next()
                .map(|(_, _, e)| e.extra_headers.is_empty())
                .unwrap_or(true)
        );
    }

    #[test]
    async fn storage_postgres_dburl_alias_deserializes() {
        let raw = r#"
default_temperature = 0.7

[storage.postgres.default]
dbURL = "postgres://user:pw@host/db"
schema = "public"
table = "memories"
connect_timeout_secs = 12
"#;

        let parsed = parse_test_config(raw);
        let pg = parsed
            .storage
            .postgres
            .get("default")
            .expect("postgres.default present");
        assert_eq!(pg.db_url.as_deref(), Some("postgres://user:pw@host/db"));
        assert_eq!(pg.schema, "public");
        assert_eq!(pg.table, "memories");
        assert_eq!(pg.connect_timeout_secs, Some(12));
    }

    #[test]
    async fn storage_lucid_timeout_overrides_deserialize() {
        let raw = r#"
default_temperature = 0.7

[storage.lucid.default]
binary_path = "/opt/lucid/bin/lucid"
recall_timeout_ms = 5000
store_timeout_ms = 4000
"#;

        let parsed = parse_test_config(raw);
        let lucid = parsed
            .storage
            .lucid
            .get("default")
            .expect("lucid.default present");
        assert_eq!(lucid.binary_path.as_deref(), Some("/opt/lucid/bin/lucid"));
        assert_eq!(lucid.recall_timeout_ms, Some(5000));
        assert_eq!(lucid.store_timeout_ms, Some(4000));
    }

    #[test]
    async fn validate_rejects_zero_lucid_timeouts_with_alias_qualified_paths() {
        for field in ["recall_timeout_ms", "store_timeout_ms"] {
            let raw = format!(
                r#"
default_temperature = 0.7

[storage.lucid.edge_arm]
{field} = 0
"#
            );
            let parsed = parse_test_config(&raw);
            let error = parsed
                .validate()
                .expect_err("zero Lucid timeout must fail validation");
            let expected_path = format!("storage.lucid.edge_arm.{field}");
            assert!(
                error.to_string().contains(&expected_path),
                "validation error must name {expected_path}: {error:#}"
            );
        }
    }

    #[test]
    async fn validate_rejects_blank_lucid_binary_with_alias_qualified_path() {
        let raw = r#"
default_temperature = 0.7

[storage.lucid.edge_arm]
binary_path = "   "
"#;
        let parsed = parse_test_config(raw);
        let error = parsed
            .validate()
            .expect_err("blank Lucid binary path must fail validation");
        assert!(
            error
                .to_string()
                .contains("storage.lucid.edge_arm.binary_path"),
            "validation error must name the alias-qualified binary path: {error:#}"
        );
    }

    #[test]
    async fn runtime_reasoning_enabled_deserializes() {
        let raw = r#"
default_temperature = 0.7

[runtime]
reasoning_enabled = false
"#;

        let parsed = parse_test_config(raw);
        assert_eq!(parsed.runtime.reasoning_enabled, Some(false));
    }

    #[test]
    async fn runtime_reasoning_effort_deserializes() {
        let raw = r#"
default_temperature = 0.7

[runtime]
reasoning_effort = "HIGH"
"#;

        let parsed: Config = toml::from_str(raw).unwrap();
        assert_eq!(parsed.runtime.reasoning_effort.as_deref(), Some("high"));
    }

    #[test]
    async fn runtime_reasoning_effort_rejects_invalid_values() {
        let raw = r#"
default_temperature = 0.7

[runtime]
reasoning_effort = "turbo"
"#;

        let error = toml::from_str::<Config>(raw).expect_err("invalid value should fail");
        assert!(error.to_string().contains("reasoning_effort"));
    }

    #[test]
    async fn agent_config_defaults() {
        let cfg = AliasedAgentConfig::default();
        assert!(cfg.resolved.compact_context);
        assert_eq!(cfg.resolved.max_tool_iterations, 10);
        assert_eq!(cfg.resolved.max_history_messages, 50);
        assert!(!cfg.resolved.parallel_tools);
        assert_eq!(cfg.resolved.tool_dispatcher, "auto");
        assert!(!cfg.resolved.strict_tool_parsing);
        assert!(cfg.precheck.enabled);
        assert_eq!(cfg.precheck.timeout_secs, 5);
    }

    #[test]
    async fn agent_precheck_config_parses_from_agent_block() {
        let raw = r#"
[agents.default]
model_provider = "custom.default"
risk_profile = "default"
runtime_profile = "default"

[agents.default.precheck]
enabled = false
timeout_secs = 12
"#;
        let parsed = parse_test_config(raw);
        let agent = parsed
            .agents
            .get("default")
            .expect("[agents.default] parses into agents map");
        assert!(!agent.precheck.enabled);
        assert_eq!(agent.precheck.timeout_secs, 12);
    }

    #[test]
    async fn validate_rejects_zero_agent_precheck_timeout() {
        let raw = r#"
[agents.default]
model_provider = "custom.default"
risk_profile = "default"
runtime_profile = "default"

[agents.default.precheck]
timeout_secs = 0
"#;
        let parsed = parse_test_config(raw);
        let error = parsed
            .validate()
            .expect_err("zero precheck timeout must be rejected");
        assert!(
            error
                .to_string()
                .contains("agents.default.precheck.timeout_secs")
        );
    }

    #[test]
    async fn agent_level_tunable_keys_are_inert() {
        let raw = r#"
default_temperature = 0.7
[agents.default]
compact_context = true
max_tool_iterations = 20
max_history_messages = 80
parallel_tools = true
tool_dispatcher = "xml"
strict_tool_parsing = true
"#;
        let parsed = parse_test_config(raw);
        let agent = parsed
            .agents
            .get("default")
            .expect("[agents.default] parses into agents map");
        assert_eq!(agent.resolved.max_tool_iterations, 10);
        assert_eq!(agent.resolved.tool_dispatcher, "auto");
        assert!(!agent.resolved.strict_tool_parsing);
    }

    #[test]
    async fn runtime_profile_max_tool_iterations_is_honored() {
        // `[runtime_profiles.*].max_tool_iterations` must actually take
        // effect. It previously had no effect (the value had to be set on
        // `[agents.*]`); now agent-inline is inert and the profile is the
        // authoritative surface, so this guards the resolved value.
        let raw = r#"
[runtime_profiles.fast]
max_tool_iterations = 25

[agents.default]
runtime_profile = "fast"
"#;
        let parsed = parse_test_config(raw);
        assert_eq!(parsed.effective_max_tool_iterations("default"), 25);
    }

    #[test]
    async fn runtime_profile_unset_max_tool_iterations_uses_default() {
        // A profile that does not set max_tool_iterations (sentinel 0) falls
        // back to the global default rather than 0.
        let raw = r#"
[runtime_profiles.fast]
max_history_messages = 80

[agents.default]
runtime_profile = "fast"
"#;
        let parsed = parse_test_config(raw);
        assert_eq!(parsed.effective_max_tool_iterations("default"), 10);
    }

    #[test]
    async fn runtime_profile_structured_history_cap_scales_when_omitted() {
        let raw = r#"
[runtime_profiles.long_turn]
max_tool_iterations = 100

[agents.default]
runtime_profile = "long_turn"
"#;
        let parsed = parse_test_config(raw);
        assert_eq!(parsed.effective_max_history_messages("default"), 50);
        assert_eq!(
            parsed.effective_structured_max_history_messages("default"),
            202
        );
        let agent = parsed.resolved_agent_config("default").unwrap();
        assert_eq!(agent.resolved.max_history_messages, 50);
    }

    #[test]
    async fn default_history_trim_low_water_is_seven_tenths() {
        let raw = r#"
[runtime_profiles.plain]

[agents.default]
runtime_profile = "plain"
"#;
        let parsed = parse_test_config(raw);
        assert_eq!(parsed.effective_history_trim_low_water("default"), 0.7);
        let agent = parsed.resolved_agent_config("default").unwrap();
        assert_eq!(agent.resolved.history_trim_low_water, 0.7);
    }

    #[test]
    async fn runtime_profile_history_trim_low_water_is_honored() {
        let raw = r#"
[runtime_profiles.mem_saver]
history_trim_low_water = 0.9

[agents.default]
runtime_profile = "mem_saver"
"#;
        let parsed = parse_test_config(raw);
        assert_eq!(parsed.effective_history_trim_low_water("default"), 0.9);
        let agent = parsed.resolved_agent_config("default").unwrap();
        assert_eq!(agent.resolved.history_trim_low_water, 0.9);
    }

    #[test]
    async fn validate_accepts_history_trim_low_water_of_one() {
        let raw = r#"
[runtime_profiles.no_hysteresis]
history_trim_low_water = 1.0
"#;
        let parsed = parse_test_config(raw);
        assert_eq!(
            parsed
                .runtime_profiles
                .get("no_hysteresis")
                .and_then(|p| p.history_trim_low_water),
            Some(1.0)
        );
        parsed
            .validate()
            .expect("history_trim_low_water = 1.0 must be accepted");
    }

    #[test]
    async fn validate_rejects_zero_history_trim_low_water() {
        let raw = r#"
[runtime_profiles.mem_saver]
history_trim_low_water = 0.0
"#;
        let parsed = parse_test_config(raw);
        let error = parsed
            .validate()
            .expect_err("history_trim_low_water = 0.0 must be rejected");
        assert!(
            error
                .to_string()
                .contains("runtime_profiles.mem_saver.history_trim_low_water")
        );
    }

    #[test]
    async fn validate_rejects_history_trim_low_water_above_one() {
        let raw = r#"
[runtime_profiles.mem_saver]
history_trim_low_water = 1.5
"#;
        let parsed = parse_test_config(raw);
        let error = parsed
            .validate()
            .expect_err("history_trim_low_water above 1.0 must be rejected");
        assert!(
            error
                .to_string()
                .contains("runtime_profiles.mem_saver.history_trim_low_water")
        );
    }

    #[test]
    async fn validate_rejects_non_finite_history_trim_low_water() {
        let raw = r#"
[runtime_profiles.mem_saver]
history_trim_low_water = nan
"#;
        let parsed = parse_test_config(raw);
        let error = parsed
            .validate()
            .expect_err("non-finite history_trim_low_water must be rejected");
        assert!(
            error
                .to_string()
                .contains("runtime_profiles.mem_saver.history_trim_low_water")
        );
    }

    #[test]
    async fn runtime_profile_history_cap_explicit_value_remains_authoritative() {
        let raw = r#"
[runtime_profiles.long_turn]
max_tool_iterations = 100
max_history_messages = 80

[agents.default]
runtime_profile = "long_turn"
"#;
        let parsed = parse_test_config(raw);
        assert_eq!(parsed.effective_max_history_messages("default"), 80);
        assert_eq!(
            parsed.effective_structured_max_history_messages("default"),
            80
        );
        let agent = parsed.resolved_agent_config("default").unwrap();
        assert_eq!(agent.resolved.max_history_messages, 80);
    }

    #[test]
    async fn runtime_profile_history_cap_explicit_zero_remains_authoritative() {
        let raw = r#"
[runtime_profiles.long_turn]
max_tool_iterations = 100
max_history_messages = 0

[agents.default]
runtime_profile = "long_turn"
"#;
        let parsed = parse_test_config(raw);
        assert_eq!(parsed.effective_max_history_messages("default"), 0);
        assert_eq!(
            parsed.effective_structured_max_history_messages("default"),
            0
        );
        let agent = parsed.resolved_agent_config("default").unwrap();
        assert_eq!(agent.resolved.max_history_messages, 0);
    }

    #[test]
    async fn runtime_profile_history_cap_saturates_at_usize_max() {
        let mut config = Config::default();
        config.runtime_profiles.insert(
            "long_turn".to_string(),
            RuntimeProfileConfig {
                max_tool_iterations: usize::MAX,
                ..RuntimeProfileConfig::default()
            },
        );
        config.agents.insert(
            "default".to_string(),
            AliasedAgentConfig {
                runtime_profile: "long_turn".into(),
                ..AliasedAgentConfig::default()
            },
        );

        assert_eq!(
            config.effective_structured_max_history_messages("default"),
            usize::MAX
        );
        assert_eq!(config.effective_max_history_messages("default"), 50);
    }

    #[test]
    async fn default_runtime_profile_history_cap_remains_50() {
        let parsed = parse_test_config("");
        assert_eq!(parsed.effective_max_tool_iterations("default"), 10);
        assert_eq!(parsed.effective_max_history_messages("default"), 50);
        assert_eq!(
            parsed.effective_structured_max_history_messages("default"),
            50
        );
    }

    #[test]
    async fn pacing_config_defaults_are_all_none_or_empty() {
        let cfg = PacingConfig::default();
        assert!(cfg.step_timeout_secs.is_none());
        assert!(cfg.loop_detection_min_elapsed_secs.is_none());
        assert!(cfg.loop_ignore_tools.is_empty());
        assert!(cfg.message_timeout_scale_max.is_none());
    }

    #[test]
    async fn pacing_config_deserializes_from_toml() {
        let raw = r#"
default_temperature = 0.7
[pacing]
step_timeout_secs = 120
loop_detection_min_elapsed_secs = 60
loop_ignore_tools = ["browser_screenshot", "browser_navigate"]
message_timeout_scale_max = 8
"#;
        let parsed: Config = toml::from_str(raw).unwrap();
        assert_eq!(parsed.pacing.step_timeout_secs, Some(120));
        assert_eq!(parsed.pacing.loop_detection_min_elapsed_secs, Some(60));
        assert_eq!(
            parsed.pacing.loop_ignore_tools,
            vec!["browser_screenshot", "browser_navigate"]
        );
        assert_eq!(parsed.pacing.message_timeout_scale_max, Some(8));
    }

    #[test]
    async fn pacing_config_absent_preserves_defaults() {
        let raw = r#"
default_temperature = 0.7
"#;
        let parsed: Config = toml::from_str(raw).unwrap();
        assert!(parsed.pacing.step_timeout_secs.is_none());
        assert!(parsed.pacing.loop_detection_min_elapsed_secs.is_none());
        assert!(parsed.pacing.loop_ignore_tools.is_empty());
        assert!(parsed.pacing.message_timeout_scale_max.is_none());
    }

    #[tokio::test]
    async fn sync_directory_handles_existing_directory() {
        let dir = std::env::temp_dir().join(format!(
            "omnesagent_test_sync_directory_{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).await.unwrap();

        sync_directory(&dir).await.unwrap();

        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn post_replace_sync_failure_keeps_disk_commit_and_backup() {
        let dir = std::env::temp_dir().join(format!(
            "omnesagent_test_post_replace_sync_failure_{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).await.unwrap();
        let config_path = dir.join("config.toml");
        let backup_path = dir.join("config.toml.bak");
        fs::write(&config_path, "schema_version = 1\n")
            .await
            .unwrap();

        let result = write_config_atomically_with_sync(
            &config_path,
            "schema_version = 2\n",
            PostReplaceSync::FailForTest,
        )
        .await;

        assert!(
            result.is_ok(),
            "a post-replace sync fault must report a committed save: {result:?}"
        );
        assert_eq!(
            fs::read_to_string(&config_path).await.unwrap(),
            "schema_version = 2\n",
            "the successful rename is the canonical on-disk result"
        );
        assert_eq!(
            fs::read_to_string(&backup_path).await.unwrap(),
            "schema_version = 1\n",
            "durability uncertainty must retain the pre-replace backup"
        );

        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn save_over_existing_config_syncs_backup_on_every_platform() {
        // The backup fsync runs on the ordinary save-over-existing-config
        // path, so the handle it uses must be valid at runtime, not merely
        // compile. Opening the `.bak` copy read-only builds everywhere but
        // fails under Windows `FlushFileBuffers`, which requires write
        // access. Executing the real path here turns that into a test
        // failure wherever the suite runs instead of a review-only catch.
        // The companion fault-injection test asserts the retained backup
        // still holds the pre-replace bytes, which is what proves the
        // widened access rights do not truncate the recovery copy.
        let dir = std::env::temp_dir().join(format!(
            "omnesagent_test_save_over_existing_{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).await.unwrap();
        let config_path = dir.join("config.toml");
        let backup_path = dir.join("config.toml.bak");
        fs::write(&config_path, "schema_version = 1\n")
            .await
            .unwrap();

        let result = write_config_atomically_with_sync(
            &config_path,
            "schema_version = 2\n",
            PostReplaceSync::Real,
        )
        .await;

        assert!(
            result.is_ok(),
            "saving over an existing config must not fail while syncing the backup: {result:?}"
        );
        assert_eq!(
            fs::read_to_string(&config_path).await.unwrap(),
            "schema_version = 2\n",
            "the replacement must be visible after a committed save"
        );
        assert!(
            !backup_path.exists(),
            "a fully durable save consumes the backup rather than leaving it behind"
        );

        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn config_save_prunes_unchanged_default_blocks() {
        // Fresh-init config without any operator edits should write a
        // tiny config.toml — only `schema_version` and any operator-
        // touched fields. The hundreds of all-default blocks
        // (LinkedIn, memory, observability, etc.) must not appear.
        let dir = std::env::temp_dir().join(format!(
            "omnesagent_save_prune_test_{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).await.unwrap();
        let config = Config {
            config_path: dir.join("config.toml"),
            data_dir: dir.join("data"),
            ..Default::default()
        };
        config.save().await.unwrap();
        let raw = fs::read_to_string(&config.config_path).await.unwrap();

        // schema_version must always survive (migration detector
        // anchor); without it a re-load would mis-detect as V1.
        assert!(
            raw.contains("schema_version"),
            "schema_version must survive pruning"
        );

        // Defaulted nested struct blocks must NOT appear in a fresh
        // save. Pick representative samples from across the schema:
        for block in [
            "[memory]",
            "[linkedin",
            "[observability]",
            "[gateway]",
            "[cost]",
        ] {
            assert!(
                !raw.contains(block),
                "pruned config.toml must not emit defaulted block {block}; got:\n{raw}",
            );
        }

        // Round-trip: load the pruned config and verify it still
        // deserializes to a `Config` (schema defaults fill the gaps).
        let _reloaded: Config = toml::from_str(&raw).expect("pruned config round-trips");

        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn config_save_keeps_operator_set_non_default_fields() {
        let dir = std::env::temp_dir().join(format!(
            "omnesagent_save_keep_test_{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).await.unwrap();
        let mut config = Config {
            config_path: dir.join("config.toml"),
            data_dir: dir.join("data"),
            ..Default::default()
        };
        // Operator picked a non-default locale + provider entry.
        config.locale = Some("ja-JP".into());
        config.providers.models.anthropic.insert(
            "claude_default".into(),
            AnthropicModelProviderConfig {
                base: ModelProviderConfig {
                    model: Some("claude-sonnet-4".into()),
                    ..Default::default()
                },
            },
        );
        config.save().await.unwrap();
        let raw = fs::read_to_string(&config.config_path).await.unwrap();

        assert!(
            raw.contains("ja-JP"),
            "operator-set locale must survive pruning; got:\n{raw}",
        );
        assert!(
            raw.contains("claude_default"),
            "operator-added provider alias must survive pruning; got:\n{raw}",
        );
        assert!(
            raw.contains("claude-sonnet-4"),
            "operator-set model must survive pruning; got:\n{raw}",
        );

        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn config_save_and_load_tmpdir() {
        let dir = std::env::temp_dir().join("omnesagent_test_config");
        let _ = fs::remove_dir_all(&dir).await;
        fs::create_dir_all(&dir).await.unwrap();

        let config_path = dir.join("config.toml");
        let mut providers = crate::providers::Providers::default();
        providers.models.openrouter.insert(
            "default".to_string(),
            OpenRouterModelProviderConfig {
                base: ModelProviderConfig {
                    api_key: Some("sk-roundtrip".into()),
                    model: Some("test-model".into()),
                    temperature: Some(0.9),
                    timeout_secs: Some(120),
                    ..Default::default()
                },
            },
        );
        let config = Config {
            eval: crate::scattered_types::EvalHarnessConfig::default(),
            degraded_security: Vec::new(),
            degraded_sections: Vec::new(),
            retired_wati_config_sections: Vec::new(),
            retired_node_transport_config: false,
            schema_version: crate::migration::CURRENT_SCHEMA_VERSION,
            providers,
            model_routes: Vec::new(),
            embedding_routes: Vec::new(),
            data_dir: dir.join("workspace"),
            config_path: config_path.clone(),
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
            skills: SkillsConfig::default(),
            pipeline: PipelineConfig::default(),
            query_classification: QueryClassificationConfig::default(),
            heartbeat: HeartbeatConfig::default(),
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
            pacing: PacingConfig::default(),
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
            ralph: crate::ralph::RalphConfig::default(),
            ob2h_bridge: crate::ralph::Ob2hBridgeConfig::default(),
            voice: VoiceConfig::default(),
            env_overridden_paths: std::collections::HashSet::new(),
            pre_override_snapshots: std::collections::HashMap::new(),
            onepassword_reference_snapshots: std::collections::HashMap::new(),
            dirty_paths: std::collections::HashSet::new(),
        };

        // ModelProvider fields are now resolved directly — no cache needed.
        config.save().await.unwrap();
        assert!(config_path.exists());

        let contents = tokio::fs::read_to_string(&config_path).await.unwrap();
        let loaded = crate::migration::migrate_to_current(&contents).unwrap();
        let entry = &loaded
            .providers
            .models
            .find("openrouter", "default")
            .expect("entry exists");
        assert!(
            entry
                .api_key
                .as_deref()
                .is_some_and(crate::secrets::SecretStore::is_encrypted)
        );
        let store = crate::secrets::SecretStore::new(&dir, true);
        let decrypted = store.decrypt(entry.api_key.as_deref().unwrap()).unwrap();
        assert_eq!(decrypted, "sk-roundtrip");
        assert_eq!(entry.model.as_deref(), Some("test-model"));
        assert!(
            entry
                .temperature
                .is_some_and(|t| (t - 0.9).abs() < f64::EPSILON)
        );

        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn config_save_encrypts_nested_credentials() {
        let dir = std::env::temp_dir().join(format!(
            "omnesagent_test_nested_credentials_{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).await.unwrap();

        let mut config = Config {
            data_dir: dir.join("workspace"),
            config_path: dir.join("config.toml"),
            ..Default::default()
        };
        config.providers.models.anthropic.insert(
            "default".to_string(),
            AnthropicModelProviderConfig {
                base: ModelProviderConfig {
                    api_key: Some("root-credential".into()),
                    extra_headers: HashMap::from([(
                        "Authorization".to_string(),
                        "Bearer provider-header-credential".to_string(),
                    )]),
                    ..Default::default()
                },
            },
        );
        // ModelProvider fields are now resolved directly — no cache needed.
        config.composio.api_key = Some("composio-credential".into());
        config.browser.computer_use.api_key = Some("browser-credential".into());
        config.web_search.brave_api_key = Some("brave-credential".into());
        config.web_search.tavily_api_key = Some("tavily-credential".into());
        config.storage.postgres.insert(
            "default".to_string(),
            PostgresStorageConfig {
                db_url: Some("postgres://user:pw@host/db".into()),
                ..PostgresStorageConfig::default()
            },
        );
        config.storage.qdrant.insert(
            "default".to_string(),
            QdrantStorageConfig {
                api_key: Some("qdrant-credential".into()),
                ..QdrantStorageConfig::default()
            },
        );
        config.reliability.api_keys = vec![
            "rotation-credential-a".into(),
            "rotation-credential-b".into(),
        ];
        config.nodes.auth_token = Some("nodes-auth-credential".into());
        config.observability.backend = ObservabilityBackend::Otel;
        config.observability.otel_headers = Some(HashMap::from([(
            "Authorization".to_string(),
            "Bearer otel-credential".to_string(),
        )]));
        config.file_upload.headers = HashMap::from([(
            "Authorization".to_string(),
            "Bearer upload-credential".to_string(),
        )]);
        config.http_request.secrets = HashMap::from([(
            "api_token".to_string(),
            "Bearer http-request-credential".to_string(),
        )]);
        config.channels.lark.insert(
            "feishu".to_string(),
            LarkConfig {
                enabled: true,
                app_id: "cli_feishu_123".into(),
                app_secret: "feishu-secret".into(),
                encrypt_key: Some("feishu-encrypt".into()),
                verification_token: Some("feishu-verify".into()),
                mention_only: false,
                use_feishu: true,
                receive_mode: LarkReceiveMode::Websocket,
                port: None,
                proxy_url: None,
                excluded_tools: vec![],
                approval_timeout_secs: 300,
                per_user_session: false,
                ack_reactions: None,
                stream_mode: StreamMode::default(),
                draft_update_interval_ms: default_draft_update_interval_ms(),
            },
        );

        config.providers.models.openrouter.insert(
            "worker".into(),
            crate::schema::OpenRouterModelProviderConfig {
                base: ModelProviderConfig {
                    api_key: Some("agent-credential".into()),
                    model: Some("model-test".into()),
                    ..Default::default()
                },
            },
        );
        config.agents.insert(
            "worker".into(),
            AliasedAgentConfig {
                model_provider: "openrouter.worker".into(),
                ..Default::default()
            },
        );

        // Webhook channel: auth_header carries a Bearer token; must be
        // encrypted alongside the existing webhook `secret` field.
        config.channels.webhook.insert(
            "primary".into(),
            WebhookConfig {
                enabled: true,
                port: 8080,
                auth_header: Some("Bearer webhook-cred".into()),
                secret: Some("webhook-shared-secret".into()),
                ..Default::default()
            },
        );
        config.gateway.webhook_secret = Some("gateway-ingress-credential".into());

        // MCP server: HTTP headers map carries an Authorization Bearer
        // token; the new `#[secret]` on `HashMap<String, String>` must
        // encrypt every value (and only every value — keys stay plain).
        config.mcp.servers.push(McpServerConfig {
            name: "primary".into(),
            transport: McpTransport::Sse,
            url: Some("https://mcp.example.invalid/sse".into()),
            env: HashMap::from([("MCP_API_KEY".to_string(), "mcp-env-credential".to_string())]),
            headers: HashMap::from([
                ("Authorization".to_string(), "Bearer mcp-cred".to_string()),
                ("X-Tenant".to_string(), "tenant-42".to_string()),
            ]),
            ..Default::default()
        });

        config.save().await.unwrap();

        let contents = tokio::fs::read_to_string(config.config_path.clone())
            .await
            .unwrap();
        for plaintext in [
            "root-credential",
            "Bearer provider-header-credential",
            "composio-credential",
            "browser-credential",
            "brave-credential",
            "tavily-credential",
            "postgres://user:pw@host/db",
            "qdrant-credential",
            "rotation-credential-a",
            "rotation-credential-b",
            "node-shared-credential",
            "nodes-auth-credential",
            "Bearer otel-credential",
            "Bearer upload-credential",
            "Bearer http-request-credential",
            "gateway-ingress-credential",
            "mcp-env-credential",
            "Bearer mcp-cred",
            "tenant-42",
        ] {
            assert!(
                !contents.contains(plaintext),
                "saved TOML must not contain plaintext credential `{plaintext}`"
            );
        }
        let stored: Config = crate::migration::migrate_to_current(&contents).unwrap();
        let store = crate::secrets::SecretStore::new(&dir, true);

        let root_encrypted = stored
            .providers
            .models
            .find("anthropic", "default")
            .and_then(|e| e.api_key.as_deref())
            .unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(root_encrypted));
        assert_eq!(store.decrypt(root_encrypted).unwrap(), "root-credential");

        let provider_header = stored
            .providers
            .models
            .find("anthropic", "default")
            .and_then(|e| e.extra_headers.get("Authorization"))
            .unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(provider_header));
        assert_eq!(
            store.decrypt(provider_header).unwrap(),
            "Bearer provider-header-credential"
        );

        let composio_encrypted = stored.composio.api_key.as_deref().unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(
            composio_encrypted
        ));
        assert_eq!(
            store.decrypt(composio_encrypted).unwrap(),
            "composio-credential"
        );

        let browser_encrypted = stored.browser.computer_use.api_key.as_deref().unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(browser_encrypted));
        assert_eq!(
            store.decrypt(browser_encrypted).unwrap(),
            "browser-credential"
        );

        let web_search_encrypted = stored.web_search.brave_api_key.as_deref().unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(
            web_search_encrypted
        ));
        assert_eq!(
            store.decrypt(web_search_encrypted).unwrap(),
            "brave-credential"
        );

        let tavily_encrypted = stored.web_search.tavily_api_key.as_deref().unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(tavily_encrypted));
        assert_eq!(
            store.decrypt(tavily_encrypted).unwrap(),
            "tavily-credential"
        );

        let worker_provider = stored
            .providers
            .models
            .find("openrouter", "worker")
            .unwrap();
        let worker_encrypted = worker_provider.api_key.as_deref().unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(worker_encrypted));
        assert_eq!(store.decrypt(worker_encrypted).unwrap(), "agent-credential");

        let storage_db_url = stored
            .storage
            .postgres
            .get("default")
            .and_then(|p| p.db_url.as_deref())
            .unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(storage_db_url));
        assert_eq!(
            store.decrypt(storage_db_url).unwrap(),
            "postgres://user:pw@host/db"
        );

        let qdrant_key = stored
            .storage
            .qdrant
            .get("default")
            .and_then(|q| q.api_key.as_deref())
            .unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(qdrant_key));
        assert_eq!(store.decrypt(qdrant_key).unwrap(), "qdrant-credential");

        for key in &stored.reliability.api_keys {
            assert!(crate::secrets::SecretStore::is_encrypted(key));
        }
        assert_eq!(
            store.decrypt(&stored.reliability.api_keys[0]).unwrap(),
            "rotation-credential-a"
        );
        assert_eq!(
            store.decrypt(&stored.reliability.api_keys[1]).unwrap(),
            "rotation-credential-b"
        );

        let nodes_auth = stored.nodes.auth_token.as_deref().unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(nodes_auth));
        assert_eq!(store.decrypt(nodes_auth).unwrap(), "nodes-auth-credential");

        let otel_auth = stored
            .observability
            .otel_headers
            .as_ref()
            .and_then(|h| h.get("Authorization"))
            .unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(otel_auth));
        assert_eq!(store.decrypt(otel_auth).unwrap(), "Bearer otel-credential");

        let upload_auth = stored.file_upload.headers.get("Authorization").unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(upload_auth));
        assert_eq!(
            store.decrypt(upload_auth).unwrap(),
            "Bearer upload-credential"
        );

        let http_request_auth = stored.http_request.secrets.get("api_token").unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(http_request_auth));
        assert_eq!(
            store.decrypt(http_request_auth).unwrap(),
            "Bearer http-request-credential"
        );

        let feishu = stored.channels.lark.get("feishu").unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(
            &feishu.app_secret
        ));
        assert_eq!(store.decrypt(&feishu.app_secret).unwrap(), "feishu-secret");
        assert!(
            feishu
                .encrypt_key
                .as_deref()
                .is_some_and(crate::secrets::SecretStore::is_encrypted)
        );
        assert_eq!(
            store
                .decrypt(feishu.encrypt_key.as_deref().unwrap())
                .unwrap(),
            "feishu-encrypt"
        );
        assert!(
            feishu
                .verification_token
                .as_deref()
                .is_some_and(crate::secrets::SecretStore::is_encrypted)
        );
        assert_eq!(
            store
                .decrypt(feishu.verification_token.as_deref().unwrap())
                .unwrap(),
            "feishu-verify"
        );

        // Webhook auth_header — newly tagged `#[secret]`.
        let webhook = stored.channels.webhook.get("primary").unwrap();
        let webhook_auth = webhook.auth_header.as_deref().unwrap();
        assert!(
            crate::secrets::SecretStore::is_encrypted(webhook_auth),
            "webhook auth_header must be encrypted on save"
        );
        assert_eq!(store.decrypt(webhook_auth).unwrap(), "Bearer webhook-cred");
        // The pre-existing webhook `secret` field stays encrypted too —
        // sanity check that the refactor didn't regress it.
        let webhook_secret = webhook.secret.as_deref().unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(webhook_secret));
        assert_eq!(
            store.decrypt(webhook_secret).unwrap(),
            "webhook-shared-secret"
        );
        let gateway_webhook_secret = stored.gateway.webhook_secret.as_deref().unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(
            gateway_webhook_secret
        ));
        assert_eq!(
            store.decrypt(gateway_webhook_secret).unwrap(),
            "gateway-ingress-credential"
        );

        // MCP server headers — every value must be encrypted; the keys
        // stay plaintext (TOML table headers are not secret).
        let mcp_server = stored
            .mcp
            .servers
            .iter()
            .find(|s| s.name == "primary")
            .expect("mcp server `primary` round-trips through save");
        for (key, value) in &mcp_server.headers {
            assert!(
                crate::secrets::SecretStore::is_encrypted(value),
                "mcp.servers.primary.headers.{key} must be encrypted on save"
            );
        }
        let mcp_env = mcp_server.env.get("MCP_API_KEY").unwrap();
        assert!(
            crate::secrets::SecretStore::is_encrypted(mcp_env),
            "mcp.servers.primary.env.MCP_API_KEY must be encrypted on save"
        );
        let auth = mcp_server.headers.get("Authorization").unwrap();
        let tenant = mcp_server.headers.get("X-Tenant").unwrap();
        assert_eq!(store.decrypt(mcp_env).unwrap(), "mcp-env-credential");
        assert_eq!(store.decrypt(auth).unwrap(), "Bearer mcp-cred");
        assert_eq!(store.decrypt(tenant).unwrap(), "tenant-42");

        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn config_save_atomic_cleanup() {
        let dir =
            std::env::temp_dir().join(format!("omnesagent_test_config_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).await.unwrap();

        let config_path = dir.join("config.toml");
        let mut config = Config {
            data_dir: dir.join("workspace"),
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.providers.models.openrouter.insert(
            "default".to_string(),
            OpenRouterModelProviderConfig {
                base: ModelProviderConfig {
                    model: Some("model-a".into()),
                    ..Default::default()
                },
            },
        );
        config.save().await.unwrap();
        assert!(config_path.exists());

        config
            .providers
            .models
            .ensure("openrouter", "default")
            .unwrap()
            .model = Some("model-b".into());
        config.save().await.unwrap();

        let contents = tokio::fs::read_to_string(&config_path).await.unwrap();
        assert!(contents.contains("model-b"));

        let mut names: Vec<String> = Vec::new();
        let mut read_dir = fs::read_dir(&dir).await.unwrap();
        while let Some(entry) = read_dir.next_entry().await.unwrap() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
        assert!(!names.iter().any(|name| name.contains(".tmp-")));
        assert!(!names.iter().any(|name| name.ends_with(".bak")));

        let _ = fs::remove_dir_all(&dir).await;
    }

    // ── Telegram / Discord config ────────────────────────────

    #[test]
    async fn telegram_config_serde() {
        let tc = TelegramConfig {
            enabled: true,
            bot_token: "123:XYZ".into(),
            api_base_url: default_telegram_api_base_url(),
            stream_mode: StreamMode::Partial,
            draft_update_interval_ms: 500,
            interrupt_on_new_message: true,
            mention_only: false,
            ack_reactions: None,
            proxy_url: None,
            approval_timeout_secs: 120,
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
            debounce_ms: None,
        };
        let json = serde_json::to_string(&tc).unwrap();
        let parsed: TelegramConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.bot_token, "123:XYZ");
        assert_eq!(parsed.stream_mode, StreamMode::Partial);
        assert_eq!(parsed.draft_update_interval_ms, 500);
        assert!(parsed.interrupt_on_new_message);
    }

    #[test]
    async fn telegram_config_defaults_stream_off() {
        let json = r#"{"bot_token":"tok","allowed_users":[]}"#;
        let parsed: TelegramConfig = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.stream_mode, StreamMode::Off);
        assert_eq!(parsed.draft_update_interval_ms, 1000);
        assert!(!parsed.interrupt_on_new_message);
        assert_eq!(parsed.api_base_url, "https://api.telegram.org");
    }

    #[test]
    async fn telegram_config_rejects_matrix_single_message_stream_mode() {
        let err = toml::from_str::<TelegramConfig>(
            r#"
bot_token = "tok"
stream_mode = "single_message"
"#,
        )
        .unwrap_err();

        assert!(err.to_string().contains("single_message"));
    }

    #[test]
    async fn discord_config_serde() {
        let dc = DiscordConfig {
            enabled: true,
            bot_token: "discord-token".into(),
            guild_ids: vec!["12345".into()],
            channel_ids: vec![],
            archive: false,
            listen_to_bots: false,
            interrupt_on_new_message: false,
            mention_only: false,
            slash_commands: false,
            slash_command_scope: SlashCommandScope::default(),
            proxy_url: None,
            stream_mode: StreamMode::default(),
            draft_update_interval_ms: 1000,
            multi_message_delay_ms: 800,
            stall_timeout_secs: 0,
            intents_mask: None,
            reaction_notifications: DiscordReactionScope::Off,
            approval_timeout_secs: 300,
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };
        let json = serde_json::to_string(&dc).unwrap();
        let parsed: DiscordConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.bot_token, "discord-token");
        assert_eq!(parsed.guild_ids, vec!["12345".to_string()]);
    }

    #[test]
    async fn discord_config_empty_guild_ids() {
        let dc = DiscordConfig {
            enabled: true,
            bot_token: "tok".into(),
            guild_ids: Vec::new(),
            channel_ids: vec![],
            archive: false,
            listen_to_bots: false,
            interrupt_on_new_message: false,
            mention_only: false,
            slash_commands: false,
            slash_command_scope: SlashCommandScope::default(),
            proxy_url: None,
            stream_mode: StreamMode::default(),
            draft_update_interval_ms: 1000,
            multi_message_delay_ms: 800,
            stall_timeout_secs: 0,
            intents_mask: None,
            reaction_notifications: DiscordReactionScope::Off,
            approval_timeout_secs: 300,
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };
        let json = serde_json::to_string(&dc).unwrap();
        let parsed: DiscordConfig = serde_json::from_str(&json).unwrap();
        assert!(parsed.guild_ids.is_empty());
    }

    // ── iMessage / Matrix config ────────────────────────────

    // iMessage `allowed_contacts` was lifted out of `IMessageConfig` in V3;
    // inbound peer authorization lives in `Config::peer_groups`. The
    // round-trip of contact-list values from a V2 TOML is exercised by
    // `imessage_v2_allowed_contacts_fold_into_peer_groups` below; per-field
    // struct serde for `allowed_contacts` no longer applies.

    #[test]
    async fn imessage_v2_allowed_contacts_fold_into_peer_groups() {
        // V2 TOML with `allowed_contacts` on the channel must be folded
        // into a synthesized `peer_groups.imessage_default` group with
        // each contact as an external peer.
        let raw = r#"
schema_version = 2

[channels.imessage]
enabled = true
allowed_contacts = ["+1234567890", "user@icloud.com"]
"#;
        let parsed = crate::migration::migrate_to_current(raw).expect("migration succeeds");
        let group = parsed
            .peer_groups
            .get("imessage_default")
            .expect("V2 imessage.allowed_contacts must fold into peer_groups.imessage_default");
        assert_eq!(group.channel, "imessage");
        let usernames: Vec<&str> = group.external_peers.iter().map(|p| p.as_str()).collect();
        assert_eq!(usernames, vec!["+1234567890", "user@icloud.com"]);
    }

    #[test]
    async fn matrix_config_serde() {
        let mc = MatrixConfig {
            enabled: true,
            homeserver: "https://matrix.org".into(),
            access_token: Some("syt_token_abc".into()),
            user_id: Some("@bot:matrix.org".into()),
            device_id: Some("DEVICE123".into()),
            allowed_rooms: vec!["!room123:matrix.org".into()],
            interrupt_on_new_message: false,
            stream_mode: MatrixStreamMode::default(),
            draft_update_interval_ms: 1500,
            multi_message_delay_ms: 800,
            stream_draft_lines: 10,
            message_max_bytes: 48_000,
            stream_draft_delete: true,
            stream_reasoning: StreamReasoningMode::Status,
            stream_tool_arguments: vec![],
            recovery_key: None,
            mention_only: false,
            password: None,
            approval_timeout_secs: 300,
            reply_in_thread: true,
            ack_reactions: Some(true),
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };
        let json = serde_json::to_string(&mc).unwrap();
        let parsed: MatrixConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.homeserver, "https://matrix.org");
        assert_eq!(parsed.access_token.as_deref(), Some("syt_token_abc"));
        assert_eq!(parsed.user_id.as_deref(), Some("@bot:matrix.org"));
        assert_eq!(parsed.device_id.as_deref(), Some("DEVICE123"));
        assert_eq!(
            parsed.allowed_rooms.first().map(|s| s.as_str()),
            Some("!room123:matrix.org")
        );
    }

    #[test]
    async fn matrix_config_toml_roundtrip() {
        let mc = MatrixConfig {
            enabled: true,
            homeserver: "https://synapse.local:8448".into(),
            access_token: Some("tok".into()),
            user_id: None,
            device_id: None,
            allowed_rooms: vec!["!abc:synapse.local".into()],
            interrupt_on_new_message: false,
            stream_mode: MatrixStreamMode::default(),
            draft_update_interval_ms: 1500,
            multi_message_delay_ms: 800,
            stream_draft_lines: 10,
            message_max_bytes: 48_000,
            stream_draft_delete: true,
            stream_reasoning: StreamReasoningMode::Status,
            stream_tool_arguments: vec![],
            recovery_key: None,
            mention_only: false,
            password: None,
            approval_timeout_secs: 300,
            reply_in_thread: true,
            ack_reactions: Some(true),
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };
        let toml_str = toml::to_string(&mc).unwrap();
        let parsed: MatrixConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.homeserver, "https://synapse.local:8448");
        assert_eq!(parsed.allowed_rooms.len(), 1);
    }

    #[test]
    async fn slack_thread_context_max_messages_defaults_to_zero() {
        let parsed: SlackConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(parsed.thread_context_max_messages, None);
        assert_eq!(parsed.effective_thread_context_max_messages(), 0);
    }

    #[test]
    async fn slack_thread_context_max_messages_deserializes_explicit_value() {
        let parsed: SlackConfig =
            serde_json::from_str(r#"{"thread_context_max_messages":7}"#).unwrap();
        assert_eq!(parsed.thread_context_max_messages, Some(7));
        assert_eq!(parsed.effective_thread_context_max_messages(), 7);
    }

    #[test]
    async fn slack_thread_context_max_messages_allows_zero_to_disable_backfill() {
        let parsed: SlackConfig =
            serde_json::from_str(r#"{"thread_context_max_messages":0}"#).unwrap();
        assert_eq!(parsed.thread_context_max_messages, Some(0));
        assert_eq!(parsed.effective_thread_context_max_messages(), 0);
    }

    #[test]
    async fn slack_thread_context_max_messages_rejects_values_above_slack_fetch_limit() {
        let mut config = Config::default();
        config.channels.slack.insert(
            "default".to_string(),
            SlackConfig {
                thread_context_max_messages: Some(51),
                ..Default::default()
            },
        );

        let err = config
            .validate()
            .expect_err("values above the Slack fetch limit must be rejected")
            .to_string();
        assert!(
            err.contains("channels.slack.default.thread_context_max_messages"),
            "unexpected validation error: {err}",
        );
    }

    #[test]
    async fn matrix_config_backward_compatible_without_session_hints() {
        // room_id in TOML is now migrated by prepare_table at the top level;
        // a bare MatrixConfig parse just ignores unknown keys.
        let toml = r#"
homeserver = "https://matrix.org"
access_token = "tok"
allowed_users = ["@ops:matrix.org"]
allowed_rooms = ["!ops:matrix.org"]
"#;

        let parsed: MatrixConfig = toml::from_str(toml).unwrap();
        assert_eq!(parsed.homeserver, "https://matrix.org");
        assert!(parsed.user_id.is_none());
        assert!(parsed.device_id.is_none());
        assert_eq!(parsed.allowed_rooms, vec!["!ops:matrix.org"]);
    }

    #[test]
    async fn matrix_config_reply_in_thread_defaults_to_true() {
        let toml = r#"
homeserver = "https://matrix.org"
access_token = "tok"
allowed_users = ["@u:matrix.org"]
"#;
        let parsed: MatrixConfig = toml::from_str(toml).unwrap();
        assert!(parsed.reply_in_thread);
    }

    #[test]
    async fn signal_config_serde() {
        let sc = SignalConfig {
            enabled: true,
            http_url: "http://127.0.0.1:8686".into(),
            account: "+1234567890".into(),
            group_ids: vec!["group123".into()],
            dm_only: false,
            ignore_attachments: true,
            ignore_stories: false,
            proxy_url: None,
            approval_timeout_secs: 300,
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };
        let json = serde_json::to_string(&sc).unwrap();
        let parsed: SignalConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.http_url, "http://127.0.0.1:8686");
        assert_eq!(parsed.account, "+1234567890");
        assert_eq!(parsed.group_ids, vec!["group123".to_string()]);
        assert!(!parsed.dm_only);
        assert!(parsed.ignore_attachments);
        assert!(!parsed.ignore_stories);
    }

    #[test]
    async fn signal_config_toml_roundtrip() {
        let sc = SignalConfig {
            enabled: true,
            http_url: "http://localhost:8080".into(),
            account: "+9876543210".into(),
            group_ids: Vec::new(),
            dm_only: true,
            ignore_attachments: false,
            ignore_stories: true,
            proxy_url: None,
            approval_timeout_secs: 300,
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };
        let toml_str = toml::to_string(&sc).unwrap();
        let parsed: SignalConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.http_url, "http://localhost:8080");
        assert_eq!(parsed.account, "+9876543210");
        assert!(parsed.group_ids.is_empty());
        assert!(parsed.dm_only);
        assert!(parsed.ignore_stories);
    }

    #[test]
    async fn signal_config_defaults() {
        let json = r#"{"http_url":"http://127.0.0.1:8686","account":"+1234567890"}"#;
        let parsed: SignalConfig = serde_json::from_str(json).unwrap();
        assert!(parsed.group_ids.is_empty());
        assert!(!parsed.dm_only);
        assert!(!parsed.ignore_attachments);
        assert!(!parsed.ignore_stories);
    }

    #[test]
    async fn channels_with_imessage_and_matrix() {
        let c = ChannelsConfig {
            cli: true,
            telegram: HashMap::new(),
            discord: HashMap::new(),
            slack: HashMap::new(),
            mattermost: HashMap::new(),
            webhook: HashMap::new(),
            imessage: HashMap::from([(
                "default".to_string(),
                IMessageConfig {
                    enabled: true,
                    excluded_tools: vec![],
                    reply_min_interval_secs: 0,
                    reply_queue_depth_max: 0,
                },
            )]),
            matrix: HashMap::from([(
                "default".to_string(),
                MatrixConfig {
                    enabled: true,
                    homeserver: "https://m.org".into(),
                    access_token: Some("tok".into()),
                    user_id: None,
                    device_id: None,
                    allowed_rooms: vec!["!r:m".into()],
                    interrupt_on_new_message: false,
                    stream_mode: MatrixStreamMode::default(),
                    draft_update_interval_ms: 1500,
                    multi_message_delay_ms: 800,
                    stream_draft_lines: 10,
                    message_max_bytes: 48_000,
                    stream_draft_delete: true,
                    stream_reasoning: StreamReasoningMode::Status,
                    stream_tool_arguments: vec![],
                    recovery_key: None,
                    mention_only: false,
                    password: None,
                    approval_timeout_secs: 300,
                    reply_in_thread: true,
                    ack_reactions: Some(true),
                    excluded_tools: vec![],
                    reply_min_interval_secs: 0,
                    reply_queue_depth_max: 0,
                },
            )]),
            signal: HashMap::new(),
            whatsapp: HashMap::new(),
            linq: HashMap::new(),
            nextcloud_talk: HashMap::new(),
            email: HashMap::new(),
            gmail_push: HashMap::new(),
            irc: HashMap::new(),
            twitch: HashMap::new(),
            lark: HashMap::new(),
            line: HashMap::new(),
            dingtalk: HashMap::new(),
            wecom: HashMap::new(),
            wecom_ws: HashMap::new(),
            wechat: HashMap::new(),
            qq: HashMap::new(),
            twitter: HashMap::new(),
            mochat: HashMap::new(),
            nostr: HashMap::new(),
            clawdtalk: HashMap::new(),
            reddit: HashMap::new(),
            bluesky: HashMap::new(),
            git: HashMap::new(),
            voice_call: HashMap::new(),
            voice_duplex: HashMap::new(),
            voice_wake: HashMap::new(),
            mqtt: HashMap::new(),
            amqp: HashMap::new(),
            filesystem: HashMap::new(),
            plugin: HashMap::new(),
            message_timeout_secs: 300,
            max_concurrent_per_channel: default_channel_max_concurrent_per_channel(),
            ack_reactions: true,
            show_tool_calls: true,
            session_persistence: true,
            session_backend: default_session_backend(),
            session_ttl_hours: 0,
            debounce_ms: 0,
        };
        let toml_str = toml::to_string_pretty(&c).unwrap();
        let parsed: ChannelsConfig = toml::from_str(&toml_str).unwrap();
        assert!(!parsed.imessage.is_empty());
        assert!(!parsed.matrix.is_empty());
        assert_eq!(
            parsed.matrix.get("default").unwrap().homeserver,
            "https://m.org"
        );
    }

    #[test]
    async fn channels_default_has_no_imessage_matrix() {
        let c = ChannelsConfig::default();
        assert!(c.imessage.is_empty());
        assert!(c.matrix.is_empty());
    }

    // ── Edge cases: serde(default) for non-secret optional fields ─────
    // The legacy `allowed_users` field is no longer carried on channel
    // configs (V3 moved inbound peer authorization into
    // `Config::peer_groups`); V2 TOMLs with `allowed_users` are folded
    // by `migrate_to_current` into `[peer_groups.<type>_<alias>]`. See
    // `discord_v2_allowed_users_fold_into_peer_groups` below.

    #[test]
    async fn discord_v2_allowed_users_fold_into_peer_groups() {
        let raw = r#"
schema_version = 2

[channels.discord]
enabled = true
bot_token = "tok"
guild_id = "123"
allowed_users = ["111", "222"]
"#;
        let parsed = crate::migration::migrate_to_current(raw).expect("migration succeeds");
        let group = parsed
            .peer_groups
            .get("discord_default")
            .expect("V2 discord.allowed_users must fold into peer_groups.discord_default");
        assert_eq!(group.channel, "discord");
        let usernames: Vec<&str> = group.external_peers.iter().map(|p| p.as_str()).collect();
        assert_eq!(usernames, vec!["111", "222"]);
    }

    #[test]
    async fn slack_v2_allowed_users_fold_into_peer_groups() {
        let raw = r#"
schema_version = 2

[channels.slack]
enabled = true
bot_token = "xoxb-tok"
allowed_users = ["U111"]
"#;
        let parsed = crate::migration::migrate_to_current(raw).expect("migration succeeds");
        let group = parsed
            .peer_groups
            .get("slack_default")
            .expect("V2 slack.allowed_users must fold into peer_groups.slack_default");
        assert_eq!(group.channel, "slack");
        let usernames: Vec<&str> = group.external_peers.iter().map(|p| p.as_str()).collect();
        assert_eq!(usernames, vec!["U111"]);
    }

    #[test]
    async fn slack_config_deserializes_with_channel_ids() {
        let json = r#"{"bot_token":"xoxb-tok","channel_ids":["C111","D222"]}"#;
        let parsed: SlackConfig = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.channel_ids, vec!["C111", "D222"]);
        assert!(!parsed.interrupt_on_new_message);
        assert_eq!(parsed.thread_replies, None);
        assert!(!parsed.mention_only);
    }

    #[test]
    async fn slack_config_deserializes_with_mention_only() {
        let json = r#"{"bot_token":"xoxb-tok","mention_only":true}"#;
        let parsed: SlackConfig = serde_json::from_str(json).unwrap();
        assert!(parsed.mention_only);
        assert!(!parsed.interrupt_on_new_message);
        assert_eq!(parsed.thread_replies, None);
    }

    #[test]
    async fn slack_config_deserializes_interrupt_on_new_message() {
        let json = r#"{"bot_token":"xoxb-tok","interrupt_on_new_message":true}"#;
        let parsed: SlackConfig = serde_json::from_str(json).unwrap();
        assert!(parsed.interrupt_on_new_message);
        assert_eq!(parsed.thread_replies, None);
        assert!(!parsed.mention_only);
    }

    #[test]
    async fn slack_config_deserializes_thread_replies() {
        let json = r#"{"bot_token":"xoxb-tok","thread_replies":false}"#;
        let parsed: SlackConfig = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.thread_replies, Some(false));
        assert!(!parsed.interrupt_on_new_message);
        assert!(!parsed.mention_only);
    }

    #[test]
    async fn discord_config_default_interrupt_on_new_message_is_false() {
        let json = r#"{"bot_token":"tok"}"#;
        let parsed: DiscordConfig = serde_json::from_str(json).unwrap();
        assert!(!parsed.interrupt_on_new_message);
    }

    #[test]
    async fn discord_config_deserializes_interrupt_on_new_message_true() {
        let json = r#"{"bot_token":"tok","interrupt_on_new_message":true}"#;
        let parsed: DiscordConfig = serde_json::from_str(json).unwrap();
        assert!(parsed.interrupt_on_new_message);
    }

    #[test]
    async fn discord_config_toml_backward_compat() {
        let toml_str = r#"
bot_token = "tok"
guild_id = "123"
"#;
        let parsed: DiscordConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(parsed.bot_token, "tok");
    }

    #[test]
    async fn slack_config_toml_with_channel_ids() {
        let toml_str = r#"
bot_token = "xoxb-tok"
channel_ids = ["C123", "D456"]
"#;
        let parsed: SlackConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(parsed.channel_ids, vec!["C123", "D456"]);
        assert!(!parsed.interrupt_on_new_message);
        assert_eq!(parsed.thread_replies, None);
        assert!(!parsed.mention_only);
    }

    #[test]
    async fn slack_config_toml_without_channel_ids_defaults_empty() {
        let toml_str = r#"
bot_token = "xoxb-tok"
"#;
        let parsed: SlackConfig = toml::from_str(toml_str).unwrap();
        assert!(parsed.channel_ids.is_empty());
    }

    #[test]
    async fn mattermost_config_default_interrupt_on_new_message_is_false() {
        let json = r#"{"url":"https://mm.example.com","bot_token":"tok"}"#;
        let parsed: MattermostConfig = serde_json::from_str(json).unwrap();
        assert!(!parsed.interrupt_on_new_message);
    }

    #[test]
    async fn mattermost_config_deserializes_interrupt_on_new_message_true() {
        let json =
            r#"{"url":"https://mm.example.com","bot_token":"tok","interrupt_on_new_message":true}"#;
        let parsed: MattermostConfig = serde_json::from_str(json).unwrap();
        assert!(parsed.interrupt_on_new_message);
    }

    #[test]
    async fn whatsapp_config_default_interrupt_on_new_message_is_false() {
        let json = r#"{"session_path":"/tmp/omnesagent-whatsapp-session.db"}"#;
        let parsed: WhatsAppConfig = serde_json::from_str(json).unwrap();
        assert!(!parsed.interrupt_on_new_message);
    }

    #[test]
    async fn whatsapp_config_deserializes_interrupt_on_new_message_true() {
        let json = r#"{"session_path":"/tmp/omnesagent-whatsapp-session.db","interrupt_on_new_message":true}"#;
        let parsed: WhatsAppConfig = serde_json::from_str(json).unwrap();
        assert!(parsed.interrupt_on_new_message);
    }

    #[test]
    async fn webhook_config_with_secret() {
        let json = r#"{"port":8080,"secret":"my-secret-key"}"#;
        let parsed: WebhookConfig = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.secret.as_deref(), Some("my-secret-key"));
    }

    #[test]
    async fn webhook_config_without_secret() {
        let json = r#"{"port":8080}"#;
        let parsed: WebhookConfig = serde_json::from_str(json).unwrap();
        assert!(parsed.secret.is_none());
        assert_eq!(parsed.port, 8080);
    }

    #[test]
    async fn webhook_config_port_defaults_when_omitted() {
        let p: WebhookConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(p.port, 8090);
    }

    #[test]
    async fn webhook_config_retry_fields_default_to_none() {
        let json = r#"{"port":8080}"#;
        let parsed: WebhookConfig = serde_json::from_str(json).unwrap();
        assert!(parsed.max_retries.is_none());
        assert!(parsed.retry_base_delay_ms.is_none());
        assert!(parsed.retry_max_delay_ms.is_none());
    }

    #[test]
    async fn webhook_config_retry_fields_roundtrip() {
        let wc = WebhookConfig {
            enabled: true,
            port: 8080,
            listen_path: None,
            send_url: Some("https://example.com/cb".into()),
            send_method: None,
            auth_header: None,
            secret: None,
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
            max_retries: Some(5),
            retry_base_delay_ms: Some(250),
            retry_max_delay_ms: Some(10_000),
        };

        let json = serde_json::to_string(&wc).unwrap();
        let parsed: WebhookConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.max_retries, Some(5));
        assert_eq!(parsed.retry_base_delay_ms, Some(250));
        assert_eq!(parsed.retry_max_delay_ms, Some(10_000));

        let toml_str = toml::to_string(&wc).unwrap();
        let parsed: WebhookConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.max_retries, Some(5));
        assert_eq!(parsed.retry_base_delay_ms, Some(250));
        assert_eq!(parsed.retry_max_delay_ms, Some(10_000));
    }

    // ── WhatsApp config ──────────────────────────────────────

    #[test]
    async fn whatsapp_config_serde() {
        let wc = WhatsAppConfig {
            enabled: true,
            access_token: Some("EAABx...".into()),
            phone_number_id: Some("123456789".into()),
            verify_token: Some("my-verify-token".into()),
            app_secret: None,
            session_path: None,
            pair_phone: None,
            pair_code: None,
            ws_url: None,
            push_name: None,
            mention_only: false,
            passive_group_context: false,
            interrupt_on_new_message: false,
            mode: WhatsAppWebMode::default(),
            dm_policy: WhatsAppChatPolicy::default(),
            group_policy: WhatsAppChatPolicy::default(),
            self_chat_mode: false,
            dm_mention_patterns: vec![],
            group_mention_patterns: vec![],
            allowed_groups: vec![],
            proxy_url: None,
            approval_timeout_secs: 300,
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };
        let json = serde_json::to_string(&wc).unwrap();
        let parsed: WhatsAppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.access_token, Some("EAABx...".into()));
        assert_eq!(parsed.phone_number_id, Some("123456789".into()));
        assert_eq!(parsed.verify_token, Some("my-verify-token".into()));
    }

    #[test]
    async fn whatsapp_config_toml_roundtrip() {
        let wc = WhatsAppConfig {
            enabled: true,
            access_token: Some("tok".into()),
            phone_number_id: Some("12345".into()),
            verify_token: Some("verify".into()),
            app_secret: Some("secret123".into()),
            session_path: None,
            pair_phone: None,
            pair_code: None,
            ws_url: None,
            push_name: None,
            mention_only: false,
            passive_group_context: false,
            interrupt_on_new_message: false,
            mode: WhatsAppWebMode::default(),
            dm_policy: WhatsAppChatPolicy::default(),
            group_policy: WhatsAppChatPolicy::default(),
            self_chat_mode: false,
            dm_mention_patterns: vec![],
            group_mention_patterns: vec![],
            allowed_groups: vec![],
            proxy_url: None,
            approval_timeout_secs: 300,
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };
        let toml_str = toml::to_string(&wc).unwrap();
        let parsed: WhatsAppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.phone_number_id, Some("12345".into()));
    }

    #[test]
    async fn whatsapp_config_passive_group_context_defaults_off() {
        let parsed: WhatsAppConfig = serde_json::from_str("{}").unwrap();
        assert!(!parsed.passive_group_context);
    }

    #[test]
    async fn whatsapp_config_passive_group_context_deserializes_true() {
        let parsed: WhatsAppConfig =
            serde_json::from_str(r#"{"passive_group_context":true}"#).unwrap();
        assert!(parsed.passive_group_context);
    }

    #[test]
    async fn whatsapp_config_push_name_absent_stays_none_and_round_trips_non_ascii() {
        let parsed: WhatsAppConfig = serde_json::from_str("{}").unwrap();
        assert!(parsed.push_name.is_none());

        let name = "סוכן – שירות";
        let wc = WhatsAppConfig {
            push_name: Some(name.into()),
            ..Default::default()
        };
        let parsed: WhatsAppConfig = toml::from_str(&toml::to_string(&wc).unwrap()).unwrap();
        assert_eq!(parsed.push_name.as_deref(), Some(name));
    }

    #[test]
    async fn whatsapp_config_push_name_is_not_a_web_selector() {
        let wc = WhatsAppConfig {
            push_name: Some("OmnesAgentAgent".into()),
            ..Default::default()
        };
        assert!(!wc.has_web_selector());
        assert_eq!(wc.backend_type(), "cloud");
    }

    #[test]
    async fn whatsapp_v2_allowed_numbers_fold_into_peer_groups() {
        // V2 `allowed_numbers` on a WhatsApp channel migrates to a
        // synthesized `peer_groups.whatsapp_default` group. The wildcard
        // `*` is dropped at synthesis; concrete numbers round-trip.
        let raw = r#"
schema_version = 2

[channels.whatsapp]
enabled = true
access_token = "tok"
phone_number_id = "123"
verify_token = "ver"
allowed_numbers = ["+1", "+2"]
"#;
        let parsed = crate::migration::migrate_to_current(raw).expect("migration succeeds");
        let group = parsed
            .peer_groups
            .get("whatsapp_default")
            .expect("V2 whatsapp.allowed_numbers must fold into peer_groups.whatsapp_default");
        assert_eq!(group.channel, "whatsapp");
        let usernames: Vec<&str> = group.external_peers.iter().map(|p| p.as_str()).collect();
        assert_eq!(usernames, vec!["+1", "+2"]);
    }

    #[test]
    async fn whatsapp_config_backend_type_cloud_precedence_when_ambiguous() {
        let wc = WhatsAppConfig {
            enabled: true,
            access_token: Some("tok".into()),
            phone_number_id: Some("123".into()),
            verify_token: Some("ver".into()),
            app_secret: None,
            session_path: Some("~/.omnesagent/state/whatsapp-web/session.db".into()),
            pair_phone: None,
            pair_code: None,
            ws_url: None,
            push_name: None,
            mention_only: false,
            passive_group_context: false,
            interrupt_on_new_message: false,
            mode: WhatsAppWebMode::default(),
            dm_policy: WhatsAppChatPolicy::default(),
            group_policy: WhatsAppChatPolicy::default(),
            self_chat_mode: false,
            dm_mention_patterns: vec![],
            group_mention_patterns: vec![],
            allowed_groups: vec![],
            proxy_url: None,
            approval_timeout_secs: 300,
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };
        assert!(wc.is_ambiguous_config());
        assert_eq!(wc.backend_type(), "cloud");
    }

    #[test]
    async fn whatsapp_config_backend_type_web() {
        let wc = WhatsAppConfig {
            enabled: true,
            access_token: None,
            phone_number_id: None,
            verify_token: None,
            app_secret: None,
            session_path: Some("~/.omnesagent/state/whatsapp-web/session.db".into()),
            pair_phone: None,
            pair_code: None,
            ws_url: None,
            push_name: None,
            mention_only: false,
            passive_group_context: false,
            interrupt_on_new_message: false,
            mode: WhatsAppWebMode::default(),
            dm_policy: WhatsAppChatPolicy::default(),
            group_policy: WhatsAppChatPolicy::default(),
            self_chat_mode: false,
            dm_mention_patterns: vec![],
            group_mention_patterns: vec![],
            allowed_groups: vec![],
            proxy_url: None,
            approval_timeout_secs: 300,
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };
        assert!(!wc.is_ambiguous_config());
        assert_eq!(wc.backend_type(), "web");
    }

    #[test]
    async fn whatsapp_config_backend_type_web_from_personal_pairing() {
        let wc = WhatsAppConfig {
            enabled: true,
            mode: WhatsAppWebMode::Personal,
            pair_phone: Some("+10000000000".into()),
            ..Default::default()
        };
        assert_eq!(wc.backend_type(), "web");
        assert!(wc.is_web_config());
        assert!(!wc.is_cloud_config());

        let pair_only = WhatsAppConfig {
            enabled: true,
            pair_phone: Some("+10000000000".into()),
            ..Default::default()
        };
        assert_eq!(pair_only.backend_type(), "web");

        let empty = WhatsAppConfig {
            enabled: true,
            ..Default::default()
        };
        assert_eq!(empty.backend_type(), "cloud");

        let cloud_plus_pairing = WhatsAppConfig {
            enabled: true,
            phone_number_id: Some("123".into()),
            pair_phone: Some("+10000000000".into()),
            ..Default::default()
        };
        assert_eq!(cloud_plus_pairing.backend_type(), "cloud");
        assert!(cloud_plus_pairing.is_ambiguous_config());
    }

    #[test]
    async fn channels_with_whatsapp() {
        let c = ChannelsConfig {
            cli: true,
            telegram: HashMap::new(),
            discord: HashMap::new(),
            slack: HashMap::new(),
            mattermost: HashMap::new(),
            webhook: HashMap::new(),
            imessage: HashMap::new(),
            matrix: HashMap::new(),
            signal: HashMap::new(),
            whatsapp: HashMap::from([(
                "default".to_string(),
                WhatsAppConfig {
                    enabled: true,
                    access_token: Some("tok".into()),
                    phone_number_id: Some("123".into()),
                    verify_token: Some("ver".into()),
                    app_secret: None,
                    session_path: None,
                    pair_phone: None,
                    pair_code: None,
                    ws_url: None,
                    push_name: None,
                    mention_only: false,
                    passive_group_context: false,
                    interrupt_on_new_message: false,
                    mode: WhatsAppWebMode::default(),
                    dm_policy: WhatsAppChatPolicy::default(),
                    group_policy: WhatsAppChatPolicy::default(),
                    self_chat_mode: false,
                    dm_mention_patterns: vec![],
                    group_mention_patterns: vec![],
                    allowed_groups: vec![],
                    proxy_url: None,
                    approval_timeout_secs: 300,
                    excluded_tools: vec![],
                    reply_min_interval_secs: 0,
                    reply_queue_depth_max: 0,
                },
            )]),
            linq: HashMap::new(),
            nextcloud_talk: HashMap::new(),
            email: HashMap::new(),
            gmail_push: HashMap::new(),
            irc: HashMap::new(),
            twitch: HashMap::new(),
            lark: HashMap::new(),
            line: HashMap::new(),
            dingtalk: HashMap::new(),
            wecom: HashMap::new(),
            wecom_ws: HashMap::new(),
            wechat: HashMap::new(),
            qq: HashMap::new(),
            twitter: HashMap::new(),
            mochat: HashMap::new(),
            nostr: HashMap::new(),
            clawdtalk: HashMap::new(),
            reddit: HashMap::new(),
            bluesky: HashMap::new(),
            git: HashMap::new(),
            voice_call: HashMap::new(),
            voice_duplex: HashMap::new(),
            voice_wake: HashMap::new(),
            mqtt: HashMap::new(),
            amqp: HashMap::new(),
            filesystem: HashMap::new(),
            plugin: HashMap::new(),
            message_timeout_secs: 300,
            max_concurrent_per_channel: default_channel_max_concurrent_per_channel(),
            ack_reactions: true,
            show_tool_calls: true,
            session_persistence: true,
            session_backend: default_session_backend(),
            session_ttl_hours: 0,
            debounce_ms: 0,
        };
        let toml_str = toml::to_string_pretty(&c).unwrap();
        let parsed: ChannelsConfig = toml::from_str(&toml_str).unwrap();
        assert!(!parsed.whatsapp.is_empty());
        let wa = parsed.whatsapp.get("default").unwrap();
        assert_eq!(wa.phone_number_id, Some("123".into()));
    }

    #[test]
    async fn channels_default_has_no_whatsapp() {
        let c = ChannelsConfig::default();
        assert!(c.whatsapp.is_empty());
    }

    #[test]
    async fn channels_default_has_no_nextcloud_talk() {
        let c = ChannelsConfig::default();
        assert!(c.nextcloud_talk.is_empty());
    }

    // ══════════════════════════════════════════════════════════
    // SECURITY CHECKLIST TESTS — Gateway config
    // ══════════════════════════════════════════════════════════

    #[test]
    async fn checklist_gateway_default_requires_pairing() {
        let g = GatewayConfig::default();
        assert!(g.require_pairing, "Pairing must be required by default");
    }

    #[test]
    async fn checklist_gateway_default_blocks_public_bind() {
        let g = GatewayConfig::default();
        assert!(
            !g.allow_public_bind,
            "Public bind must be blocked by default"
        );
    }

    #[test]
    async fn checklist_gateway_default_no_tokens() {
        let g = GatewayConfig::default();
        assert!(
            g.paired_tokens.is_empty(),
            "No pre-paired tokens by default"
        );
        assert_eq!(g.pair_rate_limit_per_minute, 10);
        assert_eq!(g.webhook_rate_limit_per_minute, 60);
        assert!(!g.trust_forwarded_headers);
        assert_eq!(g.rate_limit_max_keys, 10_000);
        assert_eq!(g.idempotency_ttl_secs, 300);
        assert_eq!(g.idempotency_max_keys, 10_000);
    }

    #[test]
    async fn checklist_gateway_cli_default_host_is_localhost() {
        // The CLI default for --host is 127.0.0.1 (checked in main.rs)
        // Here we verify the config default matches
        let c = Config::default();
        assert!(
            c.gateway.require_pairing,
            "Config default must require pairing"
        );
        assert!(
            !c.gateway.allow_public_bind,
            "Config default must block public bind"
        );
    }

    #[test]
    async fn checklist_gateway_serde_roundtrip() {
        let g = GatewayConfig {
            port: 42617,
            host: "127.0.0.1".into(),
            require_pairing: true,
            allow_public_bind: false,
            allow_remote_admin: false,
            paired_tokens: vec!["zc_test_token".into()],
            pair_rate_limit_per_minute: 12,
            webhook_rate_limit_per_minute: 80,
            webhook_secret: None,
            trust_forwarded_headers: true,
            path_prefix: Some("/omnesagent".into()),
            rate_limit_max_keys: 2048,
            idempotency_ttl_secs: 600,
            idempotency_max_keys: 4096,
            session_persistence: true,
            session_ttl_hours: 0,
            websocket_ping_interval_secs: 30,
            pairing_dashboard: PairingDashboardConfig::default(),
            web_dist_dir: None,
            tls: None,
            request_timeout_secs: 30,
            long_running_request_timeout_secs: 600,
            check_updates: true,
            allow_self_upgrade: false,
        };
        let toml_str = toml::to_string(&g).unwrap();
        let parsed: GatewayConfig = toml::from_str(&toml_str).unwrap();
        assert!(parsed.require_pairing);
        assert!(parsed.session_persistence);
        assert_eq!(parsed.session_ttl_hours, 0);
        assert!(!parsed.allow_public_bind);
        assert_eq!(parsed.paired_tokens, vec!["zc_test_token"]);
        assert_eq!(parsed.pair_rate_limit_per_minute, 12);
        assert_eq!(parsed.webhook_rate_limit_per_minute, 80);
        assert!(parsed.trust_forwarded_headers);
        assert_eq!(parsed.path_prefix.as_deref(), Some("/omnesagent"));
        assert_eq!(parsed.rate_limit_max_keys, 2048);
        assert_eq!(parsed.idempotency_ttl_secs, 600);
        assert_eq!(parsed.idempotency_max_keys, 4096);
        assert!(parsed.check_updates);
        assert!(!parsed.allow_self_upgrade);
    }

    #[test]
    async fn checklist_gateway_backward_compat_no_gateway_section() {
        // Old configs without [gateway] should get secure defaults
        let minimal = r#"
workspace_dir = "/tmp/ws"
config_path = "/tmp/config.toml"
default_temperature = 0.7
"#;
        let parsed = parse_test_config(minimal);
        assert!(
            parsed.gateway.require_pairing,
            "Missing [gateway] must default to require_pairing=true"
        );
        assert!(
            !parsed.gateway.allow_public_bind,
            "Missing [gateway] must default to allow_public_bind=false"
        );
    }

    #[test]
    async fn checklist_risk_profile_default_is_workspace_scoped() {
        let a = RiskProfileConfig::default();
        assert!(a.workspace_only, "Default profile must be workspace_only");
        assert!(
            !a.forbidden_paths.is_empty(),
            "Default forbidden_paths must not be empty"
        );
        #[cfg(not(target_os = "windows"))]
        {
            assert!(
                a.forbidden_paths.iter().any(|p| p == "/etc"),
                "Must block /etc on Unix"
            );
            assert!(
                a.forbidden_paths.iter().any(|p| p == "/proc"),
                "Must block /proc on Unix"
            );
        }
        #[cfg(target_os = "windows")]
        {
            assert!(
                a.forbidden_paths.iter().any(|p| p == "C:\\Windows"),
                "Must block C:\\Windows on Windows"
            );
            assert!(
                a.forbidden_paths.iter().any(|p| p == "C:\\Program Files"),
                "Must block C:\\Program Files on Windows"
            );
        }
        assert!(
            a.forbidden_paths.contains(&"~/.ssh".to_string()),
            "Must block ~/.ssh"
        );
    }

    // ══════════════════════════════════════════════════════════
    // COMPOSIO CONFIG TESTS
    // ══════════════════════════════════════════════════════════

    #[test]
    async fn composio_config_default_disabled() {
        let c = ComposioConfig::default();
        assert!(!c.enabled, "Composio must be disabled by default");
        assert!(c.api_key.is_none(), "No API key by default");
        assert_eq!(c.entity_id, "default");
    }

    #[test]
    async fn composio_config_serde_roundtrip() {
        let c = ComposioConfig {
            enabled: true,
            api_key: Some("comp-key-123".into()),
            entity_id: "user42".into(),
        };
        let toml_str = toml::to_string(&c).unwrap();
        let parsed: ComposioConfig = toml::from_str(&toml_str).unwrap();
        assert!(parsed.enabled);
        assert_eq!(parsed.api_key.as_deref(), Some("comp-key-123"));
        assert_eq!(parsed.entity_id, "user42");
    }

    #[test]
    async fn composio_config_backward_compat_missing_section() {
        let minimal = r#"
workspace_dir = "/tmp/ws"
config_path = "/tmp/config.toml"
default_temperature = 0.7
"#;
        let parsed = parse_test_config(minimal);
        assert!(
            !parsed.composio.enabled,
            "Missing [composio] must default to disabled"
        );
        assert!(parsed.composio.api_key.is_none());
    }

    #[test]
    async fn composio_config_partial_toml() {
        let toml_str = r"
enabled = true
";
        let parsed: ComposioConfig = toml::from_str(toml_str).unwrap();
        assert!(parsed.enabled);
        assert!(parsed.api_key.is_none());
        assert_eq!(parsed.entity_id, "default");
    }

    #[test]
    async fn composio_config_enable_alias_supported() {
        let toml_str = r"
enable = true
";
        let parsed: ComposioConfig = toml::from_str(toml_str).unwrap();
        assert!(parsed.enabled);
        assert!(parsed.api_key.is_none());
        assert_eq!(parsed.entity_id, "default");
    }

    // ══════════════════════════════════════════════════════════
    // SECRETS CONFIG TESTS
    // ══════════════════════════════════════════════════════════

    #[test]
    async fn secrets_config_default_encrypts() {
        let s = SecretsConfig::default();
        assert!(s.encrypt, "Encryption must be enabled by default");
    }

    #[test]
    async fn secrets_config_serde_roundtrip() {
        let s = SecretsConfig { encrypt: false };
        let toml_str = toml::to_string(&s).unwrap();
        let parsed: SecretsConfig = toml::from_str(&toml_str).unwrap();
        assert!(!parsed.encrypt);
    }

    #[test]
    async fn secrets_config_backward_compat_missing_section() {
        let minimal = r#"
workspace_dir = "/tmp/ws"
config_path = "/tmp/config.toml"
default_temperature = 0.7
"#;
        let parsed = parse_test_config(minimal);
        assert!(
            parsed.secrets.encrypt,
            "Missing [secrets] must default to encrypt=true"
        );
    }

    #[test]
    async fn config_default_has_composio_and_secrets() {
        let c = Config::default();
        assert!(!c.composio.enabled);
        assert!(c.composio.api_key.is_none());
        assert!(c.secrets.encrypt);
        assert!(c.browser.enabled);
        assert_eq!(c.browser.allowed_domains, vec!["*".to_string()]);
    }

    #[test]
    async fn browser_config_default_enabled() {
        let b = BrowserConfig::default();
        assert!(b.enabled);
        assert_eq!(b.allowed_domains, vec!["*".to_string()]);
        assert_eq!(b.backend, "agent_browser");
        assert_eq!(b.headed, None);
        assert!(b.native_headless);
        assert_eq!(b.native_webdriver_url, "http://127.0.0.1:9515");
        assert!(b.native_chrome_path.is_none());
        assert_eq!(b.computer_use.endpoint, "http://127.0.0.1:8787/v1/actions");
        assert_eq!(b.computer_use.timeout_ms, 15_000);
        assert!(!b.computer_use.allow_remote_endpoint);
        assert!(b.computer_use.window_allowlist.is_empty());
        assert!(b.computer_use.max_coordinate_x.is_none());
        assert!(b.computer_use.max_coordinate_y.is_none());
    }

    #[test]
    async fn browser_config_serde_roundtrip() {
        let b = BrowserConfig {
            enabled: true,
            allowed_domains: vec!["example.com".into(), "docs.example.com".into()],
            session_name: None,
            backend: "auto".into(),
            headed: Some(true),
            native_headless: false,
            native_webdriver_url: "http://localhost:4444".into(),
            native_chrome_path: Some("/usr/bin/chromium".into()),
            computer_use: BrowserComputerUseConfig {
                endpoint: "https://computer-use.example.com/v1/actions".into(),
                api_key: Some("test-token".into()),
                timeout_ms: 8_000,
                allow_remote_endpoint: true,
                window_allowlist: vec!["Chrome".into(), "Visual Studio Code".into()],
                max_coordinate_x: Some(3840),
                max_coordinate_y: Some(2160),
            },
            allowed_private_hosts: vec![],
        };
        let toml_str = toml::to_string(&b).unwrap();
        let parsed: BrowserConfig = toml::from_str(&toml_str).unwrap();
        assert!(parsed.enabled);
        assert_eq!(parsed.allowed_domains.len(), 2);
        assert_eq!(parsed.allowed_domains[0], "example.com");
        assert_eq!(parsed.backend, "auto");
        assert_eq!(parsed.headed, Some(true));
        assert!(!parsed.native_headless);
        assert_eq!(parsed.native_webdriver_url, "http://localhost:4444");
        assert_eq!(
            parsed.native_chrome_path.as_deref(),
            Some("/usr/bin/chromium")
        );
        assert_eq!(
            parsed.computer_use.endpoint,
            "https://computer-use.example.com/v1/actions"
        );
        assert_eq!(parsed.computer_use.api_key.as_deref(), Some("test-token"));
        assert_eq!(parsed.computer_use.timeout_ms, 8_000);
        assert!(parsed.computer_use.allow_remote_endpoint);
        assert_eq!(parsed.computer_use.window_allowlist.len(), 2);
        assert_eq!(parsed.computer_use.max_coordinate_x, Some(3840));
        assert_eq!(parsed.computer_use.max_coordinate_y, Some(2160));
    }

    #[test]
    async fn browser_config_parses_headed_true() {
        let parsed: BrowserConfig = toml::from_str(
            r#"
backend = "agent_browser"
headed = true
"#,
        )
        .unwrap();

        assert_eq!(parsed.backend, "agent_browser");
        assert_eq!(parsed.headed, Some(true));
        assert!(parsed.native_headless);
    }

    #[test]
    async fn browser_config_backward_compat_missing_section() {
        let minimal = r#"
workspace_dir = "/tmp/ws"
config_path = "/tmp/config.toml"
default_temperature = 0.7
"#;
        let parsed = parse_test_config(minimal);
        assert!(parsed.browser.enabled);
        assert_eq!(parsed.browser.allowed_domains, vec!["*".to_string()]);
    }

    async fn env_override_lock() -> MutexGuard<'static, ()> {
        // Delegate to the crate-shared lock so env-mutating tests in this
        // module serialize against `env_overrides::tests` too. Without
        // this, tests across the two modules race on `OMNESAGENT_*` vars.
        crate::env_overrides::env_test_lock().await
    }

    #[test]
    async fn slack_config_deserializes_without_bot_token() {
        // Regression for /: before `bot_token` became
        // `Option<String>` + `#[serde(default)]`, a config that omitted it
        // failed to deserialize with `missing field 'bot_token'`, aborting
        // startup before the env-var fallback could ever run.
        let parsed: SlackConfig = toml::from_str("enabled = true\n")
            .expect("SlackConfig must deserialize without bot_token");
        assert!(parsed.bot_token.is_none());
    }

    #[test]
    async fn slack_config_deserializes_explicit_bot_token() {
        let parsed: SlackConfig =
            toml::from_str("enabled = true\nbot_token = \"xoxb-from-toml\"\n").unwrap();
        assert_eq!(parsed.bot_token.as_deref(), Some("xoxb-from-toml"));
    }

    /// Set (`Some`) or clear (`None`) an env var. Callers must hold
    /// `env_override_lock()`. Used to snapshot-and-restore the Slack token
    /// vars so these tests leave the process environment exactly as found.
    fn set_or_clear_env(key: &str, value: Option<&str>) {
        // SAFETY: callers serialize on env_override_lock().
        unsafe {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }

    #[test]
    async fn slack_resolved_bot_token_falls_back_to_env() {
        let _env_guard = env_override_lock().await;
        let prev_bot = std::env::var("SLACK_BOT_TOKEN").ok();
        let prev_zc = std::env::var("OMNESAGENT_SLACK_BOT_TOKEN").ok();
        set_or_clear_env("OMNESAGENT_SLACK_BOT_TOKEN", None);
        set_or_clear_env("SLACK_BOT_TOKEN", Some("xoxb-from-env"));

        let cfg = SlackConfig {
            bot_token: None,
            ..Default::default()
        };
        assert_eq!(cfg.resolved_bot_token().as_deref(), Some("xoxb-from-env"));

        set_or_clear_env("SLACK_BOT_TOKEN", prev_bot.as_deref());
        set_or_clear_env("OMNESAGENT_SLACK_BOT_TOKEN", prev_zc.as_deref());
    }

    #[test]
    async fn slack_resolved_bot_token_prefers_omnesagent_prefix() {
        let _env_guard = env_override_lock().await;
        let prev_bot = std::env::var("SLACK_BOT_TOKEN").ok();
        let prev_zc = std::env::var("OMNESAGENT_SLACK_BOT_TOKEN").ok();
        set_or_clear_env("SLACK_BOT_TOKEN", Some("xoxb-generic"));
        set_or_clear_env("OMNESAGENT_SLACK_BOT_TOKEN", Some("xoxb-omnesagent"));

        let cfg = SlackConfig {
            bot_token: None,
            ..Default::default()
        };
        assert_eq!(cfg.resolved_bot_token().as_deref(), Some("xoxb-omnesagent"));

        set_or_clear_env("SLACK_BOT_TOKEN", prev_bot.as_deref());
        set_or_clear_env("OMNESAGENT_SLACK_BOT_TOKEN", prev_zc.as_deref());
    }

    #[test]
    async fn slack_resolved_bot_token_prefers_config_over_env() {
        let _env_guard = env_override_lock().await;
        let prev_bot = std::env::var("SLACK_BOT_TOKEN").ok();
        set_or_clear_env("SLACK_BOT_TOKEN", Some("xoxb-from-env"));

        let cfg = SlackConfig {
            bot_token: Some("xoxb-from-config".to_string()),
            ..Default::default()
        };
        assert_eq!(
            cfg.resolved_bot_token().as_deref(),
            Some("xoxb-from-config")
        );

        set_or_clear_env("SLACK_BOT_TOKEN", prev_bot.as_deref());
    }

    #[test]
    async fn v1_known_provider_migrates_with_globals_folded_onto_typed_slot() {
        // Top-level `model_provider` + `model` + `default_temperature` flow
        // onto the migrated typed-slot entry. Vendor-canonical names like
        // `openai` map straight to their typed slot; `wire_api` and
        // `requires_openai_auth` survive the move.
        //
        // (Unknown V1 names like `sub2api` are intentionally silent-dropped
        // by the V2→V3 migration — see the `Unknown/passthrough` arm of
        // `normalize_provider_type` in schema/v2.rs.)
        let raw = r#"
default_temperature = 0.7
model_provider = "openai"
model = "gpt-5.3-codex"

[model_providers.openai]
api_key = "sk-test"
uri = "https://api.openai.com/v1"
wire_api = "responses"
requires_openai_auth = true
"#;

        let parsed = crate::migration::migrate_to_current(raw).expect("migration succeeds");
        assert!(
            parsed
                .providers
                .models
                .contains_model_provider_type("openai"),
            "vendor-canonical V1 provider should land in its typed slot",
        );
        let profile = parsed
            .providers
            .models
            .find("openai", "default")
            .expect("openai.default entry");
        assert_eq!(profile.api_key.as_deref(), Some("sk-test"));
        assert_eq!(profile.uri.as_deref(), Some("https://api.openai.com/v1"));
        assert_eq!(profile.model.as_deref(), Some("gpt-5.3-codex"));
        assert_eq!(profile.wire_api, Some(WireApi::Responses));
        assert!(profile.requires_openai_auth);
    }

    #[test]
    async fn typed_custom_slot_routes_uri_through_find() {
        let _env_guard = env_override_lock().await;
        let mut config = Config::default();
        config.providers.models.custom.insert(
            "default".to_string(),
            CustomModelProviderConfig {
                base: ModelProviderConfig {
                    uri: Some("https://api.tonsof.blue/v1".to_string()),
                    ..Default::default()
                },
            },
        );

        assert_eq!(
            config
                .providers
                .models
                .find("custom", "default")
                .and_then(|e| e.uri.as_deref()),
            Some("https://api.tonsof.blue/v1")
        );
        assert!(config.providers.models.find("custom", "default").is_some());
    }

    #[test]
    async fn openai_codex_alias_carries_responses_wire_api_and_requires_openai_auth() {
        let _env_guard = env_override_lock().await;
        let mut config = Config::default();
        config.providers.models.openai.insert(
            "codex".to_string(),
            OpenAIModelProviderConfig {
                base: ModelProviderConfig {
                    uri: Some("https://api.tonsof.blue".to_string()),
                    wire_api: Some(WireApi::Responses),
                    requires_openai_auth: true,
                    ..Default::default()
                },
            },
        );

        let entry = config
            .providers
            .models
            .find("openai", "codex")
            .expect("openai.codex entry");
        assert_eq!(entry.uri.as_deref(), Some("https://api.tonsof.blue"));
        assert_eq!(entry.wire_api, Some(WireApi::Responses));
        assert!(entry.requires_openai_auth);
    }

    /// Round-trip test for the config CLI: a TOML file with a typed-family
    /// model entry must deserialize, find via the typed accessor, and
    /// re-serialize without losing any field.
    #[test]
    async fn provider_models_round_trips_through_load_apply_serialize() {
        let _env_guard = env_override_lock().await;
        let toml_in = r#"
schema_version = 3

[providers.models.openrouter.default]
uri = "https://example.invalid/v1"
model = "primary-model"
"#;

        let config: Config = toml::from_str(toml_in).expect("parse toml");

        assert_eq!(
            config
                .providers
                .models
                .find("openrouter", "default")
                .and_then(|e| e.model.as_deref()),
            Some("primary-model"),
        );

        // What `config save` would write back to disk.
        let toml_out = toml::to_string(&config).expect("serialize toml");
        assert!(
            toml_out.contains("primary-model"),
            "serialized config must keep model value; got:\n{toml_out}",
        );
    }

    /// `resolve_default_model` returns the first available `models.*` entry's
    /// model. Returning `None` is reserved for "no model_provider has any model
    /// configured", which callers must surface as a configuration error
    /// rather than silently substituting a vendor default.
    #[test]
    async fn resolve_default_model_picks_first_available() {
        let _env_guard = env_override_lock().await;
        let mut config = Config::default();
        // Empty config: no model anywhere -> None (caller errors loudly).
        assert_eq!(config.resolve_default_model(), None);

        // Add an entry without a model -> still None.
        config
            .providers
            .models
            .anthropic
            .insert("default".into(), AnthropicModelProviderConfig::default());
        assert_eq!(config.resolve_default_model(), None);

        // Add an entry with a model -> first-available wins.
        config.providers.models.together.insert(
            "default".to_string(),
            TogetherModelProviderConfig {
                base: ModelProviderConfig {
                    model: Some("tertiary-model".to_string()),
                    ..Default::default()
                },
            },
        );
        assert_eq!(
            config.resolve_default_model().as_deref(),
            Some("tertiary-model"),
        );

        // Add a model_provider with a model — resolve_default_model finds it.
        config.providers.models.openrouter.insert(
            "default".to_string(),
            OpenRouterModelProviderConfig {
                base: ModelProviderConfig {
                    model: Some("primary-model".to_string()),
                    ..Default::default()
                },
            },
        );
        // resolve_default_model returns the first non-empty model across all model_providers.
        assert!(config.resolve_default_model().is_some());
    }

    #[test]
    async fn save_repairs_bare_config_filename_using_runtime_resolution() {
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let workspace_dir = temp_home.join("workspace");
        let resolved_config_path = temp_home.join(".omnesagent").join("config.toml");

        let original_home = std::env::var("HOME").ok();
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("HOME", &temp_home) };
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("OMNESAGENT_WORKSPACE", &workspace_dir) };

        let mut config = Config {
            data_dir: workspace_dir,
            config_path: PathBuf::from("config.toml"),
            ..Default::default()
        };
        config.providers.models.anthropic.insert(
            "default".to_string(),
            AnthropicModelProviderConfig {
                base: ModelProviderConfig {
                    temperature: Some(0.5),
                    ..Default::default()
                },
            },
        );
        // ModelProvider fields are now resolved directly — no cache needed.
        config.save().await.unwrap();

        assert!(resolved_config_path.exists());
        let saved = tokio::fs::read_to_string(&resolved_config_path)
            .await
            .unwrap();
        let parsed = parse_test_config(&saved);
        assert!(
            (parsed
                .providers
                .models
                .find("anthropic", "default")
                .and_then(|e| e.temperature)
                .unwrap_or(0.7)
                - 0.5)
                .abs()
                < f64::EPSILON
        );

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };
        if let Some(home) = original_home {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::remove_var("HOME") };
        }
        let _ = tokio::fs::remove_dir_all(temp_home).await;
    }

    /// `Config::default()` previously computed its `config_path`/`data_dir`
    /// from `UserDirs::home_dir()` directly, ignoring `OMNESAGENT_CONFIG_DIR`
    /// entirely -- unlike every other path-resolution entry point in this
    /// file. A `Config::default()` built under a `OMNESAGENT_CONFIG_DIR`-isolated
    /// test or deployment therefore still resolved to the real machine's
    /// `~/.omnesagent`.
    #[test]
    async fn default_config_honors_omnesagent_config_dir() {
        let _env_guard = env_override_lock().await;
        let custom_dir =
            std::env::temp_dir().join(format!("omnesagent_test_custom_{}", uuid::Uuid::new_v4()));
        let _config_guard = EnvValueGuard::set("OMNESAGENT_CONFIG_DIR", &custom_dir);

        let config = Config::default();

        assert_eq!(config.config_path, custom_dir.join("config.toml"));
        assert_eq!(config.data_dir, custom_dir.join("data"));
    }

    #[test]
    async fn save_refuses_to_overwrite_existing_runtime_config_from_bare_path() {
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let workspace_dir = temp_home.join("workspace");
        let resolved_config_path = temp_home.join(".omnesagent").join("config.toml");
        let original = "schema_version = 5\n\n[operator_only]\nkeep = true\n";
        tokio::fs::create_dir_all(resolved_config_path.parent().unwrap())
            .await
            .unwrap();
        tokio::fs::write(&resolved_config_path, original)
            .await
            .unwrap();

        let original_home = std::env::var("HOME").ok();
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("HOME", &temp_home) };
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("OMNESAGENT_WORKSPACE", &workspace_dir) };

        let mut config = Config {
            data_dir: workspace_dir,
            config_path: PathBuf::from("config.toml"),
            ..Default::default()
        };
        let save_result = config.save().await;
        config.mark_dirty("observability.backend");
        let save_dirty_result = config.save_dirty().await;
        let written = tokio::fs::read_to_string(&resolved_config_path)
            .await
            .unwrap();

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };
        if let Some(home) = original_home {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::remove_var("HOME") };
        }
        let _ = tokio::fs::remove_dir_all(temp_home).await;

        assert!(
            save_result
                .unwrap_err()
                .to_string()
                .contains("refusing to overwrite existing config"),
        );
        assert!(
            save_dirty_result
                .unwrap_err()
                .to_string()
                .contains("refusing to overwrite existing config"),
        );
        assert_eq!(written, original);
    }

    #[test]
    async fn validate_ollama_cloud_model_requires_remote_api_url() {
        let _env_guard = env_override_lock().await;
        let mut config = Config::default();
        config.providers.models.ollama.insert(
            "default".to_string(),
            OllamaModelProviderConfig {
                base: ModelProviderConfig {
                    model: Some("glm-5:cloud".to_string()),
                    uri: None,
                    api_key: Some("ollama-key".to_string()),
                    ..Default::default()
                },
                ..OllamaModelProviderConfig::default()
            },
        );

        let error = config.validate().expect_err("expected validation to fail");
        assert!(error.to_string().contains(
            "providers.models.ollama.default.model uses ':cloud', but uri is local or unset"
        ));
    }

    #[test]
    async fn validate_ollama_cloud_model_accepts_private_remote_without_api_key() {
        let _env_guard = env_override_lock().await;
        let mut config = Config::default();
        config.providers.models.ollama.insert(
            "default".to_string(),
            OllamaModelProviderConfig {
                base: ModelProviderConfig {
                    model: Some("glm-5:cloud".to_string()),
                    uri: Some("http://192.168.1.100:11434".to_string()),
                    api_key: None,
                    ..Default::default()
                },
                ..OllamaModelProviderConfig::default()
            },
        );

        let result = config.validate();
        assert!(result.is_ok(), "expected validation to pass: {result:?}");
    }

    #[test]
    async fn validate_ollama_cloud_model_requires_api_key_for_official_endpoint() {
        let _env_guard = env_override_lock().await;
        let mut config = Config::default();
        config.providers.models.ollama.insert(
            "default".to_string(),
            OllamaModelProviderConfig {
                base: ModelProviderConfig {
                    model: Some("glm-5:cloud".to_string()),
                    uri: Some("https://ollama.com/api".to_string()),
                    api_key: None,
                    ..Default::default()
                },
                ..OllamaModelProviderConfig::default()
            },
        );

        let error = config.validate().expect_err("expected validation to fail");
        assert!(error.to_string().contains(
            "providers.models.ollama.default.model uses ':cloud', but no API key is configured"
        ));
    }

    #[test]
    async fn validate_ollama_cloud_model_accepts_remote_endpoint_with_typed_api_key() {
        // V0.8.0: env-var fallback (`OLLAMA_API_KEY`) eradicated.
        // Operators set the credential on the typed alias.
        let _env_guard = env_override_lock().await;
        let mut config = Config::default();
        config.providers.models.ollama.insert(
            "default".to_string(),
            OllamaModelProviderConfig {
                base: ModelProviderConfig {
                    model: Some("glm-5:cloud".to_string()),
                    uri: Some("https://ollama.com/api".to_string()),
                    api_key: Some("ollama-typed-key".to_string()),
                    ..Default::default()
                },
                ..OllamaModelProviderConfig::default()
            },
        );

        let result = config.validate();
        assert!(result.is_ok(), "expected validation to pass: {result:?}");
    }

    #[test]
    async fn validate_ollama_cloud_model_checks_each_alias_for_official_key() {
        let _env_guard = env_override_lock().await;
        let mut config = Config::default();
        config.providers.models.ollama.insert(
            "local".to_string(),
            OllamaModelProviderConfig {
                base: ModelProviderConfig {
                    model: Some("llama3".to_string()),
                    uri: Some("http://192.168.1.100:11434".to_string()),
                    ..Default::default()
                },
                ..OllamaModelProviderConfig::default()
            },
        );
        config.providers.models.ollama.insert(
            "cloud".to_string(),
            OllamaModelProviderConfig {
                base: ModelProviderConfig {
                    model: Some("glm-5:cloud".to_string()),
                    uri: Some("https://ollama.com/api".to_string()),
                    api_key: None,
                    ..Default::default()
                },
                ..OllamaModelProviderConfig::default()
            },
        );

        let error = config.validate().expect_err("expected validation to fail");
        assert!(error.to_string().contains(
            "providers.models.ollama.cloud.model uses ':cloud', but no API key is configured"
        ));
    }

    #[test]
    async fn deserialize_rejects_unknown_model_provider_wire_api() {
        let toml = r#"
schema_version = 3

[providers.models.openrouter.default]
uri = "https://api.tonsof.blue/v1"
wire_api = "ws"
"#;
        let err = toml::from_str::<Config>(toml).expect_err("expected deserialize failure");
        let msg = err.to_string();
        assert!(
            msg.contains("wire_api") || msg.contains("ws"),
            "error should reference the invalid wire_api value, got: {msg}"
        );
    }

    #[test]
    async fn resolve_runtime_config_dirs_accepts_legacy_omnesagent_workspace() {
        let _env_guard = env_override_lock().await;
        let default_config_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        let default_workspace_dir = default_config_dir.join("workspace");
        let workspace_dir = default_config_dir.join("profile-a");

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("OMNESAGENT_WORKSPACE", &workspace_dir) };
        let (config_dir, resolved_workspace_dir, source) =
            resolve_runtime_config_dirs(&default_config_dir, &default_workspace_dir)
                .await
                .unwrap();

        // OMNESAGENT_WORKSPACE is the deprecated alias for OMNESAGENT_DATA_DIR.
        // Resolution treats the path as the config root and derives the data
        // sub-dir from it; the source label reflects the deprecated entry.
        assert_eq!(source, ConfigResolutionSource::EnvWorkspaceLegacy);
        assert_eq!(config_dir, workspace_dir);
        assert_eq!(resolved_workspace_dir, workspace_dir.join("data"));

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };
        let _ = fs::remove_dir_all(default_config_dir).await;
    }

    #[test]
    async fn resolve_runtime_config_dirs_uses_env_config_dir_first() {
        let _env_guard = env_override_lock().await;
        let default_config_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        let default_workspace_dir = default_config_dir.join("workspace");
        let explicit_config_dir = default_config_dir.join("explicit-config");

        fs::create_dir_all(&default_config_dir).await.unwrap();

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("OMNESAGENT_CONFIG_DIR", &explicit_config_dir) };
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };

        let (config_dir, resolved_workspace_dir, source) =
            resolve_runtime_config_dirs(&default_config_dir, &default_workspace_dir)
                .await
                .unwrap();

        assert_eq!(source, ConfigResolutionSource::EnvConfigDir);
        assert_eq!(config_dir, explicit_config_dir);
        assert_eq!(resolved_workspace_dir, explicit_config_dir.join("data"));

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_CONFIG_DIR") };
        let _ = fs::remove_dir_all(default_config_dir).await;
    }

    #[test]
    async fn resolve_runtime_config_dirs_falls_back_to_default_layout() {
        let _env_guard = env_override_lock().await;
        let default_config_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        let default_workspace_dir = default_config_dir.join("workspace");

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };
        let (config_dir, resolved_workspace_dir, source) =
            resolve_runtime_config_dirs(&default_config_dir, &default_workspace_dir)
                .await
                .unwrap();

        assert_eq!(source, ConfigResolutionSource::DefaultConfigDir);
        assert_eq!(config_dir, default_config_dir);
        assert_eq!(resolved_workspace_dir, default_workspace_dir);

        let _ = fs::remove_dir_all(default_config_dir).await;
    }

    /// The daemon locks the data directory `resolve_runtime_dirs` reports
    /// before it loads the config, then refuses to start if the loaded
    /// `config.data_dir` differs. Every layout must agree, and the loaded
    /// data directory must stay where that layout keeps its databases.
    #[test]
    #[allow(clippy::large_futures)]
    async fn runtime_dirs_match_the_loaded_data_dir_in_every_layout() {
        let _env_guard = env_override_lock().await;
        let _workspace_guard = EnvValueGuard::remove("OMNESAGENT_WORKSPACE");
        let root =
            std::env::temp_dir().join(format!("omnesagent_test_dirs_{}", uuid::Uuid::new_v4()));

        // (layout, HOME, OMNESAGENT_CONFIG_DIR, OMNESAGENT_DATA_DIR, existing
        // config.toml, expected config dir, expected data dir)
        let docker = root.join("docker");
        let default_home = root.join("default");
        let explicit = root.join("explicit");
        let inline = root.join("inline");
        let fresh = root.join("fresh");
        let cases = [
            (
                "container: OMNESAGENT_DATA_DIR beside .omnesagent/config.toml",
                docker.clone(),
                None,
                Some(docker.join("data")),
                Some(docker.join(".omnesagent/config.toml")),
                docker.join(".omnesagent"),
                docker.join(".omnesagent/data"),
            ),
            (
                "default ~/.omnesagent",
                default_home.clone(),
                None,
                None,
                Some(default_home.join(".omnesagent/config.toml")),
                default_home.join(".omnesagent"),
                default_home.join(".omnesagent/data"),
            ),
            (
                "explicit --config-dir",
                root.join("explicit-home"),
                Some(explicit.clone()),
                None,
                Some(explicit.join("config.toml")),
                explicit.clone(),
                explicit.join("data"),
            ),
            (
                "OMNESAGENT_DATA_DIR holding config.toml",
                root.join("inline-home"),
                None,
                Some(inline.clone()),
                Some(inline.join("config.toml")),
                inline.clone(),
                inline.join("data"),
            ),
            (
                "fresh OMNESAGENT_DATA_DIR named data",
                root.join("fresh-home"),
                None,
                Some(fresh.join("data")),
                None,
                fresh.join(".omnesagent"),
                fresh.join(".omnesagent/data"),
            ),
        ];

        let mut mismatches = Vec::new();
        for (layout, home, config_dir, data_dir, config_file, want_config, want_data) in cases {
            if let Some(config_file) = &config_file {
                fs::create_dir_all(config_file.parent().unwrap())
                    .await
                    .unwrap();
                fs::write(config_file, "schema_version = 3\n")
                    .await
                    .unwrap();
            }
            let _home_guard = EnvValueGuard::set("HOME", &home);
            let _config_guard = match &config_dir {
                Some(dir) => EnvValueGuard::set("OMNESAGENT_CONFIG_DIR", dir),
                None => EnvValueGuard::remove("OMNESAGENT_CONFIG_DIR"),
            };
            let _data_guard = match &data_dir {
                Some(dir) => EnvValueGuard::set("OMNESAGENT_DATA_DIR", dir),
                None => EnvValueGuard::remove("OMNESAGENT_DATA_DIR"),
            };

            let (locked_config, locked_data) = resolve_runtime_dirs().await.unwrap();
            let loaded = Box::pin(Config::load_or_init()).await.unwrap();
            if locked_data != loaded.data_dir {
                mismatches.push(format!(
                    "{layout}: locked {}, loaded {}",
                    locked_data.display(),
                    loaded.data_dir.display()
                ));
            }
            assert_eq!(locked_config, want_config, "{layout}: config dir");
            assert_eq!(
                loaded.data_dir, want_data,
                "{layout}: database placement moved"
            );
        }
        let _ = fs::remove_dir_all(&root).await;
        assert!(
            mismatches.is_empty(),
            "pre-lock and loaded data dirs disagree:\n{}",
            mismatches.join("\n")
        );
    }

    /// Startup creates and locks `config-lifecycle.lock` in the resolved data
    /// directory before it loads the config. A V2-to-V3 filesystem migration
    /// interrupted after moving an identity file, with the device database
    /// still in the legacy workspace, must still resume on that load: the
    /// database ends up readable in the loaded `data_dir`, and the legacy
    /// copy is moved out and kept in the migration backup.
    #[test]
    #[allow(clippy::large_futures)]
    async fn an_interrupted_v2_migration_resumes_despite_the_pre_load_lock() {
        let _env_guard = env_override_lock().await;
        let _workspace_guard = EnvValueGuard::remove("OMNESAGENT_WORKSPACE");

        let mut failures = Vec::new();
        for layout in [
            "default ~/.omnesagent",
            "container",
            "explicit --config-dir",
        ] {
            let temp = TempDir::new().unwrap();
            let home = temp.path();
            let (install, config_dir_env, data_dir_env) = match layout {
                "container" => (home.join(".omnesagent"), None, Some(home.join("data"))),
                "explicit --config-dir" => {
                    (home.join("explicit"), Some(home.join("explicit")), None)
                }
                _ => (home.join(".omnesagent"), None, None),
            };
            let legacy = install.join("workspace");
            std::fs::create_dir_all(&legacy).unwrap();
            std::fs::write(install.join("config.toml"), "schema_version = 2\n").unwrap();
            let agent = install.join("agents/default/workspace");
            std::fs::create_dir_all(&agent).unwrap();
            std::fs::write(agent.join("IDENTITY.md"), "moved before the interruption").unwrap();
            {
                let db = rusqlite::Connection::open(legacy.join("devices.db")).unwrap();
                db.execute_batch(
                    "CREATE TABLE marker(value TEXT NOT NULL);
                     INSERT INTO marker VALUES('existing device');",
                )
                .unwrap();
            }

            let _home_guard = EnvValueGuard::set("HOME", home);
            let _config_guard = match &config_dir_env {
                Some(dir) => EnvValueGuard::set("OMNESAGENT_CONFIG_DIR", dir),
                None => EnvValueGuard::remove("OMNESAGENT_CONFIG_DIR"),
            };
            let _data_guard = match &data_dir_env {
                Some(dir) => EnvValueGuard::set("OMNESAGENT_DATA_DIR", dir),
                None => EnvValueGuard::remove("OMNESAGENT_DATA_DIR"),
            };

            // The pre-load lock step: create the lifecycle lock where the
            // daemon would, and hold an advisory lock on it across the load.
            let (_, locked_data) = resolve_runtime_dirs().await.unwrap();
            std::fs::create_dir_all(&locked_data).unwrap();
            let lock = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(locked_data.join("config-lifecycle.lock"))
                .unwrap();
            lock.try_lock().unwrap();

            let loaded = Box::pin(Config::load_or_init()).await.unwrap();
            drop(lock);

            let moved = loaded.data_dir.join("devices.db");
            let marker: Option<String> = rusqlite::Connection::open_with_flags(
                &moved,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
            .and_then(|db| db.query_row("SELECT value FROM marker", [], |row| row.get(0)))
            .ok();
            let backed_up = std::fs::read_dir(&install).unwrap().flatten().any(|entry| {
                entry.file_name().to_string_lossy().starts_with("backup-")
                    && entry.path().join("legacy-workspace/devices.db").is_file()
            });
            if marker.as_deref() != Some("existing device")
                || legacy.join("devices.db").exists()
                || !backed_up
            {
                failures.push(format!(
                    "{layout}: locked {}, loaded {}; row at loaded data dir: {marker:?}; \
                     still in legacy workspace: {}; in migration backup: {backed_up}",
                    locked_data.display(),
                    loaded.data_dir.display(),
                    legacy.join("devices.db").exists()
                ));
            }
        }
        assert!(
            failures.is_empty(),
            "the device database was not migrated:\n{}",
            failures.join("\n")
        );
    }

    #[test]
    async fn classify_runtime_config_kind_uses_runtime_resolution_source() {
        let _env_guard = env_override_lock().await;
        #[cfg(unix)]
        let fake_home =
            PathBuf::from("/non-temp-omnesagent-test-home").join(uuid::Uuid::new_v4().to_string());
        #[cfg(not(unix))]
        let fake_home = UserDirs::new()
            .expect("user directories should be available")
            .home_dir()
            .to_path_buf();
        let explicit_config_dir = fake_home.join("explicit-config");

        #[cfg(unix)]
        let _home_guard = EnvValueGuard::set("HOME", &fake_home);
        #[cfg(not(unix))]
        let _home_guard = EnvValueGuard::remove("HOME");
        let _data_guard = EnvValueGuard::remove("OMNESAGENT_DATA_DIR");
        let _workspace_guard = EnvValueGuard::remove("OMNESAGENT_WORKSPACE");

        assert_eq!(
            classify_runtime_config_kind(&fake_home.join(".omnesagent").join("config.toml")).await,
            RuntimeConfigKind::Default
        );

        let _config_guard = EnvValueGuard::set("OMNESAGENT_CONFIG_DIR", &explicit_config_dir);
        assert_eq!(
            classify_runtime_config_kind(&explicit_config_dir.join("config.toml")).await,
            RuntimeConfigKind::Custom
        );
    }

    #[test]
    async fn classify_runtime_config_kind_reports_temporary_paths() {
        let tmp = tempfile::TempDir::new().unwrap();

        assert_eq!(
            classify_runtime_config_kind(&tmp.path().join("config.toml")).await,
            RuntimeConfigKind::Temporary
        );
    }

    async fn create_homebrew_prefix() -> TempDir {
        let prefix = TempDir::new().expect("homebrew prefix temp dir");
        fs::create_dir_all(prefix.path().join("Cellar"))
            .await
            .expect("create Cellar marker");
        prefix
    }

    #[test]
    async fn try_resolve_macos_homebrew_config_dir_detects_cellar_layout() {
        let prefix = create_homebrew_prefix().await;
        let exe = prefix
            .path()
            .join("Cellar")
            .join("omnesagent")
            .join("0.7.0")
            .join("bin")
            .join("omnesagent");

        let config_dir = try_resolve_macos_homebrew_config_dir(&exe)
            .await
            .expect("expected Homebrew layout");

        assert_eq!(config_dir, prefix.path().join("var").join("omnesagent"));
    }

    #[test]
    async fn try_resolve_macos_homebrew_config_dir_detects_prefix_bin_layout() {
        let prefix = create_homebrew_prefix().await;
        let exe = prefix.path().join("bin").join("omnesagent");

        let config_dir = try_resolve_macos_homebrew_config_dir(&exe)
            .await
            .expect("expected Homebrew layout");

        assert_eq!(config_dir, prefix.path().join("var").join("omnesagent"));
    }

    #[test]
    async fn try_resolve_macos_homebrew_config_dir_detects_opt_bin_layout() {
        let prefix = create_homebrew_prefix().await;
        let exe = prefix
            .path()
            .join("opt")
            .join("omnesagent")
            .join("bin")
            .join("omnesagent");

        let config_dir = try_resolve_macos_homebrew_config_dir(&exe)
            .await
            .expect("expected Homebrew layout");

        assert_eq!(config_dir, prefix.path().join("var").join("omnesagent"));
    }

    #[test]
    async fn try_resolve_macos_homebrew_config_dir_rejects_non_homebrew_layout() {
        let prefix = TempDir::new().expect("non-homebrew temp dir");
        let exe = prefix.path().join("bin").join("omnesagent");

        assert!(try_resolve_macos_homebrew_config_dir(&exe).await.is_none());
    }

    #[test]
    async fn default_path_under_config_dir_respects_omnesagent_config_dir() {
        let _env_guard = env_override_lock().await;
        let custom_dir = std::env::temp_dir().join("omnesagent-test-profile");
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("OMNESAGENT_CONFIG_DIR", &custom_dir) };

        let result = default_path_under_config_dir("knowledge.db");

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_CONFIG_DIR") };

        assert_eq!(
            result,
            custom_dir.join("knowledge.db").to_string_lossy().as_ref(),
            "expected path under OMNESAGENT_CONFIG_DIR, got: {result}"
        );
    }

    #[test]
    async fn load_or_init_seeds_runtime_proxy_from_config_file() {
        let _env_guard = env_override_lock().await;
        let config_dir = TempDir::new().unwrap();
        let _config_dir_guard = EnvValueGuard::set("OMNESAGENT_CONFIG_DIR", config_dir.path());
        fs::write(
            config_dir.path().join("config.toml"),
            r#"
schema_version = 3

[proxy]
enabled = true
http_proxy = "http://boot-proxy.example:3128"
scope = "omnesagent"
"#,
        )
        .await
        .unwrap();

        let loaded = Box::pin(Config::load_or_init()).await.unwrap();
        let runtime = runtime_proxy_config();

        assert_eq!(loaded.proxy.http_proxy, runtime.http_proxy);
        assert!(runtime.enabled);
        assert_eq!(
            runtime.http_proxy.as_deref(),
            Some("http://boot-proxy.example:3128")
        );

        set_runtime_proxy_config(ProxyConfig::default());
    }

    #[test]
    async fn load_or_init_workspace_override_uses_workspace_root_for_config() {
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let workspace_dir = temp_home.join("profile-a");

        let original_home = std::env::var("HOME").ok();
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("HOME", &temp_home) };
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("OMNESAGENT_WORKSPACE", &workspace_dir) };

        let config = Box::pin(Config::load_or_init()).await.unwrap();

        // V3 fresh init: `config.data_dir` lives at `<install>/data/`
        // (the shared databases root); the install root holds
        // `config.toml`. No synthesized `agents/default/workspace/` is
        // created at boot — `default` is migration-only, and per-agent
        // workspaces are created lazily at agent-loop entry.
        assert_eq!(config.data_dir, workspace_dir.join("data"));
        assert_eq!(config.config_path, workspace_dir.join("config.toml"));
        assert!(workspace_dir.join("config.toml").exists());
        assert!(
            !workspace_dir.join("agents").exists(),
            "fresh init must not create agents/ tree"
        );

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };
        if let Some(home) = original_home {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::remove_var("HOME") };
        }
        let _ = fs::remove_dir_all(temp_home).await;
    }

    #[test]
    async fn load_or_init_workspace_suffix_uses_legacy_config_layout() {
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let workspace_dir = temp_home.join("workspace");
        let legacy_config_dir = temp_home.join(".omnesagent");
        let legacy_config_path = legacy_config_dir.join("config.toml");

        let original_home = std::env::var("HOME").ok();
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("HOME", &temp_home) };
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("OMNESAGENT_WORKSPACE", &workspace_dir) };

        let config = Box::pin(Config::load_or_init()).await.unwrap();

        // V3: `config.data_dir` lives at `<install>/data/`. The
        // OMNESAGENT_WORKSPACE env var (deprecated alias) resolved to the
        // legacy config layout where the install root is the parent of
        // the env-var path; data sits at `<install>/data/`.
        assert_eq!(config.data_dir, legacy_config_dir.join("data"));
        assert_eq!(config.config_path, legacy_config_path);
        assert!(config.config_path.exists());

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };
        if let Some(home) = original_home {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::remove_var("HOME") };
        }
        let _ = fs::remove_dir_all(temp_home).await;
    }

    #[test]
    async fn load_or_init_workspace_override_keeps_existing_legacy_config() {
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let workspace_dir = temp_home.join("custom-workspace");
        let legacy_config_dir = temp_home.join(".omnesagent");
        let legacy_config_path = legacy_config_dir.join("config.toml");

        fs::create_dir_all(&legacy_config_dir).await.unwrap();
        fs::write(
            &legacy_config_path,
            r#"default_temperature = 0.7
default_model = "legacy-model"
"#,
        )
        .await
        .unwrap();

        let original_home = std::env::var("HOME").ok();
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("HOME", &temp_home) };
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("OMNESAGENT_WORKSPACE", &workspace_dir) };

        let config = Box::pin(Config::load_or_init()).await.unwrap();

        // V3: `config.data_dir` resolves to `<install>/data/` under
        // the install root (the directory holding the existing
        // `config.toml`), regardless of the OMNESAGENT_WORKSPACE
        // (deprecated) override.
        assert_eq!(config.data_dir, legacy_config_dir.join("data"));
        assert_eq!(config.config_path, legacy_config_path);
        assert_eq!(
            config
                .providers
                .models
                .find("openrouter", "default")
                .and_then(|e| e.model.as_deref()),
            Some("legacy-model")
        );

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };
        if let Some(home) = original_home {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::remove_var("HOME") };
        }
        let _ = fs::remove_dir_all(temp_home).await;
    }

    #[test]
    async fn load_or_init_decrypts_feishu_channel_secrets() {
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let config_dir = temp_home.join(".omnesagent");
        let config_path = config_dir.join("config.toml");

        fs::create_dir_all(&config_dir).await.unwrap();

        let original_home = std::env::var("HOME").ok();
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("HOME", &temp_home) };
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };

        let mut config = Config {
            config_path: config_path.clone(),
            data_dir: config_dir.join("workspace"),
            ..Default::default()
        };
        config.secrets.encrypt = true;
        config.channels.lark.insert(
            "feishu".to_string(),
            LarkConfig {
                enabled: true,
                app_id: "cli_feishu_123".into(),
                app_secret: "feishu-secret".into(),
                encrypt_key: Some("feishu-encrypt".into()),
                verification_token: Some("feishu-verify".into()),
                mention_only: false,
                use_feishu: true,
                receive_mode: LarkReceiveMode::Websocket,
                port: None,
                proxy_url: None,
                excluded_tools: vec![],
                approval_timeout_secs: 300,
                per_user_session: false,
                ack_reactions: None,
                stream_mode: StreamMode::default(),
                draft_update_interval_ms: default_draft_update_interval_ms(),
            },
        );
        config.save().await.unwrap();

        let loaded = Box::pin(Config::load_or_init()).await.unwrap();
        let feishu = loaded.channels.lark.get("feishu").unwrap();
        assert_eq!(feishu.app_secret, "feishu-secret");
        assert_eq!(feishu.encrypt_key.as_deref(), Some("feishu-encrypt"));
        assert_eq!(feishu.verification_token.as_deref(), Some("feishu-verify"));

        if let Some(home) = original_home {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::remove_var("HOME") };
        }
        let _ = fs::remove_dir_all(temp_home).await;
    }

    #[test]
    #[allow(clippy::large_futures)]
    async fn load_or_init_logs_existing_config_as_initialized() {
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let workspace_dir = temp_home.join("profile-a");
        let config_path = workspace_dir.join("config.toml");

        fs::create_dir_all(&workspace_dir).await.unwrap();
        fs::write(
            &config_path,
            r#"default_temperature = 0.7
default_model = "persisted-profile"
"#,
        )
        .await
        .unwrap();

        let original_home = std::env::var("HOME").ok();
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("HOME", &temp_home) };
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("OMNESAGENT_WORKSPACE", &workspace_dir) };

        let mut rx = capture_log_events();

        let config = Box::pin(Config::load_or_init()).await.unwrap();

        let logs = drain_captured(&mut rx);

        // V3: shared databases live at `<install>/data/`, per-agent
        // identity at `<install>/agents/<alias>/workspace/`. The
        // OMNESAGENT_WORKSPACE env var (deprecated alias for
        // OMNESAGENT_DATA_DIR) pinned the install root, so data_dir is
        // `<install>/data/` derived from the resolved root.
        assert_eq!(config.data_dir, workspace_dir.join("data"));
        assert_eq!(config.config_path, config_path);
        assert_eq!(
            config
                .providers
                .models
                .find("openrouter", "default")
                .and_then(|e| e.model.as_deref()),
            Some("persisted-profile")
        );
        assert!(logs.contains("Config loaded"), "{logs}");
        assert!(logs.contains("\"initialized\":true"), "{logs}");
        assert!(!logs.contains("\"initialized\":false"), "{logs}");

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };
        if let Some(home) = original_home {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::remove_var("HOME") };
        }
        let _ = fs::remove_dir_all(temp_home).await;
    }

    #[test]
    #[allow(clippy::large_futures)]
    async fn load_or_init_warns_for_current_and_legacy_wati_config() {
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let _home_guard = EnvValueGuard::set("HOME", &temp_home);
        let _config_guard = EnvValueGuard::remove("OMNESAGENT_CONFIG_DIR");
        let _data_guard = EnvValueGuard::remove("OMNESAGENT_DATA_DIR");

        let cases = [
            (
                "current",
                r#"schema_version = 3

[channels.wati.production]
enabled = true
api_token = "current-placeholder-token"
"#,
                "channels.wati",
                "current-placeholder-token",
            ),
            (
                "legacy",
                r#"[channels_config.wati]
enabled = true
api_token = "legacy-placeholder-token"
"#,
                "channels_config.wati",
                "legacy-placeholder-token",
            ),
        ];

        for (case, raw, expected_path, secret_value) in cases {
            let install = temp_home.join(case);
            fs::create_dir_all(&install).await.unwrap();
            fs::write(install.join("config.toml"), raw).await.unwrap();
            let _workspace_guard = EnvValueGuard::set("OMNESAGENT_WORKSPACE", &install);
            let mut rx = capture_log_events();

            let config = Box::pin(Config::load_or_init()).await.unwrap();
            let logs = drain_captured(&mut rx);

            assert!(
                config
                    .channels_by_alias()
                    .iter()
                    .all(|entry| entry.channel_type != "wati"),
                "retired WATI config must not re-enable a live channel"
            );
            assert_eq!(
                config.retired_wati_config_sections,
                vec![expected_path.to_string()],
                "load-time diagnostics must preserve the retired section path"
            );
            assert!(
                logs.contains("Retired WATI channel config section"),
                "missing WATI retirement warning for {case}: {logs}"
            );
            assert!(
                logs.contains(expected_path),
                "warning must name {expected_path}: {logs}"
            );
            assert!(
                !logs.contains(secret_value),
                "warning must never copy retired WATI credentials into logs: {logs}"
            );
        }

        let _ = fs::remove_dir_all(temp_home).await;
    }

    #[test]
    #[allow(clippy::large_futures)]
    async fn load_or_init_warns_for_retired_node_transport_without_logging_secret() {
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let install = temp_home.join("profile");
        fs::create_dir_all(&install).await.unwrap();
        fs::write(
            install.join("config.toml"),
            r#"schema_version = 3

[node_transport]
enabled = true
shared_secret = "retired-node-transport-sentinel"
"#,
        )
        .await
        .unwrap();

        let _home_guard = EnvValueGuard::set("HOME", &temp_home);
        let _workspace_guard = EnvValueGuard::set("OMNESAGENT_WORKSPACE", &install);
        let _config_guard = EnvValueGuard::remove("OMNESAGENT_CONFIG_DIR");
        let _data_guard = EnvValueGuard::remove("OMNESAGENT_DATA_DIR");
        let mut rx = capture_log_events();

        let config = Box::pin(Config::load_or_init()).await.unwrap();
        let logs = drain_captured(&mut rx);

        assert!(config.retired_node_transport_config);
        assert!(
            logs.contains("Retired `[node_transport]` config is ignored"),
            "missing retirement warning: {logs}"
        );
        assert!(
            logs.contains("\"retired_config\":\"node_transport\""),
            "warning must carry structured retirement attribution: {logs}"
        );
        assert!(
            !logs.contains("retired-node-transport-sentinel"),
            "warning must never copy the retired shared secret into logs: {logs}"
        );

        let _ = fs::remove_dir_all(temp_home).await;
    }

    #[test]
    #[allow(clippy::large_futures)]
    async fn load_or_init_assigns_degraded_security_for_malformed_section() {
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let workspace_dir = temp_home.join("profile-a");
        let config_path = workspace_dir.join("config.toml");

        fs::create_dir_all(&workspace_dir).await.unwrap();
        // `[security] audit` must be a table; a scalar forces the security
        // section to drop to its default on the resilient daemon path.
        fs::write(
            &config_path,
            r#"schema_version = 3
audit = "should-be-a-table-not-a-string"

[security]
audit = "should-be-a-table-not-a-string"
"#,
        )
        .await
        .unwrap();

        let original_home = std::env::var("HOME").ok();
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("HOME", &temp_home) };
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("OMNESAGENT_WORKSPACE", &workspace_dir) };

        let config = Box::pin(Config::load_or_init()).await.unwrap();

        assert!(
            config.degraded_security.iter().any(|s| s == "security"),
            "load_or_init must surface a dropped [security] section on degraded_security, got {:?}",
            config.degraded_security
        );

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };
        if let Some(home) = original_home {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::remove_var("HOME") };
        }
        let _ = fs::remove_dir_all(temp_home).await;
    }

    #[test]
    #[allow(clippy::large_futures)]
    async fn load_or_init_assigns_degraded_sections_for_malformed_channel_alias() {
        // Regression: `doctor` was blind to degraded_sections even though
        // load_or_init already populates it correctly. A [channels.telegram]
        // alias with a type-corrupt `bot_token` (not merely missing - see
        // the scenario where a missing bot_token must survive salvage instead of
        // being dropped) must be pruned (not fatal) and its path recorded on
        // `degraded_sections` so downstream diagnostics (omnesagent-runtime's
        // check_degraded_sections) can name it.
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let workspace_dir = temp_home.join("profile-a");
        let config_path = workspace_dir.join("config.toml");

        fs::create_dir_all(&workspace_dir).await.unwrap();
        fs::write(
            &config_path,
            r#"schema_version = 3

[channels.telegram.default]
enabled = true
bot_token = 42
"#,
        )
        .await
        .unwrap();

        let original_home = std::env::var("HOME").ok();
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("HOME", &temp_home) };
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("OMNESAGENT_WORKSPACE", &workspace_dir) };

        let config = Box::pin(Config::load_or_init()).await.unwrap();

        assert!(
            config
                .degraded_sections
                .iter()
                .any(|s| s == "channels.telegram.default"),
            "load_or_init must surface a dropped [channels.telegram.default] alias on \
             degraded_sections, got {:?}",
            config.degraded_sections
        );

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };
        if let Some(home) = original_home {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::remove_var("HOME") };
        }
        let _ = fs::remove_dir_all(temp_home).await;
    }

    #[test]
    async fn load_or_init_keeps_partial_channel_alias_out_of_degraded_sections() {
        // End-to-end companion to the salvage-layer tests: through the
        // real load_or_init entry point, a partial (tokenless) telegram alias
        // must load intact and must NOT be reported on degraded_sections.
        // Disabled so Config::validate() stays quiet about the missing token.
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let workspace_dir = temp_home.join("profile-a");
        let config_path = workspace_dir.join("config.toml");

        fs::create_dir_all(&workspace_dir).await.unwrap();
        fs::write(
            &config_path,
            r#"schema_version = 3

[channels.telegram.default]
enabled = false
"#,
        )
        .await
        .unwrap();

        let original_home = std::env::var("HOME").ok();
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("HOME", &temp_home) };
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("OMNESAGENT_WORKSPACE", &workspace_dir) };

        let config = Box::pin(Config::load_or_init()).await.unwrap();

        assert!(
            config.channels.telegram.contains_key("default"),
            "a partial (tokenless) alias must survive load_or_init, got {:?}",
            config.channels.telegram.keys().collect::<Vec<_>>()
        );
        assert!(
            config.degraded_sections.is_empty(),
            "a partial (tokenless) alias must not be reported as degraded, got {:?}",
            config.degraded_sections
        );

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };
        if let Some(home) = original_home {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::remove_var("HOME") };
        }
        let _ = fs::remove_dir_all(temp_home).await;
    }

    #[test]
    async fn salvage_reports_dropped_plugins_section_for_malformed_entries() {
        // `[plugins.entries]` written as a table instead of an array of
        // tables (`[[plugins.entries]]`) drops the whole [plugins] section
        // to defaults on the resilient path. That drop must land on
        // `ResilientLoad::dropped`; load_or_init copies it onto
        // `degraded_sections` so the CLI surfaces it on stderr instead of
        // the operator discovering `enabled = false` by accident.
        let raw = r#"schema_version = 3

[plugins]
enabled = true

[plugins.entries]
name = "weather-tool"
"#;
        let load = crate::migration::migrate_to_current_salvaged(raw);
        assert!(
            load.dropped.iter().any(|s| s == "plugins"),
            "a malformed [plugins] section must be reported on dropped, got {:?}",
            load.dropped
        );
        assert!(
            !load.config.plugins.enabled,
            "the malformed section must have been reset to defaults"
        );
    }

    #[test]
    #[allow(clippy::large_futures)]
    async fn load_or_init_marks_whole_config_degraded_for_unparseable_file() {
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let workspace_dir = temp_home.join("profile-a");
        let config_path = workspace_dir.join("config.toml");

        fs::create_dir_all(&workspace_dir).await.unwrap();
        // Not valid TOML at all: the whole config defaults, so every
        // security-critical section is lost at once. load_or_init must surface
        // that on degraded_security so the serving gate refuses to start.
        fs::write(&config_path, "this is not valid TOML {{{")
            .await
            .unwrap();

        let original_home = std::env::var("HOME").ok();
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("HOME", &temp_home) };
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("OMNESAGENT_WORKSPACE", &workspace_dir) };

        let config = Box::pin(Config::load_or_init()).await.unwrap();

        assert!(
            !config.degraded_security.is_empty(),
            "load_or_init must surface a whole-config loss on degraded_security, got {:?}",
            config.degraded_security
        );

        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };
        if let Some(home) = original_home {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: test-only, single-threaded test runner.
            unsafe { std::env::remove_var("HOME") };
        }
        let _ = fs::remove_dir_all(temp_home).await;
    }

    #[test]
    #[allow(clippy::large_futures)]
    async fn load_or_init_keeps_agents_with_object_form_delegates() {
        // Regression for the observed regression: a current
        // schema config containing an object-form delegate must not degrade and
        // drop the whole `agents` section.
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let workspace_dir = temp_home.join("profile-a");
        let config_path = workspace_dir.join("config.toml");

        fs::create_dir_all(&workspace_dir).await.unwrap();
        fs::write(
            &config_path,
            r#"schema_version = 3

[providers.models.ollama.default]

[risk_profiles.shared]

[runtime_profiles.default]

[agents.task_orchestrator]
model_provider = "ollama.default"
risk_profile = "shared"
runtime_profile = "default"
delegates = [
  "reviewer",
  { agent = "sysadmin", mode = "independent" },
]

[agents.reviewer]
model_provider = "ollama.default"
risk_profile = "shared"
runtime_profile = "default"

[agents.sysadmin]
model_provider = "ollama.default"
risk_profile = "shared"
runtime_profile = "default"
"#,
        )
        .await
        .unwrap();

        let original_home = std::env::var("HOME").ok();
        // SAFETY: test-only, guarded by env_override_lock.
        unsafe { std::env::set_var("HOME", &temp_home) };
        // SAFETY: test-only, guarded by env_override_lock.
        unsafe { std::env::set_var("OMNESAGENT_WORKSPACE", &workspace_dir) };

        let config = Box::pin(Config::load_or_init()).await.unwrap();

        assert!(
            config.degraded_security.is_empty(),
            "{:?}",
            config.degraded_security
        );
        assert!(config.agents.contains_key("task_orchestrator"));
        assert!(config.agents.contains_key("reviewer"));
        assert!(config.agents.contains_key("sysadmin"));
        assert_eq!(
            config.agents["task_orchestrator"].delegates,
            vec![
                DelegateTargetConfig::bounded("reviewer"),
                DelegateTargetConfig {
                    agent: "sysadmin".to_string(),
                    mode: DelegateExecutionMode::Independent,
                },
            ]
        );

        // SAFETY: test-only, guarded by env_override_lock.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };
        if let Some(home) = original_home {
            // SAFETY: test-only, guarded by env_override_lock.
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: test-only, guarded by env_override_lock.
            unsafe { std::env::remove_var("HOME") };
        }
        let _ = fs::remove_dir_all(temp_home).await;
    }

    #[test]
    #[allow(clippy::large_futures)]
    async fn load_or_init_migrates_agents_with_object_form_delegates() {
        // Same shape as above, but without schema_version so the migration path
        // proves it accepts mixed string/object delegates before validation.
        let _env_guard = env_override_lock().await;
        let temp_home =
            std::env::temp_dir().join(format!("omnesagent_test_home_{}", uuid::Uuid::new_v4()));
        let workspace_dir = temp_home.join("profile-a");
        let config_path = workspace_dir.join("config.toml");

        fs::create_dir_all(&workspace_dir).await.unwrap();
        fs::write(
            &config_path,
            r#"
[providers.models.ollama.default]

[risk_profiles.shared]

[runtime_profiles.default]

[agents.task_orchestrator]
model_provider = "ollama.default"
risk_profile = "shared"
runtime_profile = "default"
delegates = [
  "reviewer",
  { agent = "sysadmin", mode = "independent" },
]

[agents.reviewer]
model_provider = "ollama.default"
risk_profile = "shared"
runtime_profile = "default"

[agents.sysadmin]
model_provider = "ollama.default"
risk_profile = "shared"
runtime_profile = "default"
"#,
        )
        .await
        .unwrap();

        let original_home = std::env::var("HOME").ok();
        // SAFETY: test-only, guarded by env_override_lock.
        unsafe { std::env::set_var("HOME", &temp_home) };
        // SAFETY: test-only, guarded by env_override_lock.
        unsafe { std::env::set_var("OMNESAGENT_WORKSPACE", &workspace_dir) };

        let config = Box::pin(Config::load_or_init()).await.unwrap();

        assert!(config.agents.contains_key("task_orchestrator"));
        assert!(config.agents.contains_key("reviewer"));
        assert!(config.agents.contains_key("sysadmin"));
        assert_eq!(
            config.agents["task_orchestrator"].delegates,
            vec![
                DelegateTargetConfig::bounded("reviewer"),
                DelegateTargetConfig {
                    agent: "sysadmin".to_string(),
                    mode: DelegateExecutionMode::Independent,
                },
            ]
        );

        // SAFETY: test-only, guarded by env_override_lock.
        unsafe { std::env::remove_var("OMNESAGENT_WORKSPACE") };
        if let Some(home) = original_home {
            // SAFETY: test-only, guarded by env_override_lock.
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: test-only, guarded by env_override_lock.
            unsafe { std::env::remove_var("HOME") };
        }
        let _ = fs::remove_dir_all(temp_home).await;
    }

    #[test]
    async fn validate_rejects_out_of_range_temperature() {
        let mut config = Config::default();
        config.providers.models.openrouter.insert(
            "default".to_string(),
            OpenRouterModelProviderConfig {
                base: ModelProviderConfig {
                    api_key: Some("sk-test".into()),
                    temperature: Some(99.0),
                    ..Default::default()
                },
            },
        );
        let err = config.validate().unwrap_err();
        assert!(
            err.to_string().contains("temperature"),
            "expected temperature validation error, got: {err}"
        );
    }

    #[test]
    async fn validate_rejects_negative_temperature() {
        let mut config = Config::default();
        config.providers.models.openrouter.insert(
            "default".to_string(),
            OpenRouterModelProviderConfig {
                base: ModelProviderConfig {
                    api_key: Some("sk-test".into()),
                    temperature: Some(-0.5),
                    ..Default::default()
                },
            },
        );
        let err = config.validate().unwrap_err();
        assert!(
            err.to_string().contains("temperature"),
            "expected temperature validation error, got: {err}"
        );
    }

    #[test]
    async fn validate_accepts_valid_temperature() {
        let mut config = Config::default();
        config.providers.models.openrouter.insert(
            "default".to_string(),
            OpenRouterModelProviderConfig {
                base: ModelProviderConfig {
                    temperature: Some(0.7),
                    ..Default::default()
                },
            },
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    async fn validate_rejects_unknown_jira_actions() {
        for action in ["delete_ticket", "drop_database", ""] {
            let mut config = Config::default();
            config.jira.enabled = true;
            config.jira.base_url = "https://jira.example.test".into();
            config.jira.api_token = "token".into();
            config.jira.allowed_actions = vec![action.into()];

            let err = config
                .validate()
                .expect_err("unknown Jira action should be rejected")
                .to_string();
            assert!(
                err.contains("jira.allowed_actions contains unknown action"),
                "expected Jira allowed action error for {action:?}, got: {err}"
            );
        }
    }

    #[test]
    async fn validate_accepts_all_published_jira_actions() {
        for action in [
            "get_ticket",
            "search_tickets",
            "comment_ticket",
            "list_projects",
            "myself",
            "list_transitions",
            "transition_ticket",
            "create_ticket",
        ] {
            let mut config = Config::default();
            config.jira.enabled = true;
            config.jira.base_url = "https://jira.example.test".into();
            config.jira.api_token = "token".into();
            config.jira.allowed_actions = vec![action.into()];

            assert!(
                config.validate().is_ok(),
                "published Jira action {action:?} should validate"
            );
        }
    }

    #[test]
    async fn jira_email_empty_string_deserializes_as_none() {
        // Legacy configs round-tripped `email = ""` to disk because the
        // pre-rename `email: String` lacked `skip_serializing_if`. The
        // current `Option<String>` would otherwise deserialize `""` as
        // `Some("")`, and JiraTool would attempt Basic auth with empty
        // username (the dropped email-required validation no longer
        // catches this). Defense-in-depth: empty strings deserialize as
        // None.
        let toml_input = r#"
enabled = true
base_url = "https://jira.example.test"
email = ""
api_token = "tok"
"#;
        let cfg: JiraConfig = toml::from_str(toml_input).expect("parses with empty email");
        assert!(
            cfg.email.is_none(),
            "empty `email = \"\"` must deserialize as None, got {:?}",
            cfg.email
        );
        // Whitespace-only is also normalized to None.
        let toml_input_ws = r#"
enabled = true
base_url = "https://jira.example.test"
email = "   "
api_token = "tok"
"#;
        let cfg_ws: JiraConfig =
            toml::from_str(toml_input_ws).expect("parses with whitespace email");
        assert!(
            cfg_ws.email.is_none(),
            "whitespace-only email must deserialize as None, got {:?}",
            cfg_ws.email
        );
        // A real email still survives.
        let toml_input_real = r#"
enabled = true
base_url = "https://jira.example.test"
email = "ops@example.com"
api_token = "tok"
"#;
        let cfg_real: JiraConfig = toml::from_str(toml_input_real).expect("parses with real email");
        assert_eq!(
            cfg_real.email.as_deref(),
            Some("ops@example.com"),
            "non-empty email must round-trip unchanged"
        );
    }

    #[test]
    async fn proxy_config_scope_services_requires_entries_when_enabled() {
        let proxy = ProxyConfig {
            enabled: true,
            http_proxy: Some("http://127.0.0.1:7890".into()),
            https_proxy: None,
            all_proxy: None,
            no_proxy: Vec::new(),
            scope: ProxyScope::Services,
            services: Vec::new(),
        };

        let error = proxy.validate().unwrap_err().to_string();
        assert!(error.contains("proxy.scope='services'"));
    }

    #[test]
    async fn proxy_config_accepts_transcription_service_selectors() {
        let mut proxy = ProxyConfig {
            enabled: true,
            http_proxy: Some("http://127.0.0.1:7890".into()),
            scope: ProxyScope::Services,
            ..ProxyConfig::default()
        };

        for selector in [
            "transcription.groq",
            "transcription.openai",
            "transcription.deepgram",
            "transcription.assemblyai",
            "transcription.google",
            "transcription.local_whisper",
            "transcription.*",
        ] {
            proxy.services = vec![selector.to_string()];
            assert!(
                proxy.validate().is_ok(),
                "exact transcription selector `{selector}` should validate"
            );
        }
    }

    #[test]
    async fn collect_warnings_surfaces_dns_pinned_proxy_conflicts() {
        let proxy_url = Some("http://proxy.example:3128".to_string());

        for (scope, http_proxy) in [
            (ProxyScope::Environment, None),
            (ProxyScope::Environment, proxy_url.clone()),
            (ProxyScope::Omnesagent, proxy_url.clone()),
        ] {
            let config = Config {
                proxy: ProxyConfig {
                    enabled: true,
                    http_proxy,
                    scope,
                    ..ProxyConfig::default()
                },
                ..Config::default()
            };
            let warning = config
                .collect_warnings()
                .into_iter()
                .find(|warning| warning.code == "proxy_conflicts_with_dns_pinned_tools")
                .expect("broad proxy scopes must warn about DNS-pinned tools");
            assert_eq!(warning.path, "proxy.scope");
            assert!(warning.message.contains("http_request and web_fetch"));
            assert!(warning.message.contains("proxy.scope = \"services\""));
        }

        let services_config = |services: Vec<&str>| Config {
            proxy: ProxyConfig {
                enabled: true,
                http_proxy: proxy_url.clone(),
                scope: ProxyScope::Services,
                services: services.into_iter().map(ToOwned::to_owned).collect(),
                ..ProxyConfig::default()
            },
            ..Config::default()
        };

        let http_warning = services_config(vec!["tool.http_request"])
            .collect_warnings()
            .into_iter()
            .find(|warning| warning.code == "proxy_conflicts_with_dns_pinned_tools")
            .expect("the explicit http_request selector must warn");
        assert_eq!(http_warning.path, "proxy.services");
        assert!(http_warning.message.contains("tool calls (http_request)"));
        assert!(!http_warning.message.contains("tool calls (web_fetch)"));

        let file_download_warning = Config {
            file_download: FileDownloadConfig {
                url: Some("https://files.example.test/download".into()),
                ..FileDownloadConfig::default()
            },
            ..services_config(vec!["tool.file_download"])
        }
        .collect_warnings()
        .into_iter()
        .find(|warning| warning.code == "proxy_conflicts_with_dns_pinned_tools")
        .expect("the explicit file_download selector must warn when file_download is enabled");
        assert_eq!(file_download_warning.path, "proxy.services");
        assert!(
            file_download_warning
                .message
                .contains("tool calls (file_download)")
        );

        let wildcard_warning = services_config(vec!["tool.*"])
            .collect_warnings()
            .into_iter()
            .find(|warning| warning.code == "proxy_conflicts_with_dns_pinned_tools")
            .expect("the tool wildcard must warn");
        assert!(
            wildcard_warning
                .message
                .contains("http_request and web_fetch")
        );

        assert!(
            services_config(vec!["model_provider.openai"])
                .collect_warnings()
                .iter()
                .all(|warning| warning.code != "proxy_conflicts_with_dns_pinned_tools")
        );
    }

    #[test]
    async fn google_workspace_allowed_operations_require_methods() {
        let mut config = Config::default();
        config.google_workspace.allowed_operations = vec![GoogleWorkspaceAllowedOperation {
            service: "gmail".into(),
            resource: "users".into(),
            sub_resource: Some("drafts".into()),
            methods: Vec::new(),
        }];

        let err = config.validate().unwrap_err().to_string();
        assert!(err.contains("google_workspace.allowed_operations[0].methods"));
    }

    #[test]
    async fn google_workspace_allowed_operations_reject_duplicate_service_resource_sub_resource_entries()
     {
        let mut config = Config::default();
        config.google_workspace.allowed_operations = vec![
            GoogleWorkspaceAllowedOperation {
                service: "gmail".into(),
                resource: "users".into(),
                sub_resource: Some("drafts".into()),
                methods: vec!["create".into()],
            },
            GoogleWorkspaceAllowedOperation {
                service: "gmail".into(),
                resource: "users".into(),
                sub_resource: Some("drafts".into()),
                methods: vec!["update".into()],
            },
        ];

        let err = config.validate().unwrap_err().to_string();
        assert!(err.contains("duplicate service/resource/sub_resource entry"));
    }

    #[test]
    async fn google_workspace_allowed_operations_allow_same_resource_different_sub_resource() {
        let mut config = Config::default();
        config.google_workspace.allowed_operations = vec![
            GoogleWorkspaceAllowedOperation {
                service: "gmail".into(),
                resource: "users".into(),
                sub_resource: Some("messages".into()),
                methods: vec!["list".into(), "get".into()],
            },
            GoogleWorkspaceAllowedOperation {
                service: "gmail".into(),
                resource: "users".into(),
                sub_resource: Some("drafts".into()),
                methods: vec!["create".into(), "update".into()],
            },
        ];

        assert!(config.validate().is_ok());
    }

    #[test]
    async fn google_workspace_allowed_operations_reject_duplicate_methods_within_entry() {
        let mut config = Config::default();
        config.google_workspace.allowed_operations = vec![GoogleWorkspaceAllowedOperation {
            service: "gmail".into(),
            resource: "users".into(),
            sub_resource: Some("drafts".into()),
            methods: vec!["create".into(), "create".into()],
        }];

        let err = config.validate().unwrap_err().to_string();
        assert!(
            err.contains("duplicate entry"),
            "expected duplicate entry error, got: {err}"
        );
    }

    #[test]
    async fn google_workspace_allowed_operations_accept_valid_entries() {
        let mut config = Config::default();
        config.google_workspace.allowed_operations = vec![
            GoogleWorkspaceAllowedOperation {
                service: "gmail".into(),
                resource: "users".into(),
                sub_resource: Some("messages".into()),
                methods: vec!["list".into(), "get".into()],
            },
            GoogleWorkspaceAllowedOperation {
                service: "drive".into(),
                resource: "files".into(),
                sub_resource: None,
                methods: vec!["list".into(), "get".into()],
            },
        ];

        assert!(config.validate().is_ok());
    }

    #[test]
    async fn google_workspace_allowed_operations_accept_camelcase_entries() {
        // Google API resource/method identifiers are camelCase; the shipped
        // examples (calendarList, quickAdd, batchUpdate) must validate.
        let mut config = Config::default();
        config.google_workspace.allowed_operations = vec![
            GoogleWorkspaceAllowedOperation {
                service: "calendar".into(),
                resource: "calendarList".into(),
                sub_resource: None,
                methods: vec!["list".into(), "get".into()],
            },
            GoogleWorkspaceAllowedOperation {
                service: "calendar".into(),
                resource: "events".into(),
                sub_resource: None,
                methods: vec!["quickAdd".into()],
            },
            GoogleWorkspaceAllowedOperation {
                service: "gmail".into(),
                resource: "users".into(),
                sub_resource: Some("sendAs".into()),
                methods: vec!["list".into()],
            },
        ];

        assert!(config.validate().is_ok());
    }

    #[test]
    async fn google_workspace_allowed_operations_reject_invalid_sub_resource_characters() {
        let mut config = Config::default();
        config.google_workspace.allowed_operations = vec![GoogleWorkspaceAllowedOperation {
            service: "gmail".into(),
            resource: "users".into(),
            sub_resource: Some("bad resource!".into()),
            methods: vec!["list".into()],
        }];

        let err = config.validate().unwrap_err().to_string();
        assert!(err.contains("sub_resource contains invalid characters"));
    }

    fn runtime_proxy_cache_contains(cache_key: &str) -> bool {
        runtime_proxy_cached_client(cache_key).is_some()
    }

    #[test]
    async fn environment_proxy_detection_respects_scheme_and_no_proxy() {
        let values = HashMap::from([
            ("HTTP_PROXY", "http://http-proxy.example:3128"),
            ("HTTPS_PROXY", "http://https-proxy.example:3128"),
            ("ALL_PROXY", "socks5://all-proxy.example:1080"),
            ("NO_PROXY", "example.com,10.0.0.0/8,2001:db8::/32,localhost"),
        ]);
        let read = |key: &str| values.get(key).map(ToString::to_string);

        assert_eq!(
            environment_proxy_for_url_with("https://public.invalid/", read),
            Some("HTTPS_PROXY")
        );
        assert_eq!(
            environment_proxy_for_url_with("http://public.invalid/", read),
            Some("HTTP_PROXY")
        );
        for bypassed in [
            "https://api.example.com/",
            "http://10.2.3.4/",
            "https://[2001:db8::1]/",
            "http://localhost/",
        ] {
            assert_eq!(
                environment_proxy_for_url_with(bypassed, read),
                None,
                "NO_PROXY should bypass {bypassed}"
            );
        }
    }

    #[test]
    async fn runtime_proxy_client_cache_reuses_default_profile_key() {
        let _env_guard = env_override_lock().await;
        let service_key = format!(
            "model_provider.cache_test.{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        );
        let cache_key = runtime_proxy_cache_key(&service_key, None, None);

        clear_runtime_proxy_client_cache();
        assert!(!runtime_proxy_cache_contains(&cache_key));

        let _ = build_runtime_proxy_client(&service_key);
        assert!(runtime_proxy_cache_contains(&cache_key));

        let _ = build_runtime_proxy_client(&service_key);
        assert!(runtime_proxy_cache_contains(&cache_key));
    }

    #[test]
    async fn runtime_proxy_client_cache_reuses_read_timeout_profile_key() {
        let _env_guard = env_override_lock().await;
        let service_key = format!(
            "model_provider.cache_read_timeout_test.{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        );
        let cache_key = format!(
            "{}|timeout=none|connect_timeout=10|read_timeout=300",
            service_key.trim().to_ascii_lowercase(),
        );

        clear_runtime_proxy_client_cache();
        assert!(!runtime_proxy_cache_contains(&cache_key));

        let _ = build_runtime_proxy_client_with_read_timeout(&service_key, 300, 10);
        assert!(runtime_proxy_cache_contains(&cache_key));

        let _ = build_runtime_proxy_client_with_read_timeout(&service_key, 300, 10);
        assert!(runtime_proxy_cache_contains(&cache_key));
    }

    #[test]
    async fn runtime_proxy_client_cache_rejects_stale_generation_insert() {
        let _env_guard = env_override_lock().await;
        let service_key = format!(
            "model_provider.cache_generation_test.{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        );
        let cache_key = runtime_proxy_cache_key(&service_key, Some(30), Some(5));

        clear_runtime_proxy_client_cache();
        let stale_generation = runtime_proxy_config_generation();
        set_runtime_proxy_config(ProxyConfig {
            enabled: true,
            http_proxy: Some("http://reload.example:8080".to_string()),
            ..Default::default()
        });

        match runtime_proxy_client_cache().write() {
            Ok(mut guard) => {
                guard.insert(
                    cache_key.clone(),
                    RuntimeProxyCachedClient {
                        generation: stale_generation,
                        client: reqwest::Client::new(),
                    },
                );
            }
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                guard.insert(
                    cache_key.clone(),
                    RuntimeProxyCachedClient {
                        generation: stale_generation,
                        client: reqwest::Client::new(),
                    },
                );
            }
        }
        assert!(
            !runtime_proxy_cache_contains(&cache_key),
            "old-generation cache entries must not be visible after reload"
        );
        clear_runtime_proxy_client_cache();

        set_runtime_proxy_cached_client(
            cache_key.clone(),
            stale_generation,
            reqwest::Client::new(),
        );
        assert!(
            !runtime_proxy_cache_contains(&cache_key),
            "old-policy client builds must not repopulate the cache after reload"
        );

        set_runtime_proxy_cached_client(
            cache_key.clone(),
            runtime_proxy_config_generation(),
            reqwest::Client::new(),
        );
        assert!(runtime_proxy_cache_contains(&cache_key));

        set_runtime_proxy_config(ProxyConfig::default());
    }

    #[test]
    async fn proxy_reload_applies_new_config_through_rwlock() {
        let _env_guard = env_override_lock().await;
        set_runtime_proxy_config(ProxyConfig {
            enabled: true,
            http_proxy: Some("http://boot.example:3128".to_string()),
            ..Default::default()
        });
        assert_eq!(
            runtime_proxy_config().http_proxy.as_deref(),
            Some("http://boot.example:3128")
        );

        set_runtime_proxy_config(ProxyConfig {
            enabled: true,
            http_proxy: Some("http://reloaded.example:8080".to_string()),
            ..Default::default()
        });
        assert_eq!(
            runtime_proxy_config().http_proxy.as_deref(),
            Some("http://reloaded.example:8080"),
            "RwLock-backed runtime config must reflect the reloaded value"
        );

        set_runtime_proxy_config(ProxyConfig::default());
    }

    #[test]
    async fn set_runtime_proxy_config_clears_runtime_proxy_client_cache() {
        let _env_guard = env_override_lock().await;
        let service_key = format!(
            "model_provider.cache_timeout_test.{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        );
        let cache_key = runtime_proxy_cache_key(&service_key, Some(30), Some(5));

        clear_runtime_proxy_client_cache();
        let _ = build_runtime_proxy_client_with_timeouts(&service_key, 30, 5);
        assert!(runtime_proxy_cache_contains(&cache_key));

        set_runtime_proxy_config(ProxyConfig::default());
        assert!(!runtime_proxy_cache_contains(&cache_key));
    }

    #[test]
    async fn gateway_config_default_values() {
        let g = GatewayConfig::default();
        assert_eq!(g.port, 42617);
        assert_eq!(g.host, "127.0.0.1");
        assert!(g.require_pairing);
        assert!(!g.allow_public_bind);
        assert!(g.paired_tokens.is_empty());
        assert!(!g.trust_forwarded_headers);
        assert_eq!(g.rate_limit_max_keys, 10_000);
        assert_eq!(g.idempotency_max_keys, 10_000);
        assert_eq!(g.websocket_ping_interval_secs, 30);
    }

    // ── Peripherals config ───────────────────────────────────────

    #[test]
    async fn peripherals_config_default_disabled() {
        let p = PeripheralsConfig::default();
        assert!(!p.enabled);
        assert!(p.boards.is_empty());
    }

    #[test]
    async fn peripheral_board_config_defaults() {
        let b = PeripheralBoardConfig::default();
        assert!(b.board.is_empty());
        assert_eq!(b.transport, "serial");
        assert!(b.path.is_none());
        assert_eq!(b.baud, 115_200);
    }

    #[test]
    async fn peripherals_config_toml_roundtrip() {
        let p = PeripheralsConfig {
            enabled: true,
            boards: vec![PeripheralBoardConfig {
                board: "nucleo-f401re".into(),
                transport: "serial".into(),
                path: Some("/dev/ttyACM0".into()),
                baud: 115_200,
            }],
            datasheet_dir: None,
        };
        let toml_str = toml::to_string(&p).unwrap();
        let parsed: PeripheralsConfig = toml::from_str(&toml_str).unwrap();
        assert!(parsed.enabled);
        assert_eq!(parsed.boards.len(), 1);
        assert_eq!(parsed.boards[0].board, "nucleo-f401re");
        assert_eq!(parsed.boards[0].path.as_deref(), Some("/dev/ttyACM0"));
    }

    #[test]
    async fn lark_config_serde() {
        let lc = LarkConfig {
            enabled: true,
            app_id: "cli_123456".into(),
            app_secret: "secret_abc".into(),
            encrypt_key: Some("encrypt_key".into()),
            verification_token: Some("verify_token".into()),
            mention_only: false,
            use_feishu: true,
            receive_mode: LarkReceiveMode::Websocket,
            port: None,
            proxy_url: None,
            excluded_tools: vec![],
            approval_timeout_secs: 300,
            per_user_session: false,
            ack_reactions: None,
            stream_mode: StreamMode::default(),
            draft_update_interval_ms: default_draft_update_interval_ms(),
        };
        let json = serde_json::to_string(&lc).unwrap();
        let parsed: LarkConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.app_id, "cli_123456");
        assert_eq!(parsed.app_secret, "secret_abc");
        assert_eq!(parsed.encrypt_key.as_deref(), Some("encrypt_key"));
        assert_eq!(parsed.verification_token.as_deref(), Some("verify_token"));
        assert!(parsed.use_feishu);
    }

    #[test]
    async fn lark_config_toml_roundtrip() {
        let lc = LarkConfig {
            enabled: true,
            app_id: "cli_123456".into(),
            app_secret: "secret_abc".into(),
            encrypt_key: Some("encrypt_key".into()),
            verification_token: Some("verify_token".into()),
            mention_only: false,
            use_feishu: false,
            receive_mode: LarkReceiveMode::Webhook,
            port: Some(9898),
            proxy_url: None,
            excluded_tools: vec![],
            approval_timeout_secs: 300,
            per_user_session: false,
            ack_reactions: None,
            stream_mode: StreamMode::default(),
            draft_update_interval_ms: default_draft_update_interval_ms(),
        };
        let toml_str = toml::to_string(&lc).unwrap();
        let parsed: LarkConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.app_id, "cli_123456");
        assert_eq!(parsed.app_secret, "secret_abc");
        assert!(!parsed.use_feishu);
    }

    #[test]
    async fn lark_config_deserializes_without_optional_fields() {
        let json = r#"{"app_id":"cli_123","app_secret":"secret"}"#;
        let parsed: LarkConfig = serde_json::from_str(json).unwrap();
        assert!(parsed.encrypt_key.is_none());
        assert!(parsed.verification_token.is_none());
        assert!(!parsed.mention_only);
        assert!(!parsed.use_feishu);
    }

    #[test]
    async fn lark_config_defaults_to_lark_endpoint() {
        let json = r#"{"app_id":"cli_123","app_secret":"secret"}"#;
        let parsed: LarkConfig = serde_json::from_str(json).unwrap();
        assert!(
            !parsed.use_feishu,
            "use_feishu should default to false (Lark)"
        );
    }

    #[test]
    async fn lark_v2_allowed_users_fold_into_peer_groups() {
        // V2 `allowed_users` on a Lark channel migrates to a synthesized
        // `peer_groups.lark_default` group. The wildcard `*` is dropped at
        // synthesis (operator-explicit lists only); concrete user IDs
        // round-trip through.
        let raw = r#"
schema_version = 2

[channels.lark]
enabled = true
app_id = "cli_123"
app_secret = "secret"
allowed_users = ["user_alpha", "user_beta"]
"#;
        let parsed = crate::migration::migrate_to_current(raw).expect("migration succeeds");
        let group = parsed
            .peer_groups
            .get("lark_default")
            .expect("V2 lark.allowed_users must fold into peer_groups.lark_default");
        assert_eq!(group.channel, "lark");
        let usernames: Vec<&str> = group.external_peers.iter().map(|p| p.as_str()).collect();
        assert_eq!(usernames, vec!["user_alpha", "user_beta"]);
    }

    // ── LINE ──────────────────────────────────────────────────

    #[test]
    async fn line_config_toml_roundtrip() {
        // Full [channels.line] TOML block — covers every user-facing field.
        //
        // channel_access_token and channel_secret can be omitted here and
        // supplied via LINE_CHANNEL_ACCESS_TOKEN / LINE_CHANNEL_SECRET env vars
        // instead; both fields default to "" when absent.
        let toml = r#"
[channels_config.line.default]
enabled = true
channel_access_token = "ChannelAccessToken=="
channel_secret = "abc123secret"
dm_policy = "pairing"
group_policy = "mention"
allowed_users = []
webhook_port = 8443
sender_name = "Popcorn"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let ln = config.channels.line.get("default").unwrap();
        assert_eq!(ln.channel_access_token, "ChannelAccessToken==");
        assert_eq!(ln.channel_secret, "abc123secret");
        assert_eq!(ln.dm_policy, LineDmPolicy::Pairing);
        assert_eq!(ln.group_policy, LineGroupPolicy::Mention);
        assert_eq!(ln.webhook_port, 8443);
        assert!(ln.proxy_url.is_none());
        assert_eq!(ln.sender_name.as_deref(), Some("Popcorn"));
    }

    #[test]
    async fn line_config_defaults() {
        // Minimal config — only the required secret fields are provided.
        // All optional fields should resolve to documented defaults.
        let toml = r#"
[channels_config.line.default]
channel_access_token = "tok"
channel_secret = "sec"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let ln = config.channels.line.get("default").unwrap();
        assert_eq!(
            ln.dm_policy,
            LineDmPolicy::Pairing,
            "dm_policy default is pairing"
        );
        assert_eq!(
            ln.group_policy,
            LineGroupPolicy::Mention,
            "group_policy default is mention"
        );
        assert_eq!(ln.webhook_port, 8443, "webhook_port default is 8443");
        assert!(ln.proxy_url.is_none());
        assert!(ln.sender_name.is_none(), "sender_name default is None");
    }

    #[test]
    async fn line_config_allowlist_policy() {
        // dm_policy = allowlist; the user ID list itself now lives on the
        // V3 `peer_groups.line_default` group (synthesized from V2's
        // `allowed_users`), not on the LineConfig struct.
        let toml = r#"
schema_version = 2

[channels.line]
enabled = true
channel_access_token = "tok"
channel_secret = "sec"
dm_policy = "allowlist"
allowed_users = ["Uabc123", "Udef456"]
"#;
        let config = crate::migration::migrate_to_current(toml).expect("migration succeeds");
        let ln = config.channels.line.get("default").unwrap();
        assert_eq!(ln.dm_policy, LineDmPolicy::Allowlist);
        let group = config
            .peer_groups
            .get("line_default")
            .expect("V2 line.allowed_users must fold into peer_groups.line_default");
        let usernames: Vec<&str> = group.external_peers.iter().map(|p| p.as_str()).collect();
        assert_eq!(usernames, vec!["Uabc123", "Udef456"]);
    }

    #[test]
    async fn line_config_open_policies() {
        // dm_policy = open + group_policy = open — most permissive combination.
        let toml = r#"
[channels_config.line.default]
channel_access_token = "tok"
channel_secret = "sec"
dm_policy = "open"
group_policy = "open"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let ln = config.channels.line.get("default").unwrap();
        assert_eq!(ln.dm_policy, LineDmPolicy::Open);
        assert_eq!(ln.group_policy, LineGroupPolicy::Open);
    }

    #[test]
    async fn line_config_group_disabled() {
        // group_policy = disabled — bot ignores all group/room messages.
        let toml = r#"
[channels_config.line.default]
channel_access_token = "tok"
channel_secret = "sec"
group_policy = "disabled"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        let ln = config.channels.line.get("default").unwrap();
        assert_eq!(ln.group_policy, LineGroupPolicy::Disabled);
    }

    #[test]
    async fn nextcloud_talk_config_serde() {
        let nc = NextcloudTalkConfig {
            enabled: true,
            base_url: "https://cloud.example.com".into(),
            app_token: None,
            bot_token: None,
            webhook_secret: Some("webhook-secret".into()),
            proxy_url: None,
            bot_name: None,
            excluded_tools: vec![],
            stream_mode: StreamMode::default(),
            draft_update_interval_ms: 1000,
        };

        let json = serde_json::to_string(&nc).unwrap();
        let parsed: NextcloudTalkConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.base_url, "https://cloud.example.com");
        assert!(parsed.app_token.is_none());
        assert_eq!(parsed.webhook_secret.as_deref(), Some("webhook-secret"));
    }

    #[test]
    async fn nextcloud_talk_config_defaults_optional_fields() {
        let json = r#"{"base_url":"https://cloud.example.com"}"#;
        let parsed: NextcloudTalkConfig = serde_json::from_str(json).unwrap();
        assert!(parsed.webhook_secret.is_none());
        assert!(parsed.app_token.is_none());
    }

    #[test]
    async fn nextcloud_talk_config_accepts_legacy_app_token_without_using_it() {
        // Pre-existing configs may still set the now-deprecated app_token;
        // it must parse (not fail-closed the whole config) and just be
        // carried as a no-op field.
        let json = r#"{"base_url":"https://cloud.example.com","app_token":"legacy-value"}"#;
        let parsed: NextcloudTalkConfig = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.app_token.as_deref(), Some("legacy-value"));
    }

    // ── Config file permission hardening (Unix only) ───────────────

    #[cfg(unix)]
    #[test]
    async fn new_config_file_has_restricted_permissions() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Create a config and save it
        let config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.save().await.unwrap();

        let meta = fs::metadata(&config_path).await.unwrap();
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "New config file should be owner-only (0600), got {mode:o}"
        );
    }

    #[cfg(unix)]
    #[test]
    async fn save_restricts_existing_world_readable_config_to_owner_only() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.save().await.unwrap();

        // Simulate the regression state observed in issue.
        std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let loose_mode = std::fs::metadata(&config_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(
            loose_mode, 0o644,
            "test setup requires world-readable config"
        );

        if let Some(entry) = config.providers.models.ensure("openrouter", "default") {
            entry.temperature = Some(0.6);
        }
        config.save().await.unwrap();

        let hardened_mode = std::fs::metadata(&config_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(
            hardened_mode, 0o600,
            "Saving config should restore owner-only permissions (0600)"
        );
    }

    #[test]
    async fn save_dirty_stamps_current_schema_version_on_stale_label() {
        // Regression for. An incremental save writes current-schema-shaped
        // sections, but `schema_version` is never a dirty path. Without an
        // explicit stamp, a file first written by an older binary keeps its
        // stale `schema_version` label while gaining a current-schema body — a
        // state that crashes older binaries with `missing field ...`.
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Seed an on-disk file labeled with a stale schema version so the
        // incremental path (not the new-file fallback to full `save`) runs.
        std::fs::write(
            &config_path,
            "schema_version = 2\n\n[observability]\nbackend = \"none\"\n",
        )
        .unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.observability.backend = ObservabilityBackend::Otel;
        config.mark_dirty("observability.backend");
        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            written.contains(&format!(
                "schema_version = {}",
                crate::migration::CURRENT_SCHEMA_VERSION
            )),
            "save_dirty must stamp the current schema_version; got:\n{written}"
        );
        assert!(
            !written.contains("schema_version = 2"),
            "stale schema_version label must be overwritten; got:\n{written}"
        );
        // The dirty value still lands, and the stamp sits at the top of the file.
        assert!(
            written.contains("backend = \"otel\""),
            "dirty value must still be written; got:\n{written}"
        );
        assert!(
            written.trim_start().starts_with("schema_version ="),
            "schema_version should remain the first key; got:\n{written}"
        );
    }

    /// Regression for the per-field `[[mcp.servers]]` editor: after
    /// `d06ed25` shipped the natural-key arm, in-memory edits succeed
    /// (the TUI / dashboard show the new value) but `save_dirty` is
    /// silently a no-op because `apply_dirty_path` walks the serialized
    /// TOML as if every segment is a `Table` — `mcp.servers` is an
    /// array of tables, so `lookup_path_in_table` returns `None` at the
    /// natural-key segment, the path is misclassified as
    /// `should_delete`, and `delete_path_in_doc` bails when it hits the
    /// array too. Net effect: the on-disk file keeps its stale value.
    #[test]
    async fn save_dirty_persists_mcp_server_field_via_natural_key() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Seed an on-disk file with a single MCP server so the
        // incremental path (not the new-file fallback to full `save`)
        // runs. Schema version is stamped to current so the writer
        // doesn't have to migrate anything.
        let seed = format!(
            "schema_version = {}\n\n\
             [[mcp.servers]]\n\
             name = \"fs\"\n\
             transport = \"stdio\"\n\
             command = \"/usr/bin/mcp-fs\"\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        // Build the in-memory config to match the seeded file. We
        // don't need to round-trip through deserialization — the bug
        // is purely on the save side, and the on-disk seed gives
        // `save_dirty` an existing file to do an incremental write
        // into (the new-file fallback to full `save` would mask the
        // dirty-path bug because it serializes the whole struct).
        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.mcp.servers.push(McpServerConfig {
            name: "fs".into(),
            transport: McpTransport::Stdio,
            command: "/usr/bin/mcp-fs".into(),
            ..Default::default()
        });
        assert_eq!(config.mcp.servers[0].command, "/usr/bin/mcp-fs");

        // The same call site the dashboard / TUI use: set_prop_persistent
        // on a natural-key-routed inner path, then flush via save_dirty.
        config
            .set_prop_persistent("mcp.servers.fs.command", "/usr/local/bin/mcp-fs")
            .expect("set_prop_persistent must route through the natural-key arm");
        // The in-memory mutation must land — this is what the UI sees.
        assert_eq!(config.mcp.servers[0].command, "/usr/local/bin/mcp-fs");

        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            written.contains("/usr/local/bin/mcp-fs"),
            "save_dirty must write the new command for `mcp.servers.fs.command`; \
             on-disk file still reads:\n{written}"
        );
        assert!(
            !written.contains("/usr/bin/mcp-fs"),
            "stale command must be overwritten; got:\n{written}"
        );
        // The natural-key field itself must stay on disk — losing
        // `name` would orphan every other field in the [[mcp.servers]]
        // entry and break subsequent loads.
        assert!(
            written.contains("name = \"fs\""),
            "natural-key `name` must survive the incremental save; got:\n{written}"
        );
    }

    /// `cost.rates.providers.models.<type>` is a
    /// `#[resource_key]` `HashMap<String, ModelCostRates>` — its key is a
    /// model id, not an operator-chosen alias, and may contain dots
    /// (`gpt-4.1`). Before the map-key-section branch landed,
    /// `apply_dirty_path` blindly split the dirty path on `.`, fragmenting
    /// the key into `gpt-4` + `1`, finding neither in the in-memory table,
    /// and silently deleting (no-op-ing) the write instead of persisting
    /// it.
    #[test]
    async fn save_dirty_persists_dotted_map_key_field() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let seed = format!(
            "schema_version = {}\n\n\
             [cost.rates.providers.models.openai.\"gpt-4.1\"]\n\
             input_per_mtok = 1.0\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.cost.rates.providers.models.openai.insert(
            "gpt-4.1".to_string(),
            ModelCostRates {
                input_per_mtok: Some(1.0),
                ..Default::default()
            },
        );

        config
            .set_prop_persistent(
                "cost.rates.providers.models.openai.gpt-4.1.input_per_mtok",
                "9.9",
            )
            .expect("set_prop_persistent must route through the dotted resource key");
        assert_eq!(
            config
                .cost
                .rates
                .providers
                .models
                .openai
                .get("gpt-4.1")
                .and_then(|r| r.input_per_mtok),
            Some(9.9)
        );

        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            written.contains("9.9"),
            "save_dirty must write the new input_per_mtok for the dotted model key; \
             on-disk file still reads:\n{written}"
        );
        assert!(
            written.contains("\"gpt-4.1\""),
            "the dotted key must survive as one quoted TOML key, not be split apart; got:\n{written}"
        );

        let reloaded: Config = toml::from_str(&written)
            .unwrap_or_else(|e| panic!("rewritten config must reparse: {e}\n---\n{written}"));
        assert_eq!(
            reloaded
                .cost
                .rates
                .providers
                .models
                .openai
                .get("gpt-4.1")
                .and_then(|r| r.input_per_mtok),
            Some(9.9),
            "reloaded config must see the persisted value; got:\n{written}"
        );
    }

    #[test]
    async fn set_prop_persistent_rejects_ambiguous_dotted_map_key_path_before_mutation() {
        let mut config = Config::default();
        config.cost.rates.providers.models.openai.insert(
            "gpt-4.1".to_string(),
            ModelCostRates {
                input_per_mtok: Some(1.0),
                ..Default::default()
            },
        );
        config.cost.rates.providers.models.openai.insert(
            "gpt-4.1.input_per_mtok".to_string(),
            ModelCostRates {
                input_per_mtok: Some(2.0),
                ..Default::default()
            },
        );

        let path = "cost.rates.providers.models.openai.gpt-4.1.input_per_mtok";
        let err = config
            .set_prop_persistent(path, "9.9")
            .expect_err("colliding whole-key and field interpretations must be rejected");
        let message = err.to_string();
        assert!(
            message.contains("cost.rates.providers.models.openai[\"gpt-4.1.input_per_mtok\"]"),
            "error must name the whole-entry interpretation: {message}"
        );
        assert!(
            message.contains("cost.rates.providers.models.openai[\"gpt-4.1\"].input_per_mtok"),
            "error must name the field interpretation: {message}"
        );
        assert_eq!(
            config
                .cost
                .rates
                .providers
                .models
                .openai
                .get("gpt-4.1")
                .and_then(|rates| rates.input_per_mtok),
            Some(1.0),
            "rejection must happen before the shorter key's field is mutated"
        );
        assert_eq!(
            config
                .cost
                .rates
                .providers
                .models
                .openai
                .get("gpt-4.1.input_per_mtok")
                .and_then(|rates| rates.input_per_mtok),
            Some(2.0),
            "rejection must not mutate the exact-key entry"
        );
        assert!(
            !config.dirty_paths.contains(path),
            "a rejected path must not be recorded as persistable"
        );
    }

    /// Control for `save_dirty_persists_dotted_map_key_field`: a dot-free
    /// resource key must keep working through the same map-key-section
    /// branch (it's no longer special-cased out — every `HashMap<String,
    /// T>` write now routes through `apply_dirty_map_key_path`).
    #[test]
    async fn save_dirty_persists_dot_free_map_key_field() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let seed = format!(
            "schema_version = {}\n\n\
             [cost.rates.providers.models.openai.gpt-4o]\n\
             input_per_mtok = 1.0\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.cost.rates.providers.models.openai.insert(
            "gpt-4o".to_string(),
            ModelCostRates {
                input_per_mtok: Some(1.0),
                ..Default::default()
            },
        );

        config
            .set_prop_persistent(
                "cost.rates.providers.models.openai.gpt-4o.input_per_mtok",
                "5.5",
            )
            .unwrap();
        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            written.contains("5.5"),
            "dot-free map key writes must still persist; got:\n{written}"
        );
    }

    /// Delete path: removing a dotted map key in memory
    /// (`delete_map_key`) must still drop the matching on-disk table.
    /// Before this fix the on-disk key was never located because the
    /// dirty path (`<section>.<key>`, no inner suffix) is exactly the
    /// shape the naive `raw.split('.')` walker mis-parses.
    #[test]
    async fn save_dirty_removes_dotted_map_key_entry() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let seed = format!(
            "schema_version = {}\n\n\
             [cost.rates.providers.models.openai.\"gpt-4.1\"]\n\
             input_per_mtok = 1.0\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.cost.rates.providers.models.openai.insert(
            "gpt-4.1".to_string(),
            ModelCostRates {
                input_per_mtok: Some(1.0),
                ..Default::default()
            },
        );

        let removed = config
            .delete_map_key("cost.rates.providers.models.openai", "gpt-4.1")
            .expect("delete_map_key must accept the dotted resource key");
        assert!(removed);
        config.mark_dirty("cost.rates.providers.models.openai.gpt-4.1");

        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            !written.contains("gpt-4.1"),
            "deleted dotted map key must be dropped from disk; got:\n{written}"
        );
    }

    /// OmnesAgent never writes inline tables but loads hand-edited ones
    /// fine, so a map-key section shaped `openai = { "gpt-4.1" = { ... } }`
    /// parses as `Item::Value(Value::InlineTable)` — invisible to a
    /// `Table`-only doc walk. Both halves must go through `TableLike`:
    /// resolving the key (read side) so the batch doesn't abort, and
    /// actually removing it from the inline table (write side) so the
    /// deletion isn't reported as successful while the key survives on
    /// disk — a mutable traversal that only understood `Table` would
    /// resolve the key, then silently return without deleting anything.
    #[test]
    async fn save_dirty_resolves_map_key_from_inline_table_on_disk() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let seed = format!(
            "schema_version = {}\n\n\
             [cost.rates.providers.models]\n\
             openai = {{ \"gpt-4.1\" = {{ input_per_mtok = 1.0 }} }}\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        // Key absent from memory (the delete half): resolution can only
        // come from the on-disk doc, i.e. through the inline table.
        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.mark_dirty("cost.rates.providers.models.openai.gpt-4.1");

        config.save_dirty().await.expect(
            "a key living only in an on-disk inline table must resolve, not abort the save",
        );

        let written = std::fs::read_to_string(&config_path).unwrap();
        let doc = written
            .parse::<toml_edit::DocumentMut>()
            .unwrap_or_else(|e| panic!("rewritten config must reparse: {e}\n---\n{written}"));

        // The delete must actually take effect on disk, not just resolve
        // and then no-op: reporting `Ok(())` while the key survives is
        // the silent-persistence failure this section of `save_dirty`
        // exists to eliminate.
        assert!(
            !written.contains("gpt-4.1"),
            "deleted key must not remain anywhere in the rewritten file; got:\n{written}"
        );
        let openai_item = doc
            .get("cost")
            .and_then(|i| i.get("rates"))
            .and_then(|i| i.get("providers"))
            .and_then(|i| i.get("models"))
            .and_then(|i| i.get("openai"))
            .expect("openai entry must survive the delete of its only sub-key");
        let openai_table = openai_item.as_table_like().expect(
            "openai entry must still be table-like (Table or InlineTable) after the delete",
        );
        assert!(
            !openai_table.contains_key("gpt-4.1"),
            "gpt-4.1 must be removed from the on-disk openai inline table; got:\n{written}"
        );
    }

    /// Write half of the inline-table fix: a value living inside a
    /// hand-edited inline table must be updated in place, not just
    /// resolved-and-ignored. Same doc shape as
    /// `save_dirty_resolves_map_key_from_inline_table_on_disk`, but the
    /// key stays in memory with a changed value instead of being dropped.
    #[test]
    async fn save_dirty_persists_write_into_inline_table_on_disk() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let seed = format!(
            "schema_version = {}\n\n\
             [cost.rates.providers.models]\n\
             openai = {{ \"gpt-4.1\" = {{ input_per_mtok = 1.0 }} }}\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.cost.rates.providers.models.openai.insert(
            "gpt-4.1".to_string(),
            ModelCostRates {
                input_per_mtok: Some(1.0),
                ..Default::default()
            },
        );
        config
            .set_prop_persistent(
                "cost.rates.providers.models.openai.gpt-4.1.input_per_mtok",
                "9.9",
            )
            .expect("set_prop_persistent must route through the dotted resource key");

        config.save_dirty().await.expect(
            "a write into a key resolved through an on-disk inline table must not abort the save",
        );

        let written = std::fs::read_to_string(&config_path).unwrap();
        let doc = written
            .parse::<toml_edit::DocumentMut>()
            .unwrap_or_else(|e| panic!("rewritten config must reparse: {e}\n---\n{written}"));

        assert!(
            written.contains("9.9"),
            "save_dirty must write the new input_per_mtok into the inline table; got:\n{written}"
        );
        let rates_item = doc
            .get("cost")
            .and_then(|i| i.get("rates"))
            .and_then(|i| i.get("providers"))
            .and_then(|i| i.get("models"))
            .and_then(|i| i.get("openai"))
            .and_then(|i| i.get("gpt-4.1"))
            .and_then(|i| i.get("input_per_mtok"))
            .expect("input_per_mtok must survive as a leaf inside the on-disk inline table");
        assert_eq!(
            rates_item.as_float(),
            Some(9.9),
            "the on-disk inline-table leaf must reflect the written value; got:\n{written}"
        );
    }

    /// Loud-failure guard: a dirty path that resolves to a
    /// map-key section but whose key exists in neither the in-memory
    /// config nor the on-disk doc must fail `save_dirty` instead of
    /// silently no-op-ing (the original bug's symptom — reported success,
    /// nothing written).
    #[test]
    async fn save_dirty_errors_on_unresolvable_map_key_path() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let seed = format!(
            "schema_version = {}\n\n\
             [observability]\n\
             backend = \"none\"\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        // Neither `full_table` nor the on-disk doc has a
        // `cost.rates.providers.models.openai.ghost-model` entry — mark it
        // dirty directly the way a stale/duplicate `mark_dirty` call
        // (e.g. a bug elsewhere, or a manually crafted RPC) would.
        config.mark_dirty("cost.rates.providers.models.openai.ghost-model.input_per_mtok");

        let err = config
            .save_dirty()
            .await
            .expect_err("an unresolvable map-key dirty path must fail loudly, not no-op");
        let msg = err.to_string();
        assert!(
            msg.contains("cost.rates.providers.models.openai.ghost-model.input_per_mtok"),
            "error must name the offending dirty path; got: {msg}"
        );
    }

    /// A dynamic secret map (`#[secret] HashMap<String, String>`, e.g.
    /// `extra_headers`) treats its whole non-empty suffix as ONE opaque
    /// key — `set_prop` stores the literal `"X-Foo.bar"`. The dirty
    /// writer must not re-split that key at dots into `X-Foo` → `bar`,
    /// or `write_or_delete_leaf` finds nothing, takes the delete branch,
    /// and `save_dirty` returns `Ok(())` having written nothing: the
    /// operator sees success and the value is gone at next reload.
    #[test]
    async fn save_dirty_writes_dotted_dynamic_secret_map_key_as_one_segment() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let seed = format!(
            "schema_version = {}\n\n\
             [providers.models.openai.fresh]\n\
             model = \"gpt-4o\"\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.secrets.encrypt = false;
        config
            .create_map_key("providers.models.openai", "fresh")
            .expect("seed the alias");

        let path = "providers.models.openai.fresh.extra_headers.X-Foo.bar";
        config
            .set_prop_persistent(path, "header-value")
            .expect("a dotted dynamic secret-map key must be insertable");
        assert_eq!(
            config
                .providers
                .models
                .openai
                .get("fresh")
                .and_then(|p| p.base.extra_headers.get("X-Foo.bar"))
                .map(String::as_str),
            Some("header-value"),
            "precondition: set_prop stores ONE literal HashMap key"
        );

        config.save_dirty().await.expect("save_dirty must succeed");

        let written = std::fs::read_to_string(&config_path).unwrap();
        let reloaded: Config = toml::from_str(&written)
            .unwrap_or_else(|e| panic!("rewritten config must reparse: {e}\n---\n{written}"));
        assert_eq!(
            reloaded
                .providers
                .models
                .openai
                .get("fresh")
                .and_then(|p| p.base.extra_headers.get("X-Foo.bar"))
                .map(String::as_str),
            Some("header-value"),
            "a dotted dynamic secret-map key must round-trip through disk as the \
             literal key `set_prop` stored, not be split at dots and dropped; got:\n{written}"
        );
    }

    /// Unset half of the same contract: clearing a dotted dynamic
    /// secret-map key must remove the literal on-disk entry rather than
    /// hunting for a nested `bar` table that never existed.
    #[test]
    async fn save_dirty_removes_dotted_dynamic_secret_map_key_as_one_segment() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let seed = format!(
            "schema_version = {}\n\n\
             [providers.models.openai.fresh]\n\
             model = \"gpt-4o\"\n\n\
             [providers.models.openai.fresh.extra_headers]\n\
             \"X-Foo.bar\" = \"header-value\"\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config: Config = toml::from_str(&seed).unwrap();
        config.config_path = config_path.clone();
        config.secrets.encrypt = false;
        assert!(
            config
                .providers
                .models
                .openai
                .get("fresh")
                .is_some_and(|p| p.base.extra_headers.contains_key("X-Foo.bar")),
            "precondition: seeded dotted header key must load"
        );

        config
            .providers
            .models
            .openai
            .get_mut("fresh")
            .unwrap()
            .base
            .extra_headers
            .remove("X-Foo.bar");
        config.mark_dirty("providers.models.openai.fresh.extra_headers.X-Foo.bar");
        config.save_dirty().await.expect("save_dirty must succeed");

        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            !written.contains("X-Foo.bar"),
            "unsetting a dotted dynamic secret-map key must delete the literal \
             on-disk entry; got:\n{written}"
        );
    }

    /// `create_map_key("mcp.servers", "new")` followed by per-field
    /// edits must produce a complete `[[mcp.servers]]` table on disk —
    /// including the seeded natural-key field. This is the path the
    /// dashboard's `+ Add MCP server` affordance walks: insert, then
    /// edit `command` / `transport` etc.
    #[test]
    async fn save_dirty_writes_new_mcp_server_added_via_create_map_key() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Seed an unrelated existing entry so we exercise the
        // append-into-existing-array path (not the create-array-from-
        // scratch path). Both matter; the empty-doc case is covered by
        // `save` instead of `save_dirty` (see the `!config_path.exists()`
        // branch at the top of `save_dirty`).
        let seed = format!(
            "schema_version = {}\n\n\
             [[mcp.servers]]\n\
             name = \"fs\"\n\
             transport = \"stdio\"\n\
             command = \"/usr/bin/mcp-fs\"\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.mcp.servers.push(McpServerConfig {
            name: "fs".into(),
            transport: McpTransport::Stdio,
            command: "/usr/bin/mcp-fs".into(),
            ..Default::default()
        });

        // The handle_config_map_key_create dispatch path runs
        // create_map_key + mark_dirty(`<section>.<key>`). Replicate
        // that here so we're testing the same wire sequence.
        let created = config
            .create_map_key("mcp.servers", "github")
            .expect("create_map_key on a natural-key section must succeed");
        assert!(created);
        config.mark_dirty("mcp.servers.github");

        // Per-field edit on the freshly-added entry.
        config
            .set_prop_persistent("mcp.servers.github.transport", "http")
            .expect("set transport on freshly-added entry must route");
        config
            .set_prop_persistent("mcp.servers.github.url", "https://mcp.example/")
            .expect("set url on freshly-added entry must route");

        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        // Both entries survive, with their distinct fields.
        assert!(
            written.contains("name = \"fs\""),
            "pre-existing entry must survive; got:\n{written}"
        );
        assert!(
            written.contains("name = \"github\""),
            "new entry's natural-key field must land on disk; got:\n{written}"
        );
        assert!(
            written.contains("transport = \"http\""),
            "per-field edit on the new entry must land; got:\n{written}"
        );
        assert!(
            written.contains("url = \"https://mcp.example/\""),
            "second per-field edit on the new entry must land; got:\n{written}"
        );
        // Round-trip: parsing the written file must yield exactly the
        // shape we built up in memory. Catches mis-shaped output (e.g.
        // a nested `mcp.servers.github` inline table sneaking in instead
        // of a second `[[mcp.servers]]`).
        let reparsed: Config = toml::from_str(&written).unwrap();
        assert_eq!(reparsed.mcp.servers.len(), 2);
        let gh = reparsed
            .mcp
            .servers
            .iter()
            .find(|s| s.name == "github")
            .expect("reparse must surface the new entry by natural key");
        assert_eq!(gh.transport, McpTransport::Http);
        assert_eq!(gh.url.as_deref(), Some("https://mcp.example/"));
    }

    /// `rename_map_key("mcp.servers", "fs", "filesystem")` rewrites the
    /// in-memory entry's `name` field in place and marks BOTH aliases
    /// dirty. The incremental writer must update the matching
    /// `[[mcp.servers]]` entry's `name` to the new value without
    /// leaving a stale duplicate behind.
    #[test]
    async fn save_dirty_persists_mcp_server_rename_via_natural_key() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let seed = format!(
            "schema_version = {}\n\n\
             [[mcp.servers]]\n\
             name = \"fs\"\n\
             transport = \"stdio\"\n\
             command = \"/usr/bin/mcp-fs\"\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.mcp.servers.push(McpServerConfig {
            name: "fs".into(),
            transport: McpTransport::Stdio,
            command: "/usr/bin/mcp-fs".into(),
            ..Default::default()
        });

        // Mirror handle_config_map_key_rename: rename, then mark both
        // the old and new aliases dirty.
        let renamed = config
            .rename_map_key("mcp.servers", "fs", "filesystem")
            .expect("rename of a unique alias must succeed");
        assert!(renamed);
        config.mark_dirty("mcp.servers.fs");
        config.mark_dirty("mcp.servers.filesystem");

        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            written.contains("name = \"filesystem\""),
            "rename target must land on disk; got:\n{written}"
        );
        assert!(
            !written.contains("name = \"fs\""),
            "stale rename source must NOT remain on disk; got:\n{written}"
        );
        // Other fields on the renamed entry are preserved.
        assert!(
            written.contains("command = \"/usr/bin/mcp-fs\""),
            "rename must preserve sibling fields on the entry; got:\n{written}"
        );

        let reparsed: Config = toml::from_str(&written).unwrap();
        assert_eq!(reparsed.mcp.servers.len(), 1);
        assert_eq!(reparsed.mcp.servers[0].name, "filesystem");
        assert_eq!(reparsed.mcp.servers[0].command, "/usr/bin/mcp-fs");
    }

    /// `delete_map_key("mcp.servers", "fs")` removes the in-memory
    /// entry and marks the alias dirty. The incremental writer must
    /// drop the corresponding `[[mcp.servers]]` entry from disk,
    /// dropping the array slot entirely when no entries remain so the
    /// file doesn't carry a dangling `[[mcp.servers]]` section header.
    #[test]
    async fn save_dirty_removes_mcp_server_deleted_via_natural_key() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Two entries on disk; we'll delete one and assert the other
        // survives.
        let seed = format!(
            "schema_version = {}\n\n\
             [[mcp.servers]]\n\
             name = \"fs\"\n\
             transport = \"stdio\"\n\
             command = \"/usr/bin/mcp-fs\"\n\n\
             [[mcp.servers]]\n\
             name = \"github\"\n\
             transport = \"http\"\n\
             url = \"https://mcp.example/\"\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.mcp.servers.push(McpServerConfig {
            name: "fs".into(),
            transport: McpTransport::Stdio,
            command: "/usr/bin/mcp-fs".into(),
            ..Default::default()
        });
        config.mcp.servers.push(McpServerConfig {
            name: "github".into(),
            transport: McpTransport::Http,
            url: Some("https://mcp.example/".to_string()),
            ..Default::default()
        });

        let deleted = config
            .delete_map_key("mcp.servers", "fs")
            .expect("delete by natural key must resolve");
        assert!(deleted);
        config.mark_dirty("mcp.servers.fs");

        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            !written.contains("name = \"fs\""),
            "deleted entry must not survive incremental save; got:\n{written}"
        );
        assert!(
            written.contains("name = \"github\""),
            "untouched sibling entry must survive; got:\n{written}"
        );

        let reparsed: Config = toml::from_str(&written).unwrap();
        assert_eq!(reparsed.mcp.servers.len(), 1);
        assert_eq!(reparsed.mcp.servers[0].name, "github");
    }

    /// Deleting the last `[[mcp.servers]]` entry drops the array slot
    /// entirely. A vestigial empty `[[mcp.servers]]` header would
    /// reparse as a single default-shaped element with an empty
    /// natural-key field, which the validator then rejects on next
    /// load — actively breaking the file the writer just produced.
    #[test]
    async fn save_dirty_drops_array_header_when_last_mcp_server_removed() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let seed = format!(
            "schema_version = {}\n\n\
             [[mcp.servers]]\n\
             name = \"fs\"\n\
             transport = \"stdio\"\n\
             command = \"/usr/bin/mcp-fs\"\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.mcp.servers.push(McpServerConfig {
            name: "fs".into(),
            transport: McpTransport::Stdio,
            command: "/usr/bin/mcp-fs".into(),
            ..Default::default()
        });

        config.delete_map_key("mcp.servers", "fs").unwrap();
        config.mark_dirty("mcp.servers.fs");
        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            !written.contains("[[mcp.servers]]"),
            "empty array header must be dropped, otherwise it reparses as a default entry; got:\n{written}"
        );
        let reparsed: Config = toml::from_str(&written).unwrap();
        assert!(reparsed.mcp.servers.is_empty());
    }

    /// `Config::map_key_sections()` must surface the natural-key field
    /// for `mcp.servers`. This is the metadata `apply_dirty_path` reads
    /// to decide whether to take the array-of-tables branch; if the
    /// derive ever stops emitting it for `#[natural_key = "..."]` Vec
    /// fields, the dirty-path writer falls back to the broken
    /// Table-only walker and silently drops MCP edits on the floor
    /// again. Lock the contract here.
    #[test]
    async fn map_key_sections_exposes_natural_key_for_mcp_servers() {
        let sections = Config::map_key_sections();
        let entry = sections
            .iter()
            .find(|s| s.path == "mcp.servers")
            .expect("mcp.servers must be discoverable in map_key_sections()");
        assert_eq!(entry.kind, crate::traits::MapKeyKind::List);
        assert_eq!(
            entry.natural_key,
            Some("name"),
            "natural_key must mirror the `#[natural_key = \"name\"]` attribute \
             on McpConfig::servers; the dirty-path writer keys off this to take \
             the array-of-tables branch"
        );

        // Sanity: a representative HashMap section (alias IS the TOML
        // key) carries `natural_key: None`. The dirty-path writer's
        // branch decision falls through to the generic Table walker
        // for these.
        let anthropic = sections
            .iter()
            .find(|s| s.path == "providers.models.anthropic")
            .expect(
                "providers.models.anthropic must surface as a HashMap-backed map-keyed section",
            );
        assert_eq!(anthropic.kind, crate::traits::MapKeyKind::Map);
        assert_eq!(anthropic.natural_key, None);
    }

    /// `model_routes` and `embedding_routes` are `#[nested]` Vec fields
    /// with `#[natural_key = "hint"]` — they must surface in
    /// `map_key_sections()` as `List` entries so the dashboard and the
    /// incremental TOML writer can address individual route entries.
    #[tokio::test]
    async fn map_key_sections_exposes_natural_key_for_model_routes() {
        let sections = Config::map_key_sections();
        let entry = sections
            .iter()
            .find(|s| s.path == "model_routes")
            .expect("model_routes must be discoverable in map_key_sections()");
        assert_eq!(entry.kind, crate::traits::MapKeyKind::List);
        assert_eq!(
            entry.natural_key,
            Some("hint"),
            "natural_key must mirror the `#[natural_key = \"hint\"]` attribute \
             on Config::model_routes; the dirty-path writer keys off this to take \
             the array-of-tables branch"
        );
    }

    #[tokio::test]
    async fn map_key_sections_exposes_natural_key_for_embedding_routes() {
        let sections = Config::map_key_sections();
        let entry = sections
            .iter()
            .find(|s| s.path == "embedding_routes")
            .expect("embedding_routes must be discoverable in map_key_sections()");
        assert_eq!(entry.kind, crate::traits::MapKeyKind::List);
        assert_eq!(
            entry.natural_key,
            Some("hint"),
            "natural_key must mirror the `#[natural_key = \"hint\"]` attribute \
             on Config::embedding_routes; the dirty-path writer keys off this to take \
             the array-of-tables branch"
        );
    }

    /// A dirty path with a kebab-shaped inner field (e.g.
    /// `mcp.servers.fs.tool-timeout-secs`) must resolve through the
    /// shared `resolve_dirty_segments` helper inside the natural-key
    /// branch the same way the top-level Table walker does — landing
    /// on the snake `tool_timeout_secs` field on disk. This pins the
    /// dash-aware resolution that's load-bearing for any future
    /// natural-key struct field whose snake_case name is multi-word.
    /// Without this, a UI client that emits kebab field names would
    /// recreate the exact symptom the PR fixes (memory updates, disk
    /// stays stale) for any such field.
    #[test]
    async fn save_dirty_persists_mcp_server_kebab_inner_field_via_natural_key() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let seed = format!(
            "schema_version = {}\n\n\
             [[mcp.servers]]\n\
             name = \"fs\"\n\
             transport = \"stdio\"\n\
             command = \"/usr/bin/mcp-fs\"\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.mcp.servers.push(McpServerConfig {
            name: "fs".into(),
            transport: McpTransport::Stdio,
            command: "/usr/bin/mcp-fs".into(),
            tool_timeout_secs: Some(45),
            ..Default::default()
        });

        // mark_dirty directly with a kebab leaf segment so the
        // dash-aware resolver inside `resolve_dirty_segments` is the
        // only thing that can possibly land this on disk. set_prop /
        // set_prop_persistent route through the macro which has its
        // own snake-only field-name lookup; this test isolates the
        // writer-side resolution. The in-memory mutation above
        // simulates the dispatcher having already routed the
        // set_prop side; what we're testing here is the save side.
        config.mark_dirty("mcp.servers.fs.tool-timeout-secs");
        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            written.contains("tool_timeout_secs = 45"),
            "kebab dirty segment must resolve to the snake on-disk field; \
             got:\n{written}"
        );
        assert!(
            !written.contains("tool-timeout-secs"),
            "kebab field name must never appear on disk; got:\n{written}"
        );
        // Other fields on the entry survive the targeted edit.
        let reparsed: Config = toml::from_str(&written).unwrap();
        assert_eq!(reparsed.mcp.servers.len(), 1);
        assert_eq!(reparsed.mcp.servers[0].name, "fs");
        assert_eq!(reparsed.mcp.servers[0].command, "/usr/bin/mcp-fs");
        assert_eq!(reparsed.mcp.servers[0].tool_timeout_secs, Some(45));
    }

    /// An explicit per-field unset (the in-memory field reverts to
    /// `None`, but the dirty path still names that specific inner
    /// field rather than the whole element) must drive the case-1
    /// `mem missing → delete` branch — removing the field from the
    /// `[[mcp.servers]]` entry on disk without touching its siblings.
    /// The pre-existing whole-element delete test covers a different
    /// path (the rename source / `delete_map_key`); this one pins
    /// the per-field delete branch independently. Without it, a
    /// future refactor that collapses case-1's `None` branch into
    /// the whole-element path would silently break "clear this field"
    /// edits — the field would survive on disk while showing as
    /// unset in the UI.
    #[test]
    async fn save_dirty_unset_per_field_drops_field_from_mcp_server_entry() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Seed an entry with tool_timeout_secs set — this is the
        // field we'll unset via a per-field dirty path.
        let seed = format!(
            "schema_version = {}\n\n\
             [[mcp.servers]]\n\
             name = \"fs\"\n\
             transport = \"stdio\"\n\
             command = \"/usr/bin/mcp-fs\"\n\
             tool_timeout_secs = 45\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        // In-memory mirror without the optional field set — i.e. the
        // UI user just cleared `tool_timeout_secs`.
        config.mcp.servers.push(McpServerConfig {
            name: "fs".into(),
            transport: McpTransport::Stdio,
            command: "/usr/bin/mcp-fs".into(),
            tool_timeout_secs: None,
            ..Default::default()
        });
        // Per-field dirty path — exactly what a UI "clear this
        // field" affordance emits. The whole-element bare alias
        // `mcp.servers.fs` is intentionally NOT marked: the bare
        // alias is the rename/delete shape; the bug-prone case is
        // the per-field one.
        config.mark_dirty("mcp.servers.fs.tool_timeout_secs");
        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            !written.contains("tool_timeout_secs"),
            "explicit per-field unset must drop the field from disk; got:\n{written}"
        );
        // Siblings survive: the entry isn't deleted, just the one
        // field, and the natural-key field itself must stay so
        // subsequent loads still know which alias this entry is.
        assert!(
            written.contains("name = \"fs\""),
            "natural-key field must survive a per-field unset; got:\n{written}"
        );
        assert!(
            written.contains("command = \"/usr/bin/mcp-fs\""),
            "sibling fields must survive a per-field unset; got:\n{written}"
        );

        let reparsed: Config = toml::from_str(&written).unwrap();
        assert_eq!(reparsed.mcp.servers.len(), 1);
        assert_eq!(reparsed.mcp.servers[0].name, "fs");
        assert_eq!(reparsed.mcp.servers[0].tool_timeout_secs, None);
    }

    /// When the on-disk node at `mcp.servers` exists but has the
    /// wrong kind — e.g. a hand-edited `mcp.servers = "foo"`, or an
    /// inline-array-of-tables literal `servers = [{ ... }]` that
    /// `toml_edit` parses as `Item::Value(Value::Array)` rather than
    /// `Item::ArrayOfTables` — the writer must refuse to clobber
    /// rather than data-loss the user's hand-edit. The bail is
    /// observable (a `WARN`-level log event), but here we just pin
    /// the don't-clobber behavior at the disk level: the original
    /// shape survives the save unchanged. This is the explicit
    /// contract test for the wrong-kind bail surface; without it,
    /// a refactor that "fixes" the bail by overwriting silently
    /// would corrupt every operator who has either of these
    /// (otherwise valid) TOML shapes in their config.
    #[test]
    async fn save_dirty_refuses_to_clobber_wrong_kind_mcp_servers_node() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Hand-edited file: operator wrote `mcp.servers` as a scalar.
        // This is invalid against the schema but is what
        // `Item::Value(Value::String)` looks like in toml_edit, and
        // it's representative of the wrong-kind case (the same bail
        // path also fires for `Item::Value(Value::Array)` inline
        // arrays). Schema validation would reject this on load; the
        // test forces the writer to face the shape regardless.
        let seed = format!(
            "schema_version = {}\n\n\
             [mcp]\n\
             servers = \"hand-edited-scalar\"\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        // In-memory has a real server; the per-field edit would
        // normally land on `[[mcp.servers]]` on disk.
        config.mcp.servers.push(McpServerConfig {
            name: "fs".into(),
            transport: McpTransport::Stdio,
            command: "/usr/bin/mcp-fs".into(),
            ..Default::default()
        });
        config.mark_dirty("mcp.servers.fs.command");

        // The save itself must succeed — bail is a no-op for the
        // wrong-kind node, not an error.
        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        // The hand-edited scalar shape survives untouched: we'd
        // rather a load-time validation error the operator sees than
        // a silent overwrite of their (possibly intentional) file
        // surgery.
        assert!(
            written.contains("servers = \"hand-edited-scalar\""),
            "wrong-kind `mcp.servers` node must survive the save unchanged; \
             got:\n{written}"
        );
        assert!(
            !written.contains("[[mcp.servers]]"),
            "writer must not synthesize an array-of-tables next to a scalar \
             hand-edit; got:\n{written}"
        );
        assert!(
            !written.contains("/usr/bin/mcp-fs"),
            "in-memory command must not leak past a wrong-kind bail; \
             got:\n{written}"
        );
    }

    #[test]
    async fn collect_warnings_flags_wire_api_on_fixed_protocol_family() {
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        // mistral has a fixed wire protocol and ignores wire_api.
        config
            .providers
            .models
            .ensure("mistral", "primary")
            .unwrap()
            .wire_api = Some(WireApi::Responses);
        // custom honors wire_api — must NOT warn.
        config
            .providers
            .models
            .ensure("custom", "vllm")
            .unwrap()
            .wire_api = Some(WireApi::Responses);

        let warnings = config.collect_warnings();
        assert_eq!(warnings.len(), 1, "exactly the mistral entry should warn");
        let w = &warnings[0];
        assert_eq!(w.code, "wire_api_not_supported_for_family");
        assert_eq!(w.path, "providers.models.mistral.primary.wire_api");
        assert!(
            !warnings.iter().any(|w| w.path.contains("custom.vllm")),
            "custom honors wire_api and must not warn",
        );
    }

    /// The section is opt-in, so an operator who has not touched it is told
    /// nothing.
    #[test]
    async fn verifiable_intent_disabled_does_not_warn() {
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        assert!(!config.verifiable_intent.enabled);

        assert!(
            !config
                .collect_warnings()
                .iter()
                .any(|w| w.code == "verifiable_intent_tool_withheld"),
        );
    }

    /// Opting in produces the structured warning. This is the delivery path
    /// that survives `observability.log_persistence = "none"`, since
    /// `omnesagent doctor` prints this list to stdout and the config API returns
    /// it, neither of which goes through the log writer.
    #[test]
    async fn verifiable_intent_enabled_warns_that_the_tool_is_withheld() {
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        config.verifiable_intent.enabled = true;

        let warnings = config.collect_warnings();
        let withheld: Vec<_> = warnings
            .iter()
            .filter(|w| w.code == "verifiable_intent_tool_withheld")
            .collect();

        assert_eq!(withheld.len(), 1, "exactly one notice, got {warnings:?}");
        assert_eq!(withheld[0].path, "verifiable_intent.enabled");
        assert!(
            withheld[0].message.contains("vi_verify"),
            "the message must name the withheld tool: {}",
            withheld[0].message
        );
    }

    /// Every `LogPersistence` variant, walked through a `match` rather than
    /// listed.
    ///
    /// Adding a variant makes the `match` non-exhaustive, so a new policy forces
    /// a decision here instead of being covered silently. An array literal keeps
    /// compiling unchanged and covers one policy less.
    ///
    /// The compile error is the whole of the guarantee. An arm returning `None`
    /// satisfies it while leaving the variant out of the returned list, so this
    /// forces the update to be considered rather than making it correct.
    fn every_log_persistence() -> Vec<LogPersistence> {
        let mut all = Vec::new();
        let mut next = Some(LogPersistence::None);
        while let Some(policy) = next {
            all.push(policy);
            next = match policy {
                LogPersistence::None => Some(LogPersistence::Rolling),
                LogPersistence::Rolling => Some(LogPersistence::Full),
                LogPersistence::Full => Some(LogPersistence::Rotating),
                LogPersistence::Rotating => None,
            };
        }
        all
    }

    /// The config surface does not consult the observability policy, which is
    /// the property that makes it a second channel rather than a second copy
    /// of the same one. Asserting it here pins the independence at the unit
    /// level; the process-level proof lives in the component test.
    ///
    /// Every current variant is covered rather than the three that motivated the
    /// change. See [`every_log_persistence`] for the extent of that.
    #[test]
    async fn verifiable_intent_warning_is_independent_of_log_persistence() {
        for policy in every_log_persistence() {
            let mut config = Config::default();
            suppress_semantic_memory_warning(&mut config);
            config.verifiable_intent.enabled = true;
            config.observability.log_persistence = policy;

            assert!(
                config
                    .collect_warnings()
                    .iter()
                    .any(|w| w.code == "verifiable_intent_tool_withheld"),
                "the notice must survive log_persistence = {policy:?}",
            );
        }
    }

    #[cfg(unix)]
    #[test]
    async fn world_readable_config_is_detectable() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Create a config file with intentionally loose permissions
        std::fs::write(&config_path, "# test config").unwrap();
        std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o644)).unwrap();

        let meta = std::fs::metadata(&config_path).unwrap();
        let mode = meta.permissions().mode();
        assert!(
            mode & 0o004 != 0,
            "Test setup: file should be world-readable (mode {mode:o})"
        );
    }

    #[test]
    async fn transcription_config_defaults() {
        let tc = TranscriptionConfig::default();
        assert!(!tc.enabled);
        assert!(tc.api_url.contains("groq.com"));
        assert_eq!(tc.model, "whisper-large-v3-turbo");
        assert!(tc.language.is_none());
        assert!(tc.max_audio_bytes.is_none());
        assert_eq!(tc.max_duration_secs, 120);
        assert!(!tc.transcribe_non_ptt_audio);
    }

    #[test]
    async fn config_roundtrip_with_transcription() {
        let mut config = Config::default();
        config.transcription.enabled = true;
        config.transcription.language = Some("en".into());

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed = parse_test_config(&toml_str);

        assert!(parsed.transcription.enabled);
        assert_eq!(parsed.transcription.language.as_deref(), Some("en"));
        assert_eq!(parsed.transcription.model, "whisper-large-v3-turbo");
    }

    #[test]
    async fn config_roundtrip_with_transcription_max_audio_bytes() {
        let mut config = Config::default();
        config.transcription.max_audio_bytes = Some(65_536);

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed = parse_test_config(&toml_str);

        assert_eq!(parsed.transcription.max_audio_bytes, Some(65_536));
    }

    #[test]
    async fn transcription_max_audio_bytes_round_trips_through_prop_path() {
        let mut config = Config::default();

        assert_eq!(
            config
                .get_prop("transcription.max_audio_bytes")
                .unwrap()
                .as_str(),
            "<unset>"
        );

        config
            .set_prop("transcription.max_audio_bytes", "65536")
            .unwrap();
        assert_eq!(config.transcription.max_audio_bytes, Some(65_536));
        assert_eq!(
            config.get_prop("transcription.max_audio_bytes").unwrap(),
            "65536"
        );

        config
            .set_prop("transcription.max_audio_bytes", "")
            .unwrap();
        assert!(config.transcription.max_audio_bytes.is_none());
        assert_eq!(
            config.get_prop("transcription.max_audio_bytes").unwrap(),
            "<unset>"
        );
    }

    #[test]
    async fn config_validate_rejects_zero_transcription_max_audio_bytes() {
        let mut config = Config::default();
        config.transcription.max_audio_bytes = Some(0);

        let err = config.validate().unwrap_err();
        assert!(
            err.to_string()
                .contains("transcription.max_audio_bytes must be greater than zero"),
            "got: {err}"
        );
    }

    #[test]
    async fn config_without_transcription_uses_defaults() {
        let toml_str = r#"
            default_model_provider = "openrouter"
            default_model = "test-model"
            default_temperature = 0.7
        "#;
        let parsed = parse_test_config(toml_str);
        assert!(!parsed.transcription.enabled);
        assert_eq!(parsed.transcription.max_duration_secs, 120);
    }

    #[test]
    async fn security_defaults_are_backward_compatible() {
        let parsed = parse_test_config(
            r#"
default_model_provider = "openrouter"
default_model = "anthropic/claude-sonnet-4.6"
default_temperature = 0.7
"#,
        );

        assert!(!parsed.security.otp.enabled);
        assert_eq!(parsed.security.otp.method, OtpMethod::Totp);
        assert!(!parsed.security.estop.enabled);
        assert!(parsed.security.estop.require_otp_to_resume);
        assert!(parsed.security.leak_detection.enabled);
        assert_eq!(parsed.security.leak_detection.sensitivity, 0.7);
        assert!(parsed.security.leak_detection.high_entropy_tokens);
    }

    #[test]
    async fn security_toml_parses_otp_and_estop_sections() {
        let parsed = parse_test_config(
            r#"
default_model_provider = "openrouter"
default_model = "anthropic/claude-sonnet-4.6"
default_temperature = 0.7

[security.otp]
enabled = true
method = "totp"
token_ttl_secs = 30
cache_valid_secs = 120
gated_actions = ["shell", "browser_open"]
gated_domains = ["*.chase.com", "accounts.google.com"]
gated_domain_categories = ["banking"]

[security.estop]
enabled = true
state_file = "~/.omnesagent/estop-state.json"
require_otp_to_resume = true
"#,
        );

        assert!(parsed.security.otp.enabled);
        assert!(parsed.security.estop.enabled);
        assert_eq!(parsed.security.otp.gated_actions.len(), 2);
        assert_eq!(parsed.security.otp.gated_domains.len(), 2);
        parsed.validate().unwrap();
    }

    #[test]
    async fn security_toml_parses_leak_detection_section() {
        let parsed = parse_test_config(
            r#"
default_model_provider = "openrouter"
default_model = "anthropic/claude-sonnet-4.6"
default_temperature = 0.7

[security.leak_detection]
enabled = false
sensitivity = 0.35
high_entropy_tokens = false
"#,
        );

        assert!(!parsed.security.leak_detection.enabled);
        assert_eq!(parsed.security.leak_detection.sensitivity, 0.35);
        assert!(!parsed.security.leak_detection.high_entropy_tokens);
        parsed.validate().unwrap();
    }

    #[test]
    async fn security_validation_rejects_out_of_range_leak_detection_sensitivity() {
        let mut config = Config::default();
        config.security.leak_detection.sensitivity = 1.5;

        let err = config
            .validate()
            .expect_err("expected invalid leak-detection sensitivity");
        assert!(
            err.to_string()
                .contains("security.leak_detection.sensitivity"),
            "got: {err}"
        );
    }

    #[test]
    async fn security_validation_rejects_invalid_domain_glob() {
        let mut config = Config::default();
        config.security.otp.gated_domains = vec!["bad domain.com".into()];

        let err = config.validate().expect_err("expected invalid domain glob");
        assert!(err.to_string().contains("gated_domains"));
    }

    #[test]
    async fn security_validation_accepts_all_default_gated_actions_without_warning() {
        let mut config = Config::default();
        config.security.otp.gated_actions = default_otp_gated_actions();

        config
            .validate()
            .expect("the canonical default gated actions must validate clean");
    }

    #[test]
    async fn security_validation_accepts_unknown_gated_action_but_does_not_bail() {
        // An unknown but well-formed action name is a silent no-op today
        // (OTP enforcement of gated_actions is not wired through). Config load
        // must not fail on it — the operator's whole config would be rejected
        // for a typo'd gate — but the runtime emits a WARN so the no-op is not
        // silent. This asserts the warn-and-continue contract: load succeeds.
        let mut config = Config::default();
        config.security.otp.gated_actions = vec!["kubectl_write".into()];

        config
            .validate()
            .expect("an unknown gated action must warn, not reject the config");
    }

    #[test]
    async fn security_validation_still_rejects_malformed_gated_action() {
        // The unknown-name warn must not weaken the existing hard checks from
        // the charset/empty validation: a name with invalid characters still
        // bails rather than degrading to a warn.
        let mut config = Config::default();
        config.security.otp.gated_actions = vec!["kubectl write".into()];

        let err = config
            .validate()
            .expect_err("malformed gated action must still be rejected");
        assert!(err.to_string().contains("gated_actions"));
    }

    // The two `validate_*_transcription_default_provider` tests were removed
    // alongside the deleted `TranscriptionConfig.default_transcription_provider`
    // field in. there is no global default-provider concept; the equivalent
    // dangling-reference enforcement now lives on the per-agent
    // `agent.transcription_provider` field (see
    // `Config::validate()` checks for `tts_provider` / `transcription_provider`).

    #[tokio::test]
    async fn channel_secret_telegram_bot_token_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "omnesagent_test_tg_bot_token_{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).await.unwrap();

        let plaintext_token = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11";

        let mut config = Config {
            data_dir: dir.join("workspace"),
            config_path: dir.join("config.toml"),
            ..Default::default()
        };
        config.channels.telegram.insert(
            "default".to_string(),
            TelegramConfig {
                enabled: true,
                bot_token: plaintext_token.into(),
                api_base_url: default_telegram_api_base_url(),
                stream_mode: StreamMode::default(),
                draft_update_interval_ms: default_draft_update_interval_ms(),
                interrupt_on_new_message: false,
                mention_only: false,
                ack_reactions: None,
                proxy_url: None,
                approval_timeout_secs: default_telegram_approval_timeout_secs(),
                excluded_tools: vec![],
                reply_min_interval_secs: 0,
                reply_queue_depth_max: 0,
                debounce_ms: None,
            },
        );

        // Save (triggers encryption)
        config.save().await.unwrap();

        // Read raw TOML and verify plaintext token is NOT present
        let raw_toml = tokio::fs::read_to_string(&config.config_path)
            .await
            .unwrap();
        assert!(
            !raw_toml.contains(plaintext_token),
            "Saved TOML must not contain the plaintext bot_token"
        );

        // Parse stored TOML and verify the value is encrypted
        let stored: Config = toml::from_str(&raw_toml).unwrap();
        let stored_token = &stored.channels.telegram.get("default").unwrap().bot_token;
        assert!(
            crate::secrets::SecretStore::is_encrypted(stored_token),
            "Stored bot_token must be marked as encrypted"
        );

        // Decrypt and verify it matches the original plaintext
        let store = crate::secrets::SecretStore::new(&dir, true);
        assert_eq!(store.decrypt(stored_token).unwrap(), plaintext_token);

        // Simulate a full load: deserialize then decrypt (mirrors load_or_init logic)
        let mut loaded: Config = toml::from_str(&raw_toml).unwrap();
        loaded.config_path = dir.join("config.toml");
        let load_store = crate::secrets::SecretStore::new(&dir, loaded.secrets.encrypt);
        loaded.decrypt_secrets(&load_store).unwrap();
        assert_eq!(
            loaded.channels.telegram.get("default").unwrap().bot_token,
            plaintext_token,
            "Loaded bot_token must match the original plaintext after decryption"
        );

        let _ = fs::remove_dir_all(&dir).await;
    }

    #[test]
    async fn security_validation_rejects_unknown_domain_category() {
        let mut config = Config::default();
        config.security.otp.gated_domain_categories = vec!["not_real".into()];

        let err = config
            .validate()
            .expect_err("expected unknown domain category");
        assert!(err.to_string().contains("gated_domain_categories"));
    }

    #[test]
    async fn security_validation_rejects_zero_token_ttl() {
        let mut config = Config::default();
        config.security.otp.token_ttl_secs = 0;

        let err = config
            .validate()
            .expect_err("expected ttl validation failure");
        assert!(err.to_string().contains("token_ttl_secs"));
    }

    // ── MCP config validation ─────────────────────────────────────────────

    fn stdio_server(name: &str, command: &str) -> McpServerConfig {
        McpServerConfig {
            name: name.to_string(),
            transport: McpTransport::Stdio,
            command: command.to_string(),
            ..Default::default()
        }
    }

    fn http_server(name: &str, url: &str) -> McpServerConfig {
        McpServerConfig {
            name: name.to_string(),
            transport: McpTransport::Http,
            url: Some(url.to_string()),
            ..Default::default()
        }
    }

    fn sse_server(name: &str, url: &str) -> McpServerConfig {
        McpServerConfig {
            name: name.to_string(),
            transport: McpTransport::Sse,
            url: Some(url.to_string()),
            ..Default::default()
        }
    }

    #[test]
    async fn validate_mcp_config_empty_servers_ok() {
        let cfg = McpConfig::default();
        assert!(validate_mcp_config(&cfg).is_ok());
    }

    #[test]
    async fn validate_mcp_config_valid_stdio_ok() {
        let cfg = McpConfig {
            enabled: true,
            servers: vec![stdio_server("fs", "/usr/bin/mcp-fs")],
            ..Default::default()
        };
        assert!(validate_mcp_config(&cfg).is_ok());
    }

    #[test]
    async fn validate_mcp_config_valid_http_ok() {
        let cfg = McpConfig {
            enabled: true,
            servers: vec![http_server("svc", "http://localhost:8080/mcp")],
            ..Default::default()
        };
        assert!(validate_mcp_config(&cfg).is_ok());
    }

    #[test]
    async fn validate_mcp_config_valid_sse_ok() {
        let cfg = McpConfig {
            enabled: true,
            servers: vec![sse_server("svc", "https://example.com/events")],
            ..Default::default()
        };
        assert!(validate_mcp_config(&cfg).is_ok());
    }

    #[test]
    async fn validate_mcp_config_enforces_custom_ca_invariants_through_config() {
        let absolute_ca = std::env::temp_dir().join("omnesagent-test-ca.pem");
        let absolute_ca_string = absolute_ca.to_string_lossy().into_owned();

        let mut config = Config::default();
        config.mcp.servers = vec![
            http_server("public", "http://localhost:8080/mcp"),
            McpServerConfig {
                name: "private".into(),
                transport: McpTransport::Sse,
                url: Some("https://internal.example.invalid/sse".into()),
                tls_ca_cert_path: Some(absolute_ca_string.clone()),
                ..Default::default()
            },
        ];
        config
            .validate()
            .expect("unset and valid custom-CA configurations should validate");

        for (path, url, expected) in [
            (
                String::new(),
                "https://internal.example.invalid/mcp",
                "must not be empty",
            ),
            (
                "relative-ca.pem".to_string(),
                "https://internal.example.invalid/mcp",
                "must be an absolute path",
            ),
            (
                absolute_ca_string,
                "http://internal.example.invalid/mcp",
                "must use https when tls_ca_cert_path is set",
            ),
        ] {
            let mut config = Config::default();
            config.mcp.servers = vec![McpServerConfig {
                name: "private".into(),
                transport: McpTransport::Http,
                url: Some(url.into()),
                tls_ca_cert_path: Some(path),
                ..Default::default()
            }];
            let error = config
                .validate()
                .expect_err("invalid custom-CA configuration must fail validation");
            assert!(
                error.to_string().contains(expected),
                "expected {expected:?}, got: {error}"
            );
        }
    }

    #[test]
    async fn mcp_custom_ca_configured_and_unset_survive_save_and_reload() {
        let dir = tempfile::TempDir::new().unwrap();
        let absolute_ca = dir.path().join("internal-ca.pem");
        let mut config = Config {
            config_path: dir.path().join("config.toml"),
            data_dir: dir.path().join("workspace"),
            ..Default::default()
        };
        config.mcp.servers = vec![
            http_server("public", "https://public.example.invalid/mcp"),
            McpServerConfig {
                name: "private".into(),
                transport: McpTransport::Sse,
                url: Some("https://private.example.invalid/sse".into()),
                tls_ca_cert_path: Some(absolute_ca.to_string_lossy().into_owned()),
                ..Default::default()
            },
        ];

        config.save().await.unwrap();
        let raw = tokio::fs::read_to_string(&config.config_path)
            .await
            .unwrap();
        let loaded: Config = crate::migration::migrate_to_current(&raw).unwrap();
        loaded
            .validate()
            .expect("saved custom-CA configuration should validate after reload");
        assert_eq!(loaded.mcp.servers.len(), 2);
        assert!(loaded.mcp.servers[0].tls_ca_cert_path.is_none());
        assert_eq!(
            loaded.mcp.servers[1].tls_ca_cert_path.as_deref(),
            Some(absolute_ca.to_string_lossy().as_ref())
        );
    }

    #[test]
    async fn validate_mcp_config_rejects_empty_name() {
        let cfg = McpConfig {
            enabled: true,
            servers: vec![stdio_server("", "/usr/bin/tool")],
            ..Default::default()
        };
        let err = validate_mcp_config(&cfg).expect_err("empty name should fail");
        assert!(
            err.to_string().contains("name must not be empty"),
            "got: {err}"
        );
    }

    #[test]
    async fn validate_mcp_config_rejects_whitespace_name() {
        let cfg = McpConfig {
            enabled: true,
            servers: vec![stdio_server("   ", "/usr/bin/tool")],
            ..Default::default()
        };
        let err = validate_mcp_config(&cfg).expect_err("whitespace name should fail");
        assert!(
            err.to_string().contains("name must not be empty"),
            "got: {err}"
        );
    }

    #[test]
    async fn validate_mcp_config_rejects_duplicate_names() {
        let cfg = McpConfig {
            enabled: true,
            servers: vec![
                stdio_server("fs", "/usr/bin/mcp-a"),
                stdio_server("fs", "/usr/bin/mcp-b"),
            ],
            ..Default::default()
        };
        let err = validate_mcp_config(&cfg).expect_err("duplicate name should fail");
        assert!(err.to_string().contains("duplicate name"), "got: {err}");
    }

    #[test]
    async fn validate_mcp_config_rejects_zero_timeout() {
        let mut server = stdio_server("fs", "/usr/bin/mcp-fs");
        server.tool_timeout_secs = Some(0);
        let cfg = McpConfig {
            enabled: true,
            servers: vec![server],
            ..Default::default()
        };
        let err = validate_mcp_config(&cfg).expect_err("zero timeout should fail");
        assert!(err.to_string().contains("greater than 0"), "got: {err}");
    }

    #[test]
    async fn validate_mcp_config_rejects_timeout_exceeding_max() {
        let mut server = stdio_server("fs", "/usr/bin/mcp-fs");
        server.tool_timeout_secs = Some(MCP_MAX_TOOL_TIMEOUT_SECS + 1);
        let cfg = McpConfig {
            enabled: true,
            servers: vec![server],
            ..Default::default()
        };
        let err = validate_mcp_config(&cfg).expect_err("oversized timeout should fail");
        assert!(err.to_string().contains("exceeds max"), "got: {err}");
    }

    #[test]
    async fn validate_mcp_config_allows_max_timeout_exactly() {
        let mut server = stdio_server("fs", "/usr/bin/mcp-fs");
        server.tool_timeout_secs = Some(MCP_MAX_TOOL_TIMEOUT_SECS);
        let cfg = McpConfig {
            enabled: true,
            servers: vec![server],
            ..Default::default()
        };
        assert!(validate_mcp_config(&cfg).is_ok());
    }

    #[test]
    async fn validate_mcp_config_rejects_stdio_with_empty_command() {
        let cfg = McpConfig {
            enabled: true,
            servers: vec![stdio_server("fs", "")],
            ..Default::default()
        };
        let err = validate_mcp_config(&cfg).expect_err("empty command should fail");
        assert!(
            err.to_string().contains("requires non-empty command"),
            "got: {err}"
        );
    }

    #[test]
    async fn validate_mcp_config_rejects_http_without_url() {
        let cfg = McpConfig {
            enabled: true,
            servers: vec![McpServerConfig {
                name: "svc".to_string(),
                transport: McpTransport::Http,
                url: None,
                ..Default::default()
            }],
            ..Default::default()
        };
        let err = validate_mcp_config(&cfg).expect_err("http without url should fail");
        assert!(err.to_string().contains("requires url"), "got: {err}");
    }

    #[test]
    async fn validate_mcp_config_rejects_sse_without_url() {
        let cfg = McpConfig {
            enabled: true,
            servers: vec![McpServerConfig {
                name: "svc".to_string(),
                transport: McpTransport::Sse,
                url: None,
                ..Default::default()
            }],
            ..Default::default()
        };
        let err = validate_mcp_config(&cfg).expect_err("sse without url should fail");
        assert!(err.to_string().contains("requires url"), "got: {err}");
    }

    #[test]
    async fn validate_mcp_config_rejects_non_http_scheme() {
        let cfg = McpConfig {
            enabled: true,
            servers: vec![http_server("svc", "ftp://example.com/mcp")],
            ..Default::default()
        };
        let err = validate_mcp_config(&cfg).expect_err("non-http scheme should fail");
        assert!(err.to_string().contains("http/https"), "got: {err}");
    }

    #[test]
    async fn validate_mcp_config_rejects_invalid_url() {
        let cfg = McpConfig {
            enabled: true,
            servers: vec![http_server("svc", "not a url at all !!!")],
            ..Default::default()
        };
        let err = validate_mcp_config(&cfg).expect_err("invalid url should fail");
        assert!(err.to_string().contains("valid URL"), "got: {err}");
    }

    #[test]
    async fn mcp_transport_required_leaf_is_the_single_source() {
        // The relationship every consumer reads. Wire names must match the
        // `rename_all = "lowercase"` serde representation.
        assert_eq!(McpTransport::Stdio.required_leaf(), "command");
        assert_eq!(McpTransport::Http.required_leaf(), "url");
        assert_eq!(McpTransport::Sse.required_leaf(), "url");
        assert_eq!(McpTransport::Stdio.wire_name(), "stdio");
        assert_eq!(McpTransport::Http.wire_name(), "http");
        assert_eq!(McpTransport::Sse.wire_name(), "sse");
        // The schema-derived enumerator must surface every variant, and
        // `wire_name` must agree with serde for each, or the emitted metadata
        // desyncs from the wire representation the form reads.
        let transports = mcp_transports();
        assert_eq!(
            transports,
            vec![McpTransport::Stdio, McpTransport::Http, McpTransport::Sse]
        );
        for transport in transports {
            let wire = serde_json::to_value(transport).expect("transport serializes");
            assert_eq!(
                wire,
                serde_json::Value::String(transport.wire_name().into())
            );
        }
    }

    #[test]
    async fn validate_mcp_config_enforces_required_leaf_for_every_transport() {
        // Drift guard: whatever `required_leaf` declares, the validator must
        // actually enforce, so the schema metadata and the runtime check can
        // never disagree about which field a transport needs.
        for transport in mcp_transports() {
            let mut server = McpServerConfig {
                name: "svc".to_string(),
                transport,
                ..Default::default()
            };
            // Populate every required leaf except the one under test, leaving
            // the declared `required_leaf` empty, and confirm rejection.
            match transport.required_leaf() {
                "command" => server.command = String::new(),
                "url" => {
                    server.command = "echo".to_string();
                    server.url = None;
                }
                other => panic!("unhandled required leaf {other} for {transport:?}"),
            }
            let cfg = McpConfig {
                enabled: true,
                servers: vec![server],
                ..Default::default()
            };
            validate_mcp_config(&cfg).expect_err(&format!(
                "{transport:?} must reject an empty {}",
                transport.required_leaf()
            ));
        }
    }

    #[test]
    async fn mcp_server_schema_emits_required_by_transport_metadata() {
        // The config form reads `x-required-by-transport` off the
        // `McpServerConfig` element schema; assert it is present and projects
        // exactly `McpTransport::required_leaf` for every variant.
        #[cfg(feature = "schema-export")]
        let schema = schemars::schema_for!(McpServerConfig);
        let schema_json = serde_json::to_value(&schema).expect("schema serializes to json");
        let map = schema_json
            .get("x-required-by-transport")
            .and_then(serde_json::Value::as_object)
            .expect("schema should carry the x-required-by-transport extension");
        let transports = mcp_transports();
        assert_eq!(map.len(), transports.len());
        for transport in transports {
            assert_eq!(
                map.get(transport.wire_name())
                    .and_then(serde_json::Value::as_str),
                Some(transport.required_leaf()),
                "metadata for {transport:?} must match required_leaf",
            );
        }
    }

    #[test]
    async fn full_config_schema_nests_required_by_transport_on_mcp_server_def() {
        // The gateway serves `schema_for!(Config)` to the Operator Console; the
        // extension must survive into that full document (under `$defs`) where
        // the form resolves the `mcp.servers` element type, not just on the
        // standalone struct schema.
        #[cfg(feature = "schema-export")]
        let schema = schemars::schema_for!(Config);
        let schema_json = serde_json::to_value(&schema).expect("schema serializes to json");

        fn find_extension(
            value: &serde_json::Value,
        ) -> Option<&serde_json::Map<String, serde_json::Value>> {
            match value {
                serde_json::Value::Object(obj) => {
                    if let Some(found) = obj
                        .get("x-required-by-transport")
                        .and_then(serde_json::Value::as_object)
                    {
                        return Some(found);
                    }
                    obj.values().find_map(find_extension)
                }
                serde_json::Value::Array(items) => items.iter().find_map(find_extension),
                _ => None,
            }
        }

        let map = find_extension(&schema_json).expect(
            "full Config schema should carry x-required-by-transport on the mcp server def",
        );
        for transport in mcp_transports() {
            assert_eq!(
                map.get(transport.wire_name())
                    .and_then(serde_json::Value::as_str),
                Some(transport.required_leaf()),
            );
        }
    }

    #[test]
    async fn mcp_config_defaults_enabled_eager_loading_with_empty_servers() {
        let cfg = McpConfig::default();
        assert!(cfg.enabled);
        assert!(!cfg.deferred_loading);
        assert!(cfg.servers.is_empty());
    }

    #[test]
    async fn mcp_config_parsed_missing_flags_uses_enabled_eager_defaults() {
        let raw = r#"
[mcp]

[[mcp.servers]]
name = "svc"
transport = "http"
url = "http://localhost:8080/mcp"
"#;
        let parsed = parse_test_config(raw);
        assert!(parsed.mcp.enabled);
        assert!(!parsed.mcp.deferred_loading);
        assert_eq!(parsed.mcp.servers.len(), 1);
    }

    #[test]
    async fn mcp_config_explicit_disable_and_deferred_loading_are_respected() {
        let raw = r#"
[mcp]
enabled = false
deferred_loading = true

[[mcp.servers]]
name = "svc"
transport = "http"
url = "http://localhost:8080/mcp"
"#;
        let parsed = parse_test_config(raw);
        assert!(!parsed.mcp.enabled);
        assert!(parsed.mcp.deferred_loading);
        assert_eq!(parsed.mcp.servers.len(), 1);
    }

    #[test]
    async fn mcp_transport_serde_roundtrip_lowercase() {
        let cases = [
            (McpTransport::Stdio, "\"stdio\""),
            (McpTransport::Http, "\"http\""),
            (McpTransport::Sse, "\"sse\""),
        ];
        for (variant, expected_json) in &cases {
            let serialized = serde_json::to_string(variant).expect("serialize");
            assert_eq!(&serialized, expected_json, "variant: {variant:?}");
            let deserialized: McpTransport =
                serde_json::from_str(expected_json).expect("deserialize");
            assert_eq!(&deserialized, variant);
        }
    }

    #[tokio::test]
    async fn nevis_client_secret_encrypt_decrypt_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "omnesagent_test_nevis_secret_{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).await.unwrap();

        let plaintext_secret = "nevis-test-client-secret-value";

        let mut config = Config {
            data_dir: dir.join("workspace"),
            config_path: dir.join("config.toml"),
            ..Default::default()
        };
        config.security.nevis.client_secret = Some(plaintext_secret.into());

        // Save (triggers encryption)
        config.save().await.unwrap();

        // Read raw TOML and verify plaintext secret is NOT present
        let raw_toml = tokio::fs::read_to_string(&config.config_path)
            .await
            .unwrap();
        assert!(
            !raw_toml.contains(plaintext_secret),
            "Saved TOML must not contain the plaintext client_secret"
        );

        // Parse stored TOML and verify the value is encrypted
        let stored: Config = toml::from_str(&raw_toml).unwrap();
        let stored_secret = stored.security.nevis.client_secret.as_ref().unwrap();
        assert!(
            crate::secrets::SecretStore::is_encrypted(stored_secret),
            "Stored client_secret must be marked as encrypted"
        );

        // Decrypt and verify it matches the original plaintext
        let store = crate::secrets::SecretStore::new(&dir, true);
        assert_eq!(store.decrypt(stored_secret).unwrap(), plaintext_secret);

        // Simulate a full load: deserialize then decrypt (mirrors load_or_init logic)
        let mut loaded: Config = toml::from_str(&raw_toml).unwrap();
        loaded.config_path = dir.join("config.toml");
        let load_store = crate::secrets::SecretStore::new(&dir, loaded.secrets.encrypt);
        loaded.decrypt_secrets(&load_store).unwrap();
        assert_eq!(
            loaded.security.nevis.client_secret.as_deref().unwrap(),
            plaintext_secret,
            "Loaded client_secret must match the original plaintext after decryption"
        );

        let _ = fs::remove_dir_all(&dir).await;
    }

    // ══════════════════════════════════════════════════════════
    // Nevis config validation tests
    // ══════════════════════════════════════════════════════════

    #[test]
    async fn nevis_config_validate_disabled_accepts_empty_fields() {
        let cfg = NevisConfig::default();
        assert!(!cfg.enabled);
        assert!(cfg.validate().is_ok());
    }

    #[test]
    async fn nevis_config_validate_rejects_empty_instance_url() {
        let cfg = NevisConfig {
            enabled: true,
            instance_url: String::new(),
            client_id: "test-client".into(),
            ..NevisConfig::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.contains("instance_url"));
    }

    #[test]
    async fn nevis_config_validate_rejects_empty_client_id() {
        let cfg = NevisConfig {
            enabled: true,
            instance_url: "https://nevis.example.com".into(),
            client_id: String::new(),
            ..NevisConfig::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.contains("client_id"));
    }

    #[test]
    async fn nevis_config_validate_rejects_empty_realm() {
        let cfg = NevisConfig {
            enabled: true,
            instance_url: "https://nevis.example.com".into(),
            client_id: "test-client".into(),
            realm: String::new(),
            ..NevisConfig::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.contains("realm"));
    }

    #[test]
    async fn nevis_config_validate_rejects_local_without_jwks() {
        let cfg = NevisConfig {
            enabled: true,
            instance_url: "https://nevis.example.com".into(),
            client_id: "test-client".into(),
            token_validation: "local".into(),
            jwks_url: None,
            ..NevisConfig::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.contains("jwks_url"));
    }

    #[test]
    async fn nevis_config_validate_rejects_zero_session_timeout() {
        let cfg = NevisConfig {
            enabled: true,
            instance_url: "https://nevis.example.com".into(),
            client_id: "test-client".into(),
            token_validation: "remote".into(),
            session_timeout_secs: 0,
            ..NevisConfig::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.contains("session_timeout_secs"));
    }

    #[test]
    async fn nevis_config_validate_accepts_valid_enabled_config() {
        let cfg = NevisConfig {
            enabled: true,
            instance_url: "https://nevis.example.com".into(),
            realm: "master".into(),
            client_id: "test-client".into(),
            token_validation: "remote".into(),
            session_timeout_secs: 3600,
            ..NevisConfig::default()
        };
        assert!(cfg.validate().is_ok());
    }

    #[test]
    async fn nevis_config_validate_rejects_invalid_token_validation() {
        let cfg = NevisConfig {
            enabled: true,
            instance_url: "https://nevis.example.com".into(),
            realm: "master".into(),
            client_id: "test-client".into(),
            token_validation: "invalid_mode".into(),
            session_timeout_secs: 3600,
            ..NevisConfig::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(
            err.contains("invalid value 'invalid_mode'"),
            "Expected invalid token_validation error, got: {err}"
        );
    }

    #[test]
    async fn nevis_config_debug_redacts_client_secret() {
        let cfg = NevisConfig {
            client_secret: Some("super-secret".into()),
            ..NevisConfig::default()
        };
        let debug_output = format!("{:?}", cfg);
        assert!(
            !debug_output.contains("super-secret"),
            "Debug output must not contain the raw client_secret"
        );
        assert!(
            debug_output.contains("[REDACTED]"),
            "Debug output must show [REDACTED] for client_secret"
        );
    }

    #[test]
    async fn git_config_debug_redacts_private_key_and_access_token() {
        let cfg = GitConfig {
            private_key: Some(
                "-----BEGIN RSA PRIVATE KEY-----\nSUPERSECRETPEM\n-----END RSA PRIVATE KEY-----"
                    .into(),
            ),
            access_token: "ghp_supersecrettoken".into(),
            ..GitConfig::default()
        };
        let debug_output = format!("{cfg:?}");
        assert!(
            !debug_output.contains("SUPERSECRETPEM"),
            "Debug output must not contain the raw private_key PEM"
        );
        assert!(
            !debug_output.contains("ghp_supersecrettoken"),
            "Debug output must not contain the raw access_token"
        );
        assert!(
            debug_output.contains("***"),
            "Debug output must mask the private_key and access_token"
        );
    }

    #[test]
    async fn telegram_config_ack_reactions_false_deserializes() {
        let toml_str = r#"
            bot_token = "123:ABC"
            allowed_users = ["alice"]
            ack_reactions = false
        "#;
        let cfg: TelegramConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.ack_reactions, Some(false));
    }

    #[test]
    async fn telegram_config_ack_reactions_true_deserializes() {
        let toml_str = r#"
            bot_token = "123:ABC"
            allowed_users = ["alice"]
            ack_reactions = true
        "#;
        let cfg: TelegramConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.ack_reactions, Some(true));
    }

    #[test]
    async fn telegram_config_ack_reactions_missing_defaults_to_none() {
        let toml_str = r#"
            bot_token = "123:ABC"
            allowed_users = ["alice"]
        "#;
        let cfg: TelegramConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.ack_reactions, None);
    }

    #[test]
    async fn telegram_config_ack_reactions_channel_overrides_top_level() {
        let tg_toml = r#"
            bot_token = "123:ABC"
            allowed_users = ["alice"]
            ack_reactions = false
        "#;
        let tg: TelegramConfig = toml::from_str(tg_toml).unwrap();
        let top_level_ack = true;
        let effective = tg.ack_reactions.unwrap_or(top_level_ack);
        assert!(
            !effective,
            "channel-level false must override top-level true"
        );
    }

    #[test]
    async fn telegram_config_ack_reactions_falls_back_to_top_level() {
        let tg_toml = r#"
            bot_token = "123:ABC"
            allowed_users = ["alice"]
        "#;
        let tg: TelegramConfig = toml::from_str(tg_toml).unwrap();
        let top_level_ack = false;
        let effective = tg.ack_reactions.unwrap_or(top_level_ack);
        assert!(
            !effective,
            "must fall back to top-level false when channel omits field"
        );
    }

    #[test]
    async fn google_workspace_allowed_operations_deserialize_from_toml() {
        let toml_str = r#"
            enabled = true

            [[allowed_operations]]
            service = "gmail"
            resource = "users"
            sub_resource = "drafts"
            methods = ["create", "update"]
        "#;

        let cfg: GoogleWorkspaceConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.allowed_operations.len(), 1);
        assert_eq!(cfg.allowed_operations[0].service, "gmail");
        assert_eq!(cfg.allowed_operations[0].resource, "users");
        assert_eq!(
            cfg.allowed_operations[0].sub_resource.as_deref(),
            Some("drafts")
        );
        assert_eq!(
            cfg.allowed_operations[0].methods,
            vec!["create".to_string(), "update".to_string()]
        );
    }

    #[test]
    async fn google_workspace_allowed_operations_deserialize_without_sub_resource() {
        let toml_str = r#"
            enabled = true

            [[allowed_operations]]
            service = "drive"
            resource = "files"
            methods = ["list", "get"]
        "#;

        let cfg: GoogleWorkspaceConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.allowed_operations[0].sub_resource, None);
    }

    #[test]
    async fn config_validate_accepts_google_workspace_allowed_operations() {
        let mut cfg = Config::default();
        cfg.google_workspace.enabled = true;
        cfg.google_workspace.allowed_services = vec!["gmail".into()];
        cfg.google_workspace.allowed_operations = vec![GoogleWorkspaceAllowedOperation {
            service: "gmail".into(),
            resource: "users".into(),
            sub_resource: Some("drafts".into()),
            methods: vec!["create".into(), "update".into()],
        }];

        cfg.validate().unwrap();
    }

    #[test]
    async fn config_validate_rejects_duplicate_google_workspace_allowed_operations() {
        let mut cfg = Config::default();
        cfg.google_workspace.enabled = true;
        cfg.google_workspace.allowed_services = vec!["gmail".into()];
        cfg.google_workspace.allowed_operations = vec![
            GoogleWorkspaceAllowedOperation {
                service: "gmail".into(),
                resource: "users".into(),
                sub_resource: Some("drafts".into()),
                methods: vec!["create".into()],
            },
            GoogleWorkspaceAllowedOperation {
                service: "gmail".into(),
                resource: "users".into(),
                sub_resource: Some("drafts".into()),
                methods: vec!["update".into()],
            },
        ];

        let err = cfg.validate().unwrap_err().to_string();
        assert!(err.contains("duplicate service/resource/sub_resource entry"));
    }

    #[test]
    async fn config_validate_rejects_operation_service_not_in_allowed_services() {
        let mut cfg = Config::default();
        cfg.google_workspace.enabled = true;
        cfg.google_workspace.allowed_services = vec!["gmail".into()];
        cfg.google_workspace.allowed_operations = vec![GoogleWorkspaceAllowedOperation {
            service: "drive".into(), // drive is not in allowed_services
            resource: "files".into(),
            sub_resource: None,
            methods: vec!["list".into()],
        }];

        let err = cfg.validate().unwrap_err().to_string();
        assert!(
            err.contains("not in the effective allowed_services"),
            "expected not-in-allowed_services error, got: {err}"
        );
    }

    #[test]
    async fn config_validate_accepts_default_service_when_allowed_services_empty() {
        // When allowed_services is empty the validator uses DEFAULT_GWS_SERVICES.
        // A known default service must pass.
        let mut cfg = Config::default();
        cfg.google_workspace.enabled = true;
        // allowed_services deliberately left empty (falls back to defaults)
        cfg.google_workspace.allowed_operations = vec![GoogleWorkspaceAllowedOperation {
            service: "drive".into(),
            resource: "files".into(),
            sub_resource: None,
            methods: vec!["list".into()],
        }];

        assert!(cfg.validate().is_ok());
    }

    #[test]
    async fn config_validate_rejects_unknown_service_when_allowed_services_empty() {
        // Even with allowed_services empty (using defaults), an operation whose
        // service is not in DEFAULT_GWS_SERVICES must fail validation — not silently
        // pass through to be rejected at runtime.
        let mut cfg = Config::default();
        cfg.google_workspace.enabled = true;
        // allowed_services deliberately left empty
        cfg.google_workspace.allowed_operations = vec![GoogleWorkspaceAllowedOperation {
            service: "not_a_real_service".into(),
            resource: "files".into(),
            sub_resource: None,
            methods: vec!["list".into()],
        }];

        let err = cfg.validate().unwrap_err().to_string();
        assert!(
            err.contains("not in the effective allowed_services"),
            "expected effective-allowed_services error, got: {err}"
        );
    }

    // ── Bootstrap files ─────────────────────────────────────

    #[tokio::test]
    async fn ensure_bootstrap_files_creates_missing_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let ws = tmp.path().join("workspace");
        let _: () = tokio::fs::create_dir_all(&ws).await.unwrap();

        ensure_bootstrap_files(&ws).await.unwrap();

        let soul: String = tokio::fs::read_to_string(ws.join("SOUL.md")).await.unwrap();
        let identity: String = tokio::fs::read_to_string(ws.join("IDENTITY.md"))
            .await
            .unwrap();
        assert!(soul.contains("SOUL.md"));
        assert!(identity.contains("IDENTITY.md"));
    }

    #[tokio::test]
    async fn ensure_bootstrap_files_does_not_overwrite_existing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let ws = tmp.path().join("workspace");
        let _: () = tokio::fs::create_dir_all(&ws).await.unwrap();

        let custom = "# My custom SOUL";
        let _: () = tokio::fs::write(ws.join("SOUL.md"), custom).await.unwrap();

        ensure_bootstrap_files(&ws).await.unwrap();

        let soul: String = tokio::fs::read_to_string(ws.join("SOUL.md")).await.unwrap();
        assert_eq!(
            soul, custom,
            "ensure_bootstrap_files must not overwrite existing files"
        );

        // IDENTITY.md should still be created since it was missing
        let identity: String = tokio::fs::read_to_string(ws.join("IDENTITY.md"))
            .await
            .unwrap();
        assert!(identity.contains("IDENTITY.md"));
    }

    // ── PacingConfig serde defaults ─────────────────────────────

    #[test]
    async fn pacing_config_serde_defaults_match_manual_default() {
        // Deserialise an empty TOML table and verify the loop-detection
        // fields receive the same defaults as `PacingConfig::default()`.
        let from_toml: PacingConfig = toml::from_str("").unwrap();
        let manual = PacingConfig::default();

        assert_eq!(
            from_toml.loop_detection_enabled,
            manual.loop_detection_enabled
        );
        assert_eq!(
            from_toml.loop_detection_window_size,
            manual.loop_detection_window_size
        );
        assert_eq!(
            from_toml.loop_detection_max_repeats,
            manual.loop_detection_max_repeats
        );

        // Verify concrete values so a silent change to the defaults is caught.
        assert!(from_toml.loop_detection_enabled, "default should be true");
        assert_eq!(from_toml.loop_detection_window_size, 20);
        assert_eq!(from_toml.loop_detection_max_repeats, 3);
    }

    // ── Docker baked config template ────────────────────────────

    /// The TOML template baked into Docker images (Dockerfile + Dockerfile.debian).
    /// Kept here so changes to the Dockerfiles can be validated by `cargo test`.
    const DOCKER_CONFIG_TEMPLATE: &str = r#"
schema_version = 3
workspace_dir = "/omnesagent-data/workspace"
config_path = "/omnesagent-data/.omnesagent/config.toml"
api_key = ""
default_model_provider = "openrouter"
default_model = "anthropic/claude-sonnet-4-20250514"
default_temperature = 0.7

[gateway]
port = 42617
host = "[::]"
allow_public_bind = true

[risk_profiles.default]
level = "supervised"
auto_approve = ["file_read", "file_write", "file_edit", "memory_recall", "memory_store", "web_search_tool", "web_fetch", "calculator", "glob_search", "content_search", "image_info", "weather", "git_operations"]
"#;

    #[test]
    async fn docker_config_template_is_parseable() {
        let cfg: Config = toml::from_str(DOCKER_CONFIG_TEMPLATE)
            .expect("Docker baked config.toml must be valid TOML that deserialises into Config");

        let auto = &cfg
            .risk_profiles
            .get("default")
            .expect("Docker config must define [risk_profiles.default]")
            .auto_approve;
        for tool in &[
            "file_read",
            "file_write",
            "file_edit",
            "memory_recall",
            "memory_store",
            "web_search_tool",
            "web_fetch",
            "calculator",
            "glob_search",
            "content_search",
            "image_info",
            "weather",
            "git_operations",
        ] {
            assert!(
                auto.iter().any(|t| t == tool),
                "Docker config risk_profiles.default.auto_approve missing expected tool: {tool}"
            );
        }
    }

    #[test]
    async fn cost_enforcement_config_defaults() {
        let config = CostEnforcementConfig::default();
        assert_eq!(config.mode, "warn");
        assert_eq!(config.route_down_model, None);
        assert_eq!(config.reserve_percent, 10);
    }

    #[test]
    async fn cost_config_includes_enforcement() {
        let config = CostConfig::default();
        assert_eq!(config.enforcement.mode, "warn");
        assert_eq!(config.enforcement.reserve_percent, 10);
    }

    // ── Configurable macro tests ──

    #[test]
    async fn matrix_secret_fields_discovered() {
        let mx = MatrixConfig {
            enabled: true,
            homeserver: "https://m.org".into(),
            access_token: Some("tok".into()),
            user_id: None,
            device_id: None,
            allowed_rooms: vec!["!r:m".into()],
            interrupt_on_new_message: false,
            stream_mode: MatrixStreamMode::default(),
            draft_update_interval_ms: 1500,
            multi_message_delay_ms: 800,
            stream_draft_lines: 10,
            message_max_bytes: 48_000,
            stream_draft_delete: true,
            stream_reasoning: StreamReasoningMode::Status,
            stream_tool_arguments: vec![],
            recovery_key: None,
            mention_only: false,
            password: None,
            approval_timeout_secs: 300,
            reply_in_thread: true,
            ack_reactions: Some(true),
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };
        let fields = mx.secret_fields();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].name, "channels.matrix.access_token");
        assert_eq!(fields[0].category, "Channels");
        assert!(fields[0].is_set);
        assert_eq!(fields[1].name, "channels.matrix.recovery_key");
        assert!(!fields[1].is_set);
        assert_eq!(fields[2].name, "channels.matrix.password");
        assert!(!fields[2].is_set);
    }

    #[test]
    async fn matrix_secret_fields_empty_not_set() {
        let mx = MatrixConfig {
            enabled: true,
            homeserver: "https://m.org".into(),
            access_token: None,
            user_id: None,
            device_id: None,
            allowed_rooms: vec!["!r:m".into()],
            interrupt_on_new_message: false,
            stream_mode: MatrixStreamMode::default(),
            draft_update_interval_ms: 1500,
            multi_message_delay_ms: 800,
            stream_draft_lines: 10,
            message_max_bytes: 48_000,
            stream_draft_delete: true,
            stream_reasoning: StreamReasoningMode::Status,
            stream_tool_arguments: vec![
                StreamToolArgumentEntry::Defaults {
                    default_base: StreamToolArgumentBase::Safe,
                    argument_chars: Some(80),
                },
                StreamToolArgumentEntry::Tool {
                    tool: "delegate".into(),
                    base: Some(StreamToolArgumentBase::None),
                    include: vec!["agent".into(), "background".into()],
                    exclude: vec![],
                    argument_chars: Some(0),
                },
            ],
            recovery_key: None,
            mention_only: false,
            password: None,
            approval_timeout_secs: 300,
            reply_in_thread: true,
            ack_reactions: Some(true),
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };
        let fields = mx.secret_fields();
        assert!(!fields[0].is_set);
    }

    #[test]
    async fn set_secret_updates_field() {
        let mut mx = MatrixConfig {
            enabled: true,
            homeserver: "https://m.org".into(),
            access_token: Some("old".into()),
            user_id: None,
            device_id: None,
            allowed_rooms: vec!["!r:m".into()],
            interrupt_on_new_message: false,
            stream_mode: MatrixStreamMode::default(),
            draft_update_interval_ms: 1500,
            multi_message_delay_ms: 800,
            stream_draft_lines: 10,
            message_max_bytes: 48_000,
            stream_draft_delete: true,
            stream_reasoning: StreamReasoningMode::Status,
            stream_tool_arguments: vec![],
            recovery_key: None,
            mention_only: false,
            password: None,
            approval_timeout_secs: 300,
            reply_in_thread: true,
            ack_reactions: Some(true),
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };
        mx.set_secret("channels.matrix.access_token", "new-token".into())
            .unwrap();
        assert_eq!(mx.access_token.as_deref(), Some("new-token"));
    }

    #[test]
    async fn set_secret_unknown_name_fails() {
        let mut mx = MatrixConfig {
            enabled: true,
            homeserver: "https://m.org".into(),
            access_token: Some("tok".into()),
            user_id: None,
            device_id: None,
            allowed_rooms: vec!["!r:m".into()],
            interrupt_on_new_message: false,
            stream_mode: MatrixStreamMode::default(),
            draft_update_interval_ms: 1500,
            multi_message_delay_ms: 800,
            stream_draft_lines: 10,
            message_max_bytes: 48_000,
            stream_draft_delete: true,
            stream_reasoning: StreamReasoningMode::Status,
            stream_tool_arguments: vec![],
            recovery_key: None,
            mention_only: false,
            password: None,
            approval_timeout_secs: 300,
            reply_in_thread: true,
            ack_reactions: Some(true),
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };
        assert!(
            mx.set_secret("channels.matrix.nonexistent", "val".into())
                .is_err()
        );
    }

    #[test]
    async fn config_tree_traversal_discovers_nested_secrets() {
        let mut config = Config::default();
        // Set api_key on first model_provider entry (or create one)
        config
            .providers
            .models
            .ensure("anthropic", "default")
            .expect("anthropic typed slot")
            .api_key = Some("test-key".into());
        config.channels.matrix.insert(
            "default".to_string(),
            MatrixConfig {
                enabled: true,
                homeserver: "https://m.org".into(),
                access_token: Some("mx-tok".into()),
                user_id: None,
                device_id: None,
                allowed_rooms: vec!["!r:m".into()],
                interrupt_on_new_message: false,
                stream_mode: MatrixStreamMode::default(),
                draft_update_interval_ms: 1500,
                multi_message_delay_ms: 800,
                stream_draft_lines: 10,
                message_max_bytes: 48_000,
                stream_draft_delete: true,
                stream_reasoning: StreamReasoningMode::Status,
                stream_tool_arguments: vec![],
                recovery_key: None,
                mention_only: false,
                password: None,
                approval_timeout_secs: 300,
                reply_in_thread: true,
                ack_reactions: Some(true),
                excluded_tools: vec![],
                reply_min_interval_secs: 0,
                reply_queue_depth_max: 0,
            },
        );

        let fields = config.secret_fields();
        let names: Vec<&str> = fields.iter().map(|f| f.name).collect();
        assert!(names.contains(&"channels.matrix.access_token"));
        assert!(names.contains(&"channels.matrix.recovery_key"));
        assert!(
            names.contains(&"http_request.secrets"),
            "http_request.secrets must be classified as a secret map"
        );
    }

    #[test]
    async fn config_set_secret_dispatches_to_child() {
        let mut config = Config::default();
        config.channels.matrix.insert(
            "default".to_string(),
            MatrixConfig {
                enabled: true,
                homeserver: "https://m.org".into(),
                access_token: Some("old".into()),
                user_id: None,
                device_id: None,
                allowed_rooms: vec!["!r:m".into()],
                interrupt_on_new_message: false,
                stream_mode: MatrixStreamMode::default(),
                draft_update_interval_ms: 1500,
                multi_message_delay_ms: 800,
                stream_draft_lines: 10,
                message_max_bytes: 48_000,
                stream_draft_delete: true,
                stream_reasoning: StreamReasoningMode::Status,
                stream_tool_arguments: vec![],
                recovery_key: None,
                mention_only: false,
                password: None,
                approval_timeout_secs: 300,
                reply_in_thread: true,
                ack_reactions: Some(true),
                excluded_tools: vec![],
                reply_min_interval_secs: 0,
                reply_queue_depth_max: 0,
            },
        );

        config
            .set_secret("channels.matrix.access_token", "new".into())
            .unwrap();
        assert_eq!(
            config
                .channels
                .matrix
                .get("default")
                .unwrap()
                .access_token
                .as_deref(),
            Some("new")
        );
    }

    #[test]
    async fn config_set_secret_dispatches_to_matrix_child() {
        let mut config = Config::default();
        config.channels.matrix.insert(
            "default".to_string(),
            MatrixConfig {
                enabled: true,
                homeserver: "https://m.org".into(),
                access_token: Some("old".into()),
                user_id: None,
                device_id: None,
                allowed_rooms: vec!["!r:m".into()],
                interrupt_on_new_message: false,
                stream_mode: MatrixStreamMode::default(),
                draft_update_interval_ms: 1500,
                multi_message_delay_ms: 800,
                stream_draft_lines: 10,
                message_max_bytes: 48_000,
                stream_draft_delete: true,
                stream_reasoning: StreamReasoningMode::Status,
                stream_tool_arguments: vec![],
                mention_only: false,
                recovery_key: None,
                password: None,
                approval_timeout_secs: 300,
                reply_in_thread: true,
                ack_reactions: Some(true),
                excluded_tools: vec![],
                reply_min_interval_secs: 0,
                reply_queue_depth_max: 0,
            },
        );
        config
            .set_secret("channels.matrix.access_token", "sk-test".into())
            .unwrap();
        assert_eq!(
            config
                .channels
                .matrix
                .get("default")
                .unwrap()
                .access_token
                .as_deref(),
            Some("sk-test")
        );
    }

    #[test]
    async fn config_set_secret_unknown_fails() {
        let mut config = Config::default();
        assert!(
            config
                .set_secret("nonexistent.field", "val".into())
                .is_err()
        );
    }

    #[test]
    async fn config_set_http_request_secret_map_key_is_masked_and_encrypted() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        tokio::fs::write(&config_path, "schema_version = 1\n")
            .await
            .unwrap();
        let mut config = Config {
            config_path: config_path.clone(),
            data_dir: dir.path().join("workspace"),
            secrets: SecretsConfig { encrypt: true },
            ..Config::default()
        };
        let path = "http_request.secrets.api_token";

        assert!(
            Config::prop_is_secret(path),
            "dynamic http_request secret map entries must be classified as secret before the key exists"
        );
        config
            .set_prop_persistent(path, "Bearer from-config-set")
            .unwrap();

        assert_eq!(
            config
                .http_request
                .secrets
                .get("api_token")
                .map(String::as_str),
            Some("Bearer from-config-set")
        );
        assert_eq!(config.get_prop(path).unwrap(), "****");

        let field = config
            .prop_fields()
            .into_iter()
            .find(|field| field.name == path)
            .expect("dynamic secret map prop field");
        assert!(field.is_secret);
        assert_eq!(field.display_value, "****");
        assert_eq!(
            field.credential_class,
            Some(crate::config::CredentialSurfaceClass::EncryptedSecret)
        );

        config.save_dirty().await.unwrap();
        let contents = tokio::fs::read_to_string(&config_path).await.unwrap();
        assert!(
            !contents.contains("Bearer from-config-set"),
            "auth secret must not be written in plaintext: {contents}"
        );

        let stored = crate::migration::migrate_to_current(&contents).unwrap();
        let encrypted = stored.http_request.secrets.get("api_token").unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(encrypted));
        let store = crate::secrets::SecretStore::new(dir.path(), true);
        assert_eq!(store.decrypt(encrypted).unwrap(), "Bearer from-config-set");
    }

    #[test]
    async fn encrypt_decrypt_roundtrip_via_macro() {
        let dir = TempDir::new().unwrap();
        let store = crate::secrets::SecretStore::new(dir.path(), true);

        let mut mx = MatrixConfig {
            enabled: true,
            homeserver: "https://m.org".into(),
            access_token: Some("plaintext-token".into()),
            user_id: None,
            device_id: None,
            allowed_rooms: vec!["!r:m".into()],
            interrupt_on_new_message: false,
            stream_mode: MatrixStreamMode::default(),
            draft_update_interval_ms: 1500,
            multi_message_delay_ms: 800,
            stream_draft_lines: 10,
            message_max_bytes: 48_000,
            stream_draft_delete: true,
            stream_reasoning: StreamReasoningMode::Status,
            stream_tool_arguments: vec![],
            recovery_key: None,
            mention_only: false,
            password: None,
            approval_timeout_secs: 300,
            reply_in_thread: true,
            ack_reactions: Some(true),
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };

        // Encrypt
        mx.encrypt_secrets(&store).unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(
            mx.access_token.as_deref().unwrap_or_default()
        ));
        assert_ne!(mx.access_token.as_deref(), Some("plaintext-token"));

        // Decrypt
        mx.decrypt_secrets(&store).unwrap();
        assert_eq!(mx.access_token.as_deref(), Some("plaintext-token"));
    }

    #[test]
    async fn encrypt_skips_already_encrypted() {
        let dir = TempDir::new().unwrap();
        let store = crate::secrets::SecretStore::new(dir.path(), true);

        let mut mx = MatrixConfig {
            enabled: true,
            homeserver: "https://m.org".into(),
            access_token: Some("plaintext-token".into()),
            user_id: None,
            device_id: None,
            allowed_rooms: vec!["!r:m".into()],
            interrupt_on_new_message: false,
            stream_mode: MatrixStreamMode::default(),
            draft_update_interval_ms: 1500,
            multi_message_delay_ms: 800,
            stream_draft_lines: 10,
            message_max_bytes: 48_000,
            stream_draft_delete: true,
            stream_reasoning: StreamReasoningMode::Status,
            stream_tool_arguments: vec![],
            recovery_key: None,
            mention_only: false,
            password: None,
            approval_timeout_secs: 300,
            reply_in_thread: true,
            ack_reactions: Some(true),
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };

        mx.encrypt_secrets(&store).unwrap();
        let first_encrypted = mx.access_token.clone();

        // Encrypt again — should be idempotent
        mx.encrypt_secrets(&store).unwrap();
        assert_eq!(mx.access_token, first_encrypted);
    }

    #[test]
    async fn encrypt_no_op_on_disabled_store() {
        let dir = TempDir::new().unwrap();
        let store = crate::secrets::SecretStore::new(dir.path(), false);

        let mut mx = MatrixConfig {
            enabled: true,
            homeserver: "https://m.org".into(),
            access_token: Some("plaintext-token".into()),
            user_id: None,
            device_id: None,
            allowed_rooms: vec!["!r:m".into()],
            interrupt_on_new_message: false,
            stream_mode: MatrixStreamMode::default(),
            draft_update_interval_ms: 1500,
            multi_message_delay_ms: 800,
            stream_draft_lines: 10,
            message_max_bytes: 48_000,
            stream_draft_delete: true,
            stream_reasoning: StreamReasoningMode::Status,
            stream_tool_arguments: vec![],
            recovery_key: None,
            mention_only: false,
            password: None,
            approval_timeout_secs: 300,
            reply_in_thread: true,
            ack_reactions: Some(true),
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        };

        mx.encrypt_secrets(&store).unwrap();
        // With encryption disabled, value should stay plaintext
        assert_eq!(mx.access_token.as_deref(), Some("plaintext-token"));
    }

    // ── Property method tests ──

    fn test_matrix_config() -> MatrixConfig {
        MatrixConfig {
            enabled: true,
            homeserver: "https://m.org".into(),
            access_token: Some("tok".into()),
            user_id: Some("@bot:m.org".into()),
            device_id: None,
            allowed_rooms: vec!["!r:m".into()],
            interrupt_on_new_message: false,
            stream_mode: MatrixStreamMode::default(),
            draft_update_interval_ms: 1500,
            multi_message_delay_ms: 800,
            stream_draft_lines: 10,
            message_max_bytes: 48_000,
            stream_draft_delete: true,
            stream_reasoning: StreamReasoningMode::Status,
            stream_tool_arguments: vec![
                StreamToolArgumentEntry::Defaults {
                    default_base: StreamToolArgumentBase::Safe,
                    argument_chars: Some(80),
                },
                StreamToolArgumentEntry::Tool {
                    tool: "delegate".into(),
                    base: Some(StreamToolArgumentBase::None),
                    include: vec!["agent".into(), "background".into()],
                    exclude: vec![],
                    argument_chars: Some(0),
                },
            ],
            recovery_key: None,
            mention_only: false,
            password: None,
            approval_timeout_secs: 300,
            reply_in_thread: true,
            ack_reactions: Some(true),
            excluded_tools: vec![],
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
        }
    }

    #[test]
    async fn prop_fields_returns_typed_entries() {
        let mx = test_matrix_config();
        let fields = mx.prop_fields();
        let by_name: std::collections::HashMap<&str, &crate::traits::PropFieldInfo> =
            fields.iter().map(|f| (f.name.as_str(), f)).collect();

        // String field
        let homeserver = by_name["channels.matrix.homeserver"];
        assert_eq!(homeserver.type_hint, "String");
        assert_eq!(homeserver.display_value, "https://m.org");

        // Option<String> — set
        let user_id = by_name["channels.matrix.user_id"];
        assert_eq!(user_id.type_hint, "Option<String>");
        assert_eq!(user_id.display_value, "@bot:m.org");

        // Option<String> — unset
        let device_id = by_name["channels.matrix.device_id"];
        assert_eq!(device_id.display_value, "<unset>");

        // u64 field
        let interval = by_name["channels.matrix.draft_update_interval_ms"];
        assert_eq!(interval.type_hint, "u64");
        assert_eq!(interval.display_value, "1500");

        // Enum field
        let stream = by_name["channels.matrix.stream_mode"];
        assert!(stream.is_enum());
        assert!(stream.enum_variants.is_some());
        let reasoning = by_name["channels.matrix.stream_reasoning"];
        assert!(reasoning.is_enum());
        assert_eq!(reasoning.display_value, "status");
        let tool_arguments = by_name["channels.matrix.stream_tool_arguments"];
        assert_eq!(tool_arguments.kind, crate::traits::PropKind::ObjectArray);

        // Secret field — masked
        let token = by_name["channels.matrix.access_token"];
        assert!(token.is_secret);
        assert_eq!(token.display_value, "****");

        // All fields have correct category
        for field in &fields {
            assert_eq!(field.category, "Channels");
        }
    }

    #[test]
    async fn generated_config_fields_keep_operator_descriptions() {
        fn assert_description(
            fields: &[crate::traits::PropFieldInfo],
            suffix: &str,
            expected: &str,
        ) {
            let matches = fields
                .iter()
                .filter(|field| field.name.ends_with(suffix))
                .collect::<Vec<_>>();
            assert_eq!(
                matches.len(),
                1,
                "expected exactly one configurable field ending in `{suffix}`"
            );
            let field = matches[0];
            assert!(
                field
                    .description
                    .to_ascii_lowercase()
                    .contains(&expected.to_ascii_lowercase()),
                "description for {} must retain `{expected}`: {}",
                field.name,
                field.description,
            );
        }

        let workspace = crate::multi_agent::AgentWorkspaceConfig::default().prop_fields();
        assert_description(&workspace, ".access", "cross-agent workspace allowlist");
        assert_description(
            &workspace,
            ".read_memory_from",
            "Cross-agent memory allowlist",
        );

        let a2a = crate::multi_agent::A2aServerConfig::default().prop_fields();
        assert_description(&a2a, ".public_base_url", "operator-supplied base URL");

        let thinking = crate::scattered_types::ThinkingConfig::default().prop_fields();
        assert_description(&thinking, ".native_thinking", "selected level has a budget");

        let compression = crate::scattered_types::ContextCompressionConfig::default().prop_fields();
        assert_description(&compression, ".summary_provider", "<type>.<alias>");
        assert_description(&compression, ".summary_model", "DEPRECATED bare model id");

        let email = crate::scattered_types::EmailConfig::default().prop_fields();
        assert_description(&email, ".observer_mode", "never modifies any IMAP flag");
    }

    #[test]
    async fn agent_workspace_path_is_a_settable_property() {
        let mut workspace = crate::multi_agent::AgentWorkspaceConfig::default();

        let path = workspace
            .prop_fields()
            .into_iter()
            .find(|field| field.name == "agent_workspace.path")
            .expect("workspace path property");
        assert_eq!(path.kind, crate::config::PropKind::String);
        assert_eq!(path.display_value, crate::config::UNSET_DISPLAY);

        workspace
            .set_prop("agent_workspace.path", "/srv/omnesagent/assistant")
            .unwrap();
        assert_eq!(
            workspace.path,
            Some(std::path::PathBuf::from("/srv/omnesagent/assistant"))
        );
        assert_eq!(
            workspace.get_prop("agent_workspace.path").unwrap(),
            "/srv/omnesagent/assistant"
        );

        workspace.set_prop("agent_workspace.path", "").unwrap();
        assert_eq!(workspace.path, None);
    }

    #[cfg(feature = "schema-export")]
    #[test]
    async fn generated_config_types_keep_schema_descriptions() {
        fn assert_schema_description<T: schemars::JsonSchema>(name: &str) {
            let schema =
                serde_json::to_value(schemars::schema_for!(T)).expect("schema serializes to json");
            let description = schema
                .get("description")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("{name} schema must have a top-level description"));
            assert!(
                !description.trim().is_empty(),
                "{name} schema description must not be empty",
            );
        }

        use crate::autonomy::{ApprovalRoute, AutonomyLevel, DelegationPolicy};
        use crate::multi_agent::{
            A2aServerConfig, A2aServerSection, AccessMode, AgentA2aConfig, AgentMemoryConfig,
            AgentWorkspaceConfig, MemoryBackendKind, OutputModality,
        };
        use crate::presets::{
            BuilderSubmission, ChannelQuickStart, ModelProviderChoice, SelectorChoice,
        };
        use crate::providers::{ModelProviders, Providers};
        use crate::scattered_types::ChannelPrecheckConfig;
        use crate::sections::{Section, SectionGroup};
        use crate::validation_warnings::ValidationWarning;

        assert_schema_description::<AutonomyLevel>("AutonomyLevel");
        assert_schema_description::<DelegationPolicy>("DelegationPolicy");
        assert_schema_description::<ApprovalRoute>("ApprovalRoute");
        assert_schema_description::<AccessMode>("AccessMode");
        assert_schema_description::<MemoryBackendKind>("MemoryBackendKind");
        assert_schema_description::<AgentWorkspaceConfig>("AgentWorkspaceConfig");
        assert_schema_description::<AgentMemoryConfig>("AgentMemoryConfig");
        assert_schema_description::<OutputModality>("OutputModality");
        assert_schema_description::<A2aServerConfig>("A2aServerConfig");
        assert_schema_description::<A2aServerSection>("A2aServerSection");
        assert_schema_description::<AgentA2aConfig>("AgentA2aConfig");
        assert_schema_description::<ModelProviderChoice>("ModelProviderChoice");
        assert_schema_description::<ChannelQuickStart>("ChannelQuickStart");
        assert_schema_description::<BuilderSubmission>("BuilderSubmission");
        assert_schema_description::<SelectorChoice<ModelProviderChoice>>("SelectorChoice");
        assert_schema_description::<ModelProviders>("ModelProviders");
        assert_schema_description::<Providers>("Providers");
        assert_schema_description::<ChannelPrecheckConfig>("ChannelPrecheckConfig");
        assert_schema_description::<SectionGroup>("SectionGroup");
        assert_schema_description::<Section>("Section");
        assert_schema_description::<ValidationWarning>("ValidationWarning");

        let map_key_schema =
            serde_json::to_value(schemars::schema_for!(crate::traits::MapKeySection))
                .expect("MapKeySection schema serializes to json");
        let natural_key = map_key_schema
            .pointer("/properties/natural_key/description")
            .and_then(serde_json::Value::as_str)
            .expect("MapKeySection.natural_key must have a schema description");
        assert!(natural_key.contains("natural key"));
    }

    #[test]
    async fn get_prop_returns_values_by_path() {
        let mx = test_matrix_config();

        assert_eq!(
            mx.get_prop("channels.matrix.homeserver").unwrap(),
            "https://m.org"
        );
        assert_eq!(
            mx.get_prop("channels.matrix.draft_update_interval_ms")
                .unwrap(),
            "1500"
        );
        assert_eq!(
            mx.get_prop("channels.matrix.user_id").unwrap(),
            "@bot:m.org"
        );
        assert_eq!(
            mx.get_prop("channels.matrix.stream_reasoning").unwrap(),
            "status"
        );
        assert_eq!(mx.get_prop("channels.matrix.device_id").unwrap(), "<unset>");
        // Secrets return masked value
        assert_eq!(
            mx.get_prop("channels.matrix.access_token").unwrap(),
            "**** (encrypted)"
        );
    }

    #[test]
    async fn matrix_stream_tool_arguments_deserialize_from_compact_toml() {
        let matrix: MatrixConfig = toml::from_str(
            r#"
homeserver = "https://matrix.example"
stream_tool_arguments = [
    { default_base = "safe", argument_chars = 120 },
    { tool = "delegate", base = "none", include = ["agent", "background", "prompt"], argument_chars = 0 },
    { tool = "mock_tool", base = "all", exclude = ["token"] },
]
"#,
        )
        .unwrap();

        assert_eq!(matrix.stream_tool_arguments.len(), 3);
        assert_eq!(
            matrix.stream_tool_arguments[0],
            StreamToolArgumentEntry::Defaults {
                default_base: StreamToolArgumentBase::Safe,
                argument_chars: Some(120),
            }
        );
        assert!(matrix.validate_stream_tool_arguments().is_ok());
    }

    #[::core::prelude::v1::test]
    fn matrix_message_budget_clamps_values_below_one_unicode_scalar() {
        for message_max_bytes in 0..MATRIX_MIN_MESSAGE_MAX_BYTES {
            let matrix = MatrixConfig {
                message_max_bytes,
                ..Default::default()
            };
            assert_eq!(
                matrix.effective_message_max_bytes(),
                MATRIX_MIN_MESSAGE_MAX_BYTES
            );
        }

        let matrix = MatrixConfig {
            message_max_bytes: MATRIX_MIN_MESSAGE_MAX_BYTES + 1,
            ..Default::default()
        };
        assert_eq!(
            matrix.effective_message_max_bytes(),
            MATRIX_MIN_MESSAGE_MAX_BYTES + 1
        );
    }

    #[test]
    async fn matrix_stream_tool_arguments_reject_ambiguous_rules() {
        let matrix = MatrixConfig {
            stream_tool_arguments: vec![
                StreamToolArgumentEntry::Defaults {
                    default_base: StreamToolArgumentBase::Safe,
                    argument_chars: None,
                },
                StreamToolArgumentEntry::Defaults {
                    default_base: StreamToolArgumentBase::None,
                    argument_chars: Some(0),
                },
            ],
            ..Default::default()
        };
        assert!(
            matrix
                .validate_stream_tool_arguments()
                .unwrap_err()
                .to_string()
                .contains("duplicates")
        );

        let matrix = MatrixConfig {
            stream_tool_arguments: vec![StreamToolArgumentEntry::Tool {
                tool: "delegate".into(),
                base: None,
                include: vec!["prompt".into()],
                exclude: vec!["prompt".into()],
                argument_chars: None,
            }],
            ..Default::default()
        };
        assert!(
            matrix
                .validate_stream_tool_arguments()
                .unwrap_err()
                .to_string()
                .contains("includes and excludes")
        );
    }

    #[test]
    async fn matrix_stream_tool_arguments_round_trip_through_configurable_prop() {
        let mut matrix = MatrixConfig::default();
        matrix
            .set_prop(
                "channels.matrix.stream_tool_arguments",
                r#"[{"default_base":"none"},{"tool":"shell","base":"all","exclude":["command"]}]"#,
            )
            .unwrap();

        assert_eq!(matrix.stream_tool_arguments.len(), 2);
        assert!(matrix.validate_stream_tool_arguments().is_ok());
        assert!(
            matrix
                .get_prop("channels.matrix.stream_tool_arguments")
                .unwrap()
                .contains("shell")
        );
    }

    #[test]
    async fn get_prop_unknown_path_fails() {
        let mx = test_matrix_config();
        assert!(mx.get_prop("channels.matrix.nonexistent").is_err());
    }

    #[test]
    async fn set_prop_string() {
        let mut mx = test_matrix_config();
        mx.set_prop("channels.matrix.homeserver", "https://new.org")
            .unwrap();
        assert_eq!(mx.homeserver, "https://new.org");
    }

    #[test]
    async fn set_prop_bool() {
        let mut mx = test_matrix_config();
        mx.set_prop("channels.matrix.interrupt_on_new_message", "true")
            .unwrap();
        assert!(mx.interrupt_on_new_message);
    }

    #[test]
    async fn set_prop_bool_rejects_invalid() {
        let mut mx = test_matrix_config();
        let err = mx
            .set_prop("channels.matrix.interrupt_on_new_message", "yes")
            .unwrap_err();
        assert!(err.to_string().contains("bool"));
    }

    #[test]
    async fn set_prop_u64() {
        let mut mx = test_matrix_config();
        mx.set_prop("channels.matrix.draft_update_interval_ms", "3000")
            .unwrap();
        assert_eq!(mx.draft_update_interval_ms, 3000);
    }

    #[test]
    async fn set_prop_u64_rejects_invalid() {
        let mut mx = test_matrix_config();
        assert!(
            mx.set_prop("channels.matrix.draft_update_interval_ms", "abc")
                .is_err()
        );
    }

    #[test]
    async fn set_prop_option_string_set_and_clear() {
        let mut mx = test_matrix_config();
        mx.set_prop("channels.matrix.user_id", "@new:m.org")
            .unwrap();
        assert_eq!(mx.user_id.as_deref(), Some("@new:m.org"));

        // Empty string clears Option
        mx.set_prop("channels.matrix.user_id", "").unwrap();
        assert!(mx.user_id.is_none());
    }

    #[test]
    async fn set_prop_enum() {
        let mut mx = test_matrix_config();
        mx.set_prop("channels.matrix.stream_mode", "partial")
            .unwrap();
        assert_eq!(mx.stream_mode, MatrixStreamMode::Partial);

        mx.set_prop("channels.matrix.stream_mode", "single_message")
            .unwrap();
        assert_eq!(mx.stream_mode, MatrixStreamMode::SingleMessage);

        mx.set_prop("channels.matrix.stream_mode", "multi_message")
            .unwrap();
        assert_eq!(mx.stream_mode, MatrixStreamMode::MultiMessage);

        mx.set_prop("channels.matrix.stream_reasoning", "off")
            .unwrap();
        assert_eq!(mx.stream_reasoning, StreamReasoningMode::Off);

        mx.set_prop("channels.matrix.stream_reasoning", "full")
            .unwrap();
        assert_eq!(mx.stream_reasoning, StreamReasoningMode::Full);
    }

    #[test]
    async fn set_prop_enum_rejects_invalid() {
        let mut mx = test_matrix_config();
        let err = mx
            .set_prop("channels.matrix.stream_mode", "invalid")
            .unwrap_err();
        assert!(err.to_string().contains("expected one of"));

        let err = mx
            .set_prop("channels.matrix.stream_reasoning", "raw")
            .unwrap_err();
        assert!(err.to_string().contains("expected one of"));
    }

    #[test]
    async fn set_prop_unknown_path_fails() {
        let mut mx = test_matrix_config();
        assert!(mx.set_prop("channels.matrix.nonexistent", "val").is_err());
    }

    #[test]
    async fn set_prop_materializes_missing_nested_option_only_on_success() {
        let mut config = Config::default();
        assert!(config.gateway.tls.is_none());

        // Probing an absent Option<T> must not leave a phantom section when
        // the dotted path is unrelated or the target value is invalid.
        assert!(config.set_prop("gateway.tls.nonexistent", "value").is_err());
        assert!(config.gateway.tls.is_none());
        assert!(
            config
                .set_prop("gateway.tls.enabled", "not-a-bool")
                .is_err()
        );
        assert!(config.gateway.tls.is_none());

        config
            .set_prop("gateway.tls.cert_path", "/tmp/omnesagent-test-cert.pem")
            .expect("a valid child write should materialize its missing parent");
        assert_eq!(
            config
                .gateway
                .tls
                .as_ref()
                .expect("successful write should commit the parent")
                .cert_path,
            "/tmp/omnesagent-test-cert.pem",
        );
    }

    #[test]
    async fn prop_is_secret_static_check() {
        assert!(MatrixConfig::prop_is_secret("channels.matrix.access_token"));
        assert!(MatrixConfig::prop_is_secret("channels.matrix.recovery_key"));
        assert!(!MatrixConfig::prop_is_secret("channels.matrix.homeserver"));
        assert!(!MatrixConfig::prop_is_secret(
            "channels.matrix.interrupt_on_new_message"
        ));
    }

    #[test]
    async fn apply_env_overrides_rejects_schema_version() {
        let _env_guard = env_override_lock().await;
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("OMNESAGENT_schema_version", "99") };
        let mut config = Config::default();
        let result = crate::env_overrides::apply_env_overrides(&mut config);
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::remove_var("OMNESAGENT_schema_version") };

        let err = result.expect_err("schema_version override must be rejected");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("schema_version") && msg.contains("not overridable"),
            "error must name the path and the reason: {msg}",
        );
        // Untouched on rejection.
        assert_eq!(
            config.schema_version,
            crate::migration::CURRENT_SCHEMA_VERSION
        );
    }

    #[test]
    async fn prop_is_env_overridden_reflects_env_overridden_paths() {
        // Empty by default — no env applied.
        let mut cfg = Config::default();
        assert!(!cfg.prop_is_env_overridden("channels.matrix.homeserver"));
        assert!(!cfg.prop_is_env_overridden("gateway.request_timeout_secs"));

        // Populate the field directly (the same set that
        // `apply_env_overrides` returns from `load_or_init`).
        cfg.env_overridden_paths = std::collections::HashSet::from([
            "channels.matrix.homeserver".to_string(),
            "gateway.request_timeout_secs".to_string(),
        ]);

        // True for paths in the list, false for anything else.
        assert!(cfg.prop_is_env_overridden("channels.matrix.homeserver"));
        assert!(cfg.prop_is_env_overridden("gateway.request_timeout_secs"));
        assert!(!cfg.prop_is_env_overridden("channels.matrix.access_token"));
        assert!(!cfg.prop_is_env_overridden("gateway.host"));
        // Empty path / non-schema path → false.
        assert!(!cfg.prop_is_env_overridden(""));
        assert!(!cfg.prop_is_env_overridden("does.not.exist"));
    }

    #[test]
    async fn prop_is_secret_routes_through_hashmap_keyed_paths() {
        // Regression: the macro's HashMap<String, T> arm previously passed the
        // full materialised path (e.g. `model_providers.openrouter.api-key`)
        // straight to the inner type's `prop_is_secret`, which then matched on
        // its own configurable_prefix and returned false. Result: the CLI's
        // `config set --json` and the gateway's PropResponse both took the
        // non-secret branch and emitted `{value}` instead of `{populated}` for
        // any secret on a map-keyed nested type.
        assert!(Config::prop_is_secret(
            "providers.models.openrouter.default.api_key"
        ));
        assert!(Config::prop_is_secret(
            "providers.models.anthropic.default.api_key"
        ));
        assert!(!Config::prop_is_secret(
            "providers.models.openrouter.default.endpoint"
        ));
        assert!(!Config::prop_is_secret(
            "providers.models.openrouter.default.context-window"
        ));
    }

    #[test]
    async fn file_transfer_header_maps_are_secret() {
        assert!(Config::prop_is_secret(
            "file_download.headers.Authorization"
        ));
        assert!(Config::prop_is_secret("file_upload.headers.Authorization"));
        assert!(Config::prop_is_secret(
            "file_upload_bundle.headers.Authorization"
        ));
        assert!(Config::prop_is_secret(
            "mcp.servers.acme.headers.Authorization"
        ));
        assert!(!Config::prop_is_secret("file_download.timeout_secs"));
        // The bare map path is the `PropKind::SecretMap` container row: it is
        // secret-classified (it fronts encrypted values) even though its
        // display carries only key names.
        assert!(Config::prop_is_secret("file_download.headers"));
    }

    fn mcp_server_fixture(name: &str) -> McpServerConfig {
        McpServerConfig {
            name: name.into(),
            transport: McpTransport::Stdio,
            command: "/usr/bin/mcp-fs".into(),
            ..Default::default()
        }
    }

    /// Regression: an empty `#[secret] HashMap<String, String>` (MCP `env`)
    /// used to emit no property rows at all, so neither Omnescode nor the
    /// dashboard could add the first key. The container row must exist on
    /// an empty map and advertise itself as `SecretMap`.
    #[test]
    async fn empty_secret_map_emits_container_row() {
        let mut config = Config::default();
        config.mcp.servers.push(mcp_server_fixture("acme"));

        let fields = config.prop_fields();
        for leaf in ["env", "headers"] {
            let path = format!("mcp.servers.acme.{leaf}");
            let row = fields
                .iter()
                .find(|f| f.name == path)
                .unwrap_or_else(|| panic!("missing container row `{path}`"));
            assert_eq!(row.kind, crate::traits::PropKind::SecretMap);
            assert!(row.is_secret);
            assert_eq!(row.display_value, crate::traits::UNSET_DISPLAY);
            assert_eq!(
                row.credential_class,
                Some(crate::config::CredentialSurfaceClass::EncryptedSecret)
            );
            assert_eq!(
                config.get_prop(&path).unwrap(),
                crate::traits::UNSET_DISPLAY
            );
            assert!(Config::prop_is_secret(&path));
        }
        assert!(
            !fields
                .iter()
                .any(|f| f.name.starts_with("mcp.servers.acme.env.")),
            "an empty map has no entry rows"
        );
    }

    #[test]
    async fn secret_map_entries_add_list_and_remove_without_leaking_values() {
        let mut config = Config::default();
        config.mcp.servers.push(mcp_server_fixture("acme"));

        config
            .set_prop("mcp.servers.acme.env.ZED_TOKEN", "zed-plaintext")
            .unwrap();
        config
            .set_prop("mcp.servers.acme.env.API_KEY", "api-plaintext")
            .unwrap();
        assert_eq!(
            config.mcp.servers[0].env.get("API_KEY").map(String::as_str),
            Some("api-plaintext")
        );

        let fields = config.prop_fields();
        let env_rows: Vec<&str> = fields
            .iter()
            .filter(|f| f.name.starts_with("mcp.servers.acme.env"))
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(
            env_rows,
            vec![
                "mcp.servers.acme.env",
                "mcp.servers.acme.env.API_KEY",
                "mcp.servers.acme.env.ZED_TOKEN",
            ],
            "container first, then entries in sorted key order"
        );
        // Container display lists key names only.
        assert_eq!(
            config.get_prop("mcp.servers.acme.env").unwrap(),
            "API_KEY, ZED_TOKEN"
        );
        for field in &fields {
            assert!(
                !field.display_value.contains("plaintext"),
                "`{}` leaked a secret value: {}",
                field.name,
                field.display_value
            );
        }

        // Writing the container itself is rejected with guidance.
        let err = config
            .set_prop("mcp.servers.acme.env", "{\"X\":\"y\"}")
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("`env` is a key/value map") && err.contains("<this path>.<KEY>"),
            "{err}"
        );

        // An empty value (what `config delete` sends) removes the entry.
        config
            .set_prop("mcp.servers.acme.env.ZED_TOKEN", "")
            .unwrap();
        assert!(!config.mcp.servers[0].env.contains_key("ZED_TOKEN"));
        assert_eq!(config.get_prop("mcp.servers.acme.env").unwrap(), "API_KEY");
        config.set_prop("mcp.servers.acme.env.API_KEY", "").unwrap();
        assert!(config.mcp.servers[0].env.is_empty());
        assert_eq!(
            config.get_prop("mcp.servers.acme.env").unwrap(),
            crate::traits::UNSET_DISPLAY
        );
    }

    /// Same call sites the dashboard / Omnescode use (`set_prop_persistent`
    /// then `save_dirty`): an added env entry lands on disk encrypted, and
    /// removing it drops the key from disk rather than leaving `KEY = ""`.
    #[test]
    async fn save_dirty_persists_secret_map_entry_add_and_remove() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");
        let seed = format!(
            "schema_version = {}\n\n\
             [[mcp.servers]]\n\
             name = \"fs\"\n\
             transport = \"stdio\"\n\
             command = \"/usr/bin/mcp-fs\"\n",
            crate::migration::CURRENT_SCHEMA_VERSION
        );
        std::fs::write(&config_path, &seed).unwrap();
        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        config.mcp.servers.push(mcp_server_fixture("fs"));

        config
            .set_prop_persistent("mcp.servers.fs.env.GBR_MAILBOX_KEY", "mailbox-plaintext")
            .unwrap();
        config.save_dirty().await.unwrap();
        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            written.contains("GBR_MAILBOX_KEY"),
            "added env key must be written; got:\n{written}"
        );
        assert!(
            !written.contains("mailbox-plaintext"),
            "env values must be encrypted at rest; got:\n{written}"
        );

        config
            .set_prop_persistent("mcp.servers.fs.env.GBR_MAILBOX_KEY", "")
            .unwrap();
        config.save_dirty().await.unwrap();
        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            !written.contains("GBR_MAILBOX_KEY"),
            "removed env key must be dropped from disk; got:\n{written}"
        );
        assert!(written.contains("name = \"fs\""), "{written}");
    }

    #[test]
    async fn typed_custom_slot_round_trips_uri_through_save_and_load() {
        // Legacy colon-URL keys (`custom:https://...`) are gone — `custom`
        // is a typed slot whose `uri` field carries the operator URL.
        // This pins: secret routing, save/encrypt, and round-trip reload
        // for the typed `custom` slot.
        let dir = TempDir::new().unwrap();
        let mut config = Config {
            config_path: dir.path().join("config.toml"),
            data_dir: dir.path().join("workspace"),
            ..Default::default()
        };
        let alias = "default";
        config
            .providers
            .models
            .ensure("custom", alias)
            .expect("custom typed slot");

        let prefix = format!("providers.models.custom.{alias}");
        let api_key_path = format!("{prefix}.api_key");
        let uri_path = format!("{prefix}.uri");
        let model_path = format!("{prefix}.model");
        let temperature_path = format!("{prefix}.temperature");

        assert!(
            Config::prop_is_secret(&api_key_path),
            "typed custom-slot api-key must route through the secret marker",
        );

        config.set_prop(&api_key_path, "sk-test-custom").unwrap();
        config
            .set_prop(&uri_path, "https://api.example.invalid/v1")
            .unwrap();
        config.set_prop(&model_path, "local-large").unwrap();
        config.set_prop(&temperature_path, "0.2").unwrap();

        let provider = config
            .providers
            .models
            .find("custom", alias)
            .expect("custom typed slot entry must be present");
        assert_eq!(provider.api_key.as_deref(), Some("sk-test-custom"));
        assert_eq!(
            provider.uri.as_deref(),
            Some("https://api.example.invalid/v1")
        );
        assert_eq!(provider.model.as_deref(), Some("local-large"));
        assert_eq!(provider.temperature, Some(0.2));

        assert_eq!(config.get_prop(&api_key_path).unwrap(), "**** (encrypted)");
        assert_eq!(
            config.get_prop(&uri_path).unwrap(),
            "https://api.example.invalid/v1"
        );

        config.save().await.unwrap();
        let raw_toml = tokio::fs::read_to_string(&config.config_path)
            .await
            .unwrap();
        assert!(
            raw_toml.contains("[providers.models.custom.default]"),
            "saved TOML should write under the typed custom slot",
        );
        assert!(
            !raw_toml.contains("sk-test-custom"),
            "saved TOML must not contain the plaintext custom provider API key",
        );

        let mut loaded: Config = crate::migration::migrate_to_current(&raw_toml).unwrap();
        loaded.config_path = config.config_path.clone();
        loaded.data_dir = config.data_dir.clone();
        let store = crate::secrets::SecretStore::new(dir.path(), loaded.secrets.encrypt);
        loaded.decrypt_secrets(&store).unwrap();
        let loaded_provider = loaded
            .providers
            .models
            .find("custom", alias)
            .expect("typed custom slot entry must round-trip through save/load");
        assert_eq!(loaded_provider.api_key.as_deref(), Some("sk-test-custom"));
        assert_eq!(
            loaded_provider.uri.as_deref(),
            Some("https://api.example.invalid/v1")
        );
        assert_eq!(loaded_provider.model.as_deref(), Some("local-large"));
        assert_eq!(loaded_provider.temperature, Some(0.2));
    }

    #[test]
    async fn env_override_save_cycle_preserves_on_disk_secret() {
        // Regression bar for the data-loss bug identified in PR
        // review: an operator with a real on-disk credential who sets a
        // `OMNESAGENT_*` env override for the same path and triggers any
        // save (dashboard auto-save, CLI `config set` for an unrelated
        // field, Quickstart finalizer) must NOT corrupt the disk file.
        //
        // Pre-fix behavior: `mask_env_overrides_for_save` read disk via
        // `get_prop`, which returns `"**** (encrypted)"` for secret-typed
        // fields regardless of underlying state. That mask string then got
        // re-encrypted as plaintext and written to disk, destroying the
        // operator's real credential on the next reload.
        //
        // Post-fix: `apply_env_overrides` snapshots the post-decrypt
        // plaintext at apply time; `mask_env_overrides_for_save` restores
        // from that snapshot before `encrypt_secrets()` runs. The disk
        // secret survives the cycle.
        let dir = TempDir::new().unwrap();
        let mut config = Config {
            config_path: dir.path().join("config.toml"),
            data_dir: dir.path().join("workspace"),
            ..Default::default()
        };
        let original_secret = "sk-ant-real-on-disk-credential";
        let api_key_path = "providers.models.anthropic.default.api_key";
        config
            .providers
            .models
            .ensure("anthropic", "default")
            .expect("typed slot");
        config.set_prop(api_key_path, original_secret).unwrap();

        // First save: encrypts the original plaintext, writes to disk.
        config.save().await.unwrap();

        // Reload from disk to confirm the original landed correctly.
        let raw = tokio::fs::read_to_string(&config.config_path)
            .await
            .unwrap();
        let mut reloaded: Config = crate::migration::migrate_to_current(&raw).unwrap();
        reloaded.config_path = config.config_path.clone();
        reloaded.data_dir = config.data_dir.clone();
        let store = crate::secrets::SecretStore::new(dir.path(), reloaded.secrets.encrypt);
        reloaded.decrypt_secrets(&store).unwrap();
        assert_eq!(
            reloaded
                .providers
                .models
                .anthropic
                .get("default")
                .and_then(|c| c.base.api_key.as_deref()),
            Some(original_secret),
            "baseline: original secret round-trips through one save/reload cycle",
        );

        // Simulate `apply_env_overrides` having injected a different value
        // for the same path — this is the state `Config::load_or_init`
        // leaves the in-memory config in when an operator boots with
        // `OMNESAGENT_providers__models__anthropic__default__api_key=...`
        // set in the environment.
        let env_value = "sk-ant-from-env-DIFFERENT";
        reloaded.env_overridden_paths = std::collections::HashSet::from([api_key_path.to_string()]);
        reloaded.pre_override_snapshots = std::collections::HashMap::from([(
            api_key_path.to_string(),
            original_secret.to_string(),
        )]);
        reloaded.set_prop(api_key_path, env_value).unwrap();

        // Save again. With the pre-fix code path, this is the moment the
        // disk file got corrupted with the encrypted display mask.
        reloaded.save().await.unwrap();

        // Reload, decrypt, and confirm the original secret survived
        // (and the env value did NOT leak to disk, and the literal mask
        // string was NOT persisted).
        let raw_after = tokio::fs::read_to_string(&reloaded.config_path)
            .await
            .unwrap();
        assert!(
            !raw_after.contains(env_value),
            "env-injected value must never reach disk: {raw_after}",
        );
        assert!(
            !raw_after.contains("**** (encrypted)"),
            "display mask must never be persisted as a secret value: {raw_after}",
        );

        let mut after: Config = crate::migration::migrate_to_current(&raw_after).unwrap();
        after.config_path = reloaded.config_path.clone();
        after.data_dir = reloaded.data_dir.clone();
        let store2 = crate::secrets::SecretStore::new(dir.path(), after.secrets.encrypt);
        after.decrypt_secrets(&store2).unwrap();
        assert_eq!(
            after
                .providers
                .models
                .anthropic
                .get("default")
                .and_then(|c| c.base.api_key.as_deref()),
            Some(original_secret),
            "original on-disk secret must survive an env-override + save cycle",
        );
    }

    #[cfg(unix)]
    #[test]
    async fn onepassword_reference_survives_load_save_cycle() {
        let _env_guard = env_override_lock().await;
        let dir = TempDir::new().unwrap();
        let bin_dir = dir.path().join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        write_fake_op(
            &bin_dir,
            r#"#!/bin/sh
if [ "$1" = "read" ] && [ "$2" = "op://omnesagent/provider/openai-api-key" ]; then
  printf '%s\n' 'sk-proj-from-onepassword'
  exit 0
fi
printf '%s\n' 'unexpected op invocation' >&2
exit 65
"#,
        );
        let path = match std::env::var_os("PATH") {
            Some(existing) if !existing.is_empty() => {
                format!("{}:{}", bin_dir.display(), existing.to_string_lossy())
            }
            _ => bin_dir.display().to_string(),
        };
        let _path_guard = EnvValueGuard::set("PATH", path);
        let _config_guard = EnvValueGuard::set("OMNESAGENT_CONFIG_DIR", dir.path());
        let _workspace_guard = EnvValueGuard::remove("OMNESAGENT_WORKSPACE");

        let config_path = dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            r#"
schema_version = 3

[providers.models.openai.default]
model = "gpt-5"
api_key = "op://omnesagent/provider/openai-api-key"
"#,
        )
        .unwrap();

        let config = Config::load_or_init().await.unwrap();
        assert_eq!(
            config
                .providers
                .models
                .openai
                .get("default")
                .and_then(|entry| entry.base.api_key.as_deref()),
            Some("sk-proj-from-onepassword"),
            "runtime config uses resolved 1Password secret"
        );

        config.save().await.unwrap();
        let raw_after = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            raw_after.contains("op://omnesagent/provider/openai-api-key"),
            "on-disk config must keep the 1Password reference: {raw_after}"
        );
        assert!(
            !raw_after.contains("sk-proj-from-onepassword"),
            "resolved secret must not be written back to disk: {raw_after}"
        );
    }

    #[cfg(unix)]
    #[allow(
        clippy::disallowed_methods,
        reason = "test asserts Tokio worker responsiveness"
    )]
    #[test(flavor = "multi_thread", worker_threads = 1)]
    async fn onepassword_reference_load_does_not_block_runtime_worker() {
        let _env_guard = env_override_lock().await;
        let dir = TempDir::new().unwrap();
        let bin_dir = dir.path().join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        write_fake_op(
            &bin_dir,
            r#"#!/bin/sh
if [ "$1" = "read" ] && [ "$2" = "op://omnesagent/provider/openai-api-key" ]; then
  sleep 3
  printf '%s\n' 'sk-proj-from-onepassword'
  exit 0
fi
exit 65
"#,
        );
        let path = match std::env::var_os("PATH") {
            Some(existing) if !existing.is_empty() => {
                format!("{}:{}", bin_dir.display(), existing.to_string_lossy())
            }
            _ => bin_dir.display().to_string(),
        };
        let _path_guard = EnvValueGuard::set("PATH", path);
        let _config_guard = EnvValueGuard::set("OMNESAGENT_CONFIG_DIR", dir.path());
        let _workspace_guard = EnvValueGuard::remove("OMNESAGENT_WORKSPACE");

        let config_path = dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            r#"
schema_version = 3

[providers.models.openai.default]
model = "gpt-5"
api_key = "op://omnesagent/provider/openai-api-key"
"#,
        )
        .unwrap();

        let started = std::time::Instant::now();
        let load_task = tokio::spawn(Config::load_or_init());
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Threshold sized against the 3s fake-op sleep: a blocked worker pins
        // elapsed at >=3s, while scheduler latency under full-suite CI load
        // stays well under 1.5s.
        assert!(
            started.elapsed() < std::time::Duration::from_millis(1500),
            "op:// config load should not block the async runtime worker"
        );

        let config = load_task.await.unwrap().unwrap();
        assert_eq!(
            config
                .providers
                .models
                .openai
                .get("default")
                .and_then(|entry| entry.base.api_key.as_deref()),
            Some("sk-proj-from-onepassword")
        );
    }

    #[cfg(unix)]
    #[test]
    async fn dirty_onepassword_secret_edit_replaces_reference() {
        let _env_guard = env_override_lock().await;
        let dir = TempDir::new().unwrap();
        let bin_dir = dir.path().join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        write_fake_op(
            &bin_dir,
            r#"#!/bin/sh
printf '%s\n' 'sk-proj-from-onepassword'
"#,
        );
        let path = match std::env::var_os("PATH") {
            Some(existing) if !existing.is_empty() => {
                format!("{}:{}", bin_dir.display(), existing.to_string_lossy())
            }
            _ => bin_dir.display().to_string(),
        };
        let _path_guard = EnvValueGuard::set("PATH", path);
        let _config_guard = EnvValueGuard::set("OMNESAGENT_CONFIG_DIR", dir.path());
        let _workspace_guard = EnvValueGuard::remove("OMNESAGENT_WORKSPACE");

        let config_path = dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            r#"
schema_version = 3

[providers.models.openai.default]
model = "gpt-5"
api_key = "op://omnesagent/provider/openai-api-key"
"#,
        )
        .unwrap();

        let mut config = Config::load_or_init().await.unwrap();
        config
            .set_prop_persistent(
                "providers.models.openai.default.api_key",
                "sk-proj-new-direct-key",
            )
            .unwrap();
        config.save_dirty().await.unwrap();

        let raw_after = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            !raw_after.contains("op://omnesagent/provider/openai-api-key"),
            "dirty secret edits should replace the old 1Password reference: {raw_after}"
        );
        assert!(
            !raw_after.contains("sk-proj-new-direct-key"),
            "direct replacement should still be encrypted at rest: {raw_after}"
        );

        let stored: Config = toml::from_str(&raw_after).unwrap();
        let encrypted = stored
            .providers
            .models
            .openai
            .get("default")
            .and_then(|entry| entry.base.api_key.as_deref())
            .unwrap();
        let store = crate::secrets::SecretStore::new(dir.path(), true);
        assert_eq!(store.decrypt(encrypted).unwrap(), "sk-proj-new-direct-key");
    }

    #[test]
    async fn enum_variants_callback_returns_values() {
        let mx = test_matrix_config();
        let fields = mx.prop_fields();
        let stream_field = fields
            .iter()
            .find(|f| f.name == "channels.matrix.stream_mode")
            .unwrap();
        let variants = (stream_field.enum_variants.unwrap())();
        assert!(variants.contains(&"off".to_string()));
        assert!(variants.contains(&"partial".to_string()));
        assert!(variants.contains(&"single_message".to_string()));
        assert!(variants.contains(&"multi_message".to_string()));
    }

    #[test]
    async fn shared_stream_mode_excludes_matrix_single_message_variant() {
        let shared = serde_json::from_str::<StreamMode>(r#""single_message""#);
        assert!(shared.is_err());

        let matrix = serde_json::from_str::<MatrixStreamMode>(r#""single_message""#).unwrap();
        assert_eq!(matrix, MatrixStreamMode::SingleMessage);
    }

    #[test]
    async fn map_key_sections_discovers_per_family_provider_slots() {
        // Typed-family split: `providers.models` is a struct of typed
        // family maps, not a single open HashMap. Each family slot
        // (`providers.models.<family>`) is its own Map-kind section; the
        // dashboard's "+ Add alias" affordance hangs off the family path.
        let sections = Config::map_key_sections();
        let anthropic = sections
            .iter()
            .find(|s| s.path == "providers.models.anthropic")
            .expect("providers.models.anthropic must be discoverable as a map-keyed section");
        assert_eq!(anthropic.kind, crate::traits::MapKeyKind::Map);
        assert_eq!(anthropic.value_type, "AnthropicModelProviderConfig");

        // agents is also #[nested] HashMap on root Config.
        assert!(
            sections.iter().any(|s| s.path == "agents"),
            "agents map should be discoverable"
        );

        // mcp.servers is a Vec<McpServerConfig> with #[nested] — should
        // surface as a List-kind section so the dashboard's "+ Add MCP
        // server" affordance picks it up. Without this, dashboard users
        // hit a silent dead-end and have to hand-edit config.toml. Pinned
        // here so a regression that drops the #[nested] annotation or the
        // Configurable derive on McpServerConfig fails CI.
        let mcp_servers = sections
            .iter()
            .find(|s| s.path == "mcp.servers")
            .expect("mcp.servers must be discoverable as a list-shaped section");
        assert_eq!(mcp_servers.kind, crate::traits::MapKeyKind::List);
        assert_eq!(mcp_servers.value_type, "McpServerConfig");
    }

    fn assert_map_key_sections_reuse_paths(
        first: &[crate::traits::MapKeySection],
        second: &[crate::traits::MapKeySection],
    ) {
        assert_eq!(first.len(), second.len());
        for (first, second) in first.iter().zip(second) {
            assert_eq!(
                (
                    first.path,
                    first.kind,
                    first.value_type,
                    first.description,
                    first.natural_key,
                    first.resource_key,
                ),
                (
                    second.path,
                    second.kind,
                    second.value_type,
                    second.description,
                    second.natural_key,
                    second.resource_key,
                ),
            );
            assert_eq!(
                first.path.as_ptr(),
                second.path.as_ptr(),
                "schema path {} must reuse its static backing storage",
                first.path,
            );
        }
    }

    #[test]
    async fn map_key_sections_reuses_static_paths() {
        let sections = Config::map_key_sections();
        for path in ["agents", "model_routes", "mcp.servers", "cost.rates.tools"] {
            assert!(sections.iter().any(|section| section.path == path));
        }
        for _ in 0..4 {
            assert_map_key_sections_reuse_paths(&sections, &Config::map_key_sections());
        }
    }

    #[test]
    async fn create_map_key_inserts_default_mcp_server() {
        // Round-trip: `POST /api/config/map-key?path=mcp.servers&key=github`.
        // The new entry's `name` field is initialized to the supplied key
        // by the macro's List-kind insertion logic.
        let mut config = Config::default();
        assert!(config.mcp.servers.is_empty());

        let created = config
            .create_map_key("mcp.servers", "github")
            .expect("mcp.servers should accept new list entries");
        assert!(created, "first add should report created=true");
        assert_eq!(config.mcp.servers.len(), 1);
        assert_eq!(
            config.mcp.servers[0].name, "github",
            "new entry must carry the supplied key as its name field"
        );
    }

    #[test]
    async fn create_map_key_seeds_plugin_entry_and_routes_config_set() {
        // The `omnesagent plugin install` seeding path: a fresh
        // `[[plugins.entries]]` entry named with the canonical instance key
        // must make `config set plugins.entries.<instance>.config.<key>` routable;
        // natural-key path routing only matches keys already present in
        // live config.
        let temp = tempfile::TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");
        let mut config = Config {
            config_path: config_path.clone(),
            ..Config::default()
        };
        let instance_key = "zpi1_WyJmaXh0dXJlLnBsdWdpbiIsInRvb2wiLCJtYWluLnh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh4eHh44KC-Il0";
        let created = config
            .create_map_key("plugins.entries", instance_key)
            .expect("plugins.entries must accept new natural-key entries");
        assert!(created, "first add should report created=true");
        assert_eq!(config.plugins.entries.len(), 1);
        assert_eq!(config.plugins.entries[0].name, instance_key);
        config.mark_dirty(&format!("plugins.entries.{instance_key}"));
        config.save_dirty().await.unwrap();

        config
            .set_prop_persistent(
                &format!("plugins.entries.{instance_key}.config.api_key"),
                "sk-test",
            )
            .expect("config set must route through the seeded entry");
        config.save_dirty().await.unwrap();
        assert_eq!(
            config
                .plugins
                .entry_config(instance_key)
                .unwrap()
                .and_then(|c| c.get("api_key"))
                .map(String::as_str),
            Some("sk-test")
        );
        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(written.contains(instance_key));
        assert!(written.contains("api_key"));
        assert!(
            !written.contains("sk-test"),
            "plugin config values must remain encrypted on disk"
        );

        // Idempotent: reinstalling must not clobber operator values.
        let again = config
            .create_map_key("plugins.entries", instance_key)
            .expect("second add still resolves the section");
        assert!(!again, "duplicate add should report created=false");
        assert_eq!(config.plugins.entries.len(), 1);
        assert_eq!(
            config
                .plugins
                .entry_config(instance_key)
                .unwrap()
                .and_then(|c| c.get("api_key"))
                .map(String::as_str),
            Some("sk-test"),
            "re-seeding must leave existing config values untouched"
        );
    }

    #[test]
    async fn create_map_key_inserts_default_alias_under_typed_family() {
        // Dashboard "+ Add alias" target is the typed family slot,
        // not a free-form provider key under `providers.models`.
        let mut config = Config::default();
        assert!(
            !config
                .providers
                .models
                .contains_model_provider_type("anthropic")
        );

        let created = config
            .create_map_key("providers.models.anthropic", "default")
            .expect("typed family slot should accept a new alias");
        assert!(created, "first add should report created=true");
        assert!(
            config
                .providers
                .models
                .find("anthropic", "default")
                .is_some(),
            "the new alias must show up under the typed family slot",
        );

        // Idempotent: second add returns false, doesn't error.
        let again = config
            .create_map_key("providers.models.anthropic", "default")
            .expect("second add still resolves the section");
        assert!(!again, "duplicate add should report created=false");
    }

    #[test]
    async fn ensure_map_key_for_path_materializes_typed_provider_maps() {
        for (path, value) in [
            ("providers.models.openai.default.model", "gpt-4o"),
            ("providers.tts.openai.default.voice", "alloy"),
            ("providers.transcription.openai.default.model", "whisper-1"),
            ("channels.telegram.default.bot_token", "tok"),
        ] {
            let mut config = Config::default();
            assert!(
                config.set_prop(path, value).is_err(),
                "precondition: {path} is unknown on a fresh config"
            );
            config.ensure_map_key_for_path(path);
            assert!(
                config.set_prop(path, value).is_ok(),
                "{path} must be settable after ensure_map_key_for_path"
            );
        }
    }

    #[test]
    async fn ensure_map_key_for_path_ignores_plain_fields() {
        let mut config = Config::default();
        config.ensure_map_key_for_path("gateway.port");
        config.ensure_map_key_for_path("locale");
        assert!(config.set_prop("gateway.port", "8080").is_ok());
    }

    // ── nested map-routed set_prop must not mask value errors as
    // "Unknown property" ────────────────────────────────────────────────
    //
    // Once the router/key lookup has confirmed a path belongs to a
    // materialized map alias, a failure from the inner `set_prop` call is a
    // real value problem (bad type, bad enum variant, ...) and must
    // propagate as-is rather than being swallowed into the generic
    // "Unknown property" fallback (which downstream consumers, e.g.
    // `omnesagent-gateway`'s `map_prop_error` and `src/main.rs`'s
    // `config_patch_map_prop_error`, translate into a 404 PathNotFound
    // instead of a 400 ValueTypeMismatch).

    #[track_caller]
    fn assert_value_error(err: &str) {
        assert!(
            !err.starts_with("Unknown property"),
            "bad value must not be reported as an unknown path: {err}"
        );
        assert!(
            err.contains("bool")
                || err.contains("Invalid")
                || err.contains("invalid")
                || err.contains("expected"),
            "error should describe the value problem: {err}"
        );
    }

    #[track_caller]
    fn assert_unknown_property(err: &str) {
        assert!(
            err.starts_with("Unknown property"),
            "an unknown leaf must still surface as a path problem (404), got: {err}"
        );
    }

    #[test]
    async fn set_prop_single_level_map_rejects_invalid_value_not_unknown_property() {
        let mut config = Config::default();
        config.ensure_map_key_for_path("channels.telegram.default.bot_token");
        let err = config
            .set_prop("channels.telegram.default.enabled", "notabool")
            .unwrap_err()
            .to_string();
        assert_value_error(&err);
    }

    #[test]
    async fn set_prop_single_level_map_accepts_valid_value() {
        let mut config = Config::default();
        config.ensure_map_key_for_path("channels.telegram.default.bot_token");
        config
            .set_prop("channels.telegram.default.enabled", "true")
            .unwrap();
        assert!(
            config
                .channels
                .telegram
                .get("default")
                .expect("alias materialized by ensure_map_key_for_path")
                .enabled
        );
    }

    #[test]
    async fn set_prop_single_level_map_unknown_leaf_still_unknown_property() {
        let mut config = Config::default();
        config.ensure_map_key_for_path("channels.telegram.default.bot_token");
        let err = config
            .set_prop("channels.telegram.default.nonexistent_field", "x")
            .unwrap_err()
            .to_string();
        assert_unknown_property(&err);
    }

    // The production schema has no `#[nested] HashMap<String, HashMap<String,
    // T: Configurable>>` field today (every `providers.models.<type>` slot is
    // itself a single-level `HashMap<String, T>` field of a plain nested
    // struct, not a hashmap key) — so the two-level routing branch in
    // `derive_configurable` (crates/omnesagent-macros/src/lib.rs, the
    // `double_value_ty` arm) can't be exercised through `Config` directly.
    // Exercise it directly with a minimal local fixture instead.
    #[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
    #[prefix = "dm_sub"]
    struct DoubleMapSub {
        #[serde(default)]
        pub value: bool,
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
    #[prefix = "dm_leaf"]
    struct DoubleMapLeaf {
        #[serde(default)]
        pub flag: bool,
        #[serde(default)]
        #[nested]
        pub sub: DoubleMapSub,
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
    #[prefix = "dm"]
    struct DoubleMapOuter {
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        #[nested]
        pub types: HashMap<String, HashMap<String, DoubleMapLeaf>>,
    }

    #[test]
    async fn map_key_sections_double_map_reuses_static_paths() {
        let sections = DoubleMapOuter::map_key_sections();
        assert_eq!(sections.len(), 1);
        let section = &sections[0];
        assert_eq!(section.path, "dm.types");
        assert_eq!(section.kind, crate::traits::MapKeyKind::Map);
        assert_eq!(section.value_type, "DoubleMapLeaf");
        assert_eq!(section.description, "");
        assert_eq!(section.natural_key, None);
        assert!(!section.resource_key);
        for _ in 0..4 {
            assert_map_key_sections_reuse_paths(&sections, &DoubleMapOuter::map_key_sections());
        }
    }

    fn double_map_fixture() -> DoubleMapOuter {
        let mut outer = DoubleMapOuter::default();
        outer
            .types
            .entry("anthropic".to_string())
            .or_default()
            .insert("default".to_string(), DoubleMapLeaf::default());
        // Dotted-outer-key ambiguity: `dm.types.a.x.sub.value` splits as
        // outer="a.x"/inner="sub" (longest outer key wins, tried first)
        // AND as outer="a"/inner="x" — the candidate loop must be able to
        // retry the shorter split when the longest one dead-ends.
        outer
            .types
            .entry("a".to_string())
            .or_default()
            .insert("x".to_string(), DoubleMapLeaf::default());
        outer
            .types
            .entry("a.x".to_string())
            .or_default()
            .insert("sub".to_string(), DoubleMapLeaf::default());
        outer
    }

    #[test]
    async fn set_prop_double_level_map_rejects_invalid_value_not_unknown_property() {
        let mut outer = double_map_fixture();
        let err = outer
            .set_prop("dm.types.anthropic.default.flag", "notabool")
            .unwrap_err()
            .to_string();
        assert_value_error(&err);
    }

    #[test]
    async fn set_prop_double_level_map_accepts_valid_value() {
        let mut outer = double_map_fixture();
        outer
            .set_prop("dm.types.anthropic.default.flag", "true")
            .unwrap();
        assert!(outer.types["anthropic"]["default"].flag);
    }

    #[test]
    async fn set_prop_double_level_map_unknown_leaf_still_unknown_property() {
        let mut outer = double_map_fixture();
        let err = outer
            .set_prop("dm.types.anthropic.default.nonexistent_field", "x")
            .unwrap_err()
            .to_string();
        assert_unknown_property(&err);
    }

    #[test]
    async fn set_prop_double_level_map_dotted_outer_key_retries_next_candidate() {
        // The longest candidate split (outer="a.x"/inner="sub") is tried
        // first and its leaf lookup yields "Unknown property" —
        // `dm_leaf.value` is not a direct DoubleMapLeaf field. The loop
        // must fall through to outer="a"/inner="x", whose nested
        // `sub.value` resolves — keeping set_prop in agreement with
        // get_prop's retry semantics on the same path.
        let mut outer = double_map_fixture();
        outer.set_prop("dm.types.a.x.sub.value", "true").unwrap();
        assert!(outer.types["a"]["x"].sub.value);
        assert_eq!(outer.get_prop("dm.types.a.x.sub.value").unwrap(), "true");
    }

    // ── regression anchor: repro through the serde(flatten)
    // delegation site (`OpenAIModelProviderConfig { #[serde(flatten)] base }`).
    // A bad-typed value on a flattened base field of a live alias must
    // propagate the value error, not degrade into "Unknown property".

    #[test]
    async fn set_prop_flatten_alias_leaf_rejects_invalid_value_not_unknown_property() {
        let mut config = Config::default();
        config
            .create_map_key("providers.models.openai", "k8")
            .expect("typed family slot accepts a new alias");

        // Happy path first: valid value round-trips.
        config
            .set_prop("providers.models.openai.k8.temperature", "0.5")
            .unwrap();
        assert_eq!(
            config
                .get_prop("providers.models.openai.k8.temperature")
                .unwrap(),
            "0.5"
        );

        // The issue's repro: non-numeric value on the same confirmed path.
        let err = config
            .set_prop("providers.models.openai.k8.temperature", "abc")
            .unwrap_err()
            .to_string();
        assert_value_error(&err);
    }

    #[test]
    async fn set_prop_flatten_alias_unknown_leaf_still_unknown_property() {
        let mut config = Config::default();
        config
            .create_map_key("providers.models.openai", "k8")
            .expect("typed family slot accepts a new alias");
        let err = config
            .set_prop("providers.models.openai.k8.nonexistent_field", "x")
            .unwrap_err()
            .to_string();
        assert_unknown_property(&err);
    }

    #[test]
    async fn set_prop_flatten_own_field_still_resolves_after_base_unknown_property() {
        // AzureModelProviderConfig has BOTH a flattened base and its own
        // direct fields (resource / deployment / api_version). Setting an
        // own-field goes through the flatten site first, which returns
        // "Unknown property" — that must keep falling through so the own
        // field still resolves (the no-over-propagation guarantee).
        let mut config = Config::default();
        config
            .create_map_key("providers.models.azure", "k8")
            .expect("typed family slot accepts a new alias");
        config
            .set_prop("providers.models.azure.k8.resource", "myres")
            .unwrap();
        assert_eq!(
            config
                .providers
                .models
                .azure
                .get("k8")
                .expect("alias created above")
                .resource
                .as_deref(),
            Some("myres")
        );
    }

    #[test]
    async fn set_prop_flatten_base_field_via_azure_alias_propagates_value_error() {
        let mut config = Config::default();
        config
            .create_map_key("providers.models.azure", "k8")
            .expect("typed family slot accepts a new alias");
        let err = config
            .set_prop("providers.models.azure.k8.temperature", "abc")
            .unwrap_err()
            .to_string();
        assert_value_error(&err);
    }

    // ── regression: a genuine value error whose own message starts with
    // "Unknown property" must still propagate, not be reclassified as the
    // generated fall-through marker and swallowed as a retry.
    // `is_unknown_property_error` used to match on that prefix alone, so a
    // validator error crafted to begin the same way would have been
    // misread as "not mine" at the `Option<T>` delegation gate, silently
    // discarded, and reported as an unknown-leaf error instead of the real
    // value problem it is. (The struct-level TOML round-trip that runs the
    // field's custom deserializer wraps the raw message with its own
    // "TOML parse error at line ..." position preamble, so the *final*
    // propagated error contains rather than starts with the crafted text —
    // the point under test is that the crafted sentence survives at all
    // instead of being replaced by the generic `Unknown property '<name>'`
    // fallback a misclassification would produce.)

    fn reject_poison_string_value<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value == "poison" {
            return Err(de::Error::custom(
                "Unknown property value rejected by a custom field validator",
            ));
        }
        Ok(value)
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
    #[prefix = "unk_collision"]
    struct UnknownPropertyCollisionInner {
        #[serde(default, deserialize_with = "reject_poison_string_value")]
        pub label: String,
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
    #[prefix = "unk_collision_outer"]
    struct UnknownPropertyCollisionOuter {
        #[serde(default)]
        #[nested]
        pub inner: Option<UnknownPropertyCollisionInner>,
    }

    #[test]
    async fn set_prop_option_nested_value_error_starting_with_unknown_property_prefix_still_propagates()
     {
        let mut outer = UnknownPropertyCollisionOuter {
            inner: Some(UnknownPropertyCollisionInner::default()),
        };
        let err = outer
            .set_prop("unk_collision.label", "poison")
            .unwrap_err()
            .to_string();
        // The specific validator message must survive — a
        // misclassification would instead produce the generic
        // `Unknown property '<name>'` fallback, which doesn't contain this
        // text.
        assert!(
            err.contains("Unknown property value rejected by a custom field validator"),
            "a genuine value error must propagate, not be masked as an unknown property: {err}"
        );
        // Sanity: confirm this isn't the generic fallback shape (which
        // would also technically satisfy a looser check).
        assert!(
            !err.contains("Unknown property 'unk_collision.label'"),
            "must not have degraded into the generic unknown-property fallback: {err}"
        );
    }

    #[test]
    async fn ensure_map_key_for_path_ignores_resource_keyed_rate_sections() {
        let mut config = Config::default();
        config.ensure_map_key_for_path("cost.rates.providers.models.openai.gpt-4.1.input_per_mtok");
        // The return value proves nothing here: it is `true` only for the
        // reserved-agent refusal, so it is already `false` without the
        // `resource_key` filter. The emptiness check is the regression signal.
        assert!(
            config.cost.rates.providers.models.openai.is_empty(),
            "a leaf write must not auto-create a resource-keyed rate row"
        );
    }

    #[test]
    async fn ensure_map_key_for_path_leaves_dotted_resource_ids_alone() {
        let mut config = Config::default();
        config
            .create_map_key("cost.rates.providers.models.openai", "gpt-4.1")
            .expect("resource-keyed sections accept dotted model ids");
        let path = "cost.rates.providers.models.openai.gpt-4.1.input_per_mtok";
        config.ensure_map_key_for_path(path);
        assert_eq!(
            config
                .get_map_keys("cost.rates.providers.models.openai")
                .expect("known section"),
            vec!["gpt-4.1".to_string()],
            "the first-dot split must not plant a phantom `gpt-4` sibling"
        );
        assert!(
            config.set_prop(path, "1.5").is_ok(),
            "editing a real dotted resource id must still work"
        );
    }

    #[test]
    async fn ensure_map_key_for_path_ignores_cost_rate_tools() {
        let mut config = Config::default();
        config.ensure_map_key_for_path("cost.rates.tools.web_search.per_call");
        assert!(
            config.cost.rates.tools.is_empty(),
            "`cost.rates.tools` is resource-keyed by the tool's registered name"
        );
    }

    #[test]
    async fn ensure_map_key_for_path_rolls_back_alias_when_tail_field_unknown() {
        let mut config = Config::default();
        let path = "providers.models.anthropic.default.not_a_real_field";
        assert!(
            config
                .get_map_keys("providers.models.anthropic")
                .is_some_and(|keys| !keys.iter().any(|k| k == "default")),
            "precondition: alias must not exist yet"
        );

        let refused = config.ensure_map_key_for_path(path);

        assert!(!refused, "not the reserved-agent refusal path");
        assert!(
            config
                .get_map_keys("providers.models.anthropic")
                .is_some_and(|keys| !keys.iter().any(|k| k == "default")),
            "the just-created alias must be rolled back when the tail field can't resolve"
        );
    }

    #[test]
    async fn ensure_map_key_for_path_keeps_preexisting_alias_on_unknown_tail_field() {
        let mut config = Config::default();
        config
            .create_map_key("providers.models.anthropic", "default")
            .expect("seed a pre-existing alias");

        let refused =
            config.ensure_map_key_for_path("providers.models.anthropic.default.not_a_real_field");

        assert!(!refused, "not the reserved-agent refusal path");
        assert!(
            config
                .get_map_keys("providers.models.anthropic")
                .is_some_and(|keys| keys.iter().any(|k| k == "default")),
            "a pre-existing alias must never be deleted for a typo'd tail field"
        );
    }

    #[test]
    async fn ensure_map_key_for_path_preserves_first_dynamic_secret_map_write() {
        let mut config = Config::default();
        let path = "providers.models.openai.fresh.extra_headers.X-Foo";

        let refused = config.ensure_map_key_for_path(path);

        assert!(!refused, "not the reserved-agent refusal path");
        config
            .set_prop(path, "bar")
            .expect("a new dynamic secret-map key must be insertable");
        assert_eq!(
            config
                .providers
                .models
                .openai
                .get("fresh")
                .and_then(|provider| provider.base.extra_headers.get("X-Foo"))
                .map(String::as_str),
            Some("bar")
        );
    }

    #[test]
    async fn create_map_key_rejects_unknown_section() {
        let mut config = Config::default();
        let err = config
            .create_map_key("not.a.real.section", "anything")
            .expect_err("unknown section path should error");
        assert!(err.contains("not.a.real.section"));
    }

    #[test]
    async fn provider_slot_names_match_struct_fields() {
        // TtsProviders/TranscriptionProviders::slot_names are inline lists
        // (their slot macros carry a rate-type param); pin them against the
        // actual serialized field names so adding a family without updating
        // slot_names fails here.
        let tts = toml::Value::try_from(crate::providers::TtsProviders {
            openai: std::iter::once(("a".to_string(), Default::default())).collect(),
            elevenlabs: std::iter::once(("a".to_string(), Default::default())).collect(),
            google: std::iter::once(("a".to_string(), Default::default())).collect(),
            edge: std::iter::once(("a".to_string(), Default::default())).collect(),
            piper: std::iter::once(("a".to_string(), Default::default())).collect(),
        })
        .unwrap();
        let mut tts_fields: Vec<&str> =
            tts.as_table().unwrap().keys().map(String::as_str).collect();
        tts_fields.sort_unstable();
        let mut tts_slots = crate::providers::TtsProviders::slot_names().to_vec();
        tts_slots.sort_unstable();
        assert_eq!(tts_fields, tts_slots);

        let tr = toml::Value::try_from(crate::providers::TranscriptionProviders {
            groq: std::iter::once(("a".to_string(), Default::default())).collect(),
            openai: std::iter::once(("a".to_string(), Default::default())).collect(),
            deepgram: std::iter::once(("a".to_string(), Default::default())).collect(),
            assemblyai: std::iter::once(("a".to_string(), Default::default())).collect(),
            google: std::iter::once(("a".to_string(), Default::default())).collect(),
            local_whisper: std::iter::once(("a".to_string(), Default::default())).collect(),
        })
        .unwrap();
        let mut tr_fields: Vec<&str> = tr.as_table().unwrap().keys().map(String::as_str).collect();
        tr_fields.sort_unstable();
        let mut tr_slots = crate::providers::TranscriptionProviders::slot_names().to_vec();
        tr_slots.sort_unstable();
        assert_eq!(tr_fields, tr_slots);
    }

    #[test]
    async fn retired_wati_config_sections_cover_current_and_legacy_shapes() {
        assert_eq!(
            Config::retired_wati_config_sections(
                "schema_version = 3\n[channels.wati.production]\nenabled = true\n",
            ),
            vec!["channels.wati".to_string()]
        );
        assert_eq!(
            Config::retired_wati_config_sections(
                "[channels_config.wati]\nenabled = true\napi_token = \"placeholder\"\n",
            ),
            vec!["channels_config.wati".to_string()]
        );
        assert!(
            Config::retired_wati_config_sections(
                "schema_version = 3\n[channels.whatsapp.production]\nenabled = true\n",
            )
            .is_empty()
        );
        assert!(Config::retired_wati_config_sections("not toml {{{").is_empty());
    }

    #[test]
    async fn retired_node_transport_detector_keeps_only_presence() {
        assert!(Config::has_retired_node_transport_config(
            "schema_version = 3\n[node_transport]\nshared_secret = \"sentinel-secret\"\n",
        ));
        assert!(!Config::has_retired_node_transport_config(
            "schema_version = 3\n[nodes]\nenabled = true\n",
        ));
        assert!(!Config::has_retired_node_transport_config("not toml {{{"));
    }

    #[test]
    async fn retired_node_transport_absent_from_current_schema_and_defaults() {
        let serialized = toml::to_string(&Config::default()).expect("default config serializes");
        assert!(!serialized.contains("node_transport"));

        #[cfg(feature = "schema-export")]
        {
            let schema = schemars::schema_for!(Config);
            let schema_json = serde_json::to_value(&schema).expect("schema serializes");
            let properties = schema_json
                .get("properties")
                .and_then(serde_json::Value::as_object)
                .expect("Config schema has top-level properties");
            assert!(!properties.contains_key("node_transport"));
        }
    }

    #[test]
    async fn unknown_provider_families_flags_silent_serde_drop() {
        // serde ignores unknown keys under providers.models, so a typo'd
        // family parses cleanly and its aliases vanish on reload. The
        // detector must flag it; known families must pass.
        let raw = r#"
schema_version = 3

[providers.models.antropic.main]
model = "claude-sonnet-4-6"

[providers.models.openai.work]
model = "gpt-4o"
"#;
        let parsed: Config = toml::from_str(raw).expect("unknown family must not fail parse");
        assert!(
            parsed.providers.models.find("antropic", "main").is_none(),
            "precondition: serde silently drops the unknown family"
        );
        assert_eq!(
            Config::unknown_provider_families(raw),
            vec!["models.antropic".to_string()]
        );
        assert_eq!(
            Config::unknown_provider_families(
                "schema_version = 3\n[providers.tts.bogustts.x]\nenabled = true\n",
            ),
            vec!["tts.bogustts".to_string()]
        );
        assert!(Config::unknown_provider_families("not even toml {{{").is_empty());
        // Hostile shapes: scalar providers node, scalar kind node,
        // array-of-tables family. as_table() filters all of them; the
        // detector must stay silent rather than panic or false-positive.
        assert!(Config::unknown_provider_families("providers = 3\n").is_empty());
        assert!(Config::unknown_provider_families("[providers]\nmodels = 3\n").is_empty());
        assert_eq!(
            Config::unknown_provider_families("[[providers.models.weird]]\nx = 1\n"),
            vec!["models.weird".to_string()],
            "array-of-tables under an unknown family is still an unknown family"
        );
    }

    #[test]
    async fn extra_nested_model_provider_tables_flags_dropped_provider_fields() {
        let finding = |family: &str, alias: &str, nested: &str| ExtraNestedModelProviderTable {
            family: family.to_string(),
            alias: alias.to_string(),
            nested: nested.to_string(),
        };

        // serde accepts `[providers.models.zai.default.default]` by treating
        // the first `default` as the provider alias and erasing the second
        // child table. The typed Config therefore looks like an empty
        // `zai.default` provider and `validate()` accepts it as an
        // in-progress quickstart entry. The raw-TOML detector must preserve
        // that diagnostic signal before deserialization erases the shape.
        let raw = r#"
schema_version = 3

[providers.models.zai.default.default]
model = "glm-5.1"
api_key = "sk-test"
endpoint = "global"

[risk_profiles.default]
level = "supervised"

[agents.default]
enabled = true
model_provider = "zai.default"
risk_profile = "default"
"#;
        let parsed: Config = toml::from_str(raw).expect("extra nesting must not fail parse");
        let provider = parsed
            .providers
            .models
            .find("zai", "default")
            .expect("outer alias is still present");
        assert!(
            provider.model.is_none() && provider.api_key.is_none(),
            "precondition: serde silently drops the nested model/api_key"
        );
        parsed
            .validate()
            .expect("typed validation cannot see the raw extra nesting");
        assert_eq!(
            Config::extra_nested_model_provider_tables(raw),
            vec![finding("zai", "default", "default")]
        );

        let valid = r#"
schema_version = 3

[providers.models.zai.default]
model = "glm-5.1"
api_key = "sk-test"
endpoint = "global"
"#;
        assert!(
            Config::extra_nested_model_provider_tables(valid).is_empty(),
            "valid V3 alias tables must not be flagged"
        );

        let valid_table_fields = r#"
schema_version = 3

[providers.models.openai.default]
model = "gpt-4o"

[providers.models.openai.default.extra_headers]
model = "header-value"
api_key = "header-value"

[providers.models.openai.default.provider_extra]
model = "router-model"
api_key = "provider-extra-value"

[providers.models.openai.default.chat_template_kwargs]
model = "template-model"
api_key = "template-value"

[providers.models.openai.default.pricing]
model = 1.0
api_key = 2.0
"#;
        assert!(
            Config::extra_nested_model_provider_tables(valid_table_fields).is_empty(),
            "valid table-valued provider fields must not be flagged even when child keys collide"
        );

        let valid_pricing = r#"
schema_version = 3

[providers.models.openai.default]
model = "gpt-4o"
pricing = { "gpt-4o.input" = 5.0, "gpt-4o.output" = 15.0 }
"#;
        assert!(
            Config::extra_nested_model_provider_tables(valid_pricing).is_empty(),
            "valid nested provider fields such as pricing must not be flagged"
        );

        let family_specific = r#"
schema_version = 3

[providers.models.azure.default.default]
api_version = "2024-10-21"

[providers.models.ollama.local.default]
num_ctx = 16384
"#;
        assert_eq!(
            Config::extra_nested_model_provider_tables(family_specific),
            vec![
                finding("azure", "default", "default"),
                finding("ollama", "local", "default")
            ],
            "family-specific fields must still be flagged when extra-nested"
        );

        let dotted_alias = r#"
schema_version = 3

[providers.models.openai."prod.v2".default]
model = "gpt-4o"
"#;
        assert_eq!(
            Config::extra_nested_model_provider_tables(dotted_alias),
            vec![finding("openai", "prod.v2", "default")],
            "aliases containing dots must stay intact in diagnostics"
        );

        assert!(
            Config::extra_nested_model_provider_tables(
                "schema_version = 3\n[providers.models.zia.default.default]\nmodel = \"x\"\n",
            )
            .is_empty(),
            "unknown families are handled by unknown_provider_families"
        );
        assert!(Config::extra_nested_model_provider_tables("not toml {{{").is_empty());
        assert!(Config::extra_nested_model_provider_tables("providers = 3\n").is_empty());
    }

    #[test]
    async fn map_key_create_survives_incremental_save() {
        // Repro for the omnescode "providers vanish after restart" report:
        // the RPC config/map-key-create path is create_map_key + mark_dirty
        // + save_dirty. The new alias must reach config.toml, otherwise it
        // exists only in-memory and a daemon restart silently drops it
        // (and any agents.*.model_provider referencing it dangles).
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Seed a non-empty on-disk file so the incremental path runs, not
        // the new-file fallback to full save().
        std::fs::write(
            &config_path,
            "schema_version = 9\n\n[observability]\nbackend = \"none\"\n",
        )
        .unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        let created = config
            .create_map_key("providers.models.openai", "myalias")
            .expect("typed family slot accepts a new alias");
        assert!(created);
        assert_eq!(
            config
                .providers
                .models
                .find("openai", "myalias")
                .and_then(|e| e.wire_api),
            Some(WireApi::Responses),
            "new OpenAI provider slots default to wire_api = responses"
        );
        config.mark_dirty("providers.models.openai.myalias");
        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        let reloaded: Config = toml::from_str(&written)
            .unwrap_or_else(|e| panic!("rewritten config must reparse: {e}\n---\n{written}"));
        assert!(
            reloaded
                .providers
                .models
                .find("openai", "myalias")
                .is_some(),
            "created alias must survive save_dirty + reload; got:\n{written}"
        );
        assert_eq!(
            reloaded
                .providers
                .models
                .find("openai", "myalias")
                .and_then(|e| e.wire_api),
            Some(WireApi::Responses),
            "default wire_api must survive save_dirty + reload; got:\n{written}"
        );
    }

    #[test]
    async fn telegram_alias_create_survives_incremental_save() {
        // Regression test: create_map_key seeds TelegramConfig::default()
        // (bot_token = ""), save_dirty's prune_empty_leaves then strips the
        // empty bot_token from the written TOML, and on reload the alias
        // must still deserialize (bot_token now has #[serde(default)])
        // instead of being silently salvage-dropped.
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Seed a non-empty on-disk file so the incremental path runs, not
        // the new-file fallback to full save().
        std::fs::write(
            &config_path,
            "schema_version = 9\n\n[observability]\nbackend = \"none\"\n",
        )
        .unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        let created = config
            .create_map_key("channels.telegram", "myalias")
            .expect("map-keyed section accepts a new alias");
        assert!(created);
        config.mark_dirty("channels.telegram.myalias");
        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        let reloaded: Config = toml::from_str(&written)
            .unwrap_or_else(|e| panic!("rewritten config must reparse: {e}\n---\n{written}"));
        assert!(
            reloaded.channels.telegram.contains_key("myalias"),
            "created telegram alias must survive save_dirty + reload; got:\n{written}"
        );
    }

    #[test]
    async fn discord_alias_create_survives_incremental_save() {
        // Discord twin of telegram_alias_create_survives_incremental_save:
        // DiscordConfig.bot_token has the same serde default.
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Seed a non-empty on-disk file so the incremental path runs, not
        // the new-file fallback to full save().
        std::fs::write(
            &config_path,
            "schema_version = 9\n\n[observability]\nbackend = \"none\"\n",
        )
        .unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };
        let created = config
            .create_map_key("channels.discord", "myalias")
            .expect("map-keyed section accepts a new alias");
        assert!(created);
        config.mark_dirty("channels.discord.myalias");
        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        let reloaded: Config = toml::from_str(&written)
            .unwrap_or_else(|e| panic!("rewritten config must reparse: {e}\n---\n{written}"));
        assert!(
            reloaded.channels.discord.contains_key("myalias"),
            "created discord alias must survive save_dirty + reload; got:\n{written}"
        );
    }

    #[test]
    async fn init_defaults_instantiates_none_sections() {
        let mut config = Config::default();
        assert!(config.channels.matrix.is_empty());

        // Channels are HashMaps — init_defaults cannot insert a default key
        // (there is no meaningful default alias). Callers use create_map_key.
        config
            .create_map_key("channels.matrix", "default")
            .expect("create_map_key should insert a default matrix entry");
        assert!(
            config.channels.matrix.contains_key("default"),
            "create_map_key must add the 'default' alias"
        );

        // init_defaults on an already-populated map section is a no-op.
        let initialized = config.init_defaults(Some("channels.matrix"));
        assert!(
            !initialized.contains(&"channels.matrix"),
            "init_defaults should not report channels.matrix when entry already exists"
        );
    }

    #[test]
    async fn create_map_key_inserts_matrix_streaming_defaults() {
        let mut config = Config::default();
        config
            .create_map_key("channels.matrix", "default")
            .expect("create_map_key should insert a default matrix entry");

        let matrix = config
            .channels
            .matrix
            .get("default")
            .expect("matrix default alias should exist");
        assert_eq!(matrix.stream_mode, MatrixStreamMode::Off);
        assert_eq!(matrix.draft_update_interval_ms, 1500);
        assert_eq!(matrix.multi_message_delay_ms, 800);
        assert_eq!(matrix.stream_draft_lines, 10);
        assert_eq!(matrix.message_max_bytes, 48_000);
        assert!(matrix.stream_draft_delete);
    }

    #[test]
    async fn deserialized_matrix_set_prop_round_trips_vec_string() {
        // Mirror the real-world daemon flow: config loaded from disk where
        // [channels.matrix] is present (possibly with all default fields),
        // then a PATCH from the dashboard hits set_prop.
        let toml_src = r#"
schema_version = 3

[channels.matrix.default]
enabled = false
homeserver = ""
access_token = ""
allowed_rooms = []
allowed_users = []
"#;
        let mut config: Config = toml::from_str(toml_src).expect("parse toml");
        assert!(
            config.channels.matrix.contains_key("default"),
            "matrix must have a 'default' alias after deserialize"
        );

        config
            .set_prop(
                "channels.matrix.default.allowed_rooms",
                r#"["alice","bob"]"#,
            )
            .expect("set_prop should succeed against deserialized matrix");
        assert_eq!(
            config.channels.matrix.get("default").unwrap().allowed_rooms,
            vec!["alice".to_string(), "bob".to_string()],
        );
    }

    #[test]
    async fn init_defaults_then_set_prop_round_trips_vec_string() {
        // Regression for Channels picker → form → save:
        // 1. create_map_key inserts channels.matrix["default"] = MatrixConfig::default()
        // 2. set_prop on channels.matrix.default.allowed_rooms must accept a JSON-array
        //    string (the shape coerce_for_set_prop emits for Vec<String>).
        // 3. get_prop reads it back.
        let mut config = Config::default();
        config
            .create_map_key("channels.matrix", "default")
            .expect("create_map_key should insert a default matrix entry");
        assert!(config.channels.matrix.contains_key("default"));

        // prop_fields must surface the kebab path so the form can render it.
        let has_field = config
            .prop_fields()
            .iter()
            .any(|f| f.name == "channels.matrix.default.allowed_rooms");
        assert!(
            has_field,
            "channels.matrix.default.allowed_rooms must appear in prop_fields after init"
        );

        // set_prop with the JSON-array string the gateway PATCH path produces.
        config
            .set_prop(
                "channels.matrix.default.allowed_rooms",
                r#"["alice","bob"]"#,
            )
            .expect("set_prop should accept JSON-array string for Vec<String>");
        assert_eq!(
            config.channels.matrix.get("default").unwrap().allowed_rooms,
            vec!["alice".to_string(), "bob".to_string()],
        );
    }

    #[test]
    async fn mcp_servers_addable_via_create_map_key_and_per_entry_props() {
        // `mcp.servers` is a `Vec<McpServerConfig>` with `#[nested]`, so the
        // `Configurable` derive surfaces it as a List section (not an
        // ObjectArray prop) — operators add servers via
        // `POST /api/config/map-key?path=mcp.servers&key=<name>` and edit
        // each server's fields via per-prop GET/PUT.
        //
        // This replaces the prior model where the entire Vec round-tripped
        // through set_prop("mcp.servers", "<json-array>"). The List model
        // matches the rest of the schema (`providers.models`, `agents`,
        // etc.) and gives the dashboard a per-field editor instead of a
        // monolithic JSON blob.
        let mut config = Config::default();

        // The List section is discoverable.
        let sections = Config::map_key_sections();
        assert!(
            sections
                .iter()
                .any(|s| s.path == "mcp.servers" && s.kind == crate::traits::MapKeyKind::List),
            "mcp.servers should surface as a List section in map_key_sections()"
        );

        // create_map_key inserts a default-valued entry and seeds its
        // `name` field from the supplied key.
        config
            .create_map_key("mcp.servers", "fs")
            .expect("mcp.servers should accept new list entries via create_map_key");
        assert_eq!(config.mcp.servers.len(), 1);
        assert_eq!(config.mcp.servers[0].name, "fs");

        // Per-entry fields are mutated via standard set_prop on the inner
        // path; routing goes through the `#[natural_key = "name"]` arm
        // on `McpConfig::servers` (see `route_vec_path` and the
        // `Configurable` derive's natural-key arm for the wiring).
        config
            .set_prop("mcp.servers.fs.command", "/usr/bin/mcp-fs")
            .expect("set_prop on mcp.servers.fs.command should route through natural-key arm");
        assert_eq!(config.mcp.servers[0].command, "/usr/bin/mcp-fs");

        // Round-trip via get_prop.
        let got = config.get_prop("mcp.servers.fs.command").expect(
            "get_prop on mcp.servers.fs.command should resolve through the natural-key arm",
        );
        assert_eq!(got, "/usr/bin/mcp-fs");

        // Enum-typed fields (transport) parse from their wire form.
        config
            .set_prop("mcp.servers.fs.transport", "http")
            .expect("transport should accept its enum variants as strings");
        assert_eq!(
            config.mcp.servers[0].transport,
            crate::schema::McpTransport::Http
        );

        // The natural-key field itself is read-only via set_prop — the
        // routing arm returns an explicit error pointing at
        // config_map_key_rename rather than mutating `name` in place
        // (which would silently re-key the entry and strand any
        // in-flight references to the old key).
        let err = config
            .set_prop("mcp.servers.fs.name", "filesystem")
            .expect_err("set_prop on the natural-key field must refuse");
        let msg = err.to_string();
        assert!(
            msg.contains("natural key")
                && msg.contains("read-only")
                && msg.contains("config_map_key_rename"),
            "unexpected error message for read-only natural-key set: {msg}"
        );

        // Rename via the dedicated path. The element's `name` field
        // changes in place; subsequent prop access uses the new key.
        let renamed = config
            .rename_map_key("mcp.servers", "fs", "filesystem")
            .expect("rename should succeed when the new key is free");
        assert!(renamed, "rename_map_key should report Ok(true) on success");
        assert_eq!(config.mcp.servers[0].name, "filesystem");
        assert_eq!(
            config.get_prop("mcp.servers.filesystem.command").unwrap(),
            "/usr/bin/mcp-fs"
        );

        // The old key no longer resolves — confirming the rename was
        // not just an alias add.
        assert!(
            config.get_prop("mcp.servers.fs.command").is_err(),
            "old natural key should stop resolving after rename"
        );

        // prop_fields enumerates the per-element fields under the
        // current natural key, and filters out the natural-key field
        // itself (no editable `name` row in the TUI).
        let paths: Vec<String> = config
            .prop_fields()
            .into_iter()
            .map(|f| f.name)
            .filter(|n| n.starts_with("mcp.servers."))
            .collect();
        assert!(
            paths.iter().any(|n| n == "mcp.servers.filesystem.command"),
            "prop_fields should surface per-element child props; got: {paths:?}"
        );
        assert!(
            !paths.iter().any(|n| n == "mcp.servers.filesystem.name"),
            "prop_fields must hide the natural-key field to keep it read-only; got: {paths:?}"
        );

        // delete_map_key by natural key removes the matching element.
        let deleted = config
            .delete_map_key("mcp.servers", "filesystem")
            .expect("delete by natural key should resolve");
        assert!(deleted);
        assert!(config.mcp.servers.is_empty());
    }

    #[test]
    async fn mcp_servers_create_map_key_is_idempotent_on_existing_natural_key() {
        // Regression for the per-field editor contract: the rest of the
        // natural-key surface (`get_prop` / `set_prop` / `rename_map_key`)
        // treats duplicate natural keys as `VecRoute::Ambiguous` and
        // refuses to mutate (see `mcp_servers_routing_is_ambiguous_on_
        // duplicate_names`). If `create_map_key` always appended, a UI
        // retry — or any caller that re-issued the same add after an
        // uncertain RPC response — would drop `mcp.servers` into that
        // invalid state and leave `mcp.servers.<name>.command` no longer
        // routing until the operator hand-repaired the duplicate.
        //
        // The contract is the same as the `HashMap<String, T>` arm:
        // re-adding an existing key is `Ok(false)` (idempotent no-op),
        // not "append a second element that happens to share the key".
        let mut config = Config::default();

        let first = config
            .create_map_key("mcp.servers", "fs")
            .expect("first add should succeed");
        assert!(first, "first add should report created=true");
        assert_eq!(config.mcp.servers.len(), 1);
        assert_eq!(config.mcp.servers[0].name, "fs");

        // Seed an inner field so we can prove the existing entry's
        // state is preserved across the no-op second add (rather than,
        // say, the second call clobbering it with a default).
        config
            .set_prop("mcp.servers.fs.command", "/usr/bin/mcp-fs")
            .expect("set_prop on the freshly-added entry should route");

        // Repeat add for the same natural key. Must report Ok(false)
        // and must not push a second element.
        let second = config
            .create_map_key("mcp.servers", "fs")
            .expect("repeat add for an existing natural key must not error");
        assert!(
            !second,
            "repeat add for an existing natural key should report created=false (idempotent)"
        );
        assert_eq!(
            config.mcp.servers.len(),
            1,
            "repeat add must not append a duplicate; got {} entries",
            config.mcp.servers.len()
        );
        assert_eq!(config.mcp.servers[0].name, "fs");

        // The natural-key surface still routes — the duplicate was
        // never created, so `set_prop` / `get_prop` are not in the
        // `VecRoute::Ambiguous` state. The previously-set command
        // round-trips and a new edit lands on the original entry.
        assert_eq!(
            config.get_prop("mcp.servers.fs.command").unwrap(),
            "/usr/bin/mcp-fs",
            "existing entry's state must survive the no-op second add"
        );
        config
            .set_prop("mcp.servers.fs.command", "/usr/local/bin/mcp-fs")
            .expect("set_prop must keep routing after the repeat add");
        assert_eq!(config.mcp.servers[0].command, "/usr/local/bin/mcp-fs");

        // A genuinely new key is still added: idempotency is per-key,
        // not "first add wins everything".
        let third = config
            .create_map_key("mcp.servers", "github")
            .expect("a distinct natural key should still be addable");
        assert!(third, "distinct-key add should report created=true");
        assert_eq!(config.mcp.servers.len(), 2);
    }

    #[test]
    async fn mcp_servers_routing_is_ambiguous_on_duplicate_names() {
        // `validate_mcp_config` rejects duplicate `name` at save time,
        // but until the operator repairs the config the in-flight
        // routing must refuse to silently mutate one of the duplicates.
        // This is the schema-side anchor for that contract; the helper
        // function's behaviour is unit-tested directly in
        // `helpers::tests::route_vec_path_reports_ambiguous_duplicates`.
        let mut config = Config::default();
        config.mcp.servers.push(McpServerConfig {
            name: "dupe".into(),
            transport: McpTransport::Stdio,
            command: "/a".into(),
            ..Default::default()
        });
        config.mcp.servers.push(McpServerConfig {
            name: "dupe".into(),
            transport: McpTransport::Stdio,
            command: "/b".into(),
            ..Default::default()
        });

        let set_err = config
            .set_prop("mcp.servers.dupe.command", "/c")
            .expect_err("set_prop on a duplicated natural key must refuse");
        assert!(
            set_err.to_string().contains("ambiguous"),
            "expected ambiguity error, got: {set_err}"
        );

        let get_err = config
            .get_prop("mcp.servers.dupe.command")
            .expect_err("get_prop on a duplicated natural key must refuse");
        assert!(
            get_err.to_string().contains("ambiguous"),
            "expected ambiguity error, got: {get_err}"
        );

        // Neither side mutated the underlying state.
        assert_eq!(config.mcp.servers[0].command, "/a");
        assert_eq!(config.mcp.servers[1].command, "/b");

        // rename_map_key likewise refuses, with an actionable message.
        let rename_err = config
            .rename_map_key("mcp.servers", "dupe", "ok")
            .expect_err("rename of a duplicated natural key must refuse");
        assert!(
            rename_err.contains("ambiguous"),
            "expected ambiguity error from rename, got: {rename_err}"
        );
    }

    #[test]
    async fn mcp_servers_rename_refuses_when_new_key_is_taken() {
        let mut config = Config::default();
        config.create_map_key("mcp.servers", "fs").unwrap();
        config.create_map_key("mcp.servers", "github").unwrap();

        let err = config
            .rename_map_key("mcp.servers", "fs", "github")
            .expect_err("rename should refuse when the target key already exists");
        assert!(
            err.contains("already exists"),
            "expected target-collision error, got: {err}"
        );

        // State untouched.
        assert_eq!(config.mcp.servers.len(), 2);
        assert_eq!(config.mcp.servers[0].name, "fs");
        assert_eq!(config.mcp.servers[1].name, "github");
    }

    #[test]
    async fn mcp_servers_get_map_keys_lists_natural_keys_in_insertion_order() {
        let mut config = Config::default();
        config.create_map_key("mcp.servers", "a").unwrap();
        config.create_map_key("mcp.servers", "b").unwrap();
        config.create_map_key("mcp.servers", "c").unwrap();

        let keys = config
            .get_map_keys("mcp.servers")
            .expect("mcp.servers must surface its natural keys via get_map_keys");
        assert_eq!(
            keys,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    async fn init_defaults_skips_already_set() {
        let mut config = Config::default();
        config
            .channels
            .matrix
            .insert("default".to_string(), test_matrix_config());

        let initialized = config.init_defaults(Some("channels.matrix"));
        // Already set — should not re-initialize
        assert!(!initialized.contains(&"channels.matrix"));
        // Original value preserved
        assert_eq!(
            config.channels.matrix.get("default").unwrap().homeserver,
            "https://m.org"
        );
    }

    /// Regression: a bare `omnesagent config init` (no
    /// section — `init_defaults(None)`) must produce a config.toml that
    /// strictly reloads. Before the fix, `init_defaults` unconditionally
    /// scaffolded every `#[nested] Option<T>` field from
    /// `T::default()`, including structs with a required non-defaulted
    /// `String` leaf (`GatewayTlsConfig::cert_path`/`key_path`,
    /// `LocalWhisperConfig::url`, `OpenVpnTunnelConfig::config_file`).
    /// Those leaves default to `""`, which `prune_empty_leaves` strips
    /// on save, leaving a partial sub-table (kept alive by a non-empty
    /// sibling like `enabled`/`max_audio_bytes`/`connect_timeout_secs`)
    /// that fails strict deserialization with `missing field ...` — the
    /// exact failure `omnesagent config migrate` hits and exits 1 on.
    ///
    /// Mirrors the production boundary of
    /// `local_whisper_config_init_preserves_transcription_section`
    /// (crates/omnesagent-channels/src/transcription.rs) but drives a bare
    /// full init instead of an exact-prefix one.
    #[test]
    async fn config_init_full_produces_strict_roundtrippable_config() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        // Pre-create config.toml (mirrors `load_or_init`) so `save_dirty`
        // below takes the incremental existing-document path, not the
        // full-save fallback for a missing file.
        Config {
            config_path: config_path.clone(),
            ..Config::default()
        }
        .save()
        .await
        .unwrap();

        // The real scaffold: bare `config init`, no section targeted —
        // the exact call `ConfigCommands::Init { section: None }` makes.
        let mut config = Config {
            config_path: config_path.clone(),
            ..Config::default()
        };
        let initialized = config.init_defaults(None);

        // The three required-field sections must be omitted by a bare
        // init (that is the fix); the fully-defaulted siblings must still
        // be scaffolded. Asserting both sides here catches an over-broad
        // `init_requires_explicit_config` predicate that skips too much —
        // which the strict-reload assert below would otherwise pass
        // silently (a missing section only looks "even more absent").
        assert!(
            !initialized.iter().any(|s| {
                *s == "gateway.tls" || *s == "transcription.local_whisper" || *s == "tunnel.openvpn"
            }),
            "required-field sections must be omitted by bare init, got: {initialized:?}"
        );
        assert!(
            initialized.contains(&"transcription.openai")
                && initialized.contains(&"tunnel.tailscale"),
            "fully-defaulted sibling sections must still scaffold on bare init, got: {initialized:?}"
        );

        for s in &initialized {
            config.mark_dirty(s);
        }
        config.save_dirty().await.unwrap();

        let contents = std::fs::read_to_string(&config_path).unwrap();

        // STRICT assert — this is what `config migrate` runs; it exits 1
        // today on unpatched code.
        assert!(
            crate::migration::migrate_to_current(&contents).is_ok(),
            "fresh full `config init` must strictly deserialize; err: {:?}",
            crate::migration::migrate_to_current(&contents).err()
        );

        // RESILIENT assert — even the salvage path must not have to drop
        // an entire parent section on a fresh, untouched init.
        let salv = crate::migration::migrate_to_current_salvaged(&contents);
        assert!(
            !salv
                .dropped
                .iter()
                .any(|p| p == "gateway" || p == "transcription" || p == "tunnel"),
            "no parent section may be salvage-reset on a fresh init, dropped: {:?}",
            salv.dropped
        );
    }

    /// The gate's other side: explicitly targeting a required-field
    /// section (as `omnesagent config init <section>` and the dashboard
    /// section picker both do) must still scaffold it so the operator can
    /// fill in the required fields. `transcription.local_whisper` already
    /// has this coverage via the channels-crate test
    /// `local_whisper_config_init_preserves_transcription_section`; this
    /// locks the same behavior for the two remaining sections.
    #[test]
    async fn config_init_explicit_prefix_still_scaffolds_required_field_sections() {
        for section in ["gateway.tls", "tunnel.openvpn"] {
            let tmp = tempfile::TempDir::new().unwrap();
            let mut config = Config {
                config_path: tmp.path().join("config.toml"),
                ..Config::default()
            };
            let initialized = config.init_defaults(Some(section));
            assert!(
                initialized.contains(&section),
                "explicit `config init {section}` must scaffold the section, got: {initialized:?}"
            );
        }
    }

    #[test]
    async fn nested_get_set_prop_traverses_config_tree() {
        let mut config = Config::default();
        config
            .channels
            .matrix
            .insert("default".to_string(), test_matrix_config());

        // get_prop traverses Config → ChannelsConfig → channels.matrix["default"] → MatrixConfig
        assert_eq!(
            config
                .get_prop("channels.matrix.default.homeserver")
                .unwrap(),
            "https://m.org"
        );

        // set_prop traverses the same path
        config
            .set_prop("channels.matrix.default.homeserver", "https://new.org")
            .unwrap();
        assert_eq!(
            config.channels.matrix.get("default").unwrap().homeserver,
            "https://new.org"
        );
    }

    #[test]
    async fn hashmap_nested_encrypt_decrypt_traverses_values() {
        let dir = TempDir::new().unwrap();
        let store = crate::secrets::SecretStore::new(dir.path(), true);

        let mut config = Config::default();
        config.providers.models.openrouter.insert(
            "test".into(),
            crate::schema::OpenRouterModelProviderConfig {
                base: ModelProviderConfig {
                    api_key: Some("secret-key".into()),
                    ..Default::default()
                },
            },
        );

        config.encrypt_secrets(&store).unwrap();
        let encrypted_key = config
            .providers
            .models
            .find("openrouter", "test")
            .expect("entry exists")
            .api_key
            .as_ref()
            .unwrap();
        assert!(crate::secrets::SecretStore::is_encrypted(encrypted_key));

        config.decrypt_secrets(&store).unwrap();
        assert_eq!(
            config
                .providers
                .models
                .find("openrouter", "test")
                .expect("entry exists")
                .api_key
                .as_deref(),
            Some("secret-key")
        );
    }

    #[test]
    async fn vec_secret_encrypt_decrypt_traverses_elements() {
        let dir = TempDir::new().unwrap();
        let store = crate::secrets::SecretStore::new(dir.path(), true);

        let mut config = Config::default();
        config.gateway.paired_tokens = vec!["token-a".into(), "token-b".into()];

        config.encrypt_secrets(&store).unwrap();
        for token in &config.gateway.paired_tokens {
            assert!(crate::secrets::SecretStore::is_encrypted(token));
        }

        config.decrypt_secrets(&store).unwrap();
        assert_eq!(config.gateway.paired_tokens, vec!["token-a", "token-b"]);
    }

    /// Walk every property on a default Config: get_prop must succeed,
    /// and set_prop must round-trip for non-secret, non-enum scalar fields.
    #[test]
    async fn every_prop_is_gettable_and_settable() {
        let mut config = Config::default();
        // Initialize all Option<T> sections so their fields are reachable
        config.init_defaults(None);

        let fields = config.prop_fields();
        assert!(
            fields.len() > 50,
            "Expected 50+ props, got {} — macro may be skipping fields",
            fields.len()
        );

        for field in &fields {
            // get_prop must not panic or error
            let get_result = config.get_prop(&field.name);
            assert!(
                get_result.is_ok(),
                "get_prop failed for '{}': {}",
                field.name,
                get_result.unwrap_err()
            );

            // set_prop: round-trip the display value back through set_prop.
            // Skip secrets (masked), enums (need valid variant), and <unset> Options.
            if field.is_secret
                || field.is_enum()
                || field.display_value == crate::traits::UNSET_DISPLAY
            {
                continue;
            }

            let set_result = config.set_prop(&field.name, &field.display_value);
            assert!(
                set_result.is_ok(),
                "set_prop failed for '{}' with value '{}': {}",
                field.name,
                field.display_value,
                set_result.unwrap_err()
            );

            // Value should survive the round-trip
            let after = config.get_prop(&field.name).unwrap();
            assert_eq!(
                after, field.display_value,
                "round-trip mismatch for '{}': set '{}', got '{}'",
                field.name, field.display_value, after
            );
        }
    }

    /// Audit gate: every path emitted by `prop_fields()` must round-trip
    /// through `get_prop`. The CLI (`omnesagent config get/set`), the TUI
    /// Quickstart prompts (`prompt_field`), the gateway list endpoint
    /// (`/api/config/list`), and the dashboard form all derive from
    /// `prop_fields()`; if a path appears here but `get_prop` rejects
    /// it, that field is unreachable on every surface.
    ///
    /// `init_defaults(None)` populates Option-shaped subsections (memory
    /// backend specifics, tunnel provider details, etc.) so the walk
    /// also exercises fields that only materialize once a backend is
    /// chosen.
    #[test]
    async fn every_prop_field_path_is_reachable_via_get_prop() {
        let mut config = Config::default();
        config.init_defaults(None);
        for field in config.prop_fields() {
            let result = config.get_prop(&field.name);
            assert!(
                result.is_ok(),
                "get_prop('{}') failed: {} \u{2014} prop_fields() advertises a path \
                 that the CLI / gateway / TUI all expect to be readable. \
                 Either the macro emits the path but routing is missing, \
                 or the field shouldn't be in prop_fields().",
                field.name,
                result.unwrap_err()
            );
        }
    }

    /// The dashboard's `/config/channels` global-settings tab filters the
    /// canonical `prop_fields()` list down to direct `[channels]` fields.
    /// Keep root settings such as `show_tool_calls` discoverable there without
    /// mixing per-alias fields into the same editor.
    #[test]
    async fn channels_root_settings_stay_on_direct_prop_surface() {
        let mut config = Config::default();
        config.init_defaults(None);
        config
            .channels
            .matrix
            .insert("default".into(), MatrixConfig::default());

        let paths: Vec<_> = config
            .prop_fields()
            .into_iter()
            .map(|field| field.name)
            .collect();
        let direct_channel_paths: Vec<_> = paths
            .iter()
            .filter_map(|path| {
                path.strip_prefix("channels.")
                    .filter(|rest| !rest.contains('.'))
                    .map(|_| path.as_str())
            })
            .collect();

        assert!(
            direct_channel_paths.contains(&"channels.show_tool_calls"),
            "root [channels] settings should include show_tool_calls: {direct_channel_paths:?}"
        );
        assert!(
            direct_channel_paths.contains(&"channels.ack_reactions"),
            "root [channels] settings should include other global channel controls"
        );
        assert!(
            paths
                .iter()
                .any(|path| path == "channels.matrix.default.enabled"),
            "fixture should include a nested channel alias field"
        );
        assert!(
            !direct_channel_paths.contains(&"channels.matrix.default.enabled"),
            "global channel settings must not include per-alias fields"
        );
    }

    /// Audit gate for RFC Phase 0: any credential-shaped property path
    /// that reaches the CLI/gateway/TUI property surface must have an explicit
    /// classification. This catches future config additions whose names imply
    /// credential handling before they silently land without a security call.
    #[test]
    async fn credential_shaped_prop_fields_have_explicit_classification() {
        let mut config = Config::default();
        config.init_defaults(None);
        config
            .providers
            .models
            .anthropic
            .insert("default".into(), AnthropicModelProviderConfig::default());
        config
            .providers
            .tts
            .openai
            .insert("default".into(), OpenAITtsProviderConfig::default());
        config.providers.transcription.openai.insert(
            "default".into(),
            OpenAiTranscriptionProviderConfig::default(),
        );
        config.providers.transcription.local_whisper.insert(
            "default".into(),
            LocalWhisperTranscriptionProviderConfig::default(),
        );
        config
            .channels
            .matrix
            .insert("default".into(), MatrixConfig::default());
        config
            .storage
            .qdrant
            .insert("default".into(), QdrantStorageConfig::default());

        let fields = config.prop_fields();
        let missing: Vec<_> = fields
            .iter()
            .filter(|field| credential_shaped_prop_path(&field.name))
            .filter(|field| field.credential_class.is_none())
            .map(|field| field.name.clone())
            .collect();

        assert!(
            missing.is_empty(),
            "credential-shaped config fields need explicit classification: {missing:?}"
        );

        let unmarked_secrets: Vec<_> = fields
            .iter()
            .filter(|field| {
                field.credential_class
                    == Some(crate::config::CredentialSurfaceClass::EncryptedSecret)
            })
            .filter(|field| !field.is_secret && !Config::prop_is_secret(&field.name))
            .map(|field| field.name.clone())
            .collect();

        assert!(
            unmarked_secrets.is_empty(),
            "EncryptedSecret classifications must route through #[secret]: {unmarked_secrets:?}"
        );
    }

    #[test]
    async fn prop_fields_carry_credential_classification_from_schema_fields() {
        let mut config = Config::default();
        config.init_defaults(None);
        config.providers.models.openai.insert(
            "codex".into(),
            OpenAIModelProviderConfig {
                base: ModelProviderConfig {
                    requires_openai_auth: true,
                    ..ModelProviderConfig::default()
                },
            },
        );
        config
            .providers
            .tts
            .openai
            .insert("default".into(), OpenAITtsProviderConfig::default());
        config.providers.transcription.local_whisper.insert(
            "default".into(),
            LocalWhisperTranscriptionProviderConfig::default(),
        );
        config
            .channels
            .matrix
            .insert("default".into(), MatrixConfig::default());

        let fields = config.prop_fields();
        let class_for = |name: &str| {
            fields
                .iter()
                .find(|field| field.name == name)
                .and_then(|field| field.credential_class)
        };

        assert_eq!(
            class_for("providers.models.openai.codex.requires_openai_auth"),
            Some(crate::config::CredentialSurfaceClass::ExternalAuthStore)
        );
        assert_eq!(
            class_for("providers.tts.openai.default.api_key"),
            Some(crate::config::CredentialSurfaceClass::EncryptedSecret)
        );
        assert_eq!(
            class_for("providers.transcription.local_whisper.default.bearer_token"),
            Some(crate::config::CredentialSurfaceClass::EncryptedSecret)
        );
        assert_eq!(
            class_for("channels.matrix.default.access_token"),
            Some(crate::config::CredentialSurfaceClass::EncryptedSecret)
        );
        // model_routes and embedding_routes are now #[nested] Vec fields —
        // they are surfaced via map_key_sections(), not as flat prop_fields.
        // After adding a route entry, its api_key sub-field appears in
        // prop_fields with EncryptedSecret classification (from #[secret]).
        config.model_routes.push(ModelRouteConfig {
            hint: "reasoning".into(),
            model_provider: "openai.default".into(),
            model: "gpt-4".into(),
            api_key: None,
        });
        config.embedding_routes.push(EmbeddingRouteConfig {
            hint: "semantic".into(),
            model_provider: "openai.embeddings".into(),
            model: "text-embedding-3-small".into(),
            dimensions: None,
            api_key: None,
        });
        let nested_fields = config.prop_fields();
        let nested_class_for = |name: &str| {
            nested_fields
                .iter()
                .find(|field| field.name == name)
                .and_then(|field| field.credential_class)
        };
        assert_eq!(
            nested_class_for("model_routes.reasoning.api_key"),
            Some(crate::config::CredentialSurfaceClass::EncryptedSecret)
        );
        assert_eq!(
            nested_class_for("embedding_routes.semantic.api_key"),
            Some(crate::config::CredentialSurfaceClass::EncryptedSecret)
        );
        assert!(Config::prop_is_secret(
            "providers.tts.openai.default.api_key"
        ));
        assert!(Config::prop_is_secret(
            "providers.transcription.local_whisper.default.bearer_token"
        ));
        assert!(Config::prop_is_secret(
            "channels.matrix.default.access_token"
        ));
    }

    fn credential_shaped_prop_path(path: &str) -> bool {
        path.split('.').any(|part| {
            let normalized = part.replace('_', "-");
            let has_term = |needle| normalized.split('-').any(|term| term == needle);
            normalized.contains("api-key")
                || normalized.contains("api-token")
                || normalized.contains("auth-file")
                || normalized.contains("auth-header")
                || normalized.contains("auth-token")
                || normalized.contains("bearer-token")
                || normalized.contains("bot-token")
                || normalized.contains("access-token")
                || normalized.contains("refresh-token")
                || normalized.contains("verification-token")
                || normalized.contains("paired-tokens")
                || part == "token"
                || has_term("credential")
                || has_term("env")
                || has_term("header")
                || has_term("headers")
                || has_term("password")
                || has_term("secret")
        })
    }

    #[test]
    async fn object_array_prop_display_redacts_nested_secret_fields() {
        let fixture = ObjectArraySecretFixture {
            entries: vec![
                ObjectArraySecretEntry {
                    name: "primary".to_string(),
                    token: Some("nested-token-credential".to_string()),
                    headers: HashMap::from([
                        (
                            "Authorization".to_string(),
                            "Bearer nested-header-credential".to_string(),
                        ),
                        ("X-Tenant".to_string(), "tenant-credential".to_string()),
                    ]),
                },
                ObjectArraySecretEntry {
                    name: "unset-secret".to_string(),
                    token: None,
                    headers: HashMap::new(),
                },
            ],
        };

        let display_value = fixture
            .prop_fields()
            .into_iter()
            .find(|field| field.name == "test.object_array.entries")
            .expect("object-array field should be surfaced")
            .display_value;
        let readback = fixture
            .get_prop("test.object_array.entries")
            .expect("object-array field should be readable");

        for rendered in [&display_value, &readback] {
            assert!(
                !rendered.contains("nested-token-credential"),
                "object-array display/readback must redact scalar nested secrets: {rendered}"
            );
            assert!(
                !rendered.contains("Bearer nested-header-credential"),
                "object-array display/readback must redact nested secret map values: {rendered}"
            );
            assert!(
                !rendered.contains("tenant-credential"),
                "object-array display/readback must redact every value in nested secret maps: {rendered}"
            );
            assert!(
                rendered.contains("primary"),
                "non-secret object-array fields should remain visible: {rendered}"
            );
            assert!(
                rendered.contains("unset-secret"),
                "non-secret fields on entries with unset secrets should remain visible: {rendered}"
            );
            assert!(
                rendered.contains("****"),
                "redacted object-array output should show masked placeholders: {rendered}"
            );
        }

        assert!(
            display_value.contains(r#""token":null"#),
            "JSON display should preserve unset optional secrets as null, not a populated mask: {display_value}"
        );
    }

    #[test]
    async fn onboard_state_prop_path_uses_top_level_kebab_field_name() {
        let mut config = Config::default();

        config
            .set_prop("onboard_state.completed_sections", "agents")
            .expect("onboard state marker path should be writable");
        assert_eq!(
            config
                .get_prop("onboard_state.completed_sections")
                .expect("onboard state marker path should be readable"),
            "[\"agents\"]"
        );
    }

    /// `onboard_state.quickstart_completed` is the flag the Quickstart
    /// flips when it lands a `BuilderSubmission`. Defaults to `false`
    /// so first launches auto-open the Quickstart; round-trips through
    /// `set_prop` / `get_prop` like any other top-level config field.
    #[test]
    async fn onboard_state_quickstart_completed_round_trips() {
        let mut config = Config::default();

        assert_eq!(
            config
                .get_prop("onboard_state.quickstart_completed")
                .expect("default quickstart-completed should be readable"),
            "false",
            "fresh configs default to quickstart-completed=false so the \
             Quickstart auto-opens on first launch",
        );

        config
            .set_prop("onboard_state.quickstart_completed", "true")
            .expect("quickstart-completed should be writable via prop path");
        assert_eq!(
            config
                .get_prop("onboard_state.quickstart_completed")
                .expect("quickstart-completed should be readable after set"),
            "true"
        );
    }

    #[test]
    async fn per_agent_nested_prop_fields_use_agent_alias_paths() {
        let mut config = Config::default();
        config
            .agents
            .insert("bob".to_string(), AliasedAgentConfig::default());
        config.runtime_profiles.insert(
            "fast".to_string(),
            crate::schema::RuntimeProfileConfig::default(),
        );

        let fields = config.prop_fields();
        assert!(
            fields
                .iter()
                .any(|field| field.name == "runtime_profiles.fast.history_pruning.enabled"),
            "history-pruning is a runtime-profile field, emitted under the profile alias"
        );
        assert!(
            !fields
                .iter()
                .any(|field| field.name.starts_with("agents.bob.history_pruning")),
            "history-pruning must no longer be settable on the agent"
        );

        config
            .set_prop("runtime_profiles.fast.history_pruning.enabled", "true")
            .expect("set_prop should accept the runtime-profile nested path");
        assert_eq!(
            config
                .get_prop("runtime_profiles.fast.history_pruning.enabled")
                .expect("get_prop should accept the runtime-profile nested path"),
            "true"
        );
    }

    /// Audit gate: every non-secret scalar prop round-trips through
    /// `set_prop(get_prop(p))`. The CLI's `omnesagent config set` and the
    /// dashboard's PATCH op both rely on this being true so an operator
    /// can read a value, edit it locally, and write it back. Vec /
    /// object-array fields are skipped — they pass through serde-JSON
    /// rather than scalar string parsing.
    #[test]
    async fn every_scalar_prop_round_trips_through_set_prop() {
        let mut config = Config::default();
        config.init_defaults(None);
        let fields = config.prop_fields();
        for field in &fields {
            if field.is_secret
                || matches!(
                    field.kind,
                    crate::config::PropKind::StringArray | crate::config::PropKind::ObjectArray
                )
            {
                continue;
            }
            let value = match config.get_prop(&field.name) {
                Ok(v) => v,
                Err(_) => continue,
            };
            // Sentinel for unset Option fields — no round-trip applies.
            if value == crate::traits::UNSET_DISPLAY {
                continue;
            }
            let result = config.set_prop(&field.name, &value);
            assert!(
                result.is_ok(),
                "round-trip set_prop('{}', '{}') failed: {}",
                field.name,
                value,
                result.unwrap_err()
            );
        }
    }

    /// Every enum field must have a working enum_variants callback, and
    /// set_prop must accept each variant it advertises.
    #[test]
    async fn every_enum_variant_is_settable() {
        let mut config = Config::default();
        config.init_defaults(None);

        for field in config.prop_fields() {
            if !field.is_enum() {
                continue;
            }
            let get_variants = field.enum_variants.unwrap_or_else(|| {
                panic!("enum field '{}' has no enum_variants callback", field.name)
            });
            let variants = get_variants();
            assert!(
                !variants.is_empty(),
                "enum field '{}' returned no variants",
                field.name
            );

            for variant in &variants {
                let result = config.set_prop(&field.name, variant);
                assert!(
                    result.is_ok(),
                    "set_prop('{}', '{}') failed: {}",
                    field.name,
                    variant,
                    result.unwrap_err()
                );
            }
        }
    }

    #[test]
    async fn channel_approval_timeout_secs_defaults_to_300() {
        let discord: DiscordConfig = serde_json::from_str(r#"{"bot_token":"tok"}"#).unwrap();
        assert_eq!(discord.approval_timeout_secs, 300);

        let slack: SlackConfig = serde_json::from_str(r#"{"bot_token":"tok"}"#).unwrap();
        assert_eq!(slack.approval_timeout_secs, 300);

        let signal: SignalConfig =
            serde_json::from_str(r#"{"http_url":"http://localhost","account":"+1"}"#).unwrap();
        assert_eq!(signal.approval_timeout_secs, 300);

        let matrix: MatrixConfig = serde_json::from_str(
            r#"{"homeserver":"https://matrix.org","access_token":"tok","allowed_users":[]}"#,
        )
        .unwrap();
        assert_eq!(matrix.approval_timeout_secs, 300);

        let whatsapp: WhatsAppConfig = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(whatsapp.approval_timeout_secs, 300);

        let lark: LarkConfig =
            serde_json::from_str(r#"{"app_id":"cli_1","app_secret":"secret"}"#).unwrap();
        assert_eq!(lark.approval_timeout_secs, 300);
    }

    /// The test above proves the SERDE path. It says nothing about the RUST
    /// path, and the two used to disagree: a derived `Default` zeroed the
    /// field while serde supplied 300.
    ///
    /// That gap is not cosmetic. An alias created through a surface that
    /// constructs the struct in Rust, rather than parsing a file, got `0`,
    /// and `Duration::from_secs(0)` is an already-elapsed deadline, so every
    /// approval on that alias denied at once with nothing in the operator's
    /// config looking wrong.
    #[test]
    async fn whatsapp_rust_default_waits_rather_than_denying() {
        assert_eq!(
            WhatsAppConfig::default().approval_timeout_secs,
            default_channel_approval_timeout_secs(),
            "a constructed alias must wait the documented timeout, not deny at once"
        );
    }

    /// Pins the whole struct rather than the one field, so a field added later
    /// with a serde default but no matching Rust default fails here instead of
    /// reaching an operator. This is also what makes the `expect` in
    /// `Default::default` safe: if any field stopped being defaultable, the
    /// deserialize would fail and this test would panic first.
    #[test]
    async fn whatsapp_rust_default_matches_serde_default() {
        let from_rust = serde_json::to_value(WhatsAppConfig::default()).unwrap();
        let from_serde =
            serde_json::to_value(serde_json::from_str::<WhatsAppConfig>("{}").unwrap()).unwrap();
        assert_eq!(
            from_rust, from_serde,
            "Default::default() and the serde defaults disagree; every field must match"
        );
    }

    /// A zero is still honoured when an operator writes it. The fix above
    /// changes what an UNSET field means, and must not take away the ability
    /// to refuse every gated tool deliberately.
    #[test]
    async fn whatsapp_explicit_zero_timeout_is_preserved() {
        let whatsapp: WhatsAppConfig =
            serde_json::from_str(r#"{"approval_timeout_secs":0}"#).unwrap();
        assert_eq!(whatsapp.approval_timeout_secs, 0);
    }

    /// The whole lifecycle that persisted the accidental zero: create an alias
    /// through the supported map-key surface, save, reload.
    ///
    /// The two tests above compare `Default::default()` against the serde
    /// defaults, which pins the TYPE. That is where the fix lives, and it is
    /// not where the defect was felt. An operator never calls
    /// `WhatsAppConfig::default()`; they add an alias, and the generated
    /// `create_map_key` inserts `<T>::default()` on their behalf. The field
    /// carries no `skip_serializing_if` and the struct sets no global skip, so
    /// whatever that default held is written out EXPLICITLY, and a serde
    /// default only fires on an ABSENT key. The zero therefore reached the file
    /// and survived every reload after it, with nothing in the operator's
    /// config looking wrong.
    ///
    /// So this drives the surface rather than the type, and asserts on the
    /// bytes as well as the reparse: the on-disk assert names the mechanism
    /// (an explicit `= 0` was persisted) while the reparse asserts what the
    /// operator gets back. Nothing here sets `approval_timeout_secs`; it is
    /// omitted throughout, which is the case that broke.
    #[test]
    async fn whatsapp_alias_created_through_the_map_key_surface_reloads_waiting() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");
        std::fs::write(
            &config_path,
            format!(
                "schema_version = {}\n",
                crate::migration::CURRENT_SCHEMA_VERSION
            ),
        )
        .unwrap();

        let mut config = Config {
            config_path: config_path.clone(),
            ..Default::default()
        };

        // The same wire sequence the create dispatch runs: create_map_key,
        // then mark the new alias dirty.
        let created = config
            .create_map_key("channels.whatsapp", "shop")
            .expect("channels.whatsapp must be a map-keyed section");
        assert!(
            created,
            "a fresh alias must be created, not silently reused"
        );
        config.mark_dirty("channels.whatsapp.shop");

        config.save_dirty().await.unwrap();

        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(
            !written.contains("approval_timeout_secs = 0"),
            "creating an alias must not persist an already-elapsed approval deadline; \
             got:\n{written}"
        );

        let reparsed: Config = toml::from_str(&written).unwrap();
        let shop = reparsed
            .channels
            .whatsapp
            .get("shop")
            .expect("the created alias must survive save and reload");
        assert_eq!(
            shop.approval_timeout_secs,
            default_channel_approval_timeout_secs(),
            "an alias whose approval_timeout_secs was never set must reload waiting the \
             documented timeout, not denying at once"
        );
    }

    /// Sibling to `whatsapp_rust_default_waits_rather_than_denying`, for the
    /// other five channel configs that carried the same split: a derived
    /// `Default` zeroed `approval_timeout_secs` while serde supplied 300, so
    /// a config built in Rust rather than parsed from a file denied every
    /// approval instantly.
    #[test]
    async fn channel_rust_default_waits_rather_than_denying() {
        assert_eq!(
            DiscordConfig::default().approval_timeout_secs,
            default_channel_approval_timeout_secs()
        );
        assert_eq!(
            SlackConfig::default().approval_timeout_secs,
            default_channel_approval_timeout_secs()
        );
        assert_eq!(
            MatrixConfig::default().approval_timeout_secs,
            default_channel_approval_timeout_secs()
        );
        assert_eq!(
            SignalConfig::default().approval_timeout_secs,
            default_channel_approval_timeout_secs()
        );
        assert_eq!(
            LarkConfig::default().approval_timeout_secs,
            default_channel_approval_timeout_secs()
        );
    }

    /// Sibling to `whatsapp_rust_default_matches_serde_default`, pinning the
    /// whole struct rather than the one field for the other five channel
    /// configs, so a field added later with a serde default but no matching
    /// Rust default fails here instead of reaching an operator.
    #[test]
    async fn channel_rust_default_matches_serde_default() {
        assert_eq!(
            serde_json::to_value(DiscordConfig::default()).unwrap(),
            serde_json::to_value(serde_json::from_str::<DiscordConfig>("{}").unwrap()).unwrap()
        );
        assert_eq!(
            serde_json::to_value(SlackConfig::default()).unwrap(),
            serde_json::to_value(serde_json::from_str::<SlackConfig>("{}").unwrap()).unwrap()
        );
        assert_eq!(
            serde_json::to_value(MatrixConfig::default()).unwrap(),
            serde_json::to_value(
                serde_json::from_str::<MatrixConfig>(r#"{"homeserver":""}"#).unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            serde_json::to_value(SignalConfig::default()).unwrap(),
            serde_json::to_value(
                serde_json::from_str::<SignalConfig>(r#"{"http_url":"","account":""}"#).unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            serde_json::to_value(LarkConfig::default()).unwrap(),
            serde_json::to_value(
                serde_json::from_str::<LarkConfig>(r#"{"app_id":"","app_secret":""}"#).unwrap()
            )
            .unwrap()
        );
    }

    /// Sibling to `whatsapp_explicit_zero_timeout_is_preserved`: an operator
    /// who deliberately writes `approval_timeout_secs = 0` keeps that zero.
    /// The fix above changes what an UNSET field means, and must not take
    /// away the ability to refuse every gated tool deliberately.
    #[test]
    async fn channel_explicit_zero_timeout_is_preserved() {
        let discord: DiscordConfig =
            serde_json::from_str(r#"{"approval_timeout_secs":0}"#).unwrap();
        assert_eq!(discord.approval_timeout_secs, 0);

        let slack: SlackConfig = serde_json::from_str(r#"{"approval_timeout_secs":0}"#).unwrap();
        assert_eq!(slack.approval_timeout_secs, 0);

        let matrix: MatrixConfig = serde_json::from_str(
            r#"{"homeserver":"https://matrix.example.com","approval_timeout_secs":0}"#,
        )
        .unwrap();
        assert_eq!(matrix.approval_timeout_secs, 0);

        let signal: SignalConfig = serde_json::from_str(
            r#"{"http_url":"http://localhost","account":"+1","approval_timeout_secs":0}"#,
        )
        .unwrap();
        assert_eq!(signal.approval_timeout_secs, 0);

        let lark: LarkConfig = serde_json::from_str(
            r#"{"app_id":"cli_1","app_secret":"secret","approval_timeout_secs":0}"#,
        )
        .unwrap();
        assert_eq!(lark.approval_timeout_secs, 0);
    }

    /// Generates the sibling of
    /// `whatsapp_alias_created_through_the_map_key_surface_reloads_waiting`
    /// for the other five channel configs, rather than hand-duplicating the
    /// lifecycle five times: create an alias through the supported map-key
    /// surface, save, reload, with `approval_timeout_secs` omitted
    /// throughout - the case that broke. `$setup` fills in whatever fields
    /// a given channel type requires beyond `enabled` so the persisted
    /// alias round-trips; it never touches `approval_timeout_secs`.
    macro_rules! channel_alias_reloads_waiting_test {
        ($test_name:ident, $section:literal, $field:ident, |$cfg:ident| $setup:block) => {
            #[test]
            async fn $test_name() {
                let tmp = tempfile::TempDir::new().unwrap();
                let config_path = tmp.path().join("config.toml");
                std::fs::write(
                    &config_path,
                    format!(
                        "schema_version = {}\n",
                        crate::migration::CURRENT_SCHEMA_VERSION
                    ),
                )
                .unwrap();

                let mut config = Config {
                    config_path: config_path.clone(),
                    ..Default::default()
                };

                let created = config
                    .create_map_key($section, "shop")
                    .expect(concat!($section, " must be a map-keyed section"));
                assert!(
                    created,
                    "a fresh alias must be created, not silently reused"
                );

                {
                    let $cfg = config.channels.$field.get_mut("shop").unwrap();
                    $setup
                }

                config.mark_dirty(&format!("{}.shop", $section));

                config.save_dirty().await.unwrap();

                let written = std::fs::read_to_string(&config_path).unwrap();
                assert!(
                    !written.contains("approval_timeout_secs = 0"),
                    "creating an alias must not persist an already-elapsed approval deadline; \
                     got:\n{written}"
                );

                let reparsed: Config = toml::from_str(&written).unwrap();
                let shop = reparsed
                    .channels
                    .$field
                    .get("shop")
                    .expect("the created alias must survive save and reload");
                assert_eq!(
                    shop.approval_timeout_secs,
                    default_channel_approval_timeout_secs(),
                    "an alias whose approval_timeout_secs was never set must reload waiting \
                     the documented timeout, not denying at once"
                );
            }
        };
    }

    channel_alias_reloads_waiting_test!(
        discord_alias_created_through_the_map_key_surface_reloads_waiting,
        "channels.discord",
        discord,
        |_cfg| {}
    );
    channel_alias_reloads_waiting_test!(
        slack_alias_created_through_the_map_key_surface_reloads_waiting,
        "channels.slack",
        slack,
        |_cfg| {}
    );
    // Matrix requires `homeserver`; set it before saving so the alias
    // round-trips, same as an operator filling in a required field through
    // the dashboard would. `approval_timeout_secs` stays untouched.
    channel_alias_reloads_waiting_test!(
        matrix_alias_created_through_the_map_key_surface_reloads_waiting,
        "channels.matrix",
        matrix,
        |cfg| {
            cfg.homeserver = "https://matrix.example.com".to_string();
        }
    );
    // Signal requires `http_url` and `account`; same reasoning as Matrix.
    channel_alias_reloads_waiting_test!(
        signal_alias_created_through_the_map_key_surface_reloads_waiting,
        "channels.signal",
        signal,
        |cfg| {
            cfg.http_url = "http://127.0.0.1:8686".to_string();
            cfg.account = "+15555550123".to_string();
        }
    );
    // Lark requires `app_id` and `app_secret`; same reasoning as Matrix.
    channel_alias_reloads_waiting_test!(
        lark_alias_created_through_the_map_key_surface_reloads_waiting,
        "channels.lark",
        lark,
        |cfg| {
            cfg.app_id = "cli_test".to_string();
            cfg.app_secret = "secret_test".to_string();
        }
    );

    #[test]
    async fn channel_approval_timeout_secs_explicit_override() {
        let discord: DiscordConfig =
            serde_json::from_str(r#"{"bot_token":"tok","approval_timeout_secs":60}"#).unwrap();
        assert_eq!(discord.approval_timeout_secs, 60);

        let slack: SlackConfig =
            serde_json::from_str(r#"{"bot_token":"tok","approval_timeout_secs":120}"#).unwrap();
        assert_eq!(slack.approval_timeout_secs, 120);

        let signal: SignalConfig = serde_json::from_str(
            r#"{"http_url":"http://localhost","account":"+1","approval_timeout_secs":90}"#,
        )
        .unwrap();
        assert_eq!(signal.approval_timeout_secs, 90);

        let matrix: MatrixConfig = serde_json::from_str(
            r#"{"homeserver":"https://matrix.org","access_token":"tok","allowed_users":[],"approval_timeout_secs":45}"#,
        )
        .unwrap();
        assert_eq!(matrix.approval_timeout_secs, 45);

        let whatsapp: WhatsAppConfig =
            serde_json::from_str(r#"{"approval_timeout_secs":180}"#).unwrap();
        assert_eq!(whatsapp.approval_timeout_secs, 180);

        let lark: LarkConfig = serde_json::from_str(
            r#"{"app_id":"cli_1","app_secret":"secret","approval_timeout_secs":75}"#,
        )
        .unwrap();
        assert_eq!(lark.approval_timeout_secs, 75);
    }

    // ── Multi-agent cross-reference validators ─────────────────────

    /// Build a minimal valid Config with one agent on a configured
    /// channel + risk profile + model provider. Each test mutates a
    /// single field to provoke a validator.
    fn multi_agent_test_config() -> Config {
        use crate::providers::ChannelRef;

        let mut config = Config::default();

        // Risk profile (mandatory for enabled agents).
        config
            .risk_profiles
            .insert("default".to_string(), RiskProfileConfig::default());

        // Anthropic model provider (mandatory for the agent).
        config.providers.models.anthropic.insert(
            "default".to_string(),
            AnthropicModelProviderConfig::default(),
        );

        // A configured Telegram channel the agent can reference. Just
        // having the entry in the map is enough for the dotted-alias
        // validator; we are not exercising channel-level behavior here.
        config
            .channels
            .telegram
            .insert("draft".to_string(), TelegramConfig::default());

        // Agent that targets the model provider, risk profile, and
        // channel. Default workspace is jailed.
        let agent = AliasedAgentConfig {
            channels: vec![ChannelRef::new("telegram.draft")],
            model_provider: crate::providers::ModelProviderRef::new("anthropic.default"),
            risk_profile: "default".into(),
            ..AliasedAgentConfig::default()
        };
        config.agents.insert("alpha".to_string(), agent);

        config
    }

    #[test]
    async fn plugin_channel_instance_uses_ordinary_agent_channel_reference() {
        let mut config = multi_agent_test_config();
        config.channels.plugin.insert(
            "operations".to_string(),
            PluginChannelConfig {
                package: "acme.chat".to_string(),
                enabled: true,
            },
        );
        config.agents.get_mut("alpha").unwrap().channels =
            vec![crate::providers::ChannelRef::new("plugin.operations")];

        config
            .validate()
            .expect("plugin channel aliases use the shared dotted reference validator");
        assert_eq!(config.agent_for_channel("plugin.operations"), Some("alpha"));
    }

    #[test]
    async fn plugin_channel_instances_allow_two_aliases_for_one_package() {
        let mut config = multi_agent_test_config();
        for alias in ["primary", "backup"] {
            config.channels.plugin.insert(
                alias.to_string(),
                PluginChannelConfig {
                    package: "acme.chat".to_string(),
                    enabled: true,
                },
            );
        }

        config
            .validate()
            .expect("one package may back isolated logical instances");
        assert!(config.channels.has_any_enabled());
    }

    #[test]
    async fn plugin_channel_instance_rejects_invalid_alias_or_package() {
        let mut config = multi_agent_test_config();
        config.channels.plugin.insert(
            "bad-alias".to_string(),
            PluginChannelConfig {
                package: "acme.chat".to_string(),
                enabled: true,
            },
        );
        let error = config
            .validate()
            .expect_err("plugin aliases use canonical config-key grammar");
        assert!(
            error.to_string().contains("channels.plugin.bad-alias"),
            "{error}"
        );

        config.channels.plugin.clear();
        config.channels.plugin.insert(
            "primary".to_string(),
            PluginChannelConfig {
                package: "Acme Chat".to_string(),
                enabled: true,
            },
        );
        let error = config
            .validate()
            .expect_err("plugin packages use canonical manifest grammar");
        assert!(
            error
                .to_string()
                .contains("channels.plugin.primary.package"),
            "{error}"
        );
    }

    #[test]
    async fn plugin_channel_instance_rejects_an_empty_package() {
        let mut config = multi_agent_test_config();
        config
            .channels
            .plugin
            .insert("primary".to_string(), PluginChannelConfig::default());

        let error = config
            .validate()
            .expect_err("a declaration with no package can never resolve to an installed plugin");
        assert!(
            error
                .to_string()
                .contains("channels.plugin.primary.package"),
            "{error}"
        );
    }

    #[test]
    async fn disabled_plugin_channel_instance_does_not_start_the_supervisor() {
        let mut channels = ChannelsConfig {
            cli: false,
            ..ChannelsConfig::default()
        };
        channels.plugin.insert(
            "paused".to_string(),
            PluginChannelConfig {
                package: "acme.chat".to_string(),
                enabled: false,
            },
        );

        assert!(!channels.has_any_enabled());
    }

    #[test]
    async fn plugin_channel_is_a_known_configured_and_deliverable_channel() {
        let mut channels = ChannelsConfig::default();
        assert!(
            !channels
                .channel_presence()
                .iter()
                .any(|(name, configured, _)| *name == "plugin" && *configured)
        );

        channels.plugin.insert(
            "operations".to_string(),
            PluginChannelConfig {
                package: "acme.chat".to_string(),
                enabled: true,
            },
        );

        let row = channels
            .channel_presence()
            .into_iter()
            .find(|(name, _, _)| *name == "plugin")
            .expect("plugin must appear in the canonical channel registry");
        assert_eq!(row, ("plugin", true, true));
        assert!(
            channels
                .channels()
                .iter()
                .any(|info| info.kind == "plugin" && info.configured)
        );
        assert!(crate::schema::v2::V3_CHANNEL_TYPES.contains(&"plugin"));
    }

    #[test]
    async fn zero_active_plugin_instance_ceiling_is_rejected() {
        let mut config = multi_agent_test_config();
        config.plugins.max_active_instances = 0;

        let error = config
            .validate()
            .expect_err("a zero ceiling admits nothing and is always an operator mistake");
        assert!(
            error.to_string().contains("plugins.max_active_instances"),
            "{error}"
        );
    }

    #[test]
    async fn validate_rejects_workspace_access_self_reference() {
        let mut config = multi_agent_test_config();
        let alpha = config.agents.get_mut("alpha").unwrap();
        alpha.workspace.access.insert(
            crate::multi_agent::AgentAlias::new("alpha"),
            crate::multi_agent::AccessMode::Read,
        );
        let err = config
            .validate()
            .expect_err("self-reference must fail validation");
        let msg = err.to_string();
        assert!(
            msg.contains("agents.alpha.workspace.access.alpha"),
            "expected field path in error, got: {msg}"
        );
        assert!(
            msg.contains("self-references"),
            "expected self-reference explanation, got: {msg}"
        );
    }

    #[test]
    async fn validate_rejects_workspace_access_dangling_target() {
        let mut config = multi_agent_test_config();
        let alpha = config.agents.get_mut("alpha").unwrap();
        alpha.workspace.access.insert(
            crate::multi_agent::AgentAlias::new("ghost"),
            crate::multi_agent::AccessMode::ReadWrite,
        );
        let err = config
            .validate()
            .expect_err("dangling target must fail validation");
        let msg = err.to_string();
        assert!(
            msg.contains("agents.ghost is not configured"),
            "expected dangling-ref explanation, got: {msg}"
        );
    }

    #[test]
    async fn validate_rejects_read_memory_from_self_reference() {
        let mut config = multi_agent_test_config();
        let alpha = config.agents.get_mut("alpha").unwrap();
        alpha
            .workspace
            .read_memory_from
            .push(crate::multi_agent::AgentAlias::new("alpha"));
        let err = config
            .validate()
            .expect_err("self-reference must fail validation");
        assert!(
            err.to_string().contains("read_memory_from[0]"),
            "expected indexed field path, got: {err}"
        );
    }

    #[test]
    async fn validate_rejects_read_memory_from_cross_backend() {
        let mut config = multi_agent_test_config();

        // Add a second agent on Postgres.
        let beta = AliasedAgentConfig {
            channels: vec![crate::providers::ChannelRef::new("telegram.draft")],
            model_provider: crate::providers::ModelProviderRef::new("anthropic.default"),
            risk_profile: "default".into(),
            memory: crate::multi_agent::AgentMemoryConfig {
                backend: crate::multi_agent::MemoryBackendKind::Postgres,
            },
            ..AliasedAgentConfig::default()
        };
        config.agents.insert("beta".to_string(), beta);

        // Alpha (Sqlite default) tries to read from beta (Postgres).
        let alpha = config.agents.get_mut("alpha").unwrap();
        alpha
            .workspace
            .read_memory_from
            .push(crate::multi_agent::AgentAlias::new("beta"));

        let err = config
            .validate()
            .expect_err("cross-backend allowlist must fail validation");
        let msg = err.to_string();
        assert!(
            msg.contains("same-backend siblings only"),
            "expected cross-backend explanation, got: {msg}"
        );
    }

    #[test]
    async fn validate_rejects_typed_memory_flags_on_non_sqlite_global_backend() {
        let mut config = multi_agent_test_config();
        config.memory.types.enabled = true;
        config.memory.backend = "postgres.work".to_string();

        let err = config
            .validate()
            .expect_err("typed memory on a non-sqlite global backend must fail validation");
        let msg = err.to_string();
        assert!(
            msg.contains("memory.types.enabled") && msg.contains("SQLite-only"),
            "expected SQLite-only explanation naming the flag, got: {msg}"
        );
    }

    #[test]
    async fn validate_rejects_typed_memory_flags_on_non_sqlite_agent_backend() {
        let mut config = multi_agent_test_config();
        config.memory.consolidation_extract_facts = true;

        let beta = AliasedAgentConfig {
            channels: vec![crate::providers::ChannelRef::new("telegram.draft")],
            model_provider: crate::providers::ModelProviderRef::new("anthropic.default"),
            risk_profile: "default".into(),
            memory: crate::multi_agent::AgentMemoryConfig {
                backend: crate::multi_agent::MemoryBackendKind::Markdown,
            },
            ..AliasedAgentConfig::default()
        };
        config.agents.insert("beta".to_string(), beta);

        let err = config
            .validate()
            .expect_err("typed memory with a non-sqlite agent backend must fail validation");
        let msg = err.to_string();
        assert!(
            msg.contains("memory.consolidation_extract_facts")
                && msg.contains("agents.beta.memory.backend")
                && msg.contains("Markdown"),
            "expected SQLite-only explanation naming the agent, got: {msg}"
        );
    }

    #[test]
    async fn validate_accepts_typed_memory_flags_on_sqlite() {
        let mut config = multi_agent_test_config();
        config.memory.types.enabled = true;
        config.memory.consolidation_extract_facts = true;
        config.memory.backend = "sqlite".to_string();

        config
            .validate()
            .expect("typed memory on sqlite everywhere must pass validation");
    }

    #[test]
    async fn validate_accepts_typed_memory_flags_on_mixed_case_sqlite() {
        // The runtime classifies backends case-insensitively
        // (`backend_kind_from_dotted` lowercases), so the SQLite-only gate
        // must accept every spelling the runtime resolves to sqlite.
        let mut config = multi_agent_test_config();
        config.memory.types.enabled = true;
        config.memory.backend = " SQLite.default ".to_string();

        config
            .validate()
            .expect("mixed-case sqlite spellings the runtime accepts must pass the typed gate");
    }

    #[test]
    async fn validate_allows_non_sqlite_backend_when_typed_memory_flags_off() {
        let mut config = multi_agent_test_config();
        config.memory.backend = "postgres.work".to_string();
        let alpha = config.agents.get_mut("alpha").unwrap();
        alpha.memory.backend = crate::multi_agent::MemoryBackendKind::Postgres;

        config
            .validate()
            .expect("flags-off configs keep every backend choice valid");
    }

    #[test]
    async fn validate_rejects_peer_group_dangling_member() {
        let mut config = multi_agent_test_config();
        let group = crate::multi_agent::PeerGroupConfig {
            channel: "telegram".into(),
            agents: vec![
                crate::multi_agent::AgentAlias::new("alpha"),
                crate::multi_agent::AgentAlias::new("ghost"),
            ],
            ..crate::multi_agent::PeerGroupConfig::default()
        };
        config.peer_groups.insert("team_chat".to_string(), group);
        let err = config
            .validate()
            .expect_err("dangling group member must fail validation");
        assert!(
            err.to_string().contains("peer_groups.team_chat.agents[1]"),
            "expected indexed field path, got: {err}"
        );
    }

    #[test]
    async fn validate_rejects_peer_group_member_without_channel() {
        let mut config = multi_agent_test_config();

        // Add a discord channel and a beta agent that ONLY uses discord.
        config
            .channels
            .discord
            .insert("ops".to_string(), DiscordConfig::default());
        let beta = AliasedAgentConfig {
            channels: vec![crate::providers::ChannelRef::new("discord.ops")],
            model_provider: crate::providers::ModelProviderRef::new("anthropic.default"),
            risk_profile: "default".into(),
            ..AliasedAgentConfig::default()
        };
        config.agents.insert("beta".to_string(), beta);

        // Group on telegram.draft includes beta (who only has discord).
        let group = crate::multi_agent::PeerGroupConfig {
            channel: "telegram".into(),
            agents: vec![
                crate::multi_agent::AgentAlias::new("alpha"),
                crate::multi_agent::AgentAlias::new("beta"),
            ],
            ..crate::multi_agent::PeerGroupConfig::default()
        };
        config.peer_groups.insert("team_chat".to_string(), group);

        let err = config
            .validate()
            .expect_err("channel-mismatch group member must fail validation");
        let msg = err.to_string();
        assert!(
            msg.contains("agents.beta.channels has no entry of type"),
            "expected channel-mismatch explanation, got: {msg}"
        );
    }

    #[test]
    async fn validate_accepts_valid_peer_group_with_two_compatible_members() {
        let mut config = multi_agent_test_config();

        // Beta on the same telegram channel.
        let beta = AliasedAgentConfig {
            channels: vec![crate::providers::ChannelRef::new("telegram.draft")],
            model_provider: crate::providers::ModelProviderRef::new("anthropic.default"),
            risk_profile: "default".into(),
            ..AliasedAgentConfig::default()
        };
        config.agents.insert("beta".to_string(), beta);

        // Group on telegram.draft includes both members.
        let group = crate::multi_agent::PeerGroupConfig {
            channel: "telegram".into(),
            agents: vec![
                crate::multi_agent::AgentAlias::new("alpha"),
                crate::multi_agent::AgentAlias::new("beta"),
            ],
            ..crate::multi_agent::PeerGroupConfig::default()
        };
        config.peer_groups.insert("team_chat".to_string(), group);

        config
            .validate()
            .expect("two-member same-channel peer group must validate cleanly");
    }

    const PEER_GROUP_CHANNEL_DANGLING_WARNING: &str = "peer_group_channel_dangling";

    #[test]
    async fn collect_warnings_flags_peer_group_dangling_alias_with_suggestion() {
        let mut config = multi_agent_test_config();
        // A second configured telegram alias close enough to the typo below
        // to be a plausible "did you mean" suggestion.
        config
            .channels
            .telegram
            .insert("alerts".to_string(), TelegramConfig::default());
        let group = crate::multi_agent::PeerGroupConfig {
            channel: "telegram.alert".into(),
            agents: vec![crate::multi_agent::AgentAlias::new("alpha")],
            ..crate::multi_agent::PeerGroupConfig::default()
        };
        config.peer_groups.insert("team_chat".to_string(), group);

        let warnings = warnings_with_code(&config, PEER_GROUP_CHANNEL_DANGLING_WARNING);
        assert_eq!(
            warnings.len(),
            1,
            "expected exactly one warning: {warnings:?}"
        );
        let warning = &warnings[0];
        assert_eq!(warning.path, "peer_groups.team_chat.channel");
        assert!(
            warning.message.contains("team_chat"),
            "message should name the offending group: {}",
            warning.message
        );
        assert!(
            warning.message.contains("telegram.alert"),
            "message should name the bad ref: {}",
            warning.message
        );
        assert!(
            warning
                .message
                .contains("did you mean \"telegram.alerts\"?"),
            "message should suggest the near-match configured alias: {}",
            warning.message
        );
    }

    #[test]
    async fn collect_warnings_flags_peer_group_dangling_alias_without_suggestion() {
        let mut config = multi_agent_test_config();
        // Only "draft" is configured; "zz" is not a plausible typo of it.
        let group = crate::multi_agent::PeerGroupConfig {
            channel: "telegram.zz".into(),
            agents: vec![crate::multi_agent::AgentAlias::new("alpha")],
            ..crate::multi_agent::PeerGroupConfig::default()
        };
        config.peer_groups.insert("team_chat".to_string(), group);

        let warnings = warnings_with_code(&config, PEER_GROUP_CHANNEL_DANGLING_WARNING);
        assert_eq!(
            warnings.len(),
            1,
            "expected exactly one warning: {warnings:?}"
        );
        assert!(
            !warnings[0].message.contains("did you mean"),
            "no near-match should mean no suggestion: {}",
            warnings[0].message
        );
    }

    #[test]
    async fn collect_warnings_peer_group_suggestion_is_deterministic_on_ties() {
        let mut config = multi_agent_test_config();
        // "alerta" and "alerts" are both distance 1 from the typo "alert";
        // the classifier sorts known_aliases, so the lexicographically first
        // candidate must win regardless of HashMap iteration order.
        config
            .channels
            .telegram
            .insert("alerts".to_string(), TelegramConfig::default());
        config
            .channels
            .telegram
            .insert("alerta".to_string(), TelegramConfig::default());
        let group = crate::multi_agent::PeerGroupConfig {
            channel: "telegram.alert".into(),
            agents: vec![crate::multi_agent::AgentAlias::new("alpha")],
            ..crate::multi_agent::PeerGroupConfig::default()
        };
        config.peer_groups.insert("team_chat".to_string(), group);

        let warnings = warnings_with_code(&config, PEER_GROUP_CHANNEL_DANGLING_WARNING);
        assert_eq!(
            warnings.len(),
            1,
            "expected exactly one warning: {warnings:?}"
        );
        assert!(
            warnings[0]
                .message
                .contains("did you mean \"telegram.alerta\"?"),
            "equidistant candidates must resolve to the lexicographically first alias: {}",
            warnings[0].message
        );
    }

    #[test]
    async fn collect_warnings_peer_group_trailing_dot_ref_warns_without_suggestion() {
        let mut config = multi_agent_test_config();
        // A trailing dot leaves an empty alias; that is dangling (warn) but
        // is not a typo of anything, so no "did you mean" — the distance
        // floor would otherwise propose an unrelated short alias.
        config
            .channels
            .telegram
            .insert("ab".to_string(), TelegramConfig::default());
        let group = crate::multi_agent::PeerGroupConfig {
            channel: "telegram.".into(),
            agents: vec![crate::multi_agent::AgentAlias::new("alpha")],
            ..crate::multi_agent::PeerGroupConfig::default()
        };
        config.peer_groups.insert("team_chat".to_string(), group);

        let warnings = warnings_with_code(&config, PEER_GROUP_CHANNEL_DANGLING_WARNING);
        assert_eq!(
            warnings.len(),
            1,
            "expected exactly one warning: {warnings:?}"
        );
        assert!(
            !warnings[0].message.contains("did you mean"),
            "an empty alias must never produce a suggestion: {}",
            warnings[0].message
        );
    }

    #[test]
    async fn collect_warnings_flags_peer_group_channel_type_unconfigured() {
        let mut config = multi_agent_test_config();
        // "foobar" is not a channel type the schema knows at all, so
        // `get_map_keys("channels.foobar")` is `None` and the classifier
        // reports the type itself as unconfigured. (A recognized type with
        // zero aliases — e.g. bare `discord` here — returns `Some(vec![])`
        // and takes the alias-not-found branch instead; see the next test.)
        let group = crate::multi_agent::PeerGroupConfig {
            channel: "foobar.ops".into(),
            agents: vec![crate::multi_agent::AgentAlias::new("alpha")],
            ..crate::multi_agent::PeerGroupConfig::default()
        };
        config.peer_groups.insert("team_chat".to_string(), group);

        let warnings = warnings_with_code(&config, PEER_GROUP_CHANNEL_DANGLING_WARNING);
        assert_eq!(
            warnings.len(),
            1,
            "expected exactly one warning: {warnings:?}"
        );
        assert!(
            warnings[0].message.contains("foobar.ops"),
            "message should name the bad ref: {}",
            warnings[0].message
        );
        assert!(
            warnings[0]
                .message
                .contains("no [channels.foobar.*] block is configured"),
            "message should use the type-unconfigured reason: {}",
            warnings[0].message
        );
        assert!(
            !warnings[0].message.contains("did you mean"),
            "an unconfigured type has no aliases to suggest: {}",
            warnings[0].message
        );
    }

    #[test]
    async fn collect_warnings_flags_peer_group_known_type_with_no_aliases() {
        let mut config = multi_agent_test_config();
        // "discord" IS a recognized schema channel type, so even with zero
        // configured aliases `get_map_keys("channels.discord")` returns
        // `Some(vec![])` — the classifier reports the type as configured
        // and the warning takes the alias-not-found reason.
        let group = crate::multi_agent::PeerGroupConfig {
            channel: "discord.ops".into(),
            agents: vec![crate::multi_agent::AgentAlias::new("alpha")],
            ..crate::multi_agent::PeerGroupConfig::default()
        };
        config.peer_groups.insert("team_chat".to_string(), group);

        let warnings = warnings_with_code(&config, PEER_GROUP_CHANNEL_DANGLING_WARNING);
        assert_eq!(
            warnings.len(),
            1,
            "expected exactly one warning: {warnings:?}"
        );
        assert!(
            warnings[0]
                .message
                .contains("[channels.discord.ops] is not configured"),
            "message should use the alias-not-found reason: {}",
            warnings[0].message
        );
        assert!(
            !warnings[0].message.contains("did you mean"),
            "zero configured aliases means nothing to suggest: {}",
            warnings[0].message
        );
    }

    #[test]
    async fn collect_warnings_silent_for_valid_peer_group_dotted_channel() {
        let mut config = multi_agent_test_config();
        let group = crate::multi_agent::PeerGroupConfig {
            channel: "telegram.draft".into(),
            agents: vec![crate::multi_agent::AgentAlias::new("alpha")],
            ..crate::multi_agent::PeerGroupConfig::default()
        };
        config.peer_groups.insert("team_chat".to_string(), group);

        assert!(
            warnings_with_code(&config, PEER_GROUP_CHANNEL_DANGLING_WARNING).is_empty(),
            "a valid dotted alias must not warn"
        );
    }

    #[test]
    async fn collect_warnings_silent_for_bare_type_peer_group_channel() {
        let mut config = multi_agent_test_config();
        let group = crate::multi_agent::PeerGroupConfig {
            channel: "telegram".into(),
            agents: vec![crate::multi_agent::AgentAlias::new("alpha")],
            ..crate::multi_agent::PeerGroupConfig::default()
        };
        config.peer_groups.insert("team_chat".to_string(), group);

        assert!(
            warnings_with_code(&config, PEER_GROUP_CHANNEL_DANGLING_WARNING).is_empty(),
            "a bare type-wide ref must never warn, even though it authorizes every alias"
        );
    }

    #[test]
    async fn config_validate_rejects_classifier_provider_pointing_at_missing_alias() {
        // Use the SHARED `typed_provider_refs` validation loop — same error
        // surface as tts_provider / transcription_provider.
        let toml = r#"
            [providers.models.custom.default]
            api_key = "k"
            model = "qwen3.6-plus"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"

            [risk_profiles.default]
            level = "supervised"

            [agents.default]
            enabled = true
            model_provider = "custom.default"
            risk_profile = "default"
            classifier_provider = "custom.does-not-exist"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let err = cfg
            .validate()
            .expect_err("missing alias must fail validate");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("classifier_provider")
                && msg.contains("does-not-exist")
                && msg.contains("providers.models.custom.does-not-exist is not configured"),
            "expected DanglingReference error mentioning field + alias + section, got: {msg}"
        );
    }

    // agent-level summary_provider validated like classifier_provider.
    #[tokio::test]
    async fn config_validate_rejects_agent_summary_provider_missing_alias() {
        let toml = r#"
            [providers.models.custom.default]
            api_key = "k"
            model = "qwen3.6-plus"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"

            [risk_profiles.default]
            level = "supervised"

            [agents.default]
            enabled = true
            model_provider = "custom.default"
            risk_profile = "default"
            summary_provider = "custom.does-not-exist"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let msg = format!("{:#}", cfg.validate().expect_err("missing alias must fail"));
        assert!(
            msg.contains("summary_provider")
                && msg.contains("providers.models.custom.does-not-exist is not configured"),
            "expected DanglingReference for agent summary_provider, got: {msg}"
        );
    }

    // profile-level summary_provider validated by the new profile loop.
    #[tokio::test]
    async fn config_validate_rejects_profile_summary_provider_missing_alias() {
        let toml = r#"
            [providers.models.custom.default]
            api_key = "k"
            model = "qwen3.6-plus"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"

            [risk_profiles.default]
            level = "supervised"

            [runtime_profiles.fast.context_compression]
            summary_provider = "custom.nope"

            [agents.default]
            enabled = true
            model_provider = "custom.default"
            risk_profile = "default"
            runtime_profile = "fast"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let msg = format!(
            "{:#}",
            cfg.validate().expect_err("missing profile alias must fail")
        );
        assert!(
            msg.contains("runtime_profiles.fast.context_compression.summary_provider")
                && msg.contains("providers.models.custom.nope is not configured"),
            "expected DanglingReference for profile summary_provider, got: {msg}"
        );
    }

    // effective_summary_provider precedence — agent → profile → None.
    #[tokio::test]
    async fn effective_summary_provider_precedence() {
        let toml = r#"
            [providers.models.custom.main]
            api_key = "k"
            model = "m-main"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"
            [providers.models.custom.cheap]
            api_key = "k"
            model = "m-cheap"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"
            [providers.models.custom.profilesum]
            api_key = "k"
            model = "m-profile"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"

            [risk_profiles.default]
            level = "supervised"

            [runtime_profiles.fast.context_compression]
            summary_provider = "custom.profilesum"

            [agents.a]
            enabled = true
            model_provider = "custom.main"
            risk_profile = "default"
            runtime_profile = "fast"
            summary_provider = "custom.cheap"

            [agents.b]
            enabled = true
            model_provider = "custom.main"
            risk_profile = "default"
            runtime_profile = "fast"

            [agents.c]
            enabled = true
            model_provider = "custom.main"
            risk_profile = "default"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        // agent override wins over the profile
        assert_eq!(
            cfg.effective_summary_provider("a").as_deref(),
            Some("custom.cheap")
        );
        // agent empty → profile value
        assert_eq!(
            cfg.effective_summary_provider("b").as_deref(),
            Some("custom.profilesum")
        );
        // no agent override + no runtime profile → None (caller uses agent's own)
        assert_eq!(cfg.effective_summary_provider("c"), None);
    }

    // config-time diagnostic for the legacy cross-provider summary_model
    // shape. A profile sets the deprecated bare summary_model and is shared by
    // two agents on DIFFERENT providers with no summary_provider override -> the
    // diagnostic fires and names the profile + the affected agents + providers.
    #[tokio::test]
    async fn collect_warnings_flags_cross_provider_summary_model() {
        let toml = r#"
            [providers.models.custom.p1]
            api_key = "k"
            model = "m1"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"
            [providers.models.custom.p2]
            api_key = "k"
            model = "m2"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"

            [risk_profiles.default]
            level = "supervised"

            [runtime_profiles.shared.context_compression]
            summary_model = "haiku"

            [agents.alpha]
            enabled = true
            model_provider = "custom.p1"
            risk_profile = "default"
            runtime_profile = "shared"

            [agents.beta]
            enabled = true
            model_provider = "custom.p2"
            risk_profile = "default"
            runtime_profile = "shared"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let warnings = cfg.collect_warnings();
        let w = warnings
            .iter()
            .find(|w| w.code == "cross_provider_summary_model")
            .expect("expected cross_provider_summary_model warning");
        assert_eq!(
            w.path,
            "runtime_profiles.shared.context_compression.summary_model"
        );
        assert!(
            w.message.contains("haiku"),
            "message names the model: {}",
            w.message
        );
        assert!(
            w.message.contains("alpha -> custom.p1"),
            "message names alpha + provider: {}",
            w.message
        );
        assert!(
            w.message.contains("beta -> custom.p2"),
            "message names beta + provider: {}",
            w.message
        );
    }

    // The `cross_provider_summary_model` diagnostic must report the setting
    // as unsupported/inert like every other `context_compression` knob, not
    // as something that is actively dispatched onto per-agent providers and
    // fails at runtime — there is no runtime consumer left to dispatch
    // anything. The cross-provider detail (which agents, which providers)
    // must still be present since it is useful context for the fix, but the
    // message must not claim any runtime behavior.
    #[tokio::test]
    async fn collect_warnings_cross_provider_summary_model_reports_inert_not_dispatch() {
        let toml = r#"
            [providers.models.custom.p1]
            api_key = "k"
            model = "m1"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"
            [providers.models.custom.p2]
            api_key = "k"
            model = "m2"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"

            [risk_profiles.default]
            level = "supervised"

            [runtime_profiles.shared.context_compression]
            summary_model = "haiku"

            [agents.alpha]
            enabled = true
            model_provider = "custom.p1"
            risk_profile = "default"
            runtime_profile = "shared"

            [agents.beta]
            enabled = true
            model_provider = "custom.p2"
            risk_profile = "default"
            runtime_profile = "shared"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let warnings = cfg.collect_warnings();
        let w = warnings
            .iter()
            .find(|w| w.code == "cross_provider_summary_model")
            .expect("expected cross_provider_summary_model warning");
        assert!(
            w.message.contains("not currently implemented") && w.message.contains("no effect"),
            "message must truthfully report the setting as unsupported/inert: {}",
            w.message
        );
        assert!(
            !w.message.contains("silently fails"),
            "message must not claim the setting silently fails at runtime: {}",
            w.message
        );
        assert!(
            !w.message.contains("dispatched"),
            "message must not claim the setting is dispatched to a provider at runtime: {}",
            w.message
        );
        // Cross-provider specificity must survive the rewrite — it is still
        // useful detail even though the setting is inert.
        assert!(
            w.message.contains("alpha -> custom.p1") && w.message.contains("beta -> custom.p2"),
            "message must keep naming the affected agents and providers: {}",
            w.message
        );
        // The remediation must NOT send the operator to another inert
        // context_compression field: this PR's per-field pass classifies a
        // non-default `summary_provider` as unsupported/inert too, so
        // "migrate to context_compression.summary_provider" would just produce
        // another no-effect setting and another warning.
        assert!(
            !w.message
                .contains("Migrate to context_compression.summary_provider"),
            "remediation must not recommend migrating to the inert summary_provider: {}",
            w.message
        );
        assert!(
            w.message
                .contains("Remove the unsupported context_compression setting"),
            "remediation should tell the operator to remove the inert setting: {}",
            w.message
        );
    }

    // The runtime context compressor was removed; nothing reads
    // `context_compression` at runtime anymore, so an explicit
    // `enabled = true` on a named runtime profile is inert and must be
    // flagged.
    #[tokio::test]
    async fn collect_warnings_flags_context_compression_enabled_on_runtime_profile() {
        let toml = r#"
            [risk_profiles.default]
            level = "supervised"

            [runtime_profiles.fast.context_compression]
            enabled = true

            [agents.alpha]
            enabled = true
            risk_profile = "default"
            runtime_profile = "fast"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let warnings = cfg.collect_warnings();
        let w = warnings
            .iter()
            .find(|w| w.code == "context_compression_unsupported")
            .expect("expected context_compression_unsupported warning");
        assert_eq!(w.path, "runtime_profiles.fast.context_compression.enabled");
        assert!(
            w.message.contains("not currently implemented"),
            "message explains the flag is inert: {}",
            w.message
        );
    }

    // The legacy pre-V3 `[agent.context_compression]` top-level table is
    // folded into `[runtime_profiles.default]` by the V1/V2→V3 migration
    // (see `schema/v2.rs`), so it must surface the same diagnostic once
    // migrated — this is the historical form of the surface commonly
    // called "agent-level" configuration.
    #[::core::prelude::v1::test]
    fn collect_warnings_flags_context_compression_enabled_via_legacy_agent_table() {
        let raw = r#"
            default_temperature = 0.7

            [agent.context_compression]
            enabled = true
        "#;
        let parsed = crate::migration::migrate_to_current(raw).expect("migration succeeds");
        let warnings = parsed.collect_warnings();
        let w = warnings
            .iter()
            .find(|w| w.code == "context_compression_unsupported")
            .expect("expected context_compression_unsupported warning after migration");
        assert_eq!(
            w.path,
            "runtime_profiles.default.context_compression.enabled"
        );
    }

    // A default config (no explicit `context_compression.enabled`) must stay
    // silent — the flag now defaults to `false`, matching the runtime, which
    // does not consult it at all.
    #[tokio::test]
    async fn collect_warnings_silent_for_context_compression_default() {
        let toml = r#"
            [risk_profiles.default]
            level = "supervised"

            [agents.alpha]
            enabled = true
            risk_profile = "default"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let warnings = cfg.collect_warnings();
        assert!(
            !warnings
                .iter()
                .any(|w| w.code == "context_compression_unsupported"),
            "default config must not flag context_compression_unsupported: {warnings:?}"
        );
    }

    // Every `context_compression` knob is inert, not just `enabled` — tuning
    // fields set to non-default values must each surface their own warning
    // with a per-field path, even with `enabled` left off, since the whole
    // struct is covered.
    #[tokio::test]
    async fn collect_warnings_flags_context_compression_tuning_fields() {
        let toml = r#"
            [risk_profiles.default]
            level = "supervised"

            [runtime_profiles.fast.context_compression]
            threshold_ratio = 0.9
            protect_first_n = 500

            [agents.alpha]
            enabled = true
            risk_profile = "default"
            runtime_profile = "fast"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let warnings = cfg.collect_warnings();
        let paths: Vec<&str> = warnings
            .iter()
            .filter(|w| w.code == "context_compression_unsupported")
            .map(|w| w.path.as_str())
            .collect();
        assert!(
            paths.contains(&"runtime_profiles.fast.context_compression.threshold_ratio"),
            "threshold_ratio must be flagged: {paths:?}"
        );
        assert!(
            paths.contains(&"runtime_profiles.fast.context_compression.protect_first_n"),
            "protect_first_n must be flagged: {paths:?}"
        );
        // `enabled` was not set (defaults to false) — no warning for it.
        assert!(
            !paths.contains(&"runtime_profiles.fast.context_compression.enabled"),
            "unset enabled must not be flagged: {paths:?}"
        );
        let w = warnings
            .iter()
            .find(|w| w.path == "runtime_profiles.fast.context_compression.threshold_ratio")
            .expect("threshold_ratio warning present");
        assert!(
            w.message.contains("non-default value"),
            "message says the value is non-default: {}",
            w.message
        );
    }

    // A knob explicitly written at its default value is indistinguishable
    // from an omitted one post-deserialization and must stay silent — the
    // same accepted limitation as `validate_memory_semantics`.
    #[tokio::test]
    async fn collect_warnings_silent_for_context_compression_default_values_written_explicitly() {
        let toml = r#"
            [risk_profiles.default]
            level = "supervised"

            [runtime_profiles.fast.context_compression]
            enabled = false
            threshold_ratio = 0.50
            protect_first_n = 3
            tool_result_retrim_chars = 2000

            [agents.alpha]
            enabled = true
            risk_profile = "default"
            runtime_profile = "fast"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let warnings = cfg.collect_warnings();
        assert!(
            !warnings
                .iter()
                .any(|w| w.code == "context_compression_unsupported"),
            "explicit default values must not flag context_compression_unsupported: {warnings:?}"
        );
    }

    // Specific-warning-wins dedup: a bare cross-provider `summary_model`
    // already draws the more specific `cross_provider_summary_model`
    // diagnostic, which itself reports the setting as inert (same fact as
    // `context_compression_unsupported`) plus the cross-provider detail, so
    // the generic inert warning must NOT also fire for the identical path —
    // doctor/gateway print both with no dedup, and it would just be the same
    // statement twice.
    #[tokio::test]
    async fn collect_warnings_context_compression_defers_to_cross_provider_summary_model() {
        let toml = r#"
            [providers.models.custom.p1]
            api_key = "k"
            model = "m1"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"
            [providers.models.custom.p2]
            api_key = "k"
            model = "m2"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"

            [risk_profiles.default]
            level = "supervised"

            [runtime_profiles.shared.context_compression]
            summary_model = "haiku"

            [agents.alpha]
            enabled = true
            model_provider = "custom.p1"
            risk_profile = "default"
            runtime_profile = "shared"

            [agents.beta]
            enabled = true
            model_provider = "custom.p2"
            risk_profile = "default"
            runtime_profile = "shared"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let warnings = cfg.collect_warnings();
        let summary_model_warnings: Vec<_> = warnings
            .iter()
            .filter(|w| w.path == "runtime_profiles.shared.context_compression.summary_model")
            .collect();
        assert_eq!(
            summary_model_warnings.len(),
            1,
            "exactly one warning for the summary_model path: {summary_model_warnings:?}"
        );
        assert_eq!(
            summary_model_warnings[0].code, "cross_provider_summary_model",
            "the specific cross-provider diagnostic wins for the shared path"
        );
    }

    // Same-provider control: without a cross-provider diagnostic covering
    // the path, the inert warning must still fire for `summary_model` — no
    // other diagnostic covers the single-provider shape.
    #[tokio::test]
    async fn collect_warnings_context_compression_flags_same_provider_summary_model() {
        let toml = r#"
            [providers.models.custom.p1]
            api_key = "k"
            model = "m1"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"

            [risk_profiles.default]
            level = "supervised"

            [runtime_profiles.shared.context_compression]
            summary_model = "haiku"

            [agents.alpha]
            enabled = true
            model_provider = "custom.p1"
            risk_profile = "default"
            runtime_profile = "shared"

            [agents.beta]
            enabled = true
            model_provider = "custom.p1"
            risk_profile = "default"
            runtime_profile = "shared"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let warnings = cfg.collect_warnings();
        let w = warnings
            .iter()
            .find(|w| w.path == "runtime_profiles.shared.context_compression.summary_model")
            .expect("expected a warning for the summary_model path");
        assert_eq!(
            w.code, "context_compression_unsupported",
            "single-provider summary_model gets the inert warning"
        );
    }

    // exposed_skills set with no skill_bundles -> the agent card resolves no
    // skills (skills: []) silently; the diagnostic fires and names the agent.
    #[tokio::test]
    async fn collect_warnings_flags_exposed_skills_without_bundles() {
        let toml = r#"
            [risk_profiles.default]
            level = "supervised"

            [agents.merchant]
            enabled = true
            risk_profile = "default"

            [agents.merchant.a2a]
            published = true
            exposed_skills = ["ucp_discovery_get", "ucp_merchant_get"]
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let warnings = cfg.collect_warnings();
        let w = warnings
            .iter()
            .find(|w| w.code == "a2a_exposed_skills_without_bundles")
            .expect("expected a2a_exposed_skills_without_bundles warning");
        assert_eq!(w.path, "agents.merchant.a2a.exposed_skills");
        assert!(
            w.message.contains("merchant"),
            "message names the agent: {}",
            w.message
        );
        assert!(
            w.message.contains("skill_bundles"),
            "message points at skill_bundles: {}",
            w.message
        );
    }

    // exposed_skills set alongside at least one declared skill_bundle -> no
    // structural diagnostic (disk resolution governs whether ids actually
    // resolve; that is out of scope for this offline check).
    #[tokio::test]
    async fn collect_warnings_silent_for_exposed_skills_with_bundles() {
        let toml = r#"
            [risk_profiles.default]
            level = "supervised"

            [agents.merchant]
            enabled = true
            risk_profile = "default"
            skill_bundles = ["commerce"]

            [agents.merchant.a2a]
            published = true
            exposed_skills = ["ucp_discovery_get"]
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let warnings = cfg.collect_warnings();
        assert!(
            !warnings
                .iter()
                .any(|w| w.code == "a2a_exposed_skills_without_bundles"),
            "no exposed_skills warning when a bundle is declared: {warnings:?}"
        );
    }

    // Control: same profile + summary_model but both agents on the SAME provider
    // -> no diagnostic (deprecated-but-correct; runtime WARN still nudges).
    #[tokio::test]
    async fn collect_warnings_silent_for_same_provider_summary_model() {
        let toml = r#"
            [providers.models.custom.p1]
            api_key = "k"
            model = "m1"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"

            [risk_profiles.default]
            level = "supervised"

            [runtime_profiles.shared.context_compression]
            summary_model = "haiku"

            [agents.alpha]
            enabled = true
            model_provider = "custom.p1"
            risk_profile = "default"
            runtime_profile = "shared"

            [agents.beta]
            enabled = true
            model_provider = "custom.p1"
            risk_profile = "default"
            runtime_profile = "shared"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert!(
            !cfg.collect_warnings()
                .iter()
                .any(|w| w.code == "cross_provider_summary_model"),
            "same-provider use must not warn"
        );
    }

    // Control: cross-provider agents but each sets an agent-level
    // summary_provider override -> the override supersedes the bare id, so no
    // diagnostic.
    #[tokio::test]
    async fn collect_warnings_silent_when_summary_provider_override_present() {
        let toml = r#"
            [providers.models.custom.p1]
            api_key = "k"
            model = "m1"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"
            [providers.models.custom.p2]
            api_key = "k"
            model = "m2"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"
            [providers.models.custom.sum]
            api_key = "k"
            model = "ms"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"

            [risk_profiles.default]
            level = "supervised"

            [runtime_profiles.shared.context_compression]
            summary_model = "haiku"

            [agents.alpha]
            enabled = true
            model_provider = "custom.p1"
            risk_profile = "default"
            runtime_profile = "shared"
            summary_provider = "custom.sum"

            [agents.beta]
            enabled = true
            model_provider = "custom.p2"
            risk_profile = "default"
            runtime_profile = "shared"
            summary_provider = "custom.sum"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert!(
            !cfg.collect_warnings()
                .iter()
                .any(|w| w.code == "cross_provider_summary_model"),
            "agent-level summary_provider override must suppress the warning"
        );
    }

    const WA_INERT_WARNING: &str = "whatsapp_chat_policy_inert";
    const WA_CLOSED_GROUPS_WARNING: &str = "whatsapp_empty_group_list_serves_no_group";

    /// A config carrying both `phone_number_id` and a Web selector runs as
    /// Cloud, and the Cloud transport consults none of the Web chat-policy
    /// keys. Diagnosing them here would describe a gate that never runs, and
    /// the open-groups warning in particular would report unintended group
    /// access on a channel whose Web group gate is not in the path.
    ///
    /// The second half is the control: strip `phone_number_id` so the same
    /// keys select the Web backend, and both warnings must return. Without it
    /// this test would also pass against a validator that had been switched
    /// off entirely.
    #[test]
    async fn whatsapp_mixed_selectors_run_as_cloud_and_report_no_web_policy_warnings() {
        let mixed = r#"
[channels.whatsapp.shop]
enabled = true
phone_number_id = "1234567890"
access_token = "token"
verify_token = "verify"
session_path = "/tmp/wa-session"
mode = "business"
self_chat_mode = true
group_policy = "allowlist"
allowed_groups = []
"#;
        let cfg: Config = toml::from_str(mixed).unwrap();
        assert_eq!(
            cfg.channels.whatsapp["shop"].backend_type(),
            "cloud",
            "a Cloud selector must win over a Web selector, or this test is not \
             exercising the mixed case"
        );
        assert!(
            warnings_with_code(&cfg, WA_INERT_WARNING).is_empty(),
            "a Cloud-backed channel must not be diagnosed against the Web chat-policy gate"
        );
        assert!(
            warnings_with_code(&cfg, WA_CLOSED_GROUPS_WARNING).is_empty(),
            "the Web group gate is not in a Cloud channel's path, so an empty \
             allowed_groups grants no group access here"
        );

        let web_only = r#"
[channels.whatsapp.shop]
enabled = true
session_path = "/tmp/wa-session"
mode = "business"
self_chat_mode = true
group_policy = "allowlist"
allowed_groups = []
"#;
        let cfg: Config = toml::from_str(web_only).unwrap();
        assert_eq!(cfg.channels.whatsapp["shop"].backend_type(), "web");
        assert!(
            !warnings_with_code(&cfg, WA_INERT_WARNING).is_empty(),
            "removing the Cloud selector must restore the inert-key diagnostic"
        );
        assert!(
            !warnings_with_code(&cfg, WA_CLOSED_GROUPS_WARNING).is_empty(),
            "removing the Cloud selector must restore the empty-group diagnostic"
        );
    }

    /// A Web channel in business mode: `dm_policy` and `group_policy` are
    /// consulted under both modes, so calling them inert would now be false.
    /// `self_chat_mode` is read only inside the personal branch, so it is the
    /// one key that stays inert here.
    #[test]
    async fn whatsapp_business_mode_flags_only_self_chat_mode() {
        let toml = r#"
[channels.whatsapp.shop]
enabled = true
mode = "business"
session_path = "/tmp/wa-session"
self_chat_mode = true
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let warnings = warnings_with_code(&cfg, WA_INERT_WARNING);
        assert!(
            !warnings
                .iter()
                .any(|w| w.path == "channels.whatsapp.shop.dm_policy"),
            "dm_policy is enforced under both modes and must not be reported inert: {warnings:?}"
        );
        assert!(
            !warnings
                .iter()
                .any(|w| w.path == "channels.whatsapp.shop.group_policy"),
            "group_policy is enforced under both modes and must not be reported inert: {warnings:?}"
        );
        assert!(
            warnings
                .iter()
                .any(|w| w.path == "channels.whatsapp.shop.self_chat_mode"),
            "self_chat_mode has no business-mode equivalent and stays inert: {warnings:?}"
        );
        assert!(
            warnings.iter().any(|w| w.message.contains("personal")),
            "the warning should name the mode that would honour it: {warnings:?}"
        );
    }

    /// Personal mode consults the policies, so there is nothing to warn about.
    #[test]
    async fn whatsapp_personal_mode_does_not_flag_chat_policies() {
        let toml = r#"
[channels.whatsapp.shop]
enabled = true
mode = "personal"
session_path = "/tmp/wa-session"
allowed_groups = ["123@g.us"]
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert!(
            warnings_with_code(&cfg, WA_INERT_WARNING).is_empty(),
            "personal mode honours the policies and must not warn"
        );
    }

    /// A Cloud-API channel has no Web transport, so neither warning applies.
    /// Without this guard the check would fire for every Cloud user.
    #[test]
    async fn whatsapp_cloud_channel_is_not_flagged() {
        let toml = r#"
[channels.whatsapp.cloud]
enabled = true
phone_number_id = "1234567890"
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert!(warnings_with_code(&cfg, WA_INERT_WARNING).is_empty());
        assert!(warnings_with_code(&cfg, WA_CLOSED_GROUPS_WARNING).is_empty());
    }

    /// A disabled channel cannot answer anything, so it must stay quiet.
    #[test]
    async fn whatsapp_disabled_channel_is_not_flagged() {
        let toml = r#"
[channels.whatsapp.shop]
enabled = false
mode = "business"
session_path = "/tmp/wa-session"
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert!(warnings_with_code(&cfg, WA_INERT_WARNING).is_empty());
        assert!(warnings_with_code(&cfg, WA_CLOSED_GROUPS_WARNING).is_empty());
    }

    /// The group gate runs in BOTH modes and now returns false when the list is
    /// empty under any policy but `all`, so the default is the newly-CLOSED
    /// case and the operator is told which capability they lost.
    #[test]
    async fn whatsapp_empty_allowed_groups_is_flagged() {
        let toml = r#"
[channels.whatsapp.shop]
enabled = true
mode = "personal"
session_path = "/tmp/wa-session"
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let warnings = warnings_with_code(&cfg, WA_CLOSED_GROUPS_WARNING);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert_eq!(warnings[0].path, "channels.whatsapp.shop.allowed_groups");
    }

    /// An explicit list is the closed case and must not warn.
    #[test]
    async fn whatsapp_populated_allowed_groups_is_not_flagged() {
        let toml = r#"
[channels.whatsapp.shop]
enabled = true
mode = "personal"
session_path = "/tmp/wa-session"
allowed_groups = ["123@g.us"]
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert!(warnings_with_code(&cfg, WA_CLOSED_GROUPS_WARNING).is_empty());
    }

    /// Personal mode with group_policy = "ignore" ALREADY dropped every group
    /// message before this change, so nothing closed and the operator lost no
    /// capability. A migration notice here would report a change that did not
    /// happen to this configuration.
    #[test]
    async fn whatsapp_personal_ignore_groups_is_not_flagged() {
        let toml = r#"
[channels.whatsapp.shop]
enabled = true
mode = "personal"
session_path = "/tmp/wa-session"
group_policy = "ignore"
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert!(
            warnings_with_code(&cfg, WA_CLOSED_GROUPS_WARNING).is_empty(),
            "personal + \"ignore\" already dropped every group message, so \
             nothing closed and the migration notice must not fire"
        );
    }

    /// group_policy = "all" is the explicit opt-in to open group access. An
    /// empty list is then consistent with a deliberate choice, not an accident,
    /// and reporting it as unsafe would flag intent as error.
    #[test]
    async fn whatsapp_personal_all_groups_is_not_flagged() {
        let toml = r#"
[channels.whatsapp.shop]
enabled = true
mode = "personal"
session_path = "/tmp/wa-session"
group_policy = "all"
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert!(
            warnings_with_code(&cfg, WA_CLOSED_GROUPS_WARNING).is_empty(),
            "group_policy = \"all\" is an explicit opt-in to open groups and must \
             not be reported as an unintended configuration"
        );
    }

    /// The default group_policy is `allowlist`, under which an empty list admits
    /// no group, so business mode with no policy set is the newly-closed case.
    /// This is the positive case that must survive narrowing the warning.
    #[test]
    async fn whatsapp_business_empty_allowed_groups_is_flagged() {
        let toml = r#"
[channels.whatsapp.shop]
enabled = true
mode = "business"
session_path = "/tmp/wa-session"
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        let warnings = warnings_with_code(&cfg, WA_CLOSED_GROUPS_WARNING);
        assert_eq!(
            warnings.len(),
            1,
            "business mode with an empty allowed_groups must still warn: {warnings:?}"
        );
        assert_eq!(warnings[0].path, "channels.whatsapp.shop.allowed_groups");
    }

    /// Business + `ignore` must NOT warn. The base already contains the merged
    /// change that made both chat policies apply under both modes, so `ignore`
    /// rejected every group before this PR and rejects every group after it.
    /// Only the gate doing the rejecting moved, from the chat-type gate to the
    /// earlier identity gate. Warning here would tell an operator their channel
    /// used to answer every group and offer to reopen a policy they set
    /// deliberately.
    #[test]
    async fn whatsapp_business_ignore_group_policy_is_not_flagged_as_newly_closed() {
        let toml = r#"
[channels.whatsapp.shop]
enabled = true
mode = "business"
session_path = "/tmp/wa-session"
group_policy = "ignore"
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert!(
            warnings_with_code(&cfg, WA_CLOSED_GROUPS_WARNING).is_empty(),
            "business + \"ignore\" was already closed before this change, so nothing              was lost and nothing may be reported"
        );

        // CONTROL: the same config differing only in group_policy DOES warn, so
        // the silence above is this policy being excluded rather than the check
        // never matching this alias at all.
        let allowlist = toml.replace("group_policy = \"ignore\"", "group_policy = \"allowlist\"");
        let cfg: Config = toml::from_str(&allowlist).unwrap();
        assert_eq!(
            warnings_with_code(&cfg, WA_CLOSED_GROUPS_WARNING).len(),
            1,
            "business + \"allowlist\" with an empty list does newly close"
        );
    }

    /// The six-cell predicate that BOTH the startup notice and `config validate`
    /// consume. Asserted directly here so the shared contract is pinned once,
    /// rather than only observed through whichever surface happens to call it.
    #[test]
    async fn whatsapp_empty_group_list_newly_closed_matrix() {
        use WhatsAppChatPolicy as P;
        use WhatsAppWebMode as M;
        let cases = [
            (M::Business, P::Allowlist, true),
            (M::Personal, P::Allowlist, true),
            // `ignore` and `all` are mode-independent in the chat-type gate, so
            // each pairs with its opposite-mode row. `ignore` rejected every
            // group before and after; `all` admits every group before and
            // after. Neither loses a capability, so neither is reported.
            (M::Business, P::Ignore, false),
            (M::Personal, P::Ignore, false),
            (M::Business, P::All, false),
            (M::Personal, P::All, false),
        ];
        for (mode, policy, expected) in cases {
            assert_eq!(
                whatsapp_empty_group_list_is_newly_closed(&mode, &policy),
                expected,
                "{mode:?} + {policy:?} newly-closed should be {expected}"
            );
        }
    }

    /// The remedy must never tell an `ignore` channel to populate
    /// `allowed_groups`: a listed group passes the identity gate and the
    /// chat-type gate drops it anyway, so that advice cannot work.
    #[test]
    async fn whatsapp_empty_group_list_remedy_is_policy_aware() {
        use WhatsAppChatPolicy as P;
        let ignore = whatsapp_empty_group_list_remedy(&P::Ignore);
        assert!(
            ignore.contains("serves no group by design"),
            "the ignore remedy must explain the policy: {ignore}"
        );
        assert!(
            !ignore.contains("list the group JIDs you intend to serve in allowed_groups"),
            "the ignore remedy must not offer a list, which cannot reopen it: {ignore}"
        );
        for policy in [P::Allowlist, P::All] {
            let other = whatsapp_empty_group_list_remedy(&policy);
            assert!(
                other.contains("allowed_groups"),
                "{policy:?} is reopened by a list, so the remedy must name it: {other}"
            );
        }
    }

    /// The `all` opt-in still admits every group under BOTH modes, so nothing
    /// closed and no migration notice is owed. Control for the two rows above:
    /// without it, a validator that warned unconditionally would still pass them.
    #[test]
    async fn whatsapp_business_all_groups_is_not_flagged() {
        let toml = r#"
[channels.whatsapp.shop]
enabled = true
mode = "business"
session_path = "/tmp/wa-session"
group_policy = "all"
"#;
        let cfg: Config = toml::from_str(toml).unwrap();
        assert!(
            warnings_with_code(&cfg, WA_CLOSED_GROUPS_WARNING).is_empty(),
            "group_policy = \"all\" still admits every group, so nothing closed"
        );
    }

    const SEMANTIC_MEMORY_WARNING: &str = "memory_semantic_search_without_embedder";

    fn warnings_with_code(
        config: &Config,
        code: &str,
    ) -> Vec<crate::validation_warnings::ValidationWarning> {
        config
            .collect_warnings()
            .into_iter()
            .filter(|warning| warning.code == code)
            .collect()
    }

    fn owned_codex_args(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| (*arg).to_string()).collect()
    }

    #[::core::prelude::v1::test]
    fn risky_codex_cli_arg_matcher_accepts_attached_split_and_short_forms() {
        let cases: &[&[&str]] = &[
            &["--sandbox=danger-full-access"],
            &["--sandbox", "danger-full-access"],
            &["-s", "danger-full-access"],
            &["-sdanger-full-access"],
            &["-s=danger-full-access"],
            &["--dangerously-bypass-approvals-and-sandbox"],
            &["--yolo"],
            &["--approve-for-me"],
            &["--not-so-yolo"],
            &["--dangerously-bypass-hook-trust"],
            &["--ignore-rules"],
            &["--add-dir=/srv/shared"],
            &["--add-dir", "/srv/shared"],
            &["--cd=/srv/project"],
            &["--cd", "/srv/project"],
            &["-C/srv/project"],
            &["-C", "/srv/project"],
            &["--config=approval_policy=never"],
            &["--config", "approval_policy=on-failure"],
            &["-capproval_policy=never"],
            &["-c", "sandbox_mode='danger-full-access'"],
            &["-c", "default_permissions=unrestricted"],
            &["--config=sandbox_permissions=[\"disk-full-read-access\"]"],
            &[
                "--config",
                "sandbox_permissions=[\"disk-full-read-access\"]",
            ],
            &["-csandbox_permissions=[\"disk-full-read-access\"]"],
            &["-c", "sandbox_permissions=[\"disk-full-read-access\"]"],
            &["-cpermissions.unrestricted.network.enabled=true"],
        ];

        for case in cases {
            let args = owned_codex_args(case);
            assert_eq!(
                risky_codex_cli_arg_matches(&args).len(),
                1,
                "expected one risky match for {case:?}"
            );
        }
    }

    #[::core::prelude::v1::test]
    fn risky_codex_cli_arg_matcher_warns_on_policy_config_source_selectors() {
        let profile_cases: &[&[&str]] = &[
            &["--profile=security-review"],
            &["--profile", "security-review"],
            &["-psecurity-review"],
            &["-p=security-review"],
            &["-p", "security-review"],
        ];

        for case in profile_cases {
            let args = owned_codex_args(case);
            let matches = risky_codex_cli_arg_matches(&args);
            assert_eq!(
                matches.len(),
                1,
                "expected one profile boundary match for {case:?}"
            );
            assert_eq!(matches[0].flag.display, "--profile / -p");
        }

        let ignore_user_config = owned_codex_args(&["--ignore-user-config"]);
        let matches = risky_codex_cli_arg_matches(&ignore_user_config);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].flag.display, "--ignore-user-config");

        let missing_profile_value = owned_codex_args(&["--profile", "--ignore-user-config"]);
        let matches = risky_codex_cli_arg_matches(&missing_profile_value);
        assert_eq!(matches.len(), 1, "only --ignore-user-config should match");
        assert_eq!(matches[0].index, 1);
    }

    #[::core::prelude::v1::test]
    fn codex_cli_effective_extra_args_match_execution_and_preserve_indices() {
        let config = CodexCliConfig {
            extra_args: owned_codex_args(&[
                "   ",
                " --sandbox ",
                "\t",
                " danger-full-access ",
                " --skip-git-repo-check ",
            ]),
            ..CodexCliConfig::default()
        };

        assert_eq!(
            config.effective_extra_args().collect::<Vec<_>>(),
            vec![
                (1, "--sandbox"),
                (3, "danger-full-access"),
                (4, "--skip-git-repo-check"),
            ]
        );
    }

    #[::core::prelude::v1::test]
    fn risky_codex_cli_arg_matcher_classifies_effective_argv_after_empty_elision() {
        let cases: &[&[&str]] = &[
            &["--sandbox", "   ", "danger-full-access"],
            &["-c", "", "mcp_servers.review={ command = 'server' }"],
            &["--profile", "\t", "security-review"],
            &["--cd", " ", "/srv/project"],
            &["--add-dir", "", "/srv/shared"],
            &["--enable", "  ", "request_permissions_tool"],
        ];

        for case in cases {
            let args = owned_codex_args(case);
            let matches = risky_codex_cli_arg_matches(&args);
            assert_eq!(
                matches.len(),
                1,
                "expected one risky match after empty argv elision for {case:?}"
            );
            assert_eq!(matches[0].index, 0, "warning must retain original index");
        }
    }

    #[::core::prelude::v1::test]
    fn risky_codex_cli_arg_matcher_warns_on_finite_config_families() {
        let cases: &[&[&str]] = &[
            &["--config=mcp_servers.review={ command = 'server' }"],
            &["--config", "hooks.after_agent={ command = ['audit'] }"],
            &["-cplugins.audit.enabled=true"],
            &["-c", "apps.connector.enabled=true"],
            &["--config=notify=['audit-helper']"],
            &["-c", "projects.'C:\\repo'.trust_level='trusted'"],
            &["--config=windows.sandbox='elevated'"],
            &["-c", "shell_environment_policy.inherit='all'"],
            &["--config=allow_login_shell=true"],
            &["-cauto_review.policy='review everything'"],
        ];

        for case in cases {
            let args = owned_codex_args(case);
            assert_eq!(
                risky_codex_cli_arg_matches(&args).len(),
                1,
                "expected one finite-family match for {case:?}"
            );
        }
    }

    #[::core::prelude::v1::test]
    fn risky_codex_cli_arg_matcher_keeps_unrelated_config_families_silent() {
        let cases: &[&[&str]] = &[
            &["--config=mcp_server.review.command='server'"],
            &["--config", "mcp_servers_extra.review.command='server'"],
            &["-chook.after_agent.command='audit'"],
            &["-c", "plugins_extra.audit.enabled=true"],
            &["--config=applications.connector.enabled=true"],
            &["-c", "notifications=['audit-helper']"],
            &["--config=project.repo.trust_level='trusted'"],
            &["-cwindows_extra.sandbox='elevated'"],
            &["-c", "shell_environment_policy_extra.inherit='all'"],
            &["--config=allow_login_shell_extra=true"],
            &["-c", "model='gpt-5.6'"],
            &["-c", "mcp_servers=''"],
            &["-c", "hooks=\"\""],
        ];

        for case in cases {
            let args = owned_codex_args(case);
            assert!(
                risky_codex_cli_arg_matches(&args).is_empty(),
                "unexpected finite-family match for {case:?}"
            );
        }
    }

    #[::core::prelude::v1::test]
    fn risky_codex_cli_arg_matcher_warns_on_feature_toggle_selectors() {
        let cases: &[&[&str]] = &[
            &["--enable", "request_permissions_tool"],
            &["--enable=web_search_request"],
            &["--disable", "plugins"],
            &["--disable=apps"],
            &["--config=features.request_permissions_tool=true"],
            &["--config", "features.web_search_request=true"],
            &["-cfeatures.use_legacy_landlock=true"],
            &["-c", "features.plugins=false"],
            &["--config=features={ request_permissions_tool = true }"],
            &["-c", "use_legacy_landlock=true"],
        ];

        for case in cases {
            let args = owned_codex_args(case);
            assert_eq!(
                risky_codex_cli_arg_matches(&args).len(),
                1,
                "expected one feature boundary match for {case:?}"
            );
        }

        let missing_value_before_flag = owned_codex_args(&["--enable", "--ignore-rules"]);
        let matches = risky_codex_cli_arg_matches(&missing_value_before_flag);
        assert_eq!(matches.len(), 1, "only --ignore-rules should match");
        assert_eq!(matches[0].index, 1);

        let after_terminator = owned_codex_args(&["--", "--enable", "request_permissions_tool"]);
        assert!(risky_codex_cli_arg_matches(&after_terminator).is_empty());
    }

    #[::core::prelude::v1::test]
    fn risky_codex_cli_arg_matcher_warns_on_approval_reviewer_overrides() {
        let cases: &[&[&str]] = &[
            &["--config=approvals_reviewer=\"auto_review\""],
            &["--config", "approvals_reviewer='guardian_subagent'"],
            &["-capprovals_reviewer=auto_review"],
            &["-c", "approvals_reviewer=\"auto_review\" # comment"],
        ];

        for case in cases {
            let args = owned_codex_args(case);
            assert_eq!(
                risky_codex_cli_arg_matches(&args).len(),
                1,
                "expected one approval-reviewer boundary match for {case:?}"
            );
        }
    }

    #[::core::prelude::v1::test]
    fn risky_codex_cli_arg_matcher_warns_on_workspace_write_boundary_overrides() {
        let cases: &[&[&str]] = &[
            &["--config=sandbox_workspace_write.network_access=true"],
            &["--config", "sandbox_workspace_write.network_access=false"],
            &["-csandbox_workspace_write.network_access=true"],
            &["-c", "sandbox_workspace_write.network_access=true"],
            &["--config=sandbox_workspace_write.writable_roots=[\"/srv/shared\"]"],
            &[
                "--config",
                "sandbox_workspace_write.writable_roots=[\"/srv/shared\"]",
            ],
            &["-csandbox_workspace_write.writable_roots=[\"/srv/shared\"]"],
            &[
                "-c",
                "sandbox_workspace_write.writable_roots=[\"/srv/shared\"]",
            ],
            &["--config=sandbox_workspace_write={ network_access = true }"],
            &[
                "-c",
                "sandbox_workspace_write={ writable_roots = [\"/srv/shared\"] }",
            ],
        ];

        for case in cases {
            let args = owned_codex_args(case);
            assert_eq!(
                risky_codex_cli_arg_matches(&args).len(),
                1,
                "expected one workspace-write boundary match for {case:?}"
            );
        }
    }

    #[::core::prelude::v1::test]
    fn risky_codex_cli_arg_matcher_warns_on_temp_directory_boundary_overrides() {
        let cases: &[&[&str]] = &[
            &["--config=sandbox_workspace_write.exclude_tmpdir_env_var=false"],
            &[
                "--config",
                "sandbox_workspace_write.exclude_tmpdir_env_var=false",
            ],
            &["-csandbox_workspace_write.exclude_tmpdir_env_var=false"],
            &["-c", "sandbox_workspace_write.exclude_tmpdir_env_var=false"],
            &["--config=sandbox_workspace_write.exclude_slash_tmp=false"],
            &[
                "--config",
                "sandbox_workspace_write.exclude_slash_tmp=false",
            ],
            &["-csandbox_workspace_write.exclude_slash_tmp=false"],
            &["-c", "sandbox_workspace_write.exclude_slash_tmp=false"],
        ];

        for case in cases {
            let args = owned_codex_args(case);
            assert_eq!(
                risky_codex_cli_arg_matches(&args).len(),
                1,
                "expected one temp-directory boundary match for {case:?}"
            );
        }
    }

    #[::core::prelude::v1::test]
    fn risky_codex_cli_arg_matcher_warns_on_all_nonempty_sandbox_mode_overrides() {
        let cases: &[&[&str]] = &[
            &["--config=sandbox_mode=workspace-write"],
            &["--config", "sandbox_mode=read-only"],
            &["-csandbox_mode=\"danger-full-access\" # comment"],
            &["-c", "sandbox_mode=\"danger-full-access\" # comment"],
            &["-c", "sandbox_mode=\"danger\\u002dfull\\u002daccess\""],
        ];

        for case in cases {
            let args = owned_codex_args(case);
            assert_eq!(
                risky_codex_cli_arg_matches(&args).len(),
                1,
                "expected one sandbox-mode boundary match for {case:?}"
            );
        }
    }

    #[::core::prelude::v1::test]
    fn risky_codex_cli_arg_matcher_rejects_safe_near_matches_and_missing_values() {
        let cases: &[&[&str]] = &[
            &[],
            &[""],
            &["--sandbox=workspace-write"],
            &["--sandbox", "read-only"],
            &["--sandboxed=danger-full-access"],
            &["--dangerously-bypass-approvals"],
            &["--dangerously-bypass-approvals-and-sandbox=true"],
            &["--approve-for-me=true"],
            &["--not-so-yolo=true"],
            &["--ignore-ruleset"],
            &["--ignore-rules=true"],
            &["--add-dir"],
            &["--add-dir", "--ignore-rules"],
            &["--cd"],
            &["-C"],
            &["--profile"],
            &["-p"],
            &["--profile="],
            &["-p="],
            &["--profiles=security-review"],
            &["--enable"],
            &["--disable"],
            &["--enable="],
            &["--disable="],
            &["--enabled=request_permissions_tool"],
            &["--disable-feature=plugins"],
            &["--ignore-user-config=true"],
            &["--ignore-user-configuration"],
            &["--ignore-user-config-extra"],
            &["--config"],
            &["--config=model=gpt-5.6"],
            &["-c", "model=gpt-5.6"],
            &["-c", "approvals_reviewer="],
            &["-c", "sandbox_permissions="],
            &["-c", "sandbox_mode="],
            &["-c", "sandbox_mode=''"],
            &["-c", "sandbox_workspace_write="],
            &["-c", "sandbox_workspace_write.network_access="],
            &["-c", "sandbox_workspace_write.writable_roots="],
            &["-c", "sandbox_workspace_write.exclude_tmpdir_env_var="],
            &["-c", "sandbox_workspace_write.exclude_slash_tmp="],
            &["-c", "features="],
            &["-c", "features.plugins="],
            &["-c", "feature.plugins=true"],
            &["-c", "features_extra.plugins=true"],
            &["-c", "sandbox_workspace_writer.network_access=true"],
            &[
                "-c",
                "sandbox_workspace_write_extra.exclude_slash_tmp=false",
            ],
            &["--skip-git-repo-check"],
            &["--", "--ignore-rules"],
        ];

        for case in cases {
            let args = owned_codex_args(case);
            let matches = risky_codex_cli_arg_matches(&args);
            if *case == ["--add-dir", "--ignore-rules"] {
                assert_eq!(matches.len(), 1, "only --ignore-rules should match");
                assert_eq!(matches[0].index, 1);
            } else {
                assert!(matches.is_empty(), "unexpected match for {case:?}");
            }
        }
    }

    #[::core::prelude::v1::test]
    fn codex_cli_policy_config_source_warnings_are_non_blocking_and_redacted() {
        let selected_profile = "sensitive-production-profile";
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        config.codex_cli.extra_args = vec![
            format!("--profile={selected_profile}"),
            "--ignore-user-config".to_string(),
        ];

        config
            .validate()
            .expect("policy config selectors must remain allowed");
        let warnings = warnings_with_code(&config, CODEX_CLI_EXTRA_ARGS_SECURITY_BOUNDARY_WARNING);
        assert_eq!(warnings.len(), 2);
        assert_eq!(warnings[0].path, "codex_cli.extra_args[0]");
        assert_eq!(warnings[1].path, "codex_cli.extra_args[1]");
        assert!(warnings[0].message.contains("--profile / -p"));
        assert!(warnings[1].message.contains("--ignore-user-config"));
        assert!(
            warnings
                .iter()
                .all(|warning| !warning.message.contains(selected_profile)),
            "warning messages must not disclose the selected profile name"
        );
    }

    #[::core::prelude::v1::test]
    fn codex_cli_feature_toggle_warnings_are_non_blocking_and_redacted() {
        let selected_feature = "sensitive-future-capability";
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        config.codex_cli.extra_args = vec![
            format!("--enable={selected_feature}"),
            "-c".to_string(),
            format!("features.{selected_feature}=true"),
        ];

        config
            .validate()
            .expect("feature toggle selectors must remain allowed");
        let warnings = warnings_with_code(&config, CODEX_CLI_EXTRA_ARGS_SECURITY_BOUNDARY_WARNING);
        assert_eq!(warnings.len(), 2);
        assert_eq!(warnings[0].path, "codex_cli.extra_args[0]");
        assert_eq!(warnings[1].path, "codex_cli.extra_args[1]");
        assert!(warnings[0].message.contains("--enable / --disable"));
        assert!(warnings[1].message.contains("--config / -c"));
        assert!(
            warnings
                .iter()
                .all(|warning| !warning.message.contains(selected_feature)),
            "warning messages must not disclose feature names or values"
        );
    }

    #[::core::prelude::v1::test]
    fn codex_cli_executable_integration_warnings_are_non_blocking_and_redacted() {
        let sensitive_server = "private-review-server";
        let sensitive_command = "C:\\private\\review-server.exe";
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        config.codex_cli.extra_args = vec![
            "-c".to_string(),
            format!("mcp_servers.{sensitive_server}={{ command = '{sensitive_command}' }}"),
        ];

        config
            .validate()
            .expect("executable-integration overrides must remain allowed");
        let warnings = warnings_with_code(&config, CODEX_CLI_EXTRA_ARGS_SECURITY_BOUNDARY_WARNING);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].path, "codex_cli.extra_args[0]");
        assert!(warnings[0].message.contains("without blocking"));
        assert!(warnings[0].message.contains("--config / -c"));
        assert!(!warnings[0].message.contains(sensitive_server));
        assert!(!warnings[0].message.contains(sensitive_command));
    }

    #[::core::prelude::v1::test]
    fn risky_codex_cli_arg_matcher_trims_and_reports_each_argument_index() {
        let args = owned_codex_args(&[
            "  --sandbox  ",
            " danger-full-access ",
            "--ignore-rules",
            "--config=sandbox_mode=\"danger-full-access\"",
        ]);

        let matches = risky_codex_cli_arg_matches(&args);
        assert_eq!(
            matches
                .iter()
                .map(|risky_match| risky_match.index)
                .collect::<Vec<_>>(),
            vec![0, 2, 3]
        );
    }

    #[::core::prelude::v1::test]
    fn codex_cli_security_warnings_are_non_blocking_and_structured() {
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        config.codex_cli.extra_args =
            owned_codex_args(&["--sandbox", "danger-full-access", "--skip-git-repo-check"]);

        config
            .validate()
            .expect("risky operator-controlled Codex arguments must remain allowed");
        let warnings = warnings_with_code(&config, CODEX_CLI_EXTRA_ARGS_SECURITY_BOUNDARY_WARNING);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].path, "codex_cli.extra_args[0]");
        assert!(warnings[0].message.contains("without blocking"));
        assert!(warnings[0].message.contains("--sandbox"));
    }

    #[::core::prelude::v1::test]
    fn codex_cli_unknown_extra_args_remain_silent_and_allowed() {
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        config.codex_cli.extra_args =
            owned_codex_args(&["--skip-git-repo-check", "--future-codex-option=value"]);

        config
            .validate()
            .expect("unknown operator-controlled Codex arguments must remain allowed");
        assert!(
            warnings_with_code(&config, CODEX_CLI_EXTRA_ARGS_SECURITY_BOUNDARY_WARNING,).is_empty()
        );
    }

    fn suppress_semantic_memory_warning(config: &mut Config) {
        config.memory.search_mode = SearchMode::Bm25;
    }

    #[test]
    async fn collect_warnings_flags_sqlite_hybrid_without_embedder() {
        let config = Config::default();

        let warnings = warnings_with_code(&config, SEMANTIC_MEMORY_WARNING);
        assert_eq!(warnings.len(), 1);
        let warning = &warnings[0];
        assert_eq!(warning.path, "memory.search_mode");
        assert!(
            warning.message.contains("sqlite"),
            "warning should name sqlite memory: {}",
            warning.message
        );
        assert!(
            warning.message.contains("keyword-only"),
            "warning should describe the runtime degradation: {}",
            warning.message
        );
    }

    #[test]
    async fn collect_warnings_flags_sqlite_embedding_without_embedder() {
        let mut config = Config::default();
        config.memory.search_mode = SearchMode::Embedding;

        let warnings = warnings_with_code(&config, SEMANTIC_MEMORY_WARNING);
        assert_eq!(warnings.len(), 1);
        assert!(
            warnings[0].message.contains("\"embedding\""),
            "warning should name embedding search mode: {}",
            warnings[0].message
        );
    }

    #[test]
    async fn collect_warnings_silent_for_valid_hint_embedding_route() {
        let mut config = Config::default();
        config.memory.embedding_model = "hint:semantic".to_string();
        config.providers.models.openai.insert(
            "default".to_string(),
            OpenAIModelProviderConfig {
                base: ModelProviderConfig {
                    api_key: Some("k".to_string()),
                    model: Some("gpt-4o".to_string()),
                    ..Default::default()
                },
            },
        );
        config.embedding_routes.push(EmbeddingRouteConfig {
            hint: "semantic".to_string(),
            model_provider: "openai.default".to_string(),
            model: "text-embedding-3-small".to_string(),
            dimensions: Some(1536),
            api_key: None,
        });
        config.validate().expect("hint route should validate");

        assert!(
            warnings_with_code(&config, SEMANTIC_MEMORY_WARNING).is_empty(),
            "valid hint route must count as an effective embedder"
        );
    }

    #[test]
    async fn collect_warnings_flags_missing_hint_embedding_route() {
        let mut config = Config::default();
        config.memory.embedding_model = "hint:semantic".to_string();

        let warnings = warnings_with_code(&config, SEMANTIC_MEMORY_WARNING);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    async fn collect_warnings_flags_invalid_hint_embedding_route() {
        let mut config = Config::default();
        config.memory.embedding_model = "hint:semantic".to_string();
        config.embedding_routes.push(EmbeddingRouteConfig {
            hint: "semantic".to_string(),
            model_provider: "none".to_string(),
            model: "text-embedding-3-small".to_string(),
            dimensions: Some(1536),
            api_key: None,
        });

        let warnings = warnings_with_code(&config, SEMANTIC_MEMORY_WARNING);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    async fn collect_warnings_silent_for_bm25_without_embedder() {
        let mut config = Config::default();
        config.memory.search_mode = SearchMode::Bm25;

        assert!(warnings_with_code(&config, SEMANTIC_MEMORY_WARNING).is_empty());
    }

    #[test]
    async fn collect_warnings_silent_for_non_sqlite_without_embedder() {
        let mut config = Config::default();
        config.memory.backend = "markdown.default".to_string();

        assert!(warnings_with_code(&config, SEMANTIC_MEMORY_WARNING).is_empty());
    }

    const INERT_MEMORY_KNOB_WARNING: &str = "memory_config_knob_inert";

    fn inert_knob_paths(config: &Config) -> Vec<String> {
        warnings_with_code(config, INERT_MEMORY_KNOB_WARNING)
            .into_iter()
            .map(|warning| warning.path)
            .collect()
    }

    #[test]
    async fn validate_memory_semantics_silent_at_defaults() {
        let config = Config::default();

        assert!(inert_knob_paths(&config).is_empty());
    }

    #[test]
    async fn validate_memory_semantics_warns_for_non_default_retrieval_stages() {
        let mut config = Config::default();
        config.memory.retrieval_stages = vec!["fts".into()];

        assert_eq!(inert_knob_paths(&config), vec!["memory.retrieval_stages"]);
    }

    #[test]
    async fn validate_memory_semantics_warns_for_non_default_fts_early_return_score() {
        let mut config = Config::default();
        config.memory.fts_early_return_score = 0.5;

        assert_eq!(
            inert_knob_paths(&config),
            vec!["memory.fts_early_return_score"]
        );
    }

    #[test]
    async fn validate_memory_semantics_warns_for_rerank_enabled() {
        let mut config = Config::default();
        config.memory.rerank_enabled = true;

        let warnings = warnings_with_code(&config, INERT_MEMORY_KNOB_WARNING);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].path, "memory.rerank_enabled");
        assert!(
            warnings[0].message.contains("currently has no effect"),
            "warning should state the knob has no effect: {}",
            warnings[0].message
        );
    }

    #[test]
    async fn validate_memory_semantics_warns_for_non_default_rerank_threshold() {
        let mut config = Config::default();
        config.memory.rerank_threshold = 10;

        assert_eq!(inert_knob_paths(&config), vec!["memory.rerank_threshold"]);
    }

    #[test]
    async fn validate_memory_semantics_reports_each_set_knob() {
        let mut config = Config::default();
        config.memory.rerank_enabled = true;
        config.memory.rerank_threshold = 10;

        assert_eq!(
            inert_knob_paths(&config),
            vec!["memory.rerank_enabled", "memory.rerank_threshold"]
        );
    }

    #[test]
    async fn config_validate_accepts_classifier_provider_pointing_at_existing_alias() {
        let toml = r#"
            [providers.models.custom.default]
            api_key = "k1"
            model = "qwen3.6-plus"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"

            [providers.models.custom.kimi-k2-5]
            api_key = "k2"
            model = "kimi-k2.5"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"

            [risk_profiles.default]
            level = "supervised"

            [agents.default]
            enabled = true
            model_provider = "custom.default"
            risk_profile = "default"
            classifier_provider = "custom.kimi-k2-5"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        cfg.validate()
            .expect("validate must succeed for resolvable ref");
        assert_eq!(
            cfg.agents
                .get("default")
                .unwrap()
                .classifier_provider
                .as_str(),
            "custom.kimi-k2-5"
        );
    }

    #[test]
    async fn config_validate_accepts_empty_classifier_provider_as_inheritance_signal() {
        // No classifier_provider field at all → must validate, must remain
        // the empty default. This pins backward compatibility.
        let toml = r#"
            [providers.models.custom.default]
            api_key = "k"
            model = "qwen3.6-plus"
            uri = "https://example.com/v1"
            wire_api = "chat_completions"

            [risk_profiles.default]
            level = "supervised"

            [agents.default]
            enabled = true
            model_provider = "custom.default"
            risk_profile = "default"
        "#;
        let cfg: Config = toml::from_str(toml).unwrap();
        cfg.validate()
            .expect("missing classifier_provider must validate");
        assert!(
            cfg.agents
                .get("default")
                .unwrap()
                .classifier_provider
                .is_empty()
        );
    }

    fn provider_entry_with_fallback(fallback: &[&str]) -> OpenAIModelProviderConfig {
        OpenAIModelProviderConfig {
            base: ModelProviderConfig {
                model: Some("gpt-4o".to_string()),
                fallback: fallback
                    .iter()
                    .map(|s| crate::providers::ModelProviderRef::new(*s))
                    .collect(),
                ..Default::default()
            },
        }
    }

    #[test]
    async fn fallback_warns_on_dangling_ref() {
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        config.providers.models.openai.insert(
            "primary".to_string(),
            provider_entry_with_fallback(&["openai.ghost"]),
        );

        let warnings = config.collect_warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "dangling_fallback_ref");
        assert_eq!(
            warnings[0].path,
            "providers.models.openai.primary.fallback[0]"
        );
    }

    #[test]
    async fn fallback_no_warning_when_ref_resolves() {
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        config.providers.models.openai.insert(
            "primary".to_string(),
            provider_entry_with_fallback(&["openai.backup"]),
        );
        config
            .providers
            .models
            .openai
            .insert("backup".to_string(), provider_entry_with_fallback(&[]));

        assert!(config.collect_warnings().is_empty());
    }

    #[test]
    async fn fallback_warns_on_two_node_cycle() {
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        config
            .providers
            .models
            .openai
            .insert("a".to_string(), provider_entry_with_fallback(&["openai.b"]));
        config
            .providers
            .models
            .openai
            .insert("b".to_string(), provider_entry_with_fallback(&["openai.a"]));

        let cycle_warnings: Vec<_> = config
            .collect_warnings()
            .into_iter()
            .filter(|w| w.code == "fallback_cycle")
            .collect();
        assert!(
            !cycle_warnings.is_empty(),
            "a->b->a must surface at least one fallback_cycle warning"
        );
    }

    #[test]
    async fn fallback_self_reference_is_a_cycle() {
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        config.providers.models.openai.insert(
            "loop".to_string(),
            provider_entry_with_fallback(&["openai.loop"]),
        );

        let warnings = config.collect_warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "fallback_cycle");
    }

    #[test]
    async fn fallback_empty_ref_is_skipped() {
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        config
            .providers
            .models
            .openai
            .insert("primary".to_string(), provider_entry_with_fallback(&[""]));

        assert!(config.collect_warnings().is_empty());
    }

    #[test]
    async fn fallback_warns_when_chain_exceeds_max_depth() {
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        let n = crate::providers::MAX_FALLBACK_DEPTH + 2;
        for i in 0..n {
            let next = if i + 1 < n {
                vec![format!("openai.a{}", i + 1)]
            } else {
                vec![]
            };
            let refs: Vec<&str> = next.iter().map(String::as_str).collect();
            config
                .providers
                .models
                .openai
                .insert(format!("a{i}"), provider_entry_with_fallback(&refs));
        }

        let depth_warnings: Vec<_> = config
            .collect_warnings()
            .into_iter()
            .filter(|w| w.code == "max_fallback_depth_exceeded")
            .collect();
        assert!(
            !depth_warnings.is_empty(),
            "a chain deeper than MAX_FALLBACK_DEPTH must surface a max_fallback_depth_exceeded warning"
        );
    }

    #[test]
    async fn fallback_models_warns_on_empty_entry() {
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        let mut entry = provider_entry_with_fallback(&[]);
        entry.base.fallback_models = vec!["".to_string()];
        config
            .providers
            .models
            .openai
            .insert("primary".to_string(), entry);

        let warnings = config.collect_warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "empty_fallback_model");
    }

    #[test]
    async fn fallback_models_warns_on_duplicate_of_primary() {
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        let mut entry = provider_entry_with_fallback(&[]);
        entry.base.fallback_models = vec!["gpt-4o".to_string()];
        config
            .providers
            .models
            .openai
            .insert("primary".to_string(), entry);

        let warnings = config.collect_warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "fallback_model_duplicates_primary");
    }

    #[test]
    async fn fallback_models_distinct_entries_do_not_warn() {
        let mut config = Config::default();
        suppress_semantic_memory_warning(&mut config);
        let mut entry = provider_entry_with_fallback(&[]);
        entry.base.fallback_models = vec!["gpt-4o-mini".to_string()];
        config
            .providers
            .models
            .openai
            .insert("primary".to_string(), entry);

        assert!(config.collect_warnings().is_empty());
    }

    fn delegate_roster_config() -> Config {
        let mut cfg = Config::default();
        cfg.providers.models.ollama.insert(
            "default".to_string(),
            crate::schema::OllamaModelProviderConfig::default(),
        );
        cfg.risk_profiles
            .insert("shared".to_string(), RiskProfileConfig::default());
        cfg.risk_profiles
            .insert("lore".to_string(), RiskProfileConfig::default());
        for (alias, profile) in [
            ("aaa", "shared"),
            ("aaatools", "shared"),
            ("aaalore", "lore"),
        ] {
            cfg.agents.insert(
                alias.to_string(),
                AliasedAgentConfig {
                    risk_profile: profile.into(),
                    model_provider: "ollama.default".into(),
                    ..AliasedAgentConfig::default()
                },
            );
        }
        cfg
    }

    fn assert_delegate_target_modes_match_roster(cfg: &Config, caller_alias: &str) {
        // `delegate_target_mode()` is the single-target convenience wrapper
        // used by runtime admission. Keep it mathematically tied to the
        // materialized roster so future resolver changes cannot split the two.
        let roster: std::collections::BTreeMap<_, _> = cfg
            .reachable_delegate_target_configs(caller_alias)
            .into_iter()
            .map(|target| (target.agent, target.mode))
            .collect();

        for alias in cfg.agents.keys() {
            assert_eq!(
                cfg.delegate_target_mode(caller_alias, alias),
                roster.get(alias).copied(),
                "direct delegate target mode must match materialized roster for caller {caller_alias:?}, target {alias:?}"
            );
        }

        assert_eq!(
            cfg.delegate_target_mode(caller_alias, ""),
            None,
            "empty target aliases are never reachable"
        );
        assert_eq!(
            cfg.delegate_target_mode(caller_alias, "missing-agent"),
            None,
            "unknown target aliases are never reachable"
        );
    }

    #[test]
    async fn reachable_targets_auto_allows_same_profile_peers() {
        let cfg = delegate_roster_config();
        assert_eq!(cfg.reachable_delegate_targets("aaa"), vec!["aaatools"]);
    }

    #[test]
    async fn reachable_targets_excludes_self() {
        let cfg = delegate_roster_config();
        assert!(
            !cfg.reachable_delegate_targets("aaa")
                .iter()
                .any(|a| a == "aaa")
        );
    }

    #[test]
    async fn reachable_targets_opt_out_hides_peers_keeps_explicit() {
        let mut cfg = delegate_roster_config();
        let aaa = cfg.agents.get_mut("aaa").unwrap();
        aaa.delegate_same_risk_profile = false;
        aaa.delegates = vec![DelegateTargetConfig::bounded("aaalore")];
        assert_eq!(cfg.reachable_delegate_targets("aaa"), vec!["aaalore"]);
    }

    #[test]
    async fn reachable_targets_unions_peers_and_explicit_cross_profile() {
        let mut cfg = delegate_roster_config();
        cfg.agents.get_mut("aaa").unwrap().delegates =
            vec![DelegateTargetConfig::bounded("aaalore")];
        assert_eq!(
            cfg.reachable_delegate_targets("aaa"),
            vec!["aaalore", "aaatools"]
        );
    }

    #[test]
    async fn delegate_target_mode_matches_reachable_roster_matrix() {
        // Matrix coverage for the roster/mode invariant: implicit peers,
        // explicit bounded targets, explicit independent overrides,
        // opt-out mode, disabled targets, and missing callers.
        let mut cfg = delegate_roster_config();
        assert_delegate_target_modes_match_roster(&cfg, "aaa");
        assert_delegate_target_modes_match_roster(&cfg, "nope");

        cfg.agents.get_mut("aaa").unwrap().delegates =
            vec![DelegateTargetConfig::bounded("aaalore")];
        assert_delegate_target_modes_match_roster(&cfg, "aaa");

        cfg.agents.get_mut("aaa").unwrap().delegates = vec![DelegateTargetConfig {
            agent: "aaatools".to_string(),
            mode: DelegateExecutionMode::Independent,
        }];
        assert_delegate_target_modes_match_roster(&cfg, "aaa");

        cfg.agents
            .get_mut("aaa")
            .unwrap()
            .delegate_same_risk_profile = false;
        assert_delegate_target_modes_match_roster(&cfg, "aaa");

        cfg.agents.get_mut("aaatools").unwrap().enabled = false;
        assert_delegate_target_modes_match_roster(&cfg, "aaa");
    }

    #[test]
    async fn delegate_target_mode_normalizes_target_alias_and_overrides_implicit_mode() {
        // Direct lookup receives a user/tool argument, not a config entry, so it
        // must trim the requested alias and still prefer explicit mode over the
        // implicit same-profile bounded default.
        let mut cfg = delegate_roster_config();
        cfg.agents.get_mut("aaa").unwrap().delegates = vec![DelegateTargetConfig {
            agent: "aaatools".to_string(),
            mode: DelegateExecutionMode::Independent,
        }];

        assert_eq!(
            cfg.delegate_target_mode("aaa", "aaatools"),
            Some(DelegateExecutionMode::Independent),
            "explicit entries must override same-profile bounded reach"
        );
        assert_eq!(
            cfg.delegate_target_mode("aaa", " aaatools "),
            Some(DelegateExecutionMode::Independent),
            "direct mode lookup must match reachable roster alias normalization"
        );
        assert_eq!(
            cfg.delegate_target_mode("aaa", "aaa"),
            None,
            "self-delegation is never reachable"
        );

        cfg.agents.get_mut("aaatools").unwrap().enabled = false;
        assert_eq!(
            cfg.delegate_target_mode("aaa", "aaatools"),
            None,
            "disabled explicit targets are not reachable"
        );
    }

    #[test]
    async fn reachable_targets_unknown_caller_is_empty() {
        let cfg = delegate_roster_config();
        assert!(cfg.reachable_delegate_targets("nope").is_empty());
    }

    #[test]
    async fn reachable_targets_excludes_disabled_same_profile_peer() {
        let mut cfg = delegate_roster_config();
        cfg.agents.get_mut("aaatools").unwrap().enabled = false;
        assert!(
            cfg.reachable_delegate_targets("aaa").is_empty(),
            "disabled same-profile peer must not be reachable"
        );
    }

    #[test]
    async fn reachable_targets_excludes_disabled_explicit_delegate() {
        let mut cfg = delegate_roster_config();
        cfg.agents
            .get_mut("aaa")
            .unwrap()
            .delegate_same_risk_profile = false;
        cfg.agents.get_mut("aaa").unwrap().delegates =
            vec![DelegateTargetConfig::bounded("aaalore")];
        cfg.agents.get_mut("aaalore").unwrap().enabled = false;
        assert!(
            cfg.reachable_delegate_targets("aaa").is_empty(),
            "disabled explicit delegate must not be reachable"
        );
    }

    #[test]
    async fn validate_rejects_dangling_delegate_target() {
        let mut cfg = delegate_roster_config();
        cfg.agents.get_mut("aaa").unwrap().delegates = vec![DelegateTargetConfig::bounded("ghost")];
        let err = cfg.validate().expect_err("dangling delegate must fail");
        assert!(format!("{err:#}").contains("delegates"), "{err:#}");
    }

    #[test]
    async fn validate_rejects_self_delegate() {
        let mut cfg = delegate_roster_config();
        cfg.agents.get_mut("aaa").unwrap().delegates = vec![DelegateTargetConfig::bounded("aaa")];
        let err = cfg.validate().expect_err("self-delegate must fail");
        assert!(format!("{err:#}").contains("itself"), "{err:#}");
    }

    #[test]
    async fn validate_rejects_duplicate_delegate_target() {
        // Duplicate detection is by target alias, not by the full object. A
        // bounded and independent entry for the same agent would otherwise make
        // runtime mode selection order-dependent.
        let mut cfg = delegate_roster_config();
        cfg.agents.get_mut("aaa").unwrap().delegates = vec![
            DelegateTargetConfig::bounded("aaalore"),
            DelegateTargetConfig {
                agent: "aaalore".to_string(),
                mode: DelegateExecutionMode::Independent,
            },
        ];
        let err = cfg.validate().expect_err("duplicate delegate must fail");
        assert!(format!("{err:#}").contains("duplicates"), "{err:#}");
    }

    #[test]
    async fn delegate_targets_parse_strings_as_bounded() {
        // Existing configs used bare strings. They must continue to parse as
        // bounded targets so simply upgrading the binary does not widen any
        // delegate's execution mode.
        let toml_src = "\
[agents.legacy]
risk_profile = \"shared\"
model_provider = \"ollama.default\"
delegates = [\"reviewer\"]
";
        let cfg: Config = toml::from_str(toml_src).expect("legacy delegate config parses");
        let agent = cfg.agents.get("legacy").expect("agent present");
        assert_eq!(
            agent.delegates,
            vec![DelegateTargetConfig::bounded("reviewer")]
        );
    }

    #[test]
    async fn delegate_targets_parse_objects_and_default_mode() {
        // Object entries are the new surface. Omitted mode deliberately keeps
        // the legacy bounded default; independent mode must be explicit.
        let toml_src = "\
[agents.legacy]
risk_profile = \"shared\"
model_provider = \"ollama.default\"
delegates = [
  { agent = \"reviewer\" },
  { agent = \"sysadmin\", mode = \"independent\" },
]
";
        let cfg: Config = toml::from_str(toml_src).expect("object delegate config parses");
        let agent = cfg.agents.get("legacy").expect("agent present");
        assert_eq!(
            agent.delegates,
            vec![
                DelegateTargetConfig::bounded("reviewer"),
                DelegateTargetConfig {
                    agent: "sysadmin".to_string(),
                    mode: DelegateExecutionMode::Independent,
                },
            ]
        );
    }

    #[test]
    async fn delegate_targets_parse_mixed_string_and_object_entries() {
        // Operators may migrate one target at a time. Mixed arrays are therefore
        // part of the supported config shape, not just a permissive accident.
        let toml_src = "\
[agents.legacy]
risk_profile = \"shared\"
model_provider = \"ollama.default\"
delegates = [
  \"reviewer\",
  { agent = \"sysadmin\", mode = \"independent\" },
]
";
        let cfg: Config = toml::from_str(toml_src).expect("mixed delegate config parses");
        let agent = cfg.agents.get("legacy").expect("agent present");
        assert_eq!(
            agent.delegates,
            vec![
                DelegateTargetConfig::bounded("reviewer"),
                DelegateTargetConfig {
                    agent: "sysadmin".to_string(),
                    mode: DelegateExecutionMode::Independent,
                },
            ]
        );
    }

    #[test]
    async fn delegate_targets_object_array_round_trips_through_set_prop_and_validate() {
        // The programmatic config-editing path must accept the same mixed shape
        // as TOML loading, then serialize back without dropping the agents
        // table. This is the CLI/config-tool version of the original failure.
        let mut cfg = delegate_roster_config();
        let value = serde_json::json!([
            "aaatools",
            { "agent": "aaalore", "mode": "independent" }
        ])
        .to_string();

        cfg.set_prop("agents.aaa.delegates", &value)
            .expect("delegate object-array set_prop accepts mixed JSON entries");
        cfg.validate()
            .expect("valid delegate object-array config validates");
        assert_eq!(
            cfg.reachable_delegate_target_configs("aaa"),
            vec![
                DelegateTargetConfig {
                    agent: "aaalore".to_string(),
                    mode: DelegateExecutionMode::Independent,
                },
                DelegateTargetConfig::bounded("aaatools"),
            ],
            "explicit independent entry should not remove configured agents from reachability"
        );

        let toml = toml::to_string(&cfg).expect("config serializes");
        let reparsed: Config = toml::from_str(&toml).expect("serialized config reparses");
        assert_eq!(
            reparsed
                .agents
                .get("aaa")
                .expect("agent remains visible")
                .delegates,
            vec![
                DelegateTargetConfig::bounded("aaatools"),
                DelegateTargetConfig {
                    agent: "aaalore".to_string(),
                    mode: DelegateExecutionMode::Independent,
                },
            ]
        );
        assert!(reparsed.agents.contains_key("aaa"));
        assert!(reparsed.agents.contains_key("aaatools"));
        assert!(reparsed.agents.contains_key("aaalore"));
    }

    #[test]
    async fn delegate_targets_serialize_as_object_array_with_explicit_mode() {
        // Saving always emits explicit objects, including bounded mode. That
        // makes future diffs unambiguous and avoids a lossy string/object mix on
        // writeback.
        let mut cfg = delegate_roster_config();
        cfg.agents.get_mut("aaa").unwrap().delegates = vec![
            DelegateTargetConfig::bounded("aaatools"),
            DelegateTargetConfig {
                agent: "aaalore".to_string(),
                mode: DelegateExecutionMode::Independent,
            },
        ];
        let toml = toml::to_string(&cfg).expect("config serializes");
        assert!(toml.contains("agent = \"aaatools\""), "{toml}");
        assert!(toml.contains("mode = \"bounded\""), "{toml}");
        assert!(toml.contains("agent = \"aaalore\""), "{toml}");
        assert!(toml.contains("mode = \"independent\""), "{toml}");
    }

    #[test]
    async fn delegate_fields_default_for_legacy_config_roundtrip() {
        let toml_src = "\
[agents.legacy]
risk_profile = \"shared\"
model_provider = \"ollama.default\"
";
        let cfg: Config = toml::from_str(toml_src).expect("legacy config parses");
        let agent = cfg.agents.get("legacy").expect("agent present");
        assert!(agent.delegate_same_risk_profile, "default must be true");
        assert!(agent.delegates.is_empty(), "default must be empty");
    }

    #[test]
    async fn channel_presence_names_are_unique_and_undeliverable_set_is_fixed() {
        let presence = ChannelsConfig::default().channel_presence();
        let mut seen = std::collections::HashSet::new();
        for (name, _, _) in presence {
            assert!(seen.insert(name), "duplicate channel_presence name: {name}");
        }
        let mut undeliverable: Vec<&str> = presence
            .iter()
            .filter(|(_, _, deliverable)| !*deliverable)
            .map(|(name, _, _)| *name)
            .collect();
        undeliverable.sort_unstable();
        assert_eq!(
            undeliverable,
            ["amqp", "filesystem", "mqtt", "voice_duplex", "voice_wake"],
            "only input-only transports may be non-deliverable; update channel_presence and is_channel_deliverable together"
        );
    }

    // ── Serde-default vs struct-Default drift guards ──────────────
    //
    // The save path prunes fields whose value equals serde's default.
    // If `#[serde(default)]` and `impl Default` disagree, a save →
    // load round-trip silently flips the field to the serde default,
    // which is. These tests catch that drift: an empty TOML
    // table (the extreme case of pruning — all fields pruned away)
    // must deserialize to the same value as the struct's `Default`.

    // ── Schema-walked reload round-trip smoke battery ─────────────
    //
    // Reload re-reads config.toml and rebuilds the in-memory Config; any
    // scalar field that does not survive a serialize -> deserialize cycle
    // is silently lost on reload (the class). This walks every
    // scalar prop the derive exposes, mutates it off-default, round-trips
    // the whole Config through TOML, and asserts the mutated value comes
    // back. Driven entirely off prop_fields() so it tracks the schema.

    fn values_match(a: &str, b: &str) -> bool {
        if a == b {
            return true;
        }
        match (a.parse::<f64>(), b.parse::<f64>()) {
            (Ok(x), Ok(y)) => (x - y).abs() < f64::EPSILON,
            _ => false,
        }
    }

    fn off_default_value_for(
        field: &crate::traits::PropFieldInfo,
        current: &str,
    ) -> Option<String> {
        match field.kind {
            PropKind::Bool => Some(if current == "true" { "false" } else { "true" }.to_string()),
            PropKind::Integer => {
                let n: i128 = current.parse().unwrap_or(0);
                Some((n.wrapping_add(7)).to_string())
            }
            PropKind::Float => {
                let n: f64 = current.parse().unwrap_or(0.0);
                Some(format!("{:.3}", n + 1.5))
            }
            PropKind::String => {
                let probe = "zc_reload_probe";
                if current == probe {
                    Some("zc_reload_probe_alt".to_string())
                } else {
                    Some(probe.to_string())
                }
            }
            PropKind::Enum => field.enum_variants.and_then(|variants| {
                variants()
                    .into_iter()
                    .find(|v| v != current)
                    .or_else(|| variants().into_iter().next())
            }),
            PropKind::AliasRef
            | PropKind::StringArray
            | PropKind::ObjectArray
            | PropKind::Object
            | PropKind::SecretMap => None,
        }
    }

    #[test]
    async fn every_scalar_field_survives_toml_reload_round_trip() {
        let mut config = Config::default();
        let fields = config.prop_fields();

        let mut mutated: Vec<(String, String)> = Vec::new();
        let mut skipped_non_scalar = 0usize;
        let mut skipped_unsettable = 0usize;

        for field in &fields {
            if field.is_secret {
                continue;
            }
            let Ok(current) = config.get_prop(&field.name) else {
                skipped_unsettable += 1;
                continue;
            };
            let Some(target) = off_default_value_for(field, &current) else {
                skipped_non_scalar += 1;
                continue;
            };
            if config.set_prop(&field.name, &target).is_err() {
                skipped_unsettable += 1;
                continue;
            }
            mutated.push((field.name.clone(), target));
        }

        let serialized = toml::to_string(&config).expect("mutated config must serialize");
        let reloaded: Config =
            toml::from_str(&serialized).expect("serialized config must deserialize");

        let mut lost: Vec<String> = Vec::new();
        for (name, expected) in &mutated {
            match reloaded.get_prop(name) {
                Ok(got) if values_match(&got, expected) => {}
                Ok(got) => lost.push(format!("{name}: set {expected:?}, reloaded {got:?}")),
                Err(e) => lost.push(format!("{name}: set {expected:?}, reload read failed: {e}")),
            }
        }

        assert!(
            lost.is_empty(),
            "{} scalar field(s) did not survive a TOML reload round-trip ({} mutated, {} non-scalar skipped, {} unsettable skipped):\n{}",
            lost.len(),
            mutated.len(),
            skipped_non_scalar,
            skipped_unsettable,
            lost.join("\n")
        );
        assert!(
            mutated.len() > 100,
            "smoke battery covered only {} scalar fields; schema walk likely regressed",
            mutated.len()
        );
    }

    #[test]
    async fn empty_table_round_trips_to_http_request_config_default() {
        let from_empty: HttpRequestConfig = toml::from_str("").unwrap();
        let default = HttpRequestConfig::default();
        assert_eq!(from_empty.enabled, default.enabled);
        assert_eq!(from_empty.allowed_domains, default.allowed_domains);
        assert_eq!(from_empty.max_response_size, default.max_response_size);
    }

    #[test]
    async fn empty_table_round_trips_to_web_fetch_config_default() {
        let from_empty: WebFetchConfig = toml::from_str("").unwrap();
        let default = WebFetchConfig::default();
        assert_eq!(from_empty.enabled, default.enabled);
    }

    #[test]
    async fn empty_table_round_trips_to_web_search_config_default() {
        let from_empty: WebSearchConfig = toml::from_str("").unwrap();
        let default = WebSearchConfig::default();
        assert_eq!(from_empty.enabled, default.enabled);
    }

    #[test]
    async fn empty_table_round_trips_to_memory_config_default() {
        let from_empty: MemoryConfig = toml::from_str("").unwrap();
        let default = MemoryConfig::default();
        assert_eq!(from_empty.backend, default.backend);
    }

    #[test]
    async fn empty_table_round_trips_to_tunnel_config_default() {
        let from_empty: TunnelConfig = toml::from_str("").unwrap();
        let default = TunnelConfig::default();
        assert_eq!(from_empty.tunnel_provider, default.tunnel_provider);
    }

    #[test]
    async fn empty_table_round_trips_to_hooks_config_default() {
        let from_empty: HooksConfig = toml::from_str("").unwrap();
        let default = HooksConfig::default();
        assert_eq!(from_empty.enabled, default.enabled);
    }

    #[test]
    async fn empty_table_round_trips_to_builtin_hooks_config_default() {
        let from_empty: BuiltinHooksConfig = toml::from_str("").unwrap();
        let default = BuiltinHooksConfig::default();
        assert_eq!(from_empty.command_logger, default.command_logger);
    }

    #[test]
    async fn whitespace_only_model_provider_is_not_dispatchable() {
        // whitespace-only model_provider should not be dispatchable
        let agent = AliasedAgentConfig {
            enabled: true,
            risk_profile: "default".into(),
            runtime_profile: "default".into(),
            model_provider: "   ".into(),
            ..Default::default()
        };
        assert!(!agent.is_dispatchable());
        // non-empty model_provider should be dispatchable
        let agent = AliasedAgentConfig {
            enabled: true,
            risk_profile: "default".into(),
            runtime_profile: "default".into(),
            model_provider: "gpt4".into(),
            ..Default::default()
        };
        assert!(agent.is_dispatchable());
    }
}
