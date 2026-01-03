import AVFoundation
import CoreAudio
import SwiftRs

/// Unified audio engine for both capture and level monitoring.
/// Uses a single AVAudioEngine instance to avoid device conflicts.
/// Follows Apple best practice: one engine per audio graph.
private class SharedAudioEngineManager {
    static let shared = SharedAudioEngineManager()

    private var audioEngine: AVAudioEngine?
    private var isRunning = false
    private var isCapturing = false
    private var recordingStartTime: Date?
    private var lastError: String?
    private var currentLevel: UInt8 = 0
    private var currentDeviceName: String?

    private let targetSampleRate: Double = 16000.0

    // File-based capture (replaces in-memory array to avoid dropped samples)
    private var captureFile: AVAudioFile?
    private var captureFileURL: URL?
    private var captureFormat: AVAudioFormat?

    // Accumulator for RMS calculation (lightweight, no lock needed)
    private var sampleCount: Int = 0
    private var sumSquares: Float = 0.0
    private let samplesPerLevelEmission: Int = 16000 / 20 // ~50ms at 16kHz

    // Serial queue for thread-safe audio operations
    private let audioQueue = DispatchQueue(label: "com.heycat.sharedaudioengine", qos: .userInteractive)

    // Lock for state reading (minimized scope - only for error/level reads)
    private let stateLock = NSLock()

    private init() {}

    // MARK: - Engine Control

    /// Start the audio engine with optional device selection.
    /// The engine provides level monitoring continuously once started.
    func startEngine(deviceName: String?) -> Bool {
        return audioQueue.sync {
            // If already running, just update device if needed
            if isRunning {
                if deviceName != currentDeviceName {
                    return switchDevice(deviceName: deviceName)
                }
                return true
            }

            // Reset state
            stateLock.lock()
            lastError = nil
            currentLevel = 0
            sampleCount = 0
            sumSquares = 0.0
            stateLock.unlock()

            do {
                let engine = AVAudioEngine()
                audioEngine = engine

                let inputNode = engine.inputNode

                // Set the input device if specified
                if let deviceName = deviceName, !deviceName.isEmpty {
                    if !setInputDevice(deviceName: deviceName, inputNode: inputNode) {
                        print("Device '\(deviceName)' not found, using default input")
                    }
                    // Allow Core Audio time to fully configure the new device
                    Thread.sleep(forTimeInterval: 0.2)
                }

                // Remove any existing tap
                inputNode.removeTap(onBus: 0)

                // Query the actual hardware format AFTER device change has propagated
                // Using nil format in installTap causes race conditions with device switching
                let inputFormat = inputNode.inputFormat(forBus: 0)
                guard inputFormat.sampleRate > 0 else {
                    stateLock.lock()
                    lastError = "Invalid input format from device (sampleRate=0)"
                    stateLock.unlock()
                    return false
                }

                // Create target format for 16kHz mono (for capture)
                guard let outputFormat = AVAudioFormat(
                    commonFormat: .pcmFormatFloat32,
                    sampleRate: targetSampleRate,
                    channels: 1,
                    interleaved: false
                ) else {
                    stateLock.lock()
                    lastError = "Failed to create output format"
                    stateLock.unlock()
                    return false
                }

                // Install tap with explicit format to avoid race condition with device changes
                let bufferSize: AVAudioFrameCount = 1024

                // Pre-initialize converter at engine start (not lazily on first buffer)
                // This moves expensive initialization off the real-time audio thread
                let converter: AVAudioConverter? =
                    (inputFormat.sampleRate != targetSampleRate || inputFormat.channelCount != 1)
                    ? AVAudioConverter(from: inputFormat, to: outputFormat)
                    : nil

                inputNode.installTap(onBus: 0, bufferSize: bufferSize, format: inputFormat) { [weak self] buffer, _ in
                    guard let self = self else { return }
                    self.processAudioBuffer(buffer, converter: converter, outputFormat: outputFormat)
                }

                // Start the engine
                try engine.start()

                isRunning = true
                currentDeviceName = deviceName

                return true

            } catch {
                stateLock.lock()
                lastError = "Failed to start audio engine: \(error.localizedDescription)"
                stateLock.unlock()
                audioEngine = nil
                return false
            }
        }
    }

    /// Stop the audio engine completely.
    func stopEngine() {
        audioQueue.sync {
            stopEngineInternal()
        }
    }

    /// Internal stop - must be called on audioQueue
    /// - Parameter preserveCaptureFile: If true, don't clear capture state (used during device switch while recording)
    private func stopEngineInternal(preserveCaptureFile: Bool = false) {
        guard isRunning else { return }

        audioEngine?.inputNode.removeTap(onBus: 0)
        audioEngine?.stop()
        audioEngine = nil

        isRunning = false

        // Only clear capture state if not preserving (e.g., during device switch while recording)
        if !preserveCaptureFile {
            isCapturing = false

            // Clean up capture file if still open
            captureFile = nil
            if let url = captureFileURL {
                try? FileManager.default.removeItem(at: url)
                captureFileURL = nil
            }
        }

        stateLock.lock()
        currentLevel = 0
        stateLock.unlock()
    }

