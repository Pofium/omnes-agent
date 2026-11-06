//! Text browser tool configuration (the `[text_browser]` section).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::TextBrowserConfig` is unchanged.

use super::*;

// ── Text browser ─────────────────────────────────────────────────

/// Text browser tool configuration (`[text_browser]` section).
///
/// Uses text-based browsers (lynx, links, w3m) to render web pages as plain
/// text. Designed for headless/SSH environments without graphical browsers.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "text_browser"]
pub struct TextBrowserConfig {
    /// Enable `text_browser` tool
    #[serde(default)]
    pub enabled: bool,
    /// Preferred text browser ("lynx", "links", or "w3m"). If unset, auto-detects.
    #[serde(default)]
    pub preferred_browser: Option<String>,
    /// Request timeout in seconds (default: 30)
    #[serde(default = "default_text_browser_timeout_secs")]
    pub timeout_secs: u64,
    /// Private/internal hosts allowed to relax the public-address SSRF check.
    /// Exact and subdomain matches are supported; `["*"]` permits private/local
    /// hosts (RFC 1918, loopback, `.local`). Default: empty (deny). All of
    /// `169.254.0.0/16`, Azure `168.63.129.16`, Alibaba `100.100.100.200`, AWS
    /// `fd00:ec2::/64`, GCP `fd20:ce::254`, and their recognized IPv4-embedded
    /// forms remain blocked after resolution regardless of this opt-in.
    /// The external browser re-resolves DNS and follows redirects without
    /// revalidation, so these checks are not a DNS-rebinding or redirect boundary.
    /// Local preflight resolution still applies under this opt-in, which may reject
    /// names available only through a browser or proxy. Prefer `web_fetch` when
    /// URLs or DNS are attacker-controlled.
    #[serde(default)]
    pub allowed_private_hosts: Vec<String>,
}

fn default_text_browser_timeout_secs() -> u64 {
    30
}

impl Default for TextBrowserConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            preferred_browser: None,
            timeout_secs: default_text_browser_timeout_secs(),
            allowed_private_hosts: vec![],
        }
    }
}
