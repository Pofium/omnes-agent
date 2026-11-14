//! Standalone file upload tool config (the `[file_upload]` section):
//! multipart/form-data POST/PUT of a local file with size and timeout
//! bounds.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::FileUploadConfig` is unchanged.

use super::*;

// ── File Upload ─────────────────────────────────────────────────

/// Standalone file upload tool configuration (`[file_upload]`).
///
/// When `url` is set to a non-empty value, registers a `file_upload` tool that
/// POSTs files from the agent's local filesystem to the configured endpoint
/// using `multipart/form-data`. The LLM provides only a file path; the host
/// reads the bytes and uploads them without ever including file content in
/// the model context.
///
/// When `url` is `None` or empty, the tool is not registered.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "file_upload"]
pub struct FileUploadConfig {
    /// Upload endpoint URL. Tool is disabled when this is `None` or empty.
    #[serde(default)]
    pub url: Option<String>,

    /// HTTP method. Only `POST` (default) and `PUT` are accepted.
    #[serde(default = "default_file_upload_method")]
    pub method: String,

    /// Multipart form-field name for the file part. Default: `file`.
    #[serde(default = "default_file_upload_field_name")]
    pub field_name: String,

    /// Maximum file size in bytes. Larger files are rejected before any
    /// bytes hit the network. Default: 25 MiB.
    #[serde(default = "default_file_upload_max_size_bytes")]
    pub max_file_size_bytes: u64,

    /// Request timeout in seconds. Default: 60.
    #[serde(default = "default_file_upload_timeout_secs")]
    pub timeout_secs: u64,

    /// Static HTTP headers attached to every upload request. Same shape as
    /// `[mcp.servers.*.headers]`.
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub headers: HashMap<String, String>,
}

fn default_file_upload_method() -> String {
    "POST".into()
}

fn default_file_upload_field_name() -> String {
    "file".into()
}

fn default_file_upload_max_size_bytes() -> u64 {
    25 * 1024 * 1024
}

fn default_file_upload_timeout_secs() -> u64 {
    60
}

impl Default for FileUploadConfig {
    fn default() -> Self {
        Self {
            url: None,
            method: default_file_upload_method(),
            field_name: default_file_upload_field_name(),
            max_file_size_bytes: default_file_upload_max_size_bytes(),
            timeout_secs: default_file_upload_timeout_secs(),
            headers: HashMap::new(),
        }
    }
}
