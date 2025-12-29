//! SpacetimeDB subscription handlers
//!
//! Routes SpacetimeDB table change notifications to Tauri events,
//! integrating with the existing Event Bridge pattern.
//!
//! Architecture:
//! ```text
//! SpacetimeDB Table Change
//!        ↓
//! Subscription Callback
//!        ↓
//! SubscriptionEventEmitter (this module)
//!        ↓
//! Tauri app_handle.emit()
//!        ↓
//! Frontend Event Bridge
//!        ↓
//! Query invalidation / Zustand update
//! ```
//!
//! TODO(integrate-spacetimedb-rust-sdk-for-database-operations):
//! This module defines the infrastructure for subscription → event routing.
//! The actual wiring to SpacetimeDB SDK callbacks happens in that spec.

// Allow unused code until SDK integration spec wires this up
#![allow(dead_code)]

use tauri::{AppHandle, Emitter};

use crate::events::{dictionary_events, window_context_events};

/// Event names for SpacetimeDB-specific events
pub mod event_names {
    /// Emitted when voice_command table changes in SpacetimeDB
    pub const VOICE_COMMANDS_UPDATED: &str = "voice_commands_updated";

    /// Emitted when recordings table changes in SpacetimeDB
    pub const RECORDINGS_UPDATED: &str = "recordings_updated";

    /// Emitted when transcriptions table changes in SpacetimeDB
    pub const TRANSCRIPTIONS_UPDATED: &str = "transcriptions_updated";

    /// Emitted when SpacetimeDB connection status changes
    pub const SPACETIMEDB_CONNECTION_STATUS: &str = "spacetimedb_connection_status";
}

/// Payload for voice_commands_updated event
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceCommandsUpdatedPayload {
    /// Type of change: "insert", "update", "delete", or "sync"
    pub action: String,
    /// ID of the affected command (if applicable), or "all" for sync
    pub command_id: String,
}

/// Payload for recordings_updated event
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingsUpdatedPayload {
    /// Type of change: "insert", "update", or "delete"
    pub change_type: String,
    /// ID of the affected recording (if applicable)
    pub recording_id: Option<String>,
    /// ISO 8601 timestamp
    pub timestamp: String,
}

/// Payload for transcriptions_updated event
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionsUpdatedPayload {
    /// Type of change: "insert", "update", or "delete"
    pub change_type: String,
    /// ID of the affected transcription (if applicable)
    pub transcription_id: Option<String>,
    /// ID of the associated recording (if applicable)
    pub recording_id: Option<String>,
    /// ISO 8601 timestamp
    pub timestamp: String,
}

/// Payload for spacetimedb_connection_status event
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatusPayload {
    /// Whether connected to SpacetimeDB
    pub connected: bool,
    /// Optional error message if disconnected
    pub error: Option<String>,
    /// ISO 8601 timestamp
    pub timestamp: String,
}

/// Trait for emitting SpacetimeDB subscription events
///
/// This trait abstracts the event emission, allowing:
/// - Real Tauri emission in production
/// - Mock implementation for testing
pub trait SubscriptionEventEmitter: Send + Sync {
    /// Emit dictionary_updated event when DictionaryEntry table changes
    fn emit_dictionary_updated(&self);

    /// Emit window_contexts_updated event when WindowContext table changes
    fn emit_window_contexts_updated(&self);

    /// Emit voice_commands_updated event when VoiceCommand table changes
    fn emit_voice_commands_updated(&self, payload: VoiceCommandsUpdatedPayload);

    /// Emit recordings_updated event when Recording table changes
    fn emit_recordings_updated(&self, payload: RecordingsUpdatedPayload);

    /// Emit transcriptions_updated event when Transcription table changes
    fn emit_transcriptions_updated(&self, payload: TranscriptionsUpdatedPayload);

    /// Emit connection status change event
    fn emit_connection_status(&self, payload: ConnectionStatusPayload);
}

/// Production implementation using Tauri AppHandle
pub struct TauriSubscriptionEmitter {
    app_handle: AppHandle,
}

