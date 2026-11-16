//! Nightly agent-scoped memory consolidation — the operational (deterministic)
//! analog of ob2h's "Dream" pillar (Ф6.4).
//!
//! Per-turn consolidation ([`crate::consolidation`]) is LLM-driven and
//! synthesizes facts from a single turn. This pass is mechanical and periodic:
//! it scans Core memory and, *within each `(agent, namespace, tenant)` scope*,
//! collapses near-duplicate entries by marking the older sibling superseded.
//!
//! Scopes are never merged (ADR-O2): entries belonging to different agents,
//! namespaces or tenants are left untouched and allowed to coexist, mirroring
//! the "Contextual Coexistence" semantics of [`crate::conflict`]. `pinned` rows
//! (the HITL-protected SOUL/USER/SYSTEM surrogates) are never touched either.
//!
//! The pass is best-effort and opt-in (`MemoryConfig::dream_enabled`, default
//! off), throttled to at most one run per [`DREAM_INTERVAL_HOURS`] by a state
//! file under `<workspace>/state/`, and SQLite-only.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use omnesagent_config::schema::MemoryConfig;

use crate::conflict;

/// One consolidation pass runs at most once per this many hours.
const DREAM_INTERVAL_HOURS: i64 = 24;
const STATE_FILE: &str = "memory_dream_state.json";
/// Upper bound on the number of Core rows scanned per pass, to keep the
/// pairwise near-duplicate comparison bounded.
const MAX_SCAN_ROWS: i64 = 5_000;

/// `(agent_id, namespace, tenant_id)` — the isolation scope a row belongs to.
type ScopeKey = (Option<String>, String, Option<String>);

fn is_zero(n: &u64) -> bool {
    *n == 0
}

/// Result of a single consolidation pass.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DreamReport {
    /// Core rows considered (non-superseded, non-pinned).
    pub scanned: u64,
    /// Distinct `(agent, namespace, tenant)` scopes that held scanned rows.
    pub scopes: u64,
    /// Rows marked superseded as near-duplicates of a newer sibling.
    pub superseded: u64,
    /// Daily rows considered since the last successful daily cursor.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub daily_scanned: u64,
    /// Daily rows marked superseded as near-duplicates of a newer sibling.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub daily_superseded: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct DreamState {
    last_run_at: Option<String>,
    last_report: DreamReport,
    /// Updated_at watermark of the newest `daily` row processed on a successful run.
    #[serde(default)]
    last_daily_cursor: Option<String>,
}

struct CoreRow {
    id: String,
    content: String,
    scope: ScopeKey,
    updated_at: String,
}

/// Run the nightly consolidation if the cadence window has elapsed.
///
/// Best-effort: callers should log and continue on failure. Returns `Ok(None)`
/// when the feature is disabled or the pass is throttled, and `Ok(Some(report))`
/// when a pass actually ran.
pub fn run_if_due(config: &MemoryConfig, workspace_dir: &Path) -> Result<Option<DreamReport>> {
    if !config.dream_enabled {
        return Ok(None);
    }

    if !should_run_now(workspace_dir)? {
        return Ok(None);
    }

    let (report, next_daily_cursor) = consolidate(workspace_dir, config, load_daily_cursor(workspace_dir).as_deref())?;
    write_state(workspace_dir, &report, next_daily_cursor)?;

    if report.superseded > 0 || report.daily_superseded > 0 {
        ::omnesagent_log::record!(
            INFO,
            ::omnesagent_log::Event::new(module_path!(), ::omnesagent_log::Action::Note),
            &format!(
                "nightly memory consolidation complete: scanned={} scopes={} superseded={} daily_scanned={} daily_superseded={}",
                report.scanned, report.scopes, report.superseded, report.daily_scanned, report.daily_superseded
            )
        );
    }

    Ok(Some(report))
}

/// Run a consolidation pass immediately, ignoring the cadence gate. Used by the
/// manual `dream_tool` trigger. Refreshes the cadence state so a scheduled pass
/// does not immediately repeat the same work.
pub fn consolidate_now(config: &MemoryConfig, workspace_dir: &Path) -> Result<DreamReport> {
    let (report, next_daily_cursor) = consolidate(workspace_dir, config, load_daily_cursor(workspace_dir).as_deref())?;
    write_state(workspace_dir, &report, next_daily_cursor)?;
    Ok(report)
}

