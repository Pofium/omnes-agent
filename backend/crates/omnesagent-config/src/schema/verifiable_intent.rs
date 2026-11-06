//! Verifiable Intent (VI) credential config (the `[verifiable_intent]`
//! section).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::VerifiableIntentConfig` is unchanged.

use super::*;

// ── Verifiable Intent ──────────────────────────────────────────────

/// Verifiable Intent (VI) credential issuance and constraint checking
/// (`[verifiable_intent]` section).
///
/// OmnesAgent implements issuance, crypto, types and constraint checking, but not
/// a credential chain verifier. Until one exists the `vi_verify` tool is
/// withheld from the model-visible registry, so neither key below enables
/// verification of a credential. The library paths are unaffected.
///
/// Enabling the section reports that gap two ways. The runtime traces it at
/// each config application, which needs log persistence to be on to reach a
/// sink. `omnesagent doctor` and the config API also report it as the
/// `verifiable_intent_tool_withheld` validation warning, which stays available
/// when persistence is off.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "verifiable_intent"]
pub struct VerifiableIntentConfig {
    /// Opt in to the VI section (default: false).
    ///
    /// While the tool is withheld this does not enable credential verification
    /// on commerce tool calls. It currently causes a warning naming that gap,
    /// emitted once per config application: at process startup, and again when
    /// a daemon reload re-reads config from disk.
    #[serde(default)]
    pub enabled: bool,

    /// Intended strictness mode for constraint evaluation.
    ///
    /// Accepts `"strict"` or `"permissive"`, and defaults to `"strict"`. No
    /// production code reads it while the tool is withheld.
    #[serde(default = "default_vi_strictness")]
    pub strictness: String,
}

fn default_vi_strictness() -> String {
    "strict".to_owned()
}

impl Default for VerifiableIntentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strictness: default_vi_strictness(),
        }
    }
}