    /// Switch to a different audio device while engine is running.
    /// Preserves capture state if recording is in progress.
    private func switchDevice(deviceName: String?) -> Bool {
        // Save capture state - if we're capturing, we need to preserve the file
        let wasCapturing = isCapturing

        // Stop current engine, preserving capture file if we were recording
        stopEngineInternal(preserveCaptureFile: wasCapturing)

        // Small delay for Core Audio cleanup
        Thread.sleep(forTimeInterval: 0.15)

        // Restart with new device
        // Note: We can't call startEngine here as we're already on audioQueue
        // So we inline the startup logic
        stateLock.lock()
        lastError = nil
        currentLevel = 0
        sampleCount = 0
        sumSquares = 0.0
        stateLock.unlock()

        do {
            let engine = AVAudioEngine()
            audioEngine = engine

            let inputNode = engine.inputNode

            if let deviceName = deviceName, !deviceName.isEmpty {
                if !setInputDevice(deviceName: deviceName, inputNode: inputNode) {
                    print("Device '\(deviceName)' not found, using default input")
                }
                // Allow Core Audio time to fully configure the new device
                Thread.sleep(forTimeInterval: 0.2)
            }

            inputNode.removeTap(onBus: 0)

            // Query the actual hardware format AFTER device change has propagated
            let inputFormat = inputNode.inputFormat(forBus: 0)
            guard inputFormat.sampleRate > 0 else {
                stateLock.lock()
                lastError = "Invalid input format from device (sampleRate=0)"
                stateLock.unlock()
                return false
            }

            guard let outputFormat = AVAudioFormat(
                commonFormat: .pcmFormatFloat32,
                sampleRate: targetSampleRate,
                channels: 1,
                interleaved: false
            ) else {
                stateLock.lock()
                lastError = "Failed to create output format"
                stateLock.unlock()
                return false
            }

            // Install tap with explicit format to avoid race condition with device changes
            let bufferSize: AVAudioFrameCount = 1024

            // Pre-initialize converter at engine start (not lazily on first buffer)
            // This moves expensive initialization off the real-time audio thread
            let converter: AVAudioConverter? =
                (inputFormat.sampleRate != targetSampleRate || inputFormat.channelCount != 1)
                ? AVAudioConverter(from: inputFormat, to: outputFormat)
                : nil

            inputNode.installTap(onBus: 0, bufferSize: bufferSize, format: inputFormat) { [weak self] buffer, _ in
                guard let self = self else { return }
                self.processAudioBuffer(buffer, converter: converter, outputFormat: outputFormat)
            }

            try engine.start()

            isRunning = true
            currentDeviceName = deviceName

            return true

        } catch {
            stateLock.lock()
            lastError = "Failed to start audio engine: \(error.localizedDescription)"
            stateLock.unlock()
            audioEngine = nil
            return false
        }
    }

    /// Set device for the engine (public API for device switching)
    func setDevice(deviceName: String?) -> Bool {
        return audioQueue.sync {
            guard isRunning else {
                stateLock.lock()
                lastError = "Engine not running"
                stateLock.unlock()
                return false
            }

            // Skip expensive device switch if already using this device
            // This avoids 350ms of Thread.sleep() calls in switchDevice()
            if deviceName == currentDeviceName {
                return true
            }

            return switchDevice(deviceName: deviceName)
        }
    }

    // MARK: - Capture Control

    /// Start capturing audio samples to file (engine must be running).
    /// Uses AVAudioFile for reliable capture without dropped samples.
    func startCapture() -> Bool {
        return audioQueue.sync {
            guard isRunning else {
                stateLock.lock()
                lastError = "Engine not running"
                stateLock.unlock()
                return false
            }

            guard !isCapturing else {
                stateLock.lock()
                lastError = "Already capturing"
                stateLock.unlock()
                return false
            }

            // Create temp file for capture
            let tempDir = FileManager.default.temporaryDirectory
            let fileURL = tempDir.appendingPathComponent("capture_\(UUID().uuidString).wav")

            // Create format for 16kHz mono float32 (our target format)
            guard let format = AVAudioFormat(
                commonFormat: .pcmFormatFloat32,
                sampleRate: targetSampleRate,
                channels: 1,
                interleaved: false
            ) else {
                stateLock.lock()
                lastError = "Failed to create capture format"
                stateLock.unlock()
                return false
            }

            do {
                captureFile = try AVAudioFile(forWriting: fileURL, settings: format.settings)
                captureFileURL = fileURL
                captureFormat = format
                isCapturing = true
                recordingStartTime = Date()
                return true
            } catch {
                stateLock.lock()
                lastError = "Failed to create capture file: \(error.localizedDescription)"
                stateLock.unlock()
                return false
            }
        }
    }

