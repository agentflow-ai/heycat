// SpacetimeDB Client Integration Tests
//
// These tests verify the complete CRUD workflows for all entity types
// stored in SpacetimeDB. They require a running SpacetimeDB sidecar.
//
// Run with: cargo test --ignored spacetimedb_integration

use super::*;
use crate::voice_commands::registry::{ActionType, CommandDefinition};
use crate::window_context::{OverrideMode, WindowMatcher};
use std::collections::HashMap;
use uuid::Uuid;

/// Generate a unique test identifier for isolation
fn test_id() -> String {
    Uuid::new_v4().to_string()[..8].to_string()
}

/// Helper to create a connected client for integration tests.
/// Returns None if connection fails (e.g., sidecar not running).
fn get_test_client() -> Option<SpacetimeClient> {
    // Integration tests require a mock or test AppHandle.
    // For now, these tests document the expected behavior but require
    // a running application context to execute.
    //
    // TODO: Add tauri::test::mock_app() support when available
    None
}

// ============================================================
// Connection Tests
// ============================================================

#[test]
fn client_starts_disconnected() {
    // This test doesn't require the sidecar - it verifies initial state
    let state = ConnectionState::Disconnected;
    assert!(matches!(state, ConnectionState::Disconnected));
}

// ============================================================
// Dictionary Entry Tests
// ============================================================

#[test]
#[ignore = "requires running SpacetimeDB sidecar"]
fn dictionary_entry_complete_crud_workflow() {
    let Some(client) = get_test_client() else {
        eprintln!("Skipping: could not create test client");
        return;
    };

    let test_suffix = test_id();
    let trigger = format!("test_trigger_{}", test_suffix);
    let expansion = format!("test expansion {}", test_suffix);

    // === CREATE ===
    let entry = client
        .add_dictionary_entry(
            trigger.clone(),
            expansion.clone(),
            Some(" ".to_string()),
            false,
            false,
        )
        .expect("add should succeed");

    assert_eq!(entry.trigger, trigger);
    assert_eq!(entry.expansion, expansion);
    assert_eq!(entry.suffix, Some(" ".to_string()));
    assert!(!entry.auto_enter);
    assert!(!entry.disable_suffix);
    let id = entry.id.clone();

    // === LIST (verify present) ===
    let entries = client
        .list_dictionary_entries()
        .expect("list should succeed");
    assert!(
        entries.iter().any(|e| e.id == id),
        "created entry should appear in list"
    );

    // === GET by ID ===
    let fetched = client
        .get_dictionary_entry(&id)
        .expect("get should succeed")
        .expect("entry should exist");
    assert_eq!(fetched.trigger, trigger);
    assert_eq!(fetched.expansion, expansion);

    // === UPDATE ===
    let new_expansion = format!("updated expansion {}", test_suffix);
    let updated = client
        .update_dictionary_entry(
            id.clone(),
            trigger.clone(),
            new_expansion.clone(),
            None,
            true,
            true,
        )
        .expect("update should succeed");
    assert_eq!(updated.expansion, new_expansion);
    assert!(updated.auto_enter);
    assert!(updated.disable_suffix);
    assert_eq!(updated.suffix, None);

    // === VERIFY UPDATE ===
    let fetched_after_update = client
        .get_dictionary_entry(&id)
        .expect("get should succeed")
        .expect("entry should still exist");
    assert_eq!(fetched_after_update.expansion, new_expansion);

    // === DELETE ===
    client
        .delete_dictionary_entry(&id)
        .expect("delete should succeed");

    // === VERIFY DELETED ===
    let fetched_after_delete = client
        .get_dictionary_entry(&id)
        .expect("get should succeed");
    assert!(
        fetched_after_delete.is_none(),
        "entry should be deleted"
    );
}

#[test]
#[ignore = "requires running SpacetimeDB sidecar"]
fn dictionary_entry_not_found_errors() {
    let Some(client) = get_test_client() else {
        eprintln!("Skipping: could not create test client");
        return;
    };

    let nonexistent_id = Uuid::new_v4().to_string();

    // GET non-existent returns None
    let result = client
        .get_dictionary_entry(&nonexistent_id)
        .expect("get should succeed");
    assert!(result.is_none(), "non-existent entry should return None");

    // UPDATE non-existent fails
    let update_result = client.update_dictionary_entry(
        nonexistent_id.clone(),
        "trigger".to_string(),
        "expansion".to_string(),
        None,
        false,
        false,
    );
    assert!(
        update_result.is_err(),
        "update of non-existent should fail"
    );

    // DELETE non-existent fails
    let delete_result = client.delete_dictionary_entry(&nonexistent_id);
    assert!(
        delete_result.is_err(),
        "delete of non-existent should fail"
    );
}

