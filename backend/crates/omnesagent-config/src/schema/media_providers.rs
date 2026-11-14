//! TTS and transcription typed-family provider slots: the `TtsEndpoint` /
//! `TranscriptionEndpoint` traits plus each family's endpoint enum and typed
//! config, mirroring the model provider family pattern.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::<Family>TtsProviderConfig` /
//! `<Family>TranscriptionProviderConfig` path is unchanged.

use super::{
    TtsProviderConfig, default_local_whisper_max_audio_bytes, default_local_whisper_timeout_secs,
};
use omnesagent_macros::Configurable;
use serde::{Deserialize, Serialize};

// ── TTS endpoint trait + per-family typed configs ──────────────────────────
//
// Mirrors the model provider typed-family pattern. Each TTS family carries
// its own typed config (composing TtsProviderConfig as the shared base via
// `#[serde(flatten)]`) and a single-variant `*TtsEndpoint` enum impl'ing
// `TtsEndpoint`. Edge and Piper skip the base — they're subprocess / local
// runtimes with no shared `api_key` / `voice` defaults.

/// One trait per family-endpoint enum. Returns the URI for the chosen
/// variant. Mirrors `ModelEndpoint` for parity across model and TTS.
pub trait TtsEndpoint {
    fn uri(&self) -> &'static str;
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OpenAITtsEndpoint {
    #[default]
    Default,
}
impl TtsEndpoint for OpenAITtsEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.openai.com/v1/audio/speech",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.tts.openai"]
pub struct OpenAITtsProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: TtsProviderConfig,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ElevenLabsTtsEndpoint {
    #[default]
    Default,
}
impl TtsEndpoint for ElevenLabsTtsEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.elevenlabs.io/v1/text-to-speech",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.tts.elevenlabs"]
pub struct ElevenLabsTtsProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: TtsProviderConfig,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum GoogleTtsEndpoint {
    #[default]
    Default,
}
impl TtsEndpoint for GoogleTtsEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://texttospeech.googleapis.com/v1/text:synthesize",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.tts.google"]
pub struct GoogleTtsProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: TtsProviderConfig,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum EdgeTtsEndpoint {
    /// Subprocess — no remote endpoint. Sentinel for trait conformity.
    #[default]
    LocalSubprocess,
}
impl TtsEndpoint for EdgeTtsEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::LocalSubprocess => "subprocess://edge-tts",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.tts.edge"]
pub struct EdgeTtsProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: TtsProviderConfig,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum PiperTtsEndpoint {
    #[default]
    LocalDefault,
}
impl TtsEndpoint for PiperTtsEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::LocalDefault => "http://127.0.0.1:5000/v1/audio/speech",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.tts.piper"]
pub struct PiperTtsProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: TtsProviderConfig,
}

// ── Transcription providers (typed-family split, mirrors models/tts) ────
//
// Six family slots: `groq`, `openai`, `deepgram`, `assemblyai`, `google`,
// `local_whisper`. Each is a `HashMap<String, *TranscriptionProviderConfig>`
// keyed by operator-chosen alias. The shared `TranscriptionProviderConfig`
// base carries `api_key` + `language` since every cloud STT family takes
// both; `local_whisper` skips the base because it's a self-hosted endpoint
// with its own auth token, not a vendor API key.

/// Shared base for cloud transcription providers. Each cloud family
/// composes this via `#[serde(flatten)] base: TranscriptionProviderConfig`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.transcription"]
pub struct TranscriptionProviderConfig {
    /// API key for the transcription provider.
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub api_key: Option<String>,
    /// Optional language hint passed to the provider (ISO-639-1 like `"en"` /
    /// `"ru"`, or BCP-47 like `"en-US"` for Google). Most providers auto-detect
    /// when this is unset.
    #[serde(default)]
    pub language: Option<String>,
    /// Whisper-style initial prompt to bias the model toward expected
    /// vocabulary (proper nouns, technical terms). Provider-specific support;
    /// silently ignored where not applicable.
    #[serde(default)]
    pub initial_prompt: Option<String>,
}

