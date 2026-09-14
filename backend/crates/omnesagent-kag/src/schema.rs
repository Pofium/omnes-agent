//! Унифицированная схема SQLite и версионные миграции.

use rusqlite::{Connection, Result, params};

pub const SCHEMA_VERSION: i64 = 6;

pub const MIGRATION_BASE: &str = r#"
CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT NOT NULL);

CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  root_path TEXT NOT NULL,
  description TEXT,
  tech_stack TEXT,
  active_branch TEXT,
  last_scanned_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_projects_root_path ON projects(root_path);

CREATE TABLE IF NOT EXISTS project_files (
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  rel_path TEXT NOT NULL,
  sha256 TEXT NOT NULL,
  file_size INTEGER NOT NULL,
  lines_count INTEGER NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (project_id, rel_path)
);
CREATE INDEX IF NOT EXISTS idx_project_files_project ON project_files(project_id);

CREATE TABLE IF NOT EXISTS memories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  key TEXT UNIQUE NOT NULL,
  content TEXT NOT NULL,
  category TEXT NOT NULL DEFAULT 'general',
  importance REAL NOT NULL DEFAULT 0.5,
  source TEXT DEFAULT 'manual',
  namespace TEXT NOT NULL DEFAULT 'default',
  agent_id TEXT,
  project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
  meta TEXT,
  embedding BLOB,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  access_count INTEGER NOT NULL DEFAULT 0,
  last_accessed TEXT,
  trust REAL NOT NULL DEFAULT 0.5,
  last_feedback_at TEXT,
  origin TEXT NOT NULL DEFAULT '',
  deleted_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_memories_cat_imp ON memories (category, importance DESC);
CREATE INDEX IF NOT EXISTS idx_memories_project ON memories (project_id);
CREATE INDEX IF NOT EXISTS idx_memories_namespace ON memories (namespace);
CREATE INDEX IF NOT EXISTS idx_memories_agent ON memories (agent_id);

CREATE TABLE IF NOT EXISTS memory_relations (
  source_key TEXT NOT NULL,
  target_key TEXT NOT NULL,
  relation_type TEXT NOT NULL,
  weight REAL NOT NULL DEFAULT 1.0,
  deleted_at TEXT,
  UNIQUE (source_key, target_key, relation_type)
);
CREATE INDEX IF NOT EXISTS idx_memory_relations_src ON memory_relations (source_key);
CREATE INDEX IF NOT EXISTS idx_memory_relations_tgt ON memory_relations (target_key);

CREATE TABLE IF NOT EXISTS documents (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT,
  path TEXT,
  meta TEXT,
  project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_documents_project ON documents(project_id);

CREATE TABLE IF NOT EXISTS chunks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  doc_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  ordinal INTEGER NOT NULL,
  text TEXT NOT NULL,
  embedding BLOB,
  project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_chunks_doc ON chunks (doc_id);
CREATE INDEX IF NOT EXISTS idx_chunks_project ON chunks (project_id);

CREATE TABLE IF NOT EXISTS graph_nodes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  node_id TEXT UNIQUE NOT NULL,
  label TEXT NOT NULL,
  node_type TEXT NOT NULL,
  description TEXT,
  val INTEGER NOT NULL DEFAULT 1,
  embedding BLOB,
  project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
  file_path TEXT,
  line_start INTEGER,
  line_end INTEGER,
  provenance TEXT NOT NULL DEFAULT 'manual',
  confidence REAL NOT NULL DEFAULT 1.0,
  is_god_node INTEGER NOT NULL DEFAULT 0,
  origin TEXT NOT NULL DEFAULT '',
  deleted_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_graph_nodes_label ON graph_nodes (label);
CREATE INDEX IF NOT EXISTS idx_graph_nodes_type ON graph_nodes (node_type);
CREATE INDEX IF NOT EXISTS idx_graph_nodes_project ON graph_nodes (project_id);
CREATE INDEX IF NOT EXISTS idx_graph_nodes_provenance ON graph_nodes (provenance);

CREATE TABLE IF NOT EXISTS graph_edges (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source_id INTEGER NOT NULL REFERENCES graph_nodes(id) ON DELETE CASCADE,
  target_id INTEGER NOT NULL REFERENCES graph_nodes(id) ON DELETE CASCADE,
  label TEXT NOT NULL,
  weight REAL NOT NULL DEFAULT 1.0,
  contexts TEXT,
  project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
  provenance TEXT NOT NULL DEFAULT 'manual',
  confidence REAL NOT NULL DEFAULT 1.0,
  origin TEXT NOT NULL DEFAULT '',
  deleted_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL DEFAULT '',
  UNIQUE (source_id, target_id, label)
);
CREATE INDEX IF NOT EXISTS idx_graph_edges_src ON graph_edges (source_id);
CREATE INDEX IF NOT EXISTS idx_graph_edges_dst ON graph_edges (target_id);
CREATE INDEX IF NOT EXISTS idx_graph_edges_project ON graph_edges (project_id);

CREATE TABLE IF NOT EXISTS dream_runs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  started_at TEXT,
  finished_at TEXT,
  status TEXT,
  trigger TEXT,
  phase_log TEXT,
  stats TEXT
);

