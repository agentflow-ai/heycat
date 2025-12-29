// Tests for voice command matching
//
// Note: These tests use match_commands() directly with command slices,
// avoiding the need for SpacetimeDB integration.

use super::*;
use crate::voice_commands::registry::{ActionType, CommandDefinition};

fn create_command(trigger: &str) -> CommandDefinition {
    CommandDefinition {
        id: Uuid::new_v4(),
        trigger: trigger.to_string(),
        action_type: ActionType::OpenApp,
        parameters: HashMap::new(),
        enabled: true,
    }
}

#[test]
fn test_exact_match_open_slack() {
    let cmd = create_command("open slack");
    let commands = vec![cmd.clone()];

    let matcher = CommandMatcher::new();
    let result = matcher.match_commands("open slack", &commands);

    match result {
        MatchResult::Exact { command, .. } => {
            assert_eq!(command.trigger, "open slack");
        }
        _ => panic!("Expected Exact match, got {:?}", result),
    }
}

#[test]
fn test_fuzzy_match_typo() {
    let cmd = create_command("open slack");
    let commands = vec![cmd.clone()];

    let matcher = CommandMatcher::new();
    let result = matcher.match_commands("opn slack", &commands);

    match result {
        MatchResult::Fuzzy { command, score, .. } => {
            assert_eq!(command.trigger, "open slack");
            assert!(score >= 0.8, "Score {} should be >= 0.8", score);
        }
        _ => panic!("Expected Fuzzy match, got {:?}", result),
    }
}

#[test]
fn test_case_insensitive_match() {
    let cmd = create_command("open slack");
    let commands = vec![cmd.clone()];

    let matcher = CommandMatcher::new();
    let result = matcher.match_commands("OPEN SLACK", &commands);

    match result {
        MatchResult::Exact { command, .. } => {
            assert_eq!(command.trigger, "open slack");
        }
        _ => panic!("Expected Exact match for case variation, got {:?}", result),
    }
}

#[test]
fn test_parameter_extraction() {
    let mut cmd = create_command("type {text}");
    cmd.action_type = ActionType::TypeText;
    let commands = vec![cmd.clone()];

    let matcher = CommandMatcher::new();
    let result = matcher.match_commands("type hello world", &commands);

    match result {
        MatchResult::Exact { parameters, .. } => {
            assert_eq!(parameters.get("text"), Some(&"hello world".to_string()));
        }
        _ => panic!("Expected Exact match with parameters, got {:?}", result),
    }
}

#[test]
fn test_no_match_different_text() {
    let cmd = create_command("open slack");
    let commands = vec![cmd.clone()];

    let matcher = CommandMatcher::new();
    let result = matcher.match_commands("xyz abc", &commands);

    assert!(matches!(result, MatchResult::NoMatch));
}

#[test]
fn test_ambiguous_similar_commands() {
    // Use commands that are very similar to each other
    let cmd1 = create_command("open slack");
    let cmd2 = create_command("open slick");
    let commands = vec![cmd1, cmd2];

    // Configure matcher with higher ambiguity delta to make the test more reliable
    let config = MatcherConfig {
        threshold: 0.7,
        ambiguity_delta: 0.15,
    };
    let matcher = CommandMatcher::with_config(config);
    // Input that's similar to both: "slaik" is between "slack" and "slick"
    let result = matcher.match_commands("open slaik", &commands);

    match result {
        MatchResult::Ambiguous { candidates } => {
            assert!(candidates.len() >= 2, "Expected at least 2 ambiguous candidates");
        }
        _ => panic!("Expected Ambiguous result, got {:?}", result),
    }
}

#[test]
fn test_whitespace_normalization() {
    let cmd = create_command("open slack");
    let commands = vec![cmd.clone()];

    let matcher = CommandMatcher::new();
    let result = matcher.match_commands("  open slack  ", &commands);

    match result {
        MatchResult::Exact { command, .. } => {
            assert_eq!(command.trigger, "open slack");
        }
        _ => panic!("Expected Exact match with trimmed input, got {:?}", result),
    }
}

#[test]
fn test_disabled_command_not_matched() {
    let mut cmd = create_command("open slack");
    cmd.enabled = false;
    let commands = vec![cmd.clone()];

    let matcher = CommandMatcher::new();
    let result = matcher.match_commands("open slack", &commands);

    assert!(matches!(result, MatchResult::NoMatch));
}

#[test]
fn test_empty_commands_returns_no_match() {
    let commands: Vec<CommandDefinition> = vec![];

    let matcher = CommandMatcher::new();
    let result = matcher.match_commands("open slack", &commands);

    assert!(matches!(result, MatchResult::NoMatch));
}

#[test]
fn test_custom_threshold() {
    let cmd = create_command("open slack");
    let commands = vec![cmd.clone()];

    // Set a very high threshold that won't match fuzzy
    let config = MatcherConfig {
        threshold: 0.99,
        ambiguity_delta: 0.1,
    };
    let matcher = CommandMatcher::with_config(config);
    let result = matcher.match_commands("opn slack", &commands);

    // With high threshold, fuzzy match shouldn't work
    assert!(matches!(result, MatchResult::NoMatch));
}

#[test]
fn test_best_match_selected_when_not_ambiguous() {
    let cmd1 = create_command("open slack");
    let cmd2 = create_command("open zoom");
    let commands = vec![cmd1, cmd2];

    let matcher = CommandMatcher::new();
    // "open slack" should match exactly, not be ambiguous with "open zoom"
    let result = matcher.match_commands("open slack", &commands);

    match result {
        MatchResult::Exact { command, .. } => {
            assert_eq!(command.trigger, "open slack");
        }
        _ => panic!("Expected Exact match, got {:?}", result),
    }
}
