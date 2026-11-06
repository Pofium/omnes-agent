//! Per-family model provider configs: typed family slots plus the endpoint
//! enums that resolve each family base URL through the `ModelEndpoint` trait.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::<Family>ModelProviderConfig` path is unchanged.

use super::{AuthMode, FamilyEndpoint, ModelEndpoint, ModelProviderConfig, WireApi};
use omnesagent_macros::Configurable;
use serde::{Deserialize, Serialize};

// ── Per-family model model_provider configs ────────────────────────────
//
// Each family carries its own typed config (composing `ModelProviderConfig`
// via `#[serde(flatten)]`) plus a per-family `*Endpoint` enum that names the
// known endpoints and resolves them via the `ModelEndpoint` trait. Families
// that support multiple auth flows additionally carry an `auth_mode` field.
//
// Pattern reference for adding a new family:
// - Single-endpoint family with no extras: see `AnthropicModelProviderConfig`
// - Family with extras: see `OpenAIModelProviderConfig`
// - Family with computed-endpoint template: see `AzureModelProviderConfig`
// - Multi-region family with a required `endpoint` field: see `MoonshotModelProviderConfig`
//
// The `ModelProviders` container in `crates/omnesagent-config/src/model_providers.rs`
// holds a typed slot per family; the runtime impls in omnesagent-providers
// consume the typed configs directly.

// ── OpenAI ──

/// OpenAI canonical endpoint. Single variant — OpenAI publishes one base URL.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OpenAIEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for OpenAIEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.openai.com/v1",
        }
    }
}

/// OpenAI model model_provider config. The OpenAI-family extras (`wire_api`,
/// `requires_openai_auth`) live on the shared `ModelProviderConfig` base
/// because they're consumed by validation and runtime helpers that operate
/// on the base struct without family awareness; this wrapper is a thin
/// typed slot, no extra fields.
///
/// New OpenAI provider entries **persisted via `create_map_key` / `ensure`**
/// (quickstart, gateway/config UI, programmatic slot creation) default to
/// `wire_api = "responses"` because OpenAI has moved recent GPT models onto
/// `POST /v1/responses` as the primary wire. OpenAI-compatible families
/// (`custom`, `llamacpp`, branded vendors, …) keep the shared
/// `ModelProviderConfig` default of unset / chat-completions.
///
/// This default governs **persisted slot creation only**. Two paths keep the
/// legacy chat-completions wire for backward compatibility:
/// - Existing persisted configs that omit `wire_api` deserialize as `None`, and
///   the factory falls back to chat-completions.
/// - Implicit dispatch with no config entry at all — a bare
///   `model_provider = "openai"` reference or a dotted ref to a nonexistent
///   alias — is built from a chat-anchored fallback config (see
///   `openai_missing_entry_fallback_config` in `omnesagent-providers`), not this
///   `Default`, and stays on the chat wire so existing bare-ref installs don't
///   flip wire + tool-calling mode on upgrade.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.openai"]
pub struct OpenAIModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

impl Default for OpenAIModelProviderConfig {
    fn default() -> Self {
        Self {
            base: ModelProviderConfig {
                // OpenAI's current default wire for new provider slots.
                wire_api: Some(WireApi::Responses),
                ..ModelProviderConfig::default()
            },
        }
    }
}

// ── Azure OpenAI ──

/// Azure OpenAI endpoint template. Single variant; the URL is computed at
/// runtime by substituting `{resource}` and `{deployment}` from the typed
/// config fields.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum AzureEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for AzureEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            // Azure's URI is a template — substitution happens in the
            // AzureModelProvider runtime constructor against the typed
            // config's resource / deployment fields.
            Self::Default => "https://{resource}.openai.azure.com/openai/deployments/{deployment}",
        }
    }
}

/// Azure OpenAI model model_provider config. Carries the Azure-specific connection
/// fields (`resource`, `deployment`, `api_version`) — the URI template
/// substitutes `{resource}` and `{deployment}` at runtime. Operators can
/// still override the entire endpoint via `base.uri`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.azure"]
pub struct AzureModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
    /// Azure resource name (the `<resource>` part of `<resource>.openai.azure.com`).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "azure_openai_resource"
    )]
    pub resource: Option<String>,
    /// Azure deployment name: the deployment created in Azure AI Studio.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "azure_openai_deployment"
    )]
    pub deployment: Option<String>,
    /// Azure API version string (e.g. `2024-10-21`).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "azure_openai_api_version"
    )]
    pub api_version: Option<String>,
}

