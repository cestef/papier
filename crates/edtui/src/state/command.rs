use jagged::Index2;

#[derive(Default)]
/// Represents the state of the command mode.
pub struct CommandState {
    pub(crate) start_cursor: Index2,
    pub(crate) input: String,
    pub available_commands: Vec<Command>,
}

/// Represents a command that can be executed in command mode.
pub struct Command {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub execute: Box<dyn Fn()>,
}

impl Default for Command {
    fn default() -> Self {
        Self { name: String::new(), description: String::new(), execute: Box::new(|| {}), aliases: Vec::new() }
    }
}

impl Command {
    pub fn new(name: String, description: String, execute: Box<dyn Fn()>, aliases: Vec<String>) -> Self {
        Self { name, description, execute, aliases }
    }

    pub fn name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    pub fn description(&mut self, description: String) -> &mut Self {
        self.description = description;
        self
    }
}

impl CommandState {
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

    pub fn execute(&self) {
        if let Some(command) = self
            .available_commands
            .iter()
            .find(|command| command.name == self.input || command.aliases.contains(&self.input))
        {
            (command.execute)();
        }
    }

    pub fn add_command(&mut self, name: String, description: String, execute: Box<dyn Fn()>, aliases: Vec<String>) {
        self.available_commands.push(Command { name, description, execute, aliases });
    }
}
