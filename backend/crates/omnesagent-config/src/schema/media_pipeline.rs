//! Automatic media understanding pipeline config (the `[media_pipeline]`
//! section): inbound media attachments are transcribed, described, and
//! summarised before reaching the agent.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::MediaPipelineConfig` is unchanged.

use super::*;

// ── Media Pipeline ──────────────────────────────────────────────

/// Automatic media understanding pipeline configuration (`[media_pipeline]`).
///
/// When enabled, inbound channel messages with media attachments are
/// pre-processed before reaching the agent: audio is transcribed, images are
/// annotated, and videos are summarised.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "media_pipeline"]
pub struct MediaPipelineConfig {
    /// Master toggle for the media pipeline (default: false).
    #[serde(default)]
    pub enabled: bool,

    /// Transcribe audio attachments using the configured transcription model_provider.
    #[serde(default = "default_true")]
    pub transcribe_audio: bool,

    /// Add image descriptions when a vision-capable model is active.
    #[serde(default = "default_true")]
    pub describe_images: bool,

    /// Summarize video attachments (placeholder — requires external API).
    #[serde(default = "default_true")]
    pub summarize_video: bool,
}

impl Default for MediaPipelineConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            transcribe_audio: true,
            describe_images: true,
            summarize_video: true,
        }
    }
}