// ── Anthropic ──

/// Anthropic canonical endpoint.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum AnthropicEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for AnthropicEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.anthropic.com",
        }
    }
}

/// Anthropic model model_provider config. No family-specific extras yet — typed
/// slot reserved for future Anthropic-only knobs (cache_control, beta
/// headers) so they land cleanly without another schema rework.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.anthropic"]
pub struct AnthropicModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Moonshot (multi-region exemplar) ──

/// Moonshot endpoint variants. Operators pick the region that matches their
/// account; the runtime resolves the URI from the chosen variant unless
/// overridden by `base.uri`. Code variant is intl-only.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum MoonshotEndpoint {
    /// Mainland China endpoint.
    Cn,
    /// International endpoint.
    #[default]
    Intl,
    /// Code-specialist endpoint (intl).
    Code,
}

impl ModelEndpoint for MoonshotEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Cn => "https://api.moonshot.cn/v1",
            Self::Intl => "https://api.moonshot.ai/v1",
            // Kimi Code (Kimi For Coding) moved to api.kimi.com — the old api.moonshot.cn/coder/v1 returns 404.
            Self::Code => "https://api.kimi.com/coding/v1",
        }
    }
}

/// Moonshot model model_provider config. The `endpoint` field is required (no
/// implicit default) — operators must pick a region explicitly. Migration
/// fills it in from collapsed `moonshot-cn` / `moonshot-intl` outer keys.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.moonshot"]
pub struct MoonshotModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
    /// Required: pick `cn`, `intl`, or `code`. Defaults to `intl` when omitted
    /// to ease transition; operators on the China endpoint should set
    /// `endpoint = "cn"` explicitly.
    #[serde(default)]
    pub endpoint: MoonshotEndpoint,
}

impl FamilyEndpoint for MoonshotModelProviderConfig {
    fn endpoint_uri(&self) -> Option<&'static str> {
        Some(self.endpoint.uri())
    }
}

// ── Qwen (multi-region + auth_mode exemplar) ──

/// Qwen endpoint variants. Operators pick the region matching their account.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum QwenEndpoint {
    /// Mainland China (DashScope).
    Cn,
    /// International (alicloud international).
    #[default]
    Intl,
    /// United States (DashScope US).
    Us,
    /// Code-specialist endpoint.
    Code,
}

impl ModelEndpoint for QwenEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Cn => "https://dashscope.aliyuncs.com/compatible-mode/v1",
            Self::Intl => "https://dashscope-intl.aliyuncs.com/compatible-mode/v1",
            Self::Us => "https://dashscope-us.aliyuncs.com/compatible-mode/v1",
            Self::Code => {
                "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation"
            }
        }
    }
}

/// Qwen model model_provider config. Multi-region (`endpoint` required) and
/// supports both API key and OAuth flows (`auth_mode` chooses which).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.qwen"]
pub struct QwenModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
    #[serde(default)]
    pub endpoint: QwenEndpoint,
    /// Auth flow. Defaults to `api_key`; set to `o_auth` to use the vendor's
    /// OAuth-cache integration instead of the `api_key` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<AuthMode>,
    /// Long-lived Qwen OAuth refresh token. When set, the runtime
    /// exchanges it for a short-lived access token at provider
    /// construction time. Operators relying on the upstream `qwen login`
    /// tool (which writes `~/.qwen/oauth_creds.json`) leave this unset;
    /// the file-cache integration takes over.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[secret(category = "model_provider")]
    pub oauth_refresh_token: Option<String>,
    /// Override of Qwen's published OAuth client_id. Most operators
    /// should leave this unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth_client_id: Option<String>,
    /// Operator override of the resource URL the refreshed access token
    /// is paired with. When unset, the runtime falls back to the
    /// `endpoint`-derived URL (or the cached `resource_url` when reading
    /// from `~/.qwen/oauth_creds.json`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth_resource_url: Option<String>,
}

