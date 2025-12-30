// Keyboard simulator tests
//
// Note: Actual keypress simulation requires system permissions (Accessibility on macOS)
// and an active display, so we mark integration tests with #[ignore].

use super::*;

#[test]
#[ignore] // Requires display and keyboard permissions
fn test_enter_keypress_integration() {
    let mut simulator = KeyboardSimulator::new().expect("Failed to create simulator");
    let result = simulator.simulate_enter_keypress();
    assert!(result.is_ok(), "Enter keypress should succeed: {:?}", result);
}
