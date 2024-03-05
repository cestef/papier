//! Editor actions such as move, insert, delete
pub mod cpaste;
pub mod delete;
pub mod insert;
pub mod motion;
pub mod search;
pub mod select;

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use self::motion::{MoveToFirstLine, MoveToLastLine};
pub use self::{
    cpaste::{CopySelection, Paste},
    delete::{DeleteChar, DeleteLine, DeleteSelection, RemoveChar},
    insert::{AppendNewline, InsertChar, InsertNewline, LineBreak},
    motion::MoveWordForwardEnd,
    motion::{
        MoveBackward, MoveDown, MoveForward, MoveToEnd, MoveToFirst, MoveToStart, MoveUp, MoveWordBackward,
        MoveWordForwardStart,
    },
    search::{
        AppendCharToSearch, FindNext, FindPrevious, RemoveCharFromSearch, StartSearch, StopSearch, TriggerSearch,
    },
    select::SelectBetween,
};

use crate::{helper::clamp_column, state::selection::Selection, EditorMode, EditorState};

#[enum_dispatch(Execute, Clone, Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum Action<I: Clone + Execute> {
    SwitchMode(SwitchMode),
    Append(Append),
    MoveForward(MoveForward),
    MoveBackward(MoveBackward),
    MoveUp(MoveUp),
    MoveDown(MoveDown),
    MoveWordForwardStart(MoveWordForwardStart),
    MoveWordFowardEnd(MoveWordForwardEnd),
    MoveWordBackward(MoveWordBackward),
    MoveToStart(MoveToStart),
    MoveToFirst(MoveToFirst),
    MoveToEnd(MoveToEnd),
    MoveToFirstLine(MoveToFirstLine),
    MoveToLastLine(MoveToLastLine),
    InsertChar(InsertChar),
    LineBreak(LineBreak),
    AppendNewline(AppendNewline),
    InsertNewline(InsertNewline),
    RemoveChar(RemoveChar),
    DeleteChar(DeleteChar),
    DeleteLine(DeleteLine),
    DeleteSelection(DeleteSelection),
    SelectBetween(SelectBetween),
    Undo(Undo),
    Redo(Redo),
    Paste(Paste),
    CopySelection(CopySelection),
    Composed(Composed<I>),
    StartSearch(StartSearch),
    StopSearch(StopSearch),
    TriggerSearch(TriggerSearch),
    FindNext(FindNext),
    FindPrevious(FindPrevious),
    AppendCharToSearch(AppendCharToSearch),
    RemoveCharFromSearch(RemoveCharFromSearch),
    Custom(Custom<I>),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Custom<I: Clone + Execute>(pub I);

impl<I> Execute for Custom<I>
where
    I: Clone + Execute,
{
    fn execute(&mut self, state: &mut EditorState) {
        self.0.execute(state);
    }
}

#[enum_dispatch]
pub trait Execute {
    fn execute(&mut self, state: &mut EditorState);
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SwitchMode(pub EditorMode);

impl Execute for SwitchMode {
    fn execute(&mut self, state: &mut EditorState) {
        clamp_column(state);
        match self.0 {
            EditorMode::Normal => {
                state.selection = None;
            },
            EditorMode::Visual => {
                state.selection = Some(Selection::new(state.cursor, state.cursor));
            },
            EditorMode::Insert | EditorMode::Search | EditorMode::Command => {},
        }
        state.mode = self.0;
    }
}

/// Switch to insert mode and move one character forward
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Append;

impl Execute for Append {
    fn execute(&mut self, state: &mut EditorState) {
        SwitchMode(EditorMode::Insert).execute(state);
        MoveForward(1).execute(state);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Undo;

impl Execute for Undo {
    fn execute(&mut self, state: &mut EditorState) {
        state.undo();
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Redo;

impl Execute for Redo {
    fn execute(&mut self, state: &mut EditorState) {
        state.redo();
    }
}

/// Executes multiple actions one after the other.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Composed<I: Clone + Execute>(Vec<Action<I>>);

impl<I> Composed<I>
where
    I: Clone + Execute + Serialize,
{
    #[must_use]
    pub fn new<A: Into<Action<I>>>(action: A) -> Self {
        Self(vec![action.into()])
    }

    #[must_use]
    pub fn chain<A: Into<Action<I>>>(mut self, action: A) -> Self {
        self.0.push(action.into());
        self
    }
}

impl<I> Execute for Composed<I>
where
    I: Clone + Execute,
{
    fn execute(&mut self, state: &mut EditorState) {
        for action in &mut self.0 {
            action.execute(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{clipboard::InternalClipboard, Index2, Lines};
    enum TestAction {
        Test,
    }
    fn test_state() -> EditorState {
        let mut state = EditorState::new(Lines::from("Hello World!\n\n123."), "txt");
        state.set_clipboard(InternalClipboard::default());
        state
    }

    #[test]
    fn test_switch_mode() {
        let mut state = test_state();
        assert_eq!(state.mode, EditorMode::Normal);

        SwitchMode(EditorMode::Insert).execute(&mut state);
        assert_eq!(state.mode, EditorMode::Insert);

        SwitchMode(EditorMode::Visual).execute(&mut state);
        assert_eq!(state.mode, EditorMode::Visual);
    }

    #[test]
    fn test_append() {
        let mut state = test_state();

        Append.execute(&mut state);
        assert_eq!(state.mode, EditorMode::Insert);
        assert_eq!(state.cursor, Index2::new(0, 1));

        state.mode = EditorMode::Normal;
        state.cursor = Index2::new(0, 11);
        Append.execute(&mut state);
        assert_eq!(state.mode, EditorMode::Insert);
        assert_eq!(state.cursor, Index2::new(0, 12));
    }
}
