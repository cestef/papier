use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    fs,
    io::{self, BufRead, Write},
    path::PathBuf,
    time::Duration,
};

use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
    PapierAction,
};
use color_eyre::eyre::{eyre, Result};
use crossterm::event::{KeyCode, KeyEvent};
use edtui::{
    actions::Execute, state::command::Command, EditorMode, EditorState, EditorTheme, EditorView, Index2, Input, Lines,
    StatusLine,
};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
// use tui_textarea::{CursorMove, Input, Key, Scrolling, TextArea};

pub struct Theme<'a> {
    pub editor: EditorTheme<'a>,
}

#[derive(Default)]
pub struct Editor {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    buffers: HashMap<String, Buffer>,
    current_buffer: Option<String>,
    message: Option<String>,
}

impl Editor {
    pub fn new() -> Self {
        let config = Config::default();
        let buffer = Buffer::new(None, config.keybindings.clone(), None).unwrap();
        let mut buffers = HashMap::new();
        buffers.insert("".to_string(), buffer);
        Self { command_tx: None, config, buffers, current_buffer: Some("".to_string()), message: None }
    }

    pub async fn open(&mut self, path: PathBuf) -> Result<()> {
        let buffer = Buffer::from_path(path.clone(), self.config.keybindings.clone(), self.command_tx.clone())?;
        self.buffers.insert(path.clone().to_string_lossy().to_string(), buffer);
        self.current_buffer = Some(path.to_string_lossy().to_string());
        Ok(())
    }

    pub fn current_buffer(&mut self) -> Option<&mut Buffer> {
        self.current_buffer.as_ref().and_then(|b| self.buffers.get_mut(b))
    }
}

pub struct Buffer {
    path: Option<PathBuf>,
    modified: bool,
    state: EditorState,
    input: Input<PapierAction>,
}

impl Buffer {
    fn from_path(path: PathBuf, keybindings: KeyBindings, tx: Option<UnboundedSender<Action>>) -> io::Result<Self> {
        let file = fs::File::open(&path)?;
        let reader = io::BufReader::new(file);
        let lines: &str = &reader.lines().map_while(Result::ok).collect::<Vec<String>>().join("\n");
        let state = EditorState::new(Lines::from(lines));

        Ok(Self { path: Some(path), modified: false, state, input: keybindings.into() })
    }

    fn new(path: Option<PathBuf>, keybindings: KeyBindings, tx: Option<UnboundedSender<Action>>) -> io::Result<Self> {
        let state = EditorState::new(Lines::from(
            "papier is a light-weight vim inspired TUI editor using the RataTUI ecosystem.

Navigate right (l), left (h), up (k) and down (j), using vim motions.
    
Traverse words forward (w) and backward (b).
        
Select text (v), including selection between \"brackets\" (ciw).
        
Copy and paste text: 
        
Built-in search using the '/' command.
        
This editor is under active development.
Don't hesitate to open issues or submit pull requests to contribute!
",
        ));

        Ok(Self { path, modified: false, state, input: keybindings.into() })
    }

    fn init_commands(&mut self, tx: Option<UnboundedSender<Action>>) {
        self.state.command.available_commands.push(Command::new(
            "quit".to_string(),
            "Quit the app".to_string(),
            Box::new(move || {
                log::info!("Quitting the app from command");
                if let Some(tx) = &tx {
                    tx.send(Action::Quit).unwrap();
                } else {
                    log::error!("No command tx available");
                }
            }),
            vec!["q".to_string()],
        ));
    }

    fn save(&mut self) -> io::Result<()> {
        if !self.modified {
            return Ok(());
        }
        if let Some(path) = &self.path {
            let mut f = io::BufWriter::new(fs::File::create(path)?);
            for (maybe_char, Index2 { col, row }) in self.state.lines.iter() {
                if let Some(c) = maybe_char {
                    write!(f, "{}", c)?;
                }
                if row < self.state.lines.len() - 1 {
                    writeln!(f)?;
                }
            }
            self.modified = false;
        }
        Ok(())
    }
}

impl Component for Editor {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        self.buffers.values_mut().for_each(|b| b.init_commands(self.command_tx.clone()));
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        self.buffers.values_mut().for_each(|b| b.input = self.config.keybindings.clone().into());
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let current_buffer = self.current_buffer().unwrap();
        let input = &mut current_buffer.input;
        let state = &mut current_buffer.state;

        let maybe_custom = input.on_key(key, state);
        if let Some(custom) = maybe_custom {
            match custom.0 {
                PapierAction::Quit => {
                    if let Some(tx) = &self.command_tx {
                        tx.send(Action::Quit)?;
                    }
                },
            }
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let [_, bottom] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
        let current_buffer = self.current_buffer().unwrap();
        let state = &mut current_buffer.state;
        let theme = EditorTheme::default()
            .base(EditorTheme::default().base.bold().bg(Color::Reset))
            .status_line(
                StatusLine::default()
                    .style_text(match state.mode {
                        edtui::EditorMode::Insert => Style::default().bg(Color::LightYellow).fg(LIGHT_GRAY),
                        edtui::EditorMode::Normal => Style::default().bg(Color::Reset).fg(LIGHT_GRAY),
                        edtui::EditorMode::Visual => Style::default().bg(Color::LightMagenta).fg(LIGHT_GRAY),
                        edtui::EditorMode::Search => Style::default().bg(Color::LightBlue).fg(LIGHT_GRAY),
                        EditorMode::Command => Style::default().bg(Color::LightRed).fg(LIGHT_GRAY),
                    })
                    .style_line(match state.mode {
                        edtui::EditorMode::Insert => Style::default().bg(Color::Yellow),
                        edtui::EditorMode::Normal => Style::default().bg(Color::Reset),
                        edtui::EditorMode::Visual => Style::default().bg(Color::Magenta),
                        edtui::EditorMode::Search => Style::default().bg(Color::Blue),
                        EditorMode::Command => Style::default().bg(Color::Red),
                    })
                    .align_left(true),
            )
            .cursor_style(
                Style::default()
                    .fg(match state.mode {
                        edtui::EditorMode::Insert => Color::White,
                        _ => Color::DarkGray,
                    })
                    .bg(match state.mode {
                        edtui::EditorMode::Insert => Color::Reset,
                        _ => Color::White,
                    })
                    .underlined(),
            )
            .line_numbers_style(Style::default().fg(Color::DarkGray).bg(Color::Reset));

        let editor = EditorView::new(state).theme(theme);
        let buf = f.buffer_mut();
        editor.render(bottom, buf);

        Ok(())
    }
}

const LIGHT_GRAY: Color = Color::Rgb(248, 250, 252);
