//! Handles key input events
pub mod key;
pub mod register;

use std::fmt::Debug;

use crossterm::event::{KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};

use self::{
    key::Key,
    register::{Register, RegisterKey},
};
use crate::{
    actions::{
        motion::{MoveToFirstLine, MoveToLastLine, MoveWordForwardEnd},
        search::StartSearch,
        Action, Append, AppendCharToSearch, AppendNewline, Composed, CopySelection, Custom, DeleteChar, DeleteLine,
        DeleteSelection, Execute, FindNext, FindPrevious, InsertChar, InsertNewline, LineBreak, MoveBackward, MoveDown,
        MoveForward, MoveToEnd, MoveToFirst, MoveToStart, MoveUp, MoveWordBackward, MoveWordForwardStart, Paste, Redo,
        RemoveChar, RemoveCharFromSearch, SelectBetween, StopSearch, SwitchMode, TriggerSearch, Undo,
    },
    state::command::CommandState,
    EditorMode, EditorState,
};

#[derive(Clone, Debug)]
pub struct Input<I>
where
    I: Clone + Execute + Serialize + for<'de> Deserialize<'de> + Default,
{
    pub register: Register<I>,
    pub command: CommandState<I>,
}

impl<I> Default for Input<I>
where
    I: Clone + Execute + Serialize + for<'de> Deserialize<'de> + Default,
{
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
        let mut r = Register::<I>::new();

        // Go into normal mode
        r.insert(RegisterKey::i(vec![Key::Esc]), Action::SwitchMode(SwitchMode(EditorMode::Normal)));
        r.insert(RegisterKey::v(vec![Key::Esc]), SwitchMode(EditorMode::Normal));

        // Go into insert mode
        r.insert(RegisterKey::n(vec![Key::Char('i')]), SwitchMode(EditorMode::Insert));

        // Go into visual mode
        r.insert(RegisterKey::n(vec![Key::Char('v')]), SwitchMode(EditorMode::Visual));

        // Goes into search mode and starts of a new search.
        r.insert(RegisterKey::n(vec![Key::Char('/')]), StartSearch);
        // Trigger initial search
        r.insert(RegisterKey::s(vec![Key::Enter]), TriggerSearch);
        // Find next
        r.insert(RegisterKey::n(vec![Key::Char('n')]), FindNext);
        // Find previous
        r.insert(RegisterKey::n(vec![Key::Char('N')]), FindPrevious);
        // Clear search
        r.insert(RegisterKey::s(vec![Key::Esc]), StopSearch);
        // Delete last character from search
        r.insert(RegisterKey::s(vec![Key::Backspace]), RemoveCharFromSearch);

        // Go into insert mode and move one char forward
        r.insert(RegisterKey::n(vec![Key::Char('a')]), Append);

        // Move cursor right
        r.insert(RegisterKey::n(vec![Key::Char('l')]), MoveForward(1));
        r.insert(RegisterKey::n(vec![Key::Right]), MoveForward(1));
        r.insert(RegisterKey::v(vec![Key::Char('l')]), MoveForward(1));
        r.insert(RegisterKey::v(vec![Key::Right]), MoveForward(1));
        r.insert(RegisterKey::i(vec![Key::Right]), MoveForward(1));

        // Move cursor left
        r.insert(RegisterKey::n(vec![Key::Char('h')]), MoveBackward(1));
        r.insert(RegisterKey::n(vec![Key::Left]), MoveBackward(1));
        r.insert(RegisterKey::v(vec![Key::Char('h')]), MoveBackward(1));
        r.insert(RegisterKey::v(vec![Key::Left]), MoveBackward(1));
        r.insert(RegisterKey::i(vec![Key::Left]), MoveBackward(1));

        // Move cursor up
        r.insert(RegisterKey::n(vec![Key::Char('k')]), MoveUp(1));
        r.insert(RegisterKey::n(vec![Key::Up]), MoveUp(1));
        r.insert(RegisterKey::v(vec![Key::Char('k')]), MoveUp(1));
        r.insert(RegisterKey::v(vec![Key::Up]), MoveUp(1));
        r.insert(RegisterKey::i(vec![Key::Up]), MoveUp(1));

        // Move cursor down
        r.insert(RegisterKey::n(vec![Key::Char('j')]), MoveDown(1));
        r.insert(RegisterKey::n(vec![Key::Down]), MoveDown(1));
        r.insert(RegisterKey::v(vec![Key::Char('j')]), MoveDown(1));
        r.insert(RegisterKey::v(vec![Key::Down]), MoveDown(1));
        r.insert(RegisterKey::i(vec![Key::Down]), MoveDown(1));

        // Move one word forward/backward
        r.insert(RegisterKey::n(vec![Key::Char('w')]), MoveWordForwardStart(1));
        r.insert(RegisterKey::n(vec![Key::Char('e')]), MoveWordForwardEnd(1));
        r.insert(RegisterKey::n(vec![Key::Char('b')]), MoveWordBackward(1));
        r.insert(RegisterKey::v(vec![Key::Char('w')]), MoveWordForwardStart(1));
        r.insert(RegisterKey::v(vec![Key::Char('e')]), MoveWordForwardEnd(1));
        r.insert(RegisterKey::v(vec![Key::Char('b')]), MoveWordBackward(1));

        // Move cursor to start/first/last position
        r.insert(RegisterKey::n(vec![Key::Char('0')]), MoveToStart());
        r.insert(RegisterKey::n(vec![Key::Char('_')]), MoveToFirst());
        r.insert(RegisterKey::n(vec![Key::Char('$')]), MoveToEnd());
        r.insert(RegisterKey::v(vec![Key::Char('0')]), MoveToStart());
        r.insert(RegisterKey::v(vec![Key::Char('_')]), MoveToFirst());
        r.insert(RegisterKey::v(vec![Key::Char('$')]), MoveToEnd());
        r.insert(RegisterKey::n(vec![Key::Char('g'), Key::Char('g')]), MoveToFirstLine());
        r.insert(RegisterKey::n(vec![Key::Char('G')]), MoveToLastLine());

        // Move cursor to start/first/last position and enter insert mode
        r.insert(
            RegisterKey::n(vec![Key::Char('I')]),
            Composed::new(MoveToFirst()).chain(SwitchMode(EditorMode::Insert)),
        );
        r.insert(RegisterKey::n(vec![Key::Char('A')]), Composed::new(MoveToEnd()).chain(Append));

        // Append/insert new line and switch into insert mode
        r.insert(
            RegisterKey::n(vec![Key::Char('o')]),
            Composed::new(AppendNewline(1)).chain(SwitchMode(EditorMode::Insert)),
        );
        r.insert(
            RegisterKey::n(vec![Key::Char('O')]),
            Composed::new(InsertNewline(1)).chain(SwitchMode(EditorMode::Insert)),
        );

        // Insert a line break
        r.insert(RegisterKey::i(vec![Key::Enter]), LineBreak(1));

        // Remove the current character
        r.insert(RegisterKey::n(vec![Key::Char('x')]), RemoveChar(1));

        // Delete the previous character
        r.insert(RegisterKey::i(vec![Key::Backspace]), DeleteChar(1));

        // Delete the current line
        r.insert(RegisterKey::n(vec![Key::Char('d'), Key::Char('d')]), DeleteLine(1));

        // Delete the current selection
        r.insert(RegisterKey::v(vec![Key::Char('d')]), DeleteSelection);

        // Select inner word between delimiters
        r.insert(
            RegisterKey::n(vec![Key::Char('c'), Key::Char('i'), Key::Char('w')]),
            SelectBetween(vec![('(', ')'), ('[', ']'), ('{', '}'), ('<', '>'), ('"', '"'), ('\'', '\'')]),
        );

        // Undo
        r.insert(RegisterKey::n(vec![Key::Char('u')]), Undo);

        // Redo
        r.insert(RegisterKey::n(vec![Key::Char('r')]), Redo);

        // Copy
        r.insert(RegisterKey::v(vec![Key::Char('y')]), CopySelection);

        // Paste
        r.insert(RegisterKey::n(vec![Key::Char('p')]), Paste);
        r.insert(RegisterKey::v(vec![Key::Char('p')]), Paste);

        Self { register: r, command: CommandState::default() }
    }
}