impl FamilyEndpoint for QwenModelProviderConfig {
    fn endpoint_uri(&self) -> Option<&'static str> {
        Some(self.endpoint.uri())
    }
}

// ── OpenRouter ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OpenRouterEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for OpenRouterEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://openrouter.ai/api/v1",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.openrouter"]
pub struct OpenRouterModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Ollama (local-default endpoint) ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OllamaEndpoint {
    #[default]
    LocalDefault,
}

impl ModelEndpoint for OllamaEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::LocalDefault => "http://localhost:11434",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.ollama"]
pub struct OllamaModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
    /// Override the Ollama `num_ctx` (context window, in tokens) sent on
    /// every `/api/chat` request. Defaults to the framework constant
    /// (`OLLAMA_DEFAULT_NUM_CTX`) when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<u32>,
    /// Override the Ollama `num_predict` (max output tokens) sent on every
    /// `/api/chat` request. Defaults to the framework constant
    /// (`OLLAMA_DEFAULT_NUM_PREDICT`) when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i32>,
    /// Force every Ollama `/api/chat` request to use this temperature,
    /// overriding the per-call value passed through
    /// `ModelProvider::chat_with_system(.., temperature)`. When unset
    /// (`None`, the default), the per-call temperature wins: full
    /// backward compatibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature_override: Option<f64>,
}

// ── Together ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum TogetherEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for TogetherEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.together.xyz/v1",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.together"]
pub struct TogetherModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Fireworks ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum FireworksEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for FireworksEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.fireworks.ai/inference/v1",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.fireworks"]
pub struct FireworksModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Groq ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum GroqEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for GroqEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.groq.com/openai/v1",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.groq"]
pub struct GroqModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Mistral ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum MistralEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for MistralEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.mistral.ai/v1",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.mistral"]
pub struct MistralModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Atomic Chat (local OpenAI-compatible runtime, e.g. Jan) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum AtomicChatEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for AtomicChatEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "http://127.0.0.1:1337/v1",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.atomic_chat"]
pub struct AtomicChatModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── DeepSeek ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum DeepseekEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for DeepseekEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.deepseek.com/v1",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.deepseek"]
pub struct DeepseekModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Cohere ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CohereEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for CohereEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.cohere.ai/compatibility/v1",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.cohere"]
pub struct CohereModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Perplexity ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum PerplexityEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for PerplexityEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.perplexity.ai",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.perplexity"]
pub struct PerplexityModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── xAI (Grok) ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum XaiEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for XaiEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.x.ai/v1",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.xai"]
pub struct XaiModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Cerebras ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CerebrasEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for CerebrasEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.cerebras.ai/v1",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.cerebras"]
pub struct CerebrasModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── SambaNova ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SambanovaEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for SambanovaEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.sambanova.ai/v1",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.sambanova"]
pub struct SambanovaModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Hyperbolic ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum HyperbolicEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for HyperbolicEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.hyperbolic.xyz/v1",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.hyperbolic"]
pub struct HyperbolicModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── DeepInfra ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum DeepinfraEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for DeepinfraEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.deepinfra.com/v1/openai",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.deepinfra"]
pub struct DeepinfraModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Hugging Face ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum HuggingfaceEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for HuggingfaceEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://router.huggingface.co/v1",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.huggingface"]
pub struct HuggingfaceModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── AI21 ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Ai21Endpoint {
    #[default]
    Default,
}
impl ModelEndpoint for Ai21Endpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.ai21.com/studio/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.ai21"]
pub struct Ai21ModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Reka ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum RekaEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for RekaEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.reka.ai/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.reka"]
pub struct RekaModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── BaseTen ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum BasetenEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for BasetenEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://inference.baseten.co/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.baseten"]
pub struct BasetenModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── NScale ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum NscaleEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for NscaleEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://inference.api.nscale.com/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.nscale"]
pub struct NscaleModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── AnyScale ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum AnyscaleEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for AnyscaleEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.endpoints.anyscale.com/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.anyscale"]
pub struct AnyscaleModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Nebius ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum NebiusEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for NebiusEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.tokenfactory.nebius.com/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.nebius"]
pub struct NebiusModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Friendli ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum FriendliEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for FriendliEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.friendli.ai/serverless/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.friendli"]
pub struct FriendliModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Stepfun ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum StepfunEndpoint {
    /// Mainland China endpoint.
    Cn,
    /// International endpoint.
    #[default]
    Intl,
}
impl ModelEndpoint for StepfunEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Cn => "https://api.stepfun.com/v1",
            Self::Intl => "https://api.stepfun.ai/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.stepfun"]
