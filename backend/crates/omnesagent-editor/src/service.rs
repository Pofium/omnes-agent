//! Buffer registry owned by the gateway (`BACKEND_SPEC` §9.1): one buffer per
//! canonical file path, revision-gated edits, atomic save, per-buffer event
//! broadcast for the `/ws/editor` channel, and working-root access control
//! (`BACKEND_SPEC` §9.9 invariant 5).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::SystemTime;

use serde::Serialize;
use tokio::sync::broadcast;

use crate::buffer::{ApplyOutcome, EditOp, EditorBuffer, RowsPage};
use crate::error::EditorError;

    /// Events pushed to every connected client of a buffer. The WS channel maps
    /// these onto `rows_changed` / `save_state` / `remote_ops` frames.
    #[derive(Debug, Clone, Serialize)]
    pub enum BufferEvent {
        Edited {
            rev: u64,
            start_line: u32,
            inval_count: u32,
            new_count: u32,
        },
        Saved {
            rev: u64,
        },
        Reloaded {
            rev: u64,
        },
        ExternalChange {
            rev: u64,
        },
        /// Фолды буфера изменились — клиенты получают `folds_state`.
        FoldsChanged {
            folds: Vec<(u32, u32)>,
        },
        Closed,
    }

/// Drift status reported by `check_drift` (§9.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DriftStatus {
    /// File changed on disk while buffer was clean; auto-reloaded.
    AutoReloaded { new_rev: u64 },
    /// File changed on disk while buffer was dirty; external conflict.
    ExternalConflict,
}

/// Metadata snapshot served by `GET /api/v1/editor/buffers` and used in the
/// `hello` frame.
#[derive(Debug, Clone, Serialize)]
pub struct BufferInfo {
    pub buffer_id: u64,
    pub path: String,
    pub rev: u64,
    pub dirty: bool,
    pub read_only: bool,
    pub language: &'static str,
    pub eol: &'static str,
    pub num_lines: u32,
    pub size_bytes: usize,
}

struct BufferEntry {
    buffer: EditorBuffer,
    events: broadcast::Sender<BufferEvent>,
}

struct Registry {
    by_id: HashMap<u64, BufferEntry>,
    by_path: HashMap<PathBuf, u64>,
}

/// Shared editor state. All methods are synchronous; the gateway calls them
/// from async handlers (locks are held only for in-memory work except the
/// save write, which serializes saves across buffers — acceptable for F0,
/// revisit with per-buffer locks in F3).
pub struct EditorService {
    registry: Mutex<Registry>,
    roots: Mutex<Vec<PathBuf>>,
    next_buffer_id: AtomicU64,
    max_buffer_bytes: u64,
}

impl EditorService {
    pub fn new() -> Self {
        Self::with_limit(crate::MAX_BUFFER_BYTES)
    }

    pub fn with_limit(max_buffer_bytes: u64) -> Self {
        Self {
            registry: Mutex::new(Registry {
                by_id: HashMap::new(),
                by_path: HashMap::new(),
            }),
            roots: Mutex::new(Vec::new()),
            next_buffer_id: AtomicU64::new(1),
            max_buffer_bytes,
        }
    }

    // ── Working roots ────────────────────────────────────────────────────

    /// Register a working root (canonicalized). Idempotent. The client sends
    /// its project root on the first file open; every editor path must fall
    /// inside a registered root.
    pub fn register_root(&self, path: &Path) -> Result<PathBuf, EditorError> {
        if !path.is_dir() {
            return Err(EditorError::NotFound(format!(
                "root is not a directory: {}",
                path.display()
            )));
        }
        let canonical = normalize_path(path)?;
        let mut roots = self.roots.lock().expect("roots mutex poisoned");
        if !roots.contains(&canonical) {
            roots.push(canonical.clone());
        }
        Ok(canonical)
    }

    pub fn roots(&self) -> Vec<String> {
        self.roots
            .lock()
            .expect("roots mutex poisoned")
            .iter()
            .map(|p| p.display().to_string())
            .collect()
    }

    pub fn ensure_in_roots(&self, path: &Path) -> Result<PathBuf, EditorError> {
        let canonical = normalize_path(path)?;
        let roots = self.roots.lock().expect("roots mutex poisoned");
        if roots.iter().any(|r| canonical.starts_with(r)) {
            Ok(canonical)
        } else {
            Err(EditorError::NotInRoot { path: canonical })
        }
    }

