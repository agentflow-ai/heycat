// Voice command registry - stores and persists command definitions
//
// Commands are stored in SpacetimeDB and cached locally for quick access.
// Mutations go through SpacetimeDB reducers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::spacetimedb::client::SpacetimeClient;

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
    /// Custom user-defined action
    Custom,
}

impl std::str::FromStr for ActionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open_app" => Ok(ActionType::OpenApp),
            "type_text" => Ok(ActionType::TypeText),
            "system_control" => Ok(ActionType::SystemControl),
            "custom" => Ok(ActionType::Custom),
            _ => Err(format!("Unknown action type: {}", s)),
        }
    }
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
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RegistryError {
    /// Trigger phrase is empty
    #[error("Trigger phrase cannot be empty")]
    EmptyTrigger,
    /// Command with this ID already exists
    #[error("Command with ID {0} already exists")]
    DuplicateId(Uuid),
    /// Command not found
    #[error("Command with ID {0} not found")]
    NotFound(Uuid),
    /// Failed to persist commands
    #[error("Failed to persist commands: {0}")]
    PersistenceError(String),
    /// Failed to load commands
    #[error("Failed to load commands: {0}")]
    LoadError(String),
}

/// Registry for voice commands
///
/// Commands are stored in SpacetimeDB and cached locally.
/// The local cache is updated on load and after each mutation.
pub struct CommandRegistry {
    /// Commands indexed by ID (local cache)
    commands: HashMap<Uuid, CommandDefinition>,
    /// SpacetimeDB client for persistence
    client: Arc<Mutex<SpacetimeClient>>,
}

impl CommandRegistry {
    /// Create a new registry with a SpacetimeDB client
    pub fn new(client: Arc<Mutex<SpacetimeClient>>) -> Self {
        Self {
            commands: HashMap::new(),
            client,
        }
    }

    /// Load commands from SpacetimeDB
    pub fn load(&mut self) -> Result<(), RegistryError> {
        crate::debug!("Loading commands from SpacetimeDB");

        let client = self.client.lock().map_err(|e| {
            RegistryError::LoadError(format!("Failed to acquire client lock: {}", e))
        })?;

        let commands = client.list_voice_commands()?;

        self.commands.clear();
        for cmd in commands {
            self.commands.insert(cmd.id, cmd);
        }

        crate::info!("Loaded {} commands from SpacetimeDB", self.commands.len());
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
    #[must_use = "this returns a Result that should be handled"]
    pub fn add(&mut self, cmd: CommandDefinition) -> Result<(), RegistryError> {
        self.validate(&cmd, true)?;

        // Persist to SpacetimeDB
        {
            let client = self.client.lock().map_err(|e| {
                RegistryError::PersistenceError(format!("Failed to acquire client lock: {}", e))
            })?;
            client.add_voice_command(&cmd)?;
        }

        // Update local cache
        self.commands.insert(cmd.id, cmd);
        crate::debug!("Added command to registry");
        Ok(())
    }

    /// Update an existing command
    #[must_use = "this returns a Result that should be handled"]
    pub fn update(&mut self, cmd: CommandDefinition) -> Result<(), RegistryError> {
        if !self.commands.contains_key(&cmd.id) {
            return Err(RegistryError::NotFound(cmd.id));
        }
        self.validate(&cmd, false)?;

        // Persist to SpacetimeDB
        {
            let client = self.client.lock().map_err(|e| {
                RegistryError::PersistenceError(format!("Failed to acquire client lock: {}", e))
            })?;
            client.update_voice_command(&cmd)?;
        }

        // Update local cache
        self.commands.insert(cmd.id, cmd);
        crate::debug!("Updated command in registry");
        Ok(())
    }

    /// Delete a command by ID
    #[must_use = "this returns a Result that should be handled"]
    pub fn delete(&mut self, id: Uuid) -> Result<(), RegistryError> {
        if !self.commands.contains_key(&id) {
            return Err(RegistryError::NotFound(id));
        }

        // Delete from SpacetimeDB
        {
            let client = self.client.lock().map_err(|e| {
                RegistryError::PersistenceError(format!("Failed to acquire client lock: {}", e))
            })?;
            client.delete_voice_command(id)?;
        }

        // Update local cache
        self.commands.remove(&id);
        crate::debug!("Deleted command from registry");
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
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if registry is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[cfg(test)]
#[path = "registry_test.rs"]
mod tests;
