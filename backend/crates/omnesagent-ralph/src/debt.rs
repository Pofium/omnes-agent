//! Ponytail Debt Harvester: scans diffs/code for `ponytail:` markers and builds a debt ledger.
//!
//! Conforms to §2 (#5), §7.3, FR-A6 of specification.

use serde::{Deserialize, Serialize};

/// A single harvested debt item from the codebase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebtItem {
    pub file: String,
    pub line: usize,
    pub raw_marker: String,
    pub ceiling: String,
    pub upgrade_trigger: Option<String>,
    pub tags: Vec<String>,
}

/// Ledger aggregating harvested debt items across iterations.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DebtLedger {
    pub items: Vec<DebtItem>,
}

impl DebtLedger {
    /// Harvest `ponytail:` markers from a git diff or file content.
    ///
    /// Marker syntax: `ponytail: <ceiling>, <upgrade-trigger>`
    /// If `<upgrade-trigger>` is absent or empty, the item receives the "no-trigger" tag.
    pub fn harvest_from_diff(diff: &str) -> Self {
        let mut items = Vec::new();
        let mut current_file = String::new();
        let mut current_line = 0usize;

        for line in diff.lines() {
            if line.starts_with("+++ b/") {
                current_file = line.trim_start_matches("+++ b/").to_string();
                current_line = 0;
                continue;
            } else if line.starts_with("@@ ") {
                // Parse chunk line number from "@@ -a,b +c,d @@"
                if let Some(plus_pos) = line.find('+') {
                    let after_plus = &line[plus_pos + 1..];
                    let end_pos = after_plus
                        .find(|c: char| c == ',' || c == ' ')
                        .unwrap_or(after_plus.len());
                    if let Ok(num) = after_plus[..end_pos].parse::<usize>() {
                        current_line = num;
                    }
                }
                continue;
            }

            if line.starts_with('+') && !line.starts_with("+++") {
                if current_file.starts_with("openspec/") {
                    current_line += 1;
                    continue;
                }
                let added_content = &line[1..];
                if let Some(item) = Self::parse_marker_line(
                    &current_file,
                    current_line,
                    added_content,
                ) {
                    items.push(item);
                }
                current_line += 1;
            } else if !line.starts_with('-') {
                current_line += 1;
            }
        }

        DebtLedger { items }
    }

    /// Parse a single line to see if it contains a `ponytail:` marker.
    pub fn parse_marker_line(file: &str, line_num: usize, content: &str) -> Option<DebtItem> {
        let marker_key = "ponytail:";
        let pos = content.find(marker_key)?;
        let raw = content[pos + marker_key.len()..].trim();

        let (ceiling, trigger, tags) = if let Some(comma_pos) = raw.find(',') {
            let c = raw[..comma_pos].trim().to_string();
            let t = raw[comma_pos + 1..].trim();
            if t.is_empty() {
                (c, None, vec!["no-trigger".to_string()])
            } else {
                (c, Some(t.to_string()), Vec::new())
            }
        } else {
            (raw.to_string(), None, vec!["no-trigger".to_string()])
        };

        Some(DebtItem {
            file: file.to_string(),
            line: line_num,
            raw_marker: format!("ponytail: {}", raw),
            ceiling,
            upgrade_trigger: trigger,
            tags,
        })
    }

    /// Number of debt items tagged with "no-trigger".
    pub fn no_trigger_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| i.tags.iter().any(|t| t == "no-trigger"))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harvest_with_and_without_trigger() {
        let diff = r#"
diff --git a/src/cache.rs b/src/cache.rs
index 0000000..1111111 100644
--- a/src/cache.rs
+++ b/src/cache.rs
@@ -10,3 +10,5 @@ fn setup() {
+    // ponytail: in-memory HashMap up to 10k items, migrate to redis if exceeds
+    let cache = HashMap::new();
+    // ponytail: unvalidated buffer
"#;

        let ledger = DebtLedger::harvest_from_diff(diff);
        assert_eq!(ledger.items.len(), 2);

        let item1 = &ledger.items[0];
        assert_eq!(item1.file, "src/cache.rs");
        assert_eq!(item1.ceiling, "in-memory HashMap up to 10k items");
        assert_eq!(
            item1.upgrade_trigger.as_deref(),
            Some("migrate to redis if exceeds")
        );
        assert!(item1.tags.is_empty());

        let item2 = &ledger.items[1];
        assert_eq!(item2.ceiling, "unvalidated buffer");
        assert_eq!(item2.upgrade_trigger, None);
        assert_eq!(item2.tags, vec!["no-trigger".to_string()]);

        assert_eq!(ledger.no_trigger_count(), 1);
    }
}