    /// Configure display-map settings (tabs, soft-wrap) for an open buffer.
    pub fn set_display_settings(
        &self,
        buffer_id: u64,
        settings: crate::display_map::DisplayMapSettings,
    ) -> Result<(), EditorError> {
        let mut reg = self.registry.lock().expect("registry mutex poisoned");
        let entry = reg
            .by_id
            .get_mut(&buffer_id)
            .ok_or_else(|| EditorError::NotFound(format!("buffer {buffer_id}")))?;
        entry.buffer.settings = settings;
        Ok(())
    }

    /// Validate a possibly not-yet-existing target path for a write: the
    /// deepest existing ancestor must be a directory inside a registered root.
    /// `create_parents` is accepted for API symmetry; validation is the same
    /// because every missing segment is created below the checked ancestor.
    pub fn ensure_in_roots_for_write(
        &self,
        path: &Path,
        create_parents: bool,
    ) -> Result<PathBuf, EditorError> {
        let _ = create_parents;
        let mut existing = path.to_path_buf();
        loop {
            if existing.exists() {
                break;
            }
            if !existing.pop() {
                return Err(EditorError::NotInRoot {
                    path: path.to_path_buf(),
                });
            }
        }
        let canonical = normalize_path(&existing)?;
        if existing != path && !canonical.is_dir() {
            return Err(EditorError::InvalidOp(format!(
                "parent is not a directory: {}",
                canonical.display()
            )));
        }
        let roots = self.roots.lock().expect("roots mutex poisoned");
        if roots.iter().any(|r| canonical.starts_with(r)) {
            Ok(path.to_path_buf())
        } else {
            Err(EditorError::NotInRoot {
                path: path.to_path_buf(),
            })
        }
    }

    // ── Open / list / close ──────────────────────────────────────────────

    /// Open (or return the already open) buffer for a file. One buffer per
    /// canonical path; the same id is returned for repeated opens.
    pub fn open_file(&self, path: &Path) -> Result<(u64, bool), EditorError> {
        // Canonicalize first: a path that cannot be canonicalized does not
        // exist — report NotFound rather than a raw IO error. The roots check
        // then runs on the resolved path (symlink-safe).
        let canonical = normalize_path(path).map_err(|e| match e {
            EditorError::Io(_) => EditorError::NotFound(format!("file not found: {}", path.display())),
            other => other,
        })?;
        let roots = self.roots.lock().expect("roots mutex poisoned");
        if !roots.iter().any(|r| canonical.starts_with(r)) {
            return Err(EditorError::NotInRoot { path: canonical });
        }
        drop(roots);
        let meta = fs::metadata(&canonical)?;
        if !meta.is_file() {
            return Err(EditorError::NotFound(format!(
                "not a file: {}",
                canonical.display()
            )));
        }
        let size = meta.len();
        if size > self.max_buffer_bytes {
            return Err(EditorError::TooLarge {
                size,
                limit: self.max_buffer_bytes,
            });
        }

        let mut registry = self.registry.lock().expect("registry mutex poisoned");
        if let Some(&id) = registry.by_path.get(&canonical) {
            return Ok((id, false));
        }

        let content = fs::read_to_string(&canonical)?;
        let mtime = meta.modified().ok();
        let id = self.next_buffer_id.fetch_add(1, Ordering::SeqCst);
        let mut buffer = EditorBuffer::from_text(id, canonical.clone(), content, false);
        buffer.mtime_at_open = mtime;
        let (events, _) = broadcast::channel(64);
        registry.by_id.insert(
            id,
            BufferEntry {
                buffer,
                events,
            },
        );
        registry.by_path.insert(canonical, id);
        Ok((id, true))
    }

    pub fn info(&self, buffer_id: u64) -> Result<BufferInfo, EditorError> {
        let registry = self.registry.lock().expect("registry mutex poisoned");
        let entry = registry
            .by_id
            .get(&buffer_id)
            .ok_or_else(|| not_found(buffer_id))?;
        Ok(buffer_info(&entry.buffer))
    }

