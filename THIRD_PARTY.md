# THIRD_PARTY.md — Third-party code and dependencies

This file tracks vendored third-party code shipped inside the OmnesAgent
repository, per `PLAN_FILE_EDITOR_ZED.md` §11.13.G4.5. Regular crates.io
dependencies are covered by their own license metadata (`cargo license`);
only *vendored / copied* code is listed here.

---

## floem-editor-core (vendored)

- **Location:** `backend/crates/omnesagent-editor/vendor/editor-core/`
- **Upstream:** <https://github.com/lapce/floem>, subcrate `editor-core`
  (published to crates.io as `floem-editor-core`).
- **Upstream revision:** `1351ffb162faeb8be983f1301d718a0d5fb59c27` (`master`, 2026-09).
- **License:** MIT — Copyright 2023 Floem (see `LICENSE` in the vendor directory).
- **Local changes:** packaging only — standalone `Cargo.toml` (workspace
  inheritance removed, empty `[workspace]` table), unused `ui-events` dependency
  dropped. No source changes. Full details: `VENDOR_NOTES.md` next to the code.

## lapce-xi-rope (crates.io dependency)

- **Used by:** `backend/crates/omnesagent-editor`
- **Version:** `0.4.0`
- **License:** Apache-2.0
- **Role:** rope text storage and `RopeDelta` edit model (the base layer of the
  file editor, `PLAN_FILE_EDITOR_ZED.md` §11.3).

## Zed (reference only)

Parts of the editor design follow the behavior of
[zed-industries/zed](https://github.com/zed-industries/zed) at revision
`bda9c0bd43a8d235d82adb01ea5bc875b861ecfc` (GPL-3.0-or-later). Per the owner's
decision recorded in `PLAN_FILE_EDITOR_ZED.md` §2/§11.12, Zed is used as a
behavioral reference; no GPL source is vendored. The Apache-2.0 `sum_tree` crate
may be vendored later (`omnesagent-editor/vendor/sum_tree`) if the Zed-style
display-map model is adopted (phase F1+); it will be registered here.
