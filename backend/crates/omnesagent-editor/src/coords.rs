//! Coordinate conversions for editor buffers.
//!
//! The client speaks (line, col-in-UTF-16-units) — the same encoding LSP uses —
//! while xi-rope stores byte offsets. Conversions clamp defensively: they never
//! return an offset inside a UTF-8 character or inside the line terminator, and
//! out-of-range positions collapse to the nearest valid one. This is the
//! "кириллица/эмодзи не рвём" requirement of `PLAN_FILE_EDITOR_ZED.md` §3.2.
//!
//! Line/col are 0-based. The column counts UTF-16 code units, so any astral
//! character (emoji) counts as 2.

use floem_editor_core::buffer::rope_text::RopeText;

/// Byte offset of (line, col_utf16). Clamping rules: the offset always lands
/// before the line terminator, never inside a UTF-8 character — a column that
/// falls inside an astral char clamps to that char's START (insert-before
/// semantics, matching LSP/VSCode behavior on surrogate pairs).
pub fn offset_of_pos_utf16(buf: &impl RopeText, line: usize, col_utf16: usize) -> usize {
    let line = line.min(buf.last_line());
    let mut offset = buf.offset_of_line(line);
    let content = buf.slice_to_cow(offset..buf.offset_of_line(line + 1));
    let mut utf16 = 0usize;
    for ch in content.chars() {
        if ch == '\n' || ch == '\r' || utf16 >= col_utf16 {
            break;
        }
        let unit_len = ch.len_utf16();
        if utf16 + unit_len > col_utf16 {
            // Column points inside this character — clamp before it.
            break;
        }
        utf16 += unit_len;
        offset += ch.len_utf8();
    }
    offset
}

/// (line, col_utf16) of a byte offset. An offset pointing at the line
/// terminator reports the end of the line content; an offset that lands
/// inside a UTF-8 character clamps to that character's start.
pub fn pos_utf16_of_offset(buf: &impl RopeText, offset: usize) -> (usize, usize) {
    let offset = offset.min(buf.len());
    let line = buf.line_of_offset(offset);
    let line_start = buf.offset_of_line(line);
    let content = buf.slice_to_cow(line_start..buf.offset_of_line(line + 1));
    let mut col = 0usize;
    let mut off = line_start;
    for ch in content.chars() {
        if ch == '\n' || ch == '\r' || off >= offset {
            break;
        }
        let len = ch.len_utf8();
        if off + len > offset {
            // Mid-character offset — clamp to the character's start.
            break;
        }
        col += ch.len_utf16();
        off += len;
    }
    (line, col)
}

/// Number of UTF-16 code units in the line content (terminator excluded).
pub fn line_len_utf16(buf: &impl RopeText, line: usize) -> usize {
    pos_utf16_of_offset(buf, buf.line_end_offset(line, true)).1
}

#[cfg(test)]
mod tests {
    use super::*;
    use floem_editor_core::buffer::rope_text::RopeTextVal;
    use floem_editor_core::xi_rope::Rope;

    fn rt(text: &str) -> RopeTextVal {
        RopeTextVal::new(Rope::from(text))
    }

    #[test]
    fn ascii_roundtrip() {
        let buf = rt("hello\nworld");
        assert_eq!(offset_of_pos_utf16(&buf, 0, 3), 3);
        assert_eq!(pos_utf16_of_offset(&buf, 3), (0, 3));
        // End of line content (before the terminator), never inside "\n".
        assert_eq!(offset_of_pos_utf16(&buf, 0, 5), 5);
        assert_eq!(offset_of_pos_utf16(&buf, 0, 100), 5);
        assert_eq!(pos_utf16_of_offset(&buf, 5), (0, 5));
        assert_eq!(pos_utf16_of_offset(&buf, 6), (1, 0));
    }

    #[test]
    fn cyrillic_columns_are_utf16_stable() {
        // 6 Cyrillic chars before the target point; each is 2 bytes, 1 UTF-16 unit.
        let buf = rt("Привет мир");
        assert_eq!(offset_of_pos_utf16(&buf, 0, 6), 12);
        assert_eq!(pos_utf16_of_offset(&buf, 12), (0, 6));
        assert_eq!(line_len_utf16(&buf, 0), 10);
    }

    #[test]
    fn emoji_count_as_two_units_and_never_split() {
        // "😀" is 4 UTF-8 bytes, 2 UTF-16 units: columns a=0, emoji=1..3, b=3.
        let buf = rt("a\u{1F600}b");
        assert_eq!(offset_of_pos_utf16(&buf, 0, 1), 1);
        // Columns inside the surrogate pair clamp to the char START.
        assert_eq!(offset_of_pos_utf16(&buf, 0, 2), 1);
        assert_eq!(offset_of_pos_utf16(&buf, 0, 3), 5);
        assert_eq!(offset_of_pos_utf16(&buf, 0, 4), 6);
        assert_eq!(pos_utf16_of_offset(&buf, 5), (0, 3));
        assert_eq!(pos_utf16_of_offset(&buf, 6), (0, 4));
        // Round-trip holds for every valid column.
        for col in [0usize, 1, 3, 4] {
            let off = offset_of_pos_utf16(&buf, 0, col);
            assert_eq!(pos_utf16_of_offset(&buf, off), (0, col));
        }
    }

    #[test]
    fn zwj_family_emoji_keeps_boundaries() {
        // ZWJ sequence: each code point is a separate char; columns count
        // per-code-point UTF-16 lengths. Offsets always land on char boundaries.
        let buf = rt("👨‍👩‍👧x");
        // Column 8 = start of "x" (2+1+2+1+2 = 8 units before it).
        let off_x = offset_of_pos_utf16(&buf, 0, 8);
        assert_eq!(off_x, buf.len() - 1);
        assert_eq!(&buf.slice_to_cow(off_x..off_x + 1), "x");
        assert_eq!(pos_utf16_of_offset(&buf, off_x), (0, 8));
    }

    #[test]
    fn crlf_offsets_avoid_the_terminator() {
        let buf = rt("hello\r\nworld");
        assert_eq!(offset_of_pos_utf16(&buf, 0, 5), 5);
        assert_eq!(offset_of_pos_utf16(&buf, 0, 100), 5);
        assert_eq!(pos_utf16_of_offset(&buf, 5), (0, 5));
        assert_eq!(pos_utf16_of_offset(&buf, 7), (1, 0));
    }

    #[test]
    fn out_of_range_line_clamps_to_last() {
        let buf = rt("ab\ncd");
        assert_eq!(offset_of_pos_utf16(&buf, 99, 0), 3);
        assert_eq!(pos_utf16_of_offset(&buf, 99), (1, 2));
    }
}
