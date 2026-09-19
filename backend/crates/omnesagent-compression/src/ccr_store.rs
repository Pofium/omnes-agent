//! In-memory Content Cache Retrieval (CCR) store based on Headroom algorithms.
//!
//! Temporarily holds offloaded items from large JSON arrays, tables, and long logs,
//! allowing reversible on-demand expansion while keeping the active prompt lean.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde_json::Value;

/// Stored offloaded payload.
#[derive(Debug, Clone)]
pub struct CcrEntry {
    pub hash: String,
    pub original_count: usize,
    pub dropped_count: usize,
    pub items: Vec<Value>,
}

/// Thread-safe in-memory CCR store.
#[derive(Clone, Default)]
pub struct CcrStore {
    entries: Arc<RwLock<HashMap<String, CcrEntry>>>,
}

impl CcrStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Stores dropped items under a generated hash.
    pub fn store(&self, hash: String, original_count: usize, dropped: Vec<Value>) {
        let dropped_count = dropped.len();
        self.entries.write().insert(
            hash.clone(),
            CcrEntry {
                hash,
                original_count,
                dropped_count,
                items: dropped,
            },
        );
    }

    /// Retrieves offloaded items by hash with optional slice (offset, limit).
    #[must_use]
    pub fn retrieve(&self, hash: &str, offset: usize, limit: Option<usize>) -> Option<Vec<Value>> {
        let store = self.entries.read();
        let entry = store.get(hash)?;

        if offset >= entry.items.len() {
            return Some(Vec::new());
        }

        let slice = &entry.items[offset..];
        let count = limit.unwrap_or(slice.len()).min(slice.len());
        Some(slice[..count].to_vec())
    }

    /// Checks if a hash exists in the CCR store.
    #[must_use]
    pub fn contains(&self, hash: &str) -> bool {
        self.entries.read().contains_key(hash)
    }

    /// Clears the store.
    pub fn clear(&self) {
        self.entries.write().clear();
    }
}
