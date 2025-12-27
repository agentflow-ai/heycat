// Window context types for context-sensitive commands

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Information about the currently active window
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActiveWindowInfo {
    pub app_name: String,
    pub bundle_id: Option<String>,
    pub window_title: Option<String>,
    pub pid: u32,
}

/// Information about a running application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RunningApplication {
    /// The localized name of the application
    pub name: String,
    /// The bundle identifier (e.g., "com.apple.Safari")
    pub bundle_id: Option<String>,
    /// Whether this is the currently active (frontmost) application
    pub is_active: bool,
}

/// Pattern for matching windows
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WindowMatcher {
    pub app_name: String,
    pub title_pattern: Option<String>,
    pub bundle_id: Option<String>,
}

/// Override behavior for commands/dictionary
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum OverrideMode {
    #[default]
    Merge,
    Replace,
}

/// A window context definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WindowContext {
    pub id: Uuid,
    pub name: String,
    pub matcher: WindowMatcher,
    pub command_mode: OverrideMode,
    pub dictionary_mode: OverrideMode,
    pub command_ids: Vec<Uuid>,
    pub dictionary_entry_ids: Vec<String>,
    pub enabled: bool,
    pub priority: i32,
}

#[cfg(test)]
#[path = "types_test.rs"]
mod tests;
