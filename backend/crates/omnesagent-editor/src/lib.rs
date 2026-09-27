//! File editor core for OmnesAgent (`omnesagent-editor`).
//!
//! Phase F0 of `PLAN_FILE_EDITOR_ZED.md`: the gateway is the single owner of
//! project-file buffers; the Flutter client is a virtual viewport that receives
//! row snapshots and sends edits over `/ws/editor/{buffer_id}`.
//!
//! Text storage and the revision/undo model come from the Lapce stack:
//! `lapce-xi-rope` (crates.io) plus the vendored `floem-editor-core`
//! (`vendor/editor-core`, MIT, revision `1351ffb16…`). Everything in this crate
//! is the thin OmnesAgent-specific layer on top: UTF-16 coordinates, the
//! `edit_ops` application contract (base-rev gating), row snapshots for the
//! render plan, atomic save, and the buffer registry with per-buffer event
//! broadcast. See `BACKEND_SPEC.md` §9.2 for the normative model.

pub mod buffer;
pub mod coords;
pub mod error;
pub mod highlight;
pub mod language;
pub mod service;

pub use buffer::{
    ApplyOutcome, EditOp, EditorBuffer, EditorPos, FoldInfo, FoldOp, InvalInfo, RowData, RowsPage,
};
pub use error::EditorError;
pub use language::language_id_from_path;
pub use service::{BufferEvent, BufferInfo, EditorService};

/// Identifier of an open buffer. Assigned by the gateway; stable for the
/// lifetime of the buffer (one buffer per canonical path, `BACKEND_SPEC` §9.1).
pub type BufferId = u64;

/// Upper bound for files opened as editable buffers. Larger files fall back to
/// a plain/read-only flow (`BACKEND_SPEC` §9.6, "большие файлы").
pub const MAX_BUFFER_BYTES: u64 = 10 * 1024 * 1024;

/// Upper bound for raw (non-text) reads such as image previews served by
/// `GET /api/v1/editor/raw`.
pub const MAX_RAW_BYTES: u64 = 64 * 1024 * 1024;