/// Trait that every transcription endpoint enum implements. Mirrors
/// `ModelEndpoint` / `TtsEndpoint` for parity.
pub trait TranscriptionEndpoint {
    fn uri(&self) -> &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum GroqTranscriptionEndpoint {
    #[default]
    Default,
}
impl TranscriptionEndpoint for GroqTranscriptionEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.groq.com/openai/v1/audio/transcriptions",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.transcription.groq"]
pub struct GroqTranscriptionProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: TranscriptionProviderConfig,
    /// Whisper model name (default: `"whisper-large-v3-turbo"`).
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OpenAiTranscriptionEndpoint {
    #[default]
    Default,
}
impl TranscriptionEndpoint for OpenAiTranscriptionEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.openai.com/v1/audio/transcriptions",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.transcription.openai"]
pub struct OpenAiTranscriptionProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: TranscriptionProviderConfig,
    /// Whisper model name (default: `"whisper-1"`).
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum DeepgramTranscriptionEndpoint {
    #[default]
    Default,
}
impl TranscriptionEndpoint for DeepgramTranscriptionEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.deepgram.com/v1/listen",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.transcription.deepgram"]
pub struct DeepgramTranscriptionProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: TranscriptionProviderConfig,
    /// Deepgram model name (default: `"nova-2"`).
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum AssemblyAiTranscriptionEndpoint {
    #[default]
    Default,
}
impl TranscriptionEndpoint for AssemblyAiTranscriptionEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://api.assemblyai.com/v2/transcript",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.transcription.assemblyai"]
pub struct AssemblyAiTranscriptionProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: TranscriptionProviderConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum GoogleTranscriptionEndpoint {
    #[default]
    Default,
}
impl TranscriptionEndpoint for GoogleTranscriptionEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::Default => "https://speech.googleapis.com/v1/speech:recognize",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.transcription.google"]
pub struct GoogleTranscriptionProviderConfig {
    #[nested]
    #[serde(flatten)]
    pub base: TranscriptionProviderConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum LocalWhisperTranscriptionEndpoint {
    /// Self-hosted endpoint — no remote URL. Sentinel for trait conformity.
    /// The actual URL lives on `LocalWhisperTranscriptionProviderConfig.uri`.
    #[default]
    SelfHosted,
}
impl TranscriptionEndpoint for LocalWhisperTranscriptionEndpoint {
    fn uri(&self) -> &'static str {
        match self {
            Self::SelfHosted => "self-hosted",
        }
    }
}

/// Local / self-hosted Whisper-compatible transcription endpoint. Skips the
/// shared `TranscriptionProviderConfig` base because it uses a bearer-token
/// scheme and a per-instance URL rather than a vendor API key.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "providers.transcription.local_whisper"]
pub struct LocalWhisperTranscriptionProviderConfig {
    /// Endpoint URL, e.g. `"http://10.10.0.1:8001/v1/transcribe"`.
    pub uri: String,
    /// Bearer token for endpoint authentication. Omit for unauthenticated
    /// local endpoints.
    #[serde(default)]
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub bearer_token: Option<String>,
    /// Optional language hint (passed through to the local endpoint).
    #[serde(default)]
    pub language: Option<String>,
    /// Maximum audio file size in bytes accepted by this endpoint.
    /// Defaults to 25 MB to match the cloud cap; raise as needed.
    #[serde(default = "default_local_whisper_max_audio_bytes")]
    pub max_audio_bytes: usize,
    /// Request timeout in seconds.
    #[serde(default = "default_local_whisper_timeout_secs")]
    pub timeout_secs: u64,
}

// `#[derive(Default)]` would leave `max_audio_bytes = 0` and `timeout_secs = 0`
// (Rust's `usize`/`u64` defaults), even though those fields have serde defaults
// pointing at `default_local_whisper_max_audio_bytes` /
// `default_local_whisper_timeout_secs`. `#[serde(default = ...)]` only fires for
// *deserialization* — Rust's `Default::default()` bypasses it.
//
// Without this manual impl, the `Configurable` macro-generated
// `create_map_key(...)` path inserts `LocalWhisperTranscriptionProviderConfig::default()`
// for any newly scaffolded `[providers.transcription.local_whisper.<alias>]`
// entry. The `Configurable` macro exposes `HashMap<String, T>` sections through
// `map_key_sections()` / `create_map_key()`, and the generated create path
// inserts `<T as Default>::default()` for new map entries — leaving
// `max_audio_bytes = 0`, `timeout_secs = 0`. `LocalWhisperProvider::from_typed_config`
// bridges those fields into `LocalWhisperProvider::from_config`, which still
// rejects zero for both fields. Defaults here mirror `LocalWhisperConfig` so
// scaffolded entries load without operator intervention.
impl Default for LocalWhisperTranscriptionProviderConfig {
    fn default() -> Self {
        Self {
            uri: String::new(),
            bearer_token: None,
            language: None,
            max_audio_bytes: default_local_whisper_max_audio_bytes(),
            timeout_secs: default_local_whisper_timeout_secs(),
        }
    }
}
