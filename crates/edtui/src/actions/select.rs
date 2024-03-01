use serde::{Deserialize, Serialize};

use super::Execute;
use crate::{state::selection::Selection, EditorMode, EditorState, Index2};
/// Selects text between specified delimiter characters.
///
/// It searches for the first occurrence of a delimiter character in the text to
/// define the start of the selection, and the next occurrence of any of the delimiter
/// characters to define the end of the selection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SelectBetween(pub Vec<(char, char)>);

impl Execute for SelectBetween {
    fn execute(&mut self, state: &mut EditorState) {
        let cursor = state.cursor;
        let mut start: Option<Index2> = None;
        let mut end: Option<Index2> = None;
        let mut prev = cursor;
        for (value, index) in state.lines.iter().from(cursor) {
            if let Some(&c) = value {
                if self.0.iter().any(|(start, _)| *start == c) {
                    end = Some(prev);
                    break;
                }
            }
            prev = index;
        }
        prev = cursor;
        for (value, index) in state.lines.iter().from(cursor).rev() {
            if let Some(&c) = value {
                if self.0.iter().any(|(_, end)| *end == c) {
                    start = Some(prev);
                    break;
                }
            }
            prev = index;
        }
        if let (Some(start), Some(end)) = (start, end) {
            state.selection = Some(Selection { start, end });
            state.mode = EditorMode::Visual;
        }
    }
}
