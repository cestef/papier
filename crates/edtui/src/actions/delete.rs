use jagged::index::RowIndex;
use serde::{Deserialize, Serialize};
use synoptic::Highlighter;

use super::Execute;
use crate::{
    clipboard::ClipboardTrait, helper::clamp_column, state::selection::Selection, EditorMode, EditorState, Index2,
    Lines,
};

/// Deletes a character at the current cursor position. Does not
/// move the cursor position unless it is at the end of the line
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RemoveChar(pub usize);

impl Execute for RemoveChar {
    fn execute(&mut self, state: &mut EditorState) {
        clamp_column(state);
        state.capture();
        for _ in 0..self.0 {
            let lines = &mut state.lines;
            let index = &mut state.cursor;
            if lines.len_col(index.row) == Some(0) {
                return;
            }
            let _ = lines.remove(*index);
            if let Some(len) = lines.len_col(index.row) {
                index.col = index.col.min(len.saturating_sub(1));
            }

            let y = index.row;
            state.highlighter.edit(y, &lines.iter_row().map(|e| e.iter().collect()).collect::<Vec<String>>()[y]);
        }
    }
}

/// Deletes a character to the left of the current cursor. Deletes
/// the line break if the the cursor is in column zero.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeleteChar(pub usize);

impl Execute for DeleteChar {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        for _ in 0..self.0 {
            delete_char(&mut state.lines, &mut state.cursor, &mut state.highlighter);
            let y = state.cursor.row;
            state.highlighter.edit(y, &state.lines.iter_row().map(|e| e.iter().collect()).collect::<Vec<String>>()[y]);
        }
    }
}

fn delete_char(lines: &mut Lines, index: &mut Index2, highlighter: &mut Highlighter) {
    fn move_left(lines: &Lines, index: &mut Index2) {
        if index.col > 0 {
            index.col -= 1;
        } else if index.row > 0 {
            index.row -= 1;
            if let Some(len) = lines.len_col(index.row) {
                index.col = len;
            }
        }
    }
    // Do nothing if the cursor is at the beginning of the file
    if index.col == 0 && index.row == 0 {
        return;
    }

    // If the cursor is at the beginning of the line, merge the current line with the previous one
    if index.col == 0 {
        let mut rest = lines.split_off(*index);
        move_left(lines, index);
        lines.merge(&mut rest);
        highlighter.edit(index.row, &lines.iter_row().map(|e| e.iter().collect()).collect::<Vec<String>>()[index.row]);
        highlighter.remove_line(index.row + 1);
    } else {
        // Otherwise, just remove the character to the left
        move_left(lines, index);
        let _ = lines.remove(*index);
        highlighter.edit(index.row, &lines.iter_row().map(|e| e.iter().collect()).collect::<Vec<String>>()[index.row]);
    }
}

/// Deletes the current line.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeleteLine(pub usize);

impl Execute for DeleteLine {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        for _ in 0..self.0 {
            if state.cursor.row >= state.lines.len() {
                break;
            }
            state.lines.remove(RowIndex::new(state.cursor.row));
            state.cursor.col = 0;
            state.cursor.row = state.cursor.row.min(state.lines.len().saturating_sub(1));

            state.highlighter.remove_line(state.cursor.row);
        }
    }
}

/// Deletes the current selection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeleteSelection;

impl Execute for DeleteSelection {
    // TODO: Implement a better way to delete a selection,
    // possibly using a drain iterator.
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        if let Some(selection) = state.selection.take() {
            state.clip.set_text(selection.extract(&state.lines).into());
            delete_selection(state, &selection);
        }
        state.selection = None;
        state.mode = EditorMode::Normal;
    }
}

pub(crate) fn delete_selection(state: &mut EditorState, selection: &Selection) {
    state.cursor = selection.end();

    if state.lines.len_col(state.cursor.row).unwrap_or_default() > 0 {
        state.cursor.col += 1;
    }
    while state.cursor != selection.start() {
        delete_char(&mut state.lines, &mut state.cursor, &mut state.highlighter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_state() -> EditorState {
        EditorState::new(Lines::from("Hello World!\n\n123."), "txt")
    }

    #[test]
    fn test_remove() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 4);
        RemoveChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 4));
        assert_eq!(state.lines, Lines::from("Hell World!\n\n123."));

        state.cursor = Index2::new(0, 10);
        RemoveChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 9));
        assert_eq!(state.lines, Lines::from("Hell World\n\n123."));
    }

    #[test]
    fn test_delete_char() {
        let mut state = test_state();

        state.cursor = Index2::new(0, 5);
        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 4));
        assert_eq!(state.lines, Lines::from("Hell World!\n\n123."));

        state.cursor = Index2::new(0, 11);
        DeleteChar(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 10));
        assert_eq!(state.lines, Lines::from("Hell World\n\n123."));
    }

    #[test]
    fn test_delete_line() {
        let mut state = test_state();
        state.cursor = Index2::new(2, 3);

        DeleteLine(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(1, 0));
        assert_eq!(state.lines, Lines::from("Hello World!\n"));

        DeleteLine(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from("Hello World!"));

        DeleteLine(1).execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 0));
        assert_eq!(state.lines, Lines::from(""));
    }

    #[test]
    fn test_delete_selection() {
        let mut state = test_state();
        let st = Index2::new(0, 1);
        let en = Index2::new(2, 0);
        state.selection = Some(Selection::new(st, en));

        DeleteSelection.execute(&mut state);
        assert_eq!(state.cursor, Index2::new(0, 1));
        assert_eq!(state.lines, Lines::from("H23."));
    }
}
