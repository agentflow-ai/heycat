---
status: pending
created: 2025-12-13
completed: null
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
