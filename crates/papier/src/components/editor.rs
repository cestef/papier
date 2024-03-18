use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
    PapierAction,
};
use color_eyre::eyre::{eyre, Result};
use config::File;
use crossterm::event::{Event, KeyCode, KeyEvent};
use edtui::{
    actions::Execute, state::command::Command, view::EditorMessage, EditorMode, EditorState, EditorTheme, EditorView,
    Index2, Input, Lines, StatusLine,
};
use log::debug;
use ratatui::{prelude::*, style::palette::tailwind::PURPLE, widgets::*};
use ratatui_explorer::{FileExplorer, Input as ExplorerInput, Theme as FileTheme};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    fs,
    io::{self, BufRead, Write},
    path::PathBuf,
    time::Duration,
};
use tokio::sync::mpsc::UnboundedSender;

pub struct Theme<'a> {
    pub editor: EditorTheme<'a>,
}

#[derive(Default)]
pub struct Editor {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    buffers: Vec<Buffer>,
    current_buffer: Option<usize>,
}

impl Editor {
    pub fn new(files: Vec<PathBuf>) -> Self {
        let config = Config::default();
        let mut buffers = Vec::new();
        if !files.is_empty() {
            if files.iter().any(|f| f.is_dir()) {
                log::error!("Directories are not supported");
                std::process::exit(1);
            }
            for file in files {
                let buffer = Buffer::new(Some(file.clone()), config.keybindings.clone(), None, None).unwrap();
                buffers.push(buffer);
            }
        } else {
            let welcome_buffer = Buffer::new(None, config.keybindings.clone(), None, Some("Welcome".into())).unwrap();
            buffers.push(welcome_buffer);
            let test_buffer = Buffer::new(None, config.keybindings.clone(), None, Some("Test".into())).unwrap();
            buffers.push(test_buffer);
            let test_buffer = Buffer::new(None, config.keybindings.clone(), None, Some("Test".into())).unwrap();
            buffers.push(test_buffer);
            let test_buffer = Buffer::new(None, config.keybindings.clone(), None, Some("Test".into())).unwrap();
            buffers.push(test_buffer);
            let test_buffer = Buffer::new(None, config.keybindings.clone(), None, Some("Test".into())).unwrap();
            buffers.push(test_buffer);
            let test_buffer = Buffer::new(None, config.keybindings.clone(), None, Some("Test".into())).unwrap();
            buffers.push(test_buffer);
            let test_buffer = Buffer::new(None, config.keybindings.clone(), None, Some("Test".into())).unwrap();
            buffers.push(test_buffer);
            let test_buffer = Buffer::new(None, config.keybindings.clone(), None, Some("Test".into())).unwrap();
            buffers.push(test_buffer);
        };
        Self { command_tx: None, config, buffers, current_buffer: Some(0) }
    }

    pub fn current_buffer(&mut self) -> Option<&mut Buffer> {
        self.current_buffer.as_ref().and_then(|b| self.buffers.get_mut(*b))
    }
}

pub struct Buffer {
    name: Option<String>,
    path: Option<PathBuf>,
    modified: bool,
    state: EditorState,
    input: Input<PapierAction>,
    message: Option<String>,
    explorer: FileExplorer,
    explorer_state: FileExplorerState,
}

struct FileExplorerState {
    open: bool,
}

