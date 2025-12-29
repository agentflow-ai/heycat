//! SpacetimeDB SDK client integration
//!
//! This module manages the connection to the local SpacetimeDB sidecar using
//! the SpacetimeDB Rust SDK. It handles:
//! - Connecting to the WebSocket endpoint
//! - Setting up table subscriptions
//! - Routing subscription callbacks to the SubscriptionHandler
//!
//! ## Architecture
//!
//! ```text
//! SpacetimeDB Sidecar (ws://127.0.0.1:3055)
//!        ↓
//! DbConnection (SDK client)
//!        ↓
//! Table Subscriptions (SELECT * FROM ...)
//!        ↓
//! on_insert/on_update/on_delete callbacks
//!        ↓
//! SubscriptionHandler
//!        ↓
//! Tauri Events → Event Bridge
//! ```

use std::sync::Arc;
use std::thread;

use spacetimedb_sdk::{DbContext, Table, TableWithPrimaryKey};
use tauri::AppHandle;
use thiserror::Error;

use super::sidecar::SidecarConfig;
use super::subscriptions::{SubscriptionHandler, TauriSubscriptionEmitter};

pub mod module_bindings;

use module_bindings::{
    DbConnection, DictionaryEntryTableAccess, RecordingTableAccess, SubscriptionEventContext,
    TranscriptionTableAccess, VoiceCommandTableAccess, WindowContextTableAccess,
};

/// Errors that can occur during client operations
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Failed to connect to SpacetimeDB: {0}")]
    ConnectionFailed(String),

    #[error("Subscription failed: {0}")]
    SubscriptionFailed(String),

    #[error("Client not initialized")]
    NotInitialized,
}

/// SpacetimeDB client connection state
#[derive(Debug, Clone)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Connecting to the sidecar
    Connecting,
    /// Connected and subscriptions active
    Connected,
    /// Connection failed
    Failed(String),
}

/// Manages the SpacetimeDB SDK client connection
///
/// This struct wraps the SDK's DbConnection and handles:
/// - Connection lifecycle
/// - Subscription setup
/// - Event routing to SubscriptionHandler
pub struct SpacetimeClient {
    /// Configuration for connecting to the sidecar
    config: SidecarConfig,
    /// Current connection state
    state: ConnectionState,
    /// Subscription handler for emitting events
    handler: Arc<SubscriptionHandler<TauriSubscriptionEmitter>>,
    /// The SDK connection (once established)
    connection: Option<DbConnection>,
    /// Background thread handle
    _thread_handle: Option<thread::JoinHandle<()>>,
}

impl SpacetimeClient {
    /// Create a new SpacetimeDB client
    ///
    /// # Arguments
    /// * `config` - Sidecar configuration with connection details
    /// * `app_handle` - Tauri AppHandle for event emission
    pub fn new(config: SidecarConfig, app_handle: AppHandle) -> Self {
        let emitter = TauriSubscriptionEmitter::new(app_handle);
        let handler = Arc::new(SubscriptionHandler::new(emitter));

        Self {
            config,
            state: ConnectionState::Disconnected,
            handler,
            connection: None,
            _thread_handle: None,
        }
    }

    /// Get the WebSocket URL for connecting
    pub fn websocket_url(&self) -> String {
        self.config.websocket_url()
    }

