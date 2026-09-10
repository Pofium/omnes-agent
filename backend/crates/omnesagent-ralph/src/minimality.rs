//! Ponytail Minimality Review: generates and safely applies delete-lists before commits.
//!
//! Conforms to §2 (#6), §7.2.1, FR-A5 of specification.

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MinimalityError {
    #[error("I/O error applying delete-list: {0}")]
    Io(#[from] std::io::Error),
    #[error("Violates carve-out: {0}")]
    CarveOutViolation(String),
}

/// Protected categories that a delete-list must never touch (§7.2.1).
pub const CARVE_OUT_KEYWORDS: &[&str] = &[
    "trust",
    "boundary",
    "auth",
    "sanitize",
    "validate",
    "security",
    "permission",
    "dataloss",
    "data_loss",
    "rollback",
    "transaction",
    "accessibility",
    "a11y",
];

/// A single deletion or simplification proposed by the minimality review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteItem {
    pub file: String,
    pub target_pattern: String,
    pub rationale: String,
}

/// A structured delete-list output by minimality review.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DeleteList {
    pub items: Vec<DeleteItem>,
}

impl DeleteList {
    /// Return whether this delete list has actionable items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Check whether an item violates any protected carve-outs (§7.2.1).
    pub fn is_carve_out(item: &DeleteItem) -> bool {
        let text_lower = format!("{} {}", item.file, item.rationale).to_lowercase();
        let pattern_lower = item.target_pattern.to_lowercase();

        for &kw in CARVE_OUT_KEYWORDS {
            if text_lower.contains(kw) || pattern_lower.contains(kw) {
                return true;
            }
        }
        false
    }

    /// Filter out any proposed deletions that collide with carve-outs.
    pub fn sanitize_carve_outs(&mut self) -> Vec<DeleteItem> {
        let mut removed = Vec::new();
        self.items.retain(|item| {
            if Self::is_carve_out(item) {
                removed.push(item.clone());
                false
            } else {
                true
            }
        });
        removed
    }

    /// Apply clean line deletions to the specified repository root.
    pub fn apply(&self, root: &Path) -> Result<usize, MinimalityError> {
        let mut applied_count = 0;

        for item in &self.items {
            if Self::is_carve_out(item) {
                continue;
            }

            let file_path = root.join(&item.file);
            if !file_path.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&file_path)?;
            if content.contains(&item.target_pattern) {
                let updated = content.replace(&item.target_pattern, "");
                std::fs::write(&file_path, updated)?;
                applied_count += 1;
            }
        }

        Ok(applied_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_carve_out_filtering() {
        let mut dl = DeleteList {
            items: vec![
                DeleteItem {
                    file: "src/util.rs".to_string(),
                    target_pattern: "// unnecessary helper\nfn foo() {}".to_string(),
                    rationale: "Unused abstraction".to_string(),
                },
                DeleteItem {
                    file: "src/auth.rs".to_string(),
                    target_pattern: "fn validate_input() {}".to_string(),
                    rationale: "Security validation boundary".to_string(),
                },
            ],
        };

        let removed = dl.sanitize_carve_outs();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].file, "src/auth.rs");
        assert_eq!(dl.items.len(), 1);
        assert_eq!(dl.items[0].file, "src/util.rs");
    }

    #[test]
    fn test_apply_delete_list() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("code.rs");
        std::fs::write(&file_path, "line1\nline_to_remove\nline3\n").unwrap();

        let dl = DeleteList {
            items: vec![DeleteItem {
                file: "code.rs".to_string(),
                target_pattern: "line_to_remove\n".to_string(),
                rationale: "Dead code".to_string(),
            }],
        };

        let count = dl.apply(dir.path()).unwrap();
        assert_eq!(count, 1);

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "line1\nline3\n");
    }
}
