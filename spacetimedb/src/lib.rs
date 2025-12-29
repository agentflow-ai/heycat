//! SpacetimeDB module for heycat data persistence
//!
//! This module defines tables and reducers for:
//! - Dictionary entries (text expansion)
//! - Window contexts (context-sensitive commands)
//! - Voice commands (trigger phrases to actions)
//! - Recording metadata (audio file references)
//! - Transcriptions (linked to recordings)

use spacetimedb::{reducer, table, ReducerContext, Table, Timestamp};

// =============================================================================
// Dictionary Entry Table
// =============================================================================

/// A dictionary entry for text expansion
#[table(name = dictionary_entry, public)]
pub struct DictionaryEntry {
    /// Unique identifier (UUID string)
    #[primary_key]
    pub id: String,
    /// Trigger word/phrase (e.g., "brb")
    #[unique]
    pub trigger: String,
    /// Expansion text (e.g., "be right back")
    pub expansion: String,
    /// Optional suffix appended after expansion
    pub suffix: Option<String>,
    /// Whether to simulate enter keypress after expansion
    pub auto_enter: bool,
    /// Whether to suppress trailing punctuation from transcription
    pub disable_suffix: bool,
    /// When the entry was created
    pub created_at: Timestamp,
}

/// Add a new dictionary entry
#[reducer]
pub fn add_dictionary_entry(
    ctx: &ReducerContext,
    id: String,
    trigger: String,
    expansion: String,
    suffix: Option<String>,
    auto_enter: bool,
    disable_suffix: bool,
) -> Result<(), String> {
    // Check for duplicate trigger
    if ctx.db.dictionary_entry().trigger().find(&trigger).is_some() {
        return Err(format!("Dictionary entry with trigger '{}' already exists", trigger));
    }

    ctx.db.dictionary_entry().insert(DictionaryEntry {
        id,
        trigger,
        expansion,
        suffix,
        auto_enter,
        disable_suffix,
        created_at: ctx.timestamp,
    });

    Ok(())
}

/// Update an existing dictionary entry
#[reducer]
pub fn update_dictionary_entry(
    ctx: &ReducerContext,
    id: String,
    trigger: String,
    expansion: String,
    suffix: Option<String>,
    auto_enter: bool,
    disable_suffix: bool,
) -> Result<(), String> {
    let entry = ctx
        .db
        .dictionary_entry()
        .id()
        .find(&id)
        .ok_or_else(|| format!("Dictionary entry with id '{}' not found", id))?;

    // Check if new trigger conflicts with another entry
    if let Some(existing) = ctx.db.dictionary_entry().trigger().find(&trigger) {
        if existing.id != id {
            return Err(format!(
                "Dictionary entry with trigger '{}' already exists",
                trigger
            ));
        }
    }

    ctx.db.dictionary_entry().id().delete(&id);
    ctx.db.dictionary_entry().insert(DictionaryEntry {
        id,
        trigger,
        expansion,
        suffix,
        auto_enter,
        disable_suffix,
        created_at: entry.created_at,
    });

    Ok(())
}

/// Delete a dictionary entry by ID
#[reducer]
pub fn delete_dictionary_entry(ctx: &ReducerContext, id: String) -> Result<(), String> {
    if ctx.db.dictionary_entry().id().find(&id).is_none() {
        return Err(format!("Dictionary entry with id '{}' not found", id));
    }

    ctx.db.dictionary_entry().id().delete(&id);
    Ok(())
}

// =============================================================================
// Window Context Table
// =============================================================================

/// A window context definition for context-sensitive commands
#[table(name = window_context, public)]
pub struct WindowContext {
    /// Unique identifier (UUID string)
    #[primary_key]
    pub id: String,
    /// Human-readable name for this context
    pub name: String,
    /// App name to match
    pub matcher_app_name: String,
    /// Optional title pattern (regex) to match
    pub matcher_title_pattern: Option<String>,
    /// Optional bundle ID to match
    pub matcher_bundle_id: Option<String>,
    /// Override mode for commands: "merge" or "replace"
    pub command_mode: String,
    /// Override mode for dictionary: "merge" or "replace"
    pub dictionary_mode: String,
    /// JSON array of command UUIDs associated with this context
    pub command_ids_json: String,
    /// JSON array of dictionary entry IDs associated with this context
    pub dictionary_entry_ids_json: String,
    /// Whether this context is enabled
    pub enabled: bool,
    /// Priority for matching (higher = checked first)
    pub priority: i32,
    /// When the context was created
    pub created_at: Timestamp,
}

