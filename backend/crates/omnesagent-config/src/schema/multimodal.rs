//! Multimodal (image) handling configuration (the `[multimodal]` section).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so
//! `crate::schema::MultimodalConfig` is unchanged.

use super::*;

/// Multimodal (image) handling configuration (`[multimodal]` section).
///
/// # Privacy and cost note
///
/// Tool results that print real local image paths (e.g. shell tools doing
/// `ls /pictures` or `find . -name '*.png'`) are canonicalized into
/// `[IMAGE:...]` markers and base64-inlined into the next provider request.
/// This means image bytes that previously stayed local will be uploaded to
/// the configured provider when surfaced by a tool.
///
/// `max_images` (and the `trim_old_images` LRU policy) bounds the per-request
/// image budget, but operators running shell-style tools over directories of
/// personal or sensitive images should be aware of the upload semantics. See
/// `docs/book/src/contributing/privacy.md` for the project's privacy stance.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "multimodal"]
pub struct MultimodalConfig {
    /// Maximum number of image attachments accepted per request.
    ///
    /// Caps the total number of `[IMAGE:...]` markers that survive into the
    /// provider request after multimodal preprocessing. Older images are
    /// dropped first when the cumulative count exceeds this limit. Acts as
    /// the upper bound on per-turn upload cost when tool outputs surface
    /// local image paths.
    #[serde(default = "default_multimodal_max_images")]
    pub max_images: usize,
    /// Maximum image payload size in MiB before base64 encoding.
    ///
    /// Measured on decoded bytes, so the encoded request payload is about a
    /// third larger. Applies to every image entering the pipeline: channel
    /// attachments, tool outputs that surface local image paths, and the web
    /// dashboard upload. Defaults to the 20 MiB ceiling that
    /// [`MultimodalConfig::effective_limits`] clamps to, so an ordinary photo
    /// is accepted without configuration; lower it to bound per-turn upload
    /// cost or gateway buffering.
    ///
    /// Providers apply their own limits on top of this one. Anthropic refuses a
    /// single image over 10 MB base64-encoded (about 7.5 MiB decoded), enforced
    /// by its provider client independently of this setting.
    #[serde(default = "default_multimodal_max_image_size_mb")]
    pub max_image_size_mb: usize,
    /// Maximum age of images in conversation turns.
    ///
    /// When non-zero, images in user messages that are more than this many
    /// turns back from the end of history are stripped before the request is
    /// sent to the provider. This prevents a single screenshot from being
    /// re-encoded and re-uploaded on every subsequent turn indefinitely.
    /// Tool-result images are already managed by the stale-tool-result
    /// mechanism and are not affected by this setting.
    ///
    /// `0` (the default) disables age-based trimming entirely — images are
    /// only evicted by the `max_images` count cap.
    #[serde(default)]
    pub max_image_turns: usize,
    /// Allow fetching remote image URLs (http/https). Disabled by default.
    #[serde(default)]
    pub allow_remote_fetch: bool,
    /// ModelProvider name to use for vision/image messages (e.g. `"ollama"`).
    /// When set, messages containing `[IMAGE:]` markers are routed to this
    /// model_provider instead of the default text model_provider.
    #[serde(default)]
    pub vision_model_provider: Option<String>,
    /// Model to use when routing to the vision model_provider (e.g. `"llava:7b"`).
    /// Only used when `vision_model_provider` is set.
    #[serde(default)]
    pub vision_model: Option<String>,
}

fn default_multimodal_max_images() -> usize {
    4
}

fn default_multimodal_max_image_size_mb() -> usize {
    20
}

impl MultimodalConfig {
    /// Clamp configured values to safe runtime bounds.
    ///
    /// The 20 MiB image ceiling is the lowest common per-image or per-request
    /// bound across the supported vision APIs (OpenAI accepts about 20 MB per
    /// image, Gemini 20 MB for an inline request, Anthropic 32 MB per request
    /// with a tighter per-image limit its own client enforces). Images are
    /// buffered whole and grow by about a third under base64, so the ceiling
    /// also bounds gateway memory per upload.
    pub fn effective_limits(&self) -> (usize, usize) {
        let max_images = self.max_images.clamp(1, 16);
        let max_image_size_mb = self.max_image_size_mb.clamp(1, 20);
        (max_images, max_image_size_mb)
    }
}

impl Default for MultimodalConfig {
    fn default() -> Self {
        Self {
            max_images: default_multimodal_max_images(),
            max_image_size_mb: default_multimodal_max_image_size_mb(),
            max_image_turns: 0,
            allow_remote_fetch: false,
            vision_model_provider: None,
            vision_model: None,
        }
    }
}
