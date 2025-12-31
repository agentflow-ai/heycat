use crate::dictionary::DictionaryError;
use crate::turso::{initialize_schema, TursoClient};
use tempfile::TempDir;

async fn setup_client() -> (TursoClient, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let client = TursoClient::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create client");
    initialize_schema(&client)
        .await
        .expect("Failed to initialize schema");
    (client, temp_dir)
}

#[tokio::test]
async fn test_add_dictionary_entry() {
    let (client, _temp) = setup_client().await;

    let entry = client
        .add_dictionary_entry(
            "ty".to_string(),
            "thank you".to_string(),
            None,
            false,
            false,
            false,
        )
        .await
        .expect("Failed to add entry");

    assert!(!entry.id.is_empty(), "ID should be generated");
    assert_eq!(entry.trigger, "ty");
    assert_eq!(entry.expansion, "thank you");
    assert_eq!(entry.suffix, None);
    assert!(!entry.auto_enter);
    assert!(!entry.disable_suffix);
    assert!(!entry.complete_match_only);
}

#[tokio::test]
async fn test_add_dictionary_entry_with_all_fields() {
    let (client, _temp) = setup_client().await;

    let entry = client
        .add_dictionary_entry(
            "sig".to_string(),
            "Best regards,\nJohn".to_string(),
            Some(" ".to_string()),
            true,
            true,
            true,
        )
        .await
        .expect("Failed to add entry");

    assert_eq!(entry.trigger, "sig");
    assert_eq!(entry.expansion, "Best regards,\nJohn");
    assert_eq!(entry.suffix, Some(" ".to_string()));
    assert!(entry.auto_enter);
    assert!(entry.disable_suffix);
    assert!(entry.complete_match_only);
}

#[tokio::test]
async fn test_add_duplicate_trigger_fails() {
    let (client, _temp) = setup_client().await;

    // Add first entry
    client
        .add_dictionary_entry("dup".to_string(), "first".to_string(), None, false, false, false)
        .await
        .expect("First add should succeed");

    // Try to add with same trigger
    let result = client
        .add_dictionary_entry("dup".to_string(), "second".to_string(), None, false, false, false)
        .await;

    assert!(result.is_err(), "Duplicate trigger should fail");
    match result.err().unwrap() {
        DictionaryError::PersistenceError(msg) => {
            assert!(msg.contains("already exists"), "Error should mention exists");
        }
        other => panic!("Expected PersistenceError, got {:?}", other),
    }
}

#[tokio::test]
async fn test_list_dictionary_entries() {
    let (client, _temp) = setup_client().await;

    // Add multiple entries
    client
        .add_dictionary_entry("a".to_string(), "apple".to_string(), None, false, false, false)
        .await
        .expect("Failed to add a");
    client
        .add_dictionary_entry("b".to_string(), "banana".to_string(), None, false, false, false)
        .await
        .expect("Failed to add b");
    client
        .add_dictionary_entry("c".to_string(), "cherry".to_string(), None, false, false, false)
        .await
        .expect("Failed to add c");

    let entries = client
        .list_dictionary_entries()
        .await
        .expect("Failed to list");

    assert_eq!(entries.len(), 3);
    // Should be ordered by created_at
    assert_eq!(entries[0].trigger, "a");
    assert_eq!(entries[1].trigger, "b");
    assert_eq!(entries[2].trigger, "c");
}

#[tokio::test]
async fn test_list_empty_dictionary() {
    let (client, _temp) = setup_client().await;

    let entries = client
        .list_dictionary_entries()
        .await
        .expect("Failed to list");

    assert!(entries.is_empty());
}

#[tokio::test]
async fn test_update_dictionary_entry() {
    let (client, _temp) = setup_client().await;

    // Add entry
    let entry = client
        .add_dictionary_entry("old".to_string(), "old value".to_string(), None, false, false, false)
        .await
        .expect("Failed to add entry");

    // Update entry
    let updated = client
        .update_dictionary_entry(
            entry.id.clone(),
            "new".to_string(),
            "new value".to_string(),
            Some("!".to_string()),
            true,
            true,
            true,
        )
        .await
        .expect("Failed to update entry");

    assert_eq!(updated.id, entry.id);
    assert_eq!(updated.trigger, "new");
    assert_eq!(updated.expansion, "new value");
    assert_eq!(updated.suffix, Some("!".to_string()));
    assert!(updated.auto_enter);
    assert!(updated.disable_suffix);
    assert!(updated.complete_match_only);

    // Verify by listing
    let entries = client.list_dictionary_entries().await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].trigger, "new");
}

#[tokio::test]
async fn test_update_nonexistent_entry_fails() {
    let (client, _temp) = setup_client().await;

    let result = client
        .update_dictionary_entry(
            "nonexistent-id".to_string(),
            "trigger".to_string(),
            "expansion".to_string(),
            None,
            false,
            false,
            false,
        )
        .await;

    match result.err().unwrap() {
        DictionaryError::NotFound(id) => assert_eq!(id, "nonexistent-id"),
        other => panic!("Expected NotFound, got {:?}", other),
    }
}

#[tokio::test]
async fn test_update_trigger_conflict_fails() {
    let (client, _temp) = setup_client().await;

    // Add two entries
    let entry1 = client
        .add_dictionary_entry("first".to_string(), "one".to_string(), None, false, false, false)
        .await
        .expect("Failed to add first");
    client
        .add_dictionary_entry("second".to_string(), "two".to_string(), None, false, false, false)
        .await
        .expect("Failed to add second");

    // Try to update first to use second's trigger
    let result = client
        .update_dictionary_entry(
            entry1.id,
            "second".to_string(),
            "updated".to_string(),
            None,
            false,
            false,
            false,
        )
        .await;

    assert!(result.is_err(), "Trigger conflict should fail");
    match result.err().unwrap() {
        DictionaryError::PersistenceError(msg) => {
            assert!(msg.contains("already exists"));
        }
        other => panic!("Expected PersistenceError, got {:?}", other),
    }
}

#[tokio::test]
async fn test_delete_dictionary_entry() {
    let (client, _temp) = setup_client().await;

    // Add entry
    let entry = client
        .add_dictionary_entry("tbd".to_string(), "to be deleted".to_string(), None, false, false, false)
        .await
        .expect("Failed to add entry");

    // Delete entry
    client
        .delete_dictionary_entry(&entry.id)
        .await
        .expect("Failed to delete entry");

    // Verify it's gone
    let entries = client.list_dictionary_entries().await.unwrap();
    assert!(entries.is_empty());
}

#[tokio::test]
async fn test_delete_nonexistent_entry_fails() {
    let (client, _temp) = setup_client().await;

    let result = client.delete_dictionary_entry("nonexistent-id").await;

    match result.err().unwrap() {
        DictionaryError::NotFound(id) => assert_eq!(id, "nonexistent-id"),
        other => panic!("Expected NotFound, got {:?}", other),
    }
}
