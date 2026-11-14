//! Append-only git-versionable journal (`journal.jsonl`) for Ralph.
//!
//! Conforms to §4.2, §7.2 and Appendix B of specification.

use crate::ast::SymbolChange;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
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
    /// Who produced `verdict` (ADR-K4): `auto_tests` | `human` | `dream`.
    #[serde(default)]
    pub verdict_source: String,
    pub ladder_rung: String,
    #[serde(default)]
    pub markers: Vec<String>,
    /// Symbol-level changes produced by this iteration (Ф3.2), from the fork AST.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ast_changes: Vec<SymbolChange>,
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
    #[serde(default)]
    pub git_head_sha: Option<String>,
    pub at: DateTime<Utc>,
}

impl FindingRecord {
    /// Проверяет свежесть вывода против текущего git HEAD SHA.
    pub fn is_fresh(&self, current_head: Option<&str>) -> bool {
        if self.verdict == "stale" {
            return false;
        }
        match (&self.git_head_sha, current_head) {
            (Some(saved), Some(current)) => saved == current,
            _ => true,
        }
    }
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

    /// Append a finding to `journal.jsonl`.
    pub fn append_finding(path: &Path, finding: &FindingRecord) -> Result<(), JournalError> {
        Self::append(path, &JournalEntry::Finding(finding.clone()))
    }

    /// Read findings, resolving the latest version of each `finding_id`.
    ///
    /// The journal is append-only, so a `stale` re-statement of an earlier
    /// finding is a later entry with the same id — the reader collapses those
    /// to the most recent record while preserving first-seen order.
    pub fn read_findings(path: &Path) -> Result<Vec<FindingRecord>, JournalError> {
        let all = Self::read_all(path)?;
        let mut order: Vec<String> = Vec::new();
        let mut latest: BTreeMap<String, FindingRecord> = BTreeMap::new();
        for entry in all {
            if let JournalEntry::Finding(f) = entry {
                if !latest.contains_key(&f.finding_id) {
                    order.push(f.finding_id.clone());
                }
                latest.insert(f.finding_id.clone(), f);
            }
        }
        Ok(order
            .into_iter()
            .filter_map(|id| latest.remove(&id))
            .collect())
    }
}

/// Findings invalidated by a symbol change, re-stated as `stale` (Ф3.3, ADR-K5).
///
/// A finding is invalidated when any of its `symbols` resolves to a label that
/// was modified or removed (`changed`). Symbols may be bare labels (`alpha`) or
/// prefixed paths (`fn:alpha`); comparison uses the last `:`-segment. Findings
/// already marked `stale` are ignored, so the pass is idempotent.
pub fn stale_findings(
    findings: &[FindingRecord],
    changed: &BTreeSet<String>,
) -> Vec<FindingRecord> {
    if changed.is_empty() {
        return Vec::new();
    }
    findings
        .iter()
        .filter(|finding| finding.verdict != "stale")
        .filter(|finding| {
            finding.symbols.iter().any(|symbol| {
                let label = symbol.rsplit(':').next().unwrap_or(symbol);
                changed.contains(label)
            })
        })
        .map(|finding| {
            let mut updated = finding.clone();
            updated.verdict = "stale".to_string();
            updated
        })
        .collect()
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
            verdict_source: "auto_tests".to_string(),
            ladder_rung: "stdlib: std::fs".to_string(),
            markers: vec!["ponytail: limit 100, upgrade with sqlite".to_string()],
            ast_changes: vec![],
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

    fn finding(id: &str, verdict: &str, symbols: &[&str]) -> FindingRecord {
        FindingRecord {
            finding_id: id.to_string(),
            task: "T-001".to_string(),
            content: format!("finding {id}"),
            verdict: verdict.to_string(),
            symbols: symbols.iter().map(|s| s.to_string()).collect(),
            git_head_sha: None,
            at: Utc::now(),
        }
    }

    #[test]
    fn read_findings_resolves_latest_version_per_id() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("journal.jsonl");

        Journal::append_finding(&path, &finding("f1", "verified", &["alpha"])).unwrap();
        Journal::append_finding(&path, &finding("f2", "verified", &["beta"])).unwrap();
        // Later re-statement of f1 as stale (append-only journal).
        Journal::append_finding(&path, &finding("f1", "stale", &["alpha"])).unwrap();

        let findings = Journal::read_findings(&path).unwrap();
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].finding_id, "f1");
        assert_eq!(findings[0].verdict, "stale");
        assert_eq!(findings[1].finding_id, "f2");
    }

    #[test]
    fn stale_findings_marks_only_matching_symbols() {
        let findings = vec![
            finding("f1", "verified", &["fn:alpha"]),
            finding("f2", "verified", &["beta"]),
            finding("f3", "stale", &["alpha"]),
        ];
        let changed = BTreeSet::from(["alpha".to_string()]);

        let stale = stale_findings(&findings, &changed);
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0].finding_id, "f1");
        assert_eq!(stale[0].verdict, "stale");
    }

    #[test]
    fn stale_findings_is_a_noop_without_changes() {
        let findings = vec![finding("f1", "verified", &["alpha"])];
        assert!(stale_findings(&findings, &BTreeSet::new()).is_empty());
    }
}
