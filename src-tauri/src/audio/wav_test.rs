#![cfg(test)]
#![cfg_attr(coverage_nightly, coverage(off))]

use super::wav::{encode_wav, parse_duration_from_file, FileWriter, SystemFileWriter, WavEncodingError};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// =============================================================================
// MockFileWriter for testing
// =============================================================================

struct MockFileWriter {
    output_dir: PathBuf,
    filename: String,
    dir_exists: bool,
    should_fail_dir_creation: bool,
    created_dirs: Arc<Mutex<Vec<PathBuf>>>,
}

impl MockFileWriter {
    fn new() -> Self {
        Self {
            output_dir: PathBuf::from("/tmp/test-recordings"),
            filename: "test-recording.wav".to_string(),
            dir_exists: false,
            should_fail_dir_creation: false,
            created_dirs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn with_existing_dir(mut self) -> Self {
        self.dir_exists = true;
        self
    }

    fn with_dir_creation_failure(mut self) -> Self {
        self.should_fail_dir_creation = true;
        self
    }

    fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = dir;
        self
    }

    fn with_filename(mut self, filename: &str) -> Self {
        self.filename = filename.to_string();
        self
    }
}

impl FileWriter for MockFileWriter {
    fn output_dir(&self) -> PathBuf {
        self.output_dir.clone()
    }

    fn generate_filename(&self) -> String {
        self.filename.clone()
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), std::io::Error> {
        if self.should_fail_dir_creation {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Permission denied",
            ));
        }
        self.created_dirs.lock().unwrap().push(path.to_path_buf());
        // Actually create the directory for hound to write to
        std::fs::create_dir_all(path)
    }

    fn path_exists(&self, path: &Path) -> bool {
        if self.dir_exists {
            true
        } else {
            // Check actual filesystem
            path.exists()
        }
    }
}

// =============================================================================
// Validation Tests
// =============================================================================

#[test]
fn test_encode_wav_empty_samples() {
    let writer = MockFileWriter::new();
    let result = encode_wav(&[], 44100, &writer);

    assert!(matches!(result, Err(WavEncodingError::InvalidInput(_))));
    if let Err(WavEncodingError::InvalidInput(msg)) = result {
        assert!(msg.contains("empty"));
    }
}

#[test]
fn test_encode_wav_nan_samples() {
    let writer = MockFileWriter::new();
    let samples = vec![0.5, f32::NAN, 0.3];
    let result = encode_wav(&samples, 44100, &writer);

    assert!(matches!(result, Err(WavEncodingError::InvalidInput(_))));
    if let Err(WavEncodingError::InvalidInput(msg)) = result {
        assert!(msg.contains("NaN") || msg.contains("infinity"));
    }
}

#[test]
fn test_encode_wav_positive_infinity_samples() {
    let writer = MockFileWriter::new();
    let samples = vec![0.5, f32::INFINITY, 0.3];
    let result = encode_wav(&samples, 44100, &writer);

    assert!(matches!(result, Err(WavEncodingError::InvalidInput(_))));
}

#[test]
fn test_encode_wav_negative_infinity_samples() {
    let writer = MockFileWriter::new();
    let samples = vec![0.5, f32::NEG_INFINITY, 0.3];
    let result = encode_wav(&samples, 44100, &writer);

    assert!(matches!(result, Err(WavEncodingError::InvalidInput(_))));
}

// =============================================================================
// Success Path Tests
// =============================================================================

