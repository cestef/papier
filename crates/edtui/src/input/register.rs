use std::collections::HashMap;

use crossterm::event::KeyEvent;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{Action, Execute},
    EditorMode, EditorState,
};

#[derive(Clone, Debug, Default)]
pub struct Register<I>
where
    I: Clone + Execute + Serialize + Deserialize<'static>,
{
    lookup: Vec<KeyEvent>,
    register: HashMap<RegisterKey, Action<I>>,
}

impl<I> Register<I>
where
    I: Clone + Execute + Serialize + Deserialize<'static>,
{
    /// Constructs a new Register
    #[must_use]
    pub fn new() -> Self {
        Self { lookup: Vec::new(), register: HashMap::new() }
    }

    /// Insert a new callback to the registry
    pub fn insert<T: Into<Action<I>>>(&mut self, k: RegisterKey, v: T) {
        self.register.insert(k, v.into());
    }

    /// Returns an action for a specific register key, if present.
    /// Returns an action only if there is an exact match. If there
    /// are multiple matches or an inexact match, the specified key
    /// is appended to the lookup vector.
    /// If there is an exact match or if none of the keys in the registry
    /// starts with the current sequence, the lookup sequence is reset.
    #[must_use]
    pub fn get(&mut self, c: KeyEvent, mode: EditorMode) -> Option<Action<I>> {
        let key = self.create_register_key(c, mode);

        match self.register.keys().filter(|k| k.mode == key.mode && k.keys.starts_with(&key.keys)).count() {
            0 => {
                self.lookup.clear();
                None
            },
            1 => self.register.get(&key).map(|action| {
                self.lookup.clear();
                action.clone()
            }),
            _ => None,
        }
    }

    fn create_register_key(&mut self, c: KeyEvent, mode: EditorMode) -> RegisterKey {
        self.lookup.push(c);
        RegisterKey::new(self.lookup.clone(), mode)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct RegisterKey {
    pub keys: Vec<KeyEvent>,
    pub mode: EditorMode,
}

pub type RegisterCB = fn(&mut EditorState);

#[derive(Clone, Debug)]
pub struct RegisterVal(pub fn(&mut EditorState));

impl RegisterKey {
    pub fn new<T>(keys: Vec<T>, mode: EditorMode) -> Self
    where
        T: Into<KeyEvent>,
    {
        Self { keys: keys.into_iter().map(Into::into).collect(), mode }
    }

    pub fn n<T>(keys: Vec<T>) -> Self
    where
        T: Into<KeyEvent>,
    {
        Self::new(keys, EditorMode::Normal)
    }

    pub fn v<T>(keys: Vec<T>) -> Self
    where
        T: Into<KeyEvent>,
    {
        Self::new(keys, EditorMode::Visual)
    }

    pub fn i<T>(keys: Vec<T>) -> Self
    where
        T: Into<KeyEvent>,
    {
        Self::new(keys, EditorMode::Insert)
    }

    pub fn s<T>(keys: Vec<T>) -> Self
    where
        T: Into<KeyEvent>,
    {
        Self::new(keys, EditorMode::Search)
    }

    pub fn c<T>(keys: Vec<T>) -> Self
    where
        T: Into<KeyEvent>,
    {
        Self::new(keys, EditorMode::Command)
    }
}
