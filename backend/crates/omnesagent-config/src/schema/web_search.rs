//! Web search tool config (the `[web_search]` section): provider selection,
//! per-provider API keys, result limits, and request timeout.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::WebSearchConfig` is unchanged.

use super::*;

// ── Web search ───────────────────────────────────────────────────

/// Web search tool configuration (`[web_search]` section).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "web_search"]
pub struct WebSearchConfig {
    /// Enable `web_search_tool` for web searches
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Search provider: "duckduckgo" (free), "brave" (requires API key), "tavily" (requires API key), "searxng" (self-hosted), "jina" (requires API key), or "bocha" (Bocha AI, requires API key — Chinese-friendly, <https://open.bochaai.com>)
    #[serde(default = "default_web_search_provider")]
    pub search_provider: String,
    /// Brave Search API key (required if search_provider is "brave")
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub brave_api_key: Option<String>,
    /// Tavily Search API key (required if search_provider is "tavily")
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub tavily_api_key: Option<String>,
    /// Jina AI API key (required if search_provider is "jina")
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub jina_api_key: Option<String>,
    /// Bocha AI Web Search API key (required if search_provider is `"bocha"`). Obtain at <https://open.bochaai.com>.
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub bocha_api_key: Option<String>,
    /// SearXNG instance URL (required if search_provider is `"searxng"`), e.g. `"https://searx.example.com"`.
    #[serde(default)]
    pub searxng_instance_url: Option<String>,
    /// Maximum results per search (1-10)
    #[serde(default = "default_web_search_max_results")]
    pub max_results: usize,
    /// Request timeout in seconds
    #[serde(default = "default_web_search_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_web_search_provider() -> String {
    "duckduckgo".into()
}

fn default_web_search_max_results() -> usize {
    5
}

fn default_web_search_timeout_secs() -> u64 {
    15
}

impl Default for WebSearchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            search_provider: default_web_search_provider(),
            brave_api_key: None,
            tavily_api_key: None,
            jina_api_key: None,
            bocha_api_key: None,
            searxng_instance_url: None,
            max_results: default_web_search_max_results(),
            timeout_secs: default_web_search_timeout_secs(),
        }
    }
}