    /// Check if currently connected
    pub fn is_connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }

    /// Get the current connection state
    pub fn state(&self) -> &ConnectionState {
        &self.state
    }

    /// Connect to the SpacetimeDB sidecar and set up subscriptions
    ///
    /// This method:
    /// 1. Creates a DbConnection to the sidecar
    /// 2. Subscribes to all relevant tables
    /// 3. Registers on_insert/on_update/on_delete callbacks
    /// 4. Starts the message processing thread
    ///
    /// # Returns
    /// * `Ok(())` on successful connection
    /// * `Err(ClientError)` if connection or subscription fails
    pub fn connect(&mut self) -> Result<(), ClientError> {
        self.state = ConnectionState::Connecting;
        let handler = self.handler.clone();
        let ws_url = self.config.websocket_url();

        crate::info!("Connecting to SpacetimeDB at {}", ws_url);

        // Clone handler for connection callbacks
        let connect_handler = handler.clone();
        let disconnect_handler = handler.clone();

        // Build the connection
        let db = DbConnection::builder()
            .with_uri(&ws_url)
            .with_module_name("heycat")
            .on_connect(move |_ctx, identity, _token| {
                crate::info!("Connected to SpacetimeDB with identity: {:?}", identity);
                connect_handler.on_connection_change(true, None);
            })
            .on_disconnect(move |_ctx, error| {
                let error_msg = error.as_ref().map(|e| e.to_string());
                crate::warn!("Disconnected from SpacetimeDB: {:?}", error_msg);
                disconnect_handler.on_connection_change(false, error_msg);
            })
            .build()
            .map_err(|e| ClientError::ConnectionFailed(e.to_string()))?;

        // Set up table subscriptions and callbacks
        self.setup_subscriptions(&db)?;

        // Start background thread to process messages
        let thread_handle = db.run_threaded();

        self.connection = Some(db);
        self._thread_handle = Some(thread_handle);
        self.state = ConnectionState::Connected;

        crate::info!("SpacetimeDB client connected and subscriptions active");
        Ok(())
    }

    /// Disconnect from SpacetimeDB
    pub fn disconnect(&mut self) {
        if let Some(db) = self.connection.take() {
            if let Err(e) = db.disconnect() {
                crate::warn!("Error disconnecting from SpacetimeDB: {}", e);
            }
        }
        self.state = ConnectionState::Disconnected;
        crate::debug!("SpacetimeDB client disconnected");
    }

    /// Set up table subscriptions with callbacks
    fn setup_subscriptions(&self, db: &DbConnection) -> Result<(), ClientError> {
        let handler = self.handler.clone();

        // Subscribe to all tables we care about
        let _subscription = db
            .subscription_builder()
            .on_applied(|_ctx: &SubscriptionEventContext| {
                crate::info!("SpacetimeDB subscriptions applied");
            })
            .on_error(|_ctx, error| {
                crate::error!("SpacetimeDB subscription error: {:?}", error);
            })
            .subscribe([
                "SELECT * FROM dictionary_entry",
                "SELECT * FROM window_context",
                "SELECT * FROM voice_command",
                "SELECT * FROM recording",
                "SELECT * FROM transcription",
            ]);

        // Set up dictionary_entry callbacks
        {
            let h = handler.clone();
            db.db.dictionary_entry().on_insert(move |_ctx, _row| {
                h.on_dictionary_change();
            });
        }
        {
            let h = handler.clone();
            db.db.dictionary_entry().on_update(move |_ctx, _old, _new| {
                h.on_dictionary_change();
            });
        }
        {
            let h = handler.clone();
            db.db.dictionary_entry().on_delete(move |_ctx, _row| {
                h.on_dictionary_change();
            });
        }

        // Set up window_context callbacks
        {
            let h = handler.clone();
            db.db.window_context().on_insert(move |_ctx, _row| {
                h.on_window_context_change();
            });
        }
        {
            let h = handler.clone();
            db.db.window_context().on_update(move |_ctx, _old, _new| {
                h.on_window_context_change();
            });
        }
        {
            let h = handler.clone();
            db.db.window_context().on_delete(move |_ctx, _row| {
                h.on_window_context_change();
            });
        }

        // Set up voice_command callbacks
        {
            let h = handler.clone();
            db.db.voice_command().on_insert(move |_ctx, _row| {
                h.on_voice_command_change();
            });
        }
        {
            let h = handler.clone();
            db.db.voice_command().on_update(move |_ctx, _old, _new| {
                h.on_voice_command_change();
            });
        }
        {
            let h = handler.clone();
            db.db.voice_command().on_delete(move |_ctx, _row| {
                h.on_voice_command_change();
            });
        }

        // Set up recording callbacks
        {
            let h = handler.clone();
            db.db.recording().on_insert(move |_ctx, row| {
                h.on_recording_change("insert", Some(row.id.clone()));
            });
        }
        {
            let h = handler.clone();
            db.db.recording().on_update(move |_ctx, _old, new| {
                h.on_recording_change("update", Some(new.id.clone()));
            });
        }
        {
            let h = handler.clone();
            db.db.recording().on_delete(move |_ctx, row| {
                h.on_recording_change("delete", Some(row.id.clone()));
            });
        }

        // Set up transcription callbacks
        {
            let h = handler.clone();
            db.db.transcription().on_insert(move |_ctx, row| {
                h.on_transcription_change(
                    "insert",
                    Some(row.id.clone()),
                    Some(row.recording_id.clone()),
                );
            });
        }
        {
            let h = handler.clone();
            db.db.transcription().on_update(move |_ctx, _old, new| {
                h.on_transcription_change(
                    "update",
                    Some(new.id.clone()),
                    Some(new.recording_id.clone()),
                );
            });
        }
        {
            let h = handler.clone();
            db.db.transcription().on_delete(move |_ctx, row| {
                h.on_transcription_change(
                    "delete",
                    Some(row.id.clone()),
                    Some(row.recording_id.clone()),
                );
            });
        }

        Ok(())
    }
}

