// Tests for voice command registry types and validation
//
// Note: CRUD operations are tested via Turso integration tests.
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

