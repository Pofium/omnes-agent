//! Edit-time blast radius (warn-only).
//!
//! Hint вычисляется на read-path из `graph_nodes.updated_at` (обновляется
//! инкрементальным ресканом ProjectWatcher'а / git-хуков) и кэшируется в kv
//! `blast_hint:<project>` с TTL 30 минут. Содержимое hint'а: последний
//! изменённый символ + top-5 прямых callers.

use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::{params, Connection, OptionalExtension};

/// TTL подсказки (сек): 30 минут.
pub const BLAST_TTL_SECS: u64 = 30 * 60;

/// Сколько callers хранить в hint.
pub const BLAST_CALLERS: usize = 5;

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlastHint {
    pub project_id: String,
    pub symbol: String,
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub callers: Vec<String>,
    /// unix-сек, до которого hint свежий
    pub expires_at: u64,
    /// unix-сек обновления hint
    pub updated_at: u64,
}

impl BlastHint {
    fn is_fresh(&self) -> bool {
        self.expires_at > now_unix()
    }
}

/// RFC3339 → unix-сек.
fn rfc3339_to_unix(s: &str) -> Option<u64> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.timestamp().max(0) as u64)
}

/// Прямые callers символа (глубина 1) по usage-рёбрам, до `limit`.
fn direct_callers(conn: &Connection, node_id: i64, limit: usize) -> rusqlite::Result<Vec<String>> {
    let labels = super::callpath::USAGE_EDGE_LABELS
        .iter()
        .map(|l| format!("'{l}'"))
        .collect::<Vec<_>>()
        .join(",");
    let mut stmt = conn.prepare(&format!(
        "SELECT DISTINCT s.label
         FROM graph_edges e
         JOIN graph_nodes s ON s.id = e.source_id
         WHERE e.label IN ({labels}) AND e.deleted_at IS NULL
           AND e.target_id = ?1 AND s.deleted_at IS NULL
         ORDER BY s.label LIMIT {limit}"
    ))?;
    let rows = stmt.query_map(params![node_id], |r| r.get::<_, String>(0))?;
    rows.collect()
}

/// Свежий hint из kv-кэша (если есть).
fn cached_hint(conn: &Connection, project_id: &str) -> rusqlite::Result<Option<BlastHint>> {
    let json: Option<String> = conn
        .query_row(
            "SELECT value FROM kv WHERE key = ?1",
            params![format!("blast_hint:{project_id}")],
            |r| r.get(0),
        )
        .optional()?;
    let hint: Option<BlastHint> = json.and_then(|j| serde_json::from_str(&j).ok());
    Ok(hint.filter(|h| h.is_fresh()))
}

/// Вычислить hint по последнему изменённому символу.
fn compute_hint(conn: &Connection, project_id: &str) -> rusqlite::Result<Option<BlastHint>> {
    let mut stmt = conn.prepare(
        "SELECT id, label, file_path, updated_at
         FROM graph_nodes
         WHERE project_id = ?1 AND deleted_at IS NULL
           AND node_type IN ('Function', 'Struct', 'Class', 'Trait', 'Method')
         ORDER BY updated_at DESC, id ASC LIMIT 5",
    )?;
    let rows = stmt
        .query_map(params![project_id], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let now = now_unix();
    let mut fallback: Option<BlastHint> = None;
    for (node_id, label, file_path, updated_at) in rows {
        let Some(updated) = rfc3339_to_unix(&updated_at) else {
            continue;
        };
        if now.saturating_sub(updated) > BLAST_TTL_SECS {
            break;
        }
        let callers = direct_callers(conn, node_id, BLAST_CALLERS)?;
        let hint = BlastHint {
            project_id: project_id.to_string(),
            symbol: label,
            file_path,
            callers,
            expires_at: (updated + BLAST_TTL_SECS).min(now + BLAST_TTL_SECS),
            updated_at: updated,
        };
        if !hint.callers.is_empty() {
            return Ok(Some(hint));
        }
        if fallback.is_none() {
            fallback = Some(hint);
        }
    }
    Ok(fallback)
}

/// Warn-блок для ресурса / подсказки радиуса поражения.
pub fn warn_block(conn: &Connection, active_project: Option<&str>, warn_enabled: bool) -> String {
    if !warn_enabled {
        return String::new();
    }
    let Some(project_id) = active_project else {
        return String::new();
    };
    let run = || -> rusqlite::Result<Option<BlastHint>> {
        match cached_hint(conn, project_id)? {
            Some(h) => Ok(Some(h)),
            None => match compute_hint(conn, project_id)? {
                Some(h) => {
                    if let Ok(json) = serde_json::to_string(&h) {
                        let _ = conn.execute(
                            "INSERT OR REPLACE INTO kv (key, value) VALUES (?1, ?2)",
                            params![format!("blast_hint:{project_id}"), json],
                        );
                    }
                    Ok(Some(h))
                }
                None => Ok(None),
            },
        }
    };
    let hint = match run() {
        Ok(h) => h,
        Err(_) => return String::new(),
    };
    let Some(hint) = hint else {
        return String::new();
    };
    let loc = hint.file_path.as_deref().unwrap_or("");
    if hint.callers.is_empty() {
        format!(
            "[blast-radius] правка `{}` ({}) — входящих связей не найдено",
            hint.symbol, loc
        )
    } else {
        format!(
            "[blast-radius] правка `{}` ({}): затронуты {}",
            hint.symbol,
            loc,
            hint.callers.join(", ")
        )
    }
}

/// Прочитать последний hint как структуру.
pub fn latest_hint(conn: &Connection, project_id: &str) -> rusqlite::Result<Option<BlastHint>> {
    cached_hint(conn, project_id)
}
