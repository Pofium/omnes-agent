//! Secrets encryption configuration (the `[secrets]` section).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::SecretsConfig` is unchanged.

use super::*;

// ── Secrets (encrypted credential store) ────────────────────────

/// Secrets encryption configuration (`[secrets]` section).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "secrets"]
pub struct SecretsConfig {
    /// Enable encryption for API keys and tokens at rest
    #[serde(default = "default_true")]
    #[credential_class = "public_value"]
    pub encrypt: bool,
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self { encrypt: true }
    }
}
