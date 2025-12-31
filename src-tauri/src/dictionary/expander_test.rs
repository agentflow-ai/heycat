use super::*;

fn make_entry(trigger: &str, expansion: &str) -> DictionaryEntry {
    DictionaryEntry {
        id: format!("test-{}", trigger),
        trigger: trigger.to_string(),
        expansion: expansion.to_string(),
        suffix: None,
        auto_enter: false,
        disable_suffix: false,
        complete_match_only: false,
    }
}

fn make_entry_with_suffix(trigger: &str, expansion: &str, suffix: &str) -> DictionaryEntry {
    DictionaryEntry {
        id: format!("test-{}", trigger),
        trigger: trigger.to_string(),
        expansion: expansion.to_string(),
        suffix: Some(suffix.to_string()),
        auto_enter: false,
        disable_suffix: false,
        complete_match_only: false,
    }
}

fn make_entry_with_auto_enter(trigger: &str, expansion: &str) -> DictionaryEntry {
    DictionaryEntry {
        id: format!("test-{}", trigger),
        trigger: trigger.to_string(),
        expansion: expansion.to_string(),
        suffix: None,
        auto_enter: true,
        disable_suffix: false,
        complete_match_only: false,
    }
}

#[test]
fn test_case_insensitive_whole_word_matching() {
    // Test case: "brb"/"BRB"/"Brb" all expand, "api" not matched in "capitalize"
    let entries = vec![
        make_entry("brb", "be right back"),
        make_entry("api", "API"),
    ];
    let expander = DictionaryExpander::new(&entries);

    // Case variations all match
    assert_eq!(expander.expand("brb").expanded_text, "be right back");
    assert_eq!(expander.expand("BRB").expanded_text, "be right back");
    assert_eq!(expander.expand("Brb").expanded_text, "be right back");

    // Whole-word only: "api" inside "capitalize" should NOT match
    assert_eq!(expander.expand("capitalize").expanded_text, "capitalize");

    // But standalone "api" should match
    assert_eq!(expander.expand("check the api").expanded_text, "check the API");
}

#[test]
fn test_multiple_entries_expand_in_single_pass() {
    // Test case: "brb" and "api" both replaced
    let entries = vec![
        make_entry("brb", "be right back"),
        make_entry("api", "API"),
    ];
    let expander = DictionaryExpander::new(&entries);

    assert_eq!(
        expander.expand("i need to brb and check the api docs").expanded_text,
        "i need to be right back and check the API docs"
    );
}

#[test]
fn test_punctuation_adjacent_triggers() {
    // Test case: "brb." and "brb," expand correctly
    let entries = vec![make_entry("brb", "be right back")];
    let expander = DictionaryExpander::new(&entries);

    assert_eq!(expander.expand("brb.").expanded_text, "be right back.");
    assert_eq!(expander.expand("brb,").expanded_text, "be right back,");
    assert_eq!(expander.expand("brb!").expanded_text, "be right back!");
    assert_eq!(expander.expand("(brb)").expanded_text, "(be right back)");
}

#[test]
fn test_no_triggers_returns_original() {
    // Test case: No triggers in text: original returned unchanged
    let entries = vec![
        make_entry("brb", "be right back"),
        make_entry("api", "API"),
    ];
    let expander = DictionaryExpander::new(&entries);

    let original = "this text has no matching triggers";
    assert_eq!(expander.expand(original).expanded_text, original);
}

#[test]
fn test_empty_entries_returns_original() {
    // Edge case: No entries means no expansions
    let expander = DictionaryExpander::new(&[]);

    let original = "brb i need to check something";
    assert_eq!(expander.expand(original).expanded_text, original);
}

#[test]
fn test_empty_text_returns_empty() {
    let entries = vec![make_entry("brb", "be right back")];
    let expander = DictionaryExpander::new(&entries);

    assert_eq!(expander.expand("").expanded_text, "");
}

// ============================================================================
// Suffix and auto_enter tests
// ============================================================================

#[test]
fn test_expand_with_suffix() {
    // Test case: Expand "brb" with suffix "." → "be right back."
    let entries = vec![make_entry_with_suffix("brb", "be right back", ".")];
    let expander = DictionaryExpander::new(&entries);

    let result = expander.expand("I'll brb");
    assert_eq!(result.expanded_text, "I'll be right back.");
    assert!(!result.should_press_enter);
}

#[test]
fn test_expand_without_suffix() {
    // Test case: Expand "brb" without suffix → "be right back"
    let entries = vec![make_entry("brb", "be right back")];
    let expander = DictionaryExpander::new(&entries);

    let result = expander.expand("brb");
    assert_eq!(result.expanded_text, "be right back");
    assert!(!result.should_press_enter);
}

#[test]
fn test_expand_with_auto_enter() {
    // Test case: Expand "brb" with auto_enter=true → should_press_enter is true
    let entries = vec![make_entry_with_auto_enter("sig", "Best regards, Michael")];
    let expander = DictionaryExpander::new(&entries);

    let result = expander.expand("sig");
    assert_eq!(result.expanded_text, "Best regards, Michael");
    assert!(result.should_press_enter);
}

#[test]
fn test_expand_multiple_entries_one_auto_enter() {
    // Test case: Multiple triggers, one has auto_enter → should_press_enter is true
    let entries = vec![
        make_entry("brb", "be right back"),
        make_entry_with_auto_enter("sig", "Best regards"),
    ];
    let expander = DictionaryExpander::new(&entries);

    let result = expander.expand("brb sig");
    assert_eq!(result.expanded_text, "be right back Best regards");
    assert!(result.should_press_enter); // sig has auto_enter
}