impl Drop for SpacetimeClient {
    fn drop(&mut self) {
        self.disconnect();
    }
}

// ============================================================
// Dictionary Operations
// ============================================================

use crate::dictionary::{DictionaryEntry, DictionaryError};
use module_bindings::{
    add_dictionary_entry, delete_dictionary_entry, update_dictionary_entry,
    DictionaryEntry as SdbDictionaryEntry,
};
use uuid::Uuid;

impl SpacetimeClient {
    /// List all dictionary entries from the client cache
    pub fn list_dictionary_entries(&self) -> Result<Vec<DictionaryEntry>, DictionaryError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| DictionaryError::LoadError("Not connected to SpacetimeDB".to_string()))?;

        Ok(conn
            .db
            .dictionary_entry()
            .iter()
            .map(|e| convert_sdb_entry(&e))
            .collect())
    }

    /// Get a dictionary entry by ID
    pub fn get_dictionary_entry(&self, id: &str) -> Result<Option<DictionaryEntry>, DictionaryError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| DictionaryError::LoadError("Not connected to SpacetimeDB".to_string()))?;

        Ok(conn
            .db
            .dictionary_entry()
            .id()
            .find(&id.to_string())
            .map(|e| convert_sdb_entry(&e)))
    }

    /// Add a new dictionary entry via SpacetimeDB reducer
    pub fn add_dictionary_entry(
        &self,
        trigger: String,
        expansion: String,
        suffix: Option<String>,
        auto_enter: bool,
        disable_suffix: bool,
    ) -> Result<DictionaryEntry, DictionaryError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| DictionaryError::PersistenceError("Not connected to SpacetimeDB".to_string()))?;

        let id = Uuid::new_v4().to_string();

        conn.reducers
            .add_dictionary_entry(
                id.clone(),
                trigger.clone(),
                expansion.clone(),
                suffix.clone(),
                auto_enter,
                disable_suffix,
            )
            .map_err(|e| DictionaryError::PersistenceError(e.to_string()))?;

        Ok(DictionaryEntry {
            id,
            trigger,
            expansion,
            suffix,
            auto_enter,
            disable_suffix,
        })
    }

    /// Update an existing dictionary entry via SpacetimeDB reducer
    pub fn update_dictionary_entry(
        &self,
        id: String,
        trigger: String,
        expansion: String,
        suffix: Option<String>,
        auto_enter: bool,
        disable_suffix: bool,
    ) -> Result<DictionaryEntry, DictionaryError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| DictionaryError::PersistenceError("Not connected to SpacetimeDB".to_string()))?;

        // Check if entry exists in cache
        if conn.db.dictionary_entry().id().find(&id).is_none() {
            return Err(DictionaryError::NotFound(id));
        }

        conn.reducers
            .update_dictionary_entry(
                id.clone(),
                trigger.clone(),
                expansion.clone(),
                suffix.clone(),
                auto_enter,
                disable_suffix,
            )
            .map_err(|e| DictionaryError::PersistenceError(e.to_string()))?;

        Ok(DictionaryEntry {
            id,
            trigger,
            expansion,
            suffix,
            auto_enter,
            disable_suffix,
        })
    }

    /// Delete a dictionary entry via SpacetimeDB reducer
    pub fn delete_dictionary_entry(&self, id: &str) -> Result<(), DictionaryError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| DictionaryError::PersistenceError("Not connected to SpacetimeDB".to_string()))?;

        // Check if entry exists in cache
        if conn.db.dictionary_entry().id().find(&id.to_string()).is_none() {
            return Err(DictionaryError::NotFound(id.to_string()));
        }

        conn.reducers
            .delete_dictionary_entry(id.to_string())
            .map_err(|e| DictionaryError::PersistenceError(e.to_string()))?;

        Ok(())
    }
}