    /// Stop capturing and return the file URL containing captured samples.
    /// Returns nil if not capturing or on error.
    func stopCapture() -> URL? {
        return audioQueue.sync {
            guard isCapturing else {
                return nil
            }

            isCapturing = false

            // Close the capture file (this flushes any buffered data)
            captureFile = nil
            captureFormat = nil

            // Return the URL - caller is responsible for reading and deleting
            let url = captureFileURL
            captureFileURL = nil
            return url
        }
    }

    // MARK: - Audio Processing

    /// Process incoming audio buffer for both level monitoring and capture.
    /// Level monitoring is lightweight (no locks needed for atomic writes).
    /// Capture writes directly to AVAudioFile (no in-memory accumulation).
    private func processAudioBuffer(_ buffer: AVAudioPCMBuffer, converter: AVAudioConverter?, outputFormat: AVAudioFormat) {
        guard isRunning else { return }

        // Convert buffer if needed (for both level monitoring and capture)
        let processBuffer: AVAudioPCMBuffer?

        if let converter = converter {
            let frameCapacity = AVAudioFrameCount(
                Double(buffer.frameLength) * (targetSampleRate / buffer.format.sampleRate)
            ) + 1

            guard let convertedBuffer = AVAudioPCMBuffer(pcmFormat: outputFormat, frameCapacity: frameCapacity) else {
                return
            }

            var error: NSError?
            let inputBlock: AVAudioConverterInputBlock = { inNumPackets, outStatus in
                outStatus.pointee = .haveData
                return buffer
            }

            converter.convert(to: convertedBuffer, error: &error, withInputFrom: inputBlock)

            if error != nil || convertedBuffer.frameLength == 0 {
                return
            }

            processBuffer = convertedBuffer
        } else {
            // No conversion needed - use original buffer
            processBuffer = buffer
        }

        guard let audioBuffer = processBuffer,
              let channelData = audioBuffer.floatChannelData else {
            return
        }

        let frameCount = Int(audioBuffer.frameLength)

        // Write to capture file if capturing (AVAudioFile handles buffering internally)
        if isCapturing, let file = captureFile {
            try? file.write(from: audioBuffer)
        }

        // Calculate RMS level for monitoring (lightweight, no lock needed)
        var sumSq: Float = 0.0
        let channelCount = Int(audioBuffer.format.channelCount)

        if channelCount == 1 {
            for i in 0..<frameCount {
                let sample = channelData[0][i]
                sumSq += sample * sample
            }
        } else {
            // Mix channels for level calculation
            for i in 0..<frameCount {
                var mixedSample: Float = 0.0
                for ch in 0..<channelCount {
                    mixedSample += channelData[ch][i]
                }
                mixedSample /= Float(channelCount)
                sumSq += mixedSample * mixedSample
            }
        }

        // Accumulate for periodic level emission
        stateLock.lock()
        sumSquares += sumSq
        sampleCount += frameCount

        // Emit level when we have enough samples (~50ms)
        if sampleCount >= samplesPerLevelEmission {
            let rms = sqrt(sumSquares / Float(sampleCount))
            let level = min(rms * 300.0, 100.0)
            currentLevel = UInt8(level)

            sampleCount = 0
            sumSquares = 0.0
        }
        stateLock.unlock()
    }

    // MARK: - State Queries

    func getLevel() -> UInt8 {
        stateLock.lock()
        defer { stateLock.unlock() }
        return currentLevel
    }

    func getIsRunning() -> Bool {
        return isRunning
    }

    func getIsCapturing() -> Bool {
        return isCapturing
    }

    func getRecordingDuration() -> Double {
        stateLock.lock()
        defer { stateLock.unlock() }

        guard let startTime = recordingStartTime, isCapturing else {
            return 0.0
        }
        return Date().timeIntervalSince(startTime)
    }

    func getSampleCount() -> Int {
        // With file-based capture, we can query the file length
        if let file = captureFile {
            return Int(file.length)
        }
        return 0
    }

    func getLastError() -> String? {
        stateLock.lock()
        defer { stateLock.unlock() }
        return lastError
    }

    // MARK: - Device Selection

