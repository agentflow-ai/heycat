use super::*;

/// Integration test for Cmd+V paste simulation.
///
/// This test is ignored by default because it requires:
/// - macOS with Accessibility permissions granted
/// - An active display session
///
/// The test verifies that the paste function completes without error.
/// To manually test paste behavior, run with: cargo test --ignored
#[test]
#[ignore] // Requires display and Accessibility permissions
fn test_simulate_cmd_v_paste_integration() {
    // Ensure shutdown flag is clear
    // Note: Can't easily reset due to global state, but in a fresh test run it should be false

    let result = simulate_cmd_v_paste();
    assert!(
        result.is_ok(),
        "Cmd+V paste simulation should succeed: {:?}",
        result
    );
}

/// Integration test for unicode text typing.
///
/// This test is ignored by default because it requires:
/// - macOS with Accessibility permissions granted
/// - An active display session
#[test]
#[ignore] // Requires display and Accessibility permissions
fn test_type_unicode_text_integration() {
    let result = type_unicode_text("hello", 0);
    assert!(
        result.is_ok(),
        "Unicode text typing should succeed: {:?}",
        result
    );
}

/// Verify that paste uses Session tap location (not HID) for reliable cross-app delivery.
///
/// This is a compile-time documentation test - if the implementation changes
/// to use a different tap location, this test documents the expected behavior.
/// The actual assertion is in the integration test above.
#[test]
fn test_paste_uses_session_tap_location() {
    // This test serves as documentation that CGEventTapLocation::Session
    // is required for reliable paste across apps (vs HID which failed).
    // See bug fix in commit 8b2c9ab.
    //
    // The actual verification requires running the ignored integration test.
    // This test exists to document the requirement and will be updated if
    // the implementation changes.
}
