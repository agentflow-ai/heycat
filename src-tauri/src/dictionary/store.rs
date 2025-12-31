// Dictionary store types for dictionary entries
//
// The DictionaryStore struct was removed as Turso is now used for persistence.
// The DictionaryEntry and DictionaryError types are kept for API compatibility.

use serde::{Deserialize, Serialize};

/// A dictionary entry for text expansion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryEntry {
    /// Unique identifier for the entry
    pub id: String,
    /// Trigger word/phrase (e.g., "brb")
    pub trigger: String,
    /// Expansion text (e.g., "be right back")
    pub expansion: String,
    /// Optional suffix appended after expansion
    #[serde(default)]
    pub suffix: Option<String>,
    /// Whether to simulate enter keypress after expansion
    #[serde(default, alias = "auto_enter")]
    pub auto_enter: bool,
    /// Whether to suppress any trailing punctuation from the transcription
    /// When true, trailing punctuation after the trigger match is stripped
    #[serde(default, alias = "disable_suffix")]
    pub disable_suffix: bool,
    /// Whether the trigger should only expand when it's the complete transcription input
    /// When true, "brb" expands only if the entire input is "brb", not if it appears within a sentence
    #[serde(default, alias = "complete_match_only")]
    pub complete_match_only: bool,
}

/// Error types for dictionary operations
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DictionaryError {
    /// Entry with this ID already exists
    #[allow(dead_code)]
    #[error("Entry with ID {0} already exists")]
    DuplicateId(String),
    /// Entry not found
    #[error("Entry with ID {0} not found")]
    NotFound(String),
    /// Failed to persist entries
    #[error("Failed to persist entries: {0}")]
    PersistenceError(String),
    /// Failed to load entries
    #[error("Failed to load entries: {0}")]
    LoadError(String),
}
