//! HTTP request tool configuration (the `[http_request]` section).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::HttpRequestConfig` is unchanged.

use super::*;

// ── HTTP request tool ───────────────────────────────────────────

/// HTTP request tool configuration (`[http_request]` section).
///
/// Domain filtering: `allowed_domains` controls which hosts are reachable (use `["*"]`
/// for all public hosts, which is the default). If `allowed_domains` is empty, all
/// requests are rejected. Requests use direct transport so locally validated DNS
/// answers remain pinned: an enabled `environment` proxy scope or runtime proxy
/// that applies to `tool.http_request` is rejected. A process environment proxy
/// outside that managed scope is warned and ignored; connection failures name
/// the ignored variable.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "http_request"]
pub struct HttpRequestConfig {
    /// Enable `http_request` tool for API interactions
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Allowed domains for HTTP requests (exact or subdomain match)
    #[serde(default = "default_allowed_domains_star")]
    pub allowed_domains: Vec<String>,
    /// Maximum response size in bytes (default: 1MB, 0 = unlimited)
    #[serde(default = "default_http_max_response_size")]
    pub max_response_size: usize,
    /// Request timeout in seconds (default: 30)
    #[serde(default = "default_http_timeout_secs")]
    pub timeout_secs: u64,
    /// Allow requests to private/LAN hosts (RFC 1918, loopback, `.local`).
    /// Default: false (deny private hosts for SSRF protection). Regardless of
    /// this setting, all of `169.254.0.0/16`, Azure `168.63.129.16`, Alibaba
    /// `100.100.100.200`, AWS `fd00:ec2::/64`, GCP `fd20:ce::254`, and their
    /// recognized IPv4-embedded forms remain blocked.
    #[serde(default)]
    pub allow_private_hosts: bool,
    /// Private/internal hosts explicitly allowed to relax the public-address check.
    /// Exact and subdomain matches are supported; `*` permits private/local hosts
    /// except the unconditional metadata/platform exclusions: `169.254.0.0/16`,
    /// `168.63.129.16`, `100.100.100.200`, `fd00:ec2::/64`, `fd20:ce::254`, and
    /// their recognized IPv4-embedded forms.
    #[serde(default)]
    pub allowed_private_hosts: Vec<String>,
    /// Named authorization secrets for `auth_secret` requests.
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub secrets: HashMap<String, String>,
}

impl Default for HttpRequestConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_domains: vec!["*".into()],
            max_response_size: default_http_max_response_size(),
            timeout_secs: default_http_timeout_secs(),
            allow_private_hosts: false,
            allowed_private_hosts: vec![],
            secrets: HashMap::new(),
        }
    }
}

fn default_http_max_response_size() -> usize {
    1_000_000 // 1MB
}

fn default_http_timeout_secs() -> u64 {
    30
}

fn default_allowed_domains_star() -> Vec<String> {
    vec!["*".into()]
}