/// Add a new window context
#[reducer]
pub fn add_window_context(
    ctx: &ReducerContext,
    id: String,
    name: String,
    matcher_app_name: String,
    matcher_title_pattern: Option<String>,
    matcher_bundle_id: Option<String>,
    command_mode: String,
    dictionary_mode: String,
    command_ids_json: String,
    dictionary_entry_ids_json: String,
    enabled: bool,
    priority: i32,
) -> Result<(), String> {
    if ctx.db.window_context().id().find(&id).is_some() {
        return Err(format!("Window context with id '{}' already exists", id));
    }

    ctx.db.window_context().insert(WindowContext {
        id,
        name,
        matcher_app_name,
        matcher_title_pattern,
        matcher_bundle_id,
        command_mode,
        dictionary_mode,
        command_ids_json,
        dictionary_entry_ids_json,
        enabled,
        priority,
        created_at: ctx.timestamp,
    });

    Ok(())
}

/// Update an existing window context
#[reducer]
pub fn update_window_context(
    ctx: &ReducerContext,
    id: String,
    name: String,
    matcher_app_name: String,
    matcher_title_pattern: Option<String>,
    matcher_bundle_id: Option<String>,
    command_mode: String,
    dictionary_mode: String,
    command_ids_json: String,
    dictionary_entry_ids_json: String,
    enabled: bool,
    priority: i32,
) -> Result<(), String> {
    let entry = ctx
        .db
        .window_context()
        .id()
        .find(&id)
        .ok_or_else(|| format!("Window context with id '{}' not found", id))?;

    ctx.db.window_context().id().delete(&id);
    ctx.db.window_context().insert(WindowContext {
        id,
        name,
        matcher_app_name,
        matcher_title_pattern,
        matcher_bundle_id,
        command_mode,
        dictionary_mode,
        command_ids_json,
        dictionary_entry_ids_json,
        enabled,
        priority,
        created_at: entry.created_at,
    });

    Ok(())
}

/// Delete a window context by ID
#[reducer]
pub fn delete_window_context(ctx: &ReducerContext, id: String) -> Result<(), String> {
    if ctx.db.window_context().id().find(&id).is_none() {
        return Err(format!("Window context with id '{}' not found", id));
    }

    ctx.db.window_context().id().delete(&id);
    Ok(())
}

// =============================================================================
// Recording Table
// =============================================================================

/// Metadata for a recorded audio file
#[table(name = recording, public)]
pub struct Recording {
    /// Unique identifier (UUID string)
    #[primary_key]
    pub id: String,
    /// Path to the audio file on disk
    #[unique]
    pub file_path: String,
    /// Duration of the recording in seconds
    pub duration_secs: f64,
    /// Number of audio samples
    pub sample_count: u64,
    /// Why recording stopped (e.g., "silence_detected", "cancel_phrase", null for user-initiated)
    pub stop_reason: Option<String>,
    /// When the recording was created
    pub created_at: Timestamp,
}

/// Add a new recording entry
#[reducer]
pub fn add_recording(
    ctx: &ReducerContext,
    id: String,
    file_path: String,
    duration_secs: f64,
    sample_count: u64,
    stop_reason: Option<String>,
) -> Result<(), String> {
    if ctx.db.recording().file_path().find(&file_path).is_some() {
        return Err(format!(
            "Recording with file_path '{}' already exists",
            file_path
        ));
    }

    ctx.db.recording().insert(Recording {
        id,
        file_path,
        duration_secs,
        sample_count,
        stop_reason,
        created_at: ctx.timestamp,
    });

    Ok(())
}

/// Delete a recording by ID
#[reducer]
pub fn delete_recording(ctx: &ReducerContext, id: String) -> Result<(), String> {
    if ctx.db.recording().id().find(&id).is_none() {
        return Err(format!("Recording with id '{}' not found", id));
    }

    // Also delete associated transcriptions
    let transcriptions: Vec<_> = ctx
        .db
        .transcription()
        .iter()
        .filter(|t| t.recording_id == id)
        .collect();

    for t in transcriptions {
        ctx.db.transcription().id().delete(&t.id);
    }

    ctx.db.recording().id().delete(&id);
    Ok(())
}

/// Delete a recording by file path
#[reducer]
pub fn delete_recording_by_path(ctx: &ReducerContext, file_path: String) -> Result<(), String> {
    let recording = ctx
        .db
        .recording()
        .file_path()
        .find(&file_path)
        .ok_or_else(|| format!("Recording with file_path '{}' not found", file_path))?;

    // Delete associated transcriptions
    let transcriptions: Vec<_> = ctx
        .db
        .transcription()
        .iter()
        .filter(|t| t.recording_id == recording.id)
        .collect();

    for t in transcriptions {
        ctx.db.transcription().id().delete(&t.id);
    }

    ctx.db.recording().file_path().delete(&file_path);
    Ok(())
}