pub struct StepfunModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
    #[serde(default)]
    pub endpoint: StepfunEndpoint,
}

impl FamilyEndpoint for StepfunModelProviderConfig {
    fn endpoint_uri(&self) -> Option<&'static str> {
        Some(self.endpoint.uri())
    }
}

// ── AIHubMix ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum AihubmixEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for AihubmixEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://aihubmix.com/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.aihubmix"]
pub struct AihubmixModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── SiliconFlow ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SiliconflowEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for SiliconflowEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.siliconflow.com/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.siliconflow"]
pub struct SiliconflowModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Astrai ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum AstraiEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for AstraiEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://as-trai.com/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.astrai"]
pub struct AstraiModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Avian ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum AvianEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for AvianEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.avian.io/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.avian"]
pub struct AvianModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── DeepMyst ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum DeepmystEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for DeepmystEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.deepmyst.com/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.deepmyst"]
pub struct DeepmystModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Venice ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum VeniceEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for VeniceEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.venice.ai",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.venice"]
pub struct VeniceModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── NEAR AI Cloud ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum NearaiEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for NearaiEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://cloud-api.near.ai/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.nearai"]
pub struct NearaiModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Novita ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum NovitaEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for NovitaEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.novita.ai/openai",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.novita"]
pub struct NovitaModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── NVIDIA ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum NvidiaEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for NvidiaEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://integrate.api.nvidia.com/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.nvidia"]
pub struct NvidiaModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Telnyx ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum TelnyxEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for TelnyxEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.telnyx.com/v2",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.telnyx"]
pub struct TelnyxModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Vercel AI Gateway ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum VercelEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for VercelEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://ai-gateway.vercel.sh/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.vercel"]
pub struct VercelModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Cloudflare AI Gateway ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CloudflareEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for CloudflareEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://gateway.ai.cloudflare.com/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.cloudflare"]
pub struct CloudflareModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Atlas Cloud ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum AtlasCloudEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for AtlasCloudEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.atlascloud.ai/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.atlascloud"]
pub struct AtlasCloudModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── OVH ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OvhEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for OvhEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://oai.endpoints.kepler.ai.cloud.ovh.net/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.ovh"]
pub struct OvhModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── GitHub Copilot ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CopilotEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for CopilotEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.githubcopilot.com",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.copilot"]
pub struct CopilotModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── GLM (multi-region) ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum GlmEndpoint {
    Cn,
    #[default]
    Global,
}
impl ModelEndpoint for GlmEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Cn => "https://open.bigmodel.cn/api/paas/v4",
            Self::Global => "https://api.z.ai/api/paas/v4",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.glm"]
pub struct GlmModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
    #[serde(default)]
    pub endpoint: GlmEndpoint,
}

impl FamilyEndpoint for GlmModelProviderConfig {
    fn endpoint_uri(&self) -> Option<&'static str> {
        Some(self.endpoint.uri())
    }
}

// ── Minimax (multi-region + auth_mode) ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum MinimaxEndpoint {
    Cn,
    #[default]
    Intl,
}
impl ModelEndpoint for MinimaxEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Cn => "https://api.minimaxi.com/v1",
            Self::Intl => "https://api.minimax.io/v1",
        }
    }
}