/// Convert from SpacetimeDB DictionaryEntry to our DictionaryEntry
fn convert_sdb_entry(sdb_entry: &SdbDictionaryEntry) -> DictionaryEntry {
    DictionaryEntry {
        id: sdb_entry.id.clone(),
        trigger: sdb_entry.trigger.clone(),
        expansion: sdb_entry.expansion.clone(),
        suffix: sdb_entry.suffix.clone(),
        auto_enter: sdb_entry.auto_enter,
        disable_suffix: sdb_entry.disable_suffix,
    }
}

// ============================================================
// Window Context Operations
// ============================================================

use crate::window_context::{
    OverrideMode, WindowContext, WindowContextStoreError, WindowMatcher,
};
use module_bindings::{
    add_window_context, delete_window_context, update_window_context,
    WindowContext as SdbWindowContext,
};

impl SpacetimeClient {
    /// List all window contexts from the client cache
    pub fn list_window_contexts(&self) -> Result<Vec<WindowContext>, WindowContextStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| WindowContextStoreError::LoadError("Not connected to SpacetimeDB".to_string()))?;

        Ok(conn
            .db
            .window_context()
            .iter()
            .filter_map(|e| convert_sdb_window_context(&e).ok())
            .collect())
    }

    /// Get a window context by ID
    pub fn get_window_context(&self, id: uuid::Uuid) -> Result<Option<WindowContext>, WindowContextStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| WindowContextStoreError::LoadError("Not connected to SpacetimeDB".to_string()))?;

        Ok(conn
            .db
            .window_context()
            .id()
            .find(&id.to_string())
            .and_then(|e| convert_sdb_window_context(&e).ok()))
    }

    /// Add a new window context via SpacetimeDB reducer
    pub fn add_window_context(
        &self,
        name: String,
        matcher: WindowMatcher,
        command_mode: OverrideMode,
        dictionary_mode: OverrideMode,
        command_ids: Vec<uuid::Uuid>,
        dictionary_entry_ids: Vec<String>,
        enabled: bool,
        priority: i32,
    ) -> Result<WindowContext, WindowContextStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| WindowContextStoreError::PersistenceError("Not connected to SpacetimeDB".to_string()))?;

        let id = uuid::Uuid::new_v4();

        // Serialize IDs to JSON strings
        let command_ids_json = serde_json::to_string(&command_ids)
            .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;
        let dictionary_entry_ids_json = serde_json::to_string(&dictionary_entry_ids)
            .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;

        conn.reducers
            .add_window_context(
                id.to_string(),
                name.clone(),
                matcher.app_name.clone(),
                matcher.title_pattern.clone(),
                matcher.bundle_id.clone(),
                override_mode_to_string(command_mode),
                override_mode_to_string(dictionary_mode),
                command_ids_json,
                dictionary_entry_ids_json,
                enabled,
                priority,
            )
            .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;

        Ok(WindowContext {
            id,
            name,
            matcher,
            command_mode,
            dictionary_mode,
            command_ids,
            dictionary_entry_ids,
            enabled,
            priority,
        })
    }

    /// Update an existing window context via SpacetimeDB reducer
    pub fn update_window_context(&self, context: WindowContext) -> Result<WindowContext, WindowContextStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| WindowContextStoreError::PersistenceError("Not connected to SpacetimeDB".to_string()))?;

        // Check if entry exists in cache
        if conn.db.window_context().id().find(&context.id.to_string()).is_none() {
            return Err(WindowContextStoreError::NotFound(context.id));
        }

        // Serialize IDs to JSON strings
        let command_ids_json = serde_json::to_string(&context.command_ids)
            .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;
        let dictionary_entry_ids_json = serde_json::to_string(&context.dictionary_entry_ids)
            .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;

        conn.reducers
            .update_window_context(
                context.id.to_string(),
                context.name.clone(),
                context.matcher.app_name.clone(),
                context.matcher.title_pattern.clone(),
                context.matcher.bundle_id.clone(),
                override_mode_to_string(context.command_mode),
                override_mode_to_string(context.dictionary_mode),
                command_ids_json,
                dictionary_entry_ids_json,
                context.enabled,
                context.priority,
            )
            .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;

        Ok(context)
    }

    /// Delete a window context via SpacetimeDB reducer
    pub fn delete_window_context(&self, id: uuid::Uuid) -> Result<(), WindowContextStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| WindowContextStoreError::PersistenceError("Not connected to SpacetimeDB".to_string()))?;

        // Check if entry exists in cache
        if conn.db.window_context().id().find(&id.to_string()).is_none() {
            return Err(WindowContextStoreError::NotFound(id));
        }

        conn.reducers
            .delete_window_context(id.to_string())
            .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;

        Ok(())
    }
}

