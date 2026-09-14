//! Honest prefetch and MMR diversification for agent memory context (OB2H v1.3 Ф22).
//!
//! Provides 5-factor scoring (0.35 hybrid/rel + 0.25 importance + 0.20 trust + 0.10 recency + 0.10 sat_access),
//! MMR diversification (lambda = 0.7), and clean record-boundary context assembly within an 8000-char budget.

use crate::traits::{MemoryCategory, MemoryEntry};
use chrono::{DateTime, Utc};

pub const DEFAULT_PREFETCH_MAX_CHARS: usize = 8000;
pub const DEFAULT_PREFETCH_HALF_LIFE_DAYS: f64 = 90.0;
pub const DEFAULT_PREFETCH_MMR_LAMBDA: f64 = 0.7;
pub const DEFAULT_PREFETCH_LIMIT: usize = 10;

/// Options for assembling prefetch context block.
#[derive(Debug, Clone)]
pub struct ContextOptions {
    pub max_chars: usize,
    pub half_life_days: f64,
    pub mmr_lambda: f64,
    pub limit: usize,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            max_chars: DEFAULT_PREFETCH_MAX_CHARS,
            half_life_days: DEFAULT_PREFETCH_HALF_LIFE_DAYS,
            mmr_lambda: DEFAULT_PREFETCH_MMR_LAMBDA,
            limit: DEFAULT_PREFETCH_LIMIT,
        }
    }
}

/// Compute the 5-factor blended score:
/// 0.35 * rel + 0.25 * importance + 0.20 * trust + 0.10 * recency + 0.10 * sat_access
#[must_use]
pub fn five_factor_score(
    rel: f64,
    importance: f64,
    trust: f64,
    recency: f64,
    sat_access: f64,
) -> f64 {
    (0.35 * rel.clamp(0.0, 1.0)
        + 0.25 * importance.clamp(0.0, 1.0)
        + 0.20 * trust.clamp(0.0, 1.0)
        + 0.10 * recency.clamp(0.0, 1.0)
        + 0.10 * sat_access.clamp(0.0, 1.0))
        .clamp(0.0, 1.0)
}

/// Calculate recency factor with exponential half-life (default 90 days).
/// Core category is considered evergreen and always returns 1.0.
#[must_use]
pub fn recency_factor(entry: &MemoryEntry, half_life_days: f64) -> f64 {
    if entry.category == MemoryCategory::Core {
        return 1.0;
    }
    let Ok(ts) = DateTime::parse_from_rfc3339(&entry.timestamp) else {
        return 1.0;
    };
    let half_life = half_life_days.max(0.1);
    let age_days = Utc::now()
        .signed_duration_since(ts.with_timezone(&Utc))
        .num_seconds()
        .max(0) as f64
        / 86_400.0;
    (-age_days / half_life).exp().clamp(0.0, 1.0)
}

/// Calculate access saturation factor: log1p scale reaching 1.0 at ~50 accesses.
#[must_use]
pub fn saturation_access(access_count: i64) -> f64 {
    let ln_50 = 50.0_f64.ln();
    ((1.0 + access_count.max(0) as f64).ln() / ln_50).clamp(0.0, 1.0)
}

/// Maximal Marginal Relevance (MMR) selection over candidate vectors.
/// lambda * rel[i] - (1 - lambda) * max_sim(i, selected).
#[must_use]
pub fn mmr_select(
    rel: &[f64],
    vecs: &[Option<Vec<f32>>],
    limit: usize,
    lambda: f64,
) -> Vec<usize> {
    if rel.is_empty() || limit == 0 {
        return Vec::new();
    }
    let lambda = lambda.clamp(0.0, 1.0);
    let mut remaining: Vec<usize> = (0..rel.len()).collect();
    let mut selected: Vec<usize> = Vec::with_capacity(limit.min(rel.len()));

    while selected.len() < limit && !remaining.is_empty() {
        let mut best: Option<(usize, f64)> = None;
        for &i in &remaining {
            let max_sim = selected
                .iter()
                .map(|&s| match (&vecs[i], &vecs[s]) {
                    (Some(a), Some(b)) => crate::vector::cosine_similarity(a, b) as f64,
                    _ => 0.0,
                })
                .fold(-1.0_f64, f64::max);

            let score = lambda * rel[i] - (1.0 - lambda) * max_sim;
            if best.is_none() || score > best.map(|(_, bs)| bs).unwrap_or(f64::NEG_INFINITY) {
                best = Some((i, score));
            }
        }
        let (i, _) = best.expect("remaining is non-empty");
        selected.push(i);
        remaining.retain(|&r| r != i);
    }
    selected
}

/// Candidate record for context compilation.
struct ScoredCandidate {
    entry: MemoryEntry,
    score: f64,
    vec: Option<Vec<f32>>,
}

