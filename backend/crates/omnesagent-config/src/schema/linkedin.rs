//! LinkedIn integration configuration (the `[linkedin]` section): the
//! `LinkedInConfig` tool toggle, its `[linkedin.content]` strategy and
//! `[linkedin.image]` generator, plus the per-provider image configs
//! (Stability / Imagen / DALL-E / Flux).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::LinkedInConfig` path is unchanged.

use super::*;

// ── LinkedIn ────────────────────────────────────────────────────

/// LinkedIn integration configuration (`[linkedin]` section).
///
/// When enabled, the `linkedin` tool is registered in the agent tool surface.
/// Requires `LINKEDIN_*` credentials in the workspace `.env` file.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "linkedin"]
pub struct LinkedInConfig {
    /// Enable the LinkedIn tool.
    #[serde(default)]
    pub enabled: bool,

    /// LinkedIn REST API version header (YYYYMM format).
    #[serde(default = "default_linkedin_api_version")]
    pub api_version: String,

    /// Content strategy for automated posting.
    #[serde(default)]
    #[nested]
    pub content: LinkedInContentConfig,

    /// Image generation for posts (`[linkedin.image]`).
    #[serde(default)]
    #[nested]
    pub image: LinkedInImageConfig,
}

impl Default for LinkedInConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_version: default_linkedin_api_version(),
            content: LinkedInContentConfig::default(),
            image: LinkedInImageConfig::default(),
        }
    }
}

fn default_linkedin_api_version() -> String {
    "202602".to_string()
}

/// Content strategy configuration for LinkedIn auto-posting (`[linkedin.content]`).
///
/// The agent reads this via the `linkedin get_content_strategy` action to know
/// what feeds to check, which repos to highlight, and how to write posts.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "linkedin.content"]
pub struct LinkedInContentConfig {
    /// RSS feed URLs to monitor for topic inspiration (titles only).
    #[serde(default)]
    pub rss_feeds: Vec<String>,

    /// GitHub usernames whose public activity to reference.
    #[serde(default)]
    pub github_users: Vec<String>,

    /// GitHub repositories to highlight (format: `owner/repo`).
    #[serde(default)]
    pub github_repos: Vec<String>,

    /// Topics of expertise and interest for post themes.
    #[serde(default)]
    pub topics: Vec<String>,

    /// Professional persona description (name, role, expertise).
    #[serde(default)]
    pub persona: String,

    /// Freeform posting instructions for the AI agent.
    #[serde(default)]
    pub instructions: String,
}

/// Image generation configuration for LinkedIn posts (`[linkedin.image]`).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "linkedin.image"]
pub struct LinkedInImageConfig {
    /// Enable image generation for posts.
    #[serde(default)]
    pub enabled: bool,

    /// ModelProvider priority order. Tried in sequence; first success wins.
    #[serde(default = "default_image_providers")]
    pub providers: Vec<String>,

    /// Generate a branded SVG text card when all AI model_providers fail.
    #[serde(default = "default_true")]
    pub fallback_card: bool,

    /// Accent color for the fallback card (CSS hex).
    #[serde(default = "default_card_accent_color")]
    pub card_accent_color: String,

    /// Temp directory for generated images, relative to workspace.
    #[serde(default = "default_image_temp_dir")]
    pub temp_dir: String,

    /// Stability AI model_provider settings.
    #[serde(default)]
    #[nested]
    pub stability: ImageProviderStabilityConfig,

    /// Google Imagen (Vertex AI) model_provider settings.
    #[serde(default)]
    #[nested]
    pub imagen: ImageProviderImagenConfig,

    /// OpenAI DALL-E model_provider settings.
    #[serde(default)]
    #[nested]
    pub dalle: ImageProviderDalleConfig,

    /// Flux (fal.ai) model_provider settings.
    #[serde(default)]
    #[nested]
    pub flux: ImageProviderFluxConfig,
}

fn default_image_providers() -> Vec<String> {
    vec![
        "stability".into(),
        "imagen".into(),
        "dalle".into(),
        "flux".into(),
    ]
}

fn default_card_accent_color() -> String {
    "#0A66C2".into()
}

fn default_image_temp_dir() -> String {
    "linkedin/images".into()
}