impl<I> Input<I>
where
    I: Clone + Execute + Serialize + for<'de> Deserialize<'de> + Default + Debug,
{
    pub fn on_key<T>(&mut self, key: T, state: &mut EditorState) -> Option<Custom<I>>
    where
        T: Into<KeyEvent> + Copy,
    {
        let mode = state.mode;
        // r.insert(RegisterKey::n(vec![Key::Char(':')]), StartCommand);
        // r.insert(RegisterKey::c(vec![Key::Esc]), StopCommand);
        // r.insert(RegisterKey::c(vec![Key::Enter]), TriggerCommand);
        // r.insert(RegisterKey::c(vec![Key::Backspace]), RemoveCharFromCommand);
        match key.into().code {
            // Always insert characters in insert mode
            KeyCode::Char(c) if mode == EditorMode::Insert => InsertChar(c).execute(state),
            // Always add characters to search in search mode
            KeyCode::Char(c) if mode == EditorMode::Search => AppendCharToSearch(c).execute(state),

            KeyCode::Char(':') if mode == EditorMode::Normal => {
                self.command.clear();
                state.command.clone_from(&self.command.input);
                state.mode = EditorMode::Command;
            },
            KeyCode::Char(c) if mode == EditorMode::Command => {
                self.command.push_char(c);
                state.command.clone_from(&self.command.input);
            },
            KeyCode::Backspace if mode == EditorMode::Command => {
                self.command.remove_char();
                state.command.clone_from(&self.command.input);
            },
            KeyCode::Esc if mode == EditorMode::Command => {
                self.command.clear();
                state.command.clone_from(&self.command.input);
                state.mode = EditorMode::Normal;
            },
            KeyCode::Enter if mode == EditorMode::Command => {
                let commands = self.command.available_commands.clone();
                let command = self.command.clone();
                let input = command.input.clone();
                let (command, args) = input.split_once(' ').unwrap_or((&input, ""));
                let command = commands.iter().find(|c| c.name == command || c.aliases.contains(&command.to_string()));
                state.mode = EditorMode::Normal;
                self.command.clear();
                if let Some(command) = command {
                    return Some(Custom((command.action)(args.to_string())));
                }
            },

            // Else lookup an action from the register
            _ => {
                if let Some(mut action) = self.register.get(key.into(), mode) {
                    match action {
                        Action::Custom(action) => return Some(action),
                        _ => action.execute(state),
                    }
                }
            },
        }
        None
    }
}
