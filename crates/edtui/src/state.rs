//! The editors state
pub mod command;
pub mod mode;
mod search;
pub mod selection;
mod undo;
mod view;

use synoptic::{from_extension, Highlighter};

use self::search::SearchState;
use self::view::ViewState;
use self::{mode::EditorMode, selection::Selection, undo::Stack};
use crate::clipboard::{Clipboard, ClipboardTrait};
use crate::{Index2, Lines};

/// Represents the state of an editor.
pub struct EditorState {
    /// The text in the editor.
    pub lines: Lines,

    /// The current cursor position in the editor.
    pub cursor: Index2,

    /// The mode of the editor (insert, visual or normal mode).
    pub mode: EditorMode,

    /// Represents the selection in the editor, if any.
    pub selection: Option<Selection>,

    /// Internal view state of the editor.
    pub(crate) view: ViewState,

    /// State holding the search results in search mode.
    pub(crate) search: SearchState,

    /// Stack for undo operations.
    pub(crate) undo: Stack,

    /// Stack for redo operations.
    pub(crate) redo: Stack,

    /// Clipboard for yank and paste operations.
    pub(crate) clip: Clipboard,

    pub highlighter: Highlighter,

    pub command: String,
}

impl Default for EditorState {
    /// Creates a default `EditorState` with no text.
    fn default() -> Self {
        let mut state = EditorState::new(Lines::default(), "");
        state.highlighter = Highlighter::new(4);
        state
    }
}

impl EditorState {
    /// Creates a new editor state.
    ///
    /// # Example
    ///
    /// ```
    /// use edtui_papier::{EditorState, Lines};
    ///
    /// let state = EditorState::new(Lines::from("First line\nSecond Line"), "txt");
    /// ```
    #[must_use]
    pub fn new(lines: Lines, ext: &str) -> EditorState {
        let mut highlighter = from_extension(ext, 4).unwrap_or(Highlighter::new(4));
        highlighter.run(&lines.iter_row().map(|e| e.iter().collect()).collect::<Vec<String>>());
        EditorState {
            lines,
            cursor: Index2::new(0, 0),
            mode: EditorMode::Normal,
            selection: None,
            view: ViewState::default(),
            search: SearchState::default(),
            undo: Stack::new(),
            redo: Stack::new(),
            clip: Clipboard::default(),
            highlighter,
            command: String::new(),
        }
    }

    /// Set a custom clipboard.
    pub fn set_clipboard(&mut self, clipboard: impl ClipboardTrait + 'static) {
        self.clip = Clipboard::new(clipboard);
    }

    pub fn reset_highlighter(&mut self) {
        self.highlighter.run(&self.lines.iter_row().map(|e| e.iter().collect()).collect::<Vec<String>>());
    }
}
