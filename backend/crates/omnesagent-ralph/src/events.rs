//! Event types broadcast by the Ralph orchestrator to the gateway and WebSocket clients.
//!
//! Conforms to §5.3 of specification.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum RalphEvent {
    RunStarted {
        run_id: String,
        slug: String,
        autonomy: String,
        mode: String,
    },
    GatePending {
        gate_id: String,
        run_id: String,
        slug: String,
        phase: String,
        description: String,
    },
    IterationCompleted {
        task: String,
        n: usize,
        verdict: String,
        ladder_rung: String,
        duration_ms: u64,
    },
    IterationFailed {
        task: String,
        n: usize,
        fingerprint: String,
        exit_code: i32,
    },
    ReviewDeleteList {
        count: usize,
        files_affected: Vec<String>,
    },
    DebtHarvested {
        count: usize,
        no_trigger_count: usize,
    },
    RunStopped {
        run_id: String,
        slug: String,
        reason: String,
    },
    RunResumed {
        run_id: String,
        slug: String,
        manual_commits_detected: bool,
    },
}
