# Vendored: floem-editor-core

- **Upstream:** <https://github.com/lapce/floem>, crate `editor-core` (published as `floem-editor-core` 0.2.0)
- **Upstream revision:** `1351ffb162faeb8be983f1301d718a0d5fb59c27` (2026-09, `master`)
- **License:** MIT — Copyright 2023 Floem (see `LICENSE` in this directory)
- **Total upstream size:** 9102 lines of Rust (`src/**/*.rs`, including `buffer/test.rs`)

## Local adaptations (packaging only, no code changes)

- `Cargo.toml` rewritten to stand alone: workspace inheritance (`version.workspace`,
  `edition.workspace`, `license.workspace`) replaced with concrete values; empty
  `[workspace]` table added so the vendored crate is detached from the OmnesAgent
  cargo workspace (it is consumed as a path dependency of `omnesagent-editor`).
- `ui-events` dependency removed: unused by `editor-core/src/**` (verified by grep
  at the vendored revision).
- `serde` support kept behind the existing optional `serde` feature; default
  features are empty.
- `lapce-xi-rope` pinned to `0.4.0` from crates.io — same version the upstream
  floem workspace declares at the vendored revision.

## Reference

Vendoring decision and obligations: `PLAN_FILE_EDITOR_ZED.md` §11.2/§11.12 (v3:
licensing obligations waived by owner; the notice above is kept for build
reproducibility). Registered in the repo-root `THIRD_PARTY.md`.
