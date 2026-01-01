// Context resolver for command and dictionary resolution
//
// Determines the effective set of commands and dictionary entries based on
// the currently active window context and its configured override mode.
//
// NOTE: This resolver is created and wired to TranscriptionService in
// transcription-integration.spec.md. The resolver is used by TranscriptionService
// to determine which commands and dictionary entries are active for the current
// window context.

use super::{OverrideMode, WindowMonitor};
use crate::dictionary::DictionaryEntry;
use crate::turso::TursoClient;
use crate::voice_commands::registry::CommandDefinition;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Resolves effective commands and dictionary entries based on active context
pub struct ContextResolver {
    /// Reference to the window monitor for current context
    monitor: Arc<Mutex<WindowMonitor>>,
    /// Reference to Turso client for window contexts
    client: Arc<TursoClient>,
}

impl ContextResolver {
    /// Create a new context resolver
    pub fn new(
        monitor: Arc<Mutex<WindowMonitor>>,
        client: Arc<TursoClient>,
    ) -> Self {
        Self { monitor, client }
    }

    /// Helper to run async operations in sync context
    fn run_async<F, T>(&self, future: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        crate::util::run_async(future)
    }

    /// Get the effective commands based on active context
    ///
    /// Takes all available commands and filters/merges based on context.
    /// - No context: returns all global commands
    /// - Replace mode: returns only context-specific commands
    /// - Merge mode: returns global + context commands (context wins on conflict)
    pub fn get_effective_commands(
        &self,
        all_commands: &[CommandDefinition],
    ) -> Vec<CommandDefinition> {
        crate::debug!(
            "[ContextResolver] get_effective_commands called with {} commands",
            all_commands.len()
        );

        // Get current context from monitor
        let context_id = match self.monitor.lock() {
            Ok(monitor) => monitor.get_current_context(),
            Err(_) => {
                crate::warn!("[ContextResolver] Failed to lock monitor, returning global commands");
                return all_commands.to_vec();
            }
        };

        // No active context - return only truly global commands (not assigned to any context)
        let context_id = match context_id {
            Some(id) => {
                crate::debug!("[ContextResolver] Active context ID for commands: {}", id);
                id
            }
            None => {
                // Get all contexts to find which commands are assigned somewhere
                let assigned_command_ids: std::collections::HashSet<Uuid> = self.run_async(async {
                    match self.client.list_window_contexts().await {
                        Ok(contexts) => contexts
                            .iter()
                            .flat_map(|ctx| ctx.command_ids.iter().cloned())
                            .collect(),
                        Err(_) => std::collections::HashSet::new(),
                    }
                });

                // Return only commands not assigned to any context
                let global_commands: Vec<CommandDefinition> = all_commands
                    .iter()
                    .filter(|c| !assigned_command_ids.contains(&c.id))
                    .cloned()
                    .collect();

                crate::debug!(
                    "[ContextResolver] No active context, returning {} global commands (filtered from {})",
                    global_commands.len(),
                    all_commands.len()
                );
                return global_commands;
            }
        };

        // Get the context from Turso
        let context = self.run_async(async {
            match self.client.get_window_context(context_id).await {
                Ok(ctx) => ctx,
                Err(e) => {
                    crate::warn!(
                        "[ContextResolver] Failed to get context from Turso: {}, returning global commands",
                        e
                    );
                    None
                }
            }
        });

        let context = match context {
            Some(ctx) => ctx,
            None => {
                crate::warn!(
                    "[ContextResolver] Context {} not found, returning global commands",
                    context_id
                );
                return all_commands.to_vec();
            }
        };

        // Build a lookup map for commands by ID
        let commands_by_id: std::collections::HashMap<Uuid, &CommandDefinition> =
            all_commands.iter().map(|c| (c.id, c)).collect();

        // Apply override mode
        match context.command_mode {
            OverrideMode::Replace => {
                // Return only context-specific commands
                context
                    .command_ids
                    .iter()
                    .filter_map(|id| commands_by_id.get(id).map(|c| (*c).clone()))
                    .collect()
            }
            OverrideMode::Merge => {
                // Start with all global commands
                let mut merged: Vec<CommandDefinition> = all_commands.to_vec();

                // Get context commands and override matching triggers
                for cmd_id in &context.command_ids {
                    if let Some(cmd) = commands_by_id.get(cmd_id) {
                        // Remove any command with the same trigger
                        merged.retain(|c| c.trigger.to_lowercase() != cmd.trigger.to_lowercase());
                        // Add the context command
                        merged.push((*cmd).clone());
                    }
                }

                merged
            }
        }
    }

