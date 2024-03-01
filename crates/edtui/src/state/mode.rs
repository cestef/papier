use serde::{Deserialize, Serialize};
/// The editor mode.
#[derive(Default, Debug, Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EditorMode {
    #[default]
    Normal,
    Insert,
    Visual,
    Search,
    Command,
}

impl EditorMode {
    /// Returns the name of the [`EditorMode`] as a string.
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Normal => "Normal".to_string(),
            Self::Insert => "Insert".to_string(),
            Self::Visual => "Visual".to_string(),
            Self::Search => "Search".to_string(),
            Self::Command => "Command".to_string(),
        }
    }
}