/// Assemble formatted memory context within budget, returning (context_str, included_keys).
pub fn build_memory_context(
    entries: Vec<MemoryEntry>,
    vecs: Vec<Option<Vec<f32>>>,
    opts: &ContextOptions,
) -> (String, Vec<String>) {
    if entries.is_empty() {
        return (String::new(), Vec::new());
    }

    let max_score = entries
        .iter()
        .map(|e| e.score.unwrap_or(0.5))
        .fold(0.0_f64, f64::max)
        .max(1e-6);

    let mut candidates: Vec<ScoredCandidate> = entries
        .into_iter()
        .zip(vecs)
        .map(|(entry, vec)| {
            let rel = (entry.score.unwrap_or(0.5) / max_score).clamp(0.0, 1.0);
            let imp = entry.importance.unwrap_or(0.5).clamp(0.0, 1.0);
            let trust = entry.trust.unwrap_or(0.5).clamp(0.0, 1.0);
            let rec = recency_factor(&entry, opts.half_life_days);
            // Saturation access: default 0 if not tracked
            let sat = saturation_access(0);
            let score = five_factor_score(rel, imp, trust, rec, sat);
            ScoredCandidate { entry, score, vec }
        })
        .collect();

    candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    let max_cand_score = candidates.iter().map(|c| c.score).fold(0.0_f64, f64::max).max(1e-6);
    let rels: Vec<f64> = candidates.iter().map(|c| c.score / max_cand_score).collect();
    let cand_vecs: Vec<Option<Vec<f32>>> = candidates.iter().map(|c| c.vec.clone()).collect();

    let picked_indices = mmr_select(&rels, &cand_vecs, opts.limit, opts.mmr_lambda);

    let budget = opts.max_chars.max(64);
    let open_tag = crate::MEMORY_CONTEXT_OPEN;
    let close_tag = crate::MEMORY_CONTEXT_CLOSE;
    let block_overhead = open_tag.chars().count() + close_tag.chars().count() + 4; // newlines

    let mut used = block_overhead;
    let mut lines_out: Vec<String> = Vec::new();
    let mut included_keys: Vec<String> = Vec::new();

    for &idx in &picked_indices {
        let cand = &candidates[idx];
        let line = format!("- [{}] {}: {}\n", cand.entry.category, cand.entry.key, cand.entry.content);
        let line_chars = line.chars().count();
        if used + line_chars > budget {
            break;
        }
        used += line_chars;
        included_keys.push(cand.entry.key.clone());
        lines_out.push(line);
    }

    if lines_out.is_empty() && !picked_indices.is_empty() {
        // Budget is very tight: strictly truncate the first record
        let cand = &candidates[picked_indices[0]];
        let prefix = format!("- [{}] {}: ", cand.entry.category, cand.entry.key);
        let avail = budget.saturating_sub(block_overhead + prefix.chars().count() + 1).max(10);
        let truncated: String = cand.entry.content.chars().take(avail).collect();
        included_keys.push(cand.entry.key.clone());
        lines_out.push(format!("{prefix}{truncated}\n"));
    }

    let mut output = String::from(open_tag);
    output.push('\n');
    for line in lines_out {
        output.push_str(&line);
    }
    output.push_str(close_tag);
    output.push_str("\n\n");

    (output, included_keys)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_five_factor_score() {
        // Perfect scores
        let s = five_factor_score(1.0, 1.0, 1.0, 1.0, 1.0);
        assert!((s - 1.0).abs() < 1e-6);

        // Baseline 0.5 across all factors
        let s_mid = five_factor_score(0.5, 0.5, 0.5, 0.5, 0.5);
        assert!((s_mid - 0.5).abs() < 1e-6);

        // Weights: 0.35*1 + 0.25*0 + 0.2*0 + 0.1*0 + 0.1*0 = 0.35
        let s_rel = five_factor_score(1.0, 0.0, 0.0, 0.0, 0.0);
        assert!((s_rel - 0.35).abs() < 1e-6);
    }

    #[test]
    fn test_saturation_access() {
        assert_eq!(saturation_access(0), 0.0);
        assert!(saturation_access(10) > 0.5);
        assert!((saturation_access(50) - 1.0).abs() < 0.02);
        assert_eq!(saturation_access(100), 1.0);
    }

    #[test]
    fn test_mmr_select_diversity() {
        let rel = vec![1.0, 0.95, 0.7];
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.99, 0.01, 0.0]; // almost identical to v1
        let v3 = vec![0.0, 1.0, 0.0];  // orthogonal (diverse)

        let vecs = vec![Some(v1), Some(v2), Some(v3)];
        let selected = mmr_select(&rel, &vecs, 2, 0.7);

        // First is v1 (index 0). With lambda=0.7, v3 (index 2) should be selected over near-duplicate v2 (index 1)
        assert_eq!(selected, vec![0, 2]);
    }

    #[test]
    fn test_build_memory_context_budget() {
        let entry1 = MemoryEntry {
            id: "1".into(),
            key: "rule-1".into(),
            content: "Always write unit tests".into(),
            category: MemoryCategory::Core,
            timestamp: Utc::now().to_rfc3339(),
            session_id: None,
            score: Some(0.9),
            namespace: "default".into(),
            importance: Some(0.8),
            superseded_by: None,
            kind: None,
            pinned: false,
            tenant_id: None,
            agent_alias: None,
            agent_id: None,
            trust: Some(0.8),
            last_feedback_at: None,
        };

        let entry2 = MemoryEntry {
            id: "2".into(),
            key: "rule-2".into(),
            content: "Use rustfmt before commit".into(),
            category: MemoryCategory::Core,
            timestamp: Utc::now().to_rfc3339(),
            session_id: None,
            score: Some(0.85),
            namespace: "default".into(),
            importance: Some(0.7),
            superseded_by: None,
            kind: None,
            pinned: false,
            tenant_id: None,
            agent_alias: None,
            agent_id: None,
            trust: Some(0.75),
            last_feedback_at: None,
        };

        let opts = ContextOptions {
            max_chars: 8000,
            limit: 5,
            ..Default::default()
        };

        let (ctx, keys) = build_memory_context(vec![entry1, entry2], vec![None, None], &opts);
        assert!(ctx.contains("[Memory context]"));
        assert!(ctx.contains("rule-1"));
        assert!(ctx.contains("rule-2"));
        assert_eq!(keys, vec!["rule-1", "rule-2"]);
    }
}