/// Group rows by `(agent, namespace, tenant)` scope and mark older near-duplicate
/// siblings as superseded by the newest matching row in the same scope.
fn collect_near_duplicates(rows: &[CoreRow], threshold: f64) -> (u64, HashMap<String, String>) {
    let mut groups: HashMap<&ScopeKey, Vec<usize>> = HashMap::new();
    for (idx, row) in rows.iter().enumerate() {
        groups.entry(&row.scope).or_default().push(idx);
    }

    let mut superseded: HashMap<String, String> = HashMap::new();
    for idxs in groups.values() {
        for (pos, &older) in idxs.iter().enumerate() {
            for &newer in &idxs[..pos] {
                if is_near_duplicate(&rows[older], &rows[newer], threshold) {
                    superseded
                        .entry(rows[older].id.clone())
                        .or_insert_with(|| rows[newer].id.clone());
                    break;
                }
            }
        }
    }

    (groups.len() as u64, superseded)
}

/// The pass itself: scan Core rows and incremental Daily rows (since `daily_cursor`)
/// and, per scope, mark near-duplicate older siblings as superseded.
fn consolidate(
    workspace_dir: &Path,
    config: &MemoryConfig,
    daily_cursor: Option<&str>,
) -> Result<(DreamReport, Option<String>)> {
    let db_path = workspace_dir.join("memory").join("brain.db");
    if !db_path.exists() {
        return Ok((DreamReport::default(), daily_cursor.map(str::to_string)));
    }

    let conn = Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;

    let threshold = config.dedup_jaccard_threshold;
    let core_rows = load_rows_by_category(&conn, "core", None)?;
    let scanned = core_rows.len() as u64;
    let (scopes, core_superseded) = collect_near_duplicates(&core_rows, threshold);

    let daily_rows = load_rows_by_category(&conn, "daily", daily_cursor)?;
    let daily_scanned = daily_rows.len() as u64;
    let next_daily_cursor = daily_rows
        .first()
        .map(|r| r.updated_at.clone())
        .or_else(|| daily_cursor.map(str::to_string));
    let (_, daily_superseded) = collect_near_duplicates(&daily_rows, threshold);

    let superseded_count = core_superseded.len() as u64;
    let daily_superseded_count = daily_superseded.len() as u64;
    if superseded_count == 0 && daily_superseded_count == 0 {
        return Ok((
            DreamReport {
                scanned,
                scopes,
                superseded: 0,
                daily_scanned,
                daily_superseded: 0,
            },
            next_daily_cursor,
        ));
    }

    let tx = conn.unchecked_transaction()?;
    for (id, keeper) in core_superseded.iter().chain(daily_superseded.iter()) {
        tx.execute(
            "UPDATE memories SET superseded_by = ?1 WHERE id = ?2",
            params![keeper, id],
        )?;
    }
    tx.commit()?;

    Ok((
        DreamReport {
            scanned,
            scopes,
            superseded: superseded_count,
            daily_scanned,
            daily_superseded: daily_superseded_count,
        },
        next_daily_cursor,
    ))
}

/// Load live, non-pinned rows for `category`, newest first. When `updated_after`
/// is supplied, only rows strictly newer than the cursor are returned.
fn load_rows_by_category(
    conn: &Connection,
    category: &str,
    updated_after: Option<&str>,
) -> Result<Vec<CoreRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, agent_id, namespace, tenant_id, updated_at \
         FROM memories \
         WHERE category = ?1 AND superseded_by IS NULL AND pinned = 0 \
           AND (?2 IS NULL OR updated_at > ?2) \
         ORDER BY updated_at DESC \
         LIMIT ?3",
    )?;

    let rows = stmt
        .query_map(params![category, updated_after, MAX_SCAN_ROWS], |row| {
            Ok(CoreRow {
                id: row.get(0)?,
                content: row.get(1)?,
                scope: (
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?
                        .unwrap_or_else(|| "default".into()),
                    row.get::<_, Option<String>>(4)?,
                ),
                updated_at: row.get(5)?,
            })
        })?
        .filter_map(|row| row.ok())
        .collect();

    Ok(rows)
}