    /// Set the input device by name using Core Audio.
    private func setInputDevice(deviceName: String, inputNode: AVAudioInputNode) -> Bool {
        var propertyAddress = AudioObjectPropertyAddress(
            mSelector: kAudioHardwarePropertyDevices,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain
        )

        var dataSize: UInt32 = 0
        var status = AudioObjectGetPropertyDataSize(
            AudioObjectID(kAudioObjectSystemObject),
            &propertyAddress,
            0,
            nil,
            &dataSize
        )

        guard status == noErr else { return false }

        let deviceCount = Int(dataSize) / MemoryLayout<AudioDeviceID>.size
        var deviceIds = [AudioDeviceID](repeating: 0, count: deviceCount)

        status = AudioObjectGetPropertyData(
            AudioObjectID(kAudioObjectSystemObject),
            &propertyAddress,
            0,
            nil,
            &dataSize,
            &deviceIds
        )

        guard status == noErr else { return false }

        for deviceId in deviceIds {
            var namePropertyAddress = AudioObjectPropertyAddress(
                mSelector: kAudioDevicePropertyDeviceNameCFString,
                mScope: kAudioObjectPropertyScopeGlobal,
                mElement: kAudioObjectPropertyElementMain
            )

            var name: CFString?
            var nameSize = UInt32(MemoryLayout<CFString?>.size)

            status = AudioObjectGetPropertyData(
                deviceId,
                &namePropertyAddress,
                0,
                nil,
                &nameSize,
                &name
            )

            if status == noErr, let deviceNameCF = name as String?, deviceNameCF == deviceName {
                var inputChannelsAddress = AudioObjectPropertyAddress(
                    mSelector: kAudioDevicePropertyStreamConfiguration,
                    mScope: kAudioDevicePropertyScopeInput,
                    mElement: kAudioObjectPropertyElementMain
                )

                var inputSize: UInt32 = 0
                status = AudioObjectGetPropertyDataSize(deviceId, &inputChannelsAddress, 0, nil, &inputSize)

                if status == noErr && inputSize > 0 {
                    guard let audioUnit = inputNode.audioUnit else { return false }

                    var deviceIdVar = deviceId
                    status = AudioUnitSetProperty(
                        audioUnit,
                        kAudioOutputUnitProperty_CurrentDevice,
                        kAudioUnitScope_Global,
                        0,
                        &deviceIdVar,
                        UInt32(MemoryLayout<AudioDeviceID>.size)
                    )

                    return status == noErr
                }
            }
        }

        return false
    }
}

// MARK: - FFI Functions

/// Start the audio engine. Returns true on success.
@_cdecl("swift_audio_engine_start")
public func audioEngineStart(deviceName: SRString?) -> Bool {
    let device = deviceName?.toString()
    return SharedAudioEngineManager.shared.startEngine(deviceName: device)
}

/// Stop the audio engine.
@_cdecl("swift_audio_engine_stop")
public func audioEngineStop() {
    SharedAudioEngineManager.shared.stopEngine()
}

/// Set the audio device. Returns true on success.
@_cdecl("swift_audio_engine_set_device")
public func audioEngineSetDevice(deviceName: SRString?) -> Bool {
    let device = deviceName?.toString()
    return SharedAudioEngineManager.shared.setDevice(deviceName: device)
}

/// Check if engine is running.
@_cdecl("swift_audio_engine_is_running")
public func audioEngineIsRunning() -> Bool {
    return SharedAudioEngineManager.shared.getIsRunning()
}

/// Get the current audio level (0-100). Available whenever engine is running.
@_cdecl("swift_audio_engine_get_level")
public func audioEngineGetLevel() -> UInt8 {
    return SharedAudioEngineManager.shared.getLevel()
}

/// Start audio capture. Engine must be running. Returns true on success.
@_cdecl("swift_audio_engine_start_capture")
public func audioEngineStartCapture() -> Bool {
    return SharedAudioEngineManager.shared.startCapture()
}

/// Stop audio capture. Returns the file path containing captured samples.
/// Returns empty string if not capturing or on error.
/// Caller is responsible for reading and deleting the file.
@_cdecl("swift_audio_engine_stop_capture")
public func audioEngineStopCapture() -> SRString {
    if let url = SharedAudioEngineManager.shared.stopCapture() {
        return SRString(url.path)
    }
    return SRString("")
}

/// Check if currently capturing.
@_cdecl("swift_audio_engine_is_capturing")
public func audioEngineIsCapturing() -> Bool {
    return SharedAudioEngineManager.shared.getIsCapturing()
}

/// Get recording duration in milliseconds.
@_cdecl("swift_audio_engine_get_duration_ms")
public func audioEngineGetDurationMs() -> Int {
    return Int(SharedAudioEngineManager.shared.getRecordingDuration() * 1000.0)
}

/// Get current sample count (useful during capture).
@_cdecl("swift_audio_engine_get_sample_count")
public func audioEngineGetSampleCount() -> Int {
    return SharedAudioEngineManager.shared.getSampleCount()
}

/// Get the last error message, if any.
@_cdecl("swift_audio_engine_get_error")
public func audioEngineGetError() -> SRString {
    if let error = SharedAudioEngineManager.shared.getLastError() {
        return SRString(error)
    }
    return SRString("")
}
