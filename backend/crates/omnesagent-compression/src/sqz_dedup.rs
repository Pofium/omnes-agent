//! Content-addressable deduplication engine based on `sqz` algorithms.
//!
//! Hashes observed tool outputs and file reads, replacing repeated occurrences
//! across turns with 13-token references (§ref:HASH|NL§) or safe markdown pointers.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use sha2::{Digest, Sha256};

/// Maximum number of blocks retained in memory per session.
pub const DEFAULT_MAX_BLOCKS: usize = 1000;

/// Minimum text length (in bytes) to consider for deduplication.
pub const MIN_DEDUP_BYTE_LENGTH: usize = 120;

use serde::{Deserialize, Serialize};

/// Block reference formatting style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DedupRefFormat {
    /// Standard compact token: `§ref:HASH|NL§` (~13 tokens).
    #[default]
    CompactToken,
    /// Safe human-readable markdown format for GLM/parsers:
    /// `[Повторный контент: hash=HASH, строк=N]`
    SafeMarkdown,
}

/// In-memory stored content block.
#[derive(Debug, Clone)]
pub struct BlockEntry {
    pub hash: String,
    pub content: String,
    pub line_count: usize,
    pub byte_length: usize,
    pub occurrences: usize,
}

/// Native Content-Addressable Dedup Engine.
#[derive(Clone)]
pub struct SqzDedupEngine {
    store: Arc<RwLock<HashMap<String, BlockEntry>>>,
    max_blocks: usize,
}

impl Default for SqzDedupEngine {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_BLOCKS)
    }
}

impl SqzDedupEngine {
    /// Creates a new deduplication engine instance.
    #[must_use]
    pub fn new(max_blocks: usize) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::with_capacity(max_blocks))),
            max_blocks,
        }
    }

    /// Computes an 8-character hex content hash for the given text.
    #[must_use]
    pub fn compute_hash(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let result = hasher.finalize();
        result[..4].iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Processes text through the deduplicator:
    /// - If text was seen previously, replaces it with a compact reference token.
    /// - If new, registers it in the block store and returns the original text.
    pub fn process_text(&self, text: &str, format: DedupRefFormat) -> String {
        let trimmed = text.trim();
        if trimmed.len() < MIN_DEDUP_BYTE_LENGTH {
            return text.to_string();
        }

        let hash = Self::compute_hash(trimmed);
        let lines = trimmed.lines().count();

        {
            let mut store = self.store.write();
            if let Some(entry) = store.get_mut(&hash) {
                entry.occurrences += 1;
                return Self::format_reference(&hash, lines, format);
            }

            // Evict arbitrary entry if capacity reached
            if store.len() >= self.max_blocks {
                if let Some(first_key) = store.keys().next().cloned() {
                    store.remove(&first_key);
                }
            }

            store.insert(
                hash.clone(),
                BlockEntry {
                    hash,
                    content: text.to_string(),
                    line_count: lines,
                    byte_length: text.len(),
                    occurrences: 1,
                },
            );
        }

        text.to_string()
    }

    /// Expands a reference hash back into its original text content if present in store.
    #[must_use]
    pub fn expand(&self, hash: &str) -> Option<String> {
        let store = self.store.read();
        store.get(hash).map(|e| e.content.clone())
    }

    /// Clears the block store.
    pub fn clear(&self) {
        self.store.write().clear();
    }

    /// Number of blocks currently tracked.
    #[must_use]
    pub fn len(&self) -> usize {
        self.store.read().len()
    }

    /// Returns true if no blocks are tracked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn format_reference(hash: &str, lines: usize, format: DedupRefFormat) -> String {
        match format {
            DedupRefFormat::CompactToken => {
                format!("§ref:{hash}|{lines}L§")
            }
            DedupRefFormat::SafeMarkdown => {
                format!("[Повторный контент: hash={hash}, строк={lines} сохранен в кэше]")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication_cycle() {
        let engine = SqzDedupEngine::default();
        let repeated_text = "This is a long file content that exceeds the minimum deduplication length threshold and is repeatedly sent across agent turns.\nLine 2 with detailed information.\nLine 3 with more code.";

        // First pass: registered, returns original
        let first = engine.process_text(repeated_text, DedupRefFormat::CompactToken);
        assert_eq!(first, repeated_text);

        // Second pass: deduplicated into compact token
        let second = engine.process_text(repeated_text, DedupRefFormat::CompactToken);
        assert!(second.starts_with("§ref:"));
        assert!(second.ends_with("L§"));

        // Safe markdown format
        let safe = engine.process_text(repeated_text, DedupRefFormat::SafeMarkdown);
        assert!(safe.contains("Повторный контент: hash="));

        // Test expansion
        let hash = SqzDedupEngine::compute_hash(repeated_text.trim());
        let expanded = engine.expand(&hash);
        assert_eq!(expanded.as_deref(), Some(repeated_text));
    }
}
