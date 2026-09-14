//! Regression benchmark gate and golden-set evaluation (OB2H v1.4 Ф30 + v1.3 Ф21).
//!
//! Evaluates retrieval quality against golden test cases using:
//! - recall@k: fraction of expected memory keys found in top-k results
//! - MRR: Mean Reciprocal Rank of the first relevant memory key
//! - is_degradation: checks if relative drop in recall@5 > 10% OR MRR > 15%
//! - GateVerdict: decision engine for auto-rollback or pass after dream/consolidation phases.

use std::collections::HashSet;
use serde::{Deserialize, Serialize};

/// Number of quick golden cases used in nightly bench-gate.
pub const QUICK_CASES: usize = 15;
/// Maximum tolerable relative drop for recall@5 before triggering rollback (10%).
pub const RECALL_DROP_LIMIT: f64 = 0.10;
/// Maximum tolerable relative drop for MRR before triggering rollback (15%).
pub const MRR_DROP_LIMIT: f64 = 0.15;

/// A single golden evaluation test case.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldenCase {
    /// Search query or context prompt
    pub query: String,
    /// Expected memory keys that must appear in the retrieved results
    pub expect_keys: Vec<String>,
    /// Optional developer note or category
    #[serde(default)]
    pub note: String,
}

/// Parse a newline-delimited JSON (JSONL) string of golden cases.
pub fn parse_golden_jsonl(text: &str) -> anyhow::Result<Vec<GoldenCase>> {
    let mut cases = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let case: GoldenCase = serde_json::from_str(trimmed)
            .map_err(|e| anyhow::anyhow!("Invalid golden case at line {}: {e}", i + 1))?;
        if case.query.trim().is_empty() {
            anyhow::bail!("Golden case at line {} has empty query", i + 1);
        }
        if case.expect_keys.is_empty() {
            anyhow::bail!("Golden case at line {} has empty expect_keys", i + 1);
        }
        cases.push(case);
    }
    Ok(cases)
}

/// Compute recall@k: the fraction of expected keys found in the top-k retrieved keys.
pub fn recall_at_k(ranked: &[String], expected: &HashSet<String>, k: usize) -> f64 {
    if expected.is_empty() {
        return 0.0;
    }
    let hits = ranked
        .iter()
        .take(k)
        .filter(|key| expected.contains(*key))
        .count();
    hits as f64 / expected.len() as f64
}

/// Compute MRR (Mean Reciprocal Rank): 1 / position of the first matching expected key (1-based).
/// Returns 0.0 if no expected key is found in the ranked list.
pub fn mrr(ranked: &[String], expected: &HashSet<String>) -> f64 {
    for (idx, key) in ranked.iter().enumerate() {
        if expected.contains(key) {
            return 1.0 / (idx + 1) as f64;
        }
    }
    0.0
}

/// Nearest-rank percentile calculation (p in range [0, 100]).
pub fn percentile(durations_ms: &mut [f64], p: f64) -> f64 {
    if durations_ms.is_empty() {
        return 0.0;
    }
    durations_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((p / 100.0) * durations_ms.len() as f64).ceil() as usize;
    let idx = idx.clamp(1, durations_ms.len());
    durations_ms[idx - 1]
}

/// Check whether current metrics represent a degradation against previous baseline.
/// Returns true if relative drop of recall@5 > 10% OR relative drop of MRR > 15%.
/// Note: a drop in MRR alone with steady recall is also treated as degradation.
pub fn is_degradation(prev: (f64, f64), cur: (f64, f64)) -> bool {
    let (prev_recall, prev_mrr) = prev;
    let (cur_recall, cur_mrr) = cur;

    let recall_drop =
        prev_recall > f64::EPSILON && (prev_recall - cur_recall) / prev_recall > RECALL_DROP_LIMIT;
    let mrr_drop = prev_mrr > f64::EPSILON && (prev_mrr - cur_mrr) / prev_mrr > MRR_DROP_LIMIT;

    recall_drop || mrr_drop
}

/// Summary metrics computed over a set of benchmark test cases.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchSummary {
    pub cases: usize,
    pub recall5: f64,
    pub recall10: f64,
    pub mrr: f64,
    pub p95_ms: f64,
    pub empty_count: usize,
}

