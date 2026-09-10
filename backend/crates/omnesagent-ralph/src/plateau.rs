//! Deterministic failure fingerprinting and plateau detection.
//!
//! Conforms to §7.2, §7.5, FR-A7 of specification.

use sha2::{Digest, Sha256};

/// Computes a deterministic SHA-256 fingerprint for failed test output.
///
/// Format: sha256(sorted(failed_test_names) + ":" + exit_code)
pub fn compute_failure_fingerprint(failed_tests: &[String], exit_code: i32) -> String {
    let mut sorted = failed_tests.to_vec();
    sorted.sort();

    let mut hasher = Sha256::new();
    for test_name in &sorted {
        hasher.update(test_name.as_bytes());
        hasher.update(b",");
    }
    hasher.update(format!("exit_code:{}", exit_code).as_bytes());
    let result = hasher.finalize();
    format!("sha256:{:x}", result)
}

/// Evaluates whether a task has hit a failure plateau.
///
/// A plateau is reached when the last `window_size` fingerprints for the same task
/// are identical and non-empty.
pub fn is_plateau(fingerprints: &[&str], window_size: usize) -> bool {
    if window_size == 0 || fingerprints.len() < window_size {
        return false;
    }

    let window = &fingerprints[fingerprints.len() - window_size..];
    let first = window[0];
    if first.is_empty() {
        return false;
    }

    window.iter().all(|&fp| fp == first)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_deterministic() {
        let failures1 = vec!["test_auth".to_string(), "test_db".to_string()];
        let failures2 = vec!["test_db".to_string(), "test_auth".to_string()];

        let fp1 = compute_failure_fingerprint(&failures1, 101);
        let fp2 = compute_failure_fingerprint(&failures2, 101);

        assert_eq!(fp1, fp2);
        assert!(fp1.starts_with("sha256:"));

        let fp_diff_code = compute_failure_fingerprint(&failures1, 1);
        assert_ne!(fp1, fp_diff_code);
    }

    #[test]
    fn test_plateau_detection() {
        let fps = vec!["sha256:1", "sha256:2", "sha256:2", "sha256:2"];
        assert!(is_plateau(&fps, 3));

        let fps2 = vec!["sha256:1", "sha256:2", "sha256:3", "sha256:2"];
        assert!(!is_plateau(&fps2, 3));

        let fps3 = vec!["sha256:2", "sha256:2"];
        assert!(!is_plateau(&fps3, 3)); // Not enough history
    }
}
