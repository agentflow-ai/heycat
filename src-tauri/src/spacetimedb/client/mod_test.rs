use super::*;

#[test]
fn test_connection_state() {
    let state = ConnectionState::Disconnected;
    assert!(matches!(state, ConnectionState::Disconnected));

    let state = ConnectionState::Connected;
    assert!(matches!(state, ConnectionState::Connected));

    let state = ConnectionState::Failed("test error".to_string());
    assert!(matches!(state, ConnectionState::Failed(_)));
}

#[test]
fn test_websocket_url_format() {
    let config = SidecarConfig::new(None);
    assert_eq!(config.websocket_url(), "ws://127.0.0.1:3055");
}

#[test]
fn test_action_type_conversion() {
    assert_eq!(action_type_to_string(&ActionType::OpenApp), "open_app");
    assert_eq!(action_type_to_string(&ActionType::TypeText), "type_text");
    assert_eq!(action_type_to_string(&ActionType::SystemControl), "system_control");
    assert_eq!(action_type_to_string(&ActionType::Custom), "custom");

    assert!(matches!(string_to_action_type("open_app"), ActionType::OpenApp));
    assert!(matches!(string_to_action_type("type_text"), ActionType::TypeText));
    assert!(matches!(string_to_action_type("system_control"), ActionType::SystemControl));
    assert!(matches!(string_to_action_type("custom"), ActionType::Custom));
    assert!(matches!(string_to_action_type("unknown"), ActionType::Custom));
}
