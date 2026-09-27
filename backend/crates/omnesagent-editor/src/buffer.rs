//! Buffer of a single open file: floem editor model (revisions, undo groups)
//! plus the OmnesAgent `edit_ops` application contract.
//!
//! One `EditorBuffer` per file path lives in the gateway for its whole lifetime
//! (`BACKEND_SPEC` §9.1). Edits arrive as `edit_ops` frames carrying the
//! base revision the client edited against; a frame is applied only when its
//! `base_rev` equals the current revision (Lapce-style rev gating, §11.10 of
//! the plan), otherwise the client gets `edit_ack {applied: false}` and
//! resyncs through an idempotent `rows_snapshot`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

use floem_editor_core::buffer::rope_text::RopeText;
use floem_editor_core::buffer::Buffer as FloemBuffer;
use floem_editor_core::buffer::InvalLines;
use floem_editor_core::cursor::CursorAffinity;
use floem_editor_core::editor::EditType;
use floem_editor_core::line_ending::LineEnding;
use floem_editor_core::selection::{SelRegion, Selection};
use serde::{Deserialize, Serialize};

use crate::coords::offset_of_pos_utf16;
use crate::error::EditorError;

/// A position in the buffer: 0-based line and 0-based column in UTF-16 code
/// units (LSP-compatible; safe for Cyrillic and astral characters).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorPos {
    pub line: u32,
    pub col: u32,
}

/// One edit primitive from an `edit_ops` frame: replace `[start, end)` with
/// `text` (empty `text` = deletion, `start == end` = insertion). Ranges are
/// positions in the pre-edit text; a frame may carry several non-overlapping
/// ops that are applied as a single revision (one undo group).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOp {
    pub start: EditorPos,
    pub end: EditorPos,
    #[serde(default)]
    pub text: String,
}

/// Одна отображаемая строка (render plan, BACKEND_SPEC §9.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowData {
    /// 0-based display row.
    pub row: u32,
    /// 0-based buffer line index in the source file.
    #[serde(default)]
    pub buffer_row: u32,
    pub text: String,
    #[serde(default)]
    pub runs: Vec<(u32, u16)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fold: Option<FoldInfo>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_wrap_continuation: bool,
}

/// Свёрнутый диапазон, начинающийся на этой строке.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct FoldInfo {
    /// Сколько строк буфера скрыто под маркером (не считая строку-маркер).
    pub hidden: u32,
}

/// Операция над фолдами (`folds`-фрейм, §9.9). Диапазоны — buffer-строки,
/// [start, end] включительно.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum FoldOp {
    Fold { start: u32, end: u32 },
    Unfold { start: u32, end: u32 },
    FoldAll,
    UnfoldAll,
}

/// Page of rows for the viewport (`rows_snapshot` frame payload).
#[derive(Debug, Clone, Serialize)]
pub struct RowsPage {
    pub rev: u64,
    pub from: u32,
    pub total_lines: u32,
    pub rows: Vec<RowData>,
}

/// Which lines a revision invalidated (`BACKEND_SPEC` §9.3 "дельта вместо
/// полного кадра"). Mirrors floem's `InvalLines`.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct InvalInfo {
    pub start_line: u32,
    pub inval_count: u32,
    pub new_count: u32,
}

impl From<InvalLines> for InvalInfo {
    fn from(v: InvalLines) -> Self {
        Self {
            start_line: v.start_line as u32,
            inval_count: v.inval_count as u32,
            new_count: v.new_count as u32,
        }
    }
}

/// Result of applying one `edit_ops` frame.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ApplyOutcome {
    pub rev: u64,
    pub inval: InvalInfo,
}

