use super::*;

// Tests removed per docs/TESTING.md:
// - test_audio_input_device_struct_serializes_correctly: Serialization derives
// - test_audio_input_device_clone: Type system guarantee
// - test_audio_input_device_debug: Debug trait derives
// - test_list_input_devices_returns_vec: Always-true assertion

#[test]
fn test_list_devices_default_first() {
    // Create a mock list and verify sorting logic
    let mut devices = vec![
        AudioInputDevice {
            name: "Device A".to_string(),
            is_default: false,
        },
        AudioInputDevice {
            name: "Device B".to_string(),
            is_default: true,
        },
        AudioInputDevice {
            name: "Device C".to_string(),
            is_default: false,
        },
    ];

    // Apply the same sorting logic used in list_input_devices
    devices.sort_by(|a, b| b.is_default.cmp(&a.is_default));

    // Default device should be first
    assert!(devices[0].is_default);
    assert_eq!(devices[0].name, "Device B");
}

#[test]
fn test_list_input_devices_via_swift() {
    // Test that we can call the Swift function and get a valid result
    let devices = list_input_devices();
    // Should return a vector (may be empty if no devices)
    // If devices exist with a default, default should be first
    if !devices.is_empty() && devices.iter().any(|d| d.is_default) {
        assert!(devices[0].is_default, "Default device should be first");
    }
}
