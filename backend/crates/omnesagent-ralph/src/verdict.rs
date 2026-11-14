//! Objective iteration verdicts for Ralph (Ф3.4, ADR-K4 ob2h parity).
//!
//! The iteration verdict is derived exclusively from objective test signals
//! ([`TestsSummary`]). The executor's free-form `self_assessment` is *never*
//! consulted — mirrors ob2h's `auto_verdict` contract, where the agent's own
//! optimism cannot promote an iteration to `verified`.

use crate::journal::TestsSummary;

/// Objective verdict label plus the source that produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verdict {
    /// `verified` | `failed` | `unconfirmed`.
    pub label: &'static str,
    /// Always `auto_tests` for machine-derived verdicts (ADR-K4).
    pub source: &'static str,
}

/// Compute the iteration verdict from objective test signals only (ADR-K4).
///
/// - any failing test (or non-zero exit collapsed into a failure) → `failed`
/// - all green with at least one passing check → `verified`
/// - exit code 0 but no observable checks → `unconfirmed`
pub fn auto_verdict(tests: &TestsSummary) -> Verdict {
    if tests.failed > 0 {
        Verdict {
            label: "failed",
            source: "auto_tests",
        }
    } else if tests.passed > 0 {
        Verdict {
            label: "verified",
            source: "auto_tests",
        }
    } else {
        Verdict {
            label: "unconfirmed",
            source: "auto_tests",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(passed: usize, failed: usize) -> TestsSummary {
        TestsSummary {
            passed,
            failed,
            failed_tests: Vec::new(),
            fingerprint: "fp".to_string(),
            exit_code: if failed > 0 { 1 } else { 0 },
        }
    }

    #[test]
    fn failing_tests_verdict_is_failed() {
        let verdict = auto_verdict(&summary(3, 1));
        assert_eq!(verdict.label, "failed");
        assert_eq!(verdict.source, "auto_tests");
    }

    #[test]
    fn passing_tests_verdict_is_verified() {
        let verdict = auto_verdict(&summary(2, 0));
        assert_eq!(verdict.label, "verified");
    }

    #[test]
    fn no_observable_checks_verdict_is_unconfirmed() {
        let verdict = auto_verdict(&summary(0, 0));
        assert_eq!(verdict.label, "unconfirmed");
    }

    #[test]
    fn failure_dominates_even_when_some_tests_pass() {
        // ADR-K4: a single objective failure is enough — self-assessment is irrelevant.
        assert_eq!(auto_verdict(&summary(10, 1)).label, "failed");
    }
}