/// Gateway-side buffer: path + floem model + open metadata.
pub struct EditorBuffer {
    pub id: crate::BufferId,
    pub path: PathBuf,
    buffer: FloemBuffer,
    pub mtime_at_open: Option<SystemTime>,
    pub read_only: bool,
    /// Chunked syntax-runs cache (Ф1): чанк = 200 строк + 12 строк контекста
    /// сверху; пересчитывается только при изменении его текста.
    runs_chunks: HashMap<usize, RunsChunk>,
    /// Счётчик пересчётов чанков — для тестов бюджета (§7).
    #[cfg(test)]
    pub(crate) runs_recompute_count: u64,
    #[cfg(not(test))]
    runs_recompute_count: u64,
    /// Свёрнутые диапазоны (buffer-строки, [start, end] включительно),
    /// отсортированы и не пересекаются.
    folds: Vec<(u32, u32)>,
    /// Display-map settings (tabs, wrap).
    pub settings: crate::display_map::DisplayMapSettings,
}

/// Строк в одном чанке подсветки.
const HIGHLIGHT_CHUNK_ROWS: u32 = 200;
/// Строк контекста перед чанком (открытые блочные комментарии/строки).
const HIGHLIGHT_CONTEXT_ROWS: u32 = 12;

struct RunsChunk {
    /// FNV-1a хэш текста чанка вместе с контекстом — ключ инвалидации.
    text_hash: u64,
    runs: Vec<Vec<(u32, u16)>>,
}