/// Convert from SpacetimeDB WindowContext to our WindowContext
fn convert_sdb_window_context(sdb_ctx: &SdbWindowContext) -> Result<WindowContext, WindowContextStoreError> {
    let id = uuid::Uuid::parse_str(&sdb_ctx.id)
        .map_err(|e| WindowContextStoreError::LoadError(format!("Invalid UUID: {}", e)))?;

    let command_ids: Vec<uuid::Uuid> = serde_json::from_str(&sdb_ctx.command_ids_json)
        .map_err(|e| WindowContextStoreError::LoadError(format!("Invalid command_ids JSON: {}", e)))?;

    let dictionary_entry_ids: Vec<String> = serde_json::from_str(&sdb_ctx.dictionary_entry_ids_json)
        .map_err(|e| WindowContextStoreError::LoadError(format!("Invalid dictionary_entry_ids JSON: {}", e)))?;

    Ok(WindowContext {
        id,
        name: sdb_ctx.name.clone(),
        matcher: WindowMatcher {
            app_name: sdb_ctx.matcher_app_name.clone(),
            title_pattern: sdb_ctx.matcher_title_pattern.clone(),
            bundle_id: sdb_ctx.matcher_bundle_id.clone(),
        },
        command_mode: string_to_override_mode(&sdb_ctx.command_mode),
        dictionary_mode: string_to_override_mode(&sdb_ctx.dictionary_mode),
        command_ids,
        dictionary_entry_ids,
        enabled: sdb_ctx.enabled,
        priority: sdb_ctx.priority,
    })
}

/// Convert OverrideMode to string for SpacetimeDB
fn override_mode_to_string(mode: OverrideMode) -> String {
    match mode {
        OverrideMode::Merge => "merge".to_string(),
        OverrideMode::Replace => "replace".to_string(),
    }
}

/// Convert string to OverrideMode
fn string_to_override_mode(s: &str) -> OverrideMode {
    match s {
        "replace" => OverrideMode::Replace,
        _ => OverrideMode::Merge, // Default to Merge
    }
}

// ============================================================
// Recording Operations
// ============================================================

use crate::audio::StopReason;
use module_bindings::{
    add_recording, delete_recording, delete_recording_by_path,
    Recording as SdbRecording,
};

/// Recording metadata stored in SpacetimeDB
#[derive(Debug, Clone)]
pub struct RecordingRecord {
    pub id: String,
    pub file_path: String,
    pub duration_secs: f64,
    pub sample_count: u64,
    pub stop_reason: Option<StopReason>,
    pub created_at: String, // ISO 8601 format
}

/// Error type for recording operations
#[derive(Debug, Clone)]
pub enum RecordingStoreError {
    NotConnected,
    NotFound(String),
    PersistenceError(String),
}

impl std::fmt::Display for RecordingStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordingStoreError::NotConnected => write!(f, "Not connected to SpacetimeDB"),
            RecordingStoreError::NotFound(id) => write!(f, "Recording not found: {}", id),
            RecordingStoreError::PersistenceError(msg) => write!(f, "Recording persistence error: {}", msg),
        }
    }
}

impl std::error::Error for RecordingStoreError {}

impl SpacetimeClient {
    /// List all recordings from the client cache
    pub fn list_recordings(&self) -> Result<Vec<RecordingRecord>, RecordingStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or(RecordingStoreError::NotConnected)?;

