// Window context store - persists and loads window context definitions
// Follows the same pattern as dictionary/store.rs for file-based persistence

use super::types::{ActiveWindowInfo, OverrideMode, WindowContext, WindowMatcher};
use regex::Regex;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

/// Error types for window context store operations
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum WindowContextStoreError {
    /// Context with this ID already exists
    #[error("Context with ID {0} already exists")]
    DuplicateId(Uuid),
    /// Context not found
    #[error("Context with ID {0} not found")]
    NotFound(Uuid),
    /// Invalid regex pattern
    #[error("Invalid title pattern regex: {0}")]
    InvalidPattern(String),
    /// Failed to persist contexts
    #[error("Failed to persist contexts: {0}")]
    PersistenceError(String),
    /// Failed to load contexts
    #[error("Failed to load contexts: {0}")]
    LoadError(String),
}

/// Store for window context definitions with file-based persistence
#[derive(Debug)]
pub struct WindowContextStore {
    /// Contexts indexed by ID
    contexts: HashMap<Uuid, WindowContext>,
    /// Path to persistence file
    config_path: PathBuf,
}

impl WindowContextStore {
    /// Create a new store with the given config path
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            contexts: HashMap::new(),
            config_path,
        }
    }

    /// Create a store using the default config path with worktree context
    pub fn with_default_path_context(
        worktree_context: Option<&crate::worktree::WorktreeContext>,
    ) -> Result<Self, WindowContextStoreError> {
        let config_dir = crate::paths::get_config_dir(worktree_context).map_err(|e| {
            WindowContextStoreError::LoadError(format!("Could not determine config directory: {}", e))
        })?;
        let config_path = config_dir.join("window_contexts.json");
        Ok(Self::new(config_path))
    }

    /// Load contexts from the persistence file
    pub fn load(&mut self) -> Result<(), WindowContextStoreError> {
        crate::debug!("Loading window contexts from {:?}", self.config_path);

        if !self.config_path.exists() {
            crate::debug!("No window contexts file found, starting with empty store");
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;

        let contexts: Vec<WindowContext> = serde_json::from_str(&content)
            .map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;

        self.contexts.clear();
        for context in contexts {
            self.contexts.insert(context.id, context);
        }

        crate::info!("Loaded {} window contexts", self.contexts.len());
        Ok(())
    }

    /// Persist contexts to the file using atomic write (temp file + rename)
    fn save(&self) -> Result<(), WindowContextStoreError> {
        crate::debug!(
            "Persisting {} window contexts to {:?}",
            self.contexts.len(),
            self.config_path
        );

        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;
        }

        let contexts: Vec<&WindowContext> = self.contexts.values().collect();
        let content = serde_json::to_string_pretty(&contexts)
            .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;

        // Use atomic temp file + rename pattern
        let temp_path = self.config_path.with_extension("tmp");

        // Write to temp file with explicit sync
        {
            let mut file = File::create(&temp_path).map_err(|e| {
                WindowContextStoreError::PersistenceError(format!("Failed to create temp file: {}", e))
            })?;
            file.write_all(content.as_bytes()).map_err(|e| {
                WindowContextStoreError::PersistenceError(format!("Failed to write: {}", e))
            })?;
            file.sync_all().map_err(|e| {
                WindowContextStoreError::PersistenceError(format!("Failed to sync: {}", e))
            })?;
        } // File closed here

        // Atomic rename
        fs::rename(&temp_path, &self.config_path).map_err(|e| {
            // Clean up temp file on error
            let _ = fs::remove_file(&temp_path);
            WindowContextStoreError::PersistenceError(format!("Failed to rename: {}", e))
        })?;

        crate::debug!("Window contexts persisted successfully");
        Ok(())
    }

    /// Validate the title_pattern regex if present
    fn validate_pattern(title_pattern: &Option<String>) -> Result<(), WindowContextStoreError> {
        if let Some(pattern) = title_pattern {
            Regex::new(pattern).map_err(|e| {
                WindowContextStoreError::InvalidPattern(format!("{}: {}", pattern, e))
            })?;
        }
        Ok(())
    }

    /// List all contexts
    pub fn list(&self) -> Vec<&WindowContext> {
        self.contexts.values().collect()
    }

    /// Get a context by ID
    pub fn get(&self, id: Uuid) -> Option<&WindowContext> {
        self.contexts.get(&id)
    }

    /// Add a new context to the store
    /// Generates a unique ID using UUID v4
    #[must_use = "this returns a Result that should be handled"]
    pub fn add(
        &mut self,
        name: String,
        matcher: WindowMatcher,
        command_mode: OverrideMode,
        dictionary_mode: OverrideMode,
        command_ids: Vec<Uuid>,
        dictionary_entry_ids: Vec<String>,
        enabled: bool,
        priority: i32,
    ) -> Result<WindowContext, WindowContextStoreError> {
        // Validate title_pattern regex
        Self::validate_pattern(&matcher.title_pattern)?;

        let id = Uuid::new_v4();
        let context = WindowContext {
            id,
            name,
            matcher,
            command_mode,
            dictionary_mode,
            command_ids,
            dictionary_entry_ids,
            enabled,
            priority,
        };

        // ID collision is extremely unlikely with UUID v4, but check anyway
        if self.contexts.contains_key(&id) {
            return Err(WindowContextStoreError::DuplicateId(id));
        }

        self.contexts.insert(id, context.clone());
        self.save()?;
        Ok(context)
    }

    /// Update an existing context
    #[must_use = "this returns a Result that should be handled"]
    pub fn update(&mut self, context: WindowContext) -> Result<WindowContext, WindowContextStoreError> {
        if !self.contexts.contains_key(&context.id) {
            return Err(WindowContextStoreError::NotFound(context.id));
        }

        // Validate title_pattern regex
        Self::validate_pattern(&context.matcher.title_pattern)?;

        self.contexts.insert(context.id, context.clone());
        self.save()?;
        Ok(context)
    }

    /// Delete a context by ID
    #[must_use = "this returns a Result that should be handled"]
    pub fn delete(&mut self, id: Uuid) -> Result<(), WindowContextStoreError> {
        if self.contexts.remove(&id).is_none() {
            return Err(WindowContextStoreError::NotFound(id));
        }
        self.save()?;
        Ok(())
    }

    /// Find the highest-priority matching context for a window
    ///
    /// Matching rules:
    /// 1. App name must match (case-insensitive)
    /// 2. If title_pattern is set, it must match window_title (regex)
    /// 3. Only enabled contexts are considered
    /// 4. Returns the highest priority match
    pub fn find_matching_context(&self, window: &ActiveWindowInfo) -> Option<&WindowContext> {
        self.contexts
            .values()
            .filter(|ctx| ctx.enabled)
            .filter(|ctx| {
                // Case-insensitive app name match
                ctx.matcher.app_name.to_lowercase() == window.app_name.to_lowercase()
            })
            .filter(|ctx| {
                // Check title pattern if present
                match (&ctx.matcher.title_pattern, &window.window_title) {
                    (Some(pattern), Some(title)) => {
                        // Regex::new can't fail here as we validate on add/update
                        Regex::new(pattern)
                            .map(|re| re.is_match(title))
                            .unwrap_or(false)
                    }
                    (Some(_), None) => false, // Pattern requires title but window has none
                    (None, _) => true,         // No pattern required
                }
            })
            .max_by_key(|ctx| ctx.priority)
    }
}

#[cfg(test)]
#[path = "store_test.rs"]
mod tests;
