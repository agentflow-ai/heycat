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
use crate::spacetimedb::client::SpacetimeClient;
use crate::voice_commands::registry::CommandDefinition;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Resolves effective commands and dictionary entries based on active context
pub struct ContextResolver {
    /// Reference to the window monitor for current context
    monitor: Arc<Mutex<WindowMonitor>>,
    /// Reference to SpacetimeDB client for window contexts
    client: Arc<Mutex<SpacetimeClient>>,
}

impl ContextResolver {
    /// Create a new context resolver
    pub fn new(
        monitor: Arc<Mutex<WindowMonitor>>,
        client: Arc<Mutex<SpacetimeClient>>,
    ) -> Self {
        Self { monitor, client }
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
        // Get current context from monitor
        let context_id = match self.monitor.lock() {
            Ok(monitor) => monitor.get_current_context(),
            Err(_) => {
                crate::warn!("[ContextResolver] Failed to lock monitor, returning global commands");
                return all_commands.to_vec();
            }
        };

        // No active context - return all global commands
        let context_id = match context_id {
            Some(id) => id,
            None => return all_commands.to_vec(),
        };

        // Get the context from SpacetimeDB
        let context = match self.client.lock() {
            Ok(client) => match client.get_window_context(context_id) {
                Ok(ctx) => ctx,
                Err(e) => {
                    crate::warn!(
                        "[ContextResolver] Failed to get context from SpacetimeDB: {}, returning global commands",
                        e
                    );
                    return all_commands.to_vec();
                }
            },
            Err(_) => {
                crate::warn!(
                    "[ContextResolver] Failed to lock SpacetimeDB client, returning global commands"
                );
                return all_commands.to_vec();
            }
        };

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

        // No active context - return all global entries
        let context_id = match context_id {
            Some(id) => id,
            None => return all_entries.to_vec(),
        };

        // Get the context from SpacetimeDB
        let context = match self.client.lock() {
            Ok(client) => match client.get_window_context(context_id) {
                Ok(ctx) => ctx,
                Err(e) => {
                    crate::warn!(
                        "[ContextResolver] Failed to get context from SpacetimeDB: {}, returning global dictionary",
                        e
                    );
                    return all_entries.to_vec();
                }
            },
            Err(_) => {
                crate::warn!(
                    "[ContextResolver] Failed to lock SpacetimeDB client, returning global dictionary"
                );
                return all_entries.to_vec();
            }
        };

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

        // Build a lookup map for entries by ID
        let entries_by_id: std::collections::HashMap<&str, &DictionaryEntry> =
            all_entries.iter().map(|e| (e.id.as_str(), e)).collect();

        // Apply override mode
        match context.dictionary_mode {
            OverrideMode::Replace => {
                // Return only context-specific entries
                context
                    .dictionary_entry_ids
                    .iter()
                    .filter_map(|id| entries_by_id.get(id.as_str()).map(|e| (*e).clone()))
                    .collect()
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

                merged
            }
        }
    }

    /// Get the currently matched context ID, if any
    pub fn get_current_context_id(&self) -> Option<Uuid> {
        self.monitor
            .lock()
            .ok()
            .and_then(|monitor| monitor.get_current_context())
    }
}

#[cfg(test)]
#[path = "resolver_test.rs"]
mod tests;
