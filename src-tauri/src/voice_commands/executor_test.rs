use super::*;
use crate::voice_commands::actions::{AppLauncherAction, TextInputAction};
use crate::voice_commands::registry::ActionType;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

/// Mock action that tracks execution count
struct MockAction {
    result: Result<ActionResult, ActionError>,
    execution_count: AtomicUsize,
}

impl MockAction {
    fn new_success(message: &str) -> Self {
        Self {
            result: Ok(ActionResult {
                message: message.to_string(),
                data: None,
            }),
            execution_count: AtomicUsize::new(0),
        }
    }

    fn new_failure(code: ActionErrorCode, message: &str) -> Self {
        Self {
            result: Err(ActionError {
                code,
                message: message.to_string(),
            }),
            execution_count: AtomicUsize::new(0),
        }
    }

    fn count(&self) -> usize {
        self.execution_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl Action for MockAction {
    async fn execute(&self, _parameters: &HashMap<String, String>) -> Result<ActionResult, ActionError> {
        self.execution_count.fetch_add(1, Ordering::SeqCst);
        self.result.clone()
    }
}

fn create_test_command(action_type: ActionType) -> CommandDefinition {
    CommandDefinition {
        id: Uuid::new_v4(),
        trigger: "test command".to_string(),
        action_type,
        parameters: HashMap::from([
            ("app".to_string(), "Slack".to_string()),
            ("text".to_string(), "Hello".to_string()),
            ("control".to_string(), "volume_up".to_string()),
            ("script".to_string(), "custom.sh".to_string()),
        ]),
        enabled: true,
    }
}

#[tokio::test]
async fn test_dispatch_open_app_action() {
    let mock = Arc::new(MockAction::new_success("App opened"));
    let dispatcher = ActionDispatcher::with_actions(
        mock.clone(),
        Arc::new(TextInputAction::new()),
        Arc::new(SystemControlAction),
        Arc::new(CustomAction),
    );

    let command = create_test_command(ActionType::OpenApp);
    let result = dispatcher.execute(&command).await;

    assert!(result.is_ok());
    assert_eq!(mock.count(), 1);
    assert_eq!(result.unwrap().message, "App opened");
}

#[tokio::test]
async fn test_dispatch_type_text_action() {
    let mock = Arc::new(MockAction::new_success("Text typed"));
    let dispatcher = ActionDispatcher::with_actions(
        Arc::new(AppLauncherAction::new()),
        mock.clone(),
        Arc::new(SystemControlAction),
        Arc::new(CustomAction),
    );

    let command = create_test_command(ActionType::TypeText);
    let result = dispatcher.execute(&command).await;

    assert!(result.is_ok());
    assert_eq!(mock.count(), 1);
    assert_eq!(result.unwrap().message, "Text typed");
}

#[tokio::test]
async fn test_action_failure_returns_error() {
    let mock = Arc::new(MockAction::new_failure(ActionErrorCode::ExecutionError, "Test failure"));
    let dispatcher = ActionDispatcher::with_actions(
        mock.clone(),
        Arc::new(TextInputAction::new()),
        Arc::new(SystemControlAction),
        Arc::new(CustomAction),
    );

    let command = create_test_command(ActionType::OpenApp);
    let result = dispatcher.execute(&command).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code, ActionErrorCode::ExecutionError);
    assert_eq!(error.message, "Test failure");
}

#[tokio::test]
async fn test_missing_parameter_returns_error() {
    let dispatcher = ActionDispatcher::new();

    let mut command = create_test_command(ActionType::OpenApp);
    command.parameters.clear(); // Remove all parameters

    let result = dispatcher.execute(&command).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code, ActionErrorCode::InvalidParameter);
    assert!(error.message.contains("app"));
}

