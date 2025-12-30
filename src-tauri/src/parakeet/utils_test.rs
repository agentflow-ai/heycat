use super::*;

/// Helper to create a TimedToken with the given text
fn make_token(text: &str) -> TimedToken {
    TimedToken {
        text: text.to_string(),
        start: 0.0,
        end: 0.0,
    }
}

#[test]
fn test_fix_parakeet_text_joins_tokens_correctly() {
    // Parakeet tokens have leading spaces for word boundaries (SentencePiece ▁)
    let tokens = vec![
        make_token("hello"),
        make_token(" world"),
        make_token(" test"),
    ];
    let result = fix_parakeet_text(&tokens);
    assert_eq!(result, "hello world test");
}

#[test]
fn test_fix_parakeet_text_trims_whitespace() {
    let tokens = vec![
        make_token("  leading"),
        make_token(" and"),
        make_token(" trailing  "),
    ];
    let result = fix_parakeet_text(&tokens);
    assert_eq!(result, "leading and trailing");
}

#[test]
fn test_fix_parakeet_text_handles_empty_tokens() {
    let tokens: Vec<TimedToken> = vec![];
    let result = fix_parakeet_text(&tokens);
    assert_eq!(result, "");
}

#[test]
fn test_fix_parakeet_text_handles_single_token() {
    let tokens = vec![make_token("hello")];
    let result = fix_parakeet_text(&tokens);
    assert_eq!(result, "hello");
}

#[test]
fn test_fix_parakeet_text_preserves_internal_spaces() {
    // When tokens contain internal spaces, they should be preserved
    let tokens = vec![
        make_token("hey"),
        make_token(" cat"),
    ];
    let result = fix_parakeet_text(&tokens);
    assert_eq!(result, "hey cat");
}

#[test]
fn test_fix_parakeet_text_handles_whitespace_only_tokens() {
    let tokens = vec![
        make_token("   "),
        make_token("   "),
    ];
    let result = fix_parakeet_text(&tokens);
    assert_eq!(result, "");
}
