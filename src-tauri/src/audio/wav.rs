// WAV encoding module for saving audio samples to disk

use std::path::{Path, PathBuf};

/// Errors that can occur during WAV encoding
#[derive(Debug, Clone, PartialEq)]
pub enum WavEncodingError {
    /// I/O error (directory creation, file write)
    IoError(String),
    /// Error during WAV encoding
    EncodingError(String),
    /// Invalid input (empty samples, NaN/infinity values)
    InvalidInput(String),
}

impl std::fmt::Display for WavEncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WavEncodingError::IoError(msg) => write!(f, "I/O error: {}", msg),
            WavEncodingError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            WavEncodingError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for WavEncodingError {}

/// Convert a hound error to WavEncodingError
#[cfg_attr(coverage_nightly, coverage(off))]
fn hound_error(e: hound::Error) -> WavEncodingError {
    WavEncodingError::EncodingError(e.to_string())
}

/// Trait for file system operations (allows mocking in tests)
pub trait FileWriter {
    /// Get the output directory path
    fn output_dir(&self) -> PathBuf;

    /// Generate a unique filename with timestamp
    fn generate_filename(&self) -> String;

    /// Create directory and all parent directories
    fn create_dir_all(&self, path: &Path) -> Result<(), std::io::Error>;

    /// Check if a path exists
    fn path_exists(&self, path: &Path) -> bool;
}

/// Production file writer using system paths and real filesystem
///
/// Supports worktree-specific recordings by accepting a pre-computed recordings directory.
pub struct SystemFileWriter {
    recordings_dir: PathBuf,
}

impl SystemFileWriter {
    /// Create a new SystemFileWriter with a specific recordings directory
    pub fn new(recordings_dir: PathBuf) -> Self {
        Self { recordings_dir }
    }

    /// Create a SystemFileWriter using the default path with optional worktree context
    pub fn with_worktree_context(
        worktree_context: Option<&crate::worktree::WorktreeContext>,
    ) -> Self {
        let recordings_dir = crate::paths::get_recordings_dir(worktree_context)
            .unwrap_or_else(|_| PathBuf::from(".").join("heycat").join("recordings"));
        Self { recordings_dir }
    }
}

impl FileWriter for SystemFileWriter {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn output_dir(&self) -> PathBuf {
        self.recordings_dir.clone()
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn generate_filename(&self) -> String {
        let now = chrono::Utc::now();
        format!("recording-{}.wav", now.format("%Y-%m-%d-%H%M%S"))
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn create_dir_all(&self, path: &Path) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(path)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

/// Encode audio samples to a WAV file
///
/// # Arguments
/// * `samples` - Audio samples as f32 values (expected range: -1.0 to 1.0)
/// * `sample_rate` - Sample rate in Hz (e.g., 44100)
/// * `writer` - File writer for filesystem operations
///
/// # Returns
/// * `Ok(String)` - Path to the created WAV file
/// * `Err(WavEncodingError)` - If encoding fails
pub fn encode_wav<W: FileWriter>(
    samples: &[f32],
    sample_rate: u32,
    writer: &W,
) -> Result<String, WavEncodingError> {
    // Validate input
    if samples.is_empty() {
        return Err(WavEncodingError::InvalidInput(
            "Cannot encode empty samples".to_string(),
        ));
    }

    if samples.iter().any(|s| !s.is_finite()) {
        return Err(WavEncodingError::InvalidInput(
            "Samples contain NaN or infinity values".to_string(),
        ));
    }

    // Ensure output directory exists
    let output_dir = writer.output_dir();
    if !writer.path_exists(&output_dir) {
        writer
            .create_dir_all(&output_dir)
            .map_err(|e| WavEncodingError::IoError(e.to_string()))?;
    }

    // Generate file path
    let filename = writer.generate_filename();
    let file_path = output_dir.join(&filename);
    crate::info!("Saving recording to: {}", file_path.display());

    // Create WAV writer
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut wav_writer =
        hound::WavWriter::create(&file_path, spec).map_err(hound_error)?;

    // Convert and write samples
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let sample_i16 = (clamped * i16::MAX as f32) as i16;
        wav_writer.write_sample(sample_i16).map_err(hound_error)?;
    }

    // Finalize
    wav_writer.finalize().map_err(hound_error)?;

    Ok(file_path.to_string_lossy().to_string())
}

/// Parse the duration of a WAV file from its header
///
/// # Arguments
/// * `path` - Path to the WAV file
///
/// # Returns
/// * `Ok(f64)` - Duration in seconds
/// * `Err(WavEncodingError)` - If the file cannot be read or is not a valid WAV
pub fn parse_duration_from_file(path: &Path) -> Result<f64, WavEncodingError> {
    let reader = hound::WavReader::open(path).map_err(hound_error)?;
    let spec = reader.spec();
    let num_samples = reader.duration(); // Total samples per channel

    if spec.sample_rate == 0 {
        return Err(WavEncodingError::InvalidInput(
            "WAV file has invalid sample rate of 0".to_string(),
        ));
    }

    let duration_secs = num_samples as f64 / spec.sample_rate as f64;
    Ok(duration_secs)
}
