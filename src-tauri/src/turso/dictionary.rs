// DictionaryEntry CRUD operations using Turso/libsql
//
// Provides database operations for dictionary entries using SQL queries.

use libsql::params;
use uuid::Uuid;

use super::client::{TursoClient, TursoError};
use crate::dictionary::{DictionaryEntry, DictionaryError};

impl TursoClient {
    /// Add a new dictionary entry.
    ///
    /// Generates a UUID for the entry, validates trigger uniqueness,
    /// and inserts into the database.
    ///
    /// # Arguments
    /// * `trigger` - The trigger word/phrase
    /// * `expansion` - The expansion text
    /// * `suffix` - Optional suffix appended after expansion
    /// * `auto_enter` - Whether to simulate enter keypress
    /// * `disable_suffix` - Whether to suppress trailing punctuation
    /// * `complete_match_only` - Whether to only expand when trigger is complete input
    ///
    /// # Returns
    /// The created DictionaryEntry with generated ID
    pub async fn add_dictionary_entry(
        &self,
        trigger: String,
        expansion: String,
        suffix: Option<String>,
        auto_enter: bool,
        disable_suffix: bool,
        complete_match_only: bool,
    ) -> Result<DictionaryEntry, DictionaryError> {
        let id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();

        self.execute(
            r#"INSERT INTO dictionary_entry
               (id, trigger, expansion, suffix, auto_enter, disable_suffix, complete_match_only, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            params![
                id.clone(),
                trigger.clone(),
                expansion.clone(),
                suffix.clone(),
                auto_enter as i32,
                disable_suffix as i32,
                complete_match_only as i32,
                created_at
            ],
        )
        .await
        .map_err(|e| match e {
            TursoError::Constraint(_) => DictionaryError::PersistenceError(
                format!("Trigger '{}' already exists", trigger),
            ),
            other => DictionaryError::PersistenceError(other.to_string()),
        })?;

        Ok(DictionaryEntry {
            id,
            trigger,
            expansion,
            suffix,
            auto_enter,
            disable_suffix,
            complete_match_only,
        })
    }

    /// Update an existing dictionary entry.
    ///
    /// # Arguments
    /// * `id` - The entry ID to update
    /// * `trigger` - The new trigger word/phrase
    /// * `expansion` - The new expansion text
    /// * `suffix` - Optional suffix appended after expansion
    /// * `auto_enter` - Whether to simulate enter keypress
    /// * `disable_suffix` - Whether to suppress trailing punctuation
    /// * `complete_match_only` - Whether to only expand when trigger is complete input
    ///
    /// # Returns
    /// The updated DictionaryEntry
    pub async fn update_dictionary_entry(
        &self,
        id: String,
        trigger: String,
        expansion: String,
        suffix: Option<String>,
        auto_enter: bool,
        disable_suffix: bool,
        complete_match_only: bool,
    ) -> Result<DictionaryEntry, DictionaryError> {
        // Check if entry exists
        let exists = self.dictionary_entry_exists(&id).await?;
        if !exists {
            return Err(DictionaryError::NotFound(id));
        }

        // Check for trigger conflict with other entries
        let mut rows = self
            .query(
                "SELECT id FROM dictionary_entry WHERE trigger = ?1 AND id != ?2",
                params![trigger.clone(), id.clone()],
            )
            .await
            .map_err(|e| DictionaryError::PersistenceError(e.to_string()))?;

        if rows
            .next()
            .await
            .map_err(|e| DictionaryError::PersistenceError(e.to_string()))?
            .is_some()
        {
            return Err(DictionaryError::PersistenceError(format!(
                "Trigger '{}' already exists",
                trigger
            )));
        }

        self.execute(
            r#"UPDATE dictionary_entry
               SET trigger = ?1, expansion = ?2, suffix = ?3, auto_enter = ?4, disable_suffix = ?5, complete_match_only = ?6
               WHERE id = ?7"#,
            params![
                trigger.clone(),
                expansion.clone(),
                suffix.clone(),
                auto_enter as i32,
                disable_suffix as i32,
                complete_match_only as i32,
                id.clone()
            ],
        )
        .await
        .map_err(|e| DictionaryError::PersistenceError(e.to_string()))?;

        Ok(DictionaryEntry {
            id,
            trigger,
            expansion,
            suffix,
            auto_enter,
            disable_suffix,
            complete_match_only,
        })
    }

    /// Delete a dictionary entry by ID.
    ///
    /// # Arguments
    /// * `id` - The entry ID to delete
    pub async fn delete_dictionary_entry(&self, id: &str) -> Result<(), DictionaryError> {
        // Check if entry exists
        let exists = self.dictionary_entry_exists(id).await?;
        if !exists {
            return Err(DictionaryError::NotFound(id.to_string()));
        }

        self.execute(
            "DELETE FROM dictionary_entry WHERE id = ?1",
            params![id.to_string()],
        )
        .await
        .map_err(|e| DictionaryError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    /// List all dictionary entries ordered by created_at.
    ///
    /// # Returns
    /// Vector of all dictionary entries
    pub async fn list_dictionary_entries(&self) -> Result<Vec<DictionaryEntry>, DictionaryError> {
        let mut rows = self
            .query(
                "SELECT id, trigger, expansion, suffix, auto_enter, disable_suffix, complete_match_only FROM dictionary_entry ORDER BY created_at",
                (),
            )
            .await
            .map_err(|e| DictionaryError::LoadError(e.to_string()))?;

        let mut entries = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DictionaryError::LoadError(e.to_string()))?
        {
            let id: String = row.get(0).map_err(|e| DictionaryError::LoadError(e.to_string()))?;
            let trigger: String = row.get(1).map_err(|e| DictionaryError::LoadError(e.to_string()))?;
            let expansion: String = row.get(2).map_err(|e| DictionaryError::LoadError(e.to_string()))?;
            let suffix: Option<String> = row.get(3).map_err(|e| DictionaryError::LoadError(e.to_string()))?;
            let auto_enter: i32 = row.get(4).map_err(|e| DictionaryError::LoadError(e.to_string()))?;
            let disable_suffix: i32 = row.get(5).map_err(|e| DictionaryError::LoadError(e.to_string()))?;
            let complete_match_only: i32 = row.get(6).map_err(|e| DictionaryError::LoadError(e.to_string()))?;

            entries.push(DictionaryEntry {
                id,
                trigger,
                expansion,
                suffix,
                auto_enter: auto_enter != 0,
                disable_suffix: disable_suffix != 0,
                complete_match_only: complete_match_only != 0,
            });
        }

        Ok(entries)
    }

    /// Check if a dictionary entry exists by ID.
    async fn dictionary_entry_exists(&self, id: &str) -> Result<bool, DictionaryError> {
        let mut rows = self
            .query(
                "SELECT 1 FROM dictionary_entry WHERE id = ?1",
                params![id.to_string()],
            )
            .await
            .map_err(|e| DictionaryError::PersistenceError(e.to_string()))?;

        Ok(rows
            .next()
            .await
            .map_err(|e| DictionaryError::PersistenceError(e.to_string()))?
            .is_some())
    }
}

#[cfg(test)]
#[path = "dictionary_test.rs"]
mod tests;