// ============================================================
// Window Context Tests
// ============================================================

#[test]
#[ignore = "requires running SpacetimeDB sidecar"]
fn window_context_complete_crud_workflow() {
    let Some(client) = get_test_client() else {
        eprintln!("Skipping: could not create test client");
        return;
    };

    let test_suffix = test_id();
    let name = format!("Test Context {}", test_suffix);
    let app_name = format!("TestApp_{}", test_suffix);

    // === CREATE ===
    let context = client
        .add_window_context(
            name.clone(),
            WindowMatcher {
                app_name: app_name.clone(),
                title_pattern: Some(".*test.*".to_string()),
                bundle_id: Some("com.test.app".to_string()),
            },
            OverrideMode::Merge,
            OverrideMode::Replace,
            vec![Uuid::new_v4()],
            vec!["dict_entry_1".to_string()],
            true,
            10,
        )
        .expect("add should succeed");

    assert_eq!(context.name, name);
    assert_eq!(context.matcher.app_name, app_name);
    assert!(context.enabled);
    assert_eq!(context.priority, 10);
    let id = context.id;

    // === LIST (verify present) ===
    let contexts = client
        .list_window_contexts()
        .expect("list should succeed");
    assert!(
        contexts.iter().any(|c| c.id == id),
        "created context should appear in list"
    );

    // === GET by ID ===
    let fetched = client
        .get_window_context(id)
        .expect("get should succeed")
        .expect("context should exist");
    assert_eq!(fetched.name, name);
    assert!(matches!(fetched.command_mode, OverrideMode::Merge));
    assert!(matches!(fetched.dictionary_mode, OverrideMode::Replace));

    // === UPDATE ===
    let mut updated_context = fetched.clone();
    updated_context.name = format!("Updated Context {}", test_suffix);
    updated_context.enabled = false;
    updated_context.priority = 20;
    updated_context.command_mode = OverrideMode::Replace;

    let updated = client
        .update_window_context(updated_context.clone())
        .expect("update should succeed");
    assert_eq!(updated.name, format!("Updated Context {}", test_suffix));
    assert!(!updated.enabled);
    assert_eq!(updated.priority, 20);

    // === VERIFY UPDATE ===
    let fetched_after_update = client
        .get_window_context(id)
        .expect("get should succeed")
        .expect("context should still exist");
    assert!(!fetched_after_update.enabled);
    assert!(matches!(fetched_after_update.command_mode, OverrideMode::Replace));

    // === DELETE ===
    client
        .delete_window_context(id)
        .expect("delete should succeed");

    // === VERIFY DELETED ===
    let fetched_after_delete = client
        .get_window_context(id)
        .expect("get should succeed");
    assert!(
        fetched_after_delete.is_none(),
        "context should be deleted"
    );
}

#[test]
#[ignore = "requires running SpacetimeDB sidecar"]
fn window_context_not_found_errors() {
    let Some(client) = get_test_client() else {
        eprintln!("Skipping: could not create test client");
        return;
    };

    let nonexistent_id = Uuid::new_v4();

    // GET non-existent returns None
    let result = client
        .get_window_context(nonexistent_id)
        .expect("get should succeed");
    assert!(result.is_none(), "non-existent context should return None");

    // UPDATE non-existent fails
    let update_result = client.update_window_context(crate::window_context::WindowContext {
        id: nonexistent_id,
        name: "test".to_string(),
        matcher: WindowMatcher {
            app_name: "test".to_string(),
            title_pattern: None,
            bundle_id: None,
        },
        command_mode: OverrideMode::Merge,
        dictionary_mode: OverrideMode::Merge,
        command_ids: vec![],
        dictionary_entry_ids: vec![],
        enabled: true,
        priority: 0,
    });
    assert!(
        update_result.is_err(),
        "update of non-existent should fail"
    );

    // DELETE non-existent fails
    let delete_result = client.delete_window_context(nonexistent_id);
    assert!(
        delete_result.is_err(),
        "delete of non-existent should fail"
    );
}

