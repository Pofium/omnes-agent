//! Модуль интеграции и гибридной синергии с CodeGraph (.codegraph/codegraph.db).
//! Обеспечивает прямое чтение индексированных символов AST, иерархии вызовов
//! и связей из локальной базы SQLite CodeGraph в граф знаний KAG (graph_nodes, graph_edges).

use std::collections::HashMap;
use std::path::Path;
use chrono::Utc;
use rusqlite::{params, Connection, OpenFlags};
use tracing::{info, warn};

/// Результат синхронизации данных из базы CodeGraph.
#[derive(Debug, Default, Clone)]
pub struct CodeGraphSyncResult {
    pub nodes_synced: usize,
    pub edges_synced: usize,
    pub db_found: bool,
}

/// Выполняет синхронизацию графа символов и вызовов из .codegraph/codegraph.db в хранилище KAG.
pub fn sync_codegraph_db(
    project_id: &str,
    project_root: &Path,
    conn: &mut Connection,
) -> anyhow::Result<CodeGraphSyncResult> {
    let codegraph_db_path = project_root.join(".codegraph").join("codegraph.db");
    if !codegraph_db_path.exists() {
        return Ok(CodeGraphSyncResult {
            nodes_synced: 0,
            edges_synced: 0,
            db_found: false,
        });
    }

    info!(
        "CodeGraph Bridge: обнаружена база {} для проекта '{}'",
        codegraph_db_path.display(),
        project_id
    );

    // Открываем базу CodeGraph исключительно в режиме чтения
    let cg_conn = match Connection::open_with_flags(
        &codegraph_db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(e) => {
            warn!("Не удалось открыть .codegraph/codegraph.db (возможно, база заблокирована или пишется): {}", e);
            return Ok(CodeGraphSyncResult {
                nodes_synced: 0,
                edges_synced: 0,
                db_found: true,
            });
        }
    };

    // 1. Читаем узлы из codegraph.db
    struct CgNode {
        id: String,
        kind: String,
        name: String,
        qualified_name: String,
        file_path: String,
        start_line: i64,
        end_line: i64,
        docstring: Option<String>,
        signature: Option<String>,
    }

    let mut nodes_stmt = cg_conn.prepare(
        r#"
        SELECT id, kind, name, qualified_name, file_path, start_line, end_line, docstring, signature
        FROM nodes
        "#,
    )?;

    let cg_nodes: Vec<CgNode> = nodes_stmt
        .query_map([], |row| {
            Ok(CgNode {
                id: row.get(0)?,
                kind: row.get(1)?,
                name: row.get(2)?,
                qualified_name: row.get(3)?,
                file_path: row.get(4)?,
                start_line: row.get(5)?,
                end_line: row.get(6)?,
                docstring: row.get(7)?,
                signature: row.get(8)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // 2. Читаем связи (рёбра) из codegraph.db
    struct CgEdge {
        source: String,
        target: String,
        kind: String,
        metadata: Option<String>,
        line: Option<i64>,
    }

    let mut edges_stmt = cg_conn.prepare(
        r#"
        SELECT source, target, kind, metadata, line
        FROM edges
        "#,
    )?;

    let cg_edges: Vec<CgEdge> = edges_stmt
        .query_map([], |row| {
            Ok(CgEdge {
                source: row.get(0)?,
                target: row.get(1)?,
                kind: row.get(2)?,
                metadata: row.get(3)?,
                line: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    drop(nodes_stmt);
    drop(edges_stmt);
    drop(cg_conn);

    if cg_nodes.is_empty() {
        return Ok(CodeGraphSyncResult {
            nodes_synced: 0,
            edges_synced: 0,
            db_found: true,
        });
    }

    let now = Utc::now().to_rfc3339();
    let tx = conn.transaction()?;

    // Очищаем предыдущие codegraph-узлы проекта для предотвращения дублирования
    tx.execute(
        "DELETE FROM graph_nodes WHERE project_id = ?1 AND provenance = 'codegraph'",
        params![project_id],
    )?;
    tx.execute(
        "DELETE FROM graph_edges WHERE project_id = ?1 AND provenance = 'codegraph'",
        params![project_id],
    )?;

    // 3. Сохраняем узлы в graph_nodes
    let mut cg_id_to_pk: HashMap<String, i64> = HashMap::with_capacity(cg_nodes.len());

    {
        let mut insert_node_stmt = tx.prepare(
            r#"
            INSERT INTO graph_nodes (
                node_id, label, node_type, description, project_id, file_path,
                line_start, line_end, provenance, confidence, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'codegraph', 1.0, ?9, ?9)
            ON CONFLICT(node_id) DO UPDATE SET
                label = excluded.label,
                node_type = excluded.node_type,
                description = excluded.description,
                project_id = excluded.project_id,
                file_path = excluded.file_path,
                line_start = excluded.line_start,
                line_end = excluded.line_end,
                provenance = 'codegraph',
                updated_at = excluded.updated_at
            RETURNING id
            "#,
        )?;

        for node in &cg_nodes {
            let scoped_node_id = format!("{}:cg:{}", project_id, node.id);
            let label = if !node.qualified_name.is_empty() {
                &node.qualified_name
            } else {
                &node.name
            };

            // Описание: docstring или signature
            let desc = match (&node.docstring, &node.signature) {
                (Some(doc), Some(sig)) if !doc.is_empty() && !sig.is_empty() => {
                    Some(format!("{}\n{}", sig, doc))
                }
                (Some(doc), _) if !doc.is_empty() => Some(doc.clone()),
                (_, Some(sig)) if !sig.is_empty() => Some(sig.clone()),
                _ => None,
            };

            if let Ok(pk) = insert_node_stmt.query_row(
                params![
                    scoped_node_id,
                    label,
                    node.kind,
                    desc,
                    project_id,
                    node.file_path,
                    node.start_line,
                    node.end_line,
                    now,
                ],
                |r| r.get::<_, i64>(0),
            ) {
                cg_id_to_pk.insert(node.id.clone(), pk);
            }
        }
    }

    // 4. Сохраняем связи в graph_edges
    let mut edges_inserted = 0;
    {
        let mut insert_edge_stmt = tx.prepare(
            r#"
            INSERT INTO graph_edges (
                source_id, target_id, label, weight, contexts,
                project_id, provenance, confidence, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'codegraph', 1.0, ?7, ?7)
            ON CONFLICT(source_id, target_id, label) DO UPDATE SET
                weight = excluded.weight,
                contexts = excluded.contexts,
                provenance = 'codegraph',
                confidence = excluded.confidence,
                updated_at = excluded.updated_at
            "#,
        )?;

        for edge in &cg_edges {
            if let (Some(&src_pk), Some(&dst_pk)) = (
                cg_id_to_pk.get(&edge.source),
                cg_id_to_pk.get(&edge.target),
            ) {
                let label = edge.kind.to_uppercase();
                let context_json = if let Some(meta) = &edge.metadata {
                    meta.clone()
                } else if let Some(line) = edge.line {
                    format!(r#"{{"line":{}}}"#, line)
                } else {
                    "{}".to_string()
                };

                if insert_edge_stmt
                    .execute(params![
                        src_pk,
                        dst_pk,
                        label,
                        1.0f64,
                        context_json,
                        project_id,
                        now,
                    ])
                    .is_ok()
                {
                    edges_inserted += 1;
                }
            }
        }
    }

    tx.commit()?;

    info!(
        "CodeGraph Bridge: успешно синхронизировано {} узлов и {} связей для проекта '{}'",
        cg_id_to_pk.len(),
        edges_inserted,
        project_id
    );

    Ok(CodeGraphSyncResult {
        nodes_synced: cg_id_to_pk.len(),
        edges_synced: edges_inserted,
        db_found: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_sync_codegraph_db_success() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let cg_dir = dir.path().join(".codegraph");
        std::fs::create_dir_all(&cg_dir)?;

        // Создаем mock codegraph.db
        let cg_db_path = cg_dir.join("codegraph.db");
        let cg_conn = Connection::open(&cg_db_path)?;
        cg_conn.execute_batch(
            r#"
            CREATE TABLE nodes (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                name TEXT NOT NULL,
                qualified_name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                language TEXT NOT NULL,
                start_line INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                docstring TEXT,
                signature TEXT
            );
            CREATE TABLE edges (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                kind TEXT NOT NULL,
                metadata TEXT,
                line INTEGER
            );
            "#,
        )?;

        cg_conn.execute_batch(
            r#"
            INSERT INTO nodes VALUES 
            ('fn::scanner::scan', 'function', 'scan', 'scanner::scan', 'src/scanner.rs', 'rust', 10, 25, 'Сканирует директорию', 'fn scan() -> bool'),
            ('fn::parser::parse', 'function', 'parse', 'parser::parse', 'src/parser.rs', 'rust', 30, 45, 'Парсит AST', 'fn parse() -> Ast');
            "#,
        )?;

        cg_conn.execute(
            r#"
            INSERT INTO edges (source, target, kind, metadata, line)
            VALUES ('fn::scanner::scan', 'fn::parser::parse', 'calls', '{"type":"call"}', 15);
            "#,
            [],
        )?;
        drop(cg_conn);

        // Инициализируем БД KAG
        let mut kag_conn = Connection::open_in_memory()?;
        crate::schema::migrate(&kag_conn)?;

        // Регистрируем проект в KAG
        kag_conn.execute(
            r#"
            INSERT INTO projects (id, name, root_path, created_at, updated_at)
            VALUES ('proj-1', 'Test', '/test', '2026-01-01', '2026-01-01')
            "#,
            [],
        )?;

        let res = sync_codegraph_db("proj-1", dir.path(), &mut kag_conn)?;
        assert!(res.db_found);
        assert_eq!(res.nodes_synced, 2);
        assert_eq!(res.edges_synced, 1);

        // Проверяем graph_nodes
        let count_nodes: i64 = kag_conn.query_row(
            "SELECT COUNT(*) FROM graph_nodes WHERE project_id = 'proj-1' AND provenance = 'codegraph'",
            [],
            |r| r.get(0),
        )?;
        assert_eq!(count_nodes, 2);

        // Проверяем graph_edges
        let count_edges: i64 = kag_conn.query_row(
            "SELECT COUNT(*) FROM graph_edges WHERE project_id = 'proj-1' AND provenance = 'codegraph' AND label = 'CALLS'",
            [],
            |r| r.get(0),
        )?;
        assert_eq!(count_edges, 1);

        Ok(())
    }

    #[test]
    fn test_sync_codegraph_db_missing_returns_gracefully() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let mut kag_conn = Connection::open_in_memory()?;
        let res = sync_codegraph_db("proj-none", dir.path(), &mut kag_conn)?;
        assert!(!res.db_found);
        assert_eq!(res.nodes_synced, 0);
        assert_eq!(res.edges_synced, 0);
        Ok(())
    }
}
