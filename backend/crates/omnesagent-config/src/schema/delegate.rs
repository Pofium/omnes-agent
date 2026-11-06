//! Delegate tool configuration (the `[delegate]` section): default timeout
//! values for non-agentic sub-agent model-provider calls and agentic
//! sub-agent runs.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::DelegateToolConfig` is unchanged.

use super::*;

/// Global delegate tool configuration for default timeout values.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "delegate"]
pub struct DelegateToolConfig {
    /// Default timeout in seconds for non-agentic sub-agent model_provider calls.
    /// Can be overridden per-agent in `[agents.<name>]` config.
    /// Default: 120 seconds.
    #[serde(default = "default_delegate_timeout_secs")]
    pub timeout_secs: u64,
    /// Default timeout in seconds for agentic sub-agent runs.
    /// Can be overridden per-agent in `[agents.<name>]` config.
    /// Default: 300 seconds.
    #[serde(default = "default_delegate_agentic_timeout_secs")]
    pub agentic_timeout_secs: u64,
}

impl Default for DelegateToolConfig {
    fn default() -> Self {
        Self {
            timeout_secs: DEFAULT_DELEGATE_TIMEOUT_SECS,
            agentic_timeout_secs: DEFAULT_DELEGATE_AGENTIC_TIMEOUT_SECS,
        }
    }
}
