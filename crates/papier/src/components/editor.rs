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
    actions::Execute, state::command::Command, view::EditorMessage, EditorMode, EditorState, EditorTheme, EditorView,
    Index2, Input, Lines, StatusLine,
};
use ratatui::{prelude::*, style::palette::tailwind::PURPLE, widgets::*};
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
}

impl Editor {
    pub fn new(file: Option<PathBuf>) -> Self {
        let config = Config::default();
        let mut buffers = HashMap::new();
        let current_buffer = if let Some(ref f) = file {
            let buffer = Buffer::from_path(f.clone(), config.keybindings.clone(), None).unwrap();
            buffers.insert(f.to_string_lossy().to_string(), buffer);
            Some(f.to_string_lossy().to_string())
        } else {
            let buffer = Buffer::new(None, config.keybindings.clone(), None).unwrap();
            buffers.insert("".to_string(), buffer);
            Some("".to_string())
        };
        Self { command_tx: None, config, buffers, current_buffer }
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
    message: Option<String>,
}

impl Buffer {
    fn from_path(path: PathBuf, keybindings: KeyBindings, tx: Option<UnboundedSender<Action>>) -> io::Result<Self> {
        let file = fs::File::open(&path)?;
        let reader = io::BufReader::new(file);
        let lines: &str = &reader.lines().map_while(Result::ok).collect::<Vec<String>>().join("\n");
        log::debug!("Read file: {}", lines);
        let state = EditorState::new(Lines::from(lines), path.to_string_lossy().split('.').last().unwrap_or_default());
        let mut input: Input<_> = keybindings.into();
        Self::init_commands(&mut input);
        Ok(Self { path: Some(path), modified: false, state, input, message: None })
    }

    fn init_commands(input: &mut Input<PapierAction>) {
        input.command.available_commands.extend([
            Command::new("quit".to_string(), "Quit the app".to_string(), vec!["q".to_string()], |_| PapierAction::Quit),
            Command::new("save".to_string(), "Save the current file".to_string(), vec!["w".to_string()], |_| {
                PapierAction::Save
            }),
            Command::new(
                "save_as".to_string(),
                "Save the current file as a new file".to_string(),
                vec!["wq".to_string()],
                PapierAction::SaveAs,
            ),
        ]);
    }

    fn new(path: Option<PathBuf>, keybindings: KeyBindings, tx: Option<UnboundedSender<Action>>) -> io::Result<Self> {
        let state = EditorState::new(
            Lines::from(
                "papier is a light-weight vim inspired TUI editor using the RataTUI ecosystem.

Navigate right (l), left (h), up (k) and down (j), using vim motions.
    
Traverse words forward (w) and backward (b).
        
Select text (v), including selection between \"brackets\" (ciw).
        
Copy and paste text: 
        
Built-in search using the '/' command.
        
This editor is under active development.
Don't hesitate to open issues or submit pull requests to contribute!
",
            ),
            "txt",
        );
        let mut input: Input<_> = keybindings.into();
        Self::init_commands(&mut input);
        Ok(Self { path, modified: false, state, input, message: None })
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

    fn save_as(&mut self, path: PathBuf) -> io::Result<()> {
        let mut f = io::BufWriter::new(fs::File::create(&path)?);
        for (maybe_char, Index2 { col, row }) in self.state.lines.iter() {
            if let Some(c) = maybe_char {
                write!(f, "{}", c)?;
            }
            if row < self.state.lines.len() - 1 {
                writeln!(f)?;
            }
        }
        self.modified = false;
        self.path = Some(path);
        Ok(())
    }
}

impl Component for Editor {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        self.buffers.values_mut().for_each(|b| {
            let mut input: Input<_> = self.config.keybindings.clone().into();
            input.command.available_commands = b.input.command.available_commands.clone();
            b.input = input;
        });
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
                PapierAction::Save => {
                    current_buffer.save()?;
                },
                PapierAction::SaveAs(i) => {
                    let args = i.split_whitespace().skip(1).collect::<Vec<&str>>();
                    if args.len() != 1 {
                        current_buffer.message = Some("Invalid arguments".to_string());
                    } else {
                        let path = PathBuf::from(args[0]);
                        current_buffer.save_as(path)?;
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
            .selection_style(Style::default().bg(Color::LightMagenta).fg(Color::Reset))
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
            .cursor_style(match state.mode {
                edtui::EditorMode::Insert => Style::default().underlined(),
                _ => Style::default().bg(Color::White).fg(Color::Black),
            })
            .line_numbers_style(Style::default().fg(Color::DarkGray).bg(Color::Reset));

        let editor = EditorView::new(state).theme(theme).message(
            current_buffer.message.as_ref().map(|m| EditorMessage::new(m.to_string(), Duration::from_secs(3))),
        );
        let buf = f.buffer_mut();
        editor.render(bottom, buf);

        Ok(())
    }
}

const LIGHT_GRAY: Color = Color::Rgb(248, 250, 252);
