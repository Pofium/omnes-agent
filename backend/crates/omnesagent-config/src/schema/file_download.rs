//! Standalone file download tool config (the `[file_download]` section):
//! streaming GET of an upstream document into the workspace filesystem with a
//! size ceiling and a private-host carveout.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::FileDownloadConfig` is unchanged.

use super::*;

// ── File Download ───────────────────────────────────────────────

/// Standalone file download tool configuration (`[file_download]`).
///
/// When `url` is set to a non-empty value, registers a `file_download` tool
/// that GETs a file from the configured endpoint and writes it to the agent's
/// workspace filesystem. The LLM supplies only a document identifier and a
/// workspace-relative destination path; the endpoint URL comes solely from this
/// config and is never model-controlled. Response bytes are streamed to disk
/// and never loaded into model context.
///
/// When `url` is `None` or empty, the tool is not registered.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "file-download"]
pub struct FileDownloadConfig {
    /// Download endpoint URL. Tool is disabled when this is `None` or empty.
    /// The file to fetch is selected by the `document_id` query parameter.
    #[serde(default)]
    pub url: Option<String>,

    /// Maximum download size in bytes. Enforced while streaming: the transfer
    /// is aborted and the partial file removed once this ceiling is exceeded,
    /// so an oversized or unbounded body never fully buffers in memory or lands
    /// on disk. Default: 25 MiB.
    #[serde(default = "default_file_download_max_size_bytes")]
    pub max_file_size_bytes: u64,

    /// Request timeout in seconds. Default: 120.
    #[serde(default = "default_file_download_timeout_secs")]
    pub timeout_secs: u64,

    /// Private, loopback, or link-local endpoint hosts that file_download may
    /// contact. Cloud metadata and credential-delivery addresses remain blocked
    /// even when their host appears here. Use only for operator-controlled
    /// internal document services.
    #[serde(default)]
    pub allowed_private_hosts: Vec<String>,

    /// Static HTTP headers attached to every download request — typically an
    /// `Authorization: Bearer …` token for the upstream endpoint. Same shape as
    /// `[mcp.servers.*.headers]`.
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub headers: HashMap<String, String>,
}

fn default_file_download_max_size_bytes() -> u64 {
    25 * 1024 * 1024
}

fn default_file_download_timeout_secs() -> u64 {
    120
}

impl Default for FileDownloadConfig {
    fn default() -> Self {
        Self {
            url: None,
            max_file_size_bytes: default_file_download_max_size_bytes(),
            timeout_secs: default_file_download_timeout_secs(),
            headers: HashMap::new(),
            allowed_private_hosts: Vec::new(),
        }
    }
}
