// Tests for voice command registry types and validation
//
// Note: CRUD operations are tested via SpacetimeDB integration tests.
// These tests cover type definitions, serialization, and error handling.

use super::*;
use std::collections::HashMap;

fn create_test_command(trigger: &str) -> CommandDefinition {
    CommandDefinition {
        id: Uuid::new_v4(),
        trigger: trigger.to_string(),
        action_type: ActionType::OpenApp,
        parameters: HashMap::new(),
        enabled: true,
    }
}

#[test]
fn test_action_type_from_str() {
    assert_eq!("open_app".parse::<ActionType>().unwrap(), ActionType::OpenApp);
    assert_eq!("type_text".parse::<ActionType>().unwrap(), ActionType::TypeText);
    assert_eq!("system_control".parse::<ActionType>().unwrap(), ActionType::SystemControl);
    assert_eq!("custom".parse::<ActionType>().unwrap(), ActionType::Custom);

    assert!("invalid".parse::<ActionType>().is_err());
}

#[test]
fn test_action_type_serialization() {
    let action = ActionType::OpenApp;
    let json = serde_json::to_string(&action).unwrap();
    assert_eq!(json, "\"open_app\"");

    let action = ActionType::TypeText;
    let json = serde_json::to_string(&action).unwrap();
    assert_eq!(json, "\"type_text\"");

    let action = ActionType::SystemControl;
    let json = serde_json::to_string(&action).unwrap();
    assert_eq!(json, "\"system_control\"");

    let action = ActionType::Custom;
    let json = serde_json::to_string(&action).unwrap();
    assert_eq!(json, "\"custom\"");
}

#[test]
fn test_action_type_deserialization() {
    let action: ActionType = serde_json::from_str("\"open_app\"").unwrap();
    assert_eq!(action, ActionType::OpenApp);

    let action: ActionType = serde_json::from_str("\"type_text\"").unwrap();
    assert_eq!(action, ActionType::TypeText);
}

#[test]
fn test_command_definition_serialization() {
    let cmd = create_test_command("open slack");
    let json = serde_json::to_string(&cmd).unwrap();

    assert!(json.contains("\"trigger\":\"open slack\""));
    assert!(json.contains("\"action_type\":\"open_app\""));
    assert!(json.contains("\"enabled\":true"));
}

#[test]
fn test_command_definition_deserialization() {
    let json = r#"{
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "trigger": "test command",
        "action_type": "type_text",
        "parameters": {"text": "hello world"},
        "enabled": false
    }"#;

    let cmd: CommandDefinition = serde_json::from_str(json).unwrap();
    assert_eq!(cmd.trigger, "test command");
    assert_eq!(cmd.action_type, ActionType::TypeText);
    assert_eq!(cmd.parameters.get("text"), Some(&"hello world".to_string()));
    assert!(!cmd.enabled);
}

#[test]
fn test_registry_error_display() {
    let err = RegistryError::EmptyTrigger;
    assert_eq!(err.to_string(), "Trigger phrase cannot be empty");

    let id = Uuid::new_v4();
    let err = RegistryError::DuplicateId(id);
    assert!(err.to_string().contains("already exists"));

    let err = RegistryError::NotFound(id);
    assert!(err.to_string().contains("not found"));

    let err = RegistryError::PersistenceError("test error".to_string());
    assert!(err.to_string().contains("test error"));

    let err = RegistryError::LoadError("load failed".to_string());
    assert!(err.to_string().contains("load failed"));
}

#[test]
fn test_command_definition_with_parameters() {
    let mut params = HashMap::new();
    params.insert("app_name".to_string(), "Slack".to_string());
    params.insert("bundle_id".to_string(), "com.tinyspeck.slackmacgap".to_string());

    let cmd = CommandDefinition {
        id: Uuid::new_v4(),
        trigger: "open slack".to_string(),
        action_type: ActionType::OpenApp,
        parameters: params.clone(),
        enabled: true,
    };

    // Verify parameters are stored correctly
    assert_eq!(cmd.parameters.get("app_name"), Some(&"Slack".to_string()));
    assert_eq!(cmd.parameters.get("bundle_id"), Some(&"com.tinyspeck.slackmacgap".to_string()));
    assert_eq!(cmd.parameters.len(), 2);

    // Verify serialization includes parameters
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("app_name"));
    assert!(json.contains("Slack"));
}

#[test]
fn test_command_definition_equality() {
    let id = Uuid::new_v4();
    let cmd1 = CommandDefinition {
        id,
        trigger: "test".to_string(),
        action_type: ActionType::Custom,
        parameters: HashMap::new(),
        enabled: true,
    };
    let cmd2 = CommandDefinition {
        id,
        trigger: "test".to_string(),
        action_type: ActionType::Custom,
        parameters: HashMap::new(),
        enabled: true,
    };

    assert_eq!(cmd1, cmd2);
}

#[test]
fn test_command_definition_clone() {
    let cmd = create_test_command("test clone");
    let cloned = cmd.clone();

    assert_eq!(cmd.id, cloned.id);
    assert_eq!(cmd.trigger, cloned.trigger);
    assert_eq!(cmd.action_type, cloned.action_type);
    assert_eq!(cmd.enabled, cloned.enabled);
}
