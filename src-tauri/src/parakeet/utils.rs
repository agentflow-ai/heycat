// Shared utilities for Parakeet transcription
// Contains workarounds and helper functions for parakeet-rs integration

use parakeet_rs::TimedToken;

/// Workaround for parakeet-rs v0.2.5 bug where `TranscribeResult.text`
/// incorrectly joins tokens with spaces (`.join(" ")`).
///
/// Instead, we concatenate tokens directly - they already have leading
/// spaces at word boundaries (from SentencePiece ▁ marker).
///
/// # Arguments
/// * `tokens` - Slice of tokens from the transcription result
///
/// # Returns
/// Properly formatted transcription text with trimmed whitespace
///
/// # Example
/// ```ignore
/// let result = tdt.transcribe_file(path, None)?;
/// let text = fix_parakeet_text(&result.tokens);
/// ```
///
/// TODO: Remove when parakeet-rs fixes this issue upstream
/// Tracking: https://github.com/nvidia-riva/parakeet/issues/XXX (parakeet-rs v0.2.5)
pub fn fix_parakeet_text(tokens: &[TimedToken]) -> String {
    tokens
        .iter()
        .map(|t| t.text.as_str())
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
#[path = "utils_test.rs"]
mod tests;
