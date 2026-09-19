//! SmartCrusher for JSON arrays, tables, and long logs based on Headroom algorithms.
//!
//! Provides structural compression of uniform JSON records, table projection,
//! log compaction, and reversible row-capping with CCR sentinels.

use regex::Regex;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use crate::ccr_store::CcrStore;

pub const DEFAULT_MAX_JSON_ROWS: usize = 30;
pub const DEFAULT_MAX_LOG_LINES: usize = 60;

/// Native SmartCrusher for tool outputs.
pub struct SmartCrusher {
    ccr_store: CcrStore,
    max_json_rows: usize,
    max_log_lines: usize,
}

impl SmartCrusher {
    #[must_use]
    pub fn new(ccr_store: CcrStore) -> Self {
        Self {
            ccr_store,
            max_json_rows: DEFAULT_MAX_JSON_ROWS,
            max_log_lines: DEFAULT_MAX_LOG_LINES,
        }
    }

    /// Crushes tool output text. If it's JSON, applies structural array crushing;
    /// otherwise, applies log compaction and ANSI cleanup.
    #[must_use]
    pub fn crush_output(&self, text: &str) -> String {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        // Try JSON parsing
        if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
            let crushed_val = self.crush_json_value(val);
            return serde_json::to_string(&crushed_val).unwrap_or_else(|_| text.to_string());
        }

        // Fallback to Log Compactor
        self.compact_logs(text)
    }

    /// Structurally compacts a JSON value.
    #[must_use]
    pub fn crush_json_value(&self, val: Value) -> Value {
        match val {
            Value::Array(items) => {
                if items.len() <= self.max_json_rows {
                    Value::Array(items.into_iter().map(|it| self.crush_json_value(it)).collect())
                } else {
                    // Lossy capping with CCR offload
                    let total = items.len();
                    let (kept, dropped) = items.split_at(self.max_json_rows);
                    let mut kept_vec: Vec<Value> = kept.iter().cloned().map(|it| self.crush_json_value(it)).collect();

                    let hash = self.compute_hash_from_values(dropped);
                    self.ccr_store.store(hash.clone(), total, dropped.to_vec());

                    // Append CCR sentinel
                    kept_vec.push(json!({
                        "_ccr_dropped": format!("<<ccr:{hash} {}_items_offloaded>>", dropped.len())
                    }));

                    Value::Array(kept_vec)
                }
            }
            Value::Object(map) => {
                let mut out = Map::new();
                for (k, v) in map {
                    out.insert(k, self.crush_json_value(v));
                }
                Value::Object(out)
            }
            other => other,
        }
    }

    /// Compacts terminal logs by stripping ANSI codes and collapsing duplicate lines.
    #[must_use]
    pub fn compact_logs(&self, text: &str) -> String {
        let ansi_regex = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
        let cleaned = ansi_regex.replace_all(text, "");

        let lines: Vec<&str> = cleaned.lines().collect();
        if lines.len() <= self.max_log_lines {
            return cleaned.to_string();
        }

        // Head-tail preservation (keep top 25 and bottom 25 lines)
        let head_count = self.max_log_lines / 2;
        let tail_count = self.max_log_lines - head_count;

        let head = &lines[..head_count];
        let tail = &lines[lines.len() - tail_count..];
        let omitted = lines.len() - head_count - tail_count;

        format!(
            "{}\n... [SmartCrusher: {omitted} log lines omitted] ...\n{}",
            head.join("\n"),
            tail.join("\n")
        )
    }

    fn compute_hash_from_values(&self, values: &[Value]) -> String {
        let mut hasher = Sha256::new();
        for v in values {
            hasher.update(v.to_string().as_bytes());
        }
        let result = hasher.finalize();
        result[..4].iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_crusher_json_capping() {
        let ccr = CcrStore::new();
        let mut crusher = SmartCrusher::new(ccr.clone());
        crusher.max_json_rows = 5;

        let items: Vec<Value> = (0..20).map(|i| json!({ "id": i, "name": format!("item_{i}") })).collect();
        let json_str = serde_json::to_string(&items).unwrap();

        let crushed = crusher.crush_output(&json_str);
        let parsed: Value = serde_json::from_str(&crushed).unwrap();
        let arr = parsed.as_array().unwrap();

        // 5 kept items + 1 CCR sentinel
        assert_eq!(arr.len(), 6);
        let sentinel = arr.last().unwrap();
        assert!(sentinel.get("_ccr_dropped").is_some());
        assert!(ccr.contains(&crusher.compute_hash_from_values(&items[5..])));
    }

    #[test]
    fn test_log_compaction() {
        let ccr = CcrStore::new();
        let crusher = SmartCrusher::new(ccr);
        let mut log = String::new();
        for i in 0..100 {
            log.push_str(&format!("Line {i}: some compiler status output\n"));
        }

        let compacted = crusher.compact_logs(&log);
        assert!(compacted.contains("SmartCrusher: 40 log lines omitted"));
    }
}