impl MinimaxEndpoint {
    /// OAuth `/oauth/token` endpoint for this region. Used by
    /// `refresh_minimax_oauth_access_token` to mint short-lived access
    /// tokens from the operator-supplied `oauth_refresh_token`.
    pub fn oauth_token_endpoint(self) -> &'static str {
        match self {
            Self::Cn => "https://api.minimaxi.com/oauth/token",
            Self::Intl => "https://api.minimax.io/oauth/token",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.minimax"]
pub struct MinimaxModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
    #[serde(default)]
    pub endpoint: MinimaxEndpoint,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<AuthMode>,
    /// Long-lived OAuth refresh token issued by MiniMax. When set, the
    /// runtime exchanges it for a short-lived access token at provider
    /// construction time and uses that as the API credential. Operators
    /// who prefer dashboard-generated long-lived API keys can leave this
    /// unset and populate `api_key` directly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[secret(category = "model_provider")]
    pub oauth_refresh_token: Option<String>,
    /// Override of MiniMax's published OAuth client_id. Most operators
    /// should leave this unset; the runtime defaults to the
    /// vendor-published client_id (same one MiniMax's own portal uses).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth_client_id: Option<String>,
}

impl FamilyEndpoint for MinimaxModelProviderConfig {
    fn endpoint_uri(&self) -> Option<&'static str> {
        Some(self.endpoint.uri())
    }
}

// ── Z.AI (multi-region) ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ZaiEndpoint {
    Cn,
    #[default]
    Global,
}
impl ModelEndpoint for ZaiEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Cn => "https://open.bigmodel.cn/api/coding/paas/v4",
            Self::Global => "https://api.z.ai/api/coding/paas/v4",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.zai"]
pub struct ZaiModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
    #[serde(default)]
    pub endpoint: ZaiEndpoint,
}

impl FamilyEndpoint for ZaiModelProviderConfig {
    fn endpoint_uri(&self) -> Option<&'static str> {
        Some(self.endpoint.uri())
    }
}

// ── Doubao (Volcengine; single canonical endpoint) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum DoubaoEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for DoubaoEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://ark.cn-beijing.volces.com/api/v3",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.doubao"]
pub struct DoubaoModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Yi (Lingyiwanwu; single endpoint) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum YiEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for YiEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.lingyiwanwu.com/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.yi"]
pub struct YiModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Hunyuan (Tencent; single endpoint) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum HunyuanEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for HunyuanEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.hunyuan.cloud.tencent.com/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.hunyuan"]
pub struct HunyuanModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Qianfan (Baidu) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum QianfanEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for QianfanEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://qianfan.baidubce.com/v2",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.qianfan"]
pub struct QianfanModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Baichuan ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum BaichuanEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for BaichuanEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.baichuan-ai.com/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.baichuan"]
pub struct BaichuanModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Gemini (OAuth-capable) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum GeminiEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for GeminiEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://generativelanguage.googleapis.com/v1beta",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.gemini"]
pub struct GeminiModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
    /// Auth flow. Defaults to `api_key`; `o_auth` uses GeminiModelProvider's
    /// OAuth-cache integration instead of the `api_key` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<AuthMode>,
    /// Google OAuth app `client_id`, used when this alias drives OmnesAgent's
    /// own browser/device-code login flow (`omnesagent auth login
    /// --model-provider gemini --profile <alias>`). Operators relying on
    /// the upstream `gemini login` tool don't need this; that tool writes
    /// its own client_id / client_secret into `~/.gemini/oauth_creds.json`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[secret(category = "model_provider")]
    pub oauth_client_id: Option<String>,
    /// Google OAuth app `client_secret`. Set alongside `oauth_client_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[secret(category = "model_provider")]
    pub oauth_client_secret: Option<String>,
    /// Pin a specific GCP project ID for the OAuth `loadCodeAssist`
    /// discovery call. When unset, the discovery probes for an
    /// already-onboarded project on the credential's account. Replaces
    /// `GOOGLE_CLOUD_PROJECT` / `GOOGLE_CLOUD_PROJECT_ID` env vars.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth_project: Option<String>,
}

// ── Gemini CLI (subprocess wrapper) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum GeminiCliEndpoint {
    #[default]
    LocalSubprocess,
}
impl ModelEndpoint for GeminiCliEndpoint {
    fn uri(&self) -> &'static str {
        // Subprocess — no remote endpoint. Sentinel for trait conformity.
        "subprocess://gemini-cli"
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.gemini_cli"]
pub struct GeminiCliModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
    /// Path to the `gemini` CLI binary. Falls back to `gemini` (PATH lookup).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_path: Option<String>,
}

