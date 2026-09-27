//! Errors of the editor core. Displayed to clients through the gateway as
//! `error` frames / HTTP status codes; never panics across the boundary.

use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum EditorError {
    /// Requested buffer id (or file path) has no open buffer / does not exist.
    NotFound(String),
    /// Path resolves outside every registered working root (`BACKEND_SPEC` §9.9
    /// invariant 5).
    NotInRoot { path: PathBuf },
    /// Filesystem failure during open/save/scan.
    Io(std::io::Error),
    /// `edit_ops` arrived against a stale revision: the client must resync via
    /// `rows_snapshot` (`BACKEND_SPEC` §9.2).
    RevMismatch { expected_base: u64, current: u64 },
    /// Refusing to close a dirty buffer without `force`.
    DirtyClose { path: PathBuf },
    /// Buffer is read-only (outside workspace / large-file plain mode).
    ReadOnly { path: PathBuf },
    /// File exceeds `MAX_BUFFER_BYTES`.
    TooLarge { size: u64, limit: u64 },
    /// Malformed edit batch (overlapping ranges).
    InvalidOp(String),
}

impl fmt::Display for EditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(p) => write!(f, "not found: {p}"),
            Self::NotInRoot { path } => {
                write!(f, "path outside registered editor roots: {}", path.display())
            }
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::RevMismatch { expected_base, current } => write!(
                f,
                "stale base_rev {expected_base}: current buffer rev is {current}"
            ),
            Self::DirtyClose { path } => {
                write!(f, "buffer has unsaved changes: {}", path.display())
            }
            Self::ReadOnly { path } => write!(f, "buffer is read-only: {}", path.display()),
            Self::TooLarge { size, limit } => {
                write!(f, "file is {size} bytes, editor limit is {limit} bytes")
            }
            Self::InvalidOp(msg) => write!(f, "invalid edit op: {msg}"),
        }
    }
}

impl std::error::Error for EditorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for EditorError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
