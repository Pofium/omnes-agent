//! Canonical on-disk model cache schema, shared by `omnesagent models refresh`
//! and the channel `/model` command.
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::ModelCacheState` path is unchanged.

use super::*;

// ── Model cache (shared between CLI refresh and channel reader) ──

/// Canonical on-disk model cache schema. Written by `omnesagent models refresh`
/// and read by the channel `/model` command. Both sides MUST use this type
/// to prevent schema drift (single-source-of-truth rule).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ModelCacheState {
    pub entries: Vec<ModelCacheEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ModelCacheEntry {
    pub model_provider: String,
    pub models: Vec<String>,
}