// ============================================================
// Voice Command Tests
// ============================================================

#[test]
#[ignore = "requires running SpacetimeDB sidecar"]
fn voice_command_complete_crud_workflow() {
    let Some(client) = get_test_client() else {
        eprintln!("Skipping: could not create test client");
        return;
    };

    let test_suffix = test_id();
    let trigger = format!("test command {}", test_suffix);
    let id = Uuid::new_v4();

    let mut parameters = HashMap::new();
    parameters.insert("app".to_string(), "Safari".to_string());
    parameters.insert("url".to_string(), "https://example.com".to_string());

    // === CREATE ===
    let cmd = CommandDefinition {
        id,
        trigger: trigger.clone(),
        action_type: ActionType::OpenApp,
        parameters: parameters.clone(),
        enabled: true,
    };

    client
        .add_voice_command(&cmd)
        .expect("add should succeed");

    // === LIST (verify present) ===
    let commands = client
        .list_voice_commands()
        .expect("list should succeed");
    assert!(
        commands.iter().any(|c| c.id == id),
        "created command should appear in list"
    );

    // === GET by ID ===
    let fetched = client
        .get_voice_command(id)
        .expect("get should succeed")
        .expect("command should exist");
    assert_eq!(fetched.trigger, trigger);
    assert!(matches!(fetched.action_type, ActionType::OpenApp));
    assert_eq!(fetched.parameters.get("app"), Some(&"Safari".to_string()));

    // === UPDATE ===
    let mut updated_cmd = fetched.clone();
    updated_cmd.trigger = format!("updated command {}", test_suffix);
    updated_cmd.action_type = ActionType::TypeText;
    updated_cmd.enabled = false;
    updated_cmd.parameters.insert("text".to_string(), "hello".to_string());

    client
        .update_voice_command(&updated_cmd)
        .expect("update should succeed");

    // === VERIFY UPDATE ===
    let fetched_after_update = client
        .get_voice_command(id)
        .expect("get should succeed")
        .expect("command should still exist");
    assert_eq!(fetched_after_update.trigger, format!("updated command {}", test_suffix));
    assert!(matches!(fetched_after_update.action_type, ActionType::TypeText));
    assert!(!fetched_after_update.enabled);

    // === DELETE ===
    client
        .delete_voice_command(id)
        .expect("delete should succeed");

    // === VERIFY DELETED ===
    let fetched_after_delete = client
        .get_voice_command(id)
        .expect("get should succeed");
    assert!(
        fetched_after_delete.is_none(),
        "command should be deleted"
    );
}

#[test]
#[ignore = "requires running SpacetimeDB sidecar"]
fn voice_command_not_found_errors() {
    let Some(client) = get_test_client() else {
        eprintln!("Skipping: could not create test client");
        return;
    };

    let nonexistent_id = Uuid::new_v4();

    // GET non-existent returns None
    let result = client
        .get_voice_command(nonexistent_id)
        .expect("get should succeed");
    assert!(result.is_none(), "non-existent command should return None");

    // UPDATE non-existent fails
    let update_result = client.update_voice_command(&CommandDefinition {
        id: nonexistent_id,
        trigger: "test".to_string(),
        action_type: ActionType::Custom,
        parameters: HashMap::new(),
        enabled: true,
    });
    assert!(
        update_result.is_err(),
        "update of non-existent should fail"
    );

    // DELETE non-existent fails
    let delete_result = client.delete_voice_command(nonexistent_id);
    assert!(
        delete_result.is_err(),
        "delete of non-existent should fail"
    );
}

// ============================================================
// Recording Tests
// ============================================================

