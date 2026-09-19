// Seamless auto-continuation engine for truncated model responses.
// Detects truncation (unclosed code fences, abrupt mid-sentence cuts)
// and handles continuation prompts without requiring user intervention.

/// Maximum number of automatic continuation rounds allowed per turn.
pub const MAX_AUTO_CONTINUE_ROUNDS: usize = 5;

/// Analysis of whether a completion was truncated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruncationAnalysis {
    pub is_truncated: bool,
    pub in_code_block: bool,
    pub reason: Option<&'static str>,
}

/// Analyze text for signs of truncation.
pub fn analyze_truncation(text: &str) -> TruncationAnalysis {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return TruncationAnalysis {
            is_truncated: false,
            in_code_block: false,
            reason: None,
        };
    }

    // 1. Unclosed markdown code fence check
    // Count occurrences of ```
    let code_fence_count = text.matches("```").count();
    let in_code_block = code_fence_count % 2 != 0;
    if in_code_block {
        return TruncationAnalysis {
            is_truncated: true,
            in_code_block: true,
            reason: Some("unclosed_code_fence"),
        };
    }

    // 2. Abrupt ending punctuation / symbols check
    // Trailing punctuation that strongly signals unfinished thought
    let last_char = trimmed.chars().last().unwrap_or(' ');
    if matches!(last_char, ',' | ':' | ';' | '-' | '—' | '(' | '[' | '{' | '\\' | '/') {
        return TruncationAnalysis {
            is_truncated: true,
            in_code_block: false,
            reason: Some("trailing_connective_punctuation"),
        };
    }

    // 3. Trailing connective words check (Russian and English)
    if ends_with_connective_word(trimmed) {
        return TruncationAnalysis {
            is_truncated: true,
            in_code_block: false,
            reason: Some("trailing_connective_word"),
        };
    }

    TruncationAnalysis {
        is_truncated: false,
        in_code_block: false,
        reason: None,
    }
}

/// Returns true if text ends with a conjunction, preposition, or connector that implies more text was expected.
fn ends_with_connective_word(text: &str) -> bool {
    const CONNECTIVES: &[&str] = &[
        // Russian
        " и", " а", " но", " или", " что", " чтобы", " как", " где", " для",
        " в", " на", " с", " к", " по", " из", " от", " так как", " потому что",
        " например,", " то есть",
        // English
        " and", " or", " but", " because", " so", " that", " which", " where",
        " to", " of", " in", " on", " with", " for", " from", " by", " as",
        " e.g.", " i.e.",
    ];

    let lower = text.to_lowercase();
    for conn in CONNECTIVES {
        if lower.ends_with(conn) {
            return true;
        }
    }

    false
}

/// Generate appropriate continuation prompt based on truncation context.
pub fn continuation_prompt(in_code_block: bool) -> &'static str {
    if in_code_block {
        "Continue immediately without repeating previous text or reopening code fences. Continue the code from where it was cut off."
    } else {
        "Continue immediately from where you stopped. Do not repeat anything from your previous message."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unclosed_code_fence() {
        let text = "Here is the code:\n```rust\nfn hello() {\n    println!(\"hi\");\n";
        let res = analyze_truncation(text);
        assert!(res.is_truncated);
        assert!(res.in_code_block);
        assert_eq!(res.reason, Some("unclosed_code_fence"));
    }

    #[test]
    fn test_closed_code_fence() {
        let text = "Here is the code:\n```rust\nfn hello() {\n    println!(\"hi\");\n}\n```\nDone!";
        let res = analyze_truncation(text);
        assert!(!res.is_truncated);
        assert!(!res.in_code_block);
    }

    #[test]
    fn test_trailing_connective_punctuation() {
        let text = "В этой архитектуре используются следующие модули:";
        let res = analyze_truncation(text);
        assert!(res.is_truncated);
        assert_eq!(res.reason, Some("trailing_connective_punctuation"));

        let text2 = "The main parameters are,";
        let res2 = analyze_truncation(text2);
        assert!(res2.is_truncated);
    }

    #[test]
    fn test_trailing_connective_word() {
        let text = "Мы настроили шлюз и";
        let res = analyze_truncation(text);
        assert!(res.is_truncated);
        assert_eq!(res.reason, Some("trailing_connective_word"));

        let text2 = "We should implement the trait and";
        let res2 = analyze_truncation(text2);
        assert!(res2.is_truncated);
    }

    #[test]
    fn test_complete_sentence_not_truncated() {
        let text = "Всё успешно настроено и готово к работе.";
        let res = analyze_truncation(text);
        assert!(!res.is_truncated);

        let text2 = "Everything is set up properly.";
        let res2 = analyze_truncation(text2);
        assert!(!res2.is_truncated);
    }
}
