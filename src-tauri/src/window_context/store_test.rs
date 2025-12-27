use super::*;
use tempfile::TempDir;

fn create_test_store() -> (WindowContextStore, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("window_contexts.json");
    let store = WindowContextStore::new(config_path);
    (store, temp_dir)
}

#[test]
fn store_loads_empty_json_file_correctly() {
    let (mut store, temp_dir) = create_test_store();
    let config_path = temp_dir.path().join("window_contexts.json");

    // Write empty JSON array
    std::fs::write(&config_path, "[]").expect("Failed to write test file");

    let result = store.load();
    assert!(result.is_ok());
    assert!(store.list().is_empty());
}

#[test]
fn store_creates_file_if_not_exists() {
    let (mut store, temp_dir) = create_test_store();

    // Add a context (this will create the file)
    let result = store.add(
        "Test Context".to_string(),
        WindowMatcher {
            app_name: "TestApp".to_string(),
            title_pattern: None,
            bundle_id: None,
        },
        OverrideMode::Merge,
        OverrideMode::Merge,
        vec![],
        vec![],
        true,
        0,
    );

    assert!(result.is_ok());
    let config_path = temp_dir.path().join("window_contexts.json");
    assert!(config_path.exists());
}

#[test]
fn add_generates_unique_uuid_and_persists() {
    let (mut store, _temp_dir) = create_test_store();

    let context1 = store
        .add(
            "Context 1".to_string(),
            WindowMatcher {
                app_name: "App1".to_string(),
                title_pattern: None,
                bundle_id: None,
            },
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            0,
        )
        .expect("add should succeed");

    let context2 = store
        .add(
            "Context 2".to_string(),
            WindowMatcher {
                app_name: "App2".to_string(),
                title_pattern: None,
                bundle_id: None,
            },
            OverrideMode::Replace,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            0,
        )
        .expect("add should succeed");

    // UUIDs should be unique
    assert_ne!(context1.id, context2.id);

    // Both should be in the store
    assert_eq!(store.list().len(), 2);
}

#[test]
fn update_modifies_existing_and_persists() {
    let (mut store, _temp_dir) = create_test_store();

    let context = store
        .add(
            "Original Name".to_string(),
            WindowMatcher {
                app_name: "App".to_string(),
                title_pattern: None,
                bundle_id: None,
            },
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            0,
        )
        .expect("add should succeed");

    let updated = WindowContext {
        id: context.id,
        name: "Updated Name".to_string(),
        matcher: WindowMatcher {
            app_name: "UpdatedApp".to_string(),
            title_pattern: Some(".*test.*".to_string()),
            bundle_id: None,
        },
        command_mode: OverrideMode::Replace,
        dictionary_mode: OverrideMode::Replace,
        command_ids: vec![],
        dictionary_entry_ids: vec![],
        enabled: false,
        priority: 10,
    };

    let result = store.update(updated.clone());
    assert!(result.is_ok());

    let stored = store.get(context.id).expect("context should exist");
    assert_eq!(stored.name, "Updated Name");
    assert_eq!(stored.matcher.app_name, "UpdatedApp");
    assert_eq!(stored.priority, 10);
    assert!(!stored.enabled);
}

#[test]
fn delete_removes_and_persists() {
    let (mut store, _temp_dir) = create_test_store();

    let context = store
        .add(
            "To Delete".to_string(),
            WindowMatcher {
                app_name: "App".to_string(),
                title_pattern: None,
                bundle_id: None,
            },
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            0,
        )
        .expect("add should succeed");

    assert_eq!(store.list().len(), 1);

    let result = store.delete(context.id);
    assert!(result.is_ok());
    assert!(store.list().is_empty());
    assert!(store.get(context.id).is_none());
}

#[test]
fn invalid_regex_pattern_returns_validation_error() {
    let (mut store, _temp_dir) = create_test_store();

    let result = store.add(
        "Invalid Pattern".to_string(),
        WindowMatcher {
            app_name: "App".to_string(),
            title_pattern: Some("[invalid".to_string()), // Invalid regex
            bundle_id: None,
        },
        OverrideMode::Merge,
        OverrideMode::Merge,
        vec![],
        vec![],
        true,
        0,
    );

    assert!(result.is_err());
    match result {
        Err(WindowContextStoreError::InvalidPattern(_)) => {}
        _ => panic!("Expected InvalidPattern error"),
    }
}

