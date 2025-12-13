---
status: completed
created: 2025-12-13
completed: 2025-12-13
dependencies:
  - action-executor
  - app-launcher-action
  - text-input-action
---

# Spec: Workflow Action

## Description

Execute multi-step command sequences. Workflows combine multiple actions (open app, type text, etc.) that execute sequentially with each step waiting for completion.

## Acceptance Criteria

- [ ] Implement `Action` trait for `WorkflowAction`
- [ ] Accept list of action steps in workflow definition
- [ ] Execute steps sequentially (blocking order)
- [ ] Wait for each step completion before next
- [ ] Aggregate results from all steps
- [ ] Stop on first error, report which step failed
- [ ] Support delays between steps (configurable)

## Test Cases

- [ ] Workflow with 2 steps executes in order
- [ ] Step 1 failure stops workflow, reports step 1 failed
- [ ] All steps success returns aggregated success result
- [ ] Delay between steps respected
- [ ] Empty workflow returns success (no-op)
- [ ] Workflow with 5 steps executes all sequentially

## Dependencies

- action-executor (dispatch individual actions)
- app-launcher-action (OpenApp step type)
- text-input-action (TypeText step type)

## Preconditions

- All referenced action types implemented

## Implementation Notes

- Location: `src-tauri/src/voice_commands/actions/workflow.rs`
- Workflow definition in CommandDefinition.parameters as JSON array
- Use executor to dispatch each step action
- Consider timeout per step to prevent hanging

## Related Specs

- action-executor.spec.md (step dispatch)
- app-launcher-action.spec.md (step type)
- text-input-action.spec.md (step type)

## Integration Points

- Production call site: `src-tauri/src/voice_commands/executor.rs` (dispatch)
- Connects to: action-executor, all action implementations

## Integration Test

- Test location: `src-tauri/src/voice_commands/actions/workflow_test.rs`
- Verification: [ ] Integration test passes

## Review

- Date: 2025-12-13

### Acceptance Criteria

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Implement `Action` trait for `WorkflowAction` | :white_check_mark: | workflow.rs:60-140 - `#[async_trait] impl Action for WorkflowAction` |
| Accept list of action steps in workflow definition | :white_check_mark: | workflow.rs:64-73 - parses `steps` parameter as JSON into `Vec<WorkflowStep>` |
| Execute steps sequentially (blocking order) | :white_check_mark: | workflow.rs:95-129 - `for` loop iterates steps with `await` on each |
| Wait for each step completion before next | :white_check_mark: | workflow.rs:100 - `action.execute(&step.parameters).await` blocks until complete |
| Aggregate results from all steps | :white_check_mark: | workflow.rs:131-138 - returns `ActionResult` with `steps_executed` and `results` array |
| Stop on first error, report which step failed | :white_check_mark: | workflow.rs:110-121 - returns error with step index (1-based) and action type |
| Support delays between steps (configurable) | :white_check_mark: | workflow.rs:86-90 (global delay_ms), workflow.rs:124-128 (step-specific delay) |

### Test Cases

| Test Case | Status | Evidence |
|-----------|--------|----------|
| Workflow with 2 steps executes in order | :white_check_mark: | workflow_test.rs:74-101 `test_workflow_with_2_steps_executes_in_order` |
| Step 1 failure stops workflow, reports step 1 failed | :white_check_mark: | workflow_test.rs:103-135 `test_step_1_failure_stops_workflow` |
| All steps success returns aggregated success result | :white_check_mark: | workflow_test.rs:137-167 `test_all_steps_success_returns_aggregated_result` |
| Delay between steps respected | :white_check_mark: | workflow_test.rs:169-198 `test_delay_between_steps_respected` |
| Empty workflow returns success (no-op) | :white_check_mark: | workflow_test.rs:200-218 `test_empty_workflow_returns_success` |
| Workflow with 5 steps executes all sequentially | :white_check_mark: | workflow_test.rs:220-265 `test_workflow_with_5_steps_executes_all_sequentially` |

### Additional Notes

- Module properly exported in `mod.rs:5` (`pub mod workflow;`) and re-exported at `mod.rs:9` (`pub use workflow::WorkflowAction;`)
- Additional edge case tests provided: `test_missing_steps_parameter_returns_error` (workflow_test.rs:267-281) and `test_invalid_json_returns_parse_error` (workflow_test.rs:283-300)

### Verdict

**APPROVED**