#[tokio::test]
async fn test_multiple_actions_execute_concurrently() {
    let execution_order = Arc::new(TokioMutex::new(Vec::new()));

    // Create actions that record execution order
    struct OrderedAction {
        id: usize,
        order: Arc<TokioMutex<Vec<usize>>>,
    }

    #[async_trait]
    impl Action for OrderedAction {
        async fn execute(&self, _params: &HashMap<String, String>) -> Result<ActionResult, ActionError> {
            self.order.lock().await.push(self.id);
            Ok(ActionResult {
                message: format!("Action {}", self.id),
                data: None,
            })
        }
    }

    let action1 = Arc::new(OrderedAction {
        id: 1,
        order: execution_order.clone(),
    });
    let action2 = Arc::new(OrderedAction {
        id: 2,
        order: execution_order.clone(),
    });

    let dispatcher1 = ActionDispatcher::with_actions(
        action1,
        Arc::new(TextInputAction::new()),
        Arc::new(SystemControlAction),
        Arc::new(CustomAction),
    );
    let dispatcher2 = ActionDispatcher::with_actions(
        action2,
        Arc::new(TextInputAction::new()),
        Arc::new(SystemControlAction),
        Arc::new(CustomAction),
    );

    let cmd1 = create_test_command(ActionType::OpenApp);
    let cmd2 = create_test_command(ActionType::OpenApp);

    // Execute concurrently
    let (r1, r2) = tokio::join!(dispatcher1.execute(&cmd1), dispatcher2.execute(&cmd2));

    assert!(r1.is_ok());
    assert!(r2.is_ok());

    let order = execution_order.lock().await;
    assert_eq!(order.len(), 2);
    // Both actions executed (order may vary due to concurrency)
    assert!(order.contains(&1));
    assert!(order.contains(&2));
}

#[tokio::test]
async fn test_stub_action_types_dispatch_correctly() {
    let dispatcher = ActionDispatcher::new();

    // Test stub action types only (OpenApp and TypeText use real implementations with system dependencies)
    let test_cases = vec![
        (ActionType::SystemControl, "Would execute system control"),
        (ActionType::Custom, "Would execute custom script"),
    ];

    for (action_type, expected_prefix) in test_cases {
        let command = create_test_command(action_type.clone());
        let result = dispatcher.execute(&command).await;

        assert!(
            result.is_ok(),
            "Failed for action type {:?}",
            action_type
        );
        assert!(
            result.unwrap().message.starts_with(expected_prefix),
            "Unexpected message for action type {:?}",
            action_type
        );
    }
}

#[tokio::test]
async fn test_type_text_dispatches_to_text_input() {
    // TypeText uses real TextInputAction - test with mock for isolation
    let mock = Arc::new(MockAction::new_success("Text typed"));
    let dispatcher = ActionDispatcher::with_actions(
        Arc::new(AppLauncherAction::new()),
        mock.clone(),
        Arc::new(SystemControlAction),
        Arc::new(CustomAction),
    );

    let command = create_test_command(ActionType::TypeText);
    let result = dispatcher.execute(&command).await;

    assert!(result.is_ok());
    assert_eq!(mock.count(), 1);
}

#[tokio::test]
async fn test_open_app_dispatches_to_app_launcher() {
    // OpenApp uses real AppLauncherAction - test with mock for isolation
    let mock = Arc::new(MockAction::new_success("App launched"));
    let dispatcher = ActionDispatcher::with_actions(
        mock.clone(),
        Arc::new(TextInputAction::new()),
        Arc::new(SystemControlAction),
        Arc::new(CustomAction),
    );

    let command = create_test_command(ActionType::OpenApp);
    let result = dispatcher.execute(&command).await;

    assert!(result.is_ok());
    assert_eq!(mock.count(), 1);
}

#[test]
fn test_action_result_serialization() {
    let result = ActionResult {
        message: "Test message".to_string(),
        data: Some(serde_json::json!({"key": "value"})),
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("Test message"));
    assert!(json.contains("key"));
}

#[test]
fn test_action_error_serialization() {
    let error = ActionError {
        code: ActionErrorCode::ExecutionError,
        message: "Test error message".to_string(),
    };
    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains("EXECUTION_ERROR"));
    assert!(json.contains("Test error message"));
}

#[test]
fn test_action_error_display() {
    let error = ActionError {
        code: ActionErrorCode::ExecutionError,
        message: "Test error message".to_string(),
    };
    assert_eq!(error.to_string(), "EXECUTION_ERROR: Test error message");
}

#[test]
fn test_command_executed_payload_serialization() {
    let payload = CommandExecutedPayload {
        command_id: "test-id".to_string(),
        trigger: "open slack".to_string(),
        message: "App opened".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("test-id"));
    assert!(json.contains("open slack"));
    assert!(json.contains("App opened"));
}

#[test]
fn test_command_failed_payload_serialization() {
    let payload = CommandFailedPayload {
        command_id: "test-id".to_string(),
        trigger: "open slack".to_string(),
        error_code: "NOT_FOUND".to_string(),
        error_message: "Application not found".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("test-id"));
    assert!(json.contains("open slack"));
    assert!(json.contains("NOT_FOUND"));
}
