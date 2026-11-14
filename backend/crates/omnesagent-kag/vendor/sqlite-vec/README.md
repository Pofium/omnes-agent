# Vendored `sqlite-vec` amalgamation

This directory contains the **amalgamation build** of
[`sqlite-vec`](https://github.com/asg017/sqlite-vec) (asg017), vendored so the
`vec0` KNN index (Ф6.3) can be statically linked into `omnesagent-kag` without a
runtime `.dll` and without depending on the (incomplete) `sqlite-vec` crate.

## Files

| File | Origin |
| --- | --- |
| `sqlite-vec.c` | `sqlite-vec` amalgamation, crate `sqlite-vec` `0.1.10-alpha.4` |
| `sqlite-vec.h` | `sqlite-vec` amalgamation header, same version |
| `sqlite3.h` | SQLite public-domain amalgamation header |
| `sqlite3ext.h` | SQLite public-domain extension header |

## License & attribution

- `sqlite-vec` (asg017) is dual-licensed **MIT OR Apache-2.0**.
- `sqlite3.h` / `sqlite3ext.h` are **public domain** (SQLite).

## Build configuration

Compiled by `build.rs` via the `cc` crate with the `vec0` Cargo feature:

- `SQLITE_CORE` — link against the bundled SQLite provided by `rusqlite`
  (no `SQLITE_EXTENSION_INIT` indirection, no external `.dll`).
- `SQLITE_VEC_ENABLE_DISKANN=0`, `SQLITE_VEC_ENABLE_RESCORE=0`,
  `SQLITE_VEC_EXPERIMENTAL_IVF_ENABLE=0` — these subsystems are disabled: their
  translation units are not part of the vendored set. Exact list rescoring is
  performed in Rust (`src/vector/simd.rs`) instead.

## Updating

Fetch the matching amalgamation from the upstream `sqlite-vec` release (or the
published crate tarball), replace the four files above, then re-run
`cargo test -p omnesagent-kag --features vec0`.