// ── Grok Build CLI (subprocess wrapper) ──

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.grok_cli"]
pub struct GrokCliModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
    /// Absolute path or bare executable name for the `grok` CLI (default: `grok`).
    /// Bare names resolve only from absolute PATH directories; empty and relative
    /// PATH entries are ignored before the subprocess working directory is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_path: Option<String>,
    /// Required absolute working directory for the `grok` subprocess and ACP
    /// session boundary. The directory must exist when the provider is built.
    /// Project-scoped Grok config is resolved relative to this path.
    pub working_directory: String,
    /// Extra environment variable names inherited by the `grok`
    /// subprocess. Values are resolved from the OmnesAgent process environment
    /// at spawn time. The default is empty so unrelated daemon secrets remain
    /// blocked. `XAI_API_KEY` is the sole supported provider-owned name and
    /// enables API-key authentication when explicitly listed and non-empty;
    /// other `XAI_*` and all `GROK_*` names are rejected.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[credential_class = "legacy_env_path"]
    pub env_passthrough: Vec<String>,
    /// Extra global Grok long flags inserted before `agent stdio`. Known
    /// options may put their value in the next token; other value-taking
    /// options use `--flag=value`. Positional and short arguments are rejected.
    /// OmnesAgent defaults to `--sandbox strict`, `--permission-mode dontAsk`,
    /// and an empty built-in tool set. Providing the corresponding flags here
    /// is an explicit per-alias opt-in to relax those defaults. ACP transport,
    /// prompt/model/session, cwd, debug-file, and update-policy flags are
    /// reserved.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<String>,
    /// Maximum cumulative stdout bytes accepted from `grok agent stdio` for
    /// one ACP request. When unset, OmnesAgent uses 4 MiB. Values must be
    /// between 1 MiB and 64 MiB; the provider rejects invalid values when it
    /// is constructed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_acp_stdout_bytes: Option<usize>,
}

// ── LMStudio (local default) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum LmstudioEndpoint {
    #[default]
    LocalDefault,
}
impl ModelEndpoint for LmstudioEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::LocalDefault => "http://localhost:1234/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.lmstudio"]
pub struct LmstudioModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── llama.cpp (local default) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum LlamacppEndpoint {
    #[default]
    LocalDefault,
}
impl ModelEndpoint for LlamacppEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::LocalDefault => "http://localhost:8080/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.llamacpp"]
pub struct LlamacppModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── SGLang (local default) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SglangEndpoint {
    #[default]
    LocalDefault,
}
impl ModelEndpoint for SglangEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::LocalDefault => "http://localhost:30000/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.sglang"]
pub struct SglangModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── vLLM (local default) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum VllmEndpoint {
    #[default]
    LocalDefault,
}
impl ModelEndpoint for VllmEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::LocalDefault => "http://localhost:8000/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.vllm"]
pub struct VllmModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Osaurus (local default) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OsaurusEndpoint {
    #[default]
    LocalDefault,
}
impl ModelEndpoint for OsaurusEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::LocalDefault => "http://localhost:1337/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.osaurus"]
pub struct OsaurusModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── LiteLLM (operator-self-hosted gateway) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum LitellmEndpoint {
    #[default]
    LocalDefault,
}
impl ModelEndpoint for LitellmEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::LocalDefault => "http://localhost:4000/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.litellm"]
pub struct LitellmModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Lepton ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum LeptonEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for LeptonEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://llama3-1-405b.lepton.run/api/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.lepton"]
pub struct LeptonModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Morph ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum MorphEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for MorphEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.morphllm.com/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.morph"]
pub struct MorphModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Manifest ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ManifestEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for ManifestEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://app.manifest.build/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.manifest"]
pub struct ManifestModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── GitHub Models ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum GithubModelsEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for GithubModelsEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://models.github.ai/inference",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.github_models"]
pub struct GithubModelsModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Upstage ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum UpstageEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for UpstageEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.upstage.ai/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.upstage"]
pub struct UpstageModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Featherless ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum FeatherlessEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for FeatherlessEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.featherless.ai/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.featherless"]
pub struct FeatherlessModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Arcee ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ArceeEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for ArceeEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            // Arcee publishes its OpenAI-compatible API at the `/api/v1` path
            // (not the conventional `/v1` root). Confirmed against Arcee docs.
            Self::Default => "https://api.arcee.ai/api/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.arcee"]
