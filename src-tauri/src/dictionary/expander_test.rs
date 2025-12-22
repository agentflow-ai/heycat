use super::*;

fn make_entry(trigger: &str, expansion: &str) -> DictionaryEntry {
    DictionaryEntry {
        id: format!("test-{}", trigger),
        trigger: trigger.to_string(),
        expansion: expansion.to_string(),
        suffix: None,
        auto_enter: false,
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
    assert_eq!(expander.expand("brb"), "be right back");
    assert_eq!(expander.expand("BRB"), "be right back");
    assert_eq!(expander.expand("Brb"), "be right back");

    // Whole-word only: "api" inside "capitalize" should NOT match
    assert_eq!(expander.expand("capitalize"), "capitalize");

    // But standalone "api" should match
    assert_eq!(expander.expand("check the api"), "check the API");
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
        expander.expand("i need to brb and check the api docs"),
        "i need to be right back and check the API docs"
    );
}

#[test]
fn test_punctuation_adjacent_triggers() {
    // Test case: "brb." and "brb," expand correctly
    let entries = vec![make_entry("brb", "be right back")];
    let expander = DictionaryExpander::new(&entries);

    assert_eq!(expander.expand("brb."), "be right back.");
    assert_eq!(expander.expand("brb,"), "be right back,");
    assert_eq!(expander.expand("brb!"), "be right back!");
    assert_eq!(expander.expand("(brb)"), "(be right back)");
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
    assert_eq!(expander.expand(original), original);
}

#[test]
fn test_empty_entries_returns_original() {
    // Edge case: No entries means no expansions
    let expander = DictionaryExpander::new(&[]);

    let original = "brb i need to check something";
    assert_eq!(expander.expand(original), original);
}

#[test]
fn test_empty_text_returns_empty() {
    let entries = vec![make_entry("brb", "be right back")];
    let expander = DictionaryExpander::new(&entries);

    assert_eq!(expander.expand(""), "");
}