    pub fn find_by_path(&self, path: &Path) -> Result<Option<u64>, EditorError> {
        // A path that no longer exists (or is not canonicalizable) simply has
        // no open buffer.
        let canonical = match normalize_path(path) {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };
        let registry = self.registry.lock().expect("registry mutex poisoned");
        Ok(registry.by_path.get(&canonical).copied())
    }

    pub fn list(&self) -> Vec<BufferInfo> {
        let registry = self.registry.lock().expect("registry mutex poisoned");
        let mut infos: Vec<BufferInfo> = registry
            .by_id
            .values()
            .map(|e| buffer_info(&e.buffer))
            .collect();
        infos.sort_by(|a, b| a.path.cmp(&b.path));
        infos
    }

    /// Close a buffer. Refuses dirty buffers unless `force` (`BACKEND_SPEC`
    /// §9.9: молчаливая потеря правок запрещена).
    pub fn close(&self, buffer_id: u64, force: bool) -> Result<BufferInfo, EditorError> {
        let mut registry = self.registry.lock().expect("registry mutex poisoned");
        let entry = registry
            .by_id
            .get(&buffer_id)
            .ok_or_else(|| not_found(buffer_id))?;
        let info = buffer_info(&entry.buffer);
        if info.dirty && !force {
            return Err(EditorError::DirtyClose {
                path: entry.buffer.path.clone(),
            });
        }
        let entry = registry.by_id.remove(&buffer_id).expect("checked above");
        registry.by_path.remove(&info_path(&entry.buffer));
        let _ = entry.events.send(BufferEvent::Closed);
        Ok(info)
    }

    // ── Editing / rows / save ────────────────────────────────────────────

    pub fn apply_edits(
        &self,
        buffer_id: u64,
        base_rev: u64,
        ops: &[EditOp],
    ) -> Result<ApplyOutcome, EditorError> {
        let mut registry = self.registry.lock().expect("registry mutex poisoned");
        let entry = registry
            .by_id
            .get_mut(&buffer_id)
            .ok_or_else(|| not_found(buffer_id))?;
        let outcome = entry.buffer.apply_ops(base_rev, ops)?;
        let _ = entry.events.send(BufferEvent::Edited {
            rev: outcome.rev,
            start_line: outcome.inval.start_line,
            inval_count: outcome.inval.inval_count,
            new_count: outcome.inval.new_count,
        });
        Ok(outcome)
    }

    pub fn rows(&self, buffer_id: u64, from: u32, count: u32) -> Result<RowsPage, EditorError> {
        let mut registry = self.registry.lock().expect("registry mutex poisoned");
        let entry = registry
            .by_id
            .get_mut(&buffer_id)
            .ok_or_else(|| not_found(buffer_id))?;
        Ok(entry.buffer.rows(from, count))
    }

    /// Save to disk atomically (temp file in the target directory + rename),
    /// then mark pristine. Returns the new mtime.
    pub fn save(&self, buffer_id: u64) -> Result<(BufferInfo, Option<SystemTime>), EditorError> {
        let (text, path) = {
            let registry = self.registry.lock().expect("registry mutex poisoned");
            let entry = registry
                .by_id
                .get(&buffer_id)
                .ok_or_else(|| not_found(buffer_id))?;
            (entry.buffer.full_text(), entry.buffer.path.clone())
        };

        let dir = path
            .parent()
            .ok_or_else(|| EditorError::InvalidOp(format!("no parent dir: {}", path.display())))?;
        let nanos = std::time::SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let tmp = dir.join(format!(
            ".{}.omnes-tmp-{}",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("buffer"),
            nanos
        ));
        fs::write(&tmp, text.as_bytes())?;
        fs::rename(&tmp, &path)?;

        let mtime = fs::metadata(&path).ok().and_then(|m| m.modified().ok());

        let mut registry = self.registry.lock().expect("registry mutex poisoned");
        let entry = registry
            .by_id
            .get_mut(&buffer_id)
            .ok_or_else(|| not_found(buffer_id))?;
        entry.buffer.mark_saved(mtime);
        let info = buffer_info(&entry.buffer);
        let _ = entry.events.send(BufferEvent::Saved { rev: info.rev });
        Ok((info, mtime))
    }