fn fxhash_text(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

impl EditorBuffer {
    /// Build from file content. floem `Buffer::new` detects the line ending
    /// and normalizes lone CRs; the detected EOL is reported in metadata and
    /// re-applied on save so the file round-trips.
    pub fn from_text(
        id: crate::BufferId,
        path: PathBuf,
        content: String,
        read_only: bool,
    ) -> Self {
        Self {
            id,
            path,
            buffer: FloemBuffer::new(content),
            mtime_at_open: None,
            read_only,
            runs_chunks: HashMap::new(),
            runs_recompute_count: 0,
            folds: Vec::new(),
            settings: crate::display_map::DisplayMapSettings::default(),
        }
    }

    pub fn rev(&self) -> u64 {
        self.buffer.rev()
    }

    pub fn is_dirty(&self) -> bool {
        !self.buffer.is_pristine()
    }

    pub fn mark_saved(&mut self, mtime: Option<SystemTime>) {
        self.buffer.set_pristine();
        if mtime.is_some() {
            self.mtime_at_open = mtime;
        }
    }

    pub fn eol(&self) -> &'static str {
        match self.buffer.line_ending() {
            LineEnding::CrLf => "crlf",
            _ => "lf",
        }
    }

    pub fn num_lines(&self) -> u32 {
        self.buffer.num_lines() as u32
    }

    pub fn size_bytes(&self) -> usize {
        self.buffer.len()
    }

    pub fn language_id(&self) -> &'static str {
        crate::language_id_from_path(&self.path)
    }

    /// Full text with the buffer's normalized line endings — what gets written
    /// to disk on save.
    pub fn full_text(&self) -> String {
        self.buffer.text().to_string()
    }

    pub fn rows(&mut self, from: u32, count: u32) -> RowsPage {
        let total = self.num_lines();
        let from = from.min(total);
        let remaining = total - from;
        let count = count.min(remaining);
        let runs_all = self.runs_for_range(from, count);
        let mut rows = Vec::with_capacity(count as usize);
        let mut display_row = from;
        for (offset, line) in (from..from + count).enumerate() {
            // Маркер фолда: строка открывает свёрнутый диапазон.
            let fold = self
                .folds
                .iter()
                .find(|&&(s, _)| s == line)
                .map(|&(s, e)| FoldInfo {
                    hidden: e - s,
                });
            let raw_text = line_content(&self.buffer, line as usize);
            let raw_runs = runs_all.get(offset).cloned().unwrap_or_default();
            let transformed = crate::display_map::transform_row(
                line,
                &raw_text,
                &raw_runs,
                fold,
                &self.settings,
                display_row,
            );
            display_row += transformed.len() as u32;
            rows.extend(transformed);
        }
        RowsPage {
            rev: self.rev(),
            from,
            total_lines: total,
            rows,
        }
    }

    /// Apply one `edit_ops` frame. All ops are positions in the pre-edit text
    /// and land in a single revision (single undo group). Overlapping ranges
    /// are rejected without touching the buffer.
    pub fn apply_ops(&mut self, base_rev: u64, ops: &[EditOp]) -> Result<ApplyOutcome, EditorError> {
        if self.read_only {
            return Err(EditorError::ReadOnly {
                path: self.path.clone(),
            });
        }
        if base_rev != self.rev() {
            return Err(EditorError::RevMismatch {
                expected_base: base_rev,
                current: self.rev(),
            });
        }
        if ops.is_empty() {
            return Err(EditorError::InvalidOp("empty ops batch".into()));
        }

        // Convert positions to byte offsets against the pre-edit text.
        let mut ranges: Vec<(usize, usize)> = Vec::with_capacity(ops.len());
        for op in ops {
            let start = offset_of_pos_utf16(&self.buffer, op.start.line as usize, op.start.col as usize);
            let end = offset_of_pos_utf16(&self.buffer, op.end.line as usize, op.end.col as usize);
            ranges.push((start.min(end), start.max(end)));
        }
        ranges.sort_by_key(|&(s, e)| (s, e));
        for pair in ranges.windows(2) {
            // Zero-length inserts at the same point are allowed; overlaps are not.
            if pair[1].0 < pair[0].1 {
                return Err(EditorError::InvalidOp(format!(
                    "op ranges overlap: {:?}",
                    ranges
                )));
            }
        }

        let items: Vec<(Selection, &str)> = ranges
            .iter()
            .zip(ops.iter())
            .map(|(&(start, end), op)| {
                (
                    Selection::sel_region(SelRegion::new(start, end, CursorAffinity::Forward, None)),
                    op.text.as_str(),
                )
            })
            .collect();

        let edit_type = frame_edit_type(ops);
        let (_, _, inval) = self.buffer.edit(items, edit_type);
        let inval: InvalInfo = inval.into();
        self.invalidate_runs(inval.start_line, inval.new_count);
        self.adjust_folds_after_edit(inval.start_line, inval.inval_count, inval.new_count);

        Ok(ApplyOutcome {
            rev: self.rev(),
            inval,
        })
    }

    /// Replace the whole content (external save / agent write landed on disk).
    /// Marks the buffer pristine when `set_pristine` is set.
    pub fn reload(&mut self, content: String, set_pristine: bool) -> InvalInfo {
        let (_, _, inval) = self.buffer.reload(content.into(), set_pristine);
        self.runs_chunks.clear();
        self.folds.clear();
        inval.into()
    }

    // ── Chunked highlight cache (Ф1, §9.4) ───────────────────────────────

    /// Drop cached runs for chunks intersecting `[start_line, start_line +
    /// new_count)` — an edit never recomputes the whole document.
    fn invalidate_runs(&mut self, start_line: u32, new_count: u32) {
        if self.runs_chunks.is_empty() {
            return;
        }
        let first = (start_line / HIGHLIGHT_CHUNK_ROWS) as usize;
        let last = ((start_line + new_count) / HIGHLIGHT_CHUNK_ROWS) as usize;
        self.runs_chunks.retain(|&chunk, _| chunk < first || chunk > last);
    }

    /// Runs for `[from, from+count)`, computed lazily per chunk. A chunk is
    /// re-highlighted only when its (context-included) text changed. Context
    /// rows before the chunk keep open block comments/strings colored; only
    /// the chunk's own rows produce runs.
    fn runs_for_range(&mut self, from: u32, count: u32) -> Vec<Vec<(u32, u16)>> {
        if !crate::highlight::supports_language(self.language_id()) {
            return Vec::new();
        }
        let total = self.num_lines();
        let last_row = from + count;
        let first_chunk = (from / HIGHLIGHT_CHUNK_ROWS) as usize;
        let last_chunk = (((last_row - 1).min(total - 1)) / HIGHLIGHT_CHUNK_ROWS) as usize;

        let mut result: Vec<Vec<(u32, u16)>> = Vec::with_capacity(count as usize);
        let language_id = self.language_id();
        for chunk in first_chunk..=last_chunk {
            let chunk_start = (chunk as u32) * HIGHLIGHT_CHUNK_ROWS;
            let chunk_end = (chunk_start + HIGHLIGHT_CHUNK_ROWS).min(total);
            let ctx_start = chunk_start.saturating_sub(HIGHLIGHT_CONTEXT_ROWS);

            // Текст чанка вместе с контекстом — ключ кэша.
            let mut text = String::new();
            for line in ctx_start..chunk_end {
                text.push_str(&line_content(&self.buffer, line as usize));
                text.push('\n');
            }
            let hash = fxhash_text(&text);
            let entry = self.runs_chunks.entry(chunk).or_insert_with(|| RunsChunk {
                text_hash: 0,
                runs: Vec::new(),
            });
            if entry.text_hash != hash {
                let highlighted = crate::highlight::row_runs(&text, language_id)
                    .unwrap_or_default();
                entry.runs = highlighted
                    .into_iter()
                    .skip((chunk_start - ctx_start) as usize)
                    .take((chunk_end - chunk_start) as usize)
                    .collect();
                entry.text_hash = hash;
                self.runs_recompute_count += 1;
            }
            let skip = from.saturating_sub(chunk_start) as usize;
            let take = (chunk_end - from).min(last_row - from) as usize;
            result.extend(entry.runs.iter().skip(skip).take(take).cloned());
        }
        result
    }

    /// Undo the latest undo group (model check; the user-facing undo lives in
    /// the client's selections collection in F2).
    #[cfg(test)]
    pub(crate) fn undo_once(&mut self) -> Option<InvalInfo> {
        let (_, _, inval, _) = self.buffer.do_undo()?;
        Some(inval.into())
    }

    // ── Фолды (Ф1) ───────────────────────────────────────────────────────

    /// Текущие свёрнутые диапазоны `[start, end]` включительно.
    pub fn folds(&self) -> &[(u32, u32)] {
        &self.folds
    }

    /// Применить операцию фолда; возвращает новое состояние. Диапазоны
    /// клиппируются к документу, результат нормализован (отсортирован,
    /// без пересечений).
    pub fn set_folds(&mut self, op: FoldOp) -> Vec<(u32, u32)> {
        let last = self.num_lines().saturating_sub(1);
        match op {
            FoldOp::Fold { start, end } => {
                let (start, end) = (start.min(end).min(last), start.max(end).min(last));
                if start < end {
                    self.insert_fold(start, end);
                }
            }
            FoldOp::Unfold { start, end } => {
                let (start, end) = (start.min(end), start.max(end));
                self.folds.retain(|&(s, e)| e < start || s > end);
            }
            FoldOp::FoldAll => {
                self.folds.clear();
                for (s, e) in self.fold_candidates() {
                    self.insert_fold(s, e);
                }
            }
            FoldOp::UnfoldAll => self.folds.clear(),
        }
        self.folds.clone()
    }

    fn insert_fold(&mut self, start: u32, end: u32) {
        // Расширить до пересечений и слить: новый диапазон поглощает части
        // существующих, чтобы фолды оставались непересекающимися.
        let mut start = start;
        let mut end = end;
        self.folds.retain(|&(s, e)| {
            if e + 1 < start || s > end + 1 {
                true
            } else {
                start = start.min(s);
                end = end.max(e);
                false
            }
        });
        let pos = self
            .folds
            .partition_point(|&(s, _)| s < start);
        self.folds.insert(pos, (start, end));
    }

    /// Кандидаты фолдов по индентации: строка L сворачивается, если следующая
    /// непустая строка имеет больший отступ; диапазон — до последней строки
    /// блока (пустые строки внутри допускаются).
    pub fn fold_candidates(&self) -> Vec<(u32, u32)> {
        let total = self.num_lines() as usize;
        let indent_of = |line: usize| -> Option<u32> {
            let text = line_content(&self.buffer, line);
            if text.trim().is_empty() {
                return None;
            }
            let mut width = 0u32;
            for ch in text.chars() {
                match ch {
                    ' ' => width += 1,
                    '\t' => width += 4,
                    _ => break,
                }
            }
            Some(width)
        };

        let mut candidates = Vec::new();
        let mut line = 0usize;
        while line < total {
            let Some(indent) = indent_of(line) else {
                line += 1;
                continue;
            };
            // Найти конец блока: последние подряд идущие строки глубже indent.
            let mut block_end = None;
            let mut probe = line + 1;
            while probe < total {
                match indent_of(probe) {
                    Some(w) if w > indent => {
                        block_end = Some(probe);
                        probe += 1;
                    }
                    Some(_) => break,
                    None => probe += 1, // пустые строки внутри блока
                }
            }
            if let Some(end) = block_end {
                candidates.push((line as u32, end as u32));
            }
            line += 1;
        }
        candidates
    }

    /// Подстройка фолдов под правку: диапазон [start, start+inval_count)
    /// заменён на new_count строк — пересекающиеся фолды снимаются,
    /// последующие сдвигаются на дельту.
    fn adjust_folds_after_edit(&mut self, start: u32, inval_count: u32, new_count: u32) {
        let replaced_end = start + inval_count;
        let delta = new_count as i64 - inval_count as i64;
        let mut adjusted = Vec::with_capacity(self.folds.len());
        for &(s, e) in &self.folds {
            if e < start || s >= replaced_end {
                let ns = if s >= replaced_end {
                    (s as i64 + delta).max(0) as u32
                } else {
                    s
                };
                let ne = if e >= replaced_end {
                    (e as i64 + delta).max(ns as i64) as u32
                } else {
                    e
                };
                if ns < ne {
                    adjusted.push((ns, ne));
                }
            }
        }
        adjusted.sort_unstable();
        self.folds = adjusted;
    }
}