/// Compute evaluation metrics given a slice of golden cases and pre-ranked results.
pub fn evaluate_ranked_results(
    cases: &[GoldenCase],
    ranked_results: &[Vec<String>],
    durations_ms: &[f64],
) -> BenchSummary {
    if cases.is_empty() || ranked_results.is_empty() {
        return BenchSummary {
            cases: 0,
            recall5: 0.0,
            recall10: 0.0,
            mrr: 0.0,
            p95_ms: 0.0,
            empty_count: 0,
        };
    }

    let n = cases.len().min(ranked_results.len());
    let mut sum_recall5 = 0.0;
    let mut sum_recall10 = 0.0;
    let mut sum_mrr = 0.0;
    let mut empty_count = 0;

    for i in 0..n {
        let case = &cases[i];
        let ranked = &ranked_results[i];
        if ranked.is_empty() {
            empty_count += 1;
        }
        let expected: HashSet<String> = case.expect_keys.iter().cloned().collect();
        sum_recall5 += recall_at_k(ranked, &expected, 5);
        sum_recall10 += recall_at_k(ranked, &expected, 10);
        sum_mrr += mrr(ranked, &expected);
    }

    let mut durs = durations_ms.to_vec();
    let p95 = percentile(&mut durs, 95.0);

    BenchSummary {
        cases: n,
        recall5: sum_recall5 / n as f64,
        recall10: sum_recall10 / n as f64,
        mrr: sum_mrr / n as f64,
        p95_ms: p95,
        empty_count,
    }
}

/// The outcome and verdict of a regression benchmark gate check.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateVerdict {
    /// "pass" | "first_run" | "rollback" | "timeout" | "skipped"
    pub outcome: String,
    pub reason: Option<String>,
    pub cases: usize,
    pub recall5: Option<f64>,
    pub recall10: Option<f64>,
    pub mrr: Option<f64>,
    pub p95_ms: Option<f64>,
    pub prev_recall5: Option<f64>,
    pub prev_mrr: Option<f64>,
    pub elapsed_ms: u64,
    pub restored_to: Option<String>,
    pub alert: Option<String>,
}

impl GateVerdict {
    /// Construct a verdict when the gate is skipped (e.g. golden set missing).
    pub fn skipped(reason: &str) -> Self {
        Self {
            outcome: "skipped".to_string(),
            reason: Some(reason.to_string()),
            cases: 0,
            recall5: None,
            recall10: None,
            mrr: None,
            p95_ms: None,
            prev_recall5: None,
            prev_mrr: None,
            elapsed_ms: 0,
            restored_to: None,
            alert: None,
        }
    }

    /// Construct a verdict when the gate execution times out.
    /// FAIL-SAFE: Timeout does NOT trigger rollback (no actions on incomplete data).
    pub fn timeout(timeout_ms: u64, cases: usize) -> Self {
        Self {
            outcome: "timeout".to_string(),
            reason: Some(format!("Gate execution exceeded budget of {timeout_ms} ms")),
            cases,
            recall5: None,
            recall10: None,
            mrr: None,
            p95_ms: None,
            prev_recall5: None,
            prev_mrr: None,
            elapsed_ms: timeout_ms,
            restored_to: None,
            alert: Some("Gate timed out; skipping decision without rollback".to_string()),
        }
    }
}