    /// Replace buffer content from outside (e.g. `PUT /api/v1/editor/file`
    /// wrote the file on disk while a buffer was open and clean). The buffer
    /// reloads and becomes pristine.
    pub fn reload_from_disk(&self, buffer_id: u64) -> Result<BufferInfo, EditorError> {
        let mut registry = self.registry.lock().expect("registry mutex poisoned");
        let entry = registry
            .by_id
            .get_mut(&buffer_id)
            .ok_or_else(|| not_found(buffer_id))?;
        let content = fs::read_to_string(&entry.buffer.path)?;
        let mtime = fs::metadata(&entry.buffer.path).ok().and_then(|m| m.modified().ok());
        entry.buffer.reload(content, true);
        if mtime.is_some() {
            entry.buffer.mtime_at_open = mtime;
        }
        let info = buffer_info(&entry.buffer);
        let _ = entry.events.send(BufferEvent::Reloaded { rev: info.rev });
        Ok(info)
    }

    /// Check for external changes on disk for an open buffer.
    /// If file mtime is newer than buffer's mtime_at_open:
    /// - If buffer is clean: auto-reload from disk and broadcast `Reloaded`.
    /// - If buffer is dirty: broadcast `ExternalChange`.
    pub fn check_drift(&self, buffer_id: u64) -> Result<Option<DriftStatus>, EditorError> {
        let (path, mtime_at_open, dirty) = {
            let registry = self.registry.lock().expect("registry mutex poisoned");
            let entry = registry
                .by_id
                .get(&buffer_id)
                .ok_or_else(|| not_found(buffer_id))?;
            (
                entry.buffer.path.clone(),
                entry.buffer.mtime_at_open,
                entry.buffer.is_dirty(),
            )
        };

        let disk_mtime = fs::metadata(&path).ok().and_then(|m| m.modified().ok());
        let has_drift = match (mtime_at_open, disk_mtime) {
            (Some(at_open), Some(on_disk)) => on_disk > at_open,
            _ => false,
        };

        if !has_drift {
            return Ok(None);
        }

        if !dirty {
            let info = self.reload_from_disk(buffer_id)?;
            Ok(Some(DriftStatus::AutoReloaded { new_rev: info.rev }))
        } else {
            let registry = self.registry.lock().expect("registry mutex poisoned");
            if let Some(entry) = registry.by_id.get(&buffer_id) {
                let rev = entry.buffer.rev();
                let _ = entry.events.send(BufferEvent::ExternalChange { rev });
            }
            Ok(Some(DriftStatus::ExternalConflict))
        }
    }

    /// Check drift across all open buffers.
    pub fn check_all_drifts(&self) -> Vec<(u64, DriftStatus)> {
        let buffer_ids: Vec<u64> = {
            let registry = self.registry.lock().expect("registry mutex poisoned");
            registry.by_id.keys().copied().collect()
        };
        let mut results = Vec::new();
        for id in buffer_ids {
            if let Ok(Some(status)) = self.check_drift(id) {
                results.push((id, status));
            }
        }
        results
    }

    /// Read raw (binary) bytes of a file inside the registered roots — the
    /// transport for non-text previews (images) that must not become buffers.
    /// Canonicalize + roots check + size cap; never opens a buffer.
    pub fn read_raw(&self, path: &Path) -> Result<Vec<u8>, EditorError> {
        let canonical = normalize_path(path).map_err(|e| match e {
            EditorError::Io(_) => EditorError::NotFound(format!("file not found: {}", path.display())),
            other => other,
        })?;
        let roots = self.roots.lock().expect("roots mutex poisoned");
        if !roots.iter().any(|r| canonical.starts_with(r)) {
            return Err(EditorError::NotInRoot { path: canonical });
        }
        drop(roots);
        let meta = fs::metadata(&canonical)?;
        if !meta.is_file() {
            return Err(EditorError::NotFound(format!(
                "not a file: {}",
                canonical.display()
            )));
        }
        let size = meta.len();
        if size > crate::MAX_RAW_BYTES {
            return Err(EditorError::TooLarge {
                size,
                limit: crate::MAX_RAW_BYTES,
            });
        }
        Ok(fs::read(&canonical)?)
    }