        Ok(conn
            .db
            .recording()
            .iter()
            .map(|e| convert_sdb_recording(&e))
            .collect())
    }

    /// Get a recording by ID
    pub fn get_recording(&self, id: &str) -> Result<Option<RecordingRecord>, RecordingStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or(RecordingStoreError::NotConnected)?;

        Ok(conn
            .db
            .recording()
            .id()
            .find(&id.to_string())
            .map(|e| convert_sdb_recording(&e)))
    }

    /// Get a recording by file path
    pub fn get_recording_by_path(&self, file_path: &str) -> Result<Option<RecordingRecord>, RecordingStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or(RecordingStoreError::NotConnected)?;

        Ok(conn
            .db
            .recording()
            .iter()
            .find(|r| r.file_path == file_path)
            .map(|e| convert_sdb_recording(&e)))
    }

    /// Add a new recording via SpacetimeDB reducer
    pub fn add_recording(
        &self,
        id: String,
        file_path: String,
        duration_secs: f64,
        sample_count: u64,
        stop_reason: Option<StopReason>,
    ) -> Result<RecordingRecord, RecordingStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or(RecordingStoreError::NotConnected)?;

        let stop_reason_str = stop_reason.as_ref().map(|r| format!("{:?}", r));

        conn.reducers
            .add_recording(
                id.clone(),
                file_path.clone(),
                duration_secs,
                sample_count,
                stop_reason_str.clone(),
            )
            .map_err(|e| RecordingStoreError::PersistenceError(e.to_string()))?;

        Ok(RecordingRecord {
            id,
            file_path,
            duration_secs,
            sample_count,
            stop_reason,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Delete a recording by ID via SpacetimeDB reducer
    pub fn delete_recording(&self, id: &str) -> Result<(), RecordingStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or(RecordingStoreError::NotConnected)?;

        // Check if entry exists in cache
        if conn.db.recording().id().find(&id.to_string()).is_none() {
            return Err(RecordingStoreError::NotFound(id.to_string()));
        }

        conn.reducers
            .delete_recording(id.to_string())
            .map_err(|e| RecordingStoreError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    /// Delete a recording by file path via SpacetimeDB reducer
    pub fn delete_recording_by_path(&self, file_path: &str) -> Result<(), RecordingStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or(RecordingStoreError::NotConnected)?;

        // Check if entry exists in cache
        let exists = conn.db.recording().iter().any(|r| r.file_path == file_path);
        if !exists {
            return Err(RecordingStoreError::NotFound(file_path.to_string()));
        }

        conn.reducers
            .delete_recording_by_path(file_path.to_string())
            .map_err(|e| RecordingStoreError::PersistenceError(e.to_string()))?;

        Ok(())
    }
}

/// Convert from SpacetimeDB Recording to RecordingRecord
fn convert_sdb_recording(sdb_recording: &SdbRecording) -> RecordingRecord {
    // Parse stop_reason from string
    let stop_reason = sdb_recording.stop_reason.as_ref().and_then(|s| {
        match s.as_str() {
            "BufferFull" => Some(StopReason::BufferFull),
            "LockError" => Some(StopReason::LockError),
            "StreamError" => Some(StopReason::StreamError),
            "ResampleOverflow" => Some(StopReason::ResampleOverflow),
            "SilenceAfterSpeech" => Some(StopReason::SilenceAfterSpeech),
            "NoSpeechTimeout" => Some(StopReason::NoSpeechTimeout),
            _ => None,
        }
    });

    RecordingRecord {
        id: sdb_recording.id.clone(),
        file_path: sdb_recording.file_path.clone(),
        duration_secs: sdb_recording.duration_secs,
        sample_count: sdb_recording.sample_count,
        stop_reason,
        created_at: sdb_recording.created_at.to_string(),
    }
}

// ============================================================
// Transcription Operations
// ============================================================

use module_bindings::{
    add_transcription, delete_transcription,
    Transcription as SdbTranscription,
};

/// Transcription record stored in SpacetimeDB
#[derive(Debug, Clone)]
pub struct TranscriptionRecord {
    pub id: String,
    pub recording_id: String,
    pub text: String,
    pub language: Option<String>,
    pub model_version: String,
    pub duration_ms: u64,
    pub created_at: String, // ISO 8601 format
}

/// Error type for transcription operations
#[derive(Debug, Clone)]
pub enum TranscriptionStoreError {
    NotConnected,
    NotFound(String),
    PersistenceError(String),
}

impl std::fmt::Display for TranscriptionStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscriptionStoreError::NotConnected => write!(f, "Not connected to SpacetimeDB"),
            TranscriptionStoreError::NotFound(id) => write!(f, "Transcription not found: {}", id),
            TranscriptionStoreError::PersistenceError(msg) => write!(f, "Transcription persistence error: {}", msg),
        }
    }
}

