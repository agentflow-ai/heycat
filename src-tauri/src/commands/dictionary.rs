// Dictionary Tauri commands for CRUD operations
// Exposes DictionaryStore to the frontend and emits dictionary_updated events on mutations
//
// This file contains Tauri-specific wrappers and is excluded from coverage.
#![cfg_attr(coverage_nightly, coverage(off))]

use crate::commands::TranscriptionServiceState;
use crate::dictionary::{DictionaryEntry, DictionaryError};
use crate::turso::{events as turso_events, TursoClient};
use std::sync::Arc;
use tauri::{AppHandle, State};

/// Type alias for Turso client state
pub type TursoClientState = Arc<TursoClient>;

/// Map DictionaryError to user-friendly error messages
fn to_user_error(error: DictionaryError) -> String {
    match error {
        DictionaryError::NotFound(id) => format!("Entry with ID '{}' not found", id),
        DictionaryError::DuplicateId(id) => format!("Entry with ID '{}' already exists", id),
        DictionaryError::PersistenceError(msg) => format!("Failed to save dictionary: {}", msg),
        DictionaryError::LoadError(msg) => format!("Failed to load dictionary: {}", msg),
    }
}

/// Refresh the dictionary expander in the transcription service with current entries from Turso
async fn refresh_dictionary_expander(
    client: &TursoClient,
    transcription_service: &TranscriptionServiceState,
) {
    // Read entries from Turso (the source of truth)
    match client.list_dictionary_entries().await {
        Ok(entries) => {
            transcription_service.update_dictionary(&entries);
        }
        Err(e) => {
            crate::warn!("Failed to refresh dictionary expander from Turso: {:?}", e);
        }
    }
}

/// List all dictionary entries
///
/// Returns all entries from Turso database.
#[tauri::command]
pub async fn list_dictionary_entries(
    turso_client: State<'_, TursoClientState>,
) -> Result<Vec<DictionaryEntry>, String> {
    turso_client
        .list_dictionary_entries()
        .await
        .map_err(to_user_error)
}

/// Add a new dictionary entry
///
/// Creates a new entry with the given trigger and expansion, generates a unique ID,
/// persists to Turso, updates the transcription service expander, and emits a dictionary_updated event.
///
/// # Arguments
/// * `trigger` - The trigger word/phrase (e.g., "brb")
/// * `expansion` - The expansion text (e.g., "be right back")
/// * `suffix` - Optional suffix appended after expansion
/// * `auto_enter` - Whether to simulate enter keypress after expansion (defaults to false)
/// * `disable_suffix` - Whether to suppress trailing punctuation (defaults to false)
/// * `complete_match_only` - Whether to only expand when trigger is complete input (defaults to false)
///
/// # Returns
/// The newly created DictionaryEntry with its generated ID
#[tauri::command]
pub async fn add_dictionary_entry(
    app_handle: AppHandle,
    turso_client: State<'_, TursoClientState>,
    transcription_service: State<'_, TranscriptionServiceState>,
    trigger: String,
    expansion: String,
    suffix: Option<String>,
    auto_enter: Option<bool>,
    disable_suffix: Option<bool>,
    complete_match_only: Option<bool>,
) -> Result<DictionaryEntry, String> {
    // Validate: trigger cannot be empty
    if trigger.trim().is_empty() {
        return Err("Trigger cannot be empty".to_string());
    }

    let auto_enter_val = auto_enter.unwrap_or(false);
    let disable_suffix_val = disable_suffix.unwrap_or(false);
    let complete_match_only_val = complete_match_only.unwrap_or(false);

    // Add entry to Turso
    let entry = turso_client
        .add_dictionary_entry(
            trigger.clone(),
            expansion.clone(),
            suffix.clone(),
            auto_enter_val,
            disable_suffix_val,
            complete_match_only_val,
        )
        .await
        .map_err(to_user_error)?;

    // Refresh the dictionary expander with entries from Turso
    refresh_dictionary_expander(&turso_client, &transcription_service).await;

    // Emit dictionary_updated event
    turso_events::emit_dictionary_updated(&app_handle, "add", &entry.id);

    crate::info!(
        "Added dictionary entry: {} -> {}",
        entry.trigger,
        entry.expansion
    );
    Ok(entry)
}

/// Update an existing dictionary entry
///
/// Updates the trigger and expansion for the entry with the given ID,
/// persists to Turso, updates the transcription service expander, and emits a dictionary_updated event.
///
/// # Arguments
/// * `id` - The unique ID of the entry to update
/// * `trigger` - The new trigger word/phrase
/// * `expansion` - The new expansion text
/// * `suffix` - Optional suffix appended after expansion
/// * `auto_enter` - Whether to simulate enter keypress after expansion (defaults to false)
/// * `disable_suffix` - Whether to suppress trailing punctuation (defaults to false)
/// * `complete_match_only` - Whether to only expand when trigger is complete input (defaults to false)
#[tauri::command]
pub async fn update_dictionary_entry(
    app_handle: AppHandle,
    turso_client: State<'_, TursoClientState>,
    transcription_service: State<'_, TranscriptionServiceState>,
    id: String,
    trigger: String,
    expansion: String,
    suffix: Option<String>,
    auto_enter: Option<bool>,
    disable_suffix: Option<bool>,
    complete_match_only: Option<bool>,
) -> Result<(), String> {
    // Validate: trigger cannot be empty
    if trigger.trim().is_empty() {
        return Err("Trigger cannot be empty".to_string());
    }

    let auto_enter_val = auto_enter.unwrap_or(false);
    let disable_suffix_val = disable_suffix.unwrap_or(false);
    let complete_match_only_val = complete_match_only.unwrap_or(false);

    // Update entry in Turso
    turso_client
        .update_dictionary_entry(
            id.clone(),
            trigger.clone(),
            expansion.clone(),
            suffix.clone(),
            auto_enter_val,
            disable_suffix_val,
            complete_match_only_val,
        )
        .await
        .map_err(to_user_error)?;

    // Refresh the dictionary expander with entries from Turso
    refresh_dictionary_expander(&turso_client, &transcription_service).await;

    // Emit dictionary_updated event
    turso_events::emit_dictionary_updated(&app_handle, "update", &id);

    crate::info!("Updated dictionary entry: {}", id);
    Ok(())
}

/// Delete a dictionary entry
///
/// Removes the entry with the given ID, persists to Turso,
/// updates the transcription service expander, and emits a dictionary_updated event.
///
/// # Arguments
/// * `id` - The unique ID of the entry to delete
#[tauri::command]
pub async fn delete_dictionary_entry(
    app_handle: AppHandle,
    turso_client: State<'_, TursoClientState>,
    transcription_service: State<'_, TranscriptionServiceState>,
    id: String,
) -> Result<(), String> {
    // Delete entry from Turso
    turso_client
        .delete_dictionary_entry(&id)
        .await
        .map_err(to_user_error)?;

    // Refresh the dictionary expander with entries from Turso
    refresh_dictionary_expander(&turso_client, &transcription_service).await;

    // Emit dictionary_updated event
    turso_events::emit_dictionary_updated(&app_handle, "delete", &id);

    crate::info!("Deleted dictionary entry: {}", id);
    Ok(())
}

#[cfg(test)]
#[path = "dictionary_test.rs"]
mod tests;