pub struct ArceeModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Lambda AI ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum LambdaAiEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for LambdaAiEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.lambda.ai/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.lambda_ai"]
pub struct LambdaAiModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Inception ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum InceptionEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for InceptionEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.inceptionlabs.ai/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.inception"]
pub struct InceptionModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Synthetic ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SyntheticEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for SyntheticEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.synthetic.new/openai/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.synthetic"]
pub struct SyntheticModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── OpenCode (Zen) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OpencodeEndpoint {
    #[default]
    Default,
}
impl ModelEndpoint for OpencodeEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://opencode.ai/zen/v1",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.opencode"]
pub struct OpencodeModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── KiloCli (subprocess wrapper) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum KiloCliEndpoint {
    #[default]
    LocalSubprocess,
}
impl ModelEndpoint for KiloCliEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::LocalSubprocess => "subprocess://kilocli",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.kilocli"]
pub struct KiloCliModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
    /// Path to the `kilo` CLI binary. Falls back to `kilo` (PATH lookup).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_path: Option<String>,
}

// ── Kilo (AI Gateway — OpenAI-compatible) ──

/// Kilo AI Gateway endpoint. Single canonical endpoint at kilo.ai.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum KiloEndpoint {
    #[default]
    Gateway,
}
impl ModelEndpoint for KiloEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Gateway => "https://api.kilo.ai/api/gateway",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.kilo"]
pub struct KiloModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
    /// Kilo endpoint variant. Defaults to the canonical Kilo AI Gateway.
    #[serde(default, skip_serializing_if = "KiloEndpoint::is_default")]
    pub endpoint: KiloEndpoint,
}

impl KiloEndpoint {
    fn is_default(&self) -> bool {
        matches!(self, Self::Gateway)
    }
}

impl FamilyEndpoint for KiloModelProviderConfig {
    fn endpoint_uri(&self) -> Option<&'static str> {
        Some(self.endpoint.uri())
    }
}

// ── ZeroRouter (LLM gateway — OpenAI-compatible; hosted or self-hosted) ──

/// ZeroRouter endpoint. The single variant points at the public hosted
/// deployment, `https://zerorouter.ai` (currently in beta) — the endpoint a
/// user who names this provider without further configuration expects, and
/// the one that works out of the box: its `/v1/models` listing is public, so
/// discovery succeeds before any key is configured. ZeroRouter is also
/// self-hostable (AGPL); operators running their own router — locally
/// (`ZEROROUTER_BIND=0.0.0.0:8080`, so `http://localhost:8080/v1`) or
/// anywhere else — set `base.uri` to reach it. A localhost default was
/// considered and rejected: it made the zero-config path a connection
/// refusal, or worse, a silent partial catalog from a stray dev instance.
/// [`ZEROROUTER_DEFAULT_URL`] is the canonical default consumed by both
/// schema and provider construction.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ZerorouterEndpoint {
    #[default]
    Default,
}

/// Default API base: the hosted ZeroRouter deployment.
pub const ZEROROUTER_DEFAULT_URL: &str = "https://zerorouter.ai/v1";

impl ModelEndpoint for ZerorouterEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => ZEROROUTER_DEFAULT_URL,
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.zerorouter"]
pub struct ZerorouterModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Custom (user-supplied URL, no canonical default) ──

/// Custom catch-all for operator-defined endpoints. The endpoint variant has
/// no canonical URL — operators must always set `base.uri`. The trait return
/// is a sentinel string; the runtime constructor must verify `base.uri` is
/// set for `custom` entries and fail with a clear error if not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CustomEndpoint {
    #[default]
    OperatorSupplied,
}
impl ModelEndpoint for CustomEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::OperatorSupplied => "operator-supplied:set-cfg-uri",
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.custom"]
pub struct CustomModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
}

// ── Bedrock (computed-endpoint exemplar, AWS region template) ──

/// AWS Bedrock endpoint template. Single variant; the URL is computed at
/// runtime by substituting `{region}` from the typed config field.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum BedrockEndpoint {
    #[default]
    Default,
}

