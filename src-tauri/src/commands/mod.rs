//! Tauri IPC commands module
//!
//! This module contains Tauri-specific command wrappers and is excluded from coverage.
//! The actual logic is in logic.rs which is fully tested.
//!
//! ## Module Organization
//!
//! - `recording`: Recording commands (start, stop, list, delete)
//! - `transcription`: Transcription commands
//! - `audio`: Audio device commands
//! - `hotkey`: Hotkey management commands
//! - `dictionary`: Dictionary management commands
//! - `window_context`: Window context commands
//! - `common`: Shared utilities (TauriEventEmitter)
//! - `logic`: Core command logic (testable)

#![cfg_attr(coverage_nightly, coverage(off))]

pub mod audio;
pub mod common;
pub mod dictionary;
pub mod hotkey;
pub mod logic;
pub mod recording;
pub mod transcription;
pub mod window;
pub mod window_context;

// Re-export TauriEventEmitter from common module for backward compatibility
pub use common::TauriEventEmitter;

// Re-export state type aliases from app::state for backward compatibility
pub use crate::app::state::{
    AudioMonitorState, AudioThreadState, HotkeyIntegrationState, HotkeyServiceState,
    KeyboardCaptureState, ProductionState, TranscriptionServiceState, TursoClientState,
};

// Worktree commands
use tauri::State;

/// Get the settings file name for the current worktree context
#[tauri::command]
pub fn get_settings_file_name(
    worktree_state: State<'_, crate::worktree::WorktreeState>,
) -> String {
    worktree_state.settings_file_name()
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