impl Default for LinkedInImageConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            providers: default_image_providers(),
            fallback_card: true,
            card_accent_color: default_card_accent_color(),
            temp_dir: default_image_temp_dir(),
            stability: ImageProviderStabilityConfig::default(),
            imagen: ImageProviderImagenConfig::default(),
            dalle: ImageProviderDalleConfig::default(),
            flux: ImageProviderFluxConfig::default(),
        }
    }
}

/// Stability AI image generation settings (`[linkedin.image.stability]`).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "linkedin.image.stability"]
pub struct ImageProviderStabilityConfig {
    /// Environment variable name holding the API key.
    #[serde(default = "default_stability_api_key_env")]
    #[credential_class = "legacy_env_path"]
    pub api_key_env: String,
    /// Stability model identifier.
    #[serde(default = "default_stability_model")]
    pub model: String,
}

fn default_stability_api_key_env() -> String {
    "STABILITY_API_KEY".into()
}
fn default_stability_model() -> String {
    "stable-diffusion-xl-1024-v1-0".into()
}

impl Default for ImageProviderStabilityConfig {
    fn default() -> Self {
        Self {
            api_key_env: default_stability_api_key_env(),
            model: default_stability_model(),
        }
    }
}

/// Google Imagen (Vertex AI) settings (`[linkedin.image.imagen]`).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "linkedin.image.imagen"]
pub struct ImageProviderImagenConfig {
    /// Environment variable name holding the API key.
    #[serde(default = "default_imagen_api_key_env")]
    #[credential_class = "legacy_env_path"]
    pub api_key_env: String,
    /// Environment variable for the Google Cloud project ID.
    #[serde(default = "default_imagen_project_id_env")]
    #[credential_class = "legacy_env_path"]
    pub project_id_env: String,
    /// Vertex AI region.
    #[serde(default = "default_imagen_region")]
    pub region: String,
}

fn default_imagen_api_key_env() -> String {
    "GOOGLE_VERTEX_API_KEY".into()
}
fn default_imagen_project_id_env() -> String {
    "GOOGLE_CLOUD_PROJECT".into()
}
fn default_imagen_region() -> String {
    "us-central1".into()
}

impl Default for ImageProviderImagenConfig {
    fn default() -> Self {
        Self {
            api_key_env: default_imagen_api_key_env(),
            project_id_env: default_imagen_project_id_env(),
            region: default_imagen_region(),
        }
    }
}

/// OpenAI DALL-E settings (`[linkedin.image.dalle]`).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "linkedin.image.dalle"]
pub struct ImageProviderDalleConfig {
    /// Environment variable name holding the OpenAI API key.
    #[serde(default = "default_dalle_api_key_env")]
    #[credential_class = "legacy_env_path"]
    pub api_key_env: String,
    /// DALL-E model identifier.
    #[serde(default = "default_dalle_model")]
    pub model: String,
    /// Image dimensions.
    #[serde(default = "default_dalle_size")]
    pub size: String,
}

fn default_dalle_api_key_env() -> String {
    "OPENAI_API_KEY".into()
}
fn default_dalle_model() -> String {
    "dall-e-3".into()
}
fn default_dalle_size() -> String {
    "1024x1024".into()
}

impl Default for ImageProviderDalleConfig {
    fn default() -> Self {
        Self {
            api_key_env: default_dalle_api_key_env(),
            model: default_dalle_model(),
            size: default_dalle_size(),
        }
    }
}

/// Flux (fal.ai) image generation settings (`[linkedin.image.flux]`).
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "linkedin.image.flux"]
pub struct ImageProviderFluxConfig {
    /// Environment variable name holding the fal.ai API key.
    #[serde(default = "default_flux_api_key_env")]
    #[credential_class = "legacy_env_path"]
    pub api_key_env: String,
    /// Flux model identifier.
    #[serde(default = "default_flux_model")]
    pub model: String,
}

fn default_flux_api_key_env() -> String {
    "FAL_API_KEY".into()
}
fn default_flux_model() -> String {
    "fal-ai/flux/schnell".into()
}

impl Default for ImageProviderFluxConfig {
    fn default() -> Self {
        Self {
            api_key_env: default_flux_api_key_env(),
            model: default_flux_model(),
        }
    }
}
