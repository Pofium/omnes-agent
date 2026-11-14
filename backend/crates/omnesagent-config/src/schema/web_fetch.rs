//! Web fetch tool configuration (`[web_fetch]` and the nested
//! `[web_fetch.firecrawl]` fallback).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::{WebFetchConfig, FirecrawlConfig, FirecrawlMode}` is
//! unchanged.

use super::*;

// ── Web fetch ────────────────────────────────────────────────────

/// Web fetch tool configuration (`[web_fetch]` section).
///
/// Fetches web pages and converts HTML to plain text for LLM consumption.
/// Domain filtering: `allowed_domains` controls which hosts are reachable (use `["*"]`
/// for all public hosts). `blocked_domains` takes priority over `allowed_domains`.
/// If `allowed_domains` is empty, all requests are rejected (deny-by-default).
/// Same-host redirects are followed; cross-host redirects are rejected so the
/// validated DNS answers remain pinned to the request transport.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "web_fetch"]
pub struct WebFetchConfig {
    /// Enable `web_fetch` tool for fetching web page content
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Allowed domains for web fetch (exact or subdomain match; `["*"]` = all public hosts)
    #[serde(default = "default_web_fetch_allowed_domains")]
    pub allowed_domains: Vec<String>,
    /// Blocked domains (exact or subdomain match; always takes priority over allowed_domains)
    #[serde(default)]
    pub blocked_domains: Vec<String>,
    /// Private/internal hosts allowed to relax the public-address SSRF check
    /// (e.g. `["192.168.1.10", "internal.local"]`). Exact and subdomain matches
    /// are supported. Every target still resolves locally, and known cloud
    /// metadata/platform addresses remain blocked. A *literal* private/local host
    /// (IP literal, localhost, .local) listed here also bypasses `allowed_domains`.
    /// The `*` wildcard permits any literal private/local host and lets a name
    /// already in `allowed_domains` resolve to a private IP, but it does NOT widen
    /// `allowed_domains` — a non-private host (matched explicitly or via `*`) still
    /// needs `allowed_domains`, so `*` cannot reach an arbitrary public host.
    /// The standard `web_fetch` request requires direct transport for DNS pinning:
    /// environment proxy discovery is disabled, and an enabled `environment` scope
    /// or runtime proxy applying to the tool is rejected. A process environment proxy
    /// outside that managed scope is warned and ignored for the standard request.
    /// The optional Firecrawl API fallback uses normal environment proxy discovery
    /// because that separate API request has no locally validated DNS pin. Use
    /// `proxy.scope = "services"` without `tool.*` to proxy other traffic.
    #[serde(default)]
    pub allowed_private_hosts: Vec<String>,
    /// Maximum response size in bytes (default: 500KB, plain text is much smaller than raw HTML)
    #[serde(default = "default_web_fetch_max_response_size")]
    pub max_response_size: usize,
    /// Request timeout in seconds (default: 30)
    #[serde(default = "default_web_fetch_timeout_secs")]
    pub timeout_secs: u64,
    /// Firecrawl fallback configuration (`[web_fetch.firecrawl]`)
    #[serde(default)]
    #[nested]
    pub firecrawl: FirecrawlConfig,
}

/// Firecrawl fallback mode: scrape a single page or crawl linked pages.
#[derive(
    Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum FirecrawlMode {
    #[default]
    Scrape,
    /// Reserved for future multi-page crawl support. Accepted in config
    /// deserialization to avoid breaking existing files, but not yet
    /// implemented — `fetch_via_firecrawl` always uses the `/scrape` endpoint.
    Crawl,
}

/// Firecrawl fallback configuration for JS-heavy and bot-blocked sites.
///
/// When enabled, if the standard web fetch fails (HTTP error, empty body, or
/// body shorter than 100 characters suggesting a JS-only page), the tool
/// falls back to the Firecrawl API for stealth content extraction.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "web_fetch.firecrawl"]
pub struct FirecrawlConfig {
    /// Enable Firecrawl fallback
    #[serde(default)]
    pub enabled: bool,
    /// Environment variable name for the Firecrawl API key
    #[serde(default = "default_firecrawl_api_key_env")]
    #[credential_class = "legacy_env_path"]
    pub api_key_env: String,
    /// Firecrawl API base URL
    #[serde(default = "default_firecrawl_api_url")]
    pub api_url: String,
    /// Firecrawl extraction mode
    #[serde(default)]
    pub mode: FirecrawlMode,
}

fn default_firecrawl_api_key_env() -> String {
    "FIRECRAWL_API_KEY".into()
}

fn default_firecrawl_api_url() -> String {
    "https://api.firecrawl.dev/v1".into()
}

impl Default for FirecrawlConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key_env: default_firecrawl_api_key_env(),
            api_url: default_firecrawl_api_url(),
            mode: FirecrawlMode::default(),
        }
    }
}

fn default_web_fetch_max_response_size() -> usize {
    500_000 // 500KB
}

fn default_web_fetch_timeout_secs() -> u64 {
    30
}

fn default_web_fetch_allowed_domains() -> Vec<String> {
    vec!["*".into()]
}

impl Default for WebFetchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_domains: vec!["*".into()],
            blocked_domains: vec![],
            allowed_private_hosts: vec![],
            max_response_size: default_web_fetch_max_response_size(),
            timeout_secs: default_web_fetch_timeout_secs(),
            firecrawl: FirecrawlConfig::default(),
        }
    }
}
