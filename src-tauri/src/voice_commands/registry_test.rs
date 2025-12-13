use super::*;
use std::collections::HashMap;
use tempfile::TempDir;

fn test_registry() -> (CommandRegistry, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("commands.json");
    let registry = CommandRegistry::new(config_path);
    (registry, temp_dir)
}

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
fn test_add_command_and_verify_in_list() {
    let (mut registry, _temp) = test_registry();
    let cmd = create_test_command("open slack");

    registry.add(cmd.clone()).unwrap();

    let commands = registry.list();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].trigger, "open slack");
}

#[test]
fn test_update_existing_command_trigger() {
    let (mut registry, _temp) = test_registry();
    let cmd = create_test_command("open slack");
    let id = cmd.id;

    registry.add(cmd).unwrap();

    let mut updated = registry.get(id).unwrap().clone();
    updated.trigger = "launch slack".to_string();
    registry.update(updated).unwrap();

    let result = registry.get(id).unwrap();
    assert_eq!(result.trigger, "launch slack");
}

#[test]
fn test_delete_command_and_verify_removal() {
    let (mut registry, _temp) = test_registry();
    let cmd = create_test_command("open slack");
    let id = cmd.id;

    registry.add(cmd).unwrap();
    assert_eq!(registry.len(), 1);

    registry.delete(id).unwrap();
    assert_eq!(registry.len(), 0);
    assert!(registry.get(id).is_none());
}

#[test]
fn test_persist_and_reload_commands() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("commands.json");
    let cmd_id;

    // Add command and persist
    {
        let mut registry = CommandRegistry::new(config_path.clone());
        let cmd = create_test_command("open slack");
        cmd_id = cmd.id;
        registry.add(cmd).unwrap();
    }

    // Create new registry and load
    {
        let mut registry = CommandRegistry::new(config_path);
        registry.load().unwrap();

        assert_eq!(registry.len(), 1);
        let loaded = registry.get(cmd_id).unwrap();
        assert_eq!(loaded.trigger, "open slack");
    }
}

#[test]
fn test_reject_empty_trigger() {
    let (mut registry, _temp) = test_registry();
    let cmd = create_test_command("");

    let result = registry.add(cmd);
    assert!(matches!(result, Err(RegistryError::EmptyTrigger)));
}

#[test]
fn test_reject_whitespace_only_trigger() {
    let (mut registry, _temp) = test_registry();
    let cmd = create_test_command("   ");

    let result = registry.add(cmd);
    assert!(matches!(result, Err(RegistryError::EmptyTrigger)));
}

#[test]
fn test_reject_duplicate_id() {
    let (mut registry, _temp) = test_registry();
    let cmd1 = create_test_command("open slack");
    let id = cmd1.id;

    registry.add(cmd1).unwrap();

    let cmd2 = CommandDefinition {
        id,
        trigger: "open discord".to_string(),
        action_type: ActionType::OpenApp,
        parameters: HashMap::new(),
        enabled: true,
    };

    let result = registry.add(cmd2);
    assert!(matches!(result, Err(RegistryError::DuplicateId(_))));
}

#[test]
fn test_update_nonexistent_command_fails() {
    let (mut registry, _temp) = test_registry();
    let cmd = create_test_command("open slack");

    let result = registry.update(cmd);
    assert!(matches!(result, Err(RegistryError::NotFound(_))));
}

#[test]
fn test_delete_nonexistent_command_fails() {
    let (mut registry, _temp) = test_registry();
    let id = Uuid::new_v4();

    let result = registry.delete(id);
    assert!(matches!(result, Err(RegistryError::NotFound(_))));
}

#[test]
fn test_action_type_serialization() {
    let action = ActionType::OpenApp;
    let json = serde_json::to_string(&action).unwrap();
    assert_eq!(json, "\"open_app\"");

    let action = ActionType::TypeText;
    let json = serde_json::to_string(&action).unwrap();
    assert_eq!(json, "\"type_text\"");
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
fn test_load_nonexistent_file_returns_empty() {
    let (mut registry, _temp) = test_registry();
    registry.load().unwrap();
    assert!(registry.is_empty());
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
}
