//! DisplayMap layers: tab expansion and soft-wrapping (PLAN_FILE_EDITOR_ZED.md §3.6, §11.9).
//!
//! Transforms buffer rows and syntax runs into virtual display rows:
//! 1. Tab expansion (`TabMap`): converts `\t` into visual spaces based on
//!    `tab_size` (default 4), adjusting syntax `runs` lengths so highlighting
//!    aligns 1:1 with expanded text.
//! 2. Soft-wrap (`WrapMap`): splits lines exceeding `wrap_column` into multiple
//!    display rows on word/boundary breaks, preserving `buffer_row` provenance
//!    and marking continuation rows with `is_wrap_continuation = true`.

use serde::{Deserialize, Serialize};

use crate::buffer::{FoldInfo, RowData};

/// Soft-wrap mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SoftWrap {
    #[default]
    None,
    Bounded,
    EditorWidth,
}

/// Settings governing display-map transformation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayMapSettings {
    pub tab_size: usize,
    pub hard_tabs: bool,
    pub soft_wrap: SoftWrap,
    pub wrap_column: usize,
}

impl Default for DisplayMapSettings {
    fn default() -> Self {
        Self {
            tab_size: 4,
            hard_tabs: false,
            soft_wrap: SoftWrap::None,
            wrap_column: 80,
        }
    }
}

/// Expands tab characters `\t` to spaces up to the next multiple of `tab_size`.
/// Simultaneously adjusts syntax `runs` (segments of `(len_utf16, style_id)`)
/// so that the runs cover the expanded text completely and accurately.
pub fn expand_tabs(
    text: &str,
    runs: &[(u32, u16)],
    tab_size: usize,
) -> (String, Vec<(u32, u16)>) {
    if !text.contains('\t') || tab_size == 0 {
        return (text.to_string(), runs.to_vec());
    }

    let mut expanded = String::with_capacity(text.len() + 16);
    // Maps each source UTF-16 code unit offset to the expanded UTF-16 length it produced.
    let mut utf16_expansion: Vec<u32> = Vec::with_capacity(text.len());

    let mut col = 0usize;
    for ch in text.chars() {
        if ch == '\t' {
            let spaces = tab_size - (col % tab_size);
            for _ in 0..spaces {
                expanded.push(' ');
            }
            col += spaces;
            utf16_expansion.push(spaces as u32);
        } else {
            expanded.push(ch);
            let u16_len = ch.len_utf16();
            col += 1;
            for _ in 0..u16_len {
                utf16_expansion.push(1);
            }
        }
    }

    // Remap runs: each run segment spans a range of source UTF-16 code units.
    // Its new length is the sum of expanded lengths in that range.
    let mut new_runs = Vec::with_capacity(runs.len());
    let mut src_cursor = 0usize;
    for &(len, style_id) in runs {
        let mut expanded_len = 0u32;
        let end = (src_cursor + len as usize).min(utf16_expansion.len());
        for i in src_cursor..end {
            expanded_len += utf16_expansion[i];
        }
        src_cursor = end;
        if expanded_len > 0 {
            new_runs.push((expanded_len, style_id));
        }
    }

    (expanded, new_runs)
}

