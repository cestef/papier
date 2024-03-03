use ratatui::prelude::*;

use super::theme::WHITE;

/// An optional file explorer for Editor.
#[derive(Debug, Clone)]
pub struct Explorer {
    /// The style for the content of the sidebar
    style: Style,
    // Whether to align content to the left (true) or the right (false)
    align_left: bool,
}

impl Default for Explorer {
    /// Creates a new instance of [`StatusLine`].
    ///
    /// This constructor initializes with default style.
    fn default() -> Self {
        Self { style: Style::default().fg(WHITE).bg(Color::Reset), align_left: true }
    }
}

impl Explorer {
    /// Set the style for the content of the sidebar
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the alignment of the content to the left
    #[must_use]
    pub fn align_left(mut self, align_left: bool) -> Self {
        self.align_left = align_left;
        self
    }
}

#[derive(Debug, Clone)]
struct ExplorerItem {
    name: String,
    is_dir: bool,
    children: Vec<ExplorerItem>,
}

impl ExplorerItem {
    fn new(name: String, is_dir: bool) -> Self {
        Self { name, is_dir, children: Vec::new() }
    }

    fn add_child(&mut self, child: &mut ExplorerItem) -> &mut Self {
        self.children.push(child.clone());
        self
    }
}

impl Widget for Explorer {
    fn render(self, _area: Rect, _buf: &mut Buffer) {}
}
