//! Model-family-aware compression router and policy configuration.
//!
//! Adapts compression strategies based on target model family:
//! - DeepSeek: Enforces Static Prefix alignment for 99% cache hit + sqz dedup.
//! - GLM: Uses safe markdown references to avoid parser deadlocks.
//! - Claude: Configures ephemeral cache breakpoints and signature minification.
//! - OpenAI: Uses Compact JSON Schema for strict schema validation.
//! - Qwen/Kimi/MiMo: Full SmartCrusher table compaction + AST collapse.

use serde::{Deserialize, Serialize};
use crate::mcp_schema::SchemaTransformMode;
use crate::sqz_dedup::DedupRefFormat;

/// Target model family classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ModelFamily {
    #[default]
    Generic,
    DeepSeek,
    Glm,
    Claude,
    OpenAi,
    Qwen,
    Kimi,
    MiMo,
}

impl ModelFamily {
    /// Detects model family from model identifier string.
    #[must_use]
    pub fn from_model_name(name: &str) -> Self {
        let lower = name.to_lowercase();
        if lower.contains("deepseek") {
            Self::DeepSeek
        } else if lower.contains("glm") || lower.contains("zhipu") {
            Self::Glm
        } else if lower.contains("claude") || lower.contains("anthropic") {
            Self::Claude
        } else if lower.contains("gpt") || lower.contains("o1") || lower.contains("o3") || lower.contains("openai") {
            Self::OpenAi
        } else if lower.contains("qwen") {
            Self::Qwen
        } else if lower.contains("kimi") || lower.contains("moonshot") {
            Self::Kimi
        } else if lower.contains("mimo") || lower.contains("xiaomi") {
            Self::MiMo
        } else {
            Self::Generic
        }
    }

    /// Alias for `from_model_name`.
    #[must_use]
    pub fn detect_from_name(name: &str) -> Self {
        Self::from_model_name(name)
    }
}

/// Compression policy applied to a turn or session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionPolicy {
    /// Target model family.
    pub family: ModelFamily,
    /// Schema transformation mode.
    pub schema_mode: SchemaTransformMode,
    /// Whether deduplication of repeated files and outputs is enabled.
    pub enable_dedup: bool,
    /// Reference formatting style (§ref§ vs safe markdown).
    pub dedup_ref_format: DedupRefFormat,
    /// Whether structural SmartCrusher (JSON arrays & logs) is enabled.
    pub enable_smart_crusher: bool,
    /// Whether AST code body collapsing is enabled.
    pub enable_ast_code_collapse: bool,
    /// Enforce static ordering of system prompts and tools for prefix caching.
    pub enforce_static_prefix: bool,
    /// Emit ephemeral cache control breakpoints for Claude.
    pub ephemeral_cache_breakpoints: bool,
}

impl Default for CompressionPolicy {
    fn default() -> Self {
        Self::for_family(ModelFamily::Generic)
    }
}

impl CompressionPolicy {
    /// Builds an optimized compression policy tailored to the model family.
    #[must_use]
    pub fn for_family(family: ModelFamily) -> Self {
        match family {
            ModelFamily::DeepSeek => Self {
                family,
                schema_mode: SchemaTransformMode::CompactSignatures,
                enable_dedup: true,
                dedup_ref_format: DedupRefFormat::CompactToken,
                enable_smart_crusher: true,
                enable_ast_code_collapse: true,
                enforce_static_prefix: true,
                ephemeral_cache_breakpoints: false,
            },
            ModelFamily::Glm => Self {
                family,
                schema_mode: SchemaTransformMode::CompactSignatures,
                enable_dedup: true,
                dedup_ref_format: DedupRefFormat::SafeMarkdown, // Safe markdown to prevent GLM token loop
                enable_smart_crusher: true,
                enable_ast_code_collapse: true,
                enforce_static_prefix: false,
                ephemeral_cache_breakpoints: false,
            },
            ModelFamily::Claude => Self {
                family,
                schema_mode: SchemaTransformMode::CompactSignatures,
                enable_dedup: true,
                dedup_ref_format: DedupRefFormat::CompactToken,
                enable_smart_crusher: true,
                enable_ast_code_collapse: true,
                enforce_static_prefix: false,
                ephemeral_cache_breakpoints: true, // Up to 4 ephemeral cache control breakpoints
            },
            ModelFamily::OpenAi => Self {
                family,
                schema_mode: SchemaTransformMode::CompactJson, // Valid JSON Schema required for structured outputs
                enable_dedup: true,
                dedup_ref_format: DedupRefFormat::CompactToken,
                enable_smart_crusher: true,
                enable_ast_code_collapse: true,
                enforce_static_prefix: true,
                ephemeral_cache_breakpoints: false,
            },
            ModelFamily::Qwen | ModelFamily::Kimi | ModelFamily::MiMo => Self {
                family,
                schema_mode: SchemaTransformMode::CompactSignatures,
                enable_dedup: true,
                dedup_ref_format: DedupRefFormat::CompactToken,
                enable_smart_crusher: true,
                enable_ast_code_collapse: true,
                enforce_static_prefix: true,
                ephemeral_cache_breakpoints: false,
            },
            ModelFamily::Generic => Self {
                family,
                schema_mode: SchemaTransformMode::CompactSignatures,
                enable_dedup: true,
                dedup_ref_format: DedupRefFormat::CompactToken,
                enable_smart_crusher: true,
                enable_ast_code_collapse: false,
                enforce_static_prefix: false,
                ephemeral_cache_breakpoints: false,
            },
        }
    }
}
