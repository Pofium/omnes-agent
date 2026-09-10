//! Append-only git-versionable journal (`journal.jsonl`) for Ralph.
//!
//! Conforms to §4.2, §7.2 and Appendix B of specification.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JournalError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Execution result reported by the executor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExecutionResult {
    pub what_done: String,
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(default)]
    pub self_assessment: String,
}

/// Summary of test execution results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TestsSummary {
    pub passed: usize,
    pub failed: usize,
    #[serde(default)]
    pub failed_tests: Vec<String>,
    pub fingerprint: String,
    pub exit_code: i32,
}

/// Single iteration record in `journal.jsonl` (Appendix B).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IterationRecord {
    pub task: String,
    pub n: usize,
    pub hypothesis: String,
    pub plan: Vec<String>,
    pub result: ExecutionResult,
    pub tests_summary: TestsSummary,
    pub verdict: String,
    pub ladder_rung: String,
    #[serde(default)]
    pub markers: Vec<String>,
    #[serde(default)]
    pub git_before: Option<String>,
    #[serde(default)]
    pub git_after: Option<String>,
    #[serde(default)]
    pub tokens_used: u64,
    #[serde(default)]
    pub duration_ms: u64,
    pub at: DateTime<Utc>,
}

/// Finding or architectural observation recorded during the cycle (§4.2, §4.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingRecord {
    pub finding_id: String,
    pub task: String,
    pub content: String,
    pub verdict: String, // verified | failed | unconfirmed | stale
    pub symbols: Vec<String>,
    pub at: DateTime<Utc>,
}

/// Envelope for records written to `journal.jsonl`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JournalEntry {
    Iteration(IterationRecord),
    Finding(FindingRecord),
}

/// Append-only journal writer and reader.
pub struct Journal;

impl Journal {
    /// Append an entry to `journal.jsonl`.
    pub fn append(path: &Path, entry: &JournalEntry) -> Result<(), JournalError> {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)?;

        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        let serialized = serde_json::to_string(entry)?;
        writeln!(file, "{}", serialized)?;
        file.sync_data()?;
        Ok(())
    }

    /// Read all entries from `journal.jsonl`.
    pub fn read_all(path: &Path) -> Result<Vec<JournalEntry>, JournalError> {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let entry: JournalEntry = serde_json::from_str(trimmed)?;
            entries.push(entry);
        }
        Ok(entries)
    }

    /// Read only iteration records from `journal.jsonl`.
    pub fn read_iterations(path: &Path) -> Result<Vec<IterationRecord>, JournalError> {
        let all = Self::read_all(path)?;
        Ok(all
            .into_iter()
            .filter_map(|e| match e {
                JournalEntry::Iteration(it) => Some(it),
                _ => None,
            })
            .collect())
    }

    /// Read only findings from `journal.jsonl`.
    pub fn read_findings(path: &Path) -> Result<Vec<FindingRecord>, JournalError> {
        let all = Self::read_all(path)?;
        Ok(all
            .into_iter()
            .filter_map(|e| match e {
                JournalEntry::Finding(f) => Some(f),
                _ => None,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_journal_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("journal.jsonl");

        let it = IterationRecord {
            task: "T-001".to_string(),
            n: 1,
            hypothesis: "Initial impl".to_string(),
            plan: vec!["Write code".to_string()],
            result: ExecutionResult {
                what_done: "Done".to_string(),
                errors: vec![],
                self_assessment: "ok".to_string(),
            },
            tests_summary: TestsSummary {
                passed: 5,
                failed: 0,
                failed_tests: vec![],
                fingerprint: "sha256:abcd".to_string(),
                exit_code: 0,
            },
            verdict: "verified".to_string(),
            ladder_rung: "stdlib: std::fs".to_string(),
            markers: vec!["ponytail: limit 100, upgrade with sqlite".to_string()],
            git_before: Some("abc".to_string()),
            git_after: Some("def".to_string()),
            tokens_used: 1200,
            duration_ms: 45000,
            at: Utc::now(),
        };

        Journal::append(&path, &JournalEntry::Iteration(it.clone())).unwrap();

        let iterations = Journal::read_iterations(&path).unwrap();
        assert_eq!(iterations.len(), 1);
        assert_eq!(iterations[0].task, "T-001");
        assert_eq!(iterations[0].verdict, "verified");
    }
}
