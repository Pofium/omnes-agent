//! Voice transcription config (the `[transcription]` section): multi-provider
//! Whisper-compatible STT (Groq / OpenAI / Deepgram / AssemblyAI / Google /
//! local).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::TranscriptionConfig` is unchanged. The `default_*_stt_*`
//! helpers are `pub(super)` because the per-provider STT configs that use
//! them stay in `schema`.

use super::*;

// ── Transcription ────────────────────────────────────────────────

fn default_transcription_api_url() -> String {
    "https://api.groq.com/openai/v1/audio/transcriptions".into()
}

fn default_transcription_model() -> String {
    "whisper-large-v3-turbo".into()
}

fn default_transcription_max_duration_secs() -> u64 {
    120
}

pub(super) fn default_openai_stt_model() -> String {
    "whisper-1".into()
}

pub(super) fn default_deepgram_stt_model() -> String {
    "nova-2".into()
}

pub(super) fn default_google_stt_language_code() -> String {
    "en-US".into()
}

/// Voice transcription configuration with multi-provider support.
///
/// The top-level `api_url`, `model`, and `api_key` fields remain for backward
/// compatibility with existing Groq-based configurations.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "transcription"]
pub struct TranscriptionConfig {
    /// Enable voice transcription for channels that support it.
    #[serde(default)]
    pub enabled: bool,
    /// API key used for transcription requests (Groq transcription provider).
    ///
    /// Use the schema-mirror env grammar (`OMNESAGENT_transcription__api_key=...`)
    /// for runtime env injection. Legacy `GROQ_API_KEY` constructor fallback was
    /// removed in v0.8.0.
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub api_key: Option<String>,
    /// Whisper API endpoint URL (Groq transcription provider).
    #[serde(default = "default_transcription_api_url")]
    pub api_url: String,
    /// Whisper model name (Groq transcription provider).
    #[serde(default = "default_transcription_model")]
    pub model: String,
    /// Optional language hint (ISO-639-1, e.g. "en", "ru") for Groq transcription provider.
    #[serde(default)]
    pub language: Option<String>,
    /// Optional initial prompt to bias transcription toward expected vocabulary
    /// (proper nouns, technical terms, etc.). Sent as the `prompt` field in the
    /// Whisper API request.
    #[serde(default)]
    pub initial_prompt: Option<String>,
    /// Optional global audio size upper bound in bytes, enforced before
    /// dispatching to any transcription provider. Provider-specific caps still
    /// apply.
    #[serde(default)]
    pub max_audio_bytes: Option<usize>,
    /// Maximum voice duration in seconds (messages longer than this are skipped).
    #[serde(default = "default_transcription_max_duration_secs")]
    pub max_duration_secs: u64,
    /// OpenAI Whisper STT model_provider configuration.
    #[serde(default)]
    #[nested]
    pub openai: Option<OpenAiSttConfig>,
    /// Deepgram STT model_provider configuration.
    #[serde(default)]
    #[nested]
    pub deepgram: Option<DeepgramSttConfig>,
    /// AssemblyAI STT model_provider configuration.
    #[serde(default)]
    #[nested]
    pub assemblyai: Option<AssemblyAiSttConfig>,
    /// Google Cloud Speech-to-Text model_provider configuration.
    #[serde(default)]
    #[nested]
    pub google: Option<GoogleSttConfig>,
    /// Local/self-hosted Whisper-compatible STT model_provider.
    #[serde(default)]
    #[nested]
    pub local_whisper: Option<LocalWhisperConfig>,
    /// Also transcribe non-PTT (forwarded/regular) audio messages on WhatsApp,
    /// not just voice notes. Default: `false` (preserves legacy behavior).
    #[serde(default)]
    pub transcribe_non_ptt_audio: bool,
}

impl Default for TranscriptionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: None,
            api_url: default_transcription_api_url(),
            model: default_transcription_model(),
            language: None,
            initial_prompt: None,
            max_audio_bytes: None,
            max_duration_secs: default_transcription_max_duration_secs(),
            openai: None,
            deepgram: None,
            assemblyai: None,
            google: None,
            local_whisper: None,
            transcribe_non_ptt_audio: false,
        }
    }
}
