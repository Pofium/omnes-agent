//! Инкрементальный скан не должен терять межфайловые CALLS-рёбра
//! (порт теста ob2h `0c79b95`).

use omnesagent_kag::{migrate, ProjectService};
use parking_lot::Mutex;
use rusqlite::{Connection, Result as SqlResult};
use std::sync::Arc;
use tempfile::tempdir;

/// Число RESOLVED-CALLS-рёбер в графе.
fn resolved_calls(conn: &Arc<Mutex<Connection>>) -> SqlResult<i64> {
    conn.lock().query_row(
        "SELECT COUNT(*) FROM graph_edges WHERE label = 'CALLS' AND provenance = 'RESOLVED'",
        [],
        |r| r.get(0),
    )
}

#[test]
fn test_incremental_crossfile_calls_preserved() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let proj_dir = dir.path().join("proj");
    let src_dir = proj_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    let file_a = src_dir.join("a.rs");
    let file_b = src_dir.join("b.rs");
    std::fs::write(
        &file_a,
        "use crate::b::helper;\n\npub fn run_app() {\n    helper();\n}\n",
    )?;
    std::fs::write(&file_b, "pub fn helper() -> i32 {\n    42\n}\n")?;

    let conn = Arc::new(Mutex::new(Connection::open(dir.path().join("test.db"))?));
    migrate(&conn.lock())?;
    let service = ProjectService::new(conn.clone());

    service.register_project(
        "test_proj",
        "Test Project",
        &proj_dir.to_string_lossy(),
        None,
        None,
    )?;

    // 1. Полное сканирование: helper() из a.rs резолвится в b.rs.
    let scan1 = service.scan_project("test_proj", None, false)?;
    eprintln!("DIAG scan1 files={} nodes={}", scan1.files_scanned, scan1.nodes.len());
    assert_eq!(scan1.files_scanned, 2);
    assert_eq!(
        resolved_calls(&conn)?,
        1,
        "ожидалось 1 CALLS-ребро после полного скана"
    );

    // 2. Правим b.rs (helper остаётся, добавляется extra).
    std::fs::write(&file_b, "pub fn helper() -> i32 {\n    42\n}\n\npub fn extra() {}\n")?;

    // 3. Инкрементальный скан: a.rs не пересканируется, но его вызовы
    // подгружаются из ast_refs и перерезолвливаются.
    let scan2 = service.scan_project("test_proj", None, true)?;
    assert_eq!(scan2.files_scanned, 1);
    assert_eq!(
        resolved_calls(&conn)?,
        1,
        "CALLS-ребро не должно исчезать после инкрементального скана"
    );

    Ok(())
}