#[test]
fn test_expand_no_match_returns_false() {
    // Test case: No matches → should_press_enter is false
    let entries = vec![make_entry_with_auto_enter("brb", "be right back")];
    let expander = DictionaryExpander::new(&entries);

    let result = expander.expand("hello world");
    assert_eq!(result.expanded_text, "hello world");
    assert!(!result.should_press_enter); // No match, no auto_enter
}

// ============================================================================
// disable_suffix tests
// ============================================================================

fn make_entry_with_disable_suffix(trigger: &str, expansion: &str) -> DictionaryEntry {
    DictionaryEntry {
        id: format!("test-{}", trigger),
        trigger: trigger.to_string(),
        expansion: expansion.to_string(),
        suffix: None,
        auto_enter: false,
        disable_suffix: true,
        complete_match_only: false,
    }
}

#[test]
fn test_expand_with_disable_suffix_strips_trailing_punctuation() {
    // When disable_suffix is true, trailing punctuation after trigger should be stripped
    let entries = vec![make_entry_with_disable_suffix("brb", "be right back")];
    let expander = DictionaryExpander::new(&entries);

    // With disable_suffix, the trailing period from transcription is stripped
    assert_eq!(expander.expand("brb.").expanded_text, "be right back");
    assert_eq!(expander.expand("brb!").expanded_text, "be right back");
    assert_eq!(expander.expand("brb?").expanded_text, "be right back");
    assert_eq!(expander.expand("brb,").expanded_text, "be right back");
    assert_eq!(expander.expand("brb;").expanded_text, "be right back");
    assert_eq!(expander.expand("brb:").expanded_text, "be right back");
}

#[test]
fn test_expand_with_disable_suffix_no_trailing_punctuation() {
    // When disable_suffix is true and there's no trailing punctuation, works normally
    let entries = vec![make_entry_with_disable_suffix("brb", "be right back")];
    let expander = DictionaryExpander::new(&entries);

    assert_eq!(expander.expand("brb").expanded_text, "be right back");
    assert_eq!(
        expander.expand("I'll brb").expanded_text,
        "I'll be right back"
    );
}

#[test]
fn test_expand_with_disable_suffix_multiple_punctuation() {
    // When disable_suffix is true, multiple trailing punctuation marks are stripped
    let entries = vec![make_entry_with_disable_suffix("brb", "be right back")];
    let expander = DictionaryExpander::new(&entries);

    assert_eq!(expander.expand("brb...").expanded_text, "be right back");
    assert_eq!(expander.expand("brb!?").expanded_text, "be right back");
}

#[test]
fn test_expand_without_disable_suffix_preserves_punctuation() {
    // When disable_suffix is false (default), punctuation is preserved
    let entries = vec![make_entry("brb", "be right back")];
    let expander = DictionaryExpander::new(&entries);

    assert_eq!(expander.expand("brb.").expanded_text, "be right back.");
    assert_eq!(expander.expand("brb!").expanded_text, "be right back!");
}

#[test]
fn test_expand_disable_suffix_ignores_explicit_suffix() {
    // When disable_suffix is true, any explicit suffix field is ignored
    let entry = DictionaryEntry {
        id: "test".to_string(),
        trigger: "brb".to_string(),
        expansion: "be right back".to_string(),
        suffix: Some(".".to_string()), // This should be ignored
        auto_enter: false,
        disable_suffix: true, // This takes precedence
        complete_match_only: false,
    };
    let expander = DictionaryExpander::new(&[entry]);

    // The suffix "." should NOT be added because disable_suffix is true
    assert_eq!(expander.expand("brb").expanded_text, "be right back");
}

#[test]
fn test_expand_with_suffix_and_disable_suffix_false() {
    // Verify normal suffix behavior when disable_suffix is false
    let entry = DictionaryEntry {
        id: "test".to_string(),
        trigger: "brb".to_string(),
        expansion: "be right back".to_string(),
        suffix: Some(".".to_string()),
        auto_enter: false,
        disable_suffix: false,
        complete_match_only: false,
    };
    let expander = DictionaryExpander::new(&[entry]);

    // The suffix "." should be added
    assert_eq!(expander.expand("brb").expanded_text, "be right back.");
}

#[test]
fn test_expand_disable_suffix_in_sentence() {
    // Test disable_suffix within a sentence context
    let entries = vec![make_entry_with_disable_suffix("brb", "be right back")];
    let expander = DictionaryExpander::new(&entries);

    // Punctuation after "brb" is stripped, rest of sentence preserved
    assert_eq!(
        expander.expand("I'll brb. Talk soon").expanded_text,
        "I'll be right back Talk soon"
    );
}

#[test]
fn test_expand_disable_suffix_case_insensitive() {
    // Test disable_suffix with case-insensitive matching
    // This is the bug: "Clear?" should become "/clear" not "/clear?"
    let entries = vec![make_entry_with_disable_suffix("clear", "/clear")];
    let expander = DictionaryExpander::new(&entries);

    // Case variations with punctuation should all strip the punctuation
    assert_eq!(expander.expand("clear?").expanded_text, "/clear");
    assert_eq!(expander.expand("Clear?").expanded_text, "/clear");
    assert_eq!(expander.expand("CLEAR?").expanded_text, "/clear");
    assert_eq!(expander.expand("Clear.").expanded_text, "/clear");
    assert_eq!(expander.expand("Clear!").expanded_text, "/clear");
}
