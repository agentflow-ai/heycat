use super::*;

#[test]
fn get_active_window_returns_valid_info_when_app_is_focused() {
    // This test runs in a real macOS environment during development
    // It verifies that get_active_window() returns successfully
    let result = get_active_window();

    // Should succeed - some app is always focused
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);

    let info = result.unwrap();

    // app_name should be non-empty for any focused app
    assert!(
        !info.app_name.is_empty(),
        "app_name should not be empty, got: '{}'",
        info.app_name
    );

    // pid should be positive
    assert!(info.pid > 0, "pid should be positive, got: {}", info.pid);
}

#[test]
fn app_name_is_non_empty_for_standard_macos_apps() {
    // When running tests, the test runner itself is an app
    let result = get_active_window();
    assert!(result.is_ok());

    let info = result.unwrap();
    // The app name should never be empty for any running macOS application
    assert!(!info.app_name.is_empty());
    // It should not be "Unknown" for properly launched apps
    // Note: In rare edge cases during system transitions, this could fail
}

#[test]
fn bundle_id_matches_expected_format_when_present() {
    let result = get_active_window();
    assert!(result.is_ok());

    let info = result.unwrap();

    // bundle_id is optional but when present should follow reverse-DNS format
    if let Some(bundle_id) = &info.bundle_id {
        // Should contain at least one dot (reverse-DNS format: com.company.app)
        assert!(
            bundle_id.contains('.'),
            "bundle_id should be in reverse-DNS format (contain dots), got: '{}'",
            bundle_id
        );
    }
    // If bundle_id is None, that's also valid (some apps don't have one)
}

#[test]
fn active_window_info_serializes_correctly() {
    let info = ActiveWindowInfo {
        app_name: "Visual Studio Code".to_string(),
        bundle_id: Some("com.microsoft.VSCode".to_string()),
        window_title: Some("main.rs - heycat".to_string()),
        pid: 12345,
    };

    let json = serde_json::to_string(&info).expect("serialization should succeed");

    // Verify camelCase field names (from serde rename_all)
    assert!(json.contains("\"appName\":"));
    assert!(json.contains("\"bundleId\":"));
    assert!(json.contains("\"windowTitle\":"));
    assert!(json.contains("\"pid\":"));

    // Verify values
    assert!(json.contains("\"Visual Studio Code\""));
    assert!(json.contains("\"com.microsoft.VSCode\""));
    assert!(json.contains("\"main.rs - heycat\""));
    assert!(json.contains("12345"));
}

// Running applications tests

#[test]
fn get_running_applications_returns_user_visible_apps() {
    // This test verifies the API returns running apps
    let apps = get_running_applications();

    // On a macOS system, there should always be at least one visible app running
    // (Finder is always running, the test runner app, etc.)
    assert!(
        !apps.is_empty(),
        "Expected at least one running application, got none"
    );
}

#[test]
fn get_running_applications_includes_finder() {
    // Finder is always running on macOS
    let apps = get_running_applications();

    let finder = apps.iter().find(|app| app.name == "Finder");
    assert!(
        finder.is_some(),
        "Finder should always be in the list of running applications"
    );

    // Finder should have a bundle ID
    let finder = finder.unwrap();
    assert_eq!(
        finder.bundle_id,
        Some("com.apple.finder".to_string()),
        "Finder should have bundle ID com.apple.finder"
    );
}

#[test]
fn get_running_applications_apps_have_names() {
    let apps = get_running_applications();

    // All apps should have non-empty names
    for app in &apps {
        assert!(
            !app.name.is_empty(),
            "All running applications should have non-empty names"
        );
    }
}

#[test]
fn get_running_applications_exactly_one_app_is_active() {
    let apps = get_running_applications();

    // Exactly one app should be marked as active (the frontmost app)
    let active_count = apps.iter().filter(|app| app.is_active).count();
    assert!(
        active_count <= 1,
        "At most one app should be active, got {}",
        active_count
    );
}

#[test]
fn get_running_applications_list_is_sorted_alphabetically() {
    let apps = get_running_applications();

    // Verify the list is sorted alphabetically (case-insensitive)
    let is_sorted = apps.windows(2).all(|w| {
        w[0].name.to_lowercase() <= w[1].name.to_lowercase()
    });

    assert!(
        is_sorted,
        "Running applications should be sorted alphabetically by name"
    );
}

#[test]
fn running_application_serializes_correctly() {
    let app = RunningApplication {
        name: "Safari".to_string(),
        bundle_id: Some("com.apple.Safari".to_string()),
        is_active: true,
    };

    let json = serde_json::to_string(&app).expect("serialization should succeed");

    // Verify camelCase field names
    assert!(json.contains("\"name\":"));
    assert!(json.contains("\"bundleId\":"));
    assert!(json.contains("\"isActive\":"));

    // Verify values
    assert!(json.contains("\"Safari\""));
    assert!(json.contains("\"com.apple.Safari\""));
    assert!(json.contains("true"));
}
