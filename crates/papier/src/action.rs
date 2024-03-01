use std::{fmt, string::ToString};

use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum Action {
    Quit,
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Refresh,
    Error(String),
}
