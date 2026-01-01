// Dictionary expander - applies dictionary expansions to transcription text
// Uses case-insensitive, whole-word matching with regex

use regex::Regex;

use super::DictionaryEntry;

/// Result of expanding text with dictionary entries
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpansionResult {
    /// The expanded text with all substitutions applied
    pub expanded_text: String,
    /// True if any matched entry had auto_enter enabled
    pub should_press_enter: bool,
}

/// Compiled pattern for a single dictionary entry (partial matching)
struct CompiledPattern {
    regex: Regex,
    entry: DictionaryEntry,
}

/// Entry for complete-match-only triggers
struct CompleteMatchEntry {
    /// Lowercase trigger for case-insensitive comparison
    trigger_lowercase: String,
    /// The original entry
    entry: DictionaryEntry,
}

/// Expander that applies dictionary expansions to text
pub struct DictionaryExpander {
    /// Patterns for partial matching (triggers that can match within text)
    partial_patterns: Vec<CompiledPattern>,
    /// Entries that only match when the trigger is the complete input
    complete_match_entries: Vec<CompleteMatchEntry>,
}

impl DictionaryExpander {
    /// Create a new expander from a list of dictionary entries
    /// Pre-compiles regex patterns for partial entries and stores complete-match entries separately
    pub fn new(entries: &[DictionaryEntry]) -> Self {
        let mut partial_patterns = Vec::new();
        let mut complete_match_entries = Vec::new();

        for entry in entries {
            if entry.complete_match_only {
                // Store as complete-match entry
                complete_match_entries.push(CompleteMatchEntry {
                    trigger_lowercase: entry.trigger.to_lowercase(),
                    entry: entry.clone(),
                });
            } else {
                // Compile regex pattern for partial matching
                let pattern = format!(r"(?i)\b{}\b", regex::escape(&entry.trigger));
                match Regex::new(&pattern) {
                    Ok(regex) => {
                        partial_patterns.push(CompiledPattern {
                            regex,
                            entry: entry.clone(),
                        });
                    }
                    Err(e) => {
                        crate::warn!(
                            "Failed to compile regex for trigger '{}': {}",
                            entry.trigger,
                            e
                        );
                    }
                }
            }
        }

        Self {
            partial_patterns,
            complete_match_entries,
        }
    }

    /// Apply all expansions to the input text
    /// Returns ExpansionResult with expanded text and whether enter should be pressed
    ///
    /// Complete-match entries are checked FIRST. If the trimmed input exactly matches
    /// (case-insensitive) a complete-match trigger, return immediately with expansion.
    /// Otherwise, fall through to partial matching.
    pub fn expand(&self, text: &str) -> ExpansionResult {
        let trimmed = text.trim();
        let trimmed_lowercase = trimmed.to_lowercase();

        // Check complete-match entries FIRST
        for complete_entry in &self.complete_match_entries {
            if trimmed_lowercase == complete_entry.trigger_lowercase {
                // Exact match found - build replacement
                let replacement = if complete_entry.entry.disable_suffix {
                    complete_entry.entry.expansion.clone()
                } else {
                    match &complete_entry.entry.suffix {
                        Some(suffix) => {
                            format!("{}{}", complete_entry.entry.expansion, suffix)
                        }
                        None => complete_entry.entry.expansion.clone(),
                    }
                };

                return ExpansionResult {
                    expanded_text: replacement,
                    should_press_enter: complete_entry.entry.auto_enter,
                };
            }
        }

        // Fall through to partial matching
        let mut result = text.to_string();
        let mut should_press_enter = false;

        for pattern in &self.partial_patterns {
            if pattern.regex.is_match(&result) {
                // Build replacement based on suffix and disable_suffix settings
                let replacement = if pattern.entry.disable_suffix {
                    // When disable_suffix is true, use expansion only (no trailing punctuation)
                    pattern.entry.expansion.clone()
                } else {
                    // Normal behavior: append suffix if present
                    match &pattern.entry.suffix {
                        Some(suffix) => format!("{}{}", pattern.entry.expansion, suffix),
                        None => pattern.entry.expansion.clone(),
                    }
                };

                // When disable_suffix is true, we also need to strip any trailing punctuation
                // that may follow the trigger in the original text
                if pattern.entry.disable_suffix {
                    // Use a capturing regex to also match and remove trailing punctuation
                    let pattern_with_punct =
                        format!(r"(?i)\b{}\b([.!?,;:]*)", regex::escape(&pattern.entry.trigger));
                    if let Ok(punct_regex) = regex::Regex::new(&pattern_with_punct) {
                        result = punct_regex
                            .replace_all(&result, replacement.as_str())
                            .to_string();
                    } else {
                        // Fallback to standard replacement if regex fails
                        result = pattern
                            .regex
                            .replace_all(&result, replacement.as_str())
                            .to_string();
                    }
                } else {
                    result = pattern
                        .regex
                        .replace_all(&result, replacement.as_str())
                        .to_string();
                }

                // Track auto_enter
                if pattern.entry.auto_enter {
                    should_press_enter = true;
                }
            }
        }

        ExpansionResult {
            expanded_text: result,
            should_press_enter,
        }
    }
}

#[cfg(test)]
#[path = "expander_test.rs"]
mod tests;