#[test]
fn test_encode_wav_success() {
    let temp_dir = std::env::temp_dir().join("heycat-wav-test-success");
    let _ = std::fs::remove_dir_all(&temp_dir); // Clean up from previous runs

    let writer = MockFileWriter::new()
        .with_output_dir(temp_dir.clone())
        .with_filename("test-success.wav");

    let samples: Vec<f32> = (0..4410)
        .map(|i| (i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 44100.0).sin())
        .collect();

    let result = encode_wav(&samples, 44100, &writer);
    assert!(result.is_ok());

    let path = result.unwrap();
    assert!(path.contains("test-success.wav"));

    // Verify directory was created
    let created = writer.created_dirs.lock().unwrap();
    assert_eq!(created.len(), 1);
    assert_eq!(created[0], temp_dir);

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_encode_wav_with_existing_directory() {
    let temp_dir = std::env::temp_dir().join("heycat-wav-test-existing");
    std::fs::create_dir_all(&temp_dir).unwrap();

    let writer = MockFileWriter::new()
        .with_output_dir(temp_dir.clone())
        .with_existing_dir()
        .with_filename("test-existing.wav");

    let samples = vec![0.5, -0.5, 0.25, -0.25];
    let result = encode_wav(&samples, 44100, &writer);

    assert!(result.is_ok());

    // Verify directory creation was NOT called (already exists)
    let created = writer.created_dirs.lock().unwrap();
    assert_eq!(created.len(), 0);

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_encode_wav_custom_sample_rate() {
    let temp_dir = std::env::temp_dir().join("heycat-wav-test-48k");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let writer = MockFileWriter::new()
        .with_output_dir(temp_dir.clone())
        .with_filename("test-48k.wav");

    let samples = vec![0.1, 0.2, 0.3, 0.4];
    let result = encode_wav(&samples, 48000, &writer);

    assert!(result.is_ok());

    // Verify file was created and can be read back with correct sample rate
    let path = result.unwrap();
    let reader = hound::WavReader::open(&path).unwrap();
    assert_eq!(reader.spec().sample_rate, 48000);

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_encode_wav_clamps_out_of_range_samples() {
    let temp_dir = std::env::temp_dir().join("heycat-wav-test-clamp");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let writer = MockFileWriter::new()
        .with_output_dir(temp_dir.clone())
        .with_filename("test-clamp.wav");

    // Samples outside [-1.0, 1.0] range
    let samples = vec![2.0, -2.0, 1.5, -1.5, 0.5];
    let result = encode_wav(&samples, 44100, &writer);

    // Should succeed (clamping applied)
    assert!(result.is_ok());

    // Verify the file is valid
    let path = result.unwrap();
    let mut reader = hound::WavReader::open(&path).unwrap();
    let read_samples: Vec<i16> = reader.samples().map(|s| s.unwrap()).collect();

    // First two samples should be clamped to max/min i16
    assert_eq!(read_samples[0], i16::MAX);
    assert_eq!(read_samples[1], i16::MIN + 1); // -1.0 maps to -32767, not -32768

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

// =============================================================================
// Error Path Tests
// =============================================================================

#[test]
fn test_encode_wav_directory_creation_failure() {
    let writer = MockFileWriter::new().with_dir_creation_failure();

    let samples = vec![0.5, -0.5];
    let result = encode_wav(&samples, 44100, &writer);

    assert!(matches!(result, Err(WavEncodingError::IoError(_))));
    if let Err(WavEncodingError::IoError(msg)) = result {
        assert!(msg.contains("Permission denied"));
    }
}

// =============================================================================
// SystemFileWriter Tests (testing the real implementation)
// =============================================================================

#[test]
fn test_system_file_writer_output_dir_uses_app_data() {
    let writer = SystemFileWriter::new(std::env::temp_dir().join("heycat-test-recordings"));
    let output_dir = writer.output_dir();

    // Should end with heycat/recordings
    let path_str = output_dir.to_string_lossy();
    assert!(path_str.contains("heycat"));
    assert!(path_str.ends_with("recordings"));
}

#[test]
fn test_system_file_writer_filename_format() {
    let writer = SystemFileWriter::new(std::env::temp_dir().join("heycat-test-recordings"));
    let filename = writer.generate_filename();

    // Should match pattern: recording-YYYY-MM-DD-HHMMSS.wav
    assert!(filename.starts_with("recording-"));
    assert!(filename.ends_with(".wav"));

    // Extract date part and validate format
    let date_part = &filename[10..filename.len() - 4]; // Remove "recording-" and ".wav"
    assert_eq!(date_part.len(), 17); // YYYY-MM-DD-HHMMSS
    assert_eq!(&date_part[4..5], "-");
    assert_eq!(&date_part[7..8], "-");
    assert_eq!(&date_part[10..11], "-");
}

#[test]
fn test_system_file_writer_path_exists() {
    let writer = SystemFileWriter::new(std::env::temp_dir().join("heycat-test-recordings"));

    // Root should exist
    assert!(writer.path_exists(Path::new("/")));

    // Random path should not exist
    assert!(!writer.path_exists(Path::new("/nonexistent/path/12345")));
}

#[test]
fn test_system_file_writer_create_dir_all() {
    let writer = SystemFileWriter::new(std::env::temp_dir().join("heycat-test-recordings"));
    let temp_dir = std::env::temp_dir().join("heycat-test-create-dir");

    // Clean up from previous runs
    let _ = std::fs::remove_dir_all(&temp_dir);

    // Create directory
    let result = writer.create_dir_all(&temp_dir);
    assert!(result.is_ok());
    assert!(temp_dir.exists());

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

// =============================================================================
// WAV Format Verification Tests
// =============================================================================

#[test]
fn test_wav_file_has_correct_format() {
    let temp_dir = std::env::temp_dir().join("heycat-wav-test-format");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let writer = MockFileWriter::new()
        .with_output_dir(temp_dir.clone())
        .with_filename("test-format.wav");

    let samples = vec![0.0, 0.5, 1.0, -0.5, -1.0];
    let result = encode_wav(&samples, 44100, &writer);
    assert!(result.is_ok());

    let path = result.unwrap();
    let reader = hound::WavReader::open(&path).unwrap();
    let spec = reader.spec();

    assert_eq!(spec.channels, 1);
    assert_eq!(spec.sample_rate, 44100);
    assert_eq!(spec.bits_per_sample, 16);
    assert_eq!(spec.sample_format, hound::SampleFormat::Int);

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_wav_file_sample_count_matches() {
    let temp_dir = std::env::temp_dir().join("heycat-wav-test-count");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let writer = MockFileWriter::new()
        .with_output_dir(temp_dir.clone())
        .with_filename("test-count.wav");

    let samples: Vec<f32> = vec![0.1; 1000];
    let result = encode_wav(&samples, 44100, &writer);
    assert!(result.is_ok());

    let path = result.unwrap();
    let reader = hound::WavReader::open(&path).unwrap();

    assert_eq!(reader.len() as usize, 1000);

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

// =============================================================================
// parse_duration_from_file Tests
// =============================================================================

#[test]
fn test_parse_duration_from_valid_wav_file() {
    let temp_dir = std::env::temp_dir().join("heycat-wav-test-duration");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let writer = MockFileWriter::new()
        .with_output_dir(temp_dir.clone())
        .with_filename("test-duration.wav");

    // Create a 1-second file at 44100 Hz
    let samples: Vec<f32> = vec![0.1; 44100];
    let path = encode_wav(&samples, 44100, &writer).unwrap();

    let duration = parse_duration_from_file(Path::new(&path)).unwrap();
    assert!((duration - 1.0).abs() < 0.001); // Should be ~1 second

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_parse_duration_from_short_wav_file() {
    let temp_dir = std::env::temp_dir().join("heycat-wav-test-duration-short");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let writer = MockFileWriter::new()
        .with_output_dir(temp_dir.clone())
        .with_filename("test-duration-short.wav");

    // Create a 0.1-second file at 44100 Hz (4410 samples)
    let samples: Vec<f32> = vec![0.1; 4410];
    let path = encode_wav(&samples, 44100, &writer).unwrap();

    let duration = parse_duration_from_file(Path::new(&path)).unwrap();
    assert!((duration - 0.1).abs() < 0.001); // Should be ~0.1 second

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_parse_duration_from_48k_wav_file() {
    let temp_dir = std::env::temp_dir().join("heycat-wav-test-duration-48k");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let writer = MockFileWriter::new()
        .with_output_dir(temp_dir.clone())
        .with_filename("test-duration-48k.wav");

    // Create a 2-second file at 48000 Hz (96000 samples)
    let samples: Vec<f32> = vec![0.1; 96000];
    let path = encode_wav(&samples, 48000, &writer).unwrap();

    let duration = parse_duration_from_file(Path::new(&path)).unwrap();
    assert!((duration - 2.0).abs() < 0.001); // Should be ~2 seconds

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_parse_duration_from_nonexistent_file() {
    let result = parse_duration_from_file(Path::new("/nonexistent/path/file.wav"));
    assert!(result.is_err());
}

#[test]
fn test_parse_duration_from_invalid_file() {
    let temp_dir = std::env::temp_dir().join("heycat-wav-test-duration-invalid");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    // Create a non-WAV file
    let path = temp_dir.join("not-a-wav.wav");
    std::fs::write(&path, b"this is not a wav file").unwrap();

    let result = parse_duration_from_file(&path);
    assert!(result.is_err());

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}