#[test]
fn find_matching_context_returns_highest_priority_match() {
    let (mut store, _temp_dir) = create_test_store();

    let _low = store
        .add(
            "Low Priority".to_string(),
            WindowMatcher {
                app_name: "Slack".to_string(),
                title_pattern: None,
                bundle_id: None,
            },
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            1,
        )
        .expect("add should succeed");

    let high = store
        .add(
            "High Priority".to_string(),
            WindowMatcher {
                app_name: "Slack".to_string(),
                title_pattern: None,
                bundle_id: None,
            },
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            10,
        )
        .expect("add should succeed");

    let window = ActiveWindowInfo {
        app_name: "Slack".to_string(),
        bundle_id: Some("com.tinyspeck.slackmacgap".to_string()),
        window_title: Some("#general".to_string()),
        pid: 12345,
    };

    let matched = store.find_matching_context(&window);
    assert!(matched.is_some());
    assert_eq!(matched.unwrap().id, high.id);
}

#[test]
fn find_matching_context_matches_app_name_case_insensitively() {
    let (mut store, _temp_dir) = create_test_store();

    let ctx = store
        .add(
            "Slack Context".to_string(),
            WindowMatcher {
                app_name: "SLACK".to_string(), // Uppercase
                title_pattern: None,
                bundle_id: None,
            },
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            0,
        )
        .expect("add should succeed");

    let window = ActiveWindowInfo {
        app_name: "Slack".to_string(), // Mixed case
        bundle_id: None,
        window_title: None,
        pid: 12345,
    };

    let matched = store.find_matching_context(&window);
    assert!(matched.is_some());
    assert_eq!(matched.unwrap().id, ctx.id);
}

#[test]
fn find_matching_context_applies_title_pattern_regex() {
    let (mut store, _temp_dir) = create_test_store();

    let _general = store
        .add(
            "General Channel".to_string(),
            WindowMatcher {
                app_name: "Slack".to_string(),
                title_pattern: Some(".*#general.*".to_string()),
                bundle_id: None,
            },
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            10,
        )
        .expect("add should succeed");

    let random = store
        .add(
            "Random Channel".to_string(),
            WindowMatcher {
                app_name: "Slack".to_string(),
                title_pattern: Some(".*#random.*".to_string()),
                bundle_id: None,
            },
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            10,
        )
        .expect("add should succeed");

    // Window showing #random channel
    let window = ActiveWindowInfo {
        app_name: "Slack".to_string(),
        bundle_id: None,
        window_title: Some("Team - #random | Slack".to_string()),
        pid: 12345,
    };

    let matched = store.find_matching_context(&window);
    assert!(matched.is_some());
    assert_eq!(matched.unwrap().id, random.id);
}

#[test]
fn find_matching_context_ignores_disabled_contexts() {
    let (mut store, _temp_dir) = create_test_store();

    let _disabled = store
        .add(
            "Disabled Context".to_string(),
            WindowMatcher {
                app_name: "App".to_string(),
                title_pattern: None,
                bundle_id: None,
            },
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            false, // Disabled
            100,   // High priority
        )
        .expect("add should succeed");

    let window = ActiveWindowInfo {
        app_name: "App".to_string(),
        bundle_id: None,
        window_title: None,
        pid: 12345,
    };

    let matched = store.find_matching_context(&window);
    assert!(matched.is_none());
}

#[test]
fn store_round_trips_through_load_save() {
    let (mut store, temp_dir) = create_test_store();

    let ctx = store
        .add(
            "Persisted Context".to_string(),
            WindowMatcher {
                app_name: "TestApp".to_string(),
                title_pattern: Some(".*test.*".to_string()),
                bundle_id: Some("com.example.test".to_string()),
            },
            OverrideMode::Replace,
            OverrideMode::Merge,
            vec![Uuid::new_v4()],
            vec!["dict-1".to_string()],
            true,
            42,
        )
        .expect("add should succeed");

    // Create a new store and load from the same file
    let config_path = temp_dir.path().join("window_contexts.json");
    let mut new_store = WindowContextStore::new(config_path);
    new_store.load().expect("load should succeed");

    let loaded = new_store.get(ctx.id).expect("context should exist");
    assert_eq!(loaded.name, "Persisted Context");
    assert_eq!(loaded.matcher.app_name, "TestApp");
    assert_eq!(loaded.matcher.title_pattern, Some(".*test.*".to_string()));
    assert_eq!(loaded.command_mode, OverrideMode::Replace);
    assert_eq!(loaded.priority, 42);
}