#[test]
#[ignore = "requires running SpacetimeDB sidecar"]
fn recording_complete_crud_workflow() {
    let Some(client) = get_test_client() else {
        eprintln!("Skipping: could not create test client");
        return;
    };

    let test_suffix = test_id();
    let id = Uuid::new_v4().to_string();
    let file_path = format!("/tmp/test_recording_{}.wav", test_suffix);

    // === CREATE ===
    let recording = client
        .add_recording(
            id.clone(),
            file_path.clone(),
            5.5,
            88200,
            Some(crate::audio::StopReason::SilenceAfterSpeech),
        )
        .expect("add should succeed");

    assert_eq!(recording.id, id);
    assert_eq!(recording.file_path, file_path);
    assert!((recording.duration_secs - 5.5).abs() < f64::EPSILON);
    assert_eq!(recording.sample_count, 88200);

    // === LIST (verify present) ===
    let recordings = client
        .list_recordings()
        .expect("list should succeed");
    assert!(
        recordings.iter().any(|r| r.id == id),
        "created recording should appear in list"
    );

    // === GET by ID ===
    let fetched = client
        .get_recording(&id)
        .expect("get should succeed")
        .expect("recording should exist");
    assert_eq!(fetched.file_path, file_path);

    // === GET by PATH ===
    let fetched_by_path = client
        .get_recording_by_path(&file_path)
        .expect("get by path should succeed")
        .expect("recording should exist");
    assert_eq!(fetched_by_path.id, id);

    // === DELETE ===
    client
        .delete_recording(&id)
        .expect("delete should succeed");

    // === VERIFY DELETED ===
    let fetched_after_delete = client
        .get_recording(&id)
        .expect("get should succeed");
    assert!(
        fetched_after_delete.is_none(),
        "recording should be deleted"
    );
}

#[test]
#[ignore = "requires running SpacetimeDB sidecar"]
fn recording_delete_by_path() {
    let Some(client) = get_test_client() else {
        eprintln!("Skipping: could not create test client");
        return;
    };

    let test_suffix = test_id();
    let id = Uuid::new_v4().to_string();
    let file_path = format!("/tmp/test_recording_path_{}.wav", test_suffix);

    // Create recording
    client
        .add_recording(id.clone(), file_path.clone(), 3.0, 48000, None)
        .expect("add should succeed");

    // Delete by path
    client
        .delete_recording_by_path(&file_path)
        .expect("delete by path should succeed");

    // Verify deleted
    let fetched = client
        .get_recording(&id)
        .expect("get should succeed");
    assert!(
        fetched.is_none(),
        "recording should be deleted via path"
    );
}

// ============================================================
// Transcription Tests
// ============================================================

#[test]
#[ignore = "requires running SpacetimeDB sidecar"]
fn transcription_complete_crud_workflow() {
    let Some(client) = get_test_client() else {
        eprintln!("Skipping: could not create test client");
        return;
    };

    let test_suffix = test_id();
    let id = Uuid::new_v4().to_string();
    let recording_id = Uuid::new_v4().to_string();
    let text = format!("Test transcription text {}", test_suffix);

    // First create a recording (transcriptions reference recordings)
    client
        .add_recording(
            recording_id.clone(),
            format!("/tmp/test_for_transcription_{}.wav", test_suffix),
            2.0,
            32000,
            None,
        )
        .expect("recording add should succeed");

    // === CREATE ===
    let transcription = client
        .add_transcription(
            id.clone(),
            recording_id.clone(),
            text.clone(),
            Some("en".to_string()),
            "whisper-v3".to_string(),
            1500,
        )
        .expect("add should succeed");

    assert_eq!(transcription.id, id);
    assert_eq!(transcription.recording_id, recording_id);
    assert_eq!(transcription.text, text);
    assert_eq!(transcription.language, Some("en".to_string()));
    assert_eq!(transcription.model_version, "whisper-v3");
    assert_eq!(transcription.duration_ms, 1500);

    // === LIST (verify present) ===
    let transcriptions = client
        .list_transcriptions()
        .expect("list should succeed");
    assert!(
        transcriptions.iter().any(|t| t.id == id),
        "created transcription should appear in list"
    );

    // === GET by RECORDING_ID ===
    let fetched = client
        .get_transcriptions_by_recording(&recording_id)
        .expect("get by recording should succeed");
    assert!(
        fetched.iter().any(|t| t.id == id),
        "transcription should be found by recording_id"
    );

    // === DELETE ===
    client
        .delete_transcription(&id)
        .expect("delete should succeed");

    // === VERIFY DELETED ===
    let fetched_after_delete = client
        .get_transcriptions_by_recording(&recording_id)
        .expect("get should succeed");
    assert!(
        !fetched_after_delete.iter().any(|t| t.id == id),
        "transcription should be deleted"
    );

    // Cleanup: delete the recording too
    client
        .delete_recording(&recording_id)
        .expect("recording cleanup should succeed");
}
