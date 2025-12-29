// Dictionary Tauri commands for CRUD operations
// Exposes DictionaryStore to the frontend and emits dictionary_updated events on mutations
//
// This file contains Tauri-specific wrappers and is excluded from coverage.
#![cfg_attr(coverage_nightly, coverage(off))]

use crate::commands::TranscriptionServiceState;
use crate::dictionary::{DictionaryEntry, DictionaryError};
use crate::events::dictionary_events::{self, DictionaryUpdatedPayload};
use crate::spacetimedb::SpacetimeClient;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

/// Type alias for SpacetimeDB client state (required - no fallback)
pub type SpacetimeClientState = Arc<Mutex<SpacetimeClient>>;

/// Helper macro to emit events with error logging
macro_rules! emit_or_warn {
    ($handle:expr, $event:expr, $payload:expr) => {
        if let Err(e) = $handle.emit($event, $payload) {
            crate::warn!("Failed to emit event '{}': {}", $event, e);
        }
    };
}

/// Map DictionaryError to user-friendly error messages
fn to_user_error(error: DictionaryError) -> String {
    match error {
        DictionaryError::NotFound(id) => format!("Entry with ID '{}' not found", id),
        DictionaryError::DuplicateId(id) => format!("Entry with ID '{}' already exists", id),
        DictionaryError::PersistenceError(msg) => format!("Failed to save dictionary: {}", msg),
        DictionaryError::LoadError(msg) => format!("Failed to load dictionary: {}", msg),
    }
}

/// Refresh the dictionary expander in the transcription service with current entries from SpacetimeDB
fn refresh_dictionary_expander(
    client: &SpacetimeClient,
    transcription_service: &TranscriptionServiceState,
) {
    // Read entries from SpacetimeDB (the source of truth)
    match client.list_dictionary_entries() {
        Ok(entries) => {
            transcription_service.update_dictionary(&entries);
        }
        Err(e) => {
            crate::warn!("Failed to refresh dictionary expander from SpacetimeDB: {:?}", e);
        }
    }
}

/// List all dictionary entries
///
/// Returns all entries from SpacetimeDB.
#[tauri::command]
pub fn list_dictionary_entries(
    spacetimedb_client: State<'_, SpacetimeClientState>,
) -> Result<Vec<DictionaryEntry>, String> {
    let client = spacetimedb_client
        .lock()
        .map_err(|_| "Failed to access SpacetimeDB client".to_string())?;
    client.list_dictionary_entries().map_err(to_user_error)
}

/// Add a new dictionary entry
///
/// Creates a new entry with the given trigger and expansion, generates a unique ID,
/// persists to SpacetimeDB, updates the transcription service expander, and emits a dictionary_updated event.
///
/// # Arguments
/// * `trigger` - The trigger word/phrase (e.g., "brb")
/// * `expansion` - The expansion text (e.g., "be right back")
/// * `suffix` - Optional suffix appended after expansion
/// * `auto_enter` - Whether to simulate enter keypress after expansion (defaults to false)
/// * `disable_suffix` - Whether to suppress trailing punctuation (defaults to false)
///
/// # Returns
/// The newly created DictionaryEntry with its generated ID
#[tauri::command]
pub fn add_dictionary_entry(
    app_handle: AppHandle,
    spacetimedb_client: State<'_, SpacetimeClientState>,
    transcription_service: State<'_, TranscriptionServiceState>,
    trigger: String,
    expansion: String,
    suffix: Option<String>,
    auto_enter: Option<bool>,
    disable_suffix: Option<bool>,
) -> Result<DictionaryEntry, String> {
    // Validate: trigger cannot be empty
    if trigger.trim().is_empty() {
        return Err("Trigger cannot be empty".to_string());
    }

    let auto_enter_val = auto_enter.unwrap_or(false);
    let disable_suffix_val = disable_suffix.unwrap_or(false);

    // Add entry to SpacetimeDB
    let client = spacetimedb_client
        .lock()
        .map_err(|_| "Failed to access SpacetimeDB client".to_string())?;

    let entry = client
        .add_dictionary_entry(
            trigger.clone(),
            expansion.clone(),
            suffix.clone(),
            auto_enter_val,
            disable_suffix_val,
        )
        .map_err(to_user_error)?;

    // Refresh the dictionary expander with entries from SpacetimeDB
    refresh_dictionary_expander(&client, &transcription_service);

    // Emit dictionary_updated event
    emit_or_warn!(
        app_handle,
        dictionary_events::DICTIONARY_UPDATED,
        DictionaryUpdatedPayload {
            action: "add".to_string(),
            entry_id: entry.id.clone(),
        }
    );

    crate::info!("Added dictionary entry: {} -> {}", entry.trigger, entry.expansion);
    Ok(entry)
}