// =============================================================================
// Transcription Table
// =============================================================================

/// A transcription result linked to a recording
#[table(name = transcription, public)]
pub struct Transcription {
    /// Unique identifier (UUID string)
    #[primary_key]
    pub id: String,
    /// ID of the associated recording
    #[index(btree)]
    pub recording_id: String,
    /// The transcribed text
    pub text: String,
    /// Detected language (if available)
    pub language: Option<String>,
    /// Version of the transcription model used
    pub model_version: String,
    /// Time taken to transcribe in milliseconds
    pub duration_ms: u64,
    /// When the transcription was created
    pub created_at: Timestamp,
}

/// Add a new transcription
#[reducer]
pub fn add_transcription(
    ctx: &ReducerContext,
    id: String,
    recording_id: String,
    text: String,
    language: Option<String>,
    model_version: String,
    duration_ms: u64,
) -> Result<(), String> {
    // Verify the recording exists
    if ctx.db.recording().id().find(&recording_id).is_none() {
        return Err(format!(
            "Recording with id '{}' not found",
            recording_id
        ));
    }

    ctx.db.transcription().insert(Transcription {
        id,
        recording_id,
        text,
        language,
        model_version,
        duration_ms,
        created_at: ctx.timestamp,
    });

    Ok(())
}

/// Delete a transcription by ID
#[reducer]
pub fn delete_transcription(ctx: &ReducerContext, id: String) -> Result<(), String> {
    if ctx.db.transcription().id().find(&id).is_none() {
        return Err(format!("Transcription with id '{}' not found", id));
    }

    ctx.db.transcription().id().delete(&id);
    Ok(())
}

// =============================================================================
// Voice Command Table
// =============================================================================

/// A voice command definition
#[table(name = voice_command, public)]
pub struct VoiceCommand {
    /// Unique identifier (UUID string)
    #[primary_key]
    pub id: String,
    /// Trigger phrase (e.g., "open slack")
    #[unique]
    pub trigger: String,
    /// Type of action: "open_app", "type_text", "system_control", "custom"
    pub action_type: String,
    /// JSON-encoded parameters HashMap<String, String>
    pub parameters_json: String,
    /// Whether the command is enabled
    pub enabled: bool,
    /// When the command was created
    pub created_at: Timestamp,
}

/// Add a new voice command
#[reducer]
pub fn add_voice_command(
    ctx: &ReducerContext,
    id: String,
    trigger: String,
    action_type: String,
    parameters_json: String,
    enabled: bool,
) -> Result<(), String> {
    // Check for duplicate trigger
    if ctx.db.voice_command().trigger().find(&trigger).is_some() {
        return Err(format!(
            "Voice command with trigger '{}' already exists",
            trigger
        ));
    }

    ctx.db.voice_command().insert(VoiceCommand {
        id,
        trigger,
        action_type,
        parameters_json,
        enabled,
        created_at: ctx.timestamp,
    });

    Ok(())
}

/// Update an existing voice command
#[reducer]
pub fn update_voice_command(
    ctx: &ReducerContext,
    id: String,
    trigger: String,
    action_type: String,
    parameters_json: String,
    enabled: bool,
) -> Result<(), String> {
    let entry = ctx
        .db
        .voice_command()
        .id()
        .find(&id)
        .ok_or_else(|| format!("Voice command with id '{}' not found", id))?;

    // Check if new trigger conflicts with another command
    if let Some(existing) = ctx.db.voice_command().trigger().find(&trigger) {
        if existing.id != id {
            return Err(format!(
                "Voice command with trigger '{}' already exists",
                trigger
            ));
        }
    }

    ctx.db.voice_command().id().delete(&id);
    ctx.db.voice_command().insert(VoiceCommand {
        id,
        trigger,
        action_type,
        parameters_json,
        enabled,
        created_at: entry.created_at,
    });

    Ok(())
}

/// Delete a voice command by ID
#[reducer]
pub fn delete_voice_command(ctx: &ReducerContext, id: String) -> Result<(), String> {
    if ctx.db.voice_command().id().find(&id).is_none() {
        return Err(format!("Voice command with id '{}' not found", id));
    }

    ctx.db.voice_command().id().delete(&id);
    Ok(())
}