impl Buffer {
    fn init_commands(input: &mut Input<PapierAction>) {
        input.command.available_commands.extend([
            Command::new("quit".to_string(), "Quit the app".to_string(), vec!["q".to_string()], |_| PapierAction::Quit),
            Command::new("save".to_string(), "Save the current file".to_string(), vec!["w".to_string()], |_| {
                PapierAction::Save
            }),
            Command::new("save_all".to_string(), "Save all open files".to_string(), vec!["wa".to_string()], |_| {
                PapierAction::SaveAll
            }),
            Command::new(
                "save_as".to_string(),
                "Save the current file as a new file".to_string(),
                vec!["W".to_string()],
                PapierAction::SaveAs,
            ),
            Command::new(
                "next_buffer".to_string(),
                "Switch to the next buffer".to_string(),
                vec!["n".to_string()],
                |_| PapierAction::NextBuffer,
            ),
            Command::new(
                "previous_buffer".to_string(),
                "Switch to the previous buffer".to_string(),
                vec!["N".to_string()],
                |_| PapierAction::PreviousBuffer,
            ),
            Command::new("open".to_string(), "Open a file".to_string(), vec!["o".to_string()], PapierAction::Open),
            Command::new("quit_all".to_string(), "Quit the app".to_string(), vec!["qa".to_string()], |_| {
                PapierAction::QuitAll
            }),
        ]);
    }

    fn new(
        path: Option<PathBuf>,
        keybindings: KeyBindings,
        tx: Option<UnboundedSender<Action>>,
        name: Option<String>,
    ) -> io::Result<Self> {
        let state = match path {
            Some(ref path) => {
                let lines = if !path.exists() {
                    log::debug!("File does not exist: {}", path.to_string_lossy());
                    "".to_string()
                } else {
                    let file = fs::File::open(path)?;
                    let reader = io::BufReader::new(file);
                    let lines = reader.lines();
                    lines.map_while(Result::ok).collect::<Vec<String>>().join("\n")
                };
                EditorState::new(
                    Lines::from(lines.as_str()),
                    path.to_string_lossy().split('.').last().unwrap_or_default(),
                )
            },
            None => EditorState::new(
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
            ),
        };
        let mut input: Input<_> = keybindings.into();
        Self::init_commands(&mut input);
        Ok(Self {
            path: path.clone(),
            modified: false,
            state,
            input,
            message: None,
            name: name.or_else(|| path.map(|p| p.file_name().unwrap().to_string_lossy().to_string())),
            explorer: FileExplorer::with_theme(FileTheme::default().add_default_title())?,
            explorer_state: FileExplorerState { open: false },
        })
    }

