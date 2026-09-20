//! Typed System One decision contract (Choice, Score, Noul).
//!
//! Compatible with the TypeSafe / Laya SDK decision specification.

use serde::{Deserialize, Serialize};

/// A typed query posed to the System One model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Question {
    /// Choice between 2 and 26 discrete string options.
    Choice {
        id: String,
        prompt: String,
        options: Vec<String>,
    },
    /// Discretized score from 1 up to `levels` (between 2 and 8).
    Score {
        id: String,
        prompt: String,
        levels: u8,
    },
    /// Calibrated binary probability $P(\text{true}) \in [0.0, 1.0]$.
    Noul {
        id: String,
        prompt: String,
    },
}

impl Question {
    /// Helper constructor for a Choice question.
    pub fn choice(id: impl Into<String>, prompt: impl Into<String>, options: Vec<String>) -> Self {
        Self::Choice {
            id: id.into(),
            prompt: prompt.into(),
            options,
        }
    }

    /// Helper constructor for a Score question.
    pub fn score(id: impl Into<String>, prompt: impl Into<String>, levels: u8) -> Self {
        Self::Score {
            id: id.into(),
            prompt: prompt.into(),
            levels: levels.clamp(2, 8),
        }
    }

    /// Helper constructor for a Noul (binary probability) question.
    pub fn noul(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self::Noul {
            id: id.into(),
            prompt: prompt.into(),
        }
    }

    /// Returns the unique identifier of the question.
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Choice { id, .. } | Self::Score { id, .. } | Self::Noul { id, .. } => id,
        }
    }

    /// Returns the prompt text of the question.
    #[must_use]
    pub fn prompt(&self) -> &str {
        match self {
            Self::Choice { prompt, .. }
            | Self::Score { prompt, .. }
            | Self::Noul { prompt, .. } => prompt,
        }
    }
}

/// A typed decision returned by the System One model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Answer {
    /// Choice resolution with selected option and probability distribution.
    Choice {
        id: String,
        selected_index: usize,
        selected_value: String,
        probabilities: Vec<f32>,
        confidence: f32,
    },
    /// Score resolution with chosen level and probability distribution.
    Score {
        id: String,
        selected_level: u8,
        probabilities: Vec<f32>,
        confidence: f32,
    },
    /// Binary probability resolution.
    Noul {
        id: String,
        probability: f32,
        is_true: bool,
    },
}

impl Answer {
    /// Returns the identifier of the question this answer corresponds to.
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Choice { id, .. } | Self::Score { id, .. } | Self::Noul { id, .. } => id,
        }
    }

    /// If this is a Choice answer, returns the selected value.
    #[must_use]
    pub fn choice_value(&self) -> Option<&str> {
        match self {
            Self::Choice { selected_value, .. } => Some(selected_value.as_str()),
            _ => None,
        }
    }

    /// If this is a Score answer, returns the selected level.
    #[must_use]
    pub fn score_level(&self) -> Option<u8> {
        match self {
            Self::Score { selected_level, .. } => Some(*selected_level),
            _ => None,
        }
    }

    /// If this is a Noul answer, returns the calibrated probability.
    #[must_use]
    pub fn noul_prob(&self) -> Option<f32> {
        match self {
            Self::Noul { probability, .. } => Some(*probability),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_serde_roundtrip() {
        let q_choice = Question::choice(
            "q1",
            "Select action",
            vec!["reply".into(), "ignore".into()],
        );
        let serialized = serde_json::to_string(&q_choice).unwrap();
        let deserialized: Question = serde_json::from_str(&serialized).unwrap();
        assert_eq!(q_choice, deserialized);
        assert_eq!(deserialized.id(), "q1");

        let q_score = Question::score("q2", "Rate importance", 5);
        let s_score = serde_json::to_string(&q_score).unwrap();
        let d_score: Question = serde_json::from_str(&s_score).unwrap();
        assert_eq!(q_score, d_score);

        let q_noul = Question::noul("q3", "Is this relevant?");
        let s_noul = serde_json::to_string(&q_noul).unwrap();
        let d_noul: Question = serde_json::from_str(&s_noul).unwrap();
        assert_eq!(q_noul, d_noul);
    }

    #[test]
    fn test_answer_accessors() {
        let a_choice = Answer::Choice {
            id: "q1".into(),
            selected_index: 0,
            selected_value: "reply".into(),
            probabilities: vec![0.9, 0.1],
            confidence: 0.9,
        };
        assert_eq!(a_choice.choice_value(), Some("reply"));
        assert_eq!(a_choice.score_level(), None);

        let a_score = Answer::Score {
            id: "q2".into(),
            selected_level: 4,
            probabilities: vec![0.0, 0.1, 0.2, 0.7, 0.0],
            confidence: 0.7,
        };
        assert_eq!(a_score.score_level(), Some(4));

        let a_noul = Answer::Noul {
            id: "q3".into(),
            probability: 0.85,
            is_true: true,
        };
        assert_eq!(a_noul.noul_prob(), Some(0.85));
    }
}