    /// Get the effective dictionary entries based on active context
    ///
    /// Takes all available dictionary entries and filters/merges based on context.
    /// - No context: returns all global entries
    /// - Replace mode: returns only context-specific entries
    /// - Merge mode: returns global + context entries (context wins on conflict)
    pub fn get_effective_dictionary(
        &self,
        all_entries: &[DictionaryEntry],
    ) -> Vec<DictionaryEntry> {
        crate::debug!(
            "[ContextResolver] get_effective_dictionary called with {} entries",
            all_entries.len()
        );

        // Get current context from monitor
        let context_id = match self.monitor.lock() {
            Ok(monitor) => monitor.get_current_context(),
            Err(_) => {
                crate::warn!(
                    "[ContextResolver] Failed to lock monitor, returning global dictionary"
                );
                return all_entries.to_vec();
            }
        };

        // No active context - return only truly global entries (not assigned to any context)
        let context_id = match context_id {
            Some(id) => {
                crate::debug!("[ContextResolver] Active context ID: {}", id);
                id
            }
            None => {
                // Get all contexts to find which entries are assigned somewhere
                let assigned_entry_ids: std::collections::HashSet<String> = self.run_async(async {
                    match self.client.list_window_contexts().await {
                        Ok(contexts) => contexts
                            .iter()
                            .flat_map(|ctx| ctx.dictionary_entry_ids.iter().cloned())
                            .collect(),
                        Err(_) => std::collections::HashSet::new(),
                    }
                });

                // Return only entries not assigned to any context
                let global_entries: Vec<DictionaryEntry> = all_entries
                    .iter()
                    .filter(|e| !assigned_entry_ids.contains(&e.id))
                    .cloned()
                    .collect();

                crate::debug!(
                    "[ContextResolver] No active context, returning {} global entries (filtered from {})",
                    global_entries.len(),
                    all_entries.len()
                );
                return global_entries;
            }
        };

        // Get the context from Turso
        let context = self.run_async(async {
            match self.client.get_window_context(context_id).await {
                Ok(ctx) => ctx,
                Err(e) => {
                    crate::warn!(
                        "[ContextResolver] Failed to get context from Turso: {}, returning global dictionary",
                        e
                    );
                    None
                }
            }
        });

        let context = match context {
            Some(ctx) => ctx,
            None => {
                crate::warn!(
                    "[ContextResolver] Context {} not found, returning global dictionary",
                    context_id
                );
                return all_entries.to_vec();
            }
        };

        crate::debug!(
            "[ContextResolver] Context '{}' found with dictionary_mode={:?}, {} assigned entries",
            context.name,
            context.dictionary_mode,
            context.dictionary_entry_ids.len()
        );

        // Build a lookup map for entries by ID
        let entries_by_id: std::collections::HashMap<&str, &DictionaryEntry> =
            all_entries.iter().map(|e| (e.id.as_str(), e)).collect();

        // Apply override mode
        let result = match context.dictionary_mode {
            OverrideMode::Replace => {
                // Return only context-specific entries
                let entries: Vec<DictionaryEntry> = context
                    .dictionary_entry_ids
                    .iter()
                    .filter_map(|id| entries_by_id.get(id.as_str()).map(|e| (*e).clone()))
                    .collect();
                crate::debug!(
                    "[ContextResolver] Replace mode: returning {} context-specific entries",
                    entries.len()
                );
                entries
            }
            OverrideMode::Merge => {
                // Start with all global entries
                let mut merged: Vec<DictionaryEntry> = all_entries.to_vec();

                // Get context entries and override matching triggers
                for entry_id in &context.dictionary_entry_ids {
                    if let Some(entry) = entries_by_id.get(entry_id.as_str()) {
                        // Remove any entry with the same trigger
                        merged.retain(|e| {
                            e.trigger.to_lowercase() != entry.trigger.to_lowercase()
                        });
                        // Add the context entry
                        merged.push((*entry).clone());
                    }
                }

                crate::debug!(
                    "[ContextResolver] Merge mode: returning {} merged entries",
                    merged.len()
                );
                merged
            }
        };

        result
    }
}

#[cfg(test)]
#[path = "resolver_test.rs"]
mod tests;