/// Near-duplicate test mirroring the write-path dedup branch: the contents must
/// differ but share enough tokens to exceed `threshold`. Entries with identical
/// content but distinct keys are left alone — at the nightly level there is no
/// incoming key to disambiguate them, so collapsing them could drop a distinct
/// fact.
fn is_near_duplicate(older: &CoreRow, newer: &CoreRow, threshold: f64) -> bool {
    if older.content == newer.content {
        return false;
    }

    // Cheap length prefilter: near-duplicates have comparable lengths.
    let (lo, hi) = (
        older.content.len().min(newer.content.len()),
        older.content.len().max(newer.content.len()),
    );
    if lo.saturating_mul(2) < hi {
        return false;
    }

    conflict::jaccard_similarity(&older.content, &newer.content) > threshold
}

fn should_run_now(workspace_dir: &Path) -> Result<bool> {
    let path = state_path(workspace_dir);
    if !path.exists() {
        return Ok(true);
    }

    let raw = fs::read_to_string(&path)?;
    let state: DreamState = match serde_json::from_str(&raw) {
        Ok(state) => state,
        Err(_) => return Ok(true),
    };

    let Some(last_run_at) = state.last_run_at else {
        return Ok(true);
    };

    let last = match DateTime::parse_from_rfc3339(&last_run_at) {
        Ok(ts) => ts.with_timezone(&Utc),
        Err(_) => return Ok(true),
    };

    Ok(Utc::now().signed_duration_since(last) >= Duration::hours(DREAM_INTERVAL_HOURS))
}

fn load_daily_cursor(workspace_dir: &Path) -> Option<String> {
    let path = state_path(workspace_dir);
    let raw = fs::read_to_string(path).ok()?;
    let state: DreamState = serde_json::from_str(&raw).ok()?;
    state.last_daily_cursor
}