CREATE TABLE IF NOT EXISTS ralph_runs (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  feature_slug TEXT NOT NULL,
  goal TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'proposed',
  autonomy TEXT NOT NULL DEFAULT 'L1',
  max_iterations_per_task INTEGER NOT NULL DEFAULT 5,
  max_total_iterations INTEGER NOT NULL DEFAULT 60,
  budget_tokens INTEGER,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  finished_at TEXT,
  stop_reason TEXT,
  UNIQUE (project_id, feature_slug)
);

CREATE TABLE IF NOT EXISTS ralph_iterations (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL REFERENCES ralph_runs(id) ON DELETE CASCADE,
  task_id TEXT NOT NULL,
  n INTEGER NOT NULL,
  git_before TEXT,
  git_after TEXT,
  hypothesis TEXT,
  plan TEXT,
  result TEXT,
  tests_summary TEXT,
  verdict TEXT NOT NULL DEFAULT 'unconfirmed'
    CHECK (verdict IN ('verified','failed','unconfirmed','overturned','stale')),
  verdict_source TEXT,
  ladder_rung TEXT,
  context_ref TEXT,
  tokens_used INTEGER,
  duration_ms INTEGER,
  created_at TEXT NOT NULL,
  UNIQUE (run_id, task_id, n)
);
CREATE INDEX IF NOT EXISTS idx_ralph_iterations_run ON ralph_iterations(run_id, task_id, n);

CREATE TABLE IF NOT EXISTS ralph_findings (
  id TEXT PRIMARY KEY,
  run_id TEXT REFERENCES ralph_runs(id) ON DELETE SET NULL,
  iteration_id TEXT REFERENCES ralph_iterations(id) ON DELETE SET NULL,
  project_id TEXT REFERENCES projects(id),
  kind TEXT NOT NULL,
  content TEXT NOT NULL,
  symbols TEXT NOT NULL DEFAULT '[]',
  verdict TEXT NOT NULL DEFAULT 'unconfirmed'
    CHECK (verdict IN ('verified','failed','unconfirmed','overturned','stale')),
  verdict_source TEXT,
  embedding BLOB,
  meta TEXT,
  created_at TEXT NOT NULL,
  stale_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_ralph_findings_proj ON ralph_findings(project_id, verdict);

CREATE TABLE IF NOT EXISTS ast_changes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  run_id TEXT REFERENCES ralph_runs(id) ON DELETE SET NULL,
  iteration_id TEXT REFERENCES ralph_iterations(id) ON DELETE SET NULL,
  path TEXT NOT NULL,
  node_key TEXT NOT NULL,
  label TEXT NOT NULL,
  node_type TEXT NOT NULL,
  change_type TEXT NOT NULL,
  sig_before TEXT,
  sig_after TEXT,
  loc_delta INTEGER DEFAULT 0,
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ast_changes_project ON ast_changes(project_id);

CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
  text, content='chunks', content_rowid='rowid', tokenize='trigram'
);
CREATE TRIGGER IF NOT EXISTS chunks_fts_ai AFTER INSERT ON chunks BEGIN
  INSERT INTO chunks_fts (rowid, text) VALUES (new.rowid, new.text);
END;
CREATE TRIGGER IF NOT EXISTS chunks_fts_ad AFTER DELETE ON chunks BEGIN
  INSERT INTO chunks_fts (chunks_fts, rowid, text) VALUES ('delete', old.rowid, old.text);
END;
CREATE TRIGGER IF NOT EXISTS chunks_fts_au AFTER UPDATE ON chunks BEGIN
  INSERT INTO chunks_fts (chunks_fts, rowid, text) VALUES ('delete', old.rowid, old.text);
  INSERT INTO chunks_fts (rowid, text) VALUES (new.rowid, new.text);
