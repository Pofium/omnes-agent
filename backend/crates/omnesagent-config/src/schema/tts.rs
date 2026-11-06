//! Text-to-Speech config (the `[tts]` section): global runtime knobs plus the
//! per-instance provider shape (`[tts_providers.<type>.<alias>]`).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::{TtsConfig, TtsProviderConfig}` are unchanged.

use super::*;

// ── TTS (Text-to-Speech) ─────────────────────────────────────────

fn default_tts_voice() -> String {
    "alloy".into()
}

fn default_tts_format() -> String {
    "mp3".into()
}

fn default_tts_max_text_length() -> usize {
    4096
}

/// Text-to-Speech subsystem configuration (`[tts]`).
///
/// Per-instance TTS configs live under `[tts_providers.<type>.<alias>]`
/// (parallel to `providers.models`). What remains here are the global
/// runtime knobs that apply to every model_provider invocation.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "tts"]
pub struct TtsConfig {
    /// Enable TTS synthesis.
    #[serde(default)]
    pub enabled: bool,
    /// Default voice ID passed to the selected tts provider.
    #[serde(default = "default_tts_voice")]
    pub default_voice: String,
    /// Default audio output format (`"mp3"`, `"opus"`, `"wav"`).
    #[serde(default = "default_tts_format")]
    pub default_format: String,
    /// Maximum input text length in characters (default 4096).
    #[serde(default = "default_tts_max_text_length")]
    pub max_text_length: usize,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_voice: default_tts_voice(),
            default_format: default_tts_format(),
            max_text_length: default_tts_max_text_length(),
        }
    }
}

/// Per-instance TTS model_provider configuration (`[tts_providers.<type>.<alias>]`).
///
/// Mirrors `ModelProviderConfig` in shape — one struct holds the union of
/// fields across backends. Only the fields relevant to the selected backend
/// (determined by the outer `<type>` map key) are read at runtime; others
/// are quietly ignored.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "tts_provider"]
#[serde(default)]
pub struct TtsProviderConfig {
    /// API key (openai, elevenlabs, google).
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub api_key: Option<String>,
    /// Model name. OpenAI uses this for `tts-1`/`tts-1-hd`; elevenlabs uses
    /// it as the model_id (e.g. `eleven_monolingual_v1`).
    pub model: Option<String>,
    /// Voice override for this instance. When empty, falls back to
    /// `[tts].default_voice`.
    pub voice: Option<String>,
    /// Playback speed multiplier (openai only; default `1.0`).
    pub speed: Option<f64>,
    /// Voice stability for elevenlabs (0.0-1.0; default `0.5`).
    pub stability: Option<f64>,
    /// Similarity boost for elevenlabs (0.0-1.0; default `0.5`).
    pub similarity_boost: Option<f64>,
    /// Language code for google (e.g. `en-US`).
    pub language_code: Option<String>,
    /// Path to backend binary (edge-tts subprocess; piper local server).
    pub binary_path: Option<String>,
    /// Audio response format sent to the TTS backend (e.g. `"opus"`, `"mp3"`,
    /// `"wav"`). Defaults to `"opus"` for the OpenAI family. Override to
    /// `"wav"` for Orpheus-class models (e.g. `canopylabs/orpheus-v1-english`
    /// on Groq) or `"mp3"` for broader compatibility.
    pub response_format: Option<String>,
    /// Endpoint URI for HTTP-based backends. Overrides the family default
    /// when pointing at a compatible third-party API (Groq, Azure, self-hosted
    /// proxies). Set to the **full** URL — there is no separate path-suffix
    /// field. Renamed from `api_url` for parity with `ModelProviderConfig.uri`.
    #[serde(alias = "api_url")]
    pub uri: Option<String>,
}