fn write_state(
    workspace_dir: &Path,
    report: &DreamReport,
    last_daily_cursor: Option<String>,
) -> Result<()> {
    let path = state_path(workspace_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let state = DreamState {
        last_run_at: Some(Utc::now().to_rfc3339()),
        last_report: report.clone(),
        last_daily_cursor,
    };
    let json = serde_json::to_vec_pretty(&state)?;
    fs::write(path, json)?;
    Ok(())
}

fn state_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join("state").join(STATE_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sqlite::SqliteMemory;
    use crate::traits::{Memory, MemoryCategory, StoreOptions};
    use tempfile::TempDir;

    fn enabled_cfg() -> MemoryConfig {
        MemoryConfig {
            dream_enabled: true,
            // Lower the threshold so the fixtures only need a small overlap.
            dedup_jaccard_threshold: 0.5,
            ..MemoryConfig::default()
        }
    }

    async fn store(workspace: &Path, key: &str, content: &str, options: StoreOptions) {
        let mem = SqliteMemory::new("sqlite", workspace).unwrap();
        mem.store_with_options(key, content, MemoryCategory::Core, None, options)
            .await
            .unwrap();
    }

    /// Pin `updated_at` so "which row is newest" is deterministic in the test
    /// (the write path only records second-resolution timestamps).
    fn set_updated_at(workspace: &Path, key: &str, ts: &str) {
        let db = workspace.join("memory").join("brain.db");
        let conn = Connection::open(db).unwrap();
        conn.execute(
            "UPDATE memories SET updated_at = ?1 WHERE key = ?2",
            params![ts, key],
        )
        .unwrap();
    }

    async fn live_core_keys(workspace: &Path) -> Vec<String> {
        let mem = SqliteMemory::new("sqlite", workspace).unwrap();
        let mut keys: Vec<String> = mem
            .list(Some(&MemoryCategory::Core), None)
            .await
            .unwrap()
            .into_iter()
            .map(|entry| entry.key)
            .collect();
        keys.sort();
        keys
    }

    #[test]
    fn disabled_feature_is_a_noop() {
        let tmp = TempDir::new().unwrap();
        let outcome = run_if_due(&MemoryConfig::default(), tmp.path()).unwrap();
        assert!(outcome.is_none(), "dream pass must not run when disabled");
    }

    #[tokio::test]
    async fn consolidates_near_duplicates_within_a_scope() {
        let tmp = TempDir::new().unwrap();
        let workspace = tmp.path();

        store(workspace, "fact_old", "alpha beta gamma", StoreOptions::default()).await;
        store(workspace, "fact_new", "alpha beta gamma delta", StoreOptions::default()).await;
        set_updated_at(workspace, "fact_old", "2026-01-01T00:00:00+00:00");
        set_updated_at(workspace, "fact_new", "2026-06-01T00:00:00+00:00");

        let report = run_if_due(&enabled_cfg(), workspace).unwrap().unwrap();
        assert_eq!(report.superseded, 1, "one near-duplicate row is superseded");
        assert_eq!(report.scopes, 1);

        let keys = live_core_keys(workspace).await;
        assert_eq!(keys, vec!["fact_new".to_string()], "the older twin is suppressed");
    }

    #[tokio::test]
    async fn never_merges_across_scopes() {
        let tmp = TempDir::new().unwrap();
        let workspace = tmp.path();

        store(
            workspace,
            "ns_a",
            "alpha beta gamma",
            StoreOptions::default().with_namespace("ns_a"),
        )
        .await;
        store(
            workspace,
            "ns_b",
            "alpha beta gamma delta",
            StoreOptions::default().with_namespace("ns_b"),
        )
        .await;

        let report = run_if_due(&enabled_cfg(), workspace).unwrap().unwrap();
        assert_eq!(report.superseded, 0, "different namespaces must coexist");
        assert_eq!(report.scopes, 2);

        let keys = live_core_keys(workspace).await;
        assert_eq!(keys, vec!["ns_a".to_string(), "ns_b".to_string()]);
    }

    #[tokio::test]
    async fn pinned_rows_are_never_superseded() {
        let tmp = TempDir::new().unwrap();
        let workspace = tmp.path();

        store(
            workspace,
            "pinned_fact",
            "alpha beta gamma",
            StoreOptions {
                pinned: true,
                ..StoreOptions::default()
            },
        )
        .await;
        store(workspace, "free_fact", "alpha beta gamma delta", StoreOptions::default()).await;

        let report = run_if_due(&enabled_cfg(), workspace).unwrap().unwrap();
        assert_eq!(report.superseded, 0, "pinned rows are excluded from the scan");
        assert_eq!(report.scanned, 1, "only the non-pinned row is scanned");
    }

    #[test]
    fn cadence_gate_skips_the_second_run() {
        let tmp = TempDir::new().unwrap();
        let workspace = tmp.path();

        let first = run_if_due(&enabled_cfg(), workspace).unwrap();
        assert!(first.is_some(), "first pass runs");
        let second = run_if_due(&enabled_cfg(), workspace).unwrap();
        assert!(second.is_none(), "second pass is throttled within the window");
    }

    #[tokio::test]
    async fn consolidate_now_processes_daily_rows_and_advances_cursor() {
        let tmp = TempDir::new().unwrap();
        let workspace = tmp.path();

        let mem = SqliteMemory::new("sqlite", workspace).unwrap();
        mem.store_with_options(
            "d1",
            "alpha beta gamma delta",
            MemoryCategory::Daily,
            None,
            StoreOptions::default(),
        )
        .await
        .unwrap();
        mem.store_with_options(
            "d2",
            "alpha beta gamma delta epsilon",
            MemoryCategory::Daily,
            None,
            StoreOptions::default(),
        )
        .await
        .unwrap();

        let first = consolidate_now(&enabled_cfg(), workspace).unwrap();
        assert_eq!(first.daily_scanned, 2);
        assert_eq!(first.daily_superseded, 1);

        // Second pass with no new daily rows skips already-processed records via cursor.
        let second = consolidate_now(&enabled_cfg(), workspace).unwrap();
        assert_eq!(second.daily_scanned, 0);
        assert_eq!(second.daily_superseded, 0);
    }
}
