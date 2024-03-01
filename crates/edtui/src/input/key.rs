use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Key {
    Char(char),
    Down,
    Up,
    Right,
    Left,
    Enter,
    Esc,
    Backspace,
    None,
}

impl From<KeyEvent> for Key {
    fn from(key: KeyEvent) -> Self {
        match key.code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Enter => Key::Enter,
            KeyCode::Down => Key::Down,
            KeyCode::Up => Key::Up,
            KeyCode::Right => Key::Right,
            KeyCode::Left => Key::Left,
            KeyCode::Esc => Key::Esc,
            KeyCode::Backspace => Key::Backspace,
            _ => Key::None,
        }
    }
}

impl From<Key> for KeyEvent {
    fn from(val: Key) -> Self {
        match val {
            Key::Char(c) => KeyEvent::from(KeyCode::Char(c)),
            Key::Enter => KeyEvent::from(KeyCode::Enter),
            Key::Down => KeyEvent::from(KeyCode::Down),
            Key::Up => KeyEvent::from(KeyCode::Up),
            Key::Right => KeyEvent::from(KeyCode::Right),
            Key::Left => KeyEvent::from(KeyCode::Left),
            Key::Esc => KeyEvent::from(KeyCode::Esc),
            Key::Backspace => KeyEvent::from(KeyCode::Backspace),
            Key::None => KeyEvent::from(KeyCode::Null),
        }
    }
}
