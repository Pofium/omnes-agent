//! Category-based redaction for durable memory content.
//!
//! [`redact`] rewrites matches for the enabled [`RedactCategory`] set to
//! `[REDACTED:<category>]` placeholders before content is persisted. The
//! write path applies it only when `[memory.policy].redact_on_write` is
//! enabled; the category list comes from
//! `[memory.policy].redact_categories`.

use regex::Regex;
use std::ops::Range;
use std::sync::LazyLock;

/// A redactable content category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedactCategory {
    Secret,
    ApiKey,
    PrivateKey,
    Email,
    Phone,
}

impl RedactCategory {
    /// Parse a config string into a category. Unknown values return `None`
    /// and are skipped by the caller.
    pub fn from_config(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "secret" => Some(Self::Secret),
            "api_key" | "apikey" => Some(Self::ApiKey),
            "private_key" | "privatekey" => Some(Self::PrivateKey),
            "email" => Some(Self::Email),
            "phone" => Some(Self::Phone),
            _ => None,
        }
    }
}

impl std::fmt::Display for RedactCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Secret => write!(f, "secret"),
            Self::ApiKey => write!(f, "api_key"),
            Self::PrivateKey => write!(f, "private_key"),
            Self::Email => write!(f, "email"),
            Self::Phone => write!(f, "phone"),
        }
    }
}

/// A single redaction: the category and the replaced byte range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactionHit {
    pub category: RedactCategory,
    pub byte_range: Range<usize>,
}

struct RedactionPattern {
    category: RedactCategory,
    regex: Regex,
    /// Optional replacement template using `$N` capture-group references. When
    /// `None`, the whole match becomes `[REDACTED:<category>]`. Templates let a
    /// non-sensitive prefix (scheme, `Bearer `, env-var name) survive redaction.
    replacement: Option<&'static str>,
}

