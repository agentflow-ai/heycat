// Voice command registry - stores and persists command definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Type of action to execute when a command matches
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// Open an application
    OpenApp,
    /// Type text via keyboard simulation
    TypeText,
    /// System control (volume, brightness, etc.)
    SystemControl,
    /// Multi-step workflow
    Workflow,
    /// Custom user-defined action
    Custom,
}

/// A voice command definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandDefinition {
    /// Unique identifier for the command
    pub id: Uuid,
    /// Trigger phrase (e.g., "open slack")
    pub trigger: String,
    /// Type of action to execute
    pub action_type: ActionType,
    /// Action-specific parameters
    pub parameters: HashMap<String, String>,
    /// Whether the command is enabled
    pub enabled: bool,
}

/// Error types for registry operations
#[derive(Debug, Clone, PartialEq)]
pub enum RegistryError {
    /// Trigger phrase is empty
    EmptyTrigger,
    /// Command with this ID already exists
    DuplicateId(Uuid),
    /// Command not found
    NotFound(Uuid),
    /// Failed to persist commands
    PersistenceError(String),
    /// Failed to load commands
    LoadError(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::EmptyTrigger => write!(f, "Trigger phrase cannot be empty"),
            RegistryError::DuplicateId(id) => write!(f, "Command with ID {} already exists", id),
            RegistryError::NotFound(id) => write!(f, "Command with ID {} not found", id),
            RegistryError::PersistenceError(msg) => write!(f, "Failed to persist commands: {}", msg),
            RegistryError::LoadError(msg) => write!(f, "Failed to load commands: {}", msg),
        }
    }
}

impl std::error::Error for RegistryError {}

/// Registry for voice commands
#[derive(Debug)]
pub struct CommandRegistry {
    /// Commands indexed by ID
    commands: HashMap<Uuid, CommandDefinition>,
    /// Path to persistence file
    config_path: PathBuf,
}

impl CommandRegistry {
    /// Create a new registry with the given config path
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            commands: HashMap::new(),
            config_path,
        }
    }

    /// Create a registry using the default config path
    pub fn with_default_path() -> Result<Self, RegistryError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| RegistryError::LoadError("Could not determine config directory".to_string()))?;
        let config_path = config_dir.join("heycat").join("commands.json");
        Ok(Self::new(config_path))
    }

    /// Load commands from the persistence file
    pub fn load(&mut self) -> Result<(), RegistryError> {
        if !self.config_path.exists() {
            // No file yet, start with empty registry
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| RegistryError::LoadError(e.to_string()))?;

        let commands: Vec<CommandDefinition> = serde_json::from_str(&content)
            .map_err(|e| RegistryError::LoadError(e.to_string()))?;

        self.commands.clear();
        for cmd in commands {
            self.commands.insert(cmd.id, cmd);
        }

        Ok(())
    }

    /// Persist commands to the file
    fn persist(&self) -> Result<(), RegistryError> {
        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;
        }

        let commands: Vec<&CommandDefinition> = self.commands.values().collect();
        let content = serde_json::to_string_pretty(&commands)
            .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        fs::write(&self.config_path, content)
            .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    /// Validate a command definition
    fn validate(&self, cmd: &CommandDefinition, is_new: bool) -> Result<(), RegistryError> {
        if cmd.trigger.trim().is_empty() {
            return Err(RegistryError::EmptyTrigger);
        }

        if is_new && self.commands.contains_key(&cmd.id) {
            return Err(RegistryError::DuplicateId(cmd.id));
        }

        Ok(())
    }

    /// Add a new command to the registry
    pub fn add(&mut self, cmd: CommandDefinition) -> Result<(), RegistryError> {
        self.validate(&cmd, true)?;
        self.commands.insert(cmd.id, cmd);
        self.persist()?;
        Ok(())
    }

    /// Update an existing command
    pub fn update(&mut self, cmd: CommandDefinition) -> Result<(), RegistryError> {
        if !self.commands.contains_key(&cmd.id) {
            return Err(RegistryError::NotFound(cmd.id));
        }
        self.validate(&cmd, false)?;
        self.commands.insert(cmd.id, cmd);
        self.persist()?;
        Ok(())
    }

    /// Delete a command by ID
    pub fn delete(&mut self, id: Uuid) -> Result<(), RegistryError> {
        if self.commands.remove(&id).is_none() {
            return Err(RegistryError::NotFound(id));
        }
        self.persist()?;
        Ok(())
    }

    /// List all commands
    pub fn list(&self) -> Vec<&CommandDefinition> {
        self.commands.values().collect()
    }

    /// Get a command by ID
    pub fn get(&self, id: Uuid) -> Option<&CommandDefinition> {
        self.commands.get(&id)
    }

    /// Get the number of commands
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[cfg(test)]
#[path = "registry_test.rs"]
mod tests;