/// Line content without the terminator. Out-of-range lines are empty.
pub fn line_content(buf: &impl RopeText, line: usize) -> String {
    if line > buf.last_line() {
        return String::new();
    }
    let content = buf.slice_to_cow(buf.offset_of_line(line)..buf.offset_of_line(line + 1));
    let trimmed = content.trim_end_matches(['\r', '\n']);
    trimmed.to_string()
}

/// Infer the floem `EditType` from the frame shape so undo grouping behaves
/// (typing runs merge, newline starts a new group — `breaks_undo_group`).
fn frame_edit_type(ops: &[EditOp]) -> EditType {
    if ops.len() == 1 {
        match ops[0].text.as_str() {
            "" => EditType::Delete,
            "\n" => EditType::InsertNewline,
            _ => EditType::InsertChars,
        }
    } else {
        EditType::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coords::pos_utf16_of_offset;

    fn buf(text: &str) -> EditorBuffer {
        EditorBuffer::from_text(1, PathBuf::from("test.txt"), text.to_string(), false)
    }

    fn op(sl: u32, sc: u32, el: u32, ec: u32, text: &str) -> EditOp {
        EditOp {
            start: EditorPos { line: sl, col: sc },
            end: EditorPos { line: el, col: ec },
            text: text.to_string(),
        }
    }

    fn rev0(b: &EditorBuffer) -> u64 {
        b.rev()
    }

    #[test]
    fn insert_text_and_revision_bump() {
        let mut b = buf("abc");
        let base = rev0(&b);
        let out = b.apply_ops(base, &[op(0, 1, 0, 1, "X")]).unwrap();
        assert_eq!(b.rows(0, 10).rows[0].text, "aXbc");
        assert_eq!(out.rev, base + 1);
        assert!(b.is_dirty());
        b.mark_saved(None);
        assert!(!b.is_dirty());
    }

    #[test]
    fn newline_splits_and_backspace_merges() {
        let mut b = buf("ab");
        let base = rev0(&b);
        b.apply_ops(base, &[op(0, 2, 0, 2, "\n")]).unwrap();
        let page = b.rows(0, 10);
        assert_eq!(page.rows.len(), 2);
        assert_eq!(page.rows[0].text, "ab");
        assert_eq!(page.rows[1].text, "");
        assert_eq!(page.total_lines, 2);

        // Backspace at (1,0): delete range (0,2)..(1,0).
        let base = b.rev();
        b.apply_ops(base, &[op(0, 2, 1, 0, "")]).unwrap();
        assert_eq!(b.rows(0, 10).rows[0].text, "ab");
    }

    #[test]
    fn cyrillic_edit_roundtrip() {
        let mut b = buf("Привет мир");
        let base = rev0(&b);
        // Insert "ы" after "При" (col 3 in UTF-16 units = byte 6).
        b.apply_ops(base, &[op(0, 3, 0, 3, "ы")]).unwrap();
        assert_eq!(b.rows(0, 10).rows[0].text, "Приывет мир");
        // Delete " мир" via UTF-16 columns (7..11).
        let base = b.rev();
        b.apply_ops(base, &[op(0, 7, 0, 11, "")]).unwrap();
        assert_eq!(b.rows(0, 10).rows[0].text, "Приывет");
    }

    #[test]
    fn emoji_insert_never_splits_characters() {
        let mut b = buf("a\u{1F600}b");
        let base = rev0(&b);
        // Column 3 = after the emoji (2 UTF-16 units at cols 1..3).
        b.apply_ops(base, &[op(0, 3, 0, 3, "!")]).unwrap();
        assert_eq!(b.rows(0, 10).rows[0].text, "a\u{1F600}!b");
    }

    #[test]
    fn stale_base_rev_is_rejected_and_buffer_untouched() {
        let mut b = buf("abc");
        let stale = rev0(&b).wrapping_sub(1);
        let err = b.apply_ops(stale, &[op(0, 0, 0, 0, "x")]).unwrap_err();
        assert!(matches!(err, EditorError::RevMismatch { .. }));
        assert_eq!(b.rows(0, 10).rows[0].text, "abc");
    }

    #[test]
    fn overlapping_ops_rejected() {
        let mut b = buf("abcdef");
        let base = rev0(&b);
        let err = b
            .apply_ops(base, &[op(0, 0, 0, 3, "x"), op(0, 2, 0, 5, "y")])
            .unwrap_err();
        assert!(matches!(err, EditorError::InvalidOp(_)));
        assert_eq!(b.rows(0, 10).rows[0].text, "abcdef");
    }

    #[test]
    fn multi_op_batch_is_single_revision() {
        let mut b = buf("one\ntwo\nthree");
        let base = rev0(&b);
        let out = b
            .apply_ops(
                base,
                &[op(0, 0, 0, 0, "A"), op(1, 3, 1, 3, "B"), op(2, 0, 2, 5, "C")],
            )
            .unwrap();
        assert_eq!(out.rev, base + 1);
        assert_eq!(b.rows(0, 10).rows[0].text, "Aone");
        assert_eq!(b.rows(0, 10).rows[1].text, "twoB");
        assert_eq!(b.rows(0, 10).rows[2].text, "C");
    }

    #[test]
    fn undo_restores_previous_state() {
        let mut b = buf("hello");
        let base = rev0(&b);
        b.apply_ops(base, &[op(0, 5, 0, 5, " world")]).unwrap();
        assert_eq!(b.rows(0, 10).rows[0].text, "hello world");
        b.undo_once();
        assert_eq!(b.rows(0, 10).rows[0].text, "hello");
    }

    #[test]
    fn crlf_is_detected_and_stripped_in_rows() {
        let mut b = buf("a\r\nb\r\n");
        assert_eq!(b.eol(), "crlf");
        let page = b.rows(0, 10);
        assert_eq!(page.total_lines, 3);
        assert_eq!(page.rows[0].text, "a");
        assert_eq!(page.rows[1].text, "b");
        assert_eq!(page.rows[2].text, "");
        // Save round-trip keeps CRLF.
        assert_eq!(b.full_text(), "a\r\nb\r\n");
    }

    #[test]
    fn read_only_buffer_refuses_edits() {
        let mut b = EditorBuffer::from_text(
            2,
            PathBuf::from("ro.txt"),
            "locked".to_string(),
            true,
        );
        let err = b.apply_ops(b.rev(), &[op(0, 0, 0, 0, "x")]).unwrap_err();
        assert!(matches!(err, EditorError::ReadOnly { .. }));
    }

    #[test]
    fn rows_paging_clamps() {
        let mut b = buf("1\n2\n3\n4\n5");
        let page = b.rows(3, 100);
        assert_eq!(page.from, 3);
        assert_eq!(page.rows.len(), 2);
        assert_eq!(page.rows[0].text, "4");
        // Beyond the end: clamped to the document end, empty page.
        let page = b.rows(99, 5);
        assert_eq!(page.from, 5);
        assert!(page.rows.is_empty());
    }

    #[test]
    fn pos_roundtrip_through_offsets() {
        let b = buf("Привет\nмир");
        // Byte offset 6 = after "При" (3 Cyrillic chars, 2 bytes each).
        let (line, col) = pos_utf16_of_offset(&b.buffer, 6);
        assert_eq!((line, col), (0, 3));
        assert_eq!(offset_of_pos_utf16(&b.buffer, line, col), 6);
        // Byte offset 13 = start of "мир" (line 1, col 0).
        assert_eq!(pos_utf16_of_offset(&b.buffer, 13), (1, 0));
    }

    #[test]
    fn fold_set_unfold_and_merge() {
        let mut b = buf("a {\n  b {\n    c\n  }\n  d\n}\ne\n");
        b.set_folds(FoldOp::Fold { start: 0, end: 2 });
        b.set_folds(FoldOp::Fold { start: 1, end: 3 });
        // Пересекающиеся фолды сливаются в один.
        assert_eq!(b.folds(), &[(0, 3)]);
        b.set_folds(FoldOp::Unfold { start: 0, end: 3 });
        assert!(b.folds().is_empty());
        // Клиппинг к документу.
        b.set_folds(FoldOp::Fold { start: 5, end: 99 });
        assert_eq!(b.folds(), &[(5, 7)]);
    }

    #[test]
    fn fold_candidates_by_indent() {
        let b = buf("fn a() {\n  fn b() {\n    x\n  }\n  y\n}\n\nz\n");
        let candidates = b.fold_candidates();
        // Строка 3 ("  }") не глубже строки 1 — блок (1,2), не (1,3).
        assert_eq!(candidates, vec![(0, 4), (1, 2)]);
    }

    #[test]
    fn rows_mark_fold_starts() {
        let mut b = buf("a\nb\nc\nd\n");
        b.set_folds(FoldOp::Fold { start: 1, end: 2 });
        let page = b.rows(0, 10);
        assert!(page.rows[0].fold.is_none());
        assert_eq!(page.rows[1].fold, Some(FoldInfo { hidden: 1 }));
        assert!(page.rows[2].fold.is_none());
    }

    #[test]
    fn folds_shift_after_edits() {
        let mut b = buf("l0\nl1\nl2\nl3\nl4\n");
        b.set_folds(FoldOp::Fold { start: 2, end: 3 });
        // Вставка строки выше фолда сдвигает его вниз.
        let base = b.rev();
        b.apply_ops(
            base,
            &[op(0, 0, 0, 0, "new\n")],
        )
        .unwrap();
        assert_eq!(b.folds(), &[(3, 4)]);
        // Удаление свёрнутого диапазона снимает фолд.
        let base = b.rev();
        b.apply_ops(base, &[op(3, 0, 5, 0, "")]).unwrap();
        assert!(b.folds().is_empty());
    }

    #[test]
    fn tab_expansion_in_rows() {
        let mut b = buf("\tfn main() {\n\t\tprintln!();\n\t}");
        b.settings.tab_size = 4;
        let page = b.rows(0, 10);
        assert_eq!(page.rows[0].text, "    fn main() {");
        assert_eq!(page.rows[1].text, "        println!();");
        assert_eq!(page.rows[2].text, "    }");
    }

    #[test]
    fn soft_wrap_in_rows() {
        let mut b = buf("short line\nthis is a very long line that exceeds the wrap limit and must be wrapped into multiple display rows\nend");
        b.settings.soft_wrap = crate::display_map::SoftWrap::Bounded;
        b.settings.wrap_column = 35;
        let page = b.rows(0, 10);
        assert!(page.rows.len() > 3);
        assert_eq!(page.rows[0].buffer_row, 0);
        assert_eq!(page.rows[0].is_wrap_continuation, false);

        assert_eq!(page.rows[1].buffer_row, 1);
        assert_eq!(page.rows[1].is_wrap_continuation, false);
        assert_eq!(page.rows[2].buffer_row, 1);
        assert_eq!(page.rows[2].is_wrap_continuation, true);
    }
}
