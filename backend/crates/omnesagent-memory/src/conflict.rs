//! Conflict resolution for memory entries.

use super::traits::{Memory, MemoryCategory, MemoryEntry};
use std::collections::HashSet;

/// Контекст входящей записи для разрешения конфликтов (семантика ob2h v2.0
/// «Contextual Coexistence»): подавлять можно только кандидатов в том же
/// пространстве имён и тенанте. Иная область — факты сосуществуют, а не конкурируют.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConflictScope {
    /// Пространство имён входящей записи (`None` → `"default"`).
    pub namespace: Option<String>,
    /// Тенант входящей записи (`None` → отсутствует).
    pub tenant_id: Option<String>,
}

impl ConflictScope {
    /// Совпадает ли кандидат с областью входящей записи.
    pub fn matches(&self, entry: &MemoryEntry) -> bool {
        let namespace = self.namespace.as_deref().unwrap_or("default");
        namespace == entry.namespace && self.tenant_id == entry.tenant_id
    }
}

/// Решение по одному кандидату: подавлять ли его как устаревший.
///
/// Учитывает семантику ob2h v2.0: контекст (`ConflictScope`), HITL для
/// защищённых записей (`pinned` — аналог SOUL/USER/SYSTEM) и порог схожести.
/// Идентичная по содержанию или та же самая запись (`key`) — обновление, не конфликт.
pub fn supersede_candidate(
    candidate: &MemoryEntry,
    key: &str,
    content: &str,
    scope: &ConflictScope,
    threshold: f64,
) -> bool {
    candidate.key != key
        && matches!(candidate.category, MemoryCategory::Core)
        && !candidate.pinned
        && scope.matches(candidate)
        && candidate.content != content
        && candidate.score.is_some_and(|score| score > threshold)
}

/// Check for conflicting memories and mark old ones as superseded.
/// Returns the list of entry IDs that were superseded.
pub async fn check_and_resolve_conflicts(
    memory: &dyn Memory,
    key: &str,
    content: &str,
    category: &MemoryCategory,
    scope: &ConflictScope,
    threshold: f64,
) -> anyhow::Result<Vec<String>> {
    // Only check conflicts for Core memories
    if !matches!(category, MemoryCategory::Core) {
        return Ok(Vec::new());
    }

    // Search for similar existing entries
    let candidates = memory.recall(content, 10, None, None, None).await?;

    let mut superseded = Vec::new();
    for candidate in &candidates {
        if supersede_candidate(candidate, key, content, scope, threshold) {
            superseded.push(candidate.id.clone());
        }
    }

    Ok(superseded)
}

/// Mark entries as superseded in SQLite by setting their `superseded_by` column.
pub fn mark_superseded(
    conn: &rusqlite::Connection,
    superseded_ids: &[String],
    new_id: &str,
) -> anyhow::Result<()> {
    if superseded_ids.is_empty() {
        return Ok(());
    }

    for id in superseded_ids {
        conn.execute(
            "UPDATE memories SET superseded_by = ?1 WHERE id = ?2",
            rusqlite::params![new_id, id],
        )?;
    }

    Ok(())
}

/// Ф62.4 (порт ob2h `superseded_among`): id записей, подавленных как устаревшие,
/// но ТОЛЬКО если запись-замена (`superseded_by`) тоже присутствует в выборке
/// `entries`. Если замена недоступна здесь же — запись не подавляем, иначе
/// знание было бы потеряно, а не обновлено.
///
/// В модели форка связь «старое → новое» хранится в колонке `memories.superseded_by`
/// (id записи-замены), поэтому алгоритм ob2h (рёбра `supersedes` между живыми
/// записями) выражается напрямую через поле `MemoryEntry::superseded_by`.
pub fn superseded_among(entries: &[MemoryEntry]) -> HashSet<String> {
    if entries.len() < 2 {
        return HashSet::new();
    }
    let present: HashSet<&str> = entries.iter().map(|entry| entry.id.as_str()).collect();
    entries
        .iter()
        .filter_map(|entry| {
            let successor = entry.superseded_by.as_deref()?;
            // Подавляем, лишь когда замена доступна в этой же выборке.
            present.contains(successor).then(|| entry.id.clone())
        })
        .collect()
}

/// Simple text-based conflict detection without embeddings.
/// Uses token overlap (Jaccard similarity) as a fast approximation
/// when vector embeddings are unavailable.
pub fn jaccard_similarity(a: &str, b: &str) -> f64 {
    let words_a: std::collections::HashSet<&str> = a.split_whitespace().collect();
    let words_b: std::collections::HashSet<&str> = b.split_whitespace().collect();

    if words_a.is_empty() && words_b.is_empty() {
        return 1.0;
    }
    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }

    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Find potentially conflicting entries using text similarity when embeddings
