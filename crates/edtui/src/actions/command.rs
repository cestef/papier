use serde::{Deserialize, Serialize};

use crate::{EditorMode, EditorState};

use super::Execute;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AppendCharToCommand(pub char);

impl Execute for AppendCharToCommand {
    fn execute(&mut self, state: &mut EditorState) {
        state.command.push_char(self.0);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RemoveCharFromCommand;

impl Execute for RemoveCharFromCommand {
    fn execute(&mut self, state: &mut EditorState) {
        state.command.remove_char();
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TriggerCommand;

impl Execute for TriggerCommand {
    fn execute(&mut self, state: &mut EditorState) {
        state.mode = EditorMode::Normal;
        state.command.execute();
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StartCommand;

impl Execute for StartCommand {
    fn execute(&mut self, state: &mut EditorState) {
        state.mode = EditorMode::Command;
        state.command.clear();
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StopCommand;

impl Execute for StopCommand {
    fn execute(&mut self, state: &mut EditorState) {
        state.mode = EditorMode::Normal;
        state.command.clear();
    }
}
