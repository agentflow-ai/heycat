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

/// Compiled pattern for a single dictionary entry
struct CompiledPattern {
    regex: Regex,
    entry: DictionaryEntry,
}

/// Expander that applies dictionary expansions to text
pub struct DictionaryExpander {
    patterns: Vec<CompiledPattern>,
}

impl DictionaryExpander {
    /// Create a new expander from a list of dictionary entries
    /// Pre-compiles regex patterns for each entry for efficient reuse
    pub fn new(entries: &[DictionaryEntry]) -> Self {
        let patterns = entries
            .iter()
            .filter_map(|entry| {
                // Build case-insensitive, whole-word pattern
                let pattern = format!(r"(?i)\b{}\b", regex::escape(&entry.trigger));
                match Regex::new(&pattern) {
                    Ok(regex) => Some(CompiledPattern {
                        regex,
                        entry: entry.clone(),
                    }),
                    Err(e) => {
                        crate::warn!(
                            "Failed to compile regex for trigger '{}': {}",
                            entry.trigger,
                            e
                        );
                        None
                    }
                }
            })
            .collect();

        Self { patterns }
    }

    /// Apply all expansions to the input text
    /// Returns ExpansionResult with expanded text and whether enter should be pressed
    pub fn expand(&self, text: &str) -> ExpansionResult {
        let mut result = text.to_string();
        let mut should_press_enter = false;

        for pattern in &self.patterns {
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