static PATTERNS: LazyLock<Vec<RedactionPattern>> = LazyLock::new(|| {
    vec![
        RedactionPattern {
            category: RedactCategory::PrivateKey,
            regex: Regex::new(r#"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----"#)
                .expect("valid private-key redaction regex"),
            replacement: None,
        },
        RedactionPattern {
            category: RedactCategory::ApiKey,
            regex: Regex::new(
                r#"(?i)\b(api[_-]?key)\s*[:=]\s*['"]?[A-Za-z0-9_./+=-]{16,}['"]?"#,
            )
            .expect("valid api-key redaction regex"),
            replacement: None,
        },
        RedactionPattern {
            category: RedactCategory::Secret,
            regex: Regex::new(
                r#"(?i)\b(secret|token|password|credential)\s*[:=]\s*['"]?[A-Za-z0-9_./+=-]{12,}['"]?"#,
            )
            .expect("valid secret redaction regex"),
            replacement: None,
        },
        RedactionPattern {
            category: RedactCategory::Email,
            regex: Regex::new(r#"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b"#)
                .expect("valid email redaction regex"),
            replacement: None,
        },
        RedactionPattern {
            category: RedactCategory::Phone,
            regex: Regex::new(r#"\b(?:\+?\d[\d .()-]{7,}\d)\b"#)
                .expect("valid phone redaction regex"),
            replacement: None,
        },
        // Ф11.1 (паритет с ob2h Ф69.1): OpenAI-стиль `sk-…` длинные ключи.
        RedactionPattern {
            category: RedactCategory::ApiKey,
            regex: Regex::new(r#"\bsk-[A-Za-z0-9_-]{16,}"#)
                .expect("valid sk-key redaction regex"),
            replacement: None,
        },
        // GitHub-токены: ghp_/gho_/ghs_/ghu_…
        RedactionPattern {
            category: RedactCategory::ApiKey,
            regex: Regex::new(r#"\bgh[opsh]_[A-Za-z0-9]{20,}"#)
                .expect("valid github-token redaction regex"),
            replacement: None,
        },
        // `Authorization: Bearer <token>` — сохраняем слово Bearer.
        RedactionPattern {
            category: RedactCategory::Secret,
            regex: Regex::new(r#"(?i)(\bbearer\s+)\S{8,}"#)
                .expect("valid bearer redaction regex"),
            replacement: Some("$1[REDACTED:secret]"),
        },
        // URL-userinfo с паролем: `scheme://user:pass@host` — сохраняем `://`.
        RedactionPattern {
            category: RedactCategory::Secret,
            regex: Regex::new(r#"(://)[^/\s@]+:[^/\s@]+@"#)
                .expect("valid url-userinfo redaction regex"),
            replacement: Some("$1[REDACTED:secret]"),
        },
        // Env-присваивания ob2h: `OB2H_…=…` — сохраняем имя переменной.
        RedactionPattern {
            category: RedactCategory::Secret,
            regex: Regex::new(r#"(\bOB2H_[A-Z0-9_]+\s*=\s*)\S+"#)
                .expect("valid ob2h-env redaction regex"),
            replacement: Some("$1[REDACTED:secret]"),
        },
    ]
});

/// Replace matches for the enabled categories with
/// `[REDACTED:<category>]` placeholders (or the pattern's template). Returns the
/// rewritten content and the list of replacements made.
pub fn redact(content: &str, categories: &[RedactCategory]) -> (String, Vec<RedactionHit>) {
    let mut redacted = content.to_string();
    let mut hits = Vec::new();

    for pattern in PATTERNS
        .iter()
        .filter(|pattern| categories.contains(&pattern.category))
    {
        let mut next = String::with_capacity(redacted.len());
        let mut last = 0usize;
        let mut matched = false;
        for captures in pattern.regex.captures_iter(&redacted) {
            let whole = captures
                .get(0)
                .expect("capture group 0 is always present");
            next.push_str(&redacted[last..whole.start()]);
            match pattern.replacement {
                Some(template) => captures.expand(template, &mut next),
                None => next.push_str(&format!("[REDACTED:{}]", pattern.category)),
            }
            hits.push(RedactionHit {
                category: pattern.category,
                byte_range: whole.start()..whole.end(),
            });
            last = whole.end();
            matched = true;
        }
        if !matched {
            continue;
        }
        next.push_str(&redacted[last..]);
        redacted = next;
    }

    (redacted, hits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_email_and_secret() {
        let (redacted, hits) = redact(
            r#"contact user@example.com with token = "abcdefghijklmnop""#,
            &[RedactCategory::Email, RedactCategory::Secret],
        );
        assert!(redacted.contains("[REDACTED:email]"));
        assert!(redacted.contains("[REDACTED:secret]"));
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().all(|hit| !hit.byte_range.is_empty()));
    }

    #[test]
    fn identity_when_no_category_matches() {
        let input = "contact user@example.com";
        let (redacted, hits) = redact(input, &[RedactCategory::Secret]);
        assert_eq!(redacted, input);
        assert!(hits.is_empty());
    }

    #[test]
    fn unknown_config_category_is_skipped() {
        assert_eq!(RedactCategory::from_config("nonsense"), None);
        assert_eq!(
            RedactCategory::from_config(" API_KEY "),
            Some(RedactCategory::ApiKey)
        );
    }

    // --- Ф11.1: parity with ob2h Ф69.1 capture-time scrubbing ------------------

    #[test]
    fn redacts_openai_style_sk_keys() {
        let (redacted, hits) = redact("export K=sk-abcdefghijklmnop123", &[RedactCategory::ApiKey]);
        assert!(redacted.contains("[REDACTED:api_key]"), "got {redacted:?}");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn short_sk_prefix_is_not_redacted() {
        let input = "sk-short";
        let (redacted, hits) = redact(input, &[RedactCategory::ApiKey]);
        assert_eq!(redacted, input);
        assert!(hits.is_empty());
    }

    #[test]
    fn redacts_github_tokens() {
        let (redacted, hits) = redact(
            "ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ0123",
            &[RedactCategory::ApiKey],
        );
        assert!(redacted.contains("[REDACTED:api_key]"), "got {redacted:?}");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn redacts_bearer_and_keeps_scheme_word() {
        let (redacted, _) = redact(
            "Authorization: Bearer abcdefgh12345",
            &[RedactCategory::Secret],
        );
        assert_eq!(redacted, "Authorization: Bearer [REDACTED:secret]");
    }

    #[test]
    fn redacts_url_userinfo_and_keeps_scheme() {
        let (redacted, _) = redact(
            "postgres://alice:s3cretpw@db:5432/x",
            &[RedactCategory::Secret],
        );
        assert_eq!(redacted, "postgres://[REDACTED:secret]db:5432/x");
    }

    #[test]
    fn redacts_ob2h_env_assignments_keeping_name() {
        let (redacted, _) = redact(
            r"OB2H_DATA_DIR=C:\Projects\ob2h\data",
            &[RedactCategory::Secret],
        );
        assert_eq!(redacted, "OB2H_DATA_DIR=[REDACTED:secret]");
    }
}
