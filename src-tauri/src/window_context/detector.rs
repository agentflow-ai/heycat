// Active window detection for macOS
//
// Uses NSWorkspace and CGWindowList APIs to detect the currently focused application
// and its window title. Requires Accessibility permission for window title detection.

use super::types::{ActiveWindowInfo, RunningApplication};
use core_foundation::base::{TCFType, ToVoid};
use core_foundation::dictionary::CFDictionaryRef;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use objc::{class, msg_send, sel, sel_impl};

/// Get information about the currently active window on macOS
///
/// Returns the frontmost application's name, bundle ID, window title, and process ID.
/// Window title detection requires Accessibility permission.
///
/// # Errors
///
/// Returns an error string if:
/// - Unable to get the frontmost application
/// - Unable to get the process ID
pub fn get_active_window() -> Result<ActiveWindowInfo, String> {
    unsafe { get_active_window_impl() }
}

/// Get a list of currently running user-visible applications
///
/// Returns applications that have a user interface (activationPolicy == .regular).
/// Background helpers, agents, and daemons are filtered out.
///
/// # Returns
///
/// A vector of `RunningApplication` structs, sorted by name.
pub fn get_running_applications() -> Vec<RunningApplication> {
    unsafe { get_running_applications_impl() }
}

#[allow(deprecated)]
unsafe fn get_active_window_impl() -> Result<ActiveWindowInfo, String> {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString as CocoaNSString;

    // Get the shared workspace
    let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
    if workspace == nil {
        return Err("Failed to get shared workspace".to_string());
    }

    // Get the frontmost application
    let frontmost_app: id = msg_send![workspace, frontmostApplication];
    if frontmost_app == nil {
        return Err("No frontmost application".to_string());
    }

    // Get app name (localizedName) using msg_send
    let app_name_ns: id = msg_send![frontmost_app, localizedName];
    let app_name = if app_name_ns != nil {
        let cstr: *const std::os::raw::c_char = CocoaNSString::UTF8String(app_name_ns);
        if !cstr.is_null() {
            std::ffi::CStr::from_ptr(cstr)
                .to_string_lossy()
                .into_owned()
        } else {
            "Unknown".to_string()
        }
    } else {
        "Unknown".to_string()
    };

    // Get bundle ID using msg_send
    let bundle_id_ns: id = msg_send![frontmost_app, bundleIdentifier];
    let bundle_id = if bundle_id_ns != nil {
        let cstr: *const std::os::raw::c_char = CocoaNSString::UTF8String(bundle_id_ns);
        if !cstr.is_null() {
            Some(
                std::ffi::CStr::from_ptr(cstr)
                    .to_string_lossy()
                    .into_owned(),
            )
        } else {
            None
        }
    } else {
        None
    };

    // Get process ID using msg_send
    let pid: i32 = msg_send![frontmost_app, processIdentifier];
    if pid <= 0 {
        return Err("Invalid process ID".to_string());
    }

    // Get window title using CGWindowList
    let window_title = get_window_title_for_pid(pid as u32);

    Ok(ActiveWindowInfo {
        app_name,
        bundle_id,
        window_title,
        pid: pid as u32,
    })
}

