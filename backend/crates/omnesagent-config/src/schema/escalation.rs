//! Escalation routing config (the `[escalation]` section).
//!
//! Controls which channels receive alert notifications when
//! `escalate_to_human` is called with high or critical urgency. Channels are
//! identified by name (e.g. `"telegram"`, `"slack"`). Alerts are sent
//! best-effort and do not block the escalation.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::EscalationConfig` is unchanged.

use super::*;

// ── Escalation routing ───────────────────────────────────────────

/// Escalation routing configuration (`[escalation]` section).
///
/// Controls which channels receive alert notifications when
/// `escalate_to_human` is called with high or critical urgency.
/// Channels are identified by name (e.g. `"telegram"`, `"slack"`).
/// Alerts are sent best-effort and do not block the escalation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "escalation"]
pub struct EscalationConfig {
    /// Channel names to alert on high/critical escalations (default: empty).
    ///
    /// Each name must match a configured channel. Unrecognised names are
    /// logged at WARN level and skipped.
    #[serde(default)]
    pub alert_channels: Vec<String>,
}
