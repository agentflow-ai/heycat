// Audio device enumeration module
// Provides types and functions for listing available audio input devices

use serde::{Deserialize, Serialize};

/// Represents an audio input device with its properties
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioInputDevice {
    /// Human-readable name of the device
    pub name: String,
    /// Whether this is the system's default input device
    pub is_default: bool,
}

/// List all available audio input devices using AVFoundation via Swift.
///
/// Returns a vector of audio input devices sorted with the default device first.
/// Returns an empty vector if no devices are available or if an error occurs.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn list_input_devices() -> Vec<AudioInputDevice> {
    crate::debug!("Listing input devices via AVFoundation/Swift");

    let swift_devices = crate::swift::list_audio_devices();

    let device_list: Vec<AudioInputDevice> = swift_devices
        .into_iter()
        .map(|d| AudioInputDevice {
            name: d.name,
            is_default: d.is_default,
        })
        .collect();

    crate::debug!("Found {} input devices", device_list.len());
    device_list
}

#[cfg(test)]
#[path = "device_test.rs"]
mod tests;
