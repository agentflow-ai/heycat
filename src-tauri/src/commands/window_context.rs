// Window context Tauri commands
// Exposes window context functionality to the frontend
//
// This file contains Tauri-specific wrappers and is excluded from coverage.
#![cfg_attr(coverage_nightly, coverage(off))]

use crate::events::window_context_events::{self, WindowContextsUpdatedPayload};
use crate::window_context::{
    get_active_window, get_running_applications, ActiveWindowInfo, OverrideMode,
    RunningApplication, WindowContext, WindowContextStore, WindowContextStoreError, WindowMatcher,
};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

/// Type alias for window context store state
pub type WindowContextStoreState = Arc<Mutex<WindowContextStore>>;

/// Helper macro to emit events with error logging
macro_rules! emit_or_warn {
    ($handle:expr, $event:expr, $payload:expr) => {
        if let Err(e) = $handle.emit($event, $payload) {
            crate::warn!("Failed to emit event '{}': {}", $event, e);
        }
    };
}

/// Map WindowContextStoreError to user-friendly error messages
fn to_user_error(error: WindowContextStoreError) -> String {
    match error {
        WindowContextStoreError::NotFound(id) => format!("Context with ID '{}' not found", id),
        WindowContextStoreError::DuplicateId(id) => {
            format!("Context with ID '{}' already exists", id)
        }
        WindowContextStoreError::InvalidPattern(msg) => format!("Invalid pattern: {}", msg),
        WindowContextStoreError::PersistenceError(msg) => format!("Failed to save contexts: {}", msg),
        WindowContextStoreError::LoadError(msg) => format!("Failed to load contexts: {}", msg),
    }
}

/// Get information about the currently active window
///
/// Returns the frontmost application's name, bundle ID, window title, and process ID.
/// Useful for testing window detection and context matching.
#[tauri::command]
pub fn get_active_window_info() -> Result<ActiveWindowInfo, String> {
    get_active_window()
}

/// List all running user-visible applications
///
/// Returns applications that have a user interface (activationPolicy == .regular).
/// Background helpers, agents, and daemons are filtered out.
/// Results are sorted alphabetically by application name.
#[tauri::command]
pub fn list_running_applications() -> Vec<RunningApplication> {
    get_running_applications()
}

/// List all window contexts
///
/// Returns all contexts from the store.
#[tauri::command]
pub fn list_window_contexts(
    store: State<'_, WindowContextStoreState>,
) -> Result<Vec<WindowContext>, String> {
    let store = store
        .lock()
        .map_err(|_| "Failed to access window context store".to_string())?;
    Ok(store.list().into_iter().cloned().collect())
}

/// Add a new window context
///
/// Creates a new context with the given parameters, generates a unique ID,
/// persists to storage, and emits a window_contexts_updated event.
#[tauri::command]
pub fn add_window_context(
    app_handle: AppHandle,
    store: State<'_, WindowContextStoreState>,
    name: String,
    app_name: String,
    title_pattern: Option<String>,
    bundle_id: Option<String>,
    command_mode: Option<String>,
    dictionary_mode: Option<String>,
    command_ids: Option<Vec<String>>,
    dictionary_entry_ids: Option<Vec<String>>,
    enabled: Option<bool>,
    priority: Option<i32>,
) -> Result<WindowContext, String> {
    // Validate: name cannot be empty
    if name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    // Validate: app_name cannot be empty
    if app_name.trim().is_empty() {
        return Err("App name cannot be empty".to_string());
    }

    let matcher = WindowMatcher {
        app_name,
        title_pattern,
        bundle_id,
    };

    let command_mode = parse_override_mode(command_mode.as_deref());
    let dictionary_mode = parse_override_mode(dictionary_mode.as_deref());

    let command_ids = command_ids
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

    let mut store = store
        .lock()
        .map_err(|_| "Failed to access window context store".to_string())?;
    let context = store
        .add(
            name,
            matcher,
            command_mode,
            dictionary_mode,
            command_ids,
            dictionary_entry_ids.unwrap_or_default(),
            enabled.unwrap_or(true),
            priority.unwrap_or(0),
        )
        .map_err(to_user_error)?;

    // Emit window_contexts_updated event
    emit_or_warn!(
        app_handle,
        window_context_events::WINDOW_CONTEXTS_UPDATED,
        WindowContextsUpdatedPayload {
            action: "add".to_string(),
            context_id: context.id.to_string(),
        }
    );

    crate::info!("Added window context: {} ({})", context.name, context.id);
    Ok(context)
}

/// Update an existing window context
///
/// Updates the context with the given ID, persists to storage,
/// and emits a window_contexts_updated event.
#[tauri::command]
pub fn update_window_context(
    app_handle: AppHandle,
    store: State<'_, WindowContextStoreState>,
    id: String,
    name: String,
    app_name: String,
    title_pattern: Option<String>,
    bundle_id: Option<String>,
    command_mode: Option<String>,
    dictionary_mode: Option<String>,
    command_ids: Option<Vec<String>>,
    dictionary_entry_ids: Option<Vec<String>>,
    enabled: Option<bool>,
    priority: Option<i32>,
) -> Result<(), String> {
    // Validate: name cannot be empty
    if name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    // Validate: app_name cannot be empty
    if app_name.trim().is_empty() {
        return Err("App name cannot be empty".to_string());
    }

    let uuid = Uuid::parse_str(&id).map_err(|_| format!("Invalid UUID: {}", id))?;

    let matcher = WindowMatcher {
        app_name,
        title_pattern,
        bundle_id,
    };

    let command_mode = parse_override_mode(command_mode.as_deref());
    let dictionary_mode = parse_override_mode(dictionary_mode.as_deref());

    let command_ids = command_ids
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

    let context = WindowContext {
        id: uuid,
        name,
        matcher,
        command_mode,
        dictionary_mode,
        command_ids,
        dictionary_entry_ids: dictionary_entry_ids.unwrap_or_default(),
        enabled: enabled.unwrap_or(true),
        priority: priority.unwrap_or(0),
    };

    let mut store = store
        .lock()
        .map_err(|_| "Failed to access window context store".to_string())?;
    store.update(context).map_err(to_user_error)?;

    // Emit window_contexts_updated event
    emit_or_warn!(
        app_handle,
        window_context_events::WINDOW_CONTEXTS_UPDATED,
        WindowContextsUpdatedPayload {
            action: "update".to_string(),
            context_id: id.clone(),
        }
    );

    crate::info!("Updated window context: {}", id);
    Ok(())
}

/// Delete a window context
///
/// Removes the context with the given ID, persists to storage,
/// and emits a window_contexts_updated event.
#[tauri::command]
pub fn delete_window_context(
    app_handle: AppHandle,
    store: State<'_, WindowContextStoreState>,
    id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|_| format!("Invalid UUID: {}", id))?;

    let mut store = store
        .lock()
        .map_err(|_| "Failed to access window context store".to_string())?;
    store.delete(uuid).map_err(to_user_error)?;

    // Emit window_contexts_updated event
    emit_or_warn!(
        app_handle,
        window_context_events::WINDOW_CONTEXTS_UPDATED,
        WindowContextsUpdatedPayload {
            action: "delete".to_string(),
            context_id: id.clone(),
        }
    );

    crate::info!("Deleted window context: {}", id);
    Ok(())
}

/// Parse override mode from string
fn parse_override_mode(mode: Option<&str>) -> OverrideMode {
    match mode {
        Some("replace") => OverrideMode::Replace,
        _ => OverrideMode::Merge,
    }
}