/// Get the window title for a specific process ID using CGWindowList
fn get_window_title_for_pid(target_pid: u32) -> Option<String> {
    use core_graphics::window::{
        kCGNullWindowID, kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly,
        CGWindowListCopyWindowInfo,
    };

    unsafe {
        // Get list of on-screen windows
        let options = kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements;
        let window_list = CGWindowListCopyWindowInfo(options, kCGNullWindowID);
        if window_list.is_null() {
            return None;
        }

        // CFArray doesn't have a direct Rust wrapper that's easy to iterate
        // Use raw Core Foundation calls
        let count = core_foundation::array::CFArrayGetCount(window_list as _);

        for i in 0..count {
            let window_info =
                core_foundation::array::CFArrayGetValueAtIndex(window_list as _, i) as CFDictionaryRef;
            if window_info.is_null() {
                continue;
            }

            // Get window owner PID
            let pid_key = CFString::new("kCGWindowOwnerPID");
            let pid_value = core_foundation::dictionary::CFDictionaryGetValue(
                window_info,
                pid_key.to_void(),
            );
            if pid_value.is_null() {
                continue;
            }

            let pid_cf = CFNumber::wrap_under_get_rule(pid_value as _);
            let pid: i32 = pid_cf.to_i32().unwrap_or(-1);

            if pid as u32 != target_pid {
                continue;
            }

            // Get window layer (0 = normal window, higher = overlays/menus)
            let layer_key = CFString::new("kCGWindowLayer");
            let layer_value = core_foundation::dictionary::CFDictionaryGetValue(
                window_info,
                layer_key.to_void(),
            );
            if !layer_value.is_null() {
                let layer_cf = CFNumber::wrap_under_get_rule(layer_value as _);
                let layer: i32 = layer_cf.to_i32().unwrap_or(0);
                // Skip windows that are not in the normal layer (0)
                // This filters out menus, tooltips, and other overlay windows
                if layer != 0 {
                    continue;
                }
            }

            // Get window title (kCGWindowName)
            let name_key = CFString::new("kCGWindowName");
            let name_value = core_foundation::dictionary::CFDictionaryGetValue(
                window_info,
                name_key.to_void(),
            );
            if name_value.is_null() {
                continue;
            }

            let name_cf = CFString::wrap_under_get_rule(name_value as _);
            let title = name_cf.to_string();

            // Skip empty titles
            if title.is_empty() {
                continue;
            }

            // Release the window list and return the first matching title
            core_foundation::base::CFRelease(window_list as *const std::ffi::c_void);
            return Some(title);
        }

        // Clean up
        core_foundation::base::CFRelease(window_list as *const std::ffi::c_void);
        None
    }
}

/// Implementation for getting running applications
#[allow(deprecated)]
unsafe fn get_running_applications_impl() -> Vec<RunningApplication> {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString as CocoaNSString;

    let mut apps = Vec::new();

    // Get the shared workspace
    let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
    if workspace == nil {
        return apps;
    }

    // Get the frontmost application to determine which app is active
    let frontmost_app: id = msg_send![workspace, frontmostApplication];
    let frontmost_bundle_id: Option<String> = if frontmost_app != nil {
        let bundle_ns: id = msg_send![frontmost_app, bundleIdentifier];
        if bundle_ns != nil {
            let cstr: *const std::os::raw::c_char = CocoaNSString::UTF8String(bundle_ns);
            if !cstr.is_null() {
                Some(
                    std::ffi::CStr::from_ptr(cstr)
                        .to_string_lossy()
                        .into_owned(),
                )
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Get all running applications
    let running_apps: id = msg_send![workspace, runningApplications];
    if running_apps == nil {
        return apps;
    }

    // Get the count of applications
    let count: usize = msg_send![running_apps, count];

    for i in 0..count {
        let app: id = msg_send![running_apps, objectAtIndex: i];
        if app == nil {
            continue;
        }

        // Check activation policy - we only want "regular" apps (0 = NSApplicationActivationPolicyRegular)
        // 1 = accessory, 2 = prohibited (background agents)
        let activation_policy: i64 = msg_send![app, activationPolicy];
        if activation_policy != 0 {
            continue;
        }

        // Get localized name
        let name_ns: id = msg_send![app, localizedName];
        let name = if name_ns != nil {
            let cstr: *const std::os::raw::c_char = CocoaNSString::UTF8String(name_ns);
            if !cstr.is_null() {
                std::ffi::CStr::from_ptr(cstr)
                    .to_string_lossy()
                    .into_owned()
            } else {
                continue; // Skip apps without names
            }
        } else {
            continue; // Skip apps without names
        };

        // Get bundle ID
        let bundle_ns: id = msg_send![app, bundleIdentifier];
        let bundle_id = if bundle_ns != nil {
            let cstr: *const std::os::raw::c_char = CocoaNSString::UTF8String(bundle_ns);
            if !cstr.is_null() {
                Some(
                    std::ffi::CStr::from_ptr(cstr)
                        .to_string_lossy()
                        .into_owned(),
                )
            } else {
                None
            }
        } else {
            None
        };

        // Determine if this is the active app
        let is_active = match (&bundle_id, &frontmost_bundle_id) {
            (Some(bid), Some(fid)) => bid == fid,
            _ => false,
        };

        apps.push(RunningApplication {
            name,
            bundle_id,
            is_active,
        });
    }

    // Sort by name for consistent ordering
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    apps
}

#[cfg(test)]
#[path = "detector_test.rs"]
mod tests;
