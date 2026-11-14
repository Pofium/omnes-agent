//! Standalone multi-file bundle upload tool config (the
//! `[file_upload_bundle]` section): one multipart/form-data request carrying
//! several files, with per-file, cumulative, and count ceilings.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::FileUploadBundleConfig` is unchanged.

use super::*;

// ── File Upload Bundle ──────────────────────────────────────────

/// Standalone multi-file bundle upload tool configuration
/// (`[file_upload_bundle]`).
///
/// When `url` is set to a non-empty value, registers a `file_upload_bundle`
/// tool that POSTs N files from the agent's local filesystem to the
/// configured endpoint as a single `multipart/form-data` request. The LLM
/// provides only file paths; the host reads the bytes.
///
/// When `url` is `None` or empty, the tool is not registered.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "file-upload-bundle"]
pub struct FileUploadBundleConfig {
    /// Upload endpoint URL. Tool is disabled when this is `None` or empty.
    #[serde(default)]
    pub url: Option<String>,

    /// HTTP method. Only `POST` (default) and `PUT` are accepted.
    #[serde(default = "default_file_upload_bundle_method")]
    pub method: String,

    /// Multipart form-field name reused across every file part. Default: `file`.
    #[serde(default = "default_file_upload_bundle_field_name")]
    pub field_name: String,

    /// Maximum per-file size in bytes. Default: 10 MiB.
    #[serde(default = "default_file_upload_bundle_max_file_size_bytes")]
    pub max_file_size_bytes: u64,

    /// Maximum cumulative size across every file in one call. Default: 32 MiB.
    #[serde(default = "default_file_upload_bundle_max_total_size_bytes")]
    pub max_total_size_bytes: u64,

    /// Maximum number of files per call. Default: 16.
    #[serde(default = "default_file_upload_bundle_max_files")]
    pub max_files: u32,

    /// Request timeout in seconds. Default: 120.
    #[serde(default = "default_file_upload_bundle_timeout_secs")]
    pub timeout_secs: u64,

    /// Maximum response body bytes to read from the upload endpoint.
    /// Prevents unbounded memory use from a malicious or verbose receiver.
    /// Default: 4096 (4 KiB).
    #[serde(default = "default_file_upload_bundle_max_response_body_bytes")]
    pub max_response_body_bytes: usize,

    /// Static HTTP headers attached to every upload request.
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub headers: HashMap<String, String>,
}

fn default_file_upload_bundle_method() -> String {
    "POST".into()
}

fn default_file_upload_bundle_field_name() -> String {
    "file".into()
}

fn default_file_upload_bundle_max_file_size_bytes() -> u64 {
    10 * 1024 * 1024
}

fn default_file_upload_bundle_max_total_size_bytes() -> u64 {
    32 * 1024 * 1024
}

fn default_file_upload_bundle_max_files() -> u32 {
    16
}

fn default_file_upload_bundle_timeout_secs() -> u64 {
    120
}

fn default_file_upload_bundle_max_response_body_bytes() -> usize {
    4 * 1024
}

impl Default for FileUploadBundleConfig {
    fn default() -> Self {
        Self {
            url: None,
            method: default_file_upload_bundle_method(),
            field_name: default_file_upload_bundle_field_name(),
            max_file_size_bytes: default_file_upload_bundle_max_file_size_bytes(),
            max_total_size_bytes: default_file_upload_bundle_max_total_size_bytes(),
            max_files: default_file_upload_bundle_max_files(),
            timeout_secs: default_file_upload_bundle_timeout_secs(),
            max_response_body_bytes: default_file_upload_bundle_max_response_body_bytes(),
            headers: HashMap::new(),
        }
    }
}
