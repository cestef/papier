use serde::{Deserialize, Serialize};

use super::Execute;
use crate::{EditorMode, EditorState};

/// Command to append a single character to the search buffer and trigger a search.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AppendCharToSearch(pub char);

impl Execute for AppendCharToSearch {
    /// Executes the command, appending the specified character to the search buffer
    /// and triggering a search based on the updated buffer.
    fn execute(&mut self, state: &mut EditorState) {
        state.search.push_char(self.0);
        state.search.trigger_search(&state.lines);
        if let Some(index) = state.search.find_first() {
            state.cursor = *index;
        }
    }
}

/// Command to remove the last character from the search buffer and trigger a search.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RemoveCharFromSearch;

impl Execute for RemoveCharFromSearch {
    /// Executes the command, removing the last character from the search buffer
    /// and triggering a search based on the updated buffer.
    fn execute(&mut self, state: &mut EditorState) {
        state.search.remove_char();
        state.search.trigger_search(&state.lines);
    }
}

/// Command to find the first match of the search pattern behind the last cursor position.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TriggerSearch;

impl Execute for TriggerSearch {
    /// Executes the command, finding the first match of the search pattern behind
    /// the last cursor position and setting the cursor to the found match.
    /// Switches to normal mode.
    fn execute(&mut self, state: &mut EditorState) {
        state.mode = EditorMode::Normal;
        if let Some(index) = state.search.find_first() {
            state.cursor = *index;
        }
    }
}

/// Command to find the next search match and update the cursor position.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FindNext;

impl Execute for FindNext {
    /// Executes the command, finding the next search match and updating the cursor position.
    /// Switches to normal mode.
    fn execute(&mut self, state: &mut EditorState) {
        state.mode = EditorMode::Normal;
        if let Some(index) = state.search.find_next() {
            state.cursor = *index;
        }
    }
}

/// Command to find the previous search match and update the cursor position.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FindPrevious;

impl Execute for FindPrevious {
    /// Executes the command, finding the previous search match and updating the cursor position.
    /// Switches to normal mode.
    fn execute(&mut self, state: &mut EditorState) {
        state.mode = EditorMode::Normal;
        if let Some(index) = state.search.find_previous() {
            state.cursor = *index;
        }
    }
}

/// Command to clear to start of the search and switch into search mode.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StartSearch;

impl Execute for StartSearch {
    /// Executes the command, starting the search state and switching to search mode.
    fn execute(&mut self, state: &mut EditorState) {
        state.mode = EditorMode::Search;
        state.search.start(state.cursor);
    }
}
/// Command to clear the search state and switch to normal mode.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StopSearch;

impl Execute for StopSearch {
    /// Executes the command, clearing the search state and switching to normal mode.
    fn execute(&mut self, state: &mut EditorState) {
        state.mode = EditorMode::Normal;
        state.search.clear();
        state.cursor = state.search.start_cursor;
    }
}
