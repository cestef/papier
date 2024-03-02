use jagged::Index2;

#[derive(Default, Clone, Debug)]
/// Represents the state of the command mode.
pub struct CommandState<I> {
    pub(crate) start_cursor: Index2,
    pub(crate) input: String,
    pub available_commands: Vec<Command<I>>,
}

#[derive(Clone, Debug)]
/// Represents a command that can be executed in command mode.
pub struct Command<I> {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub action: fn(String) -> I,
}

impl<I: Default> Default for Command<I> {
    fn default() -> Self {
        Self { name: String::new(), description: String::new(), aliases: Vec::new(), action: |_| I::default() }
    }
}

impl<I> Command<I> {
    pub fn new(name: String, description: String, aliases: Vec<String>, action: fn(String) -> I) -> Self {
        Self { name, description, aliases, action }
    }

    pub fn name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    pub fn description(&mut self, description: String) -> &mut Self {
        self.description = description;
        self
    }

    pub fn aliases(&mut self, aliases: Vec<String>) -> &mut Self {
        self.aliases = aliases;
        self
    }
}

impl<I> CommandState<I> {
    /// Returns the length of the current search pattern.
    pub(crate) fn command_len(&self) -> usize {
        self.input.len()
    }

    /// Clears both the search pattern and matched indices.
    pub(crate) fn clear(&mut self) {
        self.input.clear();
    }

    /// Appends a character to the command.
    pub(crate) fn push_char(&mut self, ch: char) {
        self.input.push(ch);
    }

    /// Removes the last character from the command.
    pub(crate) fn remove_char(&mut self) {
        self.input.pop();
    }

    pub fn add_command(&mut self, name: String, description: String, aliases: Vec<String>, action: fn(String) -> I) {
        self.available_commands.push(Command { name, description, aliases, action });
    }
}