/// Slices a list of `runs` `[(len_utf16, style_id)]` for a substring that
/// spans UTF-16 code unit range `[slice_start, slice_start + slice_len)`.
pub fn slice_runs(
    runs: &[(u32, u16)],
    slice_start: usize,
    slice_len: usize,
) -> Vec<(u32, u16)> {
    if slice_len == 0 {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut cursor = 0usize;
    let slice_end = slice_start + slice_len;

    for &(len, style_id) in runs {
        let run_start = cursor;
        let run_end = cursor + len as usize;
        cursor = run_end;

        if run_end <= slice_start {
            continue;
        }
        if run_start >= slice_end {
            break;
        }

        let start_in_run = slice_start.saturating_sub(run_start);
        let end_in_run = (slice_end.min(run_end)) - run_start;
        let seg_len = (end_in_run - start_in_run) as u32;
        if seg_len > 0 {
            result.push((seg_len, style_id));
        }
    }
    result
}

/// Finds the best split point (character boundary) in `text` at or before `max_cols`.
/// Prefers splitting on whitespace or punctuation boundary if within a reasonable window.
fn find_wrap_boundary(text: &str, max_cols: usize) -> usize {
    if text.chars().count() <= max_cols {
        return text.len();
    }

    // Convert char index limit to byte offset.
    let mut char_count = 0usize;
    let mut byte_limit = text.len();
    for (b_idx, _) in text.char_indices() {
        if char_count == max_cols {
            byte_limit = b_idx;
            break;
        }
        char_count += 1;
    }

    let search_window = &text[..byte_limit];
    // Look backward for a whitespace break opportunity.
    if let Some(ws_idx) = search_window.rfind(|c: char| c.is_whitespace()) {
        if ws_idx > 0 && ws_idx >= byte_limit.saturating_sub(25) {
            // Break right after the whitespace character.
            let next_char_idx = ws_idx + search_window[ws_idx..].chars().next().map_or(1, |c| c.len_utf8());
            return next_char_idx;
        }
    }

    // Look backward for punctuation break opportunity (e.g. ',', ';', '.', '/', ')').
    if let Some(p_idx) = search_window.rfind(|c: char| matches!(c, ',' | ';' | '.' | '/' | '\\' | ')' | ']' | '}')) {
        if p_idx > 0 && p_idx >= byte_limit.saturating_sub(15) {
            let next_char_idx = p_idx + search_window[p_idx..].chars().next().map_or(1, |c| c.len_utf8());
            return next_char_idx;
        }
    }

    byte_limit
}

/// Applies tab expansion and soft-wrapping to a single buffer row.
/// Emits one or more `RowData` items.
pub fn transform_row(
    buffer_row: u32,
    raw_text: &str,
    raw_runs: &[(u32, u16)],
    fold: Option<FoldInfo>,
    settings: &DisplayMapSettings,
    display_row_start: u32,
) -> Vec<RowData> {
    // 1. Expand tabs.
    let (expanded_text, expanded_runs) = expand_tabs(raw_text, raw_runs, settings.tab_size);

    // 2. Determine if soft-wrap applies.
    let should_wrap = match settings.soft_wrap {
        SoftWrap::None => false,
        SoftWrap::Bounded | SoftWrap::EditorWidth => settings.wrap_column > 0,
    };

    if !should_wrap || expanded_text.chars().count() <= settings.wrap_column {
        return vec![RowData {
            row: display_row_start,
            buffer_row,
            text: expanded_text,
            runs: expanded_runs,
            fold,
            is_wrap_continuation: false,
        }];
    }

    // 3. Perform soft-wrapping into visual lines.
    let mut rows = Vec::new();
    let mut remaining_text = &expanded_text[..];
    let mut utf16_offset = 0usize;
    let mut current_display_row = display_row_start;
    let mut is_first = true;

    while !remaining_text.is_empty() {
        let split_byte = find_wrap_boundary(remaining_text, settings.wrap_column);
        let (head, tail) = remaining_text.split_at(split_byte);
        let head_utf16_len: usize = head.encode_utf16().count();

        let head_runs = slice_runs(&expanded_runs, utf16_offset, head_utf16_len);

        rows.push(RowData {
            row: current_display_row,
            buffer_row,
            text: head.to_string(),
            runs: head_runs,
            fold: if is_first { fold } else { None },
            is_wrap_continuation: !is_first,
        });

        current_display_row += 1;
        is_first = false;
        utf16_offset += head_utf16_len;
        remaining_text = tail;
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tabs_single_tab() {
        let text = "\thello";
        let runs = vec![(6, 1)]; // 1 tab + 5 chars = 6 utf16
        let (expanded, new_runs) = expand_tabs(text, &runs, 4);
        assert_eq!(expanded, "    hello");
        assert_eq!(new_runs, vec![(9, 1)]); // 4 spaces + 5 chars = 9
    }

    #[test]
    fn test_expand_tabs_multiple_tabs_and_styles() {
        let text = "fn\tfoo(\t)";
        let runs = vec![(2, 1), (1, 2), (4, 3), (1, 2), (1, 4)];
        let (expanded, new_runs) = expand_tabs(text, &runs, 4);
        // "fn" (2) -> tab at 2 needs 2 spaces -> "fn  foo(" (8) -> tab at 8 needs 4 spaces -> "fn  foo(    )"
        assert_eq!(expanded, "fn  foo(    )");
        let total_runs_len: u32 = new_runs.iter().map(|(l, _)| *l).sum();
        assert_eq!(total_runs_len as usize, expanded.len());
    }

    #[test]
    fn test_slice_runs() {
        let runs = vec![(4, 1), (6, 2), (5, 3)]; // 0..4 (style 1), 4..10 (style 2), 10..15 (style 3)
        let sliced = slice_runs(&runs, 2, 7); // slice 2..9 -> 2..4 (len 2, style 1), 4..9 (len 5, style 2)
        assert_eq!(sliced, vec![(2, 1), (5, 2)]);
    }

    #[test]
    fn test_transform_row_wrapping() {
        let text = "const longIdentifierNameWithExtraDetails = someFunctionCall(argumentOne, argumentTwo);";
        let runs = vec![(text.len() as u32, 1)];
        let settings = DisplayMapSettings {
            tab_size: 4,
            hard_tabs: false,
            soft_wrap: SoftWrap::Bounded,
            wrap_column: 45,
        };

        let rows = transform_row(0, text, &runs, None, &settings, 0);
        assert!(rows.len() >= 2);
        assert_eq!(rows[0].buffer_row, 0);
        assert_eq!(rows[0].is_wrap_continuation, false);
        assert_eq!(rows[1].buffer_row, 0);
        assert_eq!(rows[1].is_wrap_continuation, true);

        // Check runs coverage on each row.
        for r in &rows {
            let total_runs: u32 = r.runs.iter().map(|(l, _)| *l).sum();
            assert_eq!(total_runs as usize, r.text.encode_utf16().count());
        }
    }
}
