//! Audit logger for System One decisions.
//!
//! Conforms to repository privacy requirements by persisting a SHA-256 hash of the
//! input context rather than plaintext content.

use std::path::PathBuf;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use crate::contract::{Answer, Question};

/// An audit entry representing a recorded decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLogEntry {
    pub timestamp: String,
    pub caller: String,
    pub state_hash: String,
    pub question_id: String,
    pub question_type: String,
    pub answer: Answer,
    pub duration_us: u64,
    pub model_id: String,
}

/// Logs a decision event asynchronously to `data/s1/decisions.jsonl`.
pub async fn log_decision(
    caller: &str,
    state: &str,
    question: &Question,
    answer: &Answer,
    duration_us: u64,
    model_id: &str,
) {
    let mut hasher = Sha256::new();
    hasher.update(state.as_bytes());
    let state_hash = hex::encode(hasher.finalize());

    let question_type = match question {
        Question::Choice { .. } => "choice",
        Question::Score { .. } => "score",
        Question::Noul { .. } => "noul",
    }.to_string();

    let entry = DecisionLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        caller: caller.to_string(),
        state_hash,
        question_id: question.id().to_string(),
        question_type,
        answer: answer.clone(),
        duration_us,
        model_id: model_id.to_string(),
    };

    let log_path = PathBuf::from("data/s1/decisions.jsonl");
    if let Some(parent) = log_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }

    let serialized = match serde_json::to_string(&entry) {
        Ok(s) => s,
        Err(_) => return,
    };

    if let Ok(mut file) = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .await
    {
        let mut line = serialized;
        line.push('\n');
        let _ = file.write_all(line.as_bytes()).await;
    }
}
