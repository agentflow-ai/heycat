---
status: pending
created: 2025-12-13
completed: null
dependencies:
  - command-registry
---

# Spec: Action Executor

## Description

Execute matched commands through a trait-based action system. Dispatches to specific action implementations based on ActionType and handles async execution with result propagation.

## Acceptance Criteria

- [ ] `Action` trait with async `execute` method returning `Result<ActionResult, ActionError>`
- [ ] `ActionType` enum: OpenApp, TypeText, SystemControl, Workflow, Custom
- [ ] Dispatcher routes ActionType to corresponding implementation
- [ ] Actions execute in spawned task (non-blocking)
- [ ] Emit events: `command_executed` (success) or `command_failed` (error)
- [ ] `test_command` Tauri command for direct execution from UI

## Test Cases

- [ ] Dispatch OpenApp action to app-launcher implementation
- [ ] Dispatch TypeText action to text-input implementation
- [ ] Unknown action type returns descriptive error
- [ ] Action execution emits success event with result
- [ ] Action failure emits error event with details
- [ ] Multiple actions can execute concurrently

## Dependencies

- command-registry (provides CommandDefinition with ActionType)

## Preconditions

- At least one action implementation available

## Implementation Notes

- Location: `src-tauri/src/voice_commands/executor.rs`
- Use `tokio::spawn` for non-blocking execution
- Event emission via Tauri's `AppHandle::emit`

## Related Specs

- command-registry.spec.md (command definitions)
- app-launcher-action.spec.md (OpenApp implementation)
- text-input-action.spec.md (TypeText implementation)
- workflow-action.spec.md (Workflow implementation)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs` (post-match)
- Connects to: all action implementations, event system

## Integration Test

- Test location: `src-tauri/src/voice_commands/executor_test.rs`
- Verification: [ ] Integration test passes
