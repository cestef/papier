use jagged::index::RowIndex;
use synoptic::Highlighter;

use crate::{state::selection::Selection, EditorMode, EditorState, Index2, Lines};

/// Inserts a character into the lines data at the given `index`.
pub fn insert_char(lines: &mut Lines, index: &mut Index2, ch: char, skip_move: bool, highlighter: &mut Highlighter) {
    if lines.is_empty() {
        lines.push(Vec::new());
        highlighter.insert_line(0, &"".to_string());
    }
    if ch == '\n' {
        line_break(lines, index, highlighter);
    } else {
        index.col = index.col.min(max_col(lines, index, EditorMode::Insert));
        lines.insert(*index, ch);
        let y = index.row;
        highlighter.edit(y, &lines.iter_row().map(|e| e.iter().collect()).collect::<Vec<String>>()[y]);
        if !skip_move {
            index.col += 1;
        }
    }
}

/// Inserts a string into the lines data at the given `index`.
pub fn insert_str(lines: &mut Lines, index: &mut Index2, text: &str, highlighter: &mut Highlighter) {
    for (i, ch) in text.chars().enumerate() {
        let is_last = i == text.len().saturating_sub(1);
        insert_char(lines, index, ch, is_last, highlighter);
    }
}

/// Appends a string into the lines data next to a given `index`.
pub fn append_str(lines: &mut Lines, index: &mut Index2, text: &str, highlighter: &mut Highlighter) {
    if !lines.is_empty() && lines.len_col(index.row) > Some(0) {
        index.col += 1;
    }
    for ch in text.chars() {
        insert_char(lines, index, ch, false, highlighter);
    }
    index.col = index.col.saturating_sub(1);
}

/// Inserts a line break at a given index. Forces a splitting of lines if
/// the index is in the middle of a line.
pub(crate) fn line_break(lines: &mut Lines, index: &mut Index2, highlighter: &mut Highlighter) {
    if index.col == 0 {
        lines.insert(RowIndex::new(index.row), vec![]);
        highlighter.insert_line(index.row, &"".to_string());
    } else {
        // Split the line at the cursor position
        let mut rest = lines.split_off(*index);
        // Create a new line with the rest of the characters
        lines.append(&mut rest);
        // Notify highlighter that the current line has been edited and that the next line has been inserted
        let y = index.row;
        let lines = lines.iter_row().map(|e| e.iter().collect()).collect::<Vec<String>>();
        highlighter.insert_line(y + 1, &lines[y + 1]);
        highlighter.edit(y, &lines[y]);
    }
    index.row += 1;
    index.col = 0;
}

/// Returns the maximum permissible column value. In normal or visual
/// mode the limit is len() - 1, in insert mode the limit is len().
pub(crate) fn max_col(lines: &Lines, index: &Index2, mode: EditorMode) -> usize {
    if lines.is_empty() {
        return 0;
    }
    match mode {
        EditorMode::Normal | EditorMode::Visual => lines.len_col(index.row).unwrap_or_default().saturating_sub(1),
        _ => lines.len_col(index.row).unwrap_or_default(),
    }
}

/// Returns the maximum permissible column value. In normal or visual
/// mode the limit is len() - 1, in insert mode the limit is len().
pub(crate) fn max_row(state: &EditorState) -> usize {
    if state.mode == EditorMode::Insert {
        state.lines.len()
    } else {
        state.lines.len().saturating_sub(1)
    }
}

/// Clamps the column of the cursor if the cursor is out of bounds.
/// In normal or visual mode, clamps on col = len() - 1, in insert
/// mode on col = len().
pub(crate) fn clamp_column(state: &mut EditorState) {
    let max_col = max_col(&state.lines, &state.cursor, state.mode);
    state.cursor.col = state.cursor.col.min(max_col);
}

/// Set the selections end positions
pub(crate) fn set_selection(selection: &mut Option<Selection>, end: Index2) {
    if let Some(start) = selection.as_ref().map(|x| x.start) {
        *selection = Some(Selection::new(start, end));
    }
}

/// Skip whitespaces moving to the right. Stop at the end of the line.
pub(crate) fn skip_whitespace(lines: &Lines, index: &mut Index2) {
    if let Some(line) = lines.get(RowIndex::new(index.row)) {
        for (i, &ch) in line.iter().enumerate().skip(index.col) {
            if !ch.is_ascii_whitespace() {
                index.col = i;
                break;
            }
        }
    }
}

/// Skip whitespaces moving to the left. Stop at the start of the line.
pub(crate) fn skip_whitespace_rev(lines: &Lines, index: &mut Index2) {
    if let Some(line) = lines.get(RowIndex::new(index.row)) {
        let skip = line.len().saturating_sub(index.col + 1);
        for &ch in line.iter().rev().skip(skip) {
            if !ch.is_ascii_whitespace() {
                break;
            }
            index.col = index.col.saturating_sub(1);
        }
    }
}

/// Get the number of columns in the current line.
#[must_use]
pub(crate) fn len_col(state: &EditorState) -> usize {
    state.lines.len_col(state.cursor.row).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_lines() -> Lines {
        Lines::from("Hello World!\n\n123.")
    }

    #[test]
    fn test_skip_whitespace() {
        let lines = Lines::from("  World!");
        let mut index = Index2::new(0, 0);

        skip_whitespace(&lines, &mut index);
        assert_eq!(index.col, 2);

        skip_whitespace(&lines, &mut index);
        assert_eq!(index.col, 2);
    }

    #[test]
    fn test_skip_whitespace_rev() {
        let lines = Lines::from("  x World!");
        let mut index = Index2::new(0, 3);

        skip_whitespace_rev(&lines, &mut index);
        assert_eq!(index.col, 2);

        skip_whitespace_rev(&lines, &mut index);
        assert_eq!(index.col, 2);

        index.col = 1;
        skip_whitespace_rev(&lines, &mut index);
        assert_eq!(index.col, 0);
    }

    #[test]
    fn test_insert_str() {
        let mut lines = test_lines();
        let mut index = Index2::new(0, 5);

        insert_str(&mut lines, &mut index, ",\n", &mut Highlighter::new(4));
        assert_eq!(index, Index2::new(1, 0));
        assert_eq!(lines, Lines::from("Hello,\n World!\n\n123."));
    }

    #[test]
    fn test_append_str() {
        let mut lines = test_lines();
        let mut index = Index2::new(0, 5);

        append_str(&mut lines, &mut index, ",\n", &mut Highlighter::new(4));
        assert_eq!(index, Index2::new(1, 0));
        assert_eq!(lines, Lines::from("Hello ,\nWorld!\n\n123."));

        let mut lines = test_lines();
        let mut index = Index2::new(1, 0);
        append_str(&mut lines, &mut index, "abc", &mut Highlighter::new(4));
        assert_eq!(index, Index2::new(1, 2));
        assert_eq!(lines, Lines::from("Hello World!\nabc\n123."));
    }
}
