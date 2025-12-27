use super::*;

#[test]
fn window_context_round_trips_through_json_serialization() {
    let context = WindowContext {
        id: Uuid::new_v4(),
        name: "Slack".to_string(),
        matcher: WindowMatcher {
            app_name: "Slack".to_string(),
            title_pattern: Some(".*#general.*".to_string()),
            bundle_id: Some("com.tinyspeck.slackmacgap".to_string()),
        },
        command_mode: OverrideMode::Replace,
        dictionary_mode: OverrideMode::Merge,
        command_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
        dictionary_entry_ids: vec!["dict1".to_string(), "dict2".to_string()],
        enabled: true,
        priority: 10,
    };

    let json = serde_json::to_string(&context).expect("serialization should succeed");
    let deserialized: WindowContext =
        serde_json::from_str(&json).expect("deserialization should succeed");

    assert_eq!(context, deserialized);
}

#[test]
fn override_mode_defaults_to_merge() {
    assert_eq!(OverrideMode::default(), OverrideMode::Merge);
}

#[test]
fn window_matcher_with_none_fields_serializes_correctly() {
    let matcher = WindowMatcher {
        app_name: "Visual Studio Code".to_string(),
        title_pattern: None,
        bundle_id: None,
    };

    let json = serde_json::to_string(&matcher).expect("serialization should succeed");
    let deserialized: WindowMatcher =
        serde_json::from_str(&json).expect("deserialization should succeed");

    assert_eq!(matcher, deserialized);
    assert!(json.contains("\"appName\":\"Visual Studio Code\""));
}