    /// Применить операцию фолда и разослать новое состояние всем клиентам
    /// буфера (включая инициатора — состояние единственное).
    pub fn set_folds(&self, buffer_id: u64, op: crate::buffer::FoldOp) -> Result<Vec<(u32, u32)>, EditorError> {
        let mut registry = self.registry.lock().expect("registry mutex poisoned");
        let entry = registry
            .by_id
            .get_mut(&buffer_id)
            .ok_or_else(|| not_found(buffer_id))?;
        let folds = entry.buffer.set_folds(op);
        let _ = entry.events.send(BufferEvent::FoldsChanged {
            folds: folds.clone(),
        });
        Ok(folds)
    }

    /// Текущие свёрнутые диапазоны буфера.
    pub fn folds(&self, buffer_id: u64) -> Result<Vec<(u32, u32)>, EditorError> {
        let registry = self.registry.lock().expect("registry mutex poisoned");
        let entry = registry
            .by_id
            .get(&buffer_id)
            .ok_or_else(|| not_found(buffer_id))?;
        Ok(entry.buffer.folds().to_vec())
    }

    /// Кандидаты фолдов по индентации (для «свернуть всё» и UI-подсказок).
    pub fn fold_candidates(&self, buffer_id: u64) -> Result<Vec<(u32, u32)>, EditorError> {
        let registry = self.registry.lock().expect("registry mutex poisoned");
        let entry = registry
            .by_id
            .get(&buffer_id)
            .ok_or_else(|| not_found(buffer_id))?;
        Ok(entry.buffer.fold_candidates())
    }

    /// Subscribe to buffer events (one receiver per WS connection).
    pub fn subscribe(&self, buffer_id: u64) -> Result<broadcast::Receiver<BufferEvent>, EditorError> {
        let registry = self.registry.lock().expect("registry mutex poisoned");
        let entry = registry
            .by_id
            .get(&buffer_id)
            .ok_or_else(|| not_found(buffer_id))?;
        Ok(entry.events.subscribe())
    }
}

impl Default for EditorService {
    fn default() -> Self {
        Self::new()
    }
}

fn not_found(buffer_id: u64) -> EditorError {
    EditorError::NotFound(format!("buffer #{buffer_id}"))
}

fn info_path(buffer: &EditorBuffer) -> PathBuf {
    buffer.path.clone()
}

fn buffer_info(buffer: &EditorBuffer) -> BufferInfo {
    BufferInfo {
        buffer_id: buffer.id,
        path: buffer.path.display().to_string(),
        rev: buffer.rev(),
        dirty: buffer.is_dirty(),
        read_only: buffer.read_only,
        language: buffer.language_id(),
        eol: buffer.eol(),
        num_lines: buffer.num_lines(),
        size_bytes: buffer.size_bytes(),
    }
}