/// are not available. Returns entries above the threshold.
pub fn find_text_conflicts(
    entries: &[MemoryEntry],
    new_content: &str,
    threshold: f64,
) -> Vec<String> {
    entries
        .iter()
        .filter(|e| {
            matches!(e.category, MemoryCategory::Core)
                && e.superseded_by.is_none()
                && jaccard_similarity(&e.content, new_content) > threshold
                && e.content != new_content
        })
        .map(|e| e.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jaccard_identical_strings() {
        let sim = jaccard_similarity("hello world", "hello world");
        assert!((sim - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn jaccard_disjoint_strings() {
        let sim = jaccard_similarity("hello world", "foo bar");
        assert!(sim.abs() < f64::EPSILON);
    }

    #[test]
    fn jaccard_partial_overlap() {
        let sim = jaccard_similarity("the quick brown fox", "the slow brown dog");
        // overlap: "the", "brown" = 2; union: "the", "quick", "brown", "fox", "slow", "dog" = 6
        assert!((sim - 2.0 / 6.0).abs() < 0.01);
    }

    #[test]
    fn jaccard_empty_strings() {
        assert!((jaccard_similarity("", "") - 1.0).abs() < f64::EPSILON);
        assert!(jaccard_similarity("hello", "").abs() < f64::EPSILON);
        assert!(jaccard_similarity("", "hello").abs() < f64::EPSILON);
    }

    #[test]
    fn find_text_conflicts_filters_correctly() {
        let entries = vec![
            MemoryEntry {
                id: "1".into(),
                key: "pref".into(),
                content: "User prefers Rust for systems work".into(),
                category: MemoryCategory::Core,
                timestamp: "now".into(),
                session_id: None,
                score: None,
                namespace: "default".into(),
                importance: Some(0.7),
                superseded_by: None,
                kind: None,
                pinned: false,
                tenant_id: None,
                agent_alias: None,
                agent_id: None,
                trust: None,
                last_feedback_at: None,
            },
            MemoryEntry {
                id: "2".into(),
                key: "daily1".into(),
                content: "User prefers Rust for systems work".into(),
                category: MemoryCategory::Daily,
                timestamp: "now".into(),
                session_id: None,
                score: None,
                namespace: "default".into(),
                importance: Some(0.3),
                superseded_by: None,
                kind: None,
                pinned: false,
                tenant_id: None,
                agent_alias: None,
                agent_id: None,
                trust: None,
                last_feedback_at: None,
            },
        ];

        // Only Core entries should be flagged
        let conflicts = find_text_conflicts(&entries, "User now prefers Go for systems work", 0.3);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0], "1");
    }

    #[test]
    fn jaccard_deduplicates_repeated_words() {
        // Token sets ignore repeats: {a, b} vs {a, b} are identical.
        assert!((jaccard_similarity("a a b", "a b b") - 1.0).abs() < f64::EPSILON);
        // {a, b} vs {a} -> intersection 1 / union 2.
        assert!((jaccard_similarity("a a b", "a") - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn find_text_conflicts_skips_superseded_and_identical() {
        let entry = |id: &str, content: &str, superseded: Option<String>| MemoryEntry {
            id: id.into(),
            key: "k".into(),
            content: content.into(),
            category: MemoryCategory::Core,
            timestamp: "now".into(),
            session_id: None,
            score: None,
            namespace: "default".into(),
            importance: Some(0.7),
            superseded_by: superseded,
            kind: None,
            pinned: false,
            tenant_id: None,
            agent_alias: None,
            agent_id: None,
            trust: None,
            last_feedback_at: None,
        };
        let entries = vec![
            entry("active", "User prefers Rust for systems work", None),
            entry(
                "old",
                "User prefers Rust for systems work",
                Some("x".into()),
            ),
        ];

        // The already-superseded "old" entry is skipped; only "active" conflicts.
        let conflicts = find_text_conflicts(&entries, "User now prefers Go for systems work", 0.3);
        assert_eq!(conflicts, vec!["active".to_string()]);

        // An identical new_content is an update, not a conflict.
        let none = find_text_conflicts(&entries, "User prefers Rust for systems work", 0.3);
        assert!(none.is_empty());
    }

    fn marker_entry(id: &str, superseded_by: Option<&str>) -> MemoryEntry {
        MemoryEntry {
            id: id.into(),
            key: id.into(),
            content: id.into(),
            category: MemoryCategory::Core,
            timestamp: "now".into(),
            session_id: None,
            score: None,
            namespace: "default".into(),
            importance: Some(0.7),
            superseded_by: superseded_by.map(str::to_string),
            kind: None,
            pinned: false,
            tenant_id: None,
            agent_alias: None,
            agent_id: None,
            trust: None,
            last_feedback_at: None,
        }
    }

    #[test]
    fn superseded_among_suppresses_only_when_successor_present() {
        // Замена "new" присутствует -> "old" подавляется.
        let both = vec![
            marker_entry("old", Some("new")),
            marker_entry("new", None),
        ];
        let suppressed = superseded_among(&both);
        assert_eq!(suppressed.len(), 1);
        assert!(suppressed.contains("old"));

        // Замена отсутствует -> запись НЕ подавляем (знание не теряется).
        let lonely = vec![marker_entry("old", Some("new"))];
        assert!(superseded_among(&lonely).is_empty());
    }

    #[test]
    fn superseded_among_ignores_entries_without_marker() {
        let entries = vec![
            marker_entry("a", None),
            marker_entry("b", None),
            marker_entry("c", Some("missing")),
        ];
        assert!(superseded_among(&entries).is_empty());
    }

    /// Build a Core candidate with an explicit context (namespace/tenant),
    /// similarity score and pin state for `supersede_candidate` tests.
    fn core_candidate(
        key: &str,
        content: &str,
        namespace: &str,
        tenant_id: Option<&str>,
        pinned: bool,
        score: Option<f64>,
    ) -> MemoryEntry {
        MemoryEntry {
            id: key.into(),
            key: key.into(),
            content: content.into(),
            category: MemoryCategory::Core,
            timestamp: "now".into(),
            session_id: None,
            score,
            namespace: namespace.into(),
            importance: Some(0.7),
            superseded_by: None,
            kind: None,
            pinned,
            tenant_id: tenant_id.map(str::to_string),
            agent_alias: None,
            agent_id: None,
            trust: None,
            last_feedback_at: None,
        }
    }

    #[test]
    fn conflict_scope_matches_namespace_and_tenant() {
        let scope = ConflictScope::default();
        // Default scope = namespace "default", no tenant.
        assert!(scope.matches(&core_candidate("a", "x", "default", None, false, None)));
        assert!(!scope.matches(&core_candidate("a", "x", "other", None, false, None)));
        assert!(!scope.matches(&core_candidate("a", "x", "default", Some("t1"), false, None)));

        let scoped = ConflictScope {
            namespace: Some("alpha".into()),
            tenant_id: Some("t1".into()),
        };
        assert!(scoped.matches(&core_candidate("a", "x", "alpha", Some("t1"), false, None)));
        assert!(!scoped.matches(&core_candidate("a", "x", "alpha", Some("t2"), false, None)));
        assert!(!scoped.matches(&core_candidate("a", "x", "beta", Some("t1"), false, None)));
    }

    #[test]
    fn supersede_candidate_suppresses_same_context_above_threshold() {
        let scope = ConflictScope::default();
        let candidate = core_candidate(
            "old",
            "User prefers Rust for systems work",
            "default",
            None,
            false,
            Some(0.9),
        );
        assert!(supersede_candidate(
            &candidate,
            "new",
            "User now prefers Go for systems work",
            &scope,
            0.3,
        ));
    }

    #[test]
    fn supersede_candidate_coexists_across_contexts() {
        // Contextual Coexistence: different namespace/tenant → not suppressed.
        let scope = ConflictScope::default();
        let candidate = core_candidate(
            "old",
            "User prefers Rust for systems work",
            "other_ns",
            None,
            false,
            Some(0.9),
        );
        assert!(!supersede_candidate(
            &candidate,
            "new",
            "User now prefers Go for systems work",
            &scope,
            0.3,
        ));

        let other_tenant = core_candidate(
            "old",
            "User prefers Rust for systems work",
            "default",
            Some("t9"),
            false,
            Some(0.9),
        );
        assert!(!supersede_candidate(
            &other_tenant,
            "new",
            "User now prefers Go for systems work",
            &scope,
            0.3,
        ));
    }

    #[test]
    fn supersede_candidate_respects_hitl_pin() {
        // Pinned (SOUL/USER/SYSTEM surrogate) → no auto-supersede (HITL).
        let scope = ConflictScope::default();
        let candidate = core_candidate(
            "old",
            "User prefers Rust for systems work",
            "default",
            None,
            true,
            Some(0.9),
        );
        assert!(!supersede_candidate(
            &candidate,
            "new",
            "User now prefers Go for systems work",
            &scope,
            0.3,
        ));
    }

    #[test]
    fn supersede_candidate_ignores_self_identical_and_low_score() {
        let scope = ConflictScope::default();
        // Same key → update, not conflict.
        let same_key = core_candidate("new", "User prefers X", "default", None, false, Some(0.9));
        assert!(!supersede_candidate(
            &same_key, "new", "User prefers Y", &scope, 0.3
        ));
        // Identical content → no conflict.
        let identical = core_candidate("old", "User prefers X", "default", None, false, Some(0.9));
        assert!(!supersede_candidate(
            &identical, "new", "User prefers X", &scope, 0.3
        ));
        // Below threshold → not suppressed.
        let low = core_candidate("old", "User prefers X", "default", None, false, Some(0.2));
        assert!(!supersede_candidate(
            &low, "new", "User prefers Y", &scope, 0.3
        ));
        // Missing score → not suppressed.
        let no_score = core_candidate("old", "User prefers X", "default", None, false, None);
        assert!(!supersede_candidate(
            &no_score, "new", "User prefers Y", &scope, 0.3
        ));
    }
}
