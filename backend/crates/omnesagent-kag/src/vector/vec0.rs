//! vec0-инкремент (Ф6.3 ob2h v2.0 / Ф72b): KNN-индекс графовых эмбеддингов
//! поверх статически слинкованного `sqlite-vec` + точный SIMD-rescore
//! шортлиста по сохранённым блобам.
//!
//! Амальгамация `sqlite-vec` вендорена в `vendor/sqlite-vec/` и компилируется
//! `build.rs` с `SQLITE_CORE` (та же bundled SQLite, что в `rusqlite`);
//! расширение регистрируется на конкретном соединении через `sqlite3_vec_init`
//! — внешних `.dll` в рантайме не нужно.

use rusqlite::{Connection, OptionalExtension, params};

use super::similarity::{cosine, deserialize, serialize};

/// Размерность эмбеддингов, зафиксированная в vec0-таблице. Совпадает с
/// размерностью провайдера по умолчанию (`default_embedding_provider`, 384).
pub const VEC0_DIM: usize = 384;

/// Имя виртуальной таблицы vec0.
pub const VEC0_TABLE: &str = "vec_graph_nodes";

/// Размер f32-BLOB для вектора размерности [`VEC0_DIM`].
const VEC0_BLOB_BYTES: usize = VEC0_DIM * size_of::<f32>();

// Entry point вендоренного расширения `sqlite-vec` (собран с `SQLITE_CORE`,
// статически слинкован с bundled SQLite из `rusqlite`). Регистрирует модуль
// `vec0` на переданном соединении.
unsafe extern "C" {
    /// Регистрирует модуль `vec0` (и соседние функции расширения) на `db`.
    fn sqlite3_vec_init(
        db: *mut rusqlite::ffi::sqlite3,
        pz_err_msg: *mut *mut std::os::raw::c_char,
        api: *const rusqlite::ffi::sqlite3_api_routines,
    ) -> std::os::raw::c_int;
}

/// Регистрирует модуль `vec0` на конкретном соединении. Идемпотентно: повторный
/// вызов на том же соединении переопределяет модуль/функции без ошибки.
/// Возвращает `true` при успехе.
pub fn register(conn: &Connection) -> bool {
    // SAFETY: `sqlite3_vec_init` — статически слинкованная функция расширения,
    // собранного с `SQLITE_CORE`; дескриптор соединения принадлежит `conn`, а
    // `pzErrMsg`/`pApi` не используются при `SQLITE_CORE` и передаются null.
    unsafe {
        let rc = sqlite3_vec_init(conn.handle(), std::ptr::null_mut(), std::ptr::null());
        rc == rusqlite::ffi::SQLITE_OK
    }
}

/// Есть ли vec0-таблица в базе.
fn table_present(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
        params![VEC0_TABLE],
        |r| r.get::<_, i64>(0),
    )
    .map(|c| c > 0)
    .unwrap_or(false)
}

/// Создаёт vec0-таблицу (при необходимости) и разово наполняет её существующими
/// эмбеддингами `graph_nodes`. Возвращает `true`, если индекс доступен.
pub fn ensure_schema(conn: &Connection) -> rusqlite::Result<bool> {
    if !register(conn) {
        return Ok(false);
    }
    let fresh = !table_present(conn);
    conn.execute_batch(&format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS {VEC0_TABLE} USING vec0(\
           node_rowid INTEGER PRIMARY KEY, \
           embedding float[{VEC0_DIM}] distance_metric=cosine\
         );"
    ))?;
    if fresh {
        // Backfill: сырой f32-BLOB легаси-формата совпадает с vec0 `float[384]`.
        conn.execute(
            &format!(
                "INSERT INTO {VEC0_TABLE}(node_rowid, embedding) \
                 SELECT id, embedding FROM graph_nodes \
                 WHERE embedding IS NOT NULL AND length(embedding) = ?1"
            ),
            params![VEC0_BLOB_BYTES as i64],
        )?;
    }
    Ok(true)
}

/// Best-effort инициализация индекса при миграции схемы (регистрация + таблица).
/// Никогда не паникует и не валит миграцию: при недоступности расширения
/// возвращает `false`, и вызывающий код работает на прежнем полном переборе.
pub fn init(conn: &Connection) -> bool {
    ensure_schema(conn).unwrap_or(false)
}

/// Синхронизирует строку vec0-индекса с эмбеддингом узла графа. Тихий no-op,
/// если индекс недоступен или размерность блоба не [`VEC0_DIM`].
pub fn sync_node(conn: &Connection, rowid: i64, embedding_blob: &[u8]) -> rusqlite::Result<()> {
    if embedding_blob.len() != VEC0_BLOB_BYTES || !table_present(conn) {
        return Ok(());
    }
    conn.execute(
        &format!("DELETE FROM {VEC0_TABLE} WHERE node_rowid = ?1"),
        params![rowid],
    )?;
    conn.execute(
        &format!("INSERT INTO {VEC0_TABLE}(node_rowid, embedding) VALUES (?1, ?2)"),
        params![rowid, embedding_blob],
    )?;
    Ok(())
}

