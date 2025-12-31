//! macOS application activation helpers
//!
//! On macOS, an application can have visible windows but not be "activated",
//! meaning it doesn't receive mouse/keyboard events properly. This module
//! ensures the app is properly activated at startup.

/// Activate the application on macOS so it receives mouse and keyboard events.
///
/// On macOS, a window can be visible but the application itself not "active",
/// which prevents it from receiving left-click events. This function calls
/// `NSApplication.activateIgnoringOtherApps()` to ensure the app is properly
/// activated at startup.
#[cfg(target_os = "macos")]
#[allow(deprecated)] // cocoa crate types are deprecated in favor of objc2, but we use existing patterns
pub fn activate_app() {
    use cocoa::base::nil;
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let app: cocoa::base::id = msg_send![class!(NSApplication), sharedApplication];
        if app != nil {
            let _: () = msg_send![app, activateIgnoringOtherApps: true];
            crate::info!("macOS app activated via NSApplication.activateIgnoringOtherApps");
        } else {
            crate::warn!("Failed to get NSApplication.sharedApplication for activation");
        }
    }
}

/// No-op on non-macOS platforms - activation is handled automatically.
#[cfg(not(target_os = "macos"))]
pub fn activate_app() {
    crate::debug!("activate_app called on non-macOS platform (no-op)");
}