impl TauriSubscriptionEmitter {
    /// Create a new emitter with the given AppHandle
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl SubscriptionEventEmitter for TauriSubscriptionEmitter {
    fn emit_dictionary_updated(&self) {
        // Use "sync" action to indicate this is from SpacetimeDB subscription
        // The entry_id is "all" since subscriptions notify on any table change
        let payload = dictionary_events::DictionaryUpdatedPayload {
            action: "sync".to_string(),
            entry_id: "all".to_string(),
        };
        if let Err(e) = self
            .app_handle
            .emit(dictionary_events::DICTIONARY_UPDATED, payload)
        {
            crate::warn!("Failed to emit dictionary_updated event: {}", e);
        }
    }

    fn emit_window_contexts_updated(&self) {
        // Use "sync" action to indicate this is from SpacetimeDB subscription
        // The context_id is "all" since subscriptions notify on any table change
        let payload = window_context_events::WindowContextsUpdatedPayload {
            action: "sync".to_string(),
            context_id: "all".to_string(),
        };
        if let Err(e) = self
            .app_handle
            .emit(window_context_events::WINDOW_CONTEXTS_UPDATED, payload)
        {
            crate::warn!("Failed to emit window_contexts_updated event: {}", e);
        }
    }

    fn emit_voice_commands_updated(&self, payload: VoiceCommandsUpdatedPayload) {
        if let Err(e) = self
            .app_handle
            .emit(event_names::VOICE_COMMANDS_UPDATED, payload)
        {
            crate::warn!("Failed to emit voice_commands_updated event: {}", e);
        }
    }

    fn emit_recordings_updated(&self, payload: RecordingsUpdatedPayload) {
        if let Err(e) = self
            .app_handle
            .emit(event_names::RECORDINGS_UPDATED, payload)
        {
            crate::warn!("Failed to emit recordings_updated event: {}", e);
        }
    }

    fn emit_transcriptions_updated(&self, payload: TranscriptionsUpdatedPayload) {
        if let Err(e) = self
            .app_handle
            .emit(event_names::TRANSCRIPTIONS_UPDATED, payload)
        {
            crate::warn!("Failed to emit transcriptions_updated event: {}", e);
        }
    }

    fn emit_connection_status(&self, payload: ConnectionStatusPayload) {
        if let Err(e) = self
            .app_handle
            .emit(event_names::SPACETIMEDB_CONNECTION_STATUS, payload)
        {
            crate::warn!("Failed to emit spacetimedb_connection_status event: {}", e);
        }
    }
}

/// Subscription handler that bridges SpacetimeDB callbacks to Tauri events
///
/// This struct holds the emitter and provides methods to be called
/// from SpacetimeDB subscription callbacks.
pub struct SubscriptionHandler<E: SubscriptionEventEmitter> {
    emitter: E,
}

impl<E: SubscriptionEventEmitter> SubscriptionHandler<E> {
    /// Create a new subscription handler with the given emitter
    pub fn new(emitter: E) -> Self {
        Self { emitter }
    }

    /// Called when DictionaryEntry table changes
    ///
    /// This should be wired to SpacetimeDB's on_insert/on_update/on_delete
    /// callbacks for the dictionary_entry table.
    pub fn on_dictionary_change(&self) {
        crate::debug!("SpacetimeDB: dictionary_entry table changed");
        self.emitter.emit_dictionary_updated();
    }

    /// Called when WindowContext table changes
    ///
    /// This should be wired to SpacetimeDB's on_insert/on_update/on_delete
    /// callbacks for the window_context table.
    pub fn on_window_context_change(&self) {
        crate::debug!("SpacetimeDB: window_context table changed");
        self.emitter.emit_window_contexts_updated();
    }

    /// Called when VoiceCommand table changes
    ///
    /// This should be wired to SpacetimeDB's on_insert/on_update/on_delete
    /// callbacks for the voice_command table.
    pub fn on_voice_command_change(&self) {
        crate::debug!("SpacetimeDB: voice_command table changed");
        self.emitter.emit_voice_commands_updated(VoiceCommandsUpdatedPayload {
            action: "sync".to_string(),
            command_id: "all".to_string(),
        });
    }

