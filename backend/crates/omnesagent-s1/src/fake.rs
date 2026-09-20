//! Deterministic Fake System One provider for unit and integration testing.

use std::collections::HashMap;
use std::sync::RwLock;
use crate::contract::{Answer, Question};
use crate::provider::SystemOne;
use anyhow::Result;
use async_trait::async_trait;
use sha2::{Digest, Sha256};

/// A deterministic, in-memory System One stub for testing without weights.
#[derive(Debug, Default)]
pub struct FakeSystemOne {
    canned_answers: RwLock<HashMap<String, Answer>>,
}

impl FakeSystemOne {
    /// Creates a new fake provider.
    #[must_use]
    pub fn new() -> Self {
        Self {
            canned_answers: RwLock::new(HashMap::new()),
        }
    }

    /// Pre-programs a canned answer for a specific question ID.
    pub fn with_canned_answer(self, question_id: impl Into<String>, answer: Answer) -> Self {
        self.canned_answers
            .write()
            .unwrap()
            .insert(question_id.into(), answer);
        self
    }

    /// Sets or overrides a canned answer dynamically.
    pub fn set_canned_answer(&self, question_id: impl Into<String>, answer: Answer) {
        self.canned_answers
            .write()
            .unwrap()
            .insert(question_id.into(), answer);
    }
}

#[async_trait]
impl SystemOne for FakeSystemOne {
    async fn decide(&self, state: &str, questions: &[Question]) -> Result<Vec<Answer>> {
        let canned = self.canned_answers.read().unwrap();
        let mut answers = Vec::with_capacity(questions.len());

        for q in questions {
            if let Some(ans) = canned.get(q.id()) {
                answers.push(ans.clone());
                continue;
            }

            // Fallback to deterministic pseudo-hash response
            let mut hasher = Sha256::new();
            hasher.update(state.as_bytes());
            hasher.update(q.id().as_bytes());
            let hash = hasher.finalize();
            let seed = u64::from_le_bytes(hash[0..8].try_into().unwrap_or_default());

            match q {
                Question::Choice { id, options, .. } => {
                    let num_opts = options.len().max(1);
                    let selected_index = (seed as usize) % num_opts;
                    let selected_value = options
                        .get(selected_index)
                        .cloned()
                        .unwrap_or_default();
                    let mut probabilities = vec![0.0f32; num_opts];
                    probabilities[selected_index] = 1.0;

                    answers.push(Answer::Choice {
                        id: id.clone(),
                        selected_index,
                        selected_value,
                        probabilities,
                        confidence: 0.95,
                    });
                }
                Question::Score { id, levels, .. } => {
                    let max_lvl = (*levels).max(2);
                    let selected_level = ((seed % (max_lvl as u64)) + 1) as u8;
                    let mut probabilities = vec![0.0f32; max_lvl as usize];
                    if let Some(p) = probabilities.get_mut((selected_level - 1) as usize) {
                        *p = 1.0;
                    }

                    answers.push(Answer::Score {
                        id: id.clone(),
                        selected_level,
                        probabilities,
                        confidence: 0.90,
                    });
                }
                Question::Noul { id, .. } => {
                    let prob = ((seed % 1000) as f32) / 1000.0;
                    answers.push(Answer::Noul {
                        id: id.clone(),
                        probability: prob,
                        is_true: prob >= 0.5,
                    });
                }
            }
        }

        Ok(answers)
    }

    fn model_id(&self) -> &str {
        "fake-laya-stub"
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fake_provider_determinism() {
        let fake = FakeSystemOne::new();
        let q = vec![
            Question::choice("q1", "Action", vec!["a".into(), "b".into(), "c".into()]),
            Question::noul("q2", "Is ok?"),
        ];

        let res1 = fake.decide("state test", &q).await.unwrap();
        let res2 = fake.decide("state test", &q).await.unwrap();

        assert_eq!(res1, res2);
    }

    #[tokio::test]
    async fn test_fake_provider_canned_answer() {
        let fake = FakeSystemOne::new().with_canned_answer(
            "q1",
            Answer::Choice {
                id: "q1".into(),
                selected_index: 1,
                selected_value: "b".into(),
                probabilities: vec![0.0, 1.0, 0.0],
                confidence: 1.0,
            },
        );

        let q = vec![Question::choice(
            "q1",
            "Action",
            vec!["a".into(), "b".into(), "c".into()],
        )];
        let res = fake.decide("any state", &q).await.unwrap();
        assert_eq!(res[0].choice_value(), Some("b"));
    }
}
