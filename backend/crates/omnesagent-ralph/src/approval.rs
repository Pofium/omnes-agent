//! Human gates and approval management for Ralph Orchestrator.
//!
//! Conforms to §5.3, §7.1, FR-A8 of specification.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

/// Gate request waiting for human intervention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingGate {
    pub gate_id: String,
    pub run_id: String,
    pub slug: String,
    pub phase: String,
    pub description: String,
    pub requested_at: chrono::DateTime<Utc>,
}

/// Human decision on a pending gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GateAction {
    Approve,
    Deny,
}

/// In-memory manager for pending gates.
#[derive(Default, Clone)]
pub struct GateManager {
    pending: Arc<Mutex<HashMap<String, PendingGate>>>,
    decisions: Arc<Mutex<HashMap<String, (GateAction, String)>>>, // gate_id -> (decision, by)
    notifier: Arc<Notify>,
}

impl GateManager {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
            decisions: Arc::new(Mutex::new(HashMap::new())),
            notifier: Arc::new(Notify::new()),
        }
    }

    /// Register a pending gate and await resolution.
    pub async fn request_gate(
        &self,
        gate_id: &str,
        run_id: &str,
        slug: &str,
        phase: &str,
        description: &str,
    ) -> (GateAction, String) {
        let gate = PendingGate {
            gate_id: gate_id.to_string(),
            run_id: run_id.to_string(),
            slug: slug.to_string(),
            phase: phase.to_string(),
            description: description.to_string(),
            requested_at: Utc::now(),
        };

        {
            let mut p = self.pending.lock().await;
            p.insert(gate_id.to_string(), gate);
        }

        loop {
            {
                let mut d = self.decisions.lock().await;
                if let Some(decision) = d.remove(gate_id) {
                    let mut p = self.pending.lock().await;
                    p.remove(gate_id);
                    return decision;
                }
            }
            self.notifier.notified().await;
        }
    }

    /// List all currently pending gates.
    pub async fn list_pending(&self) -> Vec<PendingGate> {
        let p = self.pending.lock().await;
        p.values().cloned().collect()
    }

    /// Submit a human decision for a gate.
    pub async fn resolve_gate(&self, gate_id: &str, action: GateAction, by: &str) -> bool {
        let is_pending = {
            let p = self.pending.lock().await;
            p.contains_key(gate_id)
        };

        if is_pending {
            let mut d = self.decisions.lock().await;
            d.insert(gate_id.to_string(), (action, by.to_string()));
            self.notifier.notify_waiters();
            true
        } else {
            false
        }
    }
}
