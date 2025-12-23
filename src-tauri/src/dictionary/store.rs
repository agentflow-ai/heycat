// Dictionary store - persists and loads dictionary entries for text expansion
// Follows the same pattern as voice_commands/registry.rs for file-based persistence
//
// NOTE: This is a foundational internal module consumed by tauri-commands.spec.md.
// The #[allow(dead_code)] attributes will be removed when production wiring is added.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

/// A dictionary entry for text expansion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DictionaryEntry {
    /// Unique identifier for the entry
    pub id: String,
    /// Trigger word/phrase (e.g., "brb")
    pub trigger: String,
    /// Expansion text (e.g., "be right back")
    pub expansion: String,
    /// Optional suffix appended after expansion
    #[serde(default)]
    pub suffix: Option<String>,
    /// Whether to simulate enter keypress after expansion
    #[serde(default, alias = "auto_enter")]
    pub auto_enter: bool,
    /// Whether to suppress any trailing punctuation from the transcription
    /// When true, trailing punctuation after the trigger match is stripped
    #[serde(default, alias = "disable_suffix")]
    pub disable_suffix: bool,
}

/// Error types for dictionary operations
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[allow(dead_code)]
pub enum DictionaryError {
    /// Entry with this ID already exists
    #[error("Entry with ID {0} already exists")]
    DuplicateId(String),
    /// Entry not found
    #[error("Entry with ID {0} not found")]
    NotFound(String),
    /// Failed to persist entries
    #[error("Failed to persist entries: {0}")]
    PersistenceError(String),
    /// Failed to load entries
    #[error("Failed to load entries: {0}")]
    LoadError(String),
}

/// Store for dictionary entries with file-based persistence
#[derive(Debug)]
#[allow(dead_code)]
pub struct DictionaryStore {
    /// Entries indexed by ID
    entries: HashMap<String, DictionaryEntry>,
    /// Path to persistence file
    config_path: PathBuf,
}

impl DictionaryStore {
    /// Create a new store with the given config path
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            entries: HashMap::new(),
            config_path,
        }
    }

    /// Create a store using the default config path with worktree context
    pub fn with_default_path_context(
        worktree_context: Option<&crate::worktree::WorktreeContext>,
    ) -> Result<Self, DictionaryError> {
        let config_dir = crate::paths::get_config_dir(worktree_context).map_err(|e| {
            DictionaryError::LoadError(format!("Could not determine config directory: {}", e))
        })?;
        let config_path = config_dir.join("dictionary.json");
        Ok(Self::new(config_path))
    }

    /// Create a store using the default config path (API-compatible, uses main repo path)
    pub fn with_default_path() -> Result<Self, DictionaryError> {
        Self::with_default_path_context(None)
    }

    /// Load entries from the persistence file
    pub fn load(&mut self) -> Result<(), DictionaryError> {
        crate::debug!("Loading dictionary from {:?}", self.config_path);

        if !self.config_path.exists() {
            crate::debug!("No dictionary file found, starting with empty store");
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| DictionaryError::LoadError(e.to_string()))?;

        let entries: Vec<DictionaryEntry> = serde_json::from_str(&content)
            .map_err(|e| DictionaryError::LoadError(e.to_string()))?;

        self.entries.clear();
        for entry in entries {
            self.entries.insert(entry.id.clone(), entry);
        }

        crate::info!("Loaded {} dictionary entries", self.entries.len());
        Ok(())
    }

    /// Persist entries to the file using atomic write (temp file + rename)
    fn save(&self) -> Result<(), DictionaryError> {
        crate::debug!(
            "Persisting {} dictionary entries to {:?}",
            self.entries.len(),
            self.config_path
        );

        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| DictionaryError::PersistenceError(e.to_string()))?;
        }

        let entries: Vec<&DictionaryEntry> = self.entries.values().collect();
        let content = serde_json::to_string_pretty(&entries)
            .map_err(|e| DictionaryError::PersistenceError(e.to_string()))?;

        // Use atomic temp file + rename pattern
        let temp_path = self.config_path.with_extension("tmp");

        // Write to temp file with explicit sync
        {
            let mut file = File::create(&temp_path).map_err(|e| {
                DictionaryError::PersistenceError(format!("Failed to create temp file: {}", e))
            })?;
            file.write_all(content.as_bytes()).map_err(|e| {
                DictionaryError::PersistenceError(format!("Failed to write: {}", e))
            })?;
            file.sync_all().map_err(|e| {
                DictionaryError::PersistenceError(format!("Failed to sync: {}", e))
            })?;
        } // File closed here

        // Atomic rename
        fs::rename(&temp_path, &self.config_path).map_err(|e| {
            // Clean up temp file on error
            let _ = fs::remove_file(&temp_path);
            DictionaryError::PersistenceError(format!("Failed to rename: {}", e))
        })?;

        crate::debug!("Dictionary entries persisted successfully");
        Ok(())
    }

    /// List all entries
    pub fn list(&self) -> Vec<&DictionaryEntry> {
        self.entries.values().collect()
    }

    /// Add a new entry to the store
    /// Generates a unique ID using UUID v4
    #[must_use = "this returns a Result that should be handled"]
    pub fn add(
        &mut self,
        trigger: String,
        expansion: String,
        suffix: Option<String>,
        auto_enter: bool,
        disable_suffix: bool,
    ) -> Result<DictionaryEntry, DictionaryError> {
        let id = Uuid::new_v4().to_string();
        let entry = DictionaryEntry {
            id: id.clone(),
            trigger,
            expansion,
            suffix,
            auto_enter,
            disable_suffix,
        };

        // ID collision is extremely unlikely with UUID v4, but check anyway
        if self.entries.contains_key(&id) {
            return Err(DictionaryError::DuplicateId(id));
        }

        self.entries.insert(id, entry.clone());
        self.save()?;
        Ok(entry)
    }

    /// Update an existing entry
    #[must_use = "this returns a Result that should be handled"]
    pub fn update(
        &mut self,
        id: String,
        trigger: String,
        expansion: String,
        suffix: Option<String>,
        auto_enter: bool,
        disable_suffix: bool,
    ) -> Result<DictionaryEntry, DictionaryError> {
        if !self.entries.contains_key(&id) {
            return Err(DictionaryError::NotFound(id));
        }

        let entry = DictionaryEntry {
            id: id.clone(),
            trigger,
            expansion,
            suffix,
            auto_enter,
            disable_suffix,
        };
        self.entries.insert(id, entry.clone());
        self.save()?;
        Ok(entry)
    }

    /// Delete an entry by ID
    #[must_use = "this returns a Result that should be handled"]
    pub fn delete(&mut self, id: &str) -> Result<(), DictionaryError> {
        if self.entries.remove(id).is_none() {
            return Err(DictionaryError::NotFound(id.to_string()));
        }
        self.save()?;
        Ok(())
    }

    /// Get an entry by ID
    pub fn get(&self, id: &str) -> Option<&DictionaryEntry> {
        self.entries.get(id)
    }
}

#[cfg(test)]
#[path = "store_test.rs"]
mod tests;
