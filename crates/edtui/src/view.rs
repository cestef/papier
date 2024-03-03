//! The editors state
pub mod explorer;
pub mod status_line;
pub mod theme;
use std::time::Duration;

use ratatui::prelude::*;
pub use status_line::StatusLine;
use synoptic::{trim, TokOpt};

use self::theme::EditorTheme;
use crate::{helper::max_col, state::EditorState, EditorMode, Index2};

#[derive(Debug, Clone, Default)]
pub struct EditorMessage {
    message: String,
    duration: Duration,
}

impl EditorMessage {
    /// Creates a new instance of [`EditorMessage`].
    #[must_use]
    pub fn new(message: String, duration: Duration) -> Self {
        Self { message, duration }
    }
}

pub struct EditorView<'a, 'b> {
    pub(crate) state: &'a mut EditorState,
    pub(crate) theme: EditorTheme<'b>,
    pub(crate) message: Option<EditorMessage>,
}

impl<'a, 'b> EditorView<'a, 'b> {
    /// Creates a new instance of [`EditorView`].
    #[must_use]
    pub fn new(state: &'a mut EditorState) -> Self {
        Self { state, theme: EditorTheme::default(), message: None }
    }

    /// Set the theme for the [`EditorView`]
    /// See [`EditorTheme`] for the customizable parameters.
    #[must_use]
    pub fn theme(mut self, theme: EditorTheme<'b>) -> Self {
        self.theme = theme;
        self
    }

    /// Set the message for the [`EditorView`]
    /// The message is displayed in the status line.
    /// If the message is `None`, the message is not displayed.
    /// The message will be displayed for the specified duration.
    #[must_use]
    pub fn message(mut self, message: Option<EditorMessage>) -> Self {
        self.message = message;
        self
    }

    /// Returns a reference to the [`EditorState`].
    #[must_use]
    pub fn get_state(&'a self) -> &'a EditorState {
        self.state
    }

    /// Returns a mutable reference to the [`EditorState`].
    #[must_use]
    pub fn get_state_mut(&'a mut self) -> &'a mut EditorState {
        self.state
    }
    fn highlight_colour(&self, name: &str) -> Color {
        self.theme.highlighting.get(name).unwrap_or(Color::Reset)
    }
}

impl Widget for EditorView<'_, '_> {
    // type State = ViewState;
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Draw the border.
        buf.set_style(area, self.theme.base);
        let area = match &self.theme.block {
            Some(b) => {
                let inner_area = b.inner(area);
                b.clone().render(area, buf);
                inner_area
            },
            None => area,
        };

        // Split into main section and status line
        let [main, status] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(if self.theme.status_line.is_some() { 1 } else { 0 }),
        ])
        .horizontal_margin(1)
        .areas(area);
        let [side, _, main] = Layout::horizontal([
            Constraint::Length(if self.theme.line_numbers_style.is_some() { 3 } else { 0 }),
            Constraint::Length(if self.theme.line_numbers_style.is_some() { 2 } else { 0 }),
            Constraint::Min(0),
        ])
        .areas(main);

        let width = main.width as usize;
        let height = main.height as usize;

        // Retrieve the displayed cursor position. The column of the displayed
        // cursor is clamped to the maximum line length.
        let cursor = displayed_cursor(self.state);

        // Update the view offset. Requires the screen size and the position
        // of the cursor. Updates the view offset only if the cursor is out
        // side of the view port. The state is stored in the `ViewOffset`.
        let size = (width, height);
        let (x_off, y_off) = self.state.view.update_offset(size, cursor);

        // Rendering the text and the selection.
        let lines = &self.state.lines;
        for (i, line) in lines.iter_row().skip(y_off).take(height).enumerate() {
            let y = (main.top() as usize) as u16 + i as u16;
            // Render the line number.
            if let Some(line_numbers_style) = self.theme.line_numbers_style {
                let line_number = (y_off + i + 1).to_string();
                let line_number_x = side.right() - line_number.len() as u16;
                buf.get_mut(line_number_x, y).set_symbol(&line_number).set_style(line_numbers_style);
            }

            let tokens = self.state.highlighter.line(y_off + i, &line.iter().collect());
            let tokens = trim(&tokens, x_off);
            let mut j = 0;
            for token in tokens {
                match token {
                    TokOpt::Some(text, kind) => {
                        let color = self.highlight_colour(&kind);
                        for c in text.chars() {
                            let x = (main.left() as usize) as u16 + j as u16;
                            if let Some(selection) = &self.state.selection {
                                let position = Index2::new(y_off + i, x_off + j);
                                if selection.within(&position) {
                                    buf.get_mut(x, y).set_style(self.theme.selection_style);
                                }
                            }
                            if x < main.right() && y < main.bottom() {
                                buf.get_mut(x, y).set_symbol(&c.to_string()).set_style(Style::default().fg(color));
                                j += 1;
                            } else {
                                break;
                            }
                        }
                    },
                    TokOpt::None(text) => {
                        for c in text.chars() {
                            let x = (main.left() as usize) as u16 + j as u16;
                            if let Some(selection) = &self.state.selection {
                                let position = Index2::new(y_off + i, x_off + j);
                                if selection.within(&position) {
                                    buf.get_mut(x, y).set_style(self.theme.selection_style);
                                }
                            }
                            if x < main.right() && y < main.bottom() {
                                buf.get_mut(x, y).set_symbol(&c.to_string());
                                j += 1;
                            } else {
                                break;
                            }
                        }
                    },
                }
            }
        }

        // Rendering of the cursor. Cursor is not rendered in the loop above,
        // as the cursor may be outside the text in input mode.
        let x_cursor = (main.left() as usize) + width.min(cursor.col.saturating_sub(x_off));
        let y_cursor = (main.top() as usize) + cursor.row.saturating_sub(y_off);
        let cursor_cell = buf.get_mut(x_cursor as u16, y_cursor as u16).set_style(self.theme.cursor_style);
        if let Some(symbol) = self.theme.cursor_symbol {
            cursor_cell.set_symbol(&symbol.to_string());
        }

        // Render the status line.
        if let Some(s) = self.theme.status_line {
            s.mode(self.state.mode.name())
                .search(if self.state.mode == EditorMode::Search {
                    Some(self.state.search.pattern.clone())
                } else {
                    None
                })
                .command(if self.state.mode == EditorMode::Command { Some(self.state.command.clone()) } else { None })
                .render(status, buf);
        }
    }
}

/// Retrieves the displayed cursor position based on the editor state.
///
/// Ensures that the displayed cursor position doesn't exceed the line length.
/// If the internal cursor position exceeds the maximum column, clamp it to
/// the maximum.
fn displayed_cursor(state: &EditorState) -> Index2 {
    let max_col = max_col(&state.lines, &state.cursor, state.mode);
    Index2::new(state.cursor.row, state.cursor.col.min(max_col))
}
