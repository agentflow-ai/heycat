// Action executor - dispatches commands to action implementations

use crate::events::{command_events, CommandExecutedPayload, CommandFailedPayload};
use crate::voice_commands::actions::{AppLauncherAction, TextInputAction, WorkflowAction};
use crate::voice_commands::registry::{ActionType, CommandDefinition};
use async_trait::async_trait;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

/// Result of an action execution
#[derive(Debug, Clone, Serialize)]
pub struct ActionResult {
    /// Description of what was done
    pub message: String,
    /// Optional additional data
    pub data: Option<serde_json::Value>,
}

/// Typed error codes for action execution failures
///
/// Using an enum instead of magic strings ensures:
/// - Compile-time validation of error codes
/// - Exhaustive matching in error handlers
/// - Consistent naming across the codebase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionErrorCode {
    /// Missing required parameter
    MissingParam,
    /// Invalid parameter value
    InvalidParameter,
    /// Application not found
    NotFound,
    /// Permission denied (e.g., accessibility)
    PermissionDenied,
    /// General execution error
    ExecutionError,
    /// Failed to create event source
    EventSourceError,
    /// Nested workflows are not supported
    NestedWorkflow,
    /// Async task panicked
    TaskPanic,
    /// Character encoding error
    EncodingError,
    /// Keyboard event creation error
    EventError,
    /// Invalid application name
    InvalidAppName,
    /// Failed to open application
    OpenFailed,
    /// Failed to close application
    CloseFailed,
    /// Failed to parse workflow steps
    ParseError,
    /// Invalid action type in workflow
    InvalidActionType,
    /// A workflow step failed
    StepFailed,
}

impl std::fmt::Display for ActionErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Serialize to get the SCREAMING_SNAKE_CASE representation
        let s = match self {
            ActionErrorCode::MissingParam => "MISSING_PARAM",
            ActionErrorCode::InvalidParameter => "INVALID_PARAMETER",
            ActionErrorCode::NotFound => "NOT_FOUND",
            ActionErrorCode::PermissionDenied => "PERMISSION_DENIED",
            ActionErrorCode::ExecutionError => "EXECUTION_ERROR",
            ActionErrorCode::EventSourceError => "EVENT_SOURCE_ERROR",
            ActionErrorCode::NestedWorkflow => "NESTED_WORKFLOW",
            ActionErrorCode::TaskPanic => "TASK_PANIC",
            ActionErrorCode::EncodingError => "ENCODING_ERROR",
            ActionErrorCode::EventError => "EVENT_ERROR",
            ActionErrorCode::InvalidAppName => "INVALID_APP_NAME",
            ActionErrorCode::OpenFailed => "OPEN_FAILED",
            ActionErrorCode::CloseFailed => "CLOSE_FAILED",
            ActionErrorCode::ParseError => "PARSE_ERROR",
            ActionErrorCode::InvalidActionType => "INVALID_ACTION_TYPE",
            ActionErrorCode::StepFailed => "STEP_FAILED",
        };
        write!(f, "{}", s)
    }
}

