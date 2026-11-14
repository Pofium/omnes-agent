//! Embedding routing config: the `[[embedding_routes]]` hint-to-provider mapping.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::EmbeddingRouteConfig` path is unchanged.

use super::*;

// ── Embedding routing ───────────────────────────────────────────

/// Route an embedding hint to a specific model_provider + model.
///
/// ```toml
/// [[embedding_routes]]
/// hint = "semantic"
/// model_provider = "openai.embeddings"
/// model = "text-embedding-3-small"
/// dimensions = 1536
///
/// [memory]
/// embedding_model = "hint:semantic"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "embedding_routes"]
pub struct EmbeddingRouteConfig {
    /// Route hint name (e.g. "semantic", "archive", "faq")
    /// `#[serde(default)]` lets the `create_map_key` macro default-construct
    /// from `{}` before the hint gets injected via `set_prop`. Empty strings
    /// are rejected by `Config::validate()`.
    #[serde(default)]
    pub hint: String,
    /// Dotted embedding-capable provider profile ref
    /// `#[serde(default)]` is required for `Default` + `create_map_key` construction.
    /// Empty strings are rejected by `Config::validate()`.
    #[serde(default)]
    pub model_provider: String,
    /// Provider-local embedding model identifier to use with that provider profile
    /// `#[serde(default)]` is required for `Default` + `create_map_key` construction.
    /// Empty strings are rejected by `Config::validate()`.
    #[serde(default)]
    pub model: String,
    /// Optional embedding dimension override for this route
    #[serde(default)]
    pub dimensions: Option<usize>,
    /// Optional API key override for this route's model_provider
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub api_key: Option<String>,
}
