// Voice command registry - stores and persists command definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
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

    /// Create a registry using the default config path with worktree context
    pub fn with_default_path_context(
        worktree_context: Option<&crate::worktree::WorktreeContext>,
    ) -> Result<Self, RegistryError> {
        let config_dir = crate::paths::get_config_dir(worktree_context).map_err(|e| {
            RegistryError::LoadError(format!("Could not determine config directory: {}", e))
        })?;
        let config_path = config_dir.join("commands.json");
        Ok(Self::new(config_path))
    }

    /// Create a registry using the default config path (API-compatible, uses main repo path)
    pub fn with_default_path() -> Result<Self, RegistryError> {
        Self::with_default_path_context(None)
    }

    /// Load commands from the persistence file
    pub fn load(&mut self) -> Result<(), RegistryError> {
        crate::debug!("Loading commands from {:?}", self.config_path);

        if !self.config_path.exists() {
            crate::debug!("No commands file found, starting with empty registry");
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

        crate::info!("Loaded {} commands from registry", self.commands.len());
        Ok(())
    }

    /// Persist commands to the file using atomic write (temp file + rename)
    fn persist(&self) -> Result<(), RegistryError> {
        crate::debug!("Persisting {} commands to {:?}", self.commands.len(), self.config_path);

        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;
        }

        let commands: Vec<&CommandDefinition> = self.commands.values().collect();
        let content = serde_json::to_string_pretty(&commands)
            .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        // Use atomic temp file + rename pattern
        let temp_path = self.config_path.with_extension("tmp");

        // Write to temp file with explicit sync
        {
            let mut file = File::create(&temp_path)
                .map_err(|e| RegistryError::PersistenceError(format!("Failed to create temp file: {}", e)))?;
            file.write_all(content.as_bytes())
                .map_err(|e| RegistryError::PersistenceError(format!("Failed to write: {}", e)))?;
            file.sync_all()
                .map_err(|e| RegistryError::PersistenceError(format!("Failed to sync: {}", e)))?;
        } // File closed here

        // Atomic rename
        fs::rename(&temp_path, &self.config_path)
            .map_err(|e| {
                // Clean up temp file on error
                let _ = fs::remove_file(&temp_path);
                RegistryError::PersistenceError(format!("Failed to rename: {}", e))
            })?;

        crate::debug!("Commands persisted successfully");
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
        self.commands.insert(cmd.id, cmd);
        self.persist()?;
        Ok(())
    }

    /// Update an existing command
    #[must_use = "this returns a Result that should be handled"]
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
    #[must_use = "this returns a Result that should be handled"]
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
