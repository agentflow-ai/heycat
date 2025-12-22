// Tests for DictionaryStore
// Test cases from spec:
// - Complete CRUD workflow: add entry, list it, update it, delete it, verify removed
// - Update/delete on non-existent ID returns error
// - Entries persist across store reload (save/load cycle)
// - Deserialize entry with suffix and auto_enter → fields populated correctly
// - Deserialize entry without suffix/auto_enter → defaults to None/false
// - Serialize entry with suffix → JSON includes suffix field
// - Round-trip serialization preserves all fields

use super::*;
use tempfile::TempDir;

/// Helper to create a store with a temporary config path
fn create_test_store() -> (DictionaryStore, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("dictionary.json");
    let store = DictionaryStore::new(config_path);
    (store, temp_dir)
}

#[test]
fn test_complete_crud_workflow() {
    let (mut store, _temp_dir) = create_test_store();

    // Add entry with new fields
    let entry = store
        .add(
            "brb".to_string(),
            "be right back".to_string(),
            Some(".".to_string()),
            true,
        )
        .unwrap();
    assert_eq!(entry.trigger, "brb");
    assert_eq!(entry.expansion, "be right back");
    assert_eq!(entry.suffix, Some(".".to_string()));
    assert!(entry.auto_enter);
    assert!(!entry.id.is_empty());

    // List shows the entry
    let entries = store.list();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].trigger, "brb");

    // Update entry
    let updated = store
        .update(
            entry.id.clone(),
            "brb".to_string(),
            "be right back!".to_string(),
            None,
            false,
        )
        .unwrap();
    assert_eq!(updated.expansion, "be right back!");
    assert_eq!(updated.suffix, None);
    assert!(!updated.auto_enter);

    // Verify update persisted
    let fetched = store.get(&entry.id).unwrap();
    assert_eq!(fetched.expansion, "be right back!");

    // Delete entry
    store.delete(&entry.id).unwrap();

    // Verify removed
    assert!(store.get(&entry.id).is_none());
    assert!(store.list().is_empty());
}

#[test]
fn test_update_nonexistent_returns_error() {
    let (mut store, _temp_dir) = create_test_store();

    let result = store.update(
        "nonexistent-id".to_string(),
        "test".to_string(),
        "test expansion".to_string(),
        None,
        false,
    );

    assert!(matches!(result, Err(DictionaryError::NotFound(_))));
}

#[test]
fn test_delete_nonexistent_returns_error() {
    let (mut store, _temp_dir) = create_test_store();

    let result = store.delete("nonexistent-id");

    assert!(matches!(result, Err(DictionaryError::NotFound(_))));
}

#[test]
fn test_entries_persist_across_reload() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("dictionary.json");

    // Create store and add entries with new fields
    let entry_id = {
        let mut store = DictionaryStore::new(config_path.clone());
        let entry1 = store
            .add(
                "brb".to_string(),
                "be right back".to_string(),
                Some(".".to_string()),
                true,
            )
            .unwrap();
        let _entry2 = store
            .add("api".to_string(), "API".to_string(), None, false)
            .unwrap();
        entry1.id
    };

    // Create new store instance and load
    let mut store2 = DictionaryStore::new(config_path);
    store2.load().unwrap();

    // Verify entries were loaded
    let entries = store2.list();
    assert_eq!(entries.len(), 2);

    // Verify specific entry exists with new fields preserved
    let loaded_entry = store2.get(&entry_id).unwrap();
    assert_eq!(loaded_entry.trigger, "brb");
    assert_eq!(loaded_entry.expansion, "be right back");
    assert_eq!(loaded_entry.suffix, Some(".".to_string()));
    assert!(loaded_entry.auto_enter);
}

#[test]
fn test_entry_serialization_with_new_fields() {
    let entry = DictionaryEntry {
        id: "123".to_string(),
        trigger: "brb".to_string(),
        expansion: "be right back".to_string(),
        suffix: Some(".".to_string()),
        auto_enter: true,
    };

    let json = serde_json::to_string(&entry).unwrap();
    let parsed: DictionaryEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.suffix, Some(".".to_string()));
    assert!(parsed.auto_enter);
}

#[test]
fn test_backward_compatible_deserialization() {
    // Old format without new fields
    let json = r#"{"id":"123","trigger":"brb","expansion":"be right back"}"#;
    let entry: DictionaryEntry = serde_json::from_str(json).unwrap();

    assert_eq!(entry.suffix, None);
    assert!(!entry.auto_enter);
}

#[test]
fn test_serialize_entry_with_suffix() {
    let entry = DictionaryEntry {
        id: "123".to_string(),
        trigger: "brb".to_string(),
        expansion: "be right back".to_string(),
        suffix: Some(".".to_string()),
        auto_enter: false,
    };

    let json = serde_json::to_string(&entry).unwrap();

    // Verify JSON includes suffix field
    assert!(json.contains(r#""suffix":".""#));
}

#[test]
fn test_roundtrip_serialization_preserves_all_fields() {
    let original = DictionaryEntry {
        id: "test-id".to_string(),
        trigger: "ty".to_string(),
        expansion: "thank you".to_string(),
        suffix: Some("!".to_string()),
        auto_enter: true,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&original).unwrap();
    // Deserialize back
    let parsed: DictionaryEntry = serde_json::from_str(&json).unwrap();

    // All fields should be preserved
    assert_eq!(parsed.id, original.id);
    assert_eq!(parsed.trigger, original.trigger);
    assert_eq!(parsed.expansion, original.expansion);
    assert_eq!(parsed.suffix, original.suffix);
    assert_eq!(parsed.auto_enter, original.auto_enter);
}