/// Update an existing dictionary entry
///
/// Updates the trigger and expansion for the entry with the given ID,
/// persists to SpacetimeDB, updates the transcription service expander, and emits a dictionary_updated event.
///
/// # Arguments
/// * `id` - The unique ID of the entry to update
/// * `trigger` - The new trigger word/phrase
/// * `expansion` - The new expansion text
/// * `suffix` - Optional suffix appended after expansion
/// * `auto_enter` - Whether to simulate enter keypress after expansion (defaults to false)
/// * `disable_suffix` - Whether to suppress trailing punctuation (defaults to false)
#[tauri::command]
pub fn update_dictionary_entry(
    app_handle: AppHandle,
    spacetimedb_client: State<'_, SpacetimeClientState>,
    transcription_service: State<'_, TranscriptionServiceState>,
    id: String,
    trigger: String,
    expansion: String,
    suffix: Option<String>,
    auto_enter: Option<bool>,
    disable_suffix: Option<bool>,
) -> Result<(), String> {
    // Validate: trigger cannot be empty
    if trigger.trim().is_empty() {
        return Err("Trigger cannot be empty".to_string());
    }

    let auto_enter_val = auto_enter.unwrap_or(false);
    let disable_suffix_val = disable_suffix.unwrap_or(false);

    // Update entry in SpacetimeDB
    let client = spacetimedb_client
        .lock()
        .map_err(|_| "Failed to access SpacetimeDB client".to_string())?;

    client
        .update_dictionary_entry(
            id.clone(),
            trigger.clone(),
            expansion.clone(),
            suffix.clone(),
            auto_enter_val,
            disable_suffix_val,
        )
        .map_err(to_user_error)?;

    // Refresh the dictionary expander with entries from SpacetimeDB
    refresh_dictionary_expander(&client, &transcription_service);

    // Emit dictionary_updated event
    emit_or_warn!(
        app_handle,
        dictionary_events::DICTIONARY_UPDATED,
        DictionaryUpdatedPayload {
            action: "update".to_string(),
            entry_id: id.clone(),
        }
    );

    crate::info!("Updated dictionary entry: {}", id);
    Ok(())
}

/// Delete a dictionary entry
///
/// Removes the entry with the given ID, persists to SpacetimeDB,
/// updates the transcription service expander, and emits a dictionary_updated event.
///
/// # Arguments
/// * `id` - The unique ID of the entry to delete
#[tauri::command]
pub fn delete_dictionary_entry(
    app_handle: AppHandle,
    spacetimedb_client: State<'_, SpacetimeClientState>,
    transcription_service: State<'_, TranscriptionServiceState>,
    id: String,
) -> Result<(), String> {
    // Delete entry from SpacetimeDB
    let client = spacetimedb_client
        .lock()
        .map_err(|_| "Failed to access SpacetimeDB client".to_string())?;

    client.delete_dictionary_entry(&id).map_err(to_user_error)?;

    // Refresh the dictionary expander with entries from SpacetimeDB
    refresh_dictionary_expander(&client, &transcription_service);

    // Emit dictionary_updated event
    emit_or_warn!(
        app_handle,
        dictionary_events::DICTIONARY_UPDATED,
        DictionaryUpdatedPayload {
            action: "delete".to_string(),
            entry_id: id.clone(),
        }
    );

    crate::info!("Deleted dictionary entry: {}", id);
    Ok(())
}

#[cfg(test)]
mod tests;