/// Удаляет строку vec0-индекса (тихий no-op, если таблицы нет).
pub fn delete_node(conn: &Connection, rowid: i64) -> rusqlite::Result<()> {
    if !table_present(conn) {
        return Ok(());
    }
    conn.execute(
        &format!("DELETE FROM {VEC0_TABLE} WHERE node_rowid = ?1"),
        params![rowid],
    )?;
    Ok(())
}

/// KNN-выборка ближайших `k` узлов по косинусной метрике (vec0). Возвращает
/// `(node_rowid, cosine_distance)`; для метрики `cosine` distance = 1 − similarity.
fn knn(conn: &Connection, query_blob: &[u8], k: usize) -> rusqlite::Result<Vec<(i64, f64)>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT node_rowid, distance FROM {VEC0_TABLE} WHERE embedding MATCH ?1 AND k = ?2"
    ))?;
    let rows = stmt.query_map(params![query_blob, k as i64], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Верхнеуровневый вход поиска: vec0-KNN → точный SIMD-rescore шортлиста по
/// сохранённым блобам `graph_nodes`. Возвращает `(id, cosine_similarity)` по
/// убыванию сходства — совместимо с [`super::similarity::top_k`]. Пустой
/// результат означает «индекс неприменим» (нет таблицы / не совпала размерность).
pub fn knn_rescore(
    conn: &Connection,
    query: &[f32],
    limit: usize,
    project_filter: Option<&str>,
) -> rusqlite::Result<Vec<(i64, f32)>> {
    if query.len() != VEC0_DIM || limit == 0 || !table_present(conn) {
        return Ok(Vec::new());
    }
    // Over-fetch: vec0 не знает про `deleted_at`/`project_id`, отсеиваем после.
    let over = limit.saturating_mul(4).max(limit);
    let candidates = knn(conn, &serialize(query), over)?;
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let mut hits: Vec<(i64, f32)> = Vec::with_capacity(candidates.len());
    for (id, _distance) in candidates {
        let blob: Option<Vec<u8>> = conn
            .query_row(
                "SELECT embedding FROM graph_nodes \
                 WHERE id = ?1 AND embedding IS NOT NULL \
                   AND (deleted_at IS NULL OR deleted_at = '') \
                   AND (?2 IS NULL OR project_id = ?2)",
                params![id, project_filter],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(blob) = blob
            && let Some(vec) = deserialize(&blob)
            && vec.len() == query.len()
        {
            hits.push((id, cosine(query, &vec)));
        }
    }
    hits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    hits.truncate(limit);
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::migrate;

    /// 384-мерный «спайк»: единица в позиции `idx`, остальное — нули. Косинус
    /// двух спайков равен 1 при совпадении индекса и 0 иначе.
    fn spike(idx: usize) -> Vec<f32> {
        let mut v = vec![0.0f32; VEC0_DIM];
        v[idx] = 1.0;
        v
    }

    /// Вставляет узел графа с эмбеддингом и синхронизирует vec0-индекс.
    fn insert_node(conn: &Connection, node_id: &str, vec: &[f32]) -> i64 {
        let blob = serialize(vec);
        conn.execute(
            "INSERT INTO graph_nodes (node_id, label, node_type, embedding, created_at, updated_at) \
             VALUES (?1, ?1, 'function', ?2, '2026-01-01', '2026-01-01')",
            params![node_id, blob],
        )
        .expect("insert node");
        let id = conn.last_insert_rowid();
        sync_node(conn, id, &blob).expect("sync vec0");
        id
    }

    #[test]
    fn init_creates_vec0_table_and_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        assert!(table_present(&conn), "vec0 table created by migrate");
        assert!(ensure_schema(&conn).unwrap(), "re-init stays active");
    }

    #[test]
    fn knn_rescore_ranks_by_exact_cosine_and_honours_limit() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let target = insert_node(&conn, "n-hit", &spike(0));
        insert_node(&conn, "n-orthogonal", &spike(1));
        insert_node(&conn, "n-other", &spike(2));

        let hits = knn_rescore(&conn, &spike(0), 2, None).unwrap();
        assert_eq!(hits.len(), 2, "limit truncates");
        assert_eq!(hits[0].0, target, "exact hit ranks first");
        assert!((hits[0].1 - 1.0).abs() < 1e-5, "cosine of identical spike ~ 1");
        assert!(hits[0].1 >= hits[1].1, "descending by similarity");
    }

    #[test]
    fn knn_rescore_rejects_dimension_mismatch() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        insert_node(&conn, "n1", &spike(0));
        let wrong = vec![1.0f32; VEC0_DIM + 1];
        assert!(knn_rescore(&conn, &wrong, 5, None).unwrap().is_empty());
    }

    #[test]
    fn sync_node_ignores_non_vec0_dimensions() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let blob = serialize(&[1.0f32, 0.0, 0.0]);
        // Не должно быть ошибки: размерность не VEC0_DIM → no-op.
        sync_node(&conn, 999, &blob).unwrap();
        let count: i64 = conn
            .query_row(&format!("SELECT count(*) FROM {VEC0_TABLE}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
