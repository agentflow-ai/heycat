// Fuzzy matcher - matches transcribed text against registered commands

use crate::voice_commands::registry::{CommandDefinition, CommandRegistry};
use serde::Serialize;
use strsim::normalized_levenshtein;
use std::collections::HashMap;
use uuid::Uuid;

/// Default similarity threshold for fuzzy matching (0.0 to 1.0)
pub const DEFAULT_THRESHOLD: f64 = 0.8;

/// Result of matching transcribed text against commands
#[derive(Debug, Clone, Serialize)]
pub enum MatchResult {
    /// Exact match found
    Exact {
        command: MatchedCommand,
        parameters: HashMap<String, String>,
    },
    /// Fuzzy match found with confidence score
    Fuzzy {
        command: MatchedCommand,
        score: f64,
        parameters: HashMap<String, String>,
    },
    /// Multiple commands match with similar confidence
    Ambiguous {
        candidates: Vec<MatchCandidate>,
    },
    /// No match found
    NoMatch,
}

/// A matched command with essential info
#[derive(Debug, Clone, Serialize)]
pub struct MatchedCommand {
    pub id: Uuid,
    pub trigger: String,
}

/// A candidate match with score
#[derive(Debug, Clone, Serialize)]
pub struct MatchCandidate {
    pub command: MatchedCommand,
    pub score: f64,
    pub parameters: HashMap<String, String>,
}

/// Configuration for the matcher
#[derive(Debug, Clone)]
pub struct MatcherConfig {
    /// Minimum similarity score for a fuzzy match (0.0 to 1.0)
    pub threshold: f64,
    /// Maximum difference between top matches to consider ambiguous
    pub ambiguity_delta: f64,
}

impl Default for MatcherConfig {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_THRESHOLD,
            ambiguity_delta: 0.1,
        }
    }
}

/// Command matcher using exact and fuzzy matching
pub struct CommandMatcher {
    config: MatcherConfig,
}

impl Default for CommandMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandMatcher {
    /// Create a new matcher with default configuration
    pub fn new() -> Self {
        Self {
            config: MatcherConfig::default(),
        }
    }

    /// Create a matcher with custom configuration
    #[allow(dead_code)]
    pub fn with_config(config: MatcherConfig) -> Self {
        Self { config }
    }

    /// Normalize input text: lowercase and trim whitespace
    fn normalize(input: &str) -> String {
        input.trim().to_lowercase()
    }

    /// Try to extract parameters from a parameterized trigger
    /// Returns (matched, parameters) if the trigger pattern matches
    fn try_extract_params(
        input: &str,
        trigger: &str,
    ) -> Option<(bool, HashMap<String, String>)> {
        // Check for parameterized pattern like "type {text}"
        if !trigger.contains('{') {
            return None;
        }

        // Split trigger into prefix and parameter name
        let parts: Vec<&str> = trigger.splitn(2, '{').collect();
        if parts.len() != 2 {
            return None;
        }

        let prefix = parts[0].trim();
        let param_part = parts[1];

        // Extract parameter name (remove trailing })
        let param_name = param_part.trim_end_matches('}').trim();
        if param_name.is_empty() {
            return None;
        }

        // Check if input starts with the prefix
        let normalized_input = Self::normalize(input);
        let normalized_prefix = Self::normalize(prefix);

        if !normalized_input.starts_with(&normalized_prefix) {
            return None;
        }

        // Extract the parameter value
        let param_value = input[prefix.len()..].trim().to_string();

        let mut params = HashMap::new();
        params.insert(param_name.to_string(), param_value);

        Some((true, params))
    }

    /// Match input against a single command
    fn match_command(
        &self,
        input: &str,
        command: &CommandDefinition,
    ) -> Option<MatchCandidate> {
        if !command.enabled {
            return None;
        }

        let normalized_input = Self::normalize(input);
        let normalized_trigger = Self::normalize(&command.trigger);

        // Try parameterized match first
        if let Some((_, params)) = Self::try_extract_params(input, &command.trigger) {
            return Some(MatchCandidate {
                command: MatchedCommand {
                    id: command.id,
                    trigger: command.trigger.clone(),
                },
                score: 1.0,
                parameters: params,
            });
        }

        // Exact match
        if normalized_input == normalized_trigger {
            return Some(MatchCandidate {
                command: MatchedCommand {
                    id: command.id,
                    trigger: command.trigger.clone(),
                },
                score: 1.0,
                parameters: HashMap::new(),
            });
        }

        // Fuzzy match using normalized Levenshtein distance
        let score = normalized_levenshtein(&normalized_input, &normalized_trigger);

        if score >= self.config.threshold {
            Some(MatchCandidate {
                command: MatchedCommand {
                    id: command.id,
                    trigger: command.trigger.clone(),
                },
                score,
                parameters: HashMap::new(),
            })
        } else {
            None
        }
    }

    /// Match input against all commands in the registry
    pub fn match_input(&self, input: &str, registry: &CommandRegistry) -> MatchResult {
        let commands: Vec<_> = registry.list().iter().map(|c| (*c).clone()).collect();
        self.match_commands(input, &commands)
    }

    /// Match input against a slice of commands
    ///
    /// This method is useful when you have a pre-filtered list of commands,
    /// such as context-resolved commands from ContextResolver.
    pub fn match_commands(&self, input: &str, commands: &[CommandDefinition]) -> MatchResult {
        // Collect all matches, filtering out any with NaN scores (defensive)
        let mut candidates: Vec<MatchCandidate> = commands
            .iter()
            .filter_map(|cmd| self.match_command(input, cmd))
            .filter(|c| c.score.is_finite()) // Filter out NaN/Inf scores
            .collect();

        // Sort by score (highest first) using total_cmp for correct NaN handling
        candidates.sort_by(|a, b| b.score.total_cmp(&a.score));

        match candidates.len() {
            0 => MatchResult::NoMatch,
            1 => {
                let candidate = candidates.remove(0);
                if candidate.score >= 1.0 - f64::EPSILON {
                    MatchResult::Exact {
                        command: candidate.command,
                        parameters: candidate.parameters,
                    }
                } else {
                    MatchResult::Fuzzy {
                        command: candidate.command,
                        score: candidate.score,
                        parameters: candidate.parameters,
                    }
                }
            }
            _ => {
                // Check if top matches are too close (ambiguous)
                let top_score = candidates[0].score;
                let close_matches: Vec<_> = candidates
                    .iter()
                    .filter(|c| top_score - c.score <= self.config.ambiguity_delta)
                    .cloned()
                    .collect();

                if close_matches.len() > 1 {
                    MatchResult::Ambiguous {
                        candidates: close_matches,
                    }
                } else {
                    let candidate = candidates.remove(0);
                    if candidate.score >= 1.0 - f64::EPSILON {
                        MatchResult::Exact {
                            command: candidate.command,
                            parameters: candidate.parameters,
                        }
                    } else {
                        MatchResult::Fuzzy {
                            command: candidate.command,
                            score: candidate.score,
                            parameters: candidate.parameters,
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "matcher_test.rs"]
mod tests;
