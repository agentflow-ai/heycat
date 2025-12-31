// Database schema definitions and migration system
//
// This module defines the SQLite schema for all heycat data tables
// and provides a migration system for future schema changes.

use super::client::{TursoClient, TursoError};

/// Current schema version
const SCHEMA_VERSION: i32 = 2;

/// SQL statements to create all tables (each as a separate string)
const CREATE_TABLES: &[&str] = &[
    // Dictionary entries for text expansion
    r#"CREATE TABLE IF NOT EXISTS dictionary_entry (
        id TEXT PRIMARY KEY,
        trigger TEXT UNIQUE NOT NULL,
        expansion TEXT NOT NULL,
        suffix TEXT,
        auto_enter INTEGER NOT NULL DEFAULT 0,
        disable_suffix INTEGER NOT NULL DEFAULT 0,
        complete_match_only INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL
    )"#,
    // Window contexts for context-sensitive commands and dictionaries
    r#"CREATE TABLE IF NOT EXISTS window_context (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        matcher_app_name TEXT NOT NULL,
        matcher_title_pattern TEXT,
        matcher_bundle_id TEXT,
        command_mode TEXT NOT NULL,
        dictionary_mode TEXT NOT NULL,
        command_ids_json TEXT NOT NULL,
        dictionary_entry_ids_json TEXT NOT NULL,
        enabled INTEGER NOT NULL DEFAULT 1,
        priority INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL
    )"#,
    // Recording metadata
    r#"CREATE TABLE IF NOT EXISTS recording (
        id TEXT PRIMARY KEY,
        file_path TEXT UNIQUE NOT NULL,
        duration_secs REAL NOT NULL,
        sample_count INTEGER NOT NULL,
        stop_reason TEXT,
        created_at TEXT NOT NULL,
        active_window_app_name TEXT,
        active_window_bundle_id TEXT,
        active_window_title TEXT
    )"#,
    // Transcription results linked to recordings
    r#"CREATE TABLE IF NOT EXISTS transcription (
        id TEXT PRIMARY KEY,
        recording_id TEXT NOT NULL,
        text TEXT NOT NULL,
        language TEXT,
        model_version TEXT NOT NULL,
        duration_ms INTEGER NOT NULL,
        created_at TEXT NOT NULL,
        FOREIGN KEY (recording_id) REFERENCES recording(id) ON DELETE CASCADE
    )"#,
    // Index for efficient transcription lookups by recording
    r#"CREATE INDEX IF NOT EXISTS idx_transcription_recording_id ON transcription(recording_id)"#,
    // Voice command definitions
    r#"CREATE TABLE IF NOT EXISTS voice_command (
        id TEXT PRIMARY KEY,
        trigger TEXT UNIQUE NOT NULL,
        action_type TEXT NOT NULL,
        parameters_json TEXT NOT NULL,
        enabled INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL
    )"#,
];

/// Initialize the database schema.
///
/// Creates all tables if they don't exist and runs any pending migrations.
/// This should be called once during application startup after TursoClient is created.
pub async fn initialize_schema(client: &TursoClient) -> Result<(), TursoError> {
    // First, ensure schema_version table exists (needed for version checking)
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY)",
            (),
        )
        .await?;

    // Check current schema version
    let current_version = get_schema_version(client).await?;

    if current_version == 0 {
        // Fresh database - create all tables
        crate::info!("Initializing Turso database schema (version {})", SCHEMA_VERSION);

        // Execute each CREATE statement
        for statement in CREATE_TABLES {
            client.execute(statement, ()).await?;
        }

        // Set schema version
        set_schema_version(client, SCHEMA_VERSION).await?;

        crate::info!("Turso database schema initialized successfully");
    } else if current_version < SCHEMA_VERSION {
        // Run migrations
        crate::info!("Migrating Turso database from version {} to {}", current_version, SCHEMA_VERSION);
        run_migrations(client, current_version, SCHEMA_VERSION).await?;
        crate::info!("Turso database migration complete");
    } else {
        crate::debug!("Turso database schema is up to date (version {})", current_version);
    }

    Ok(())
}

/// Get the current schema version from the database.
/// Returns 0 if the schema_version table doesn't exist yet.
async fn get_schema_version(client: &TursoClient) -> Result<i32, TursoError> {
    // Check if schema_version table exists
    let mut rows = client
        .query(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='schema_version'",
            (),
        )
        .await?;

    if rows.next().await.map_err(|e| TursoError::Query(e.to_string()))?.is_none() {
        return Ok(0);
    }

    // Get current version
    let mut rows = client
        .query("SELECT version FROM schema_version ORDER BY version DESC LIMIT 1", ())
        .await?;

    match rows.next().await.map_err(|e| TursoError::Query(e.to_string()))? {
        Some(row) => {
            let version: i32 = row.get(0).map_err(|e| TursoError::Query(e.to_string()))?;
            Ok(version)
        }
        None => Ok(0),
    }
}

/// Set the schema version in the database.
async fn set_schema_version(client: &TursoClient, version: i32) -> Result<(), TursoError> {
    client
        .execute(
            "INSERT OR REPLACE INTO schema_version (version) VALUES (?1)",
            libsql::params![version],
        )
        .await?;
    Ok(())
}

/// Run migrations from one version to another.
/// Each migration is a function that updates the schema.
async fn run_migrations(
    client: &TursoClient,
    from_version: i32,
    to_version: i32,
) -> Result<(), TursoError> {
    for version in (from_version + 1)..=to_version {
        match version {
            2 => migrate_v1_to_v2(client).await?,
            // 3 => migrate_v2_to_v3(client).await?,
            _ => {
                // No migration needed for this version
                crate::debug!("No migration needed for version {}", version);
            }
        }
        set_schema_version(client, version).await?;
    }
    Ok(())
}

/// Migrate from schema version 1 to 2.
/// Adds complete_match_only column to dictionary_entry table.
async fn migrate_v1_to_v2(client: &TursoClient) -> Result<(), TursoError> {
    crate::info!("Running migration v1 -> v2: adding complete_match_only column to dictionary_entry");
    client
        .execute(
            "ALTER TABLE dictionary_entry ADD COLUMN complete_match_only INTEGER NOT NULL DEFAULT 0",
            (),
        )
        .await?;
    Ok(())
}

#[cfg(test)]
#[path = "schema_test.rs"]
mod tests;
