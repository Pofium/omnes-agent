//! System One provider trait and fallback implementation.

use std::sync::Arc;
use crate::contract::{Answer, Question};
use anyhow::Result;
use async_trait::async_trait;

/// Core interface for typed System One inference.
#[async_trait]
pub trait SystemOne: Send + Sync {
    /// Answers a batch of typed questions given an input state text.
    async fn decide(&self, state: &str, questions: &[Question]) -> Result<Vec<Answer>>;

    /// Returns the canonical model identifier (e.g. "convaiinnovations/laya" or "fake").
    fn model_id(&self) -> &str;

    /// Whether this provider is available and healthy.
    fn is_available(&self) -> bool {
        true
    }
}

/// Fallback / Disabled provider that returns empty answers or fails closed.
#[derive(Debug, Default, Clone)]
pub struct NoOpSystemOne;

#[async_trait]
impl SystemOne for NoOpSystemOne {
    async fn decide(&self, _state: &str, _questions: &[Question]) -> Result<Vec<Answer>> {
        anyhow::bail!("System One provider is disabled or unavailable")
    }

    fn model_id(&self) -> &str {
        "disabled"
    }

    fn is_available(&self) -> bool {
        false
    }
}

/// A thread-safe shared handle to a SystemOne provider.
pub type SharedSystemOne = Arc<dyn SystemOne>;