/// Make a gate decision comparing current benchmark summary with prior baseline.
pub fn evaluate_gate_decision(
    summary: &BenchSummary,
    prev_baseline: Option<(f64, f64)>,
    snapshot_id: Option<&str>,
    dream_id: Option<&str>,
    elapsed_ms: u64,
) -> GateVerdict {
    let cur = (summary.recall5, summary.mrr);
    let (outcome, alert, restored_to) = match prev_baseline {
        None => ("first_run".to_string(), None, None),
        Some(prev) => {
            if is_degradation(prev, cur) {
                let target = snapshot_id.map(|s| s.to_string());
                let alert_msg = format!(
                    "ROLLBACK: dream {} degraded recall@5 from {:.3} to {:.3} or MRR from {:.3} to {:.3}; restoring to {}",
                    dream_id.unwrap_or("unknown"),
                    prev.0,
                    cur.0,
                    prev.1,
                    cur.1,
                    snapshot_id.unwrap_or("baseline")
                );
                ("rollback".to_string(), Some(alert_msg), target)
            } else {
                ("pass".to_string(), None, None)
            }
        }
    };

    GateVerdict {
        outcome,
        reason: None,
        cases: summary.cases,
        recall5: Some(summary.recall5),
        recall10: Some(summary.recall10),
        mrr: Some(summary.mrr),
        p95_ms: Some(summary.p95_ms),
        prev_recall5: prev_baseline.map(|p| p.0),
        prev_mrr: prev_baseline.map(|p| p.1),
        elapsed_ms,
        restored_to,
        alert,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_golden_jsonl() {
        let jsonl = r#"
{"query": "database connection settings", "expect_keys": ["db_config", "env_setup"], "note": "config test"}
# This is a comment
{"query": "authentication jwt token", "expect_keys": ["auth_jwt"]}
"#;
        let cases = parse_golden_jsonl(jsonl).expect("parse should succeed");
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].query, "database connection settings");
        assert_eq!(cases[0].expect_keys, vec!["db_config", "env_setup"]);
        assert_eq!(cases[0].note, "config test");
        assert_eq!(cases[1].query, "authentication jwt token");
        assert_eq!(cases[1].expect_keys, vec!["auth_jwt"]);
    }

    #[test]
    fn test_recall_at_k() {
        let ranked = vec![
            "doc1".to_string(),
            "doc2".to_string(),
            "doc3".to_string(),
            "doc4".to_string(),
        ];
        let mut expected = HashSet::new();
        expected.insert("doc1".to_string());
        expected.insert("doc3".to_string());

        // At k=1: only doc1 found -> 1/2 = 0.5
        assert!((recall_at_k(&ranked, &expected, 1) - 0.5).abs() < f64::EPSILON);
        // At k=2: only doc1 found -> 1/2 = 0.5
        assert!((recall_at_k(&ranked, &expected, 2) - 0.5).abs() < f64::EPSILON);
        // At k=3: doc1 and doc3 found -> 2/2 = 1.0
        assert!((recall_at_k(&ranked, &expected, 3) - 1.0).abs() < f64::EPSILON);
        // Empty expected -> 0.0
        assert_eq!(recall_at_k(&ranked, &HashSet::new(), 3), 0.0);
    }

    #[test]
    fn test_mrr() {
        let ranked = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ];

        let mut exp_a = HashSet::new();
        exp_a.insert("a".to_string());
        assert!((mrr(&ranked, &exp_a) - 1.0).abs() < f64::EPSILON);

        let mut exp_b = HashSet::new();
        exp_b.insert("b".to_string());
        assert!((mrr(&ranked, &exp_b) - 0.5).abs() < f64::EPSILON);

        let mut exp_c = HashSet::new();
        exp_c.insert("c".to_string());
        assert!((mrr(&ranked, &exp_c) - (1.0 / 3.0)).abs() < 1e-5);

        let mut exp_none = HashSet::new();
        exp_none.insert("z".to_string());
        assert_eq!(mrr(&ranked, &exp_none), 0.0);
    }

    #[test]
    fn test_is_degradation_thresholds() {
        let baseline = (0.80, 0.70);

        // Case 1: minor fluctuations within tolerance (recall drop 5%, mrr drop 5%) -> not degradation
        let cur_ok = (0.76, 0.67);
        assert!(!is_degradation(baseline, cur_ok));

        // Case 2: recall drops > 10% (0.80 -> 0.71, drop is 11.25%) -> degradation!
        let cur_recall_drop = (0.71, 0.70);
        assert!(is_degradation(baseline, cur_recall_drop));

        // Case 3: recall steady, but MRR drops > 15% (0.70 -> 0.59, drop is 15.7%) -> degradation!
        let cur_mrr_drop = (0.80, 0.59);
        assert!(is_degradation(baseline, cur_mrr_drop));

        // Case 4: improvements -> not degradation
        let cur_better = (0.85, 0.75);
        assert!(!is_degradation(baseline, cur_better));
    }

    #[test]
    fn test_gate_verdict_decision_flow() {
        let summary_good = BenchSummary {
            cases: 15,
            recall5: 0.85,
            recall10: 0.95,
            mrr: 0.78,
            p95_ms: 45.0,
            empty_count: 0,
        };

        // First run (no baseline)
        let v1 = evaluate_gate_decision(&summary_good, None, None, None, 120);
        assert_eq!(v1.outcome, "first_run");
        assert!(v1.alert.is_none());

        // Passing run
        let v2 = evaluate_gate_decision(&summary_good, Some((0.84, 0.77)), None, None, 130);
        assert_eq!(v2.outcome, "pass");
        assert!(v2.alert.is_none());

        // Degraded run triggering rollback
        let summary_degraded = BenchSummary {
            cases: 15,
            recall5: 0.65, // dropped from 0.84 (>10%)
            recall10: 0.75,
            mrr: 0.55,    // dropped from 0.77 (>15%)
            p95_ms: 50.0,
            empty_count: 2,
        };
        let v3 = evaluate_gate_decision(
            &summary_degraded,
            Some((0.84, 0.77)),
            Some("snap_before_dream_123"),
            Some("dream_456"),
            140,
        );
        assert_eq!(v3.outcome, "rollback");
        assert_eq!(v3.restored_to, Some("snap_before_dream_123".to_string()));
        assert!(v3.alert.is_some());
        assert!(v3.alert.unwrap().contains("ROLLBACK"));

        // Timeout fail-safe verdict: does not rollback
        let v_timeout = GateVerdict::timeout(3000, 15);
        assert_eq!(v_timeout.outcome, "timeout");
        assert_eq!(v_timeout.restored_to, None);
    }
}
