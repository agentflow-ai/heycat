// Accessibility permission handling for macOS
// CGEventTap requires Accessibility permission for full key capture including fn keys
// This module provides functions to check and guide users to enable the permission

use std::process::Command;

// FFI bindings for Accessibility permission checking
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    /// Check if the current process has Accessibility permission
    /// Returns true if the app is trusted (has Accessibility permission)
    fn AXIsProcessTrusted() -> bool;

    /// Check if the current process has Accessibility permission, with option to prompt
    /// If kAXTrustedCheckOptionPrompt is set to true, shows a dialog prompting the user
    /// to add the app to the Accessibility list
    fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
}

// Core Foundation types for creating the options dictionary
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFDictionaryCreate(
        allocator: *const std::ffi::c_void,
        keys: *const *const std::ffi::c_void,
        values: *const *const std::ffi::c_void,
        num_values: isize,
        key_callbacks: *const std::ffi::c_void,
        value_callbacks: *const std::ffi::c_void,
    ) -> *const std::ffi::c_void;

    fn CFRelease(cf: *const std::ffi::c_void);

    static kCFTypeDictionaryKeyCallBacks: std::ffi::c_void;
    static kCFTypeDictionaryValueCallBacks: std::ffi::c_void;
    static kCFBooleanTrue: *const std::ffi::c_void;
    static kAXTrustedCheckOptionPrompt: *const std::ffi::c_void;
}

/// Check if the application has Accessibility permission
///
/// Returns true if Accessibility is enabled for this app in System Settings.
pub fn check_accessibility_permission() -> bool {
    // SAFETY: AXIsProcessTrusted is a safe C function that just checks permission state
    unsafe { AXIsProcessTrusted() }
}

/// Check if the application has Accessibility permission, prompting if not
///
/// If the app doesn't have permission, shows a system dialog prompting the user
/// to open System Settings and enable it. The app will be automatically added
/// to the Accessibility list (but disabled - user must enable it).
///
/// Returns true if permission is already granted, false otherwise.
pub fn check_accessibility_permission_with_prompt() -> bool {
    unsafe {
        // Create options dictionary with kAXTrustedCheckOptionPrompt = true
        let keys = [kAXTrustedCheckOptionPrompt];
        let values = [kCFBooleanTrue];

        let options = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks,
            &kCFTypeDictionaryValueCallBacks,
        );

        let result = AXIsProcessTrustedWithOptions(options);

        if !options.is_null() {
            CFRelease(options);
        }

        result
    }
}

/// Open System Settings to the Accessibility pane
///
/// Opens the Privacy & Security > Accessibility section where users can
/// grant permission to this app.
///
/// Returns Ok(()) if the settings were opened successfully, or an error message.
pub fn open_accessibility_settings() -> Result<(), String> {
    let url = "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

    Command::new("open")
        .arg(url)
        .spawn()
        .map_err(|e| format!("Failed to open System Settings: {}", e))?;

    Ok(())
}

/// Error returned when Accessibility permission is not granted
#[derive(Debug, Clone)]
pub struct AccessibilityPermissionError {
    pub message: String,
}

impl std::fmt::Display for AccessibilityPermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AccessibilityPermissionError {}

impl AccessibilityPermissionError {
    pub fn new() -> Self {
        Self {
            message: "Accessibility permission required. Please grant permission in System Settings > Privacy & Security > Accessibility, then restart the app.".to_string(),
        }
    }
}

impl Default for AccessibilityPermissionError {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "permissions_test.rs"]
mod tests;
