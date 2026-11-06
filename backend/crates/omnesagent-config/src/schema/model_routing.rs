//! Model routing config: the `[[model_routes]]` hint-to-provider mapping.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::ModelRouteConfig` path is unchanged.

use super::*;

// ── Model routing ────────────────────────────────────────────────

/// Route a task hint to a specific model_provider + model.
///
/// ```toml
/// [[model_routes]]
/// hint = "reasoning"
/// model_provider = "openrouter.default"
/// model = "anthropic/claude-opus-4-20250514"
///
/// [[model_routes]]
/// hint = "fast"
/// model_provider = "groq.low-latency"
/// model = "llama-3.3-70b-versatile"
/// ```
///
/// Usage: pass `hint:reasoning` as the model parameter to route the request.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "model_routes"]
pub struct ModelRouteConfig {
    /// Task hint name (e.g. "reasoning", "fast", "code", "summarize")
    /// `#[serde(default)]` lets the `create_map_key` macro default-construct
    /// from `{}` before the hint gets injected via `set_prop`. Empty strings
    /// are rejected by `Config::validate()`.
    #[serde(default)]
    pub hint: String,
    /// Dotted provider profile ref to route to (must resolve to `providers.models.<type>.<alias>`)
    /// `#[serde(default)]` is required for `Default` + `create_map_key` construction.
    /// Empty strings are rejected by `Config::validate()`.
    #[serde(default)]
    pub model_provider: String,
    /// Provider-local model identifier to use with that provider profile
    /// `#[serde(default)]` is required for `Default` + `create_map_key` construction.
    /// Empty strings are rejected by `Config::validate()`.
    #[serde(default)]
    pub model: String,
    /// Optional API key override for this route's model provider
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub api_key: Option<String>,
}
