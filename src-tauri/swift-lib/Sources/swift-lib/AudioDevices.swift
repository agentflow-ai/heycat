import AVFoundation
import CoreAudio
import SwiftRs

/// Get the default audio input device ID using Core Audio.
private func getDefaultInputDeviceId() -> AudioDeviceID? {
    var propertyAddress = AudioObjectPropertyAddress(
        mSelector: kAudioHardwarePropertyDefaultInputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain
    )

    var deviceId: AudioDeviceID = 0
    var size = UInt32(MemoryLayout<AudioDeviceID>.size)

    let status = AudioObjectGetPropertyData(
        AudioObjectID(kAudioObjectSystemObject),
        &propertyAddress,
        0,
        nil,
        &size,
        &deviceId
    )

    return status == noErr ? deviceId : nil
}

/// Get the AudioDeviceID for an AVCaptureDevice by its uniqueID.
private func getAudioDeviceId(for uniqueId: String) -> AudioDeviceID? {
    var propertyAddress = AudioObjectPropertyAddress(
        mSelector: kAudioHardwarePropertyDeviceForUID,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain
    )

    var deviceId: AudioDeviceID = 0
    var cfUID: CFString = uniqueId as CFString
    var translation = AudioValueTranslation(
        mInputData: &cfUID,
        mInputDataSize: UInt32(MemoryLayout<CFString>.size),
        mOutputData: &deviceId,
        mOutputDataSize: UInt32(MemoryLayout<AudioDeviceID>.size)
    )

    var translationSize = UInt32(MemoryLayout<AudioValueTranslation>.size)

    let status = AudioObjectGetPropertyData(
        AudioObjectID(kAudioObjectSystemObject),
        &propertyAddress,
        0,
        nil,
        &translationSize,
        &translation
    )

    return status == noErr ? deviceId : nil
}

/// Struct to hold device info for a single device.
/// We'll return device info one at a time via indexed access.
private var cachedDevices: [(name: String, isDefault: Bool)] = []

/// Refresh the cached list of audio devices.
@_cdecl("swift_refresh_audio_devices")
public func refreshAudioDevices() -> Int {
    cachedDevices.removeAll()

    // Get the default input device ID for comparison
    let defaultDeviceId = getDefaultInputDeviceId()

    // Use AVCaptureDevice.DiscoverySession to find audio devices
    let discoverySession = AVCaptureDevice.DiscoverySession(
        deviceTypes: [.microphone, .builtInMicrophone, .externalUnknown],
        mediaType: .audio,
        position: .unspecified
    )

    for captureDevice in discoverySession.devices {
        let deviceAudioId = getAudioDeviceId(for: captureDevice.uniqueID)
        let isDefault = (deviceAudioId != nil && deviceAudioId == defaultDeviceId)

        cachedDevices.append((name: captureDevice.localizedName, isDefault: isDefault))
    }

    // Sort with default device first
    cachedDevices.sort { $0.isDefault && !$1.isDefault }

    return cachedDevices.count
}

/// Get the name of the device at the given index.
/// Call refresh_audio_devices first to populate the cache.
@_cdecl("swift_get_device_name")
public func getDeviceName(index: Int) -> SRString {
    guard index >= 0 && index < cachedDevices.count else {
        return SRString("")
    }
    return SRString(cachedDevices[index].name)
}

/// Get whether the device at the given index is the default.
/// Call refresh_audio_devices first to populate the cache.
@_cdecl("swift_get_device_is_default")
public func getDeviceIsDefault(index: Int) -> Bool {
    guard index >= 0 && index < cachedDevices.count else {
        return false
    }
    return cachedDevices[index].isDefault
}
