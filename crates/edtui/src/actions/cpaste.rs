use serde::{Deserialize, Serialize};

use super::{delete::delete_selection, Execute};
use crate::{
    clipboard::ClipboardTrait,
    helper::{append_str, clamp_column, insert_str},
    EditorMode, EditorState,
};
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Paste;

impl Execute for Paste {
    fn execute(&mut self, state: &mut EditorState) {
        state.capture();
        clamp_column(state);
        if let Some(selection) = state.selection.take() {
            delete_selection(state, &selection);
            insert_str(&mut state.lines, &mut state.cursor, &state.clip.get_text());
        } else {
            append_str(&mut state.lines, &mut state.cursor, &state.clip.get_text());
        }
        state.mode = EditorMode::Normal;
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CopySelection;

impl Execute for CopySelection {
    fn execute(&mut self, state: &mut EditorState) {
        if let Some(s) = &state.selection {
            state.clip.set_text(s.extract(&state.lines).into());
            state.mode = EditorMode::Normal;
            state.selection = None;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{actions::Undo, clipboard::InternalClipboard, state::selection::Selection, Index2, Lines};
    fn test_state() -> EditorState {
        let mut state = EditorState::new(Lines::from("Hello World!\n\n123."), "txt");
        state.set_clipboard(InternalClipboard::default());
        state
    }

    #[test]
    fn test_copy_paste() {
        let mut state = test_state();
        let selection = Selection::new(Index2::new(0, 0), Index2::new(0, 2));
        state.selection = Some(selection);

        CopySelection.execute(&mut state);
        Paste.execute(&mut state);

        assert_eq!(state.cursor, Index2::new(0, 3));
        assert_eq!(state.lines, Lines::from("HHelello World!\n\n123."));
    }

    #[test]
    fn test_paste_over_selection() {
        let mut state = test_state();
        state.selection = Some(Selection::new(Index2::new(0, 6), Index2::new(0, 10)));
        state.clip.set_text(String::from("Earth"));
        state.mode = EditorMode::Visual;

        Paste.execute(&mut state);

        assert_eq!(state.lines, Lines::from("Hello Earth!\n\n123."));
        assert_eq!(state.cursor, Index2::new(0, 10));
        assert_eq!(state.mode, EditorMode::Normal);

        Undo.execute(&mut state);

        assert_eq!(state.lines, Lines::from("Hello World!\n\n123."));
        assert_eq!(state.mode, EditorMode::Normal);
    }
}
