// Text normalization for transcription output
// Handles SentencePiece-style tokenization and cleans up spacing

/// Normalize transcription output by joining subword tokens
///
/// Handles SentencePiece-style tokenization where:
/// - "▁" prefix indicates start of a new word
/// - Tokens without "▁" are continuations of the previous word
///
/// Also handles cases where tokens are space-separated without markers,
/// specifically the pattern where words are split character-by-character
/// like "T est ing" -> "Testing"
pub fn normalize_transcription(text: &str) -> String {
    // First, handle SentencePiece markers if present
    if text.contains('▁') {
        // SentencePiece style: ▁ marks word boundaries
        return text
            .replace("▁", " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
    }

    // No SentencePiece markers - check if we need to join split tokens
    // Only apply joining if we detect the split pattern (single char followed by lowercase)
    if has_split_pattern(text) {
        return join_split_tokens(text);
    }

    // No special handling needed - just normalize whitespace
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Detect if text has the split token pattern
/// Pattern: single uppercase letter followed by space and lowercase letters
fn has_split_pattern(text: &str) -> bool {
    let tokens: Vec<&str> = text.split_whitespace().collect();

    for window in tokens.windows(2) {
        let first = window[0];
        let second = window[1];

        // Check for single uppercase letter followed by lowercase token
        if first.len() == 1
            && first.chars().next().map_or(false, |c| c.is_uppercase())
            && second.chars().all(|c| c.is_lowercase())
        {
            return true;
        }
    }

    false
}

/// Join tokens that appear to be incorrectly split
///
/// Strategy: A new word starts when token:
/// - Starts with uppercase AND is more than 1 character (e.g., "Hello")
/// - OR is a standalone word token
///
/// Tokens are joined when:
/// - All lowercase (suffix like "est", "ing")
/// - Single character (split initial)
/// - Punctuation
fn join_split_tokens(text: &str) -> String {
    let tokens: Vec<&str> = text.split_whitespace().collect();

    if tokens.is_empty() {
        return String::new();
    }

    let mut result = String::new();

    for (i, token) in tokens.iter().enumerate() {
        if i == 0 {
            result.push_str(token);
            continue;
        }

        // Determine if this token starts a new word
        let starts_new_word = is_word_start(token);

        if starts_new_word {
            result.push(' ');
        }

        result.push_str(token);
    }

    result
}

/// Determine if a token represents the start of a new word
fn is_word_start(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }

    let first_char = token.chars().next().unwrap();

    // Punctuation is never a new word start (joins to previous)
    if first_char.is_ascii_punctuation() {
        return false;
    }

    // Single characters are continuations (not new word starts)
    if token.len() == 1 {
        return false;
    }

    // All lowercase tokens are continuations (suffixes)
    if token.chars().all(|c| c.is_lowercase()) {
        return false;
    }

    // Multi-char tokens starting with uppercase are new words
    if first_char.is_uppercase() && token.len() > 1 {
        return true;
    }

    // Default: treat as new word
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // Core functionality tests

    #[test]
    fn test_normalize_sentencepiece_style() {
        assert_eq!(normalize_transcription("▁Hello▁world"), "Hello world");
    }

    #[test]
    fn test_normalize_split_tokens() {
        // The main use case: "T est ing" -> "Testing"
        assert_eq!(normalize_transcription("T est ing"), "Testing");
    }

    #[test]
    fn test_normalize_split_with_punctuation() {
        assert_eq!(normalize_transcription("T est ing ."), "Testing.");
    }

    #[test]
    fn test_normalize_split_words() {
        // Individual split words (the actual parakeet-rs output pattern)
        assert_eq!(normalize_transcription("H ello"), "Hello");
        assert_eq!(normalize_transcription("W orld"), "World");
        assert_eq!(normalize_transcription("T est ing"), "Testing");
    }

    #[test]
    fn test_normalize_already_correct() {
        // Should NOT modify already correct text
        assert_eq!(normalize_transcription("Hello world"), "Hello world");
    }

    #[test]
    fn test_normalize_collapses_spaces() {
        assert_eq!(normalize_transcription("Hello   world"), "Hello world");
    }

    #[test]
    fn test_normalize_trims() {
        assert_eq!(normalize_transcription("  Hello world  "), "Hello world");
    }

    #[test]
    fn test_normalize_empty() {
        assert_eq!(normalize_transcription(""), "");
    }

    #[test]
    fn test_normalize_preserves_punctuation_in_sentence() {
        // Already correct text with punctuation should stay correct
        assert_eq!(normalize_transcription("Hello, world!"), "Hello, world!");
    }

    #[test]
    fn test_normalize_mixed_case_words() {
        // Words starting with capitals should stay separate
        assert_eq!(
            normalize_transcription("The Quick Brown Fox"),
            "The Quick Brown Fox"
        );
    }

    // Pattern detection tests

    #[test]
    fn test_has_split_pattern_true() {
        assert!(has_split_pattern("T est ing"));
        assert!(has_split_pattern("H ello"));
    }

    #[test]
    fn test_has_split_pattern_false() {
        assert!(!has_split_pattern("Hello world"));
        assert!(!has_split_pattern("The Quick Brown"));
    }

    // Word start detection tests

    #[test]
    fn test_is_word_start_uppercase_multichar() {
        assert!(is_word_start("Hello"));
        assert!(is_word_start("The"));
    }

    #[test]
    fn test_is_word_start_single_char() {
        // Single chars are NOT word starts (they're continuations)
        assert!(!is_word_start("T"));
        assert!(!is_word_start("a"));
    }

    #[test]
    fn test_is_word_start_lowercase() {
        // All lowercase are continuations (suffixes)
        assert!(!is_word_start("est"));
        assert!(!is_word_start("ing"));
        assert!(!is_word_start("world"));
    }

    #[test]
    fn test_is_word_start_punctuation() {
        assert!(!is_word_start("."));
        assert!(!is_word_start("!"));
    }
}
