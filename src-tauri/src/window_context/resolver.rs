// Context resolver for command and dictionary resolution
//
// Determines the effective set of commands and dictionary entries based on
// the currently active window context and its configured override mode.
//
// NOTE: This resolver is created and wired to TranscriptionService in
// transcription-integration.spec.md. The resolver is used by TranscriptionService
// to determine which commands and dictionary entries are active for the current
// window context.
// DEFERRAL: Production wiring deferred to transcription-integration.spec.md

use super::{OverrideMode, WindowContextStore, WindowMonitor};
use crate::dictionary::{DictionaryEntry, DictionaryStore};
use crate::voice_commands::registry::{CommandDefinition, CommandRegistry};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Resolves effective commands and dictionary entries based on active context
pub struct ContextResolver {
    /// Reference to the window monitor for current context
    monitor: Arc<Mutex<WindowMonitor>>,
    /// Reference to the window context store
    context_store: Arc<Mutex<WindowContextStore>>,
}

impl ContextResolver {
    /// Create a new context resolver
    pub fn new(
        monitor: Arc<Mutex<WindowMonitor>>,
        context_store: Arc<Mutex<WindowContextStore>>,
    ) -> Self {
        Self {
            monitor,
            context_store,
        }
    }

    /// Get the effective commands based on active context
    ///
    /// - No context: returns all global commands
    /// - Replace mode: returns only context-specific commands
    /// - Merge mode: returns global + context commands (context wins on conflict)
    pub fn get_effective_commands(
        &self,
        global_registry: &CommandRegistry,
    ) -> Vec<CommandDefinition> {
        // Get current context from monitor
        let context_id = match self.monitor.lock() {
            Ok(monitor) => monitor.get_current_context(),
            Err(_) => {
                crate::warn!("[ContextResolver] Failed to lock monitor, returning global commands");
                return global_registry.list().into_iter().cloned().collect();
            }
        };

        // No active context - return all global commands
        let context_id = match context_id {
            Some(id) => id,
            None => return global_registry.list().into_iter().cloned().collect(),
        };

        // Get the context from store
        let context = match self.context_store.lock() {
            Ok(store) => store.get(context_id).cloned(),
            Err(_) => {
                crate::warn!(
                    "[ContextResolver] Failed to lock context store, returning global commands"
                );
                return global_registry.list().into_iter().cloned().collect();
            }
        };

        let context = match context {
            Some(ctx) => ctx,
            None => {
                crate::warn!(
                    "[ContextResolver] Context {} not found, returning global commands",
                    context_id
                );
                return global_registry.list().into_iter().cloned().collect();
            }
        };

        // Apply override mode
        match context.command_mode {
            OverrideMode::Replace => {
                // Return only context-specific commands
                context
                    .command_ids
                    .iter()
                    .filter_map(|id| global_registry.get(*id).cloned())
                    .collect()
            }
            OverrideMode::Merge => {
                // Start with all global commands
                let mut merged: Vec<CommandDefinition> =
                    global_registry.list().into_iter().cloned().collect();

                // Get context commands and override matching triggers
                for cmd_id in &context.command_ids {
                    if let Some(cmd) = global_registry.get(*cmd_id) {
                        // Remove any command with the same trigger
                        merged.retain(|c| c.trigger.to_lowercase() != cmd.trigger.to_lowercase());
                        // Add the context command
                        merged.push(cmd.clone());
                    }
                }

                merged
            }
        }
    }

    /// Get the effective dictionary entries based on active context
    ///
    /// - No context: returns all global entries
    /// - Replace mode: returns only context-specific entries
    /// - Merge mode: returns global + context entries (context wins on conflict)
    pub fn get_effective_dictionary(
        &self,
        global_store: &DictionaryStore,
    ) -> Vec<DictionaryEntry> {
        // Get current context from monitor
        let context_id = match self.monitor.lock() {
            Ok(monitor) => monitor.get_current_context(),
            Err(_) => {
                crate::warn!(
                    "[ContextResolver] Failed to lock monitor, returning global dictionary"
                );
                return global_store.list().into_iter().cloned().collect();
            }
        };

        // No active context - return all global entries
        let context_id = match context_id {
            Some(id) => id,
            None => return global_store.list().into_iter().cloned().collect(),
        };

        // Get the context from store
        let context = match self.context_store.lock() {
            Ok(store) => store.get(context_id).cloned(),
            Err(_) => {
                crate::warn!(
                    "[ContextResolver] Failed to lock context store, returning global dictionary"
                );
                return global_store.list().into_iter().cloned().collect();
            }
        };

        let context = match context {
            Some(ctx) => ctx,
            None => {
                crate::warn!(
                    "[ContextResolver] Context {} not found, returning global dictionary",
                    context_id
                );
                return global_store.list().into_iter().cloned().collect();
            }
        };

        // Apply override mode
        match context.dictionary_mode {
            OverrideMode::Replace => {
                // Return only context-specific entries
                context
                    .dictionary_entry_ids
                    .iter()
                    .filter_map(|id| global_store.get(id).cloned())
                    .collect()
            }
            OverrideMode::Merge => {
                // Start with all global entries
                let mut merged: Vec<DictionaryEntry> =
                    global_store.list().into_iter().cloned().collect();

                // Get context entries and override matching triggers
                for entry_id in &context.dictionary_entry_ids {
                    if let Some(entry) = global_store.get(entry_id) {
                        // Remove any entry with the same trigger
                        merged.retain(|e| {
                            e.trigger.to_lowercase() != entry.trigger.to_lowercase()
                        });
                        // Add the context entry
                        merged.push(entry.clone());
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
