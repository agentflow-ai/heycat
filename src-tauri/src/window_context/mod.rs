// Window context module for context-sensitive commands

mod detector;
mod monitor;
mod resolver;
mod store;
mod types;

pub use detector::{get_active_window, get_running_applications};
pub use monitor::{MonitorConfig, WindowMonitor};
pub use resolver::ContextResolver;
pub use store::{WindowContextStore, WindowContextStoreError};
pub use types::{ActiveWindowInfo, OverrideMode, RunningApplication, WindowContext, WindowMatcher};