/// Error during action execution
#[derive(Debug, Clone, Serialize)]
pub struct ActionError {
    /// Typed error code for categorization
    pub code: ActionErrorCode,
    /// Human-readable error message
    pub message: String,
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ActionError {}

/// Trait for action implementations
#[async_trait]
pub trait Action: Send + Sync {
    /// Execute the action with the given parameters
    async fn execute(&self, parameters: &HashMap<String, String>) -> Result<ActionResult, ActionError>;
}

// CommandExecutedPayload and CommandFailedPayload are imported from events.rs


/// Stub implementation for SystemControl action
pub struct SystemControlAction;

#[async_trait]
impl Action for SystemControlAction {
    async fn execute(&self, parameters: &HashMap<String, String>) -> Result<ActionResult, ActionError> {
        let control = parameters.get("control").ok_or_else(|| ActionError {
            code: ActionErrorCode::MissingParam,
            message: "Missing 'control' parameter".to_string(),
        })?;

        // Stub implementation
        Ok(ActionResult {
            message: format!("Would execute system control: {}", control),
            data: None,
        })
    }
}

// WorkflowAction is imported from actions::workflow and used in ActionDispatcher

/// Stub implementation for Custom action
pub struct CustomAction;

#[async_trait]
impl Action for CustomAction {
    async fn execute(&self, parameters: &HashMap<String, String>) -> Result<ActionResult, ActionError> {
        let script = parameters.get("script").ok_or_else(|| ActionError {
            code: ActionErrorCode::MissingParam,
            message: "Missing 'script' parameter".to_string(),
        })?;

        // Stub implementation
        Ok(ActionResult {
            message: format!("Would execute custom script: {}", script),
            data: None,
        })
    }
}

/// Stub workflow action used to break circular dependency in ActionDispatcher
/// This is used in the base dispatcher that WorkflowAction receives
struct StubWorkflowAction;

#[async_trait]
impl Action for StubWorkflowAction {
    async fn execute(&self, _parameters: &HashMap<String, String>) -> Result<ActionResult, ActionError> {
        Err(ActionError {
            code: ActionErrorCode::NestedWorkflow,
            message: "Nested workflows are not supported".to_string(),
        })
    }
}

/// Action dispatcher - routes commands to their implementations
pub struct ActionDispatcher {
    open_app: Arc<dyn Action>,
    type_text: Arc<dyn Action>,
    system_control: Arc<dyn Action>,
    workflow: Arc<dyn Action>,
    custom: Arc<dyn Action>,
}

impl Default for ActionDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionDispatcher {
    /// Create a new dispatcher with default action implementations
    ///
    /// Note: For workflow execution, a base dispatcher is created first to avoid
    /// circular dependencies. This means deeply nested workflows (workflow within
    /// workflow) will use simplified action routing.
    pub fn new() -> Self {
        // Create base actions that don't need the dispatcher
        let open_app: Arc<dyn Action> = Arc::new(AppLauncherAction::new());
        let type_text: Arc<dyn Action> = Arc::new(TextInputAction::new());
        let system_control: Arc<dyn Action> = Arc::new(SystemControlAction);
        let custom: Arc<dyn Action> = Arc::new(CustomAction);

        // Create a base dispatcher for workflow execution (with stub workflow to break circular dep)
        let base_dispatcher = Arc::new(Self {
            open_app: open_app.clone(),
            type_text: type_text.clone(),
            system_control: system_control.clone(),
            workflow: Arc::new(StubWorkflowAction),
            custom: custom.clone(),
        });

        // Create the real workflow action with the base dispatcher
        let workflow: Arc<dyn Action> = Arc::new(WorkflowAction::new(base_dispatcher));

        Self {
            open_app,
            type_text,
            system_control,
            workflow,
            custom,
        }
    }

    /// Create a dispatcher with custom action implementations (for testing)
    #[allow(dead_code)]
    pub fn with_actions(
        open_app: Arc<dyn Action>,
        type_text: Arc<dyn Action>,
        system_control: Arc<dyn Action>,
        workflow: Arc<dyn Action>,
        custom: Arc<dyn Action>,
    ) -> Self {
        Self {
            open_app,
            type_text,
            system_control,
            workflow,
            custom,
        }
    }

    /// Get the action implementation for a given action type
    pub fn get_action(&self, action_type: &ActionType) -> Arc<dyn Action> {
        match action_type {
            ActionType::OpenApp => self.open_app.clone(),
            ActionType::TypeText => self.type_text.clone(),
            ActionType::SystemControl => self.system_control.clone(),
            ActionType::Workflow => self.workflow.clone(),
            ActionType::Custom => self.custom.clone(),
        }
    }

    /// Execute a command asynchronously
    pub async fn execute(&self, command: &CommandDefinition) -> Result<ActionResult, ActionError> {
        let action = self.get_action(&command.action_type);
        action.execute(&command.parameters).await
    }
}

/// State for the executor
pub struct ExecutorState {
    pub dispatcher: Arc<ActionDispatcher>,
}

impl Default for ExecutorState {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutorState {
    pub fn new() -> Self {
        Self {
            dispatcher: Arc::new(ActionDispatcher::new()),
        }
    }
}

/// Test a command by ID - executes immediately and returns result
#[tauri::command]
pub async fn test_command(
    app_handle: AppHandle,
    state: tauri::State<'_, crate::voice_commands::VoiceCommandsState>,
    executor_state: tauri::State<'_, ExecutorState>,
    id: String,
) -> Result<ActionResult, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| format!("Invalid UUID: {}", e))?;

    let command = {
        let registry = state
            .registry
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        registry.get(uuid).cloned().ok_or_else(|| format!("Command not found: {}", id))?
    };

    let result = executor_state.dispatcher.execute(&command).await;

    match &result {
        Ok(action_result) => {
            let payload = CommandExecutedPayload {
                command_id: command.id.to_string(),
                trigger: command.trigger.clone(),
                message: action_result.message.clone(),
            };
            let _ = app_handle.emit(command_events::COMMAND_EXECUTED, payload);
        }
        Err(action_error) => {
            let payload = CommandFailedPayload {
                command_id: command.id.to_string(),
                trigger: command.trigger.clone(),
                error_code: action_error.code.to_string(),
                error_message: action_error.message.clone(),
            };
            let _ = app_handle.emit(command_events::COMMAND_FAILED, payload);
        }
    }

    result.map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "executor_test.rs"]
mod tests;