    fn save(&mut self) -> io::Result<()> {
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
        self.buffers.iter_mut().for_each(|b| {
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
        let explorer = &mut current_buffer.explorer;
        let explorer_state = &mut current_buffer.explorer_state;

        if explorer_state.open {
            if key.code == KeyCode::Esc {
                explorer_state.open = false;
                return Ok(None);
            }
            explorer.handle(&Event::Key(key))?;

            if !explorer.current().is_dir() && (key.code == KeyCode::Enter || key.code == KeyCode::Char('l')) {
                let path = explorer.current().path().to_path_buf();
                let buffer = Buffer::new(Some(path), self.config.keybindings.clone(), None, None)?;
                self.buffers.push(buffer);
                self.current_buffer = Some(self.buffers.len() - 1);
                return Ok(None);
            }
            return Ok(None);
        }

        let maybe_custom = input.on_key(key, state);

        if let Some(custom) = maybe_custom {
            match custom.0 {
                PapierAction::Quit => {
                    // If there is still a buffer open, close it, else quit the app
                    if self.buffers.len() > 1 {
                        let index = self.current_buffer.unwrap();
                        self.buffers.remove(index);
                        // check if the previous buffer exists
                        if !self.buffers.is_empty() {
                            self.current_buffer = Some((index + self.buffers.len() - 1) % self.buffers.len());
                            self.current_buffer().unwrap().state.reset_highlighter();
                        }
                    } else {
                        return Ok(Some(Action::Quit));
                    }
                },
                PapierAction::Save => {
                    debug!("Saving buffer");
                    current_buffer.save()?;
                },
                PapierAction::SaveAll => {
                    debug!("Saving all buffers");
                    for buffer in self.buffers.iter_mut() {
                        buffer.save()?;
                    }
                },
                PapierAction::SaveAs(i) => {
                    let path = PathBuf::from(i);
                    debug!("Saving buffer as: {:?}", path);
                    current_buffer.save_as(path)?;
                },
                PapierAction::NextBuffer => {
                    let index = self.current_buffer.unwrap();
                    let next = (index + 1) % self.buffers.len();
                    self.current_buffer = Some(next);
                    self.current_buffer().unwrap().state.reset_highlighter();
                },
                PapierAction::PreviousBuffer => {
                    let index = self.current_buffer.unwrap();
                    let next = (index + self.buffers.len() - 1) % self.buffers.len();
                    self.current_buffer = Some(next);
                    self.current_buffer().unwrap().state.reset_highlighter();
                },
                PapierAction::Open(i) => {
                    let path = PathBuf::from(i);
                    debug!("Opening file: {:?}", path);
                    let buffer = Buffer::new(Some(path), self.config.keybindings.clone(), None, None)?;
                    self.buffers.push(buffer);
                    self.current_buffer = Some(self.buffers.len() - 1);
                },
                PapierAction::QuitAll => return Ok(Some(Action::Quit)),
                PapierAction::ToggleExplorer => {
                    current_buffer.explorer_state.open = !current_buffer.explorer_state.open;
                },
            }
        };

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let buffer_index = self.current_buffer.unwrap();
        let buffer_count = self.buffers.len();
        let current_buffer = self.current_buffer().unwrap();
        let state = &mut current_buffer.state;

        let area = area.inner(&Margin { horizontal: 1, vertical: 1 });
        let [explorer, editor] = Layout::horizontal([
            Constraint::Length(if current_buffer.explorer_state.open { 20 } else { 0 }),
            Constraint::Min(0),
        ])
        .areas(area);
        let [top, bottom] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(editor);

        if current_buffer.explorer_state.open {
            f.render_widget(&current_buffer.explorer.widget(), explorer);
        }

        let theme = EditorTheme::default()
            .base(EditorTheme::default().base.bold().bg(Color::Reset))
            .selection_style(Style::default().bg(Color::LightMagenta).fg(Color::Reset))
            .status_line(
                StatusLine::default()
                    .style_mode(match state.mode {
                        edtui::EditorMode::Insert => Style::default().bg(Color::LightYellow).fg(LIGHT_GRAY),
                        edtui::EditorMode::Normal => Style::default().bg(Color::Reset).fg(LIGHT_GRAY),
                        edtui::EditorMode::Visual => Style::default().bg(Color::LightMagenta).fg(LIGHT_GRAY),
                        edtui::EditorMode::Search => Style::default().bg(Color::LightBlue).fg(LIGHT_GRAY),
                        EditorMode::Command => Style::default().bg(Color::Gray).fg(LIGHT_GRAY),
                    })
                    .style_line(match state.mode {
                        edtui::EditorMode::Insert => Style::default().bg(Color::Yellow),
                        edtui::EditorMode::Normal => Style::default().bg(Color::Reset),
                        edtui::EditorMode::Visual => Style::default().bg(Color::Magenta),
                        edtui::EditorMode::Search => Style::default().bg(Color::Blue),
                        EditorMode::Command => Style::default().bg(Color::DarkGray),
                    })
                    .text(Some(format!(
                        "{}/{} {}:{}",
                        buffer_index + 1,
                        buffer_count,
                        state.cursor.row,
                        state.cursor.col
                    )))
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
        let n = self.buffers.len();
        let top_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints((0..n).map(|_| Constraint::Ratio(1, n as u32)).collect::<Vec<_>>())
            .split(top);

        for (i, area) in top_areas.iter().enumerate() {
            let buffer = &self.buffers[i];
            let style = if i == buffer_index { Style::default().bg(Color::DarkGray) } else { Style::default() };
            let text = buffer.name.clone().unwrap_or_else(|| "Untitled".to_string());
            Paragraph::new(format!(" {}: {}", i + 1, text)).style(style).wrap(Wrap { trim: false }).render(*area, buf);
        }
        Ok(())
    }
}

const LIGHT_GRAY: Color = Color::Rgb(248, 250, 252);
