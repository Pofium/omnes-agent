//! State machine and atomic state persistence for Ralph Orchestrator.
//!
//! Conforms to §4.1, §5.5, §7.1, §7.5.1 and Appendix A of specification.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Errors arising from state management and transitions.
#[derive(Debug, Error)]
pub enum StateError {
    #[error("An active run already exists for slug '{0}': ERR_RUN_ACTIVE")]
    RunActive(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Invalid state transition from {0:?} to {1:?}")]
    InvalidTransition(RalphPhase, RalphPhase),
}

/// Lifecycle phases of a Ralph feature run (§7.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RalphPhase {
    Explore,
    Proposed,
    Applying,
    Verifying,
    Archived,
    Stopped,
}

impl RalphPhase {
    /// Check if this phase represents a finished/dormant state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Archived | Self::Stopped)
    }

    /// Check if this phase can transition to target.
    pub fn can_transition_to(&self, next: Self) -> bool {
        if *self == next {
            return true;
        }
        if next == Self::Stopped {
            return true;
        }
        matches!(
            (*self, next),
            (Self::Explore, Self::Proposed)
                | (Self::Proposed, Self::Applying)
                | (Self::Applying, Self::Verifying)
                | (Self::Verifying, Self::Applying) // failure during verify -> applying
                | (Self::Verifying, Self::Archived)
        )
    }
}

/// Fingerprint entry for a failed iteration (§7.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureFingerprint {
    pub task: String,
    pub n: usize,
    pub fp: String,
}

/// Record of a human gate approval/denial (§5.3, §7.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateDecision {
    pub phase: String,
    pub decision: String,
    pub by: String,
    pub at: DateTime<Utc>,
}

/// Complete state structure persisted to `state.json` (Appendix A).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RalphState {
    pub run_id: String,
    pub feature_slug: String,
    pub phase: RalphPhase,
    pub autonomy: String,
    pub mode: String,
    #[serde(default)]
    pub done: Vec<String>,
    #[serde(default)]
    pub blocked: Vec<String>,
    #[serde(default)]
    pub iterations: HashMap<String, usize>,
    #[serde(default)]
    pub fingerprints: Vec<FailureFingerprint>,
    #[serde(default)]
    pub gates: Vec<GateDecision>,
    #[serde(default)]
    pub degraded: bool,
    #[serde(default)]
    pub last_known_head: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl RalphState {
    /// Create a new RalphState in Explore or Proposed phase.
    pub fn new(run_id: String, feature_slug: String, autonomy: String, mode: String) -> Self {
        Self {
            run_id,
            feature_slug,
            phase: RalphPhase::Explore,
            autonomy,
            mode,
            done: Vec::new(),
            blocked: Vec::new(),
            iterations: HashMap::new(),
            fingerprints: Vec::new(),
            gates: Vec::new(),
            degraded: false,
            last_known_head: None,
            updated_at: Utc::now(),
        }
    }

    /// Read state from the specified path.
    pub fn read_from(path: &Path) -> Result<Self, StateError> {
        let content = std::fs::read_to_string(path)?;
        let state: Self = serde_json::from_str(&content)?;
        Ok(state)
    }

    /// Atomic write to `state.json` via temporary file and rename (ADR-A4, §4.1).
    pub fn write_atomic(&mut self, path: &Path) -> Result<(), StateError> {
        self.updated_at = Utc::now();
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)?;

        let tmp_path = parent.join(format!(
            ".{}.tmp.{}",
            path.file_name().unwrap_or_default().to_string_lossy(),
            std::process::id()
        ));

        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&tmp_path, data)?;
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }

    /// Ensure concurrency isolation: fail if an active run exists for slug (§5.5).
    pub fn check_concurrency(state_path: &Path, slug: &str) -> Result<(), StateError> {
        if state_path.exists() {
            if let Ok(existing) = Self::read_from(state_path) {
                if !existing.phase.is_terminal() {
                    return Err(StateError::RunActive(slug.to_string()));
                }
            }
        }
        Ok(())
    }

    /// Transition to a new phase, enforcing valid graph transitions (§7.1).
    pub fn transition_to(&mut self, next: RalphPhase) -> Result<(), StateError> {
        if !self.phase.can_transition_to(next) {
            return Err(StateError::InvalidTransition(self.phase, next));
        }
        self.phase = next;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if state needs resume (§7.5.1).
    pub fn can_resume(&self) -> bool {
        matches!(self.phase, RalphPhase::Applying | RalphPhase::Verifying)
    }

    /// Mark task completed.
    pub fn mark_done(&mut self, task_id: &str) {
        if !self.done.contains(&task_id.to_string()) {
            self.done.push(task_id.to_string());
        }
        self.blocked.retain(|id| id != task_id);
    }

    /// Mark task blocked.
    pub fn mark_blocked(&mut self, task_id: &str) {
        if !self.blocked.contains(&task_id.to_string()) {
            self.blocked.push(task_id.to_string());
        }
    }

    /// Get current iteration count for a task.
    pub fn get_iterations(&self, task_id: &str) -> usize {
        self.iterations.get(task_id).copied().unwrap_or(0)
    }

    /// Increment and return new iteration count for a task.
    pub fn increment_iteration(&mut self, task_id: &str) -> usize {
        let n = self.get_iterations(task_id) + 1;
        self.iterations.insert(task_id.to_string(), n);
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_state_atomic_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");

        let mut state = RalphState::new(
            "run-123".to_string(),
            "test-feature".to_string(),
            "L1".to_string(),
            "full".to_string(),
        );
        state.write_atomic(&path).unwrap();

        let loaded = RalphState::read_from(&path).unwrap();
        assert_eq!(loaded.run_id, "run-123");
        assert_eq!(loaded.phase, RalphPhase::Explore);
        assert!(!loaded.can_resume());
    }

    #[test]
    fn test_phase_transitions() {
        let mut state = RalphState::new(
            "run-1".to_string(),
            "f".to_string(),
            "L1".to_string(),
            "full".to_string(),
        );
        assert!(state.transition_to(RalphPhase::Proposed).is_ok());
        assert!(state.transition_to(RalphPhase::Applying).is_ok());
        assert!(state.can_resume());
        assert!(state.transition_to(RalphPhase::Verifying).is_ok());
        assert!(state.transition_to(RalphPhase::Archived).is_ok());
        assert!(!state.can_resume());
    }

    #[test]
    fn test_concurrency_protection() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");

        let mut state = RalphState::new(
            "run-1".to_string(),
            "f".to_string(),
            "L1".to_string(),
            "full".to_string(),
        );
        state.phase = RalphPhase::Applying;
        state.write_atomic(&path).unwrap();

        let err = RalphState::check_concurrency(&path, "f").unwrap_err();
        match err {
            StateError::RunActive(s) => assert_eq!(s, "f"),
            _ => panic!("Expected RunActive error"),
        }

        // Terminal phase allows new run
        state.phase = RalphPhase::Archived;
        state.write_atomic(&path).unwrap();
        assert!(RalphState::check_concurrency(&path, "f").is_ok());
    }
}
