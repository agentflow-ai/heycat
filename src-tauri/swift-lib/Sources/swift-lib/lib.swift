import SwiftRs
import AppKit
import CoreAudio

/// A simple test function to verify Swift-Rust interop is working.
/// Returns "Hello from Swift!" as a SRString (Swift-Rs String type).
@_cdecl("swift_hello")
public func swiftHello() -> SRString {
    SRString("Hello from Swift!")
}

// MARK: - System Wake Notification Observer

/// Type alias for the C callback function pointer.
/// The callback takes no arguments and returns void.
public typealias WakeCallback = @convention(c) () -> Void

/// Singleton manager for system wake notifications.
/// Observes NSWorkspace.didWakeNotification and invokes a registered callback.
private class WakeNotificationManager {
    static let shared = WakeNotificationManager()

    private var observer: NSObjectProtocol?
    private var callback: WakeCallback?
    private let lock = NSLock()

    private init() {}

    /// Register a callback to be invoked when the system wakes from sleep.
    /// Only one callback can be registered at a time; calling again replaces the previous callback.
    /// - Parameter callback: C function pointer to call on system wake
    func registerCallback(_ callback: @escaping WakeCallback) {
        lock.lock()

        // Store the callback
        self.callback = callback

        // If already observing, no need to register again
        if observer != nil {
            lock.unlock()
            return
        }

        lock.unlock()

        // Register for wake notifications on the main thread
        // NSWorkspace notifications must be observed on the main run loop
        // Use async to avoid blocking in test environments without a run loop
        if Thread.isMainThread {
            registerObserver()
        } else {
            DispatchQueue.main.async { [weak self] in
                self?.registerObserver()
            }
        }
    }

    /// Unregister the wake callback and stop observing.
    func unregisterCallback() {
        lock.lock()
        defer { lock.unlock() }

        callback = nil

        if let obs = observer {
            NSWorkspace.shared.notificationCenter.removeObserver(obs)
            observer = nil
        }
    }

    /// Internal: register the observer (must be called on main thread)
    private func registerObserver() {
        observer = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didWakeNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.handleWake()
        }
    }

    /// Internal: handle wake notification
    private func handleWake() {
        lock.lock()
        let cb = callback
        lock.unlock()

        if let cb = cb {
            cb()
        }
    }
}

/// Register a callback to be invoked when the system wakes from sleep.
/// The callback will be called on the main thread.
/// - Parameter callbackPtr: Raw pointer to a C function that takes no arguments and returns void
@_cdecl("swift_register_wake_callback")
public func registerWakeCallback(callbackPtr: UnsafeRawPointer) {
    let callback = unsafeBitCast(callbackPtr, to: WakeCallback.self)
    WakeNotificationManager.shared.registerCallback(callback)
}

/// Unregister the wake callback and stop observing wake notifications.
@_cdecl("swift_unregister_wake_callback")
public func unregisterWakeCallback() {
    WakeNotificationManager.shared.unregisterCallback()
}

// MARK: - Audio Device Change Notification Observer

/// Type alias for the C callback function pointer for device changes.
/// The callback takes no arguments and returns void.
public typealias DeviceChangeCallback = @convention(c) () -> Void

/// Singleton manager for audio device change notifications.
/// Observes Core Audio kAudioHardwarePropertyDevices and invokes a registered callback.
private class AudioDeviceChangeManager {
    static let shared = AudioDeviceChangeManager()

    private var callback: DeviceChangeCallback?
    private var isListening = false
    private let lock = NSLock()

    /// Property address for device list changes
    private var propertyAddress = AudioObjectPropertyAddress(
        mSelector: kAudioHardwarePropertyDevices,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain
    )

    private init() {}

    /// Register a callback to be invoked when audio devices connect/disconnect.
    /// Only one callback can be registered at a time; calling again replaces the previous callback.
    /// - Parameter callback: C function pointer to call on device change
    func registerCallback(_ callback: @escaping DeviceChangeCallback) {
        lock.lock()
        defer { lock.unlock() }

        // Store the callback
        self.callback = callback

        // If already listening, no need to register again
        if isListening {
            return
        }

        // Register for device change notifications
        let status = AudioObjectAddPropertyListener(
            AudioObjectID(kAudioObjectSystemObject),
            &propertyAddress,
            deviceChangeListener,
            Unmanaged.passUnretained(self).toOpaque()
        )

        if status == noErr {
            isListening = true
        } else {
            NSLog("[heycat] Failed to register audio device change listener: \(status)")
        }
    }

    /// Unregister the device change callback and stop listening.
    func unregisterCallback() {
        lock.lock()
        defer { lock.unlock() }

        callback = nil

        if isListening {
            AudioObjectRemovePropertyListener(
                AudioObjectID(kAudioObjectSystemObject),
                &propertyAddress,
                deviceChangeListener,
                Unmanaged.passUnretained(self).toOpaque()
            )
            isListening = false
        }
    }

    /// Internal: handle device change notification
    fileprivate func handleDeviceChange() {
        lock.lock()
        let cb = callback
        lock.unlock()

        if let cb = cb {
            cb()
        }
    }
}

/// C callback function for Core Audio property listener.
/// This is called by Core Audio when the device list changes.
private func deviceChangeListener(
    _: AudioObjectID,
    _: UInt32,
    _: UnsafePointer<AudioObjectPropertyAddress>,
    clientData: UnsafeMutableRawPointer?
) -> OSStatus {
    guard let clientData = clientData else { return noErr }

    let manager = Unmanaged<AudioDeviceChangeManager>.fromOpaque(clientData).takeUnretainedValue()
    manager.handleDeviceChange()

    return noErr
}

/// Register a callback to be invoked when audio devices connect/disconnect.
/// The callback will be called when Core Audio detects device list changes.
/// - Parameter callbackPtr: Raw pointer to a C function that takes no arguments and returns void
@_cdecl("swift_register_device_change_callback")
public func registerDeviceChangeCallback(callbackPtr: UnsafeRawPointer) {
    let callback = unsafeBitCast(callbackPtr, to: DeviceChangeCallback.self)
    AudioDeviceChangeManager.shared.registerCallback(callback)
}

/// Unregister the device change callback and stop listening for device changes.
@_cdecl("swift_unregister_device_change_callback")
public func unregisterDeviceChangeCallback() {
    AudioDeviceChangeManager.shared.unregisterCallback()
}