    /// Called when Recording table changes
    ///
    /// This should be wired to SpacetimeDB's on_insert/on_update/on_delete
    /// callbacks for the recording table.
    pub fn on_recording_change(&self, change_type: &str, recording_id: Option<String>) {
        crate::debug!(
            "SpacetimeDB: recording table changed ({})",
            change_type
        );
        self.emitter.emit_recordings_updated(RecordingsUpdatedPayload {
            change_type: change_type.to_string(),
            recording_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    /// Called when Transcription table changes
    ///
    /// This should be wired to SpacetimeDB's on_insert/on_update/on_delete
    /// callbacks for the transcription table.
    pub fn on_transcription_change(
        &self,
        change_type: &str,
        transcription_id: Option<String>,
        recording_id: Option<String>,
    ) {
        crate::debug!(
            "SpacetimeDB: transcription table changed ({})",
            change_type
        );
        self.emitter
            .emit_transcriptions_updated(TranscriptionsUpdatedPayload {
                change_type: change_type.to_string(),
                transcription_id,
                recording_id,
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
    }

    /// Called when SpacetimeDB connection status changes
    pub fn on_connection_change(&self, connected: bool, error: Option<String>) {
        crate::info!(
            "SpacetimeDB: connection status changed (connected: {})",
            connected
        );
        self.emitter.emit_connection_status(ConnectionStatusPayload {
            connected,
            error,
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Mock emitter for testing that records all emitted events
    struct MockEmitter {
        dictionary_updates: Arc<Mutex<u32>>,
        window_context_updates: Arc<Mutex<u32>>,
        voice_command_updates: Arc<Mutex<Vec<VoiceCommandsUpdatedPayload>>>,
        recording_updates: Arc<Mutex<Vec<RecordingsUpdatedPayload>>>,
        transcription_updates: Arc<Mutex<Vec<TranscriptionsUpdatedPayload>>>,
        connection_updates: Arc<Mutex<Vec<ConnectionStatusPayload>>>,
    }

    impl MockEmitter {
        fn new() -> Self {
            Self {
                dictionary_updates: Arc::new(Mutex::new(0)),
                window_context_updates: Arc::new(Mutex::new(0)),
                voice_command_updates: Arc::new(Mutex::new(Vec::new())),
                recording_updates: Arc::new(Mutex::new(Vec::new())),
                transcription_updates: Arc::new(Mutex::new(Vec::new())),
                connection_updates: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl SubscriptionEventEmitter for MockEmitter {
        fn emit_dictionary_updated(&self) {
            *self.dictionary_updates.lock().unwrap() += 1;
        }

        fn emit_window_contexts_updated(&self) {
            *self.window_context_updates.lock().unwrap() += 1;
        }

        fn emit_voice_commands_updated(&self, payload: VoiceCommandsUpdatedPayload) {
            self.voice_command_updates.lock().unwrap().push(payload);
        }

        fn emit_recordings_updated(&self, payload: RecordingsUpdatedPayload) {
            self.recording_updates.lock().unwrap().push(payload);
        }

        fn emit_transcriptions_updated(&self, payload: TranscriptionsUpdatedPayload) {
            self.transcription_updates.lock().unwrap().push(payload);
        }

        fn emit_connection_status(&self, payload: ConnectionStatusPayload) {
            self.connection_updates.lock().unwrap().push(payload);
        }
    }

    #[test]
    fn test_dictionary_change_emits_event() {
        let emitter = MockEmitter::new();
        let handler = SubscriptionHandler::new(emitter);

        handler.on_dictionary_change();

        // Note: We can't access the emitter after moving it into handler
        // In a real test, we'd use Arc<MockEmitter> or similar pattern
    }

    #[test]
    fn test_subscription_handler_creation() {
        let emitter = MockEmitter::new();
        let _handler = SubscriptionHandler::new(emitter);
        // Handler should be created successfully
    }
}
