//! Ф1.1 (план ob2h-интеграции): golden dual-scan — паритет встроенного
//! kag-сканера с внешним ob2h на фиксированном корпусе.
//!
//! * Корпус: `tests/golden/corpus/` (коммитится, неизменяемый).
//! * Эталон: `tests/golden/ob2h-<version>.expected.json` — вывод ob2h-сканера,
//!   перегоняется `tests/golden/regenerate.ps1` (изолированный OB2H_DATA_DIR).
//! * Метрики: symbol F1 (узлы) и call F1 (CALLS-рёбра) против эталона.
//!
//! Пороги — DoD Ф1: symbol F1 >= 0.95, call F1 >= 0.90.
//!
//! Golden-эталон снимается с внешнего ob2h (regex-сканер близнеца, дефолтная
//! сборка). Экспериментальный tree-sitter путь (`--features treesitter`) намеренно
//! резолвит больше вызовов (CALLS/type-resolve, Ф6.1), поэтому паритет с эталоном
//! ob2h v1.9.2 к нему не применяется — харнесс скоупится на дефолтную сборку.

#![cfg(not(feature = "treesitter"))]

use omnesagent_kag::{migrate, ProjectService};
use parking_lot::Mutex;
use rusqlite::Connection;
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

/// Порог symbol F1 из DoD Ф1.
const SYMBOL_F1_MIN: f64 = 0.95;
/// Порог call F1 из DoD Ф1.
const CALL_F1_MIN: f64 = 0.90;

/// Эталонный набор, выгруженный из ob2h (см. `export_ob2h.py`).
#[derive(Deserialize)]
struct GoldenBaseline {
    ob2h_version: String,
    symbols: Vec<String>,
    edges: Vec<String>,
}

/// Каталог golden-набора.
fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// Загружает единственный файл эталона `ob2h-*.expected.json`.
fn load_baseline() -> GoldenBaseline {
    let dir = golden_dir();
    let mut found = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("read golden dir") {
        let path = entry.expect("entry").path();
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name.starts_with("ob2h-") && name.ends_with(".expected.json") {
            found.push(path);
        }
    }
    assert_eq!(
        found.len(),
        1,
        "ожидается ровно один эталон ob2h-*.expected.json: {found:?}"
    );
    let text = std::fs::read_to_string(&found[0]).expect("read baseline");
    serde_json::from_str(&text).expect("parse baseline")
}

/// Сканирует корпус встроенным kag-сканером; возвращает (symbols, edges)
/// в той же нормализации, что и `export_ob2h.py`.
fn scan_corpus() -> (BTreeSet<String>, BTreeSet<String>) {
    let corpus = golden_dir().join("corpus");
    assert!(corpus.is_dir(), "корпус не найден: {}", corpus.display());

    let dir = tempdir().expect("tempdir");
    // БД держим вне сканируемого корня, чтобы файл БД не попадал в скан.
    let conn = Arc::new(Mutex::new(
        Connection::open(dir.path().join("kag.db")).expect("open db"),
    ));
    migrate(&conn.lock()).expect("migrate");
    let svc = ProjectService::new(conn.clone());
    svc.register_project("golden", "Golden", &corpus.to_string_lossy(), None, None)
        .expect("register");
    svc.scan_project("golden", None, false).expect("scan");

    let mut id_to_node: HashMap<i64, String> = HashMap::new();
    let mut symbols: BTreeSet<String> = BTreeSet::new();
    {
        let c = conn.lock();
        let mut stmt = c
            .prepare("SELECT id, node_id, provenance FROM graph_nodes WHERE project_id = 'golden'")
            .expect("prepare nodes");
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .expect("query nodes");
        for row in rows {
            let (id, node_id, prov) = row.expect("node row");
            // kag хранит node_id со scoped-префиксом `<project_id>:` (исправление
            // глобально-уникального node_id об2h); для сравнения структуры
            // префикс снимаем.
            let unscoped = node_id
                .strip_prefix("golden:")
                .map(str::to_string)
                .unwrap_or(node_id);
            id_to_node.insert(id, unscoped.clone());
            if prov == "ast" {
                symbols.insert(unscoped);
            }
        }
    }

    let mut edges: BTreeSet<String> = BTreeSet::new();
    {
        let c = conn.lock();
        let mut stmt = c
            .prepare(
                "SELECT source_id, target_id, label FROM graph_edges \
                 WHERE project_id = 'golden' AND provenance != 'codegraph'",
            )
            .expect("prepare edges");
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .expect("query edges");
        for row in rows {
            let (s, t, label) = row.expect("edge row");
            if let (Some(sn), Some(tn)) = (id_to_node.get(&s), id_to_node.get(&t)) {
                edges.insert(format!("{sn}\t{label}\t{tn}"));
            }
        }
    }

    (symbols, edges)
}

/// Precision / Recall / F1 по множествам.
fn prf(expected: &BTreeSet<String>, actual: &BTreeSet<String>) -> (f64, f64, f64) {
    let tp = expected.intersection(actual).count() as f64;
    let p = if actual.is_empty() {
        0.0
    } else {
        tp / actual.len() as f64
    };
    let r = if expected.is_empty() {
        0.0
    } else {
        tp / expected.len() as f64
    };
    let f = if p + r == 0.0 {
        0.0
    } else {
        2.0 * p * r / (p + r)
    };
    (p, r, f)
}

/// Подмножество ключей рёбер с заданной меткой (`CALLS`, `IMPORTS`, ...).
fn edges_with_label(edges: &BTreeSet<String>, label: &str) -> BTreeSet<String> {
    edges
        .iter()
        .filter(|e| e.split('\t').nth(1) == Some(label))
        .cloned()
        .collect()
}

#[test]
fn golden_dual_scan_parity() {
    let base = load_baseline();
    let expected_symbols: BTreeSet<String> = base.symbols.into_iter().collect();
    let expected_edges: BTreeSet<String> = base.edges.into_iter().collect();

    let (actual_symbols, actual_edges) = scan_corpus();

    let (sp, sr, sf) = prf(&expected_symbols, &actual_symbols);
    let expected_calls = edges_with_label(&expected_edges, "CALLS");
    let actual_calls = edges_with_label(&actual_edges, "CALLS");
    let (cp, cr, cf) = prf(&expected_calls, &actual_calls);

    let missing: Vec<&String> = expected_symbols.difference(&actual_symbols).collect();
    let extra: Vec<&String> = actual_symbols.difference(&expected_symbols).collect();
    eprintln!("ob2h v{}: symbol P={sp:.3} R={sr:.3} F1={sf:.3}", base.ob2h_version);
    eprintln!("  missing symbols: {missing:?}");
    eprintln!("  extra symbols:   {extra:?}");
    eprintln!("  call P={cp:.3} R={cr:.3} F1={cf:.3}");
    eprintln!("  missing calls: {:?}", expected_calls.difference(&actual_calls).collect::<Vec<_>>());
    eprintln!("  extra calls:   {:?}", actual_calls.difference(&expected_calls).collect::<Vec<_>>());

    assert!(sf >= SYMBOL_F1_MIN, "symbol F1 {sf:.3} < {SYMBOL_F1_MIN}");
    assert!(cf >= CALL_F1_MIN, "call F1 {cf:.3} < {CALL_F1_MIN}");
}
