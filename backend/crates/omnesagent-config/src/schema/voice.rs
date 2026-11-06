//! Voice input config (the `[voice]` section): Handy offline dictation
//! integration settings.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::{VoiceConfig, HandyConfig}` are unchanged.

use super::*;

fn default_handy_install_policy() -> String {
    "ask".to_string()
}

fn default_handy_launch_on_agent_start() -> bool {
    true
}

fn default_handy_start_hidden() -> bool {
    true
}

fn default_handy_download_host_allowlist() -> Vec<String> {
    vec![
        "github.com".to_string(),
        "objects.githubusercontent.com".to_string(),
        "release-assets.githubusercontent.com".to_string(),
    ]
}

fn default_handy_max_download_bytes() -> u64 {
    786_432_000
}

fn default_handy_transcribe_timeout_secs() -> u64 {
    120
}

/// Voice input and offline dictation configuration (`[voice]`).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "voice"]
pub struct VoiceConfig {
    /// Handy offline dictation integration (`[voice.handy]`).
    #[serde(default)]
    #[nested]
    pub handy: HandyConfig,
}

/// Handy offline dictation integration settings (`[voice.handy]`).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "voice.handy"]
pub struct HandyConfig {
    /// Enable voice dictation via Handy.
    #[serde(default)]
    pub enabled: bool,
    /// Install policy: "ask", "auto", or "manual".
    #[serde(default = "default_handy_install_policy")]
    pub install_policy: String,
    /// Pin a specific Handy version (e.g. "0.9.6"). Empty string means latest release.
    #[serde(default)]
    pub pin_version: String,
    /// Allow downgrading Handy version during install.
    #[serde(default)]
    pub allow_downgrade: bool,
    /// Launch Handy when agent gateway starts (if installed).
    #[serde(default = "default_handy_launch_on_agent_start")]
    pub launch_on_agent_start: bool,
    /// Configure Handy to start with Windows.
    #[serde(default)]
    pub auto_start_with_os: bool,
    /// Launch Handy with `--start-hidden`.
    #[serde(default = "default_handy_start_hidden")]
    pub start_hidden: bool,
    /// Custom path to Handy.exe (for portable or non-standard installations).
    #[serde(default)]
    pub custom_exe_path: Option<String>,
    /// Allowlist of hostnames for downloading Handy.
    #[serde(default = "default_handy_download_host_allowlist")]
    pub download_host_allowlist: Vec<String>,
    /// Maximum allowed download size in bytes.
    #[serde(default = "default_handy_max_download_bytes")]
    pub max_download_bytes: u64,
    /// Timeout in seconds for offline audio transcription.
    #[serde(default = "default_handy_transcribe_timeout_secs")]
    pub transcribe_timeout_secs: u64,
}

impl Default for HandyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            install_policy: default_handy_install_policy(),
            pin_version: String::new(),
            allow_downgrade: false,
            launch_on_agent_start: default_handy_launch_on_agent_start(),
            auto_start_with_os: false,
            start_hidden: default_handy_start_hidden(),
            custom_exe_path: None,
            download_host_allowlist: default_handy_download_host_allowlist(),
            max_download_bytes: default_handy_max_download_bytes(),
            transcribe_timeout_secs: default_handy_transcribe_timeout_secs(),
        }
    }
}