/// Canonicalize without the Windows `\\?\` extended-length prefix so paths
/// compare equal to what clients send.
fn normalize_path(path: &Path) -> Result<PathBuf, EditorError> {
    let canonical = path.canonicalize()?;
    let s = canonical.as_os_str().to_string_lossy();
    let stripped = s.strip_prefix(r"\\?\").map(str::to_string);
    Ok(stripped.map(PathBuf::from).unwrap_or(canonical))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmp_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let p = dir.join(name);
        let mut f = fs::File::create(&p).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        p
    }

    fn service_with_root(dir: &Path) -> EditorService {
        let svc = EditorService::new();
        svc.register_root(dir).unwrap();
        svc
    }

    #[test]
    fn open_is_idempotent_per_path() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = service_with_root(tmp.path());
        let file = tmp_file(tmp.path(), "a.rs", "fn main() {}\n");
        let (id1, created1) = svc.open_file(&file).unwrap();
        let (id2, created2) = svc.open_file(&file).unwrap();
        assert_eq!(id1, id2);
        assert!(created1);
        assert!(!created2);
    }

    #[test]
    fn path_outside_roots_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let svc = EditorService::new();
        svc.register_root(tmp.path()).unwrap();
        let file = tmp_file(other.path(), "secret.txt", "x");
        let err = svc.open_file(&file).unwrap_err();
        assert!(matches!(err, EditorError::NotInRoot { .. }));
    }

    #[test]
    fn edits_update_dirty_flag_and_broadcast_events() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = service_with_root(tmp.path());
        let file = tmp_file(tmp.path(), "b.txt", "hello\n");
        let (id, _) = svc.open_file(&file).unwrap();
        let mut rx = svc.subscribe(id).unwrap();

        let info = svc.info(id).unwrap();
        svc.apply_edits(
            id,
            info.rev,
            &[EditOp {
                start: crate::buffer::EditorPos { line: 0, col: 5 },
                end: crate::buffer::EditorPos { line: 0, col: 5 },
                text: " world".into(),
            }],
        )
        .unwrap();

        let info = svc.info(id).unwrap();
        assert!(info.dirty);
        match rx.try_recv().unwrap() {
            BufferEvent::Edited { rev, new_count, .. } => {
                assert_eq!(rev, info.rev);
                assert_eq!(new_count, 1);
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn save_is_atomic_and_clears_dirty() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = service_with_root(tmp.path());
        let file = tmp_file(tmp.path(), "c.txt", "v1\n");
        let (id, _) = svc.open_file(&file).unwrap();
        let info = svc.info(id).unwrap();
        svc.apply_edits(
            id,
            info.rev,
            &[EditOp {
                start: crate::buffer::EditorPos { line: 0, col: 2 },
                end: crate::buffer::EditorPos { line: 0, col: 2 },
                text: "v2".into(),
            }],
        )
        .unwrap();
        let (info, _) = svc.save(id).unwrap();
        assert!(!info.dirty);
        let on_disk = fs::read_to_string(&file).unwrap();
        assert_eq!(on_disk, "v1v2\n");
        // Temp file renamed away — directory holds only the target.
        let entries: Vec<_> = fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(entries.len(), 1, "temp files must not leak: {entries:?}");
    }

    #[test]
    fn close_refuses_dirty_buffer_unless_forced() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = service_with_root(tmp.path());
        let file = tmp_file(tmp.path(), "d.txt", "x\n");
        let (id, _) = svc.open_file(&file).unwrap();
        let info = svc.info(id).unwrap();
        svc.apply_edits(
            id,
            info.rev,
            &[EditOp {
                start: crate::buffer::EditorPos { line: 0, col: 1 },
                end: crate::buffer::EditorPos { line: 0, col: 1 },
                text: "y".into(),
            }],
        )
        .unwrap();
        assert!(matches!(
            svc.close(id, false),
            Err(EditorError::DirtyClose { .. })
        ));
        let mut rx = svc.subscribe(id).ok(); // closed below
        let info = svc.close(id, true).unwrap();
        assert!(info.dirty);
        assert!(matches!(rx.as_mut().map(|r| r.try_recv()), Some(Ok(BufferEvent::Closed))));
        rx.take();
        assert!(svc.info(id).is_err());
    }

    #[test]
    fn oversized_file_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = EditorService::with_limit(8);
        svc.register_root(tmp.path()).unwrap();
        let file = tmp_file(tmp.path(), "big.txt", "1234567890");
        let err = svc.open_file(&file).unwrap_err();
        assert!(matches!(err, EditorError::TooLarge { .. }));
    }

    #[test]
    fn read_raw_serves_bytes_inside_roots_only() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = service_with_root(tmp.path());
        let file = tmp.path().join("pic.bin");
        std::fs::write(&file, b"\x89PNG\r\n\x1a\n").unwrap();
        let bytes = svc.read_raw(&file).unwrap();
        assert_eq!(bytes, b"\x89PNG\r\n\x1a\n");

        let outside = tempfile::tempdir().unwrap();
        let stranger = tmp_file(outside.path(), "secret.bin", "x");
        assert!(matches!(
            svc.read_raw(&stranger),
            Err(EditorError::NotInRoot { .. })
        ));

        // Oversized raw read is rejected.
        let svc_small = EditorService::new();
        svc_small.register_root(tmp.path()).unwrap();
        let big = tmp.path().join("big.bin");
        std::fs::write(&big, vec![0u8; (crate::MAX_RAW_BYTES + 1) as usize]).unwrap();
        assert!(matches!(
            svc_small.read_raw(&big),
            Err(EditorError::TooLarge { .. })
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn event_receiver_sees_save_event() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = service_with_root(tmp.path());
        let file = tmp_file(tmp.path(), "e.txt", "x\n");
        let (id, _) = svc.open_file(&file).unwrap();
        let mut rx = svc.subscribe(id).unwrap();
        svc.save(id).unwrap();
        let evt = rx.try_recv().unwrap();
        assert!(matches!(evt, BufferEvent::Saved { .. }));
    }
}
