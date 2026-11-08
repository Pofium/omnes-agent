//! Тесты задач Ф1.2b (порт ob2h `2ad9027`):
//!  * сгенерированный protobuf (`*_pb2.py`, `*_pb2_grpc.py`) не сканируется;
//!  * полный рескан (incremental=false) пересобирает узлы даже без изменения файлов;
//!  * смена версии AST-экстрактора автоматически форсирует полный рескан.

use omnesagent_kag::ast::AstCodeExtractor;
use omnesagent_kag::project::AST_EXTRACTOR_VERSION;
use omnesagent_kag::{migrate, ProjectService};
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::sync::Arc;
use tempfile::tempdir;

/// Число узлов с заданной меткой в проекте.
fn label_count(conn: &Arc<Mutex<Connection>>, id: &str, label: &str) -> i64 {
    conn.lock()
        .query_row(
            "SELECT COUNT(*) FROM graph_nodes WHERE project_id = ?1 AND label = ?2",
            params![id, label],
            |r| r.get(0),
        )
        .expect("count")
}

/// Помечает узел `foo` как устаревший (`STALE`), чтобы проверить пересборку.
fn stale_the_node(conn: &Arc<Mutex<Connection>>, id: &str) {
    conn.lock()
        .execute(
            "UPDATE graph_nodes SET label='STALE' WHERE project_id = ?1 AND label='foo'",
            params![id],
        )
        .expect("stale");
}

/// Готовит временный проект с одним python-файлом и in-memory-подобной sqlite-БД.
fn setup_project(
    id: &str,
) -> Result<(tempfile::TempDir, Arc<Mutex<Connection>>, ProjectService), Box<dyn std::error::Error>>
{
    let dir = tempdir()?;
    let root = dir.path().join("proj");
    std::fs::create_dir_all(root.join("pkg"))?;
    std::fs::write(root.join("pkg/mod.py"), "def foo():\n    return 1\n")?;

    // БД держим вне сканируемого корня, чтобы файл БД не попадал в скан.
    let conn = Arc::new(Mutex::new(Connection::open(dir.path().join("kag.db"))?));
    migrate(&conn.lock())?;
    let svc = ProjectService::new(conn.clone());
    svc.register_project(id, "T", &root.to_string_lossy(), None, None)?;
    Ok((dir, conn, svc))
}

#[test]
fn generated_pb2_files_are_not_scanned() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let root = tmp.path();
    std::fs::write(root.join("handwritten.py"), "def real_fn():\n    return 1\n")?;
    std::fs::write(root.join("arks_pb2.py"), "def generated_fn():\n    return 1\n")?;
    std::fs::write(root.join("thing_pb2_grpc.py"), "def grpc_fn():\n    return 1\n")?;

    let extractor = AstCodeExtractor::new();
    let res = extractor.scan_directory(root, None);

    let files: Vec<&String> = res.file_hashes.keys().collect();
    assert!(files.iter().any(|f| f.ends_with("handwritten.py")));
    assert!(
        files.iter().all(|f| !f.contains("_pb2")),
        "сгенерированный protobuf не должен сканироваться: {files:?}"
    );
    assert!(res
        .nodes
        .iter()
        .all(|n| n.label != "generated_fn" && n.label != "grpc_fn"));
    Ok(())
}

#[test]
fn full_scan_rebuilds_nodes_without_file_changes() -> Result<(), Box<dyn std::error::Error>> {
    let (_tmp, conn, svc) = setup_project("fullp")?;
    svc.scan_project("fullp", None, false)?;
    assert_eq!(label_count(&conn, "fullp", "foo"), 1);

    stale_the_node(&conn, "fullp");
    assert_eq!(label_count(&conn, "fullp", "foo"), 0);

    // Полный рескан пересобирает узел даже без изменения файлов.
    svc.scan_project("fullp", None, false)?;
    assert_eq!(label_count(&conn, "fullp", "foo"), 1);
    assert_eq!(label_count(&conn, "fullp", "STALE"), 0);
    Ok(())
}

#[test]
fn extractor_version_bump_forces_full_rescan() -> Result<(), Box<dyn std::error::Error>> {
    let (_tmp, conn, svc) = setup_project("verp")?;

    // Первый инкрементальный скан фиксирует текущую версию экстрактора.
    svc.scan_project("verp", None, true)?;
    let v: String = conn.lock().query_row(
        "SELECT value FROM kv WHERE key = 'ast_extractor_version:verp'",
        [],
        |r| r.get(0),
    )?;
    assert_eq!(v, AST_EXTRACTOR_VERSION);

    // Узел «устарел» и версия в kv откатывается на несовпадающую.
    stale_the_node(&conn, "verp");
    conn.lock().execute(
        "INSERT OR REPLACE INTO kv (key, value) VALUES ('ast_extractor_version:verp', 'old')",
        [],
    )?;

    // Скан запрошен инкрементальный, но версия не совпала → авто-полный рескан.
    svc.scan_project("verp", None, true)?;
    assert_eq!(label_count(&conn, "verp", "foo"), 1);
    assert_eq!(label_count(&conn, "verp", "STALE"), 0);
    Ok(())
}
