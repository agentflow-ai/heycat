//! Tauri event emitter implementation.
//!
//! Provides TauriEventEmitter which implements all event emitter traits
//! for production use with Tauri's event system.

use tauri::{AppHandle, Emitter};

use crate::emit_or_warn;
use crate::events::{
    command_events, event_names, hotkey_events, CommandAmbiguousPayload, CommandEventEmitter,
    CommandExecutedPayload, CommandFailedPayload, CommandMatchedPayload, HotkeyEventEmitter,
    RecordingCancelledPayload, RecordingErrorPayload, RecordingEventEmitter,
    RecordingStartedPayload, RecordingStoppedPayload, TranscriptionCompletedPayload,
    TranscriptionErrorPayload, TranscriptionEventEmitter, TranscriptionStartedPayload,
};
use crate::util::SettingsAccess;

/// Tauri AppHandle-based event emitter for production use.
///
/// Implements all event emitter traits (RecordingEventEmitter, TranscriptionEventEmitter,
/// CommandEventEmitter, HotkeyEventEmitter) using Tauri's native event system.
pub struct TauriEventEmitter {
    app_handle: AppHandle,
}

impl TauriEventEmitter {
    /// Create a new TauriEventEmitter with the given AppHandle.
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// Get a reference to the underlying AppHandle.
    pub fn app_handle(&self) -> &AppHandle {
        &self.app_handle
    }
}

impl SettingsAccess for TauriEventEmitter {
    fn app_handle(&self) -> Option<&AppHandle> {
        Some(&self.app_handle)
    }
}

impl RecordingEventEmitter for TauriEventEmitter {
    fn emit_recording_started(&self, payload: RecordingStartedPayload) {
        emit_or_warn!(self.app_handle, event_names::RECORDING_STARTED, payload);
    }

    fn emit_recording_stopped(&self, payload: RecordingStoppedPayload) {
        emit_or_warn!(self.app_handle, event_names::RECORDING_STOPPED, payload);
    }

    fn emit_recording_cancelled(&self, payload: RecordingCancelledPayload) {
        emit_or_warn!(self.app_handle, event_names::RECORDING_CANCELLED, payload);
    }

    fn emit_recording_error(&self, payload: RecordingErrorPayload) {
        emit_or_warn!(self.app_handle, event_names::RECORDING_ERROR, payload);
    }
}

impl TranscriptionEventEmitter for TauriEventEmitter {
    fn emit_transcription_started(&self, payload: TranscriptionStartedPayload) {
        emit_or_warn!(self.app_handle, event_names::TRANSCRIPTION_STARTED, payload);
    }

    fn emit_transcription_completed(&self, payload: TranscriptionCompletedPayload) {
        emit_or_warn!(
            self.app_handle,
            event_names::TRANSCRIPTION_COMPLETED,
            payload
        );
    }

    fn emit_transcription_error(&self, payload: TranscriptionErrorPayload) {
        emit_or_warn!(self.app_handle, event_names::TRANSCRIPTION_ERROR, payload);
    }
}

impl CommandEventEmitter for TauriEventEmitter {
    fn emit_command_matched(&self, payload: CommandMatchedPayload) {
        emit_or_warn!(self.app_handle, command_events::COMMAND_MATCHED, payload);
    }

    fn emit_command_executed(&self, payload: CommandExecutedPayload) {
        emit_or_warn!(self.app_handle, command_events::COMMAND_EXECUTED, payload);
    }

    fn emit_command_failed(&self, payload: CommandFailedPayload) {
        emit_or_warn!(self.app_handle, command_events::COMMAND_FAILED, payload);
    }

    fn emit_command_ambiguous(&self, payload: CommandAmbiguousPayload) {
        emit_or_warn!(self.app_handle, command_events::COMMAND_AMBIGUOUS, payload);
    }
}

impl HotkeyEventEmitter for TauriEventEmitter {
    fn emit_key_blocking_unavailable(
        &self,
        payload: hotkey_events::KeyBlockingUnavailablePayload,
    ) {
        emit_or_warn!(
            self.app_handle,
            hotkey_events::KEY_BLOCKING_UNAVAILABLE,
            payload
        );
    }
}

#[cfg(test)]
#[path = "emitter_test.rs"]
mod tests;