impl std::error::Error for TranscriptionStoreError {}

impl SpacetimeClient {
    /// List all transcriptions from the client cache
    pub fn list_transcriptions(&self) -> Result<Vec<TranscriptionRecord>, TranscriptionStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or(TranscriptionStoreError::NotConnected)?;

        Ok(conn
            .db
            .transcription()
            .iter()
            .map(|e| convert_sdb_transcription(&e))
            .collect())
    }

    /// Get transcriptions by recording ID
    pub fn get_transcriptions_by_recording(&self, recording_id: &str) -> Result<Vec<TranscriptionRecord>, TranscriptionStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or(TranscriptionStoreError::NotConnected)?;

        Ok(conn
            .db
            .transcription()
            .iter()
            .filter(|t| t.recording_id == recording_id)
            .map(|e| convert_sdb_transcription(&e))
            .collect())
    }

    /// Add a new transcription via SpacetimeDB reducer
    pub fn add_transcription(
        &self,
        id: String,
        recording_id: String,
        text: String,
        language: Option<String>,
        model_version: String,
        duration_ms: u64,
    ) -> Result<TranscriptionRecord, TranscriptionStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or(TranscriptionStoreError::NotConnected)?;

        conn.reducers
            .add_transcription(
                id.clone(),
                recording_id.clone(),
                text.clone(),
                language.clone(),
                model_version.clone(),
                duration_ms,
            )
            .map_err(|e| TranscriptionStoreError::PersistenceError(e.to_string()))?;

        Ok(TranscriptionRecord {
            id,
            recording_id,
            text,
            language,
            model_version,
            duration_ms,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Delete a transcription by ID via SpacetimeDB reducer
    pub fn delete_transcription(&self, id: &str) -> Result<(), TranscriptionStoreError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or(TranscriptionStoreError::NotConnected)?;

        // Check if entry exists in cache
        if conn.db.transcription().id().find(&id.to_string()).is_none() {
            return Err(TranscriptionStoreError::NotFound(id.to_string()));
        }

        conn.reducers
            .delete_transcription(id.to_string())
            .map_err(|e| TranscriptionStoreError::PersistenceError(e.to_string()))?;

        Ok(())
    }
}

/// Convert from SpacetimeDB Transcription to TranscriptionRecord
fn convert_sdb_transcription(sdb_transcription: &SdbTranscription) -> TranscriptionRecord {
    TranscriptionRecord {
        id: sdb_transcription.id.clone(),
        recording_id: sdb_transcription.recording_id.clone(),
        text: sdb_transcription.text.clone(),
        language: sdb_transcription.language.clone(),
        model_version: sdb_transcription.model_version.clone(),
        duration_ms: sdb_transcription.duration_ms,
        created_at: sdb_transcription.created_at.to_string(),
    }
}

// ============================================================
// Voice Command Operations
// ============================================================

use crate::voice_commands::registry::{ActionType, CommandDefinition, RegistryError};
use module_bindings::{
    add_voice_command, delete_voice_command, update_voice_command,
    VoiceCommand as SdbVoiceCommand,
};
use std::collections::HashMap;