impl ModelEndpoint for BedrockEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            // Bedrock URI is a template — substitution happens in the
            // BedrockModelProvider runtime constructor against cfg.region.
            Self::Default => "https://bedrock-runtime.{region}.amazonaws.com",
        }
    }
}

/// AWS Bedrock model model_provider config. Carries the AWS region (the URI
/// template substitutes `{region}` from this field). Bedrock auth is
/// SigV4 — credentials come from the standard AWS credential chain
/// (env vars, instance metadata, profile), not from `api_key`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.models.bedrock"]
pub struct BedrockModelProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: ModelProviderConfig,
    /// AWS region for the Bedrock endpoint (e.g. `us-east-1`, `eu-west-1`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

// ── FamilyEndpoint default impls (single-endpoint families) ─────
//
// Multi-endpoint families (Moonshot, Qwen, Glm, Minimax, Zai, Stepfun) define
// their own `impl FamilyEndpoint` next to the struct. Every other family
// gets the `None` default via this list. The list is exhaustive: a new
// family with no impl here AND no manual impl elsewhere will fail to
// compile against `ModelProviders::resolved_endpoint_uri`, which expands
// `endpoint_uri()` per slot through `for_each_model_provider_slot!`.

macro_rules! impl_default_family_endpoint {
    ($($t:ty),+ $(,)?) => {
        $( impl FamilyEndpoint for $t {} )+
    };
}

impl_default_family_endpoint! {
    OpenAIModelProviderConfig,
    AzureModelProviderConfig,
    AnthropicModelProviderConfig,
    AtomicChatModelProviderConfig,
    OpenRouterModelProviderConfig,
    OllamaModelProviderConfig,
    TogetherModelProviderConfig,
    FireworksModelProviderConfig,
    GroqModelProviderConfig,
    MistralModelProviderConfig,
    DeepseekModelProviderConfig,
    CohereModelProviderConfig,
    PerplexityModelProviderConfig,
    XaiModelProviderConfig,
    CerebrasModelProviderConfig,
    SambanovaModelProviderConfig,
    HyperbolicModelProviderConfig,
    DeepinfraModelProviderConfig,
    HuggingfaceModelProviderConfig,
    Ai21ModelProviderConfig,
    RekaModelProviderConfig,
    BasetenModelProviderConfig,
    NscaleModelProviderConfig,
    AnyscaleModelProviderConfig,
    NebiusModelProviderConfig,
    FriendliModelProviderConfig,
    AihubmixModelProviderConfig,
    SiliconflowModelProviderConfig,
    AstraiModelProviderConfig,
    AvianModelProviderConfig,
    DeepmystModelProviderConfig,
    VeniceModelProviderConfig,
    NearaiModelProviderConfig,
    NovitaModelProviderConfig,
    ZerorouterModelProviderConfig,
    NvidiaModelProviderConfig,
    TelnyxModelProviderConfig,
    VercelModelProviderConfig,
    CloudflareModelProviderConfig,
    AtlasCloudModelProviderConfig,
    OvhModelProviderConfig,
    CopilotModelProviderConfig,
    DoubaoModelProviderConfig,
    YiModelProviderConfig,
    HunyuanModelProviderConfig,
    QianfanModelProviderConfig,
    BaichuanModelProviderConfig,
    GeminiModelProviderConfig,
    GeminiCliModelProviderConfig,
    GrokCliModelProviderConfig,
    LmstudioModelProviderConfig,
    LlamacppModelProviderConfig,
    SglangModelProviderConfig,
    VllmModelProviderConfig,
    OsaurusModelProviderConfig,
    LitellmModelProviderConfig,
    LeptonModelProviderConfig,
    ManifestModelProviderConfig,
    MorphModelProviderConfig,
    GithubModelsModelProviderConfig,
    UpstageModelProviderConfig,
    FeatherlessModelProviderConfig,
    ArceeModelProviderConfig,
    LambdaAiModelProviderConfig,
    InceptionModelProviderConfig,
    SyntheticModelProviderConfig,
    OpencodeModelProviderConfig,
    KiloCliModelProviderConfig,
    CustomModelProviderConfig,
    BedrockModelProviderConfig,
}
