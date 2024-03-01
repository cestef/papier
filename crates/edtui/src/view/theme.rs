use std::collections::HashMap;

use ratatui::{
    style::{Color, Style},
    widgets::Block,
};

use super::{explorer::Explorer, StatusLine};

/// The theme data of the Editor.
pub struct EditorTheme<'a> {
    /// The base text style
    pub base: Style,
    /// The cursor style
    pub cursor_style: Style,
    /// The cursor symbol
    pub cursor_symbol: Option<char>,
    /// The text style in visual mode when a text is selected
    pub selection_style: Style,
    /// The surrounding block
    pub block: Option<Block<'a>>,
    /// An optional [`StatusLine`] displaying the editor mode
    pub status_line: Option<StatusLine>,
    /// The text style for the line numbers
    pub line_numbers_style: Option<Style>,
    /// The file explorer
    pub explorer: Option<Explorer>,
    /// Highlighting token to color mapping
    pub highlighting: Highlighting,
}

pub struct Highlighting(HashMap<String, Color>);
impl Highlighting {
    pub fn get(&self, name: &str) -> Option<Color> {
        self.0.get(name).cloned()
    }
}
impl Default for Highlighting {
    fn default() -> Self {
        Self(HashMap::from([
            ("string".to_string(), Color::Rgb(54, 161, 102)),
            ("comment".to_string(), Color::Rgb(108, 107, 90)),
            ("digit".to_string(), Color::Rgb(157, 108, 124)),
            ("keyword".to_string(), Color::Rgb(91, 157, 72)),
            ("attribute".to_string(), Color::Rgb(95, 145, 130)),
            ("character".to_string(), Color::Rgb(125, 151, 38)),
            ("type".to_string(), Color::Rgb(165, 152, 13)),
            ("function".to_string(), Color::Rgb(174, 115, 19)),
            ("header".to_string(), Color::Rgb(174, 115, 19)),
            ("macro".to_string(), Color::Rgb(157, 108, 124)),
            ("namespace".to_string(), Color::Rgb(125, 151, 38)),
            ("struct".to_string(), Color::Rgb(125, 151, 38)),
            ("operator".to_string(), Color::Rgb(95, 145, 130)),
            ("boolean".to_string(), Color::Rgb(54, 161, 102)),
            ("reference".to_string(), Color::Rgb(91, 157, 72)),
            ("tag".to_string(), Color::Rgb(95, 145, 130)),
            ("heading".to_string(), Color::Rgb(174, 115, 19)),
            ("link".to_string(), Color::Rgb(157, 108, 124)),
            ("key".to_string(), Color::Rgb(157, 108, 124)),
            ("table".to_string(), Color::Rgb(157, 108, 124)),
        ]))
    }
}

impl Default for EditorTheme<'_> {
    /// Creates a new instance of [`EditorTheme`].
    ///
    /// This constructor initializes with default style.
    fn default() -> Self {
        Self {
            base: Style::default().bg(DARK_BLUE),
            block: None,
            cursor_style: Style::default().bg(WHITE).fg(DARK_BLUE),
            cursor_symbol: None,
            selection_style: Style::default().bg(YELLOW).fg(DARK_BLUE),
            status_line: Some(StatusLine::default()),
            line_numbers_style: None,
            explorer: None,
            highlighting: Highlighting::default(),
        }
    }
}

impl<'a> EditorTheme<'a> {
    /// This method allows you to customize the base appearance of the
    /// Editor.
    #[must_use]
    pub fn base(mut self, base: Style) -> Self {
        self.base = base;
        self
    }

    /// Returns the base style.
    #[must_use]
    pub fn base_style(&self) -> Style {
        self.base
    }

    /// This method allows you to customize the block surrrounding
    /// the Editor.
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// This method allows you to customize the style of the cursor of
    /// the Editor.
    #[must_use]
    pub fn cursor_style(mut self, style: Style) -> Self {
        self.cursor_style = style;
        self
    }

    /// This method allows you to customize the symbol of the cursor of
    /// the Editor.
    #[must_use]
    pub fn cursor_symbol(mut self, symbol: Option<char>) -> Self {
        self.cursor_symbol = symbol;
        self
    }

    /// Hides the cursors.
    #[must_use]
    pub fn hide_cursor(mut self) -> Self {
        self.cursor_style = self.base;
        self
    }

    /// This method allows you to customize the style of the selection of
    /// the Editor in visual mode.
    #[must_use]
    pub fn selection_style(mut self, style: Style) -> Self {
        self.selection_style = style;
        self
    }

    /// This method allows you to customize the style of the [`StatusLine`]
    /// of the Editor. See [`StatusLine`] on how to modify its appearance.
    /// Use `hide_status_line` to hide the status line.
    #[must_use]
    pub fn status_line(mut self, status_line: StatusLine) -> Self {
        self.status_line = Some(status_line);
        self
    }

    /// This method allows you to customize the style of the file explorer
    /// of the Editor. See [`Explorer`] on how to modify its appearance.
    /// Use `hide_explorer` to hide the file explorer.
    #[must_use]
    pub fn explorer(mut self, explorer: Explorer) -> Self {
        self.explorer = Some(explorer);
        self
    }

    /// Hides the file explorer.
    #[must_use]
    pub fn hide_explorer(mut self) -> Self {
        self.explorer = None;
        self
    }

    /// Hides the status lilne.
    #[must_use]
    pub fn hide_status_line(mut self) -> Self {
        self.status_line = None;
        self
    }

    /// This method allows you to customize the style of the line numbers
    /// of the Editor.
    #[must_use]
    pub fn line_numbers_style(mut self, style: Style) -> Self {
        self.line_numbers_style = Some(style);
        self
    }
}

// Tailwind slate c100
pub(crate) const LIGHT_GRAY: Color = Color::Rgb(248, 250, 252);

// Tailwind slate c50
pub(crate) const WHITE: Color = Color::Rgb(248, 250, 252);

// Tailwind slate c900
pub(crate) const DARK_BLUE: Color = Color::Rgb(15, 23, 42);

// Tailwind purple c700 & c900
pub(crate) const LIGHT_PURPLE: Color = Color::Rgb(126, 34, 206);
pub(crate) const DARK_PURPLE: Color = Color::Rgb(88, 28, 135);

// Tailwind yellow c400
pub(crate) const YELLOW: Color = Color::Rgb(250, 204, 21);