impl SpacetimeClient {
    /// List all voice commands from the client cache
    pub fn list_voice_commands(&self) -> Result<Vec<CommandDefinition>, RegistryError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| RegistryError::LoadError("Not connected to SpacetimeDB".to_string()))?;

        conn.db
            .voice_command()
            .iter()
            .map(|e| convert_sdb_voice_command(&e))
            .collect()
    }

    /// Get a voice command by ID
    pub fn get_voice_command(&self, id: Uuid) -> Result<Option<CommandDefinition>, RegistryError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| RegistryError::LoadError("Not connected to SpacetimeDB".to_string()))?;

        conn.db
            .voice_command()
            .id()
            .find(&id.to_string())
            .map(|e| convert_sdb_voice_command(&e))
            .transpose()
    }

    /// Add a new voice command via SpacetimeDB reducer
    pub fn add_voice_command(&self, cmd: &CommandDefinition) -> Result<(), RegistryError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| RegistryError::PersistenceError("Not connected to SpacetimeDB".to_string()))?;

        // Serialize parameters to JSON
        let parameters_json = serde_json::to_string(&cmd.parameters)
            .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        conn.reducers
            .add_voice_command(
                cmd.id.to_string(),
                cmd.trigger.clone(),
                action_type_to_string(&cmd.action_type),
                parameters_json,
                cmd.enabled,
            )
            .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    /// Update an existing voice command via SpacetimeDB reducer
    pub fn update_voice_command(&self, cmd: &CommandDefinition) -> Result<(), RegistryError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| RegistryError::PersistenceError("Not connected to SpacetimeDB".to_string()))?;

        // Check if command exists in cache
        if conn.db.voice_command().id().find(&cmd.id.to_string()).is_none() {
            return Err(RegistryError::NotFound(cmd.id));
        }

        // Serialize parameters to JSON
        let parameters_json = serde_json::to_string(&cmd.parameters)
            .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        conn.reducers
            .update_voice_command(
                cmd.id.to_string(),
                cmd.trigger.clone(),
                action_type_to_string(&cmd.action_type),
                parameters_json,
                cmd.enabled,
            )
            .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    /// Delete a voice command via SpacetimeDB reducer
    pub fn delete_voice_command(&self, id: Uuid) -> Result<(), RegistryError> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| RegistryError::PersistenceError("Not connected to SpacetimeDB".to_string()))?;

        // Check if command exists in cache
        if conn.db.voice_command().id().find(&id.to_string()).is_none() {
            return Err(RegistryError::NotFound(id));
        }

        conn.reducers
            .delete_voice_command(id.to_string())
            .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        Ok(())
    }
}

/// Convert from SpacetimeDB VoiceCommand to CommandDefinition
fn convert_sdb_voice_command(sdb_cmd: &SdbVoiceCommand) -> Result<CommandDefinition, RegistryError> {
    let id = Uuid::parse_str(&sdb_cmd.id)
        .map_err(|e| RegistryError::LoadError(format!("Invalid UUID: {}", e)))?;

    let parameters: HashMap<String, String> = serde_json::from_str(&sdb_cmd.parameters_json)
        .map_err(|e| RegistryError::LoadError(format!("Invalid parameters JSON: {}", e)))?;

    let action_type = string_to_action_type(&sdb_cmd.action_type);

    Ok(CommandDefinition {
        id,
        trigger: sdb_cmd.trigger.clone(),
        action_type,
        parameters,
        enabled: sdb_cmd.enabled,
    })
}

/// Convert ActionType to string for SpacetimeDB storage
fn action_type_to_string(action_type: &ActionType) -> String {
    match action_type {
        ActionType::OpenApp => "open_app".to_string(),
        ActionType::TypeText => "type_text".to_string(),
        ActionType::SystemControl => "system_control".to_string(),
        ActionType::Custom => "custom".to_string(),
    }
}

/// Convert string to ActionType
fn string_to_action_type(s: &str) -> ActionType {
    match s {
        "open_app" => ActionType::OpenApp,
        "type_text" => ActionType::TypeText,
        "system_control" => ActionType::SystemControl,
        "custom" => ActionType::Custom,
        _ => ActionType::Custom, // Default to Custom for unknown types
    }
}

#[cfg(test)]
mod integration_test;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state() {
        let state = ConnectionState::Disconnected;
        assert!(matches!(state, ConnectionState::Disconnected));

        let state = ConnectionState::Connected;
        assert!(matches!(state, ConnectionState::Connected));

        let state = ConnectionState::Failed("test error".to_string());
        assert!(matches!(state, ConnectionState::Failed(_)));
    }

    #[test]
    fn test_websocket_url_format() {
        let config = SidecarConfig::new(None);
        assert_eq!(config.websocket_url(), "ws://127.0.0.1:3055");
    }

    #[test]
    fn test_action_type_conversion() {
        assert_eq!(action_type_to_string(&ActionType::OpenApp), "open_app");
        assert_eq!(action_type_to_string(&ActionType::TypeText), "type_text");
        assert_eq!(action_type_to_string(&ActionType::SystemControl), "system_control");
        assert_eq!(action_type_to_string(&ActionType::Custom), "custom");

        assert!(matches!(string_to_action_type("open_app"), ActionType::OpenApp));
        assert!(matches!(string_to_action_type("type_text"), ActionType::TypeText));
        assert!(matches!(string_to_action_type("system_control"), ActionType::SystemControl));
        assert!(matches!(string_to_action_type("custom"), ActionType::Custom));
        assert!(matches!(string_to_action_type("unknown"), ActionType::Custom));
    }
}