END;
"#;

pub fn schema_version(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT value FROM kv WHERE key = 'schema_version'",
        [],
        |row| {
            let val: String = row.get(0)?;
            Ok(val.parse::<i64>().unwrap_or(0))
        },
    )
    .unwrap_or(0)
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> bool {
    let query = format!("PRAGMA table_info({})", table);
    if let Ok(mut stmt) = conn.prepare(&query)
        && let Ok(mut rows) = stmt.query([])
    {
        while let Ok(Some(row)) = rows.next() {
            if let Ok(name) = row.get::<_, String>(1)
                && name.eq_ignore_ascii_case(column)
            {
                return true;
            }
        }
    }
    false
}

/// Применяет миграции схемы SQLite. Гарантирует обратную совместимость и сохранение данных.
pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA synchronous=NORMAL;",
    )?;

    // Если таблица memories уже существует от старого OmnesAgent, проверяем и добавляем недостающие колонки
    let table_exists: bool = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='memories'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    if table_exists {
        if !table_has_column(conn, "memories", "origin") {
            let _ = conn.execute(
                "ALTER TABLE memories ADD COLUMN origin TEXT NOT NULL DEFAULT ''",
                [],
            );
        }
        if !table_has_column(conn, "memories", "deleted_at") {
            let _ = conn.execute("ALTER TABLE memories ADD COLUMN deleted_at TEXT", []);
        }
        if !table_has_column(conn, "memories", "project_id") {
            let _ = conn.execute("ALTER TABLE memories ADD COLUMN project_id TEXT", []);
        }
        if !table_has_column(conn, "memories", "namespace") {
            let _ = conn.execute(
                "ALTER TABLE memories ADD COLUMN namespace TEXT NOT NULL DEFAULT 'default'",
                [],
            );
        }
        if !table_has_column(conn, "memories", "source") {
            let _ = conn.execute(
                "ALTER TABLE memories ADD COLUMN source TEXT DEFAULT 'manual'",
                [],
            );
        }
        if !table_has_column(conn, "memories", "agent_id") {
            // Required by idx_memories_agent in MIGRATION_BASE; without this
            // ALTER the legacy upgrade path fails with "no such column".
            let _ = conn.execute("ALTER TABLE memories ADD COLUMN agent_id TEXT", []);
        }
        if !table_has_column(conn, "memories", "access_count") {
            let _ = conn.execute(
                "ALTER TABLE memories ADD COLUMN access_count INTEGER NOT NULL DEFAULT 0",
                [],
            );
        }
        if !table_has_column(conn, "memories", "last_accessed") {
            let _ = conn.execute("ALTER TABLE memories ADD COLUMN last_accessed TEXT", []);
        }
        if !table_has_column(conn, "memories", "trust") {
            let _ = conn.execute(
                "ALTER TABLE memories ADD COLUMN trust REAL NOT NULL DEFAULT 0.5",
                [],
            );
        }
        if !table_has_column(conn, "memories", "last_feedback_at") {
            let _ = conn.execute("ALTER TABLE memories ADD COLUMN last_feedback_at TEXT", []);
        }
    }

    let rel_exists: bool = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='memory_relations'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);
    if rel_exists && !table_has_column(conn, "memory_relations", "deleted_at") {
        let _ = conn.execute("ALTER TABLE memory_relations ADD COLUMN deleted_at TEXT", []);
    }

    conn.execute_batch(MIGRATION_BASE)?;

    let has_memories_fts: bool = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='memories_fts'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    if !has_memories_fts {
        conn.execute_batch(
            r#"
CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
  content, content='memories', content_rowid='rowid', tokenize='trigram'
);
CREATE TRIGGER IF NOT EXISTS memories_fts_ai AFTER INSERT ON memories BEGIN
  INSERT INTO memories_fts (rowid, content) VALUES (new.rowid, new.content);
END;
CREATE TRIGGER IF NOT EXISTS memories_fts_ad AFTER DELETE ON memories BEGIN
  INSERT INTO memories_fts (memories_fts, rowid, content) VALUES ('delete', old.rowid, old.content);
END;
CREATE TRIGGER IF NOT EXISTS memories_fts_au AFTER UPDATE ON memories BEGIN
  INSERT INTO memories_fts (memories_fts, rowid, content) VALUES ('delete', old.rowid, old.content);
  INSERT INTO memories_fts (rowid, content) VALUES (new.rowid, new.content);
END;
"#,
        )?;
    }

    conn.execute(
        "INSERT OR REPLACE INTO kv (key, value) VALUES ('schema_version', ?1)",
        params![SCHEMA_VERSION.to_string()],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_names(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .expect("prepare sqlite_master query");
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query sqlite_master");
        rows.collect::<Result<Vec<_>>>().expect("collect tables")
    }

    #[test]
    fn migrate_on_fresh_db_creates_full_schema_and_stamps_version() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        assert_eq!(schema_version(&conn), SCHEMA_VERSION);
        let tables = table_names(&conn);
        for expected in [
            "kv",
            "projects",
            "project_files",
            "memories",
            "memory_relations",
            "documents",
            "chunks",
            "graph_nodes",
            "graph_edges",
            "dream_runs",
            "memories_fts",
            "chunks_fts",
        ] {
            assert!(
                tables.iter().any(|t| t == expected),
                "missing table {expected} after migrate; got {tables:?}"
            );
        }
    }

    #[test]
    fn migrate_is_idempotent_and_preserves_rows() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn.execute(
            "INSERT INTO memories (key, content, category, importance, namespace, created_at, updated_at)
             VALUES ('k', 'v', 'general', 0.9, 'default', 't0', 't0')",
            [],
        )
        .unwrap();
        migrate(&conn).unwrap();
        assert_eq!(schema_version(&conn), SCHEMA_VERSION);
        let count: i64 = conn
            .query_row("SELECT count(*) FROM memories", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "re-migrate must not duplicate or drop rows");
    }

    #[test]
    fn fts_triggers_keep_the_search_index_in_sync() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn.execute(
            "INSERT INTO memories (key, content, category, importance, namespace, created_at, updated_at)
             VALUES ('a', 'квантовый лифт памятки', 'general', 0.5, 'default', 't', 't')",
            [],
        )
        .unwrap();
        let hits: i64 = conn
            .query_row(
                "SELECT count(*) FROM memories_fts WHERE memories_fts MATCH 'квантовый'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hits, 1, "insert trigger must index the content");

        conn.execute("DELETE FROM memories WHERE key = 'a'", [])
            .unwrap();
        let hits: i64 = conn
            .query_row(
                "SELECT count(*) FROM memories_fts WHERE memories_fts MATCH 'квантовый'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hits, 0, "delete trigger must unindex the content");
    }

    #[test]
    fn migrate_upgrades_legacy_memories_table_without_losing_rows() {
        let conn = Connection::open_in_memory().unwrap();
        // Pre-v2 layout: a memories table without any of the columns later
        // revisions added. migrate() must ALTER them in and keep the data.
        conn.execute_batch(
            "CREATE TABLE memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT UNIQUE NOT NULL,
                content TEXT NOT NULL,
                category TEXT NOT NULL,
                importance REAL NOT NULL DEFAULT 0.5,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            INSERT INTO memories (key, content, category, importance, created_at, updated_at)
            VALUES ('legacy', 'old row', 'general', 0.5, 't0', 't0');",
        )
        .unwrap();

        migrate(&conn).unwrap();
        assert_eq!(schema_version(&conn), SCHEMA_VERSION);
        for column in [
            "origin",
            "deleted_at",
            "project_id",
            "namespace",
            "source",
            "access_count",
            "last_accessed",
            "trust",
            "last_feedback_at",
        ] {
            assert!(
                table_has_column(&conn, "memories", column),
                "legacy table must gain column {column}"
            );
        }
        let (origin, namespace, access_count): (String, String, i64) = conn
            .query_row(
                "SELECT origin, namespace, access_count FROM memories WHERE key = 'legacy'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(origin, "");
        assert_eq!(namespace, "default");
        assert_eq!(access_count, 0);
    }

    #[test]
    fn schema_version_returns_zero_for_missing_or_garbage_marker() {
        let conn = Connection::open_in_memory().unwrap();
        assert_eq!(schema_version(&conn), 0, "no kv row at all");
        migrate(&conn).unwrap();
        conn.execute(
            "UPDATE kv SET value = 'not-a-number' WHERE key = 'schema_version'",
            [],
        )
        .unwrap();
        assert_eq!(schema_version(&conn), 0, "unparseable version marker");
    }
}
