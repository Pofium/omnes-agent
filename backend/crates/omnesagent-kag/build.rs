//! Ф6.3 (ob2h v2.0 / Ф72b): сборка вендоренной амальгамации `sqlite-vec`.
//!
//! Расширение компилируется со `SQLITE_CORE`, то есть линкуется статически с
//! bundled SQLite из `rusqlite` (внешний `.dll` в рантайме не нужен) и
//! регистрируется на соединении через `sqlite3_vec_init`.
//!
//! Подсистемы DISKANN / RESCORE / экспериментальный IVF отключены: их
//! `#include`-файлы (`sqlite-vec-diskann.c`, `sqlite-vec-rescore.c`,
//! `sqlite-vec-ivf*.c`) не входят в вендоренный набор, а точный rescore
//! выполняется SIMD-ядром в Rust. Без этих define-ов амальгамация не собирается.

fn main() {
    // Компилируем C-расширение только когда включена фича `vec0`.
    if std::env::var_os("CARGO_FEATURE_VEC0").is_none() {
        return;
    }

    cc::Build::new()
        .file("vendor/sqlite-vec/sqlite-vec.c")
        .include("vendor/sqlite-vec")
        .define("SQLITE_CORE", None)
        .define("SQLITE_VEC_ENABLE_DISKANN", "0")
        .define("SQLITE_VEC_ENABLE_RESCORE", "0")
        .define("SQLITE_VEC_EXPERIMENTAL_IVF_ENABLE", "0")
        .warnings(false)
        .compile("sqlite_vec0");

    println!("cargo:rerun-if-changed=vendor/sqlite-vec/sqlite-vec.c");
    println!("cargo:rerun-if-changed=vendor/sqlite-vec/sqlite-vec.h");
}
