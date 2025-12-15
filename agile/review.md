# Integration-Focused Review Instructions

You are reviewing a spec for a **Tauri v2 desktop application** with React frontend and Rust backend. The primary failure mode is code that exists but is not wired up end-to-end.

## Review Protocol

For each spec, complete ALL sections below. Use `file:line` evidence for every verification.

IMPORTANT: Apart from detailed instructions, be sure to reason about and find evidence for the entire spec actually being properly intergrated and works end to end in a real data flow, not just state transitions or other signals that may mislead you. Be sceptical and verify that the spec is actually working properly.

---

## 1. Acceptance Criteria Verification

Create a table mapping each acceptance criterion to evidence:

| Criterion | Status | Evidence |
|-----------|--------|----------|
| [criterion text] | PASS/FAIL | `file:line` - description |

**FAIL** any criterion without line-level code evidence.

---

## 2. Integration Path Trace

For specs involving frontend-backend interaction, trace and diagram the complete data flow.

### Required Path (verify each link exists):

```
[UI Action]
     |
     v
[Hook] ----invoke()----> [Command Handler]
                              |
                              v
                         [Business Logic]
                              |
                              v
                         [Event Emission]
                              |
     <----listen()------------+
     |
     v
[State Update]
     |
     v
[UI Re-render]
```

### Verification Table:

| Step | Expected | Actual Location | Status |
|------|----------|-----------------|--------|
| Hook calls invoke | `invoke("command_name")` | `src/hooks/useX.ts:NN` | PASS/FAIL |
| Command registered | Listed in invoke_handler | `src-tauri/src/lib.rs:193-208` | PASS/FAIL |
| Logic executed | Function called | `src-tauri/src/module/file.rs:NN` | PASS/FAIL |
| Event emitted | `emit!()` or `emit_or_warn!()` | `src-tauri/src/commands/mod.rs:NN` | PASS/FAIL |
| Event listened | `listen()` in hook | `src/hooks/useX.ts:NN` | PASS/FAIL |
| State updated | setState or store update | `src/hooks/useX.ts:NN` | PASS/FAIL |

**NEEDS_WORK** if any link is broken or missing.

---

## 3. Registration Audit

Verify all new code is properly registered in the application:

### Backend Registration Points:

| Component | Check | Location to Verify |
|-----------|-------|-------------------|
| Tauri commands | Listed in `invoke_handler![]` | `src-tauri/src/lib.rs:193-208` |
| Managed state | Passed to `app.manage()` | `src-tauri/src/lib.rs:56-99` |
| Builder wiring | Added to HotkeyIntegration builder | `src-tauri/src/lib.rs:118-139` |
| Event names | Defined in events module | `src-tauri/src/events.rs` |

### Frontend Registration Points:

| Component | Check | Location to Verify |
|-----------|-------|-------------------|
| Hooks | Imported and called | `src/App.tsx` or component files |
| Event listeners | Set up in useEffect | Hook files in `src/hooks/` |
| Components | Rendered in component tree | Parent component files |

**List each new item and its registration status:**

| Item | Type | Registered? | Evidence |
|------|------|-------------|----------|
| [name] | command/state/event/hook | YES/NO | `file:line` |

---

## 4. Mock-to-Production Audit

For every mock used in tests, verify production instantiation exists.

**Search pattern:** Look for `Mock*`, `Fake*`, `Test*` structs/classes in test files.

| Mock | Test Location | Production Counterpart | Production Instantiation |
|------|---------------|----------------------|-------------------------|
| MockEventEmitter | `*_test.rs` | TauriEventEmitter | `src-tauri/src/lib.rs:NN` |
| [add others] | | | |

**NEEDS_WORK** if any mock has no production counterpart instantiated.

---

## 5. Event Subscription Audit

For every event emitted by backend, verify frontend subscribes.

**Backend events to check** (search `emit!`, `emit_or_warn!`, `app_handle.emit`):

| Event Name | Emission Location | Frontend Listener | Listener Location |
|------------|-------------------|-------------------|-------------------|
| recording_started | `commands/mod.rs:NN` | YES/NO | `useRecording.ts:NN` |
| [add new events] | | | |

**NEEDS_WORK** if any event has no frontend listener.

---

## 6. Deferral Tracking

Search implementation for deferred work:

**Search terms:** `TODO`, `FIXME`, `XXX`, `HACK`, `handled separately`, `will be implemented`, `for now`, `temporary`

| Deferral Text | Location | Referenced Spec | Status |
|---------------|----------|-----------------|--------|
| "TODO: handle error case" | `file.rs:NN` | `error-handling.spec.md` | OK |
| "handled separately" | `file.rs:NN` | NONE | NEEDS_WORK |

**NEEDS_WORK** if any deferral lacks a spec reference in `agile/`.

---

## 7. Test Coverage Audit

Map spec test cases to actual tests:

| Test Case (from spec) | Test Location | Status |
|----------------------|---------------|--------|
| [test case description] | `file_test.rs:NN` or `file.test.ts:NN` | PASS/MISSING |

---

## 8. Build Warning Audit

Run builds and verify no new warnings related to the spec:

### Backend (Rust)
```bash
cd src-tauri && cargo build 2>&1 | grep -E "(warning|unused|dead_code)"
```

### Frontend (TypeScript)
```bash
bun run build 2>&1 | grep -E "(warning|unused|never used)"
```

**Check for these warning types:**

| Warning Type | Indicates |
|-------------|-----------|
| `unused import` | Code imported but never called |
| `never constructed` | Struct/enum defined but never instantiated |
| `never used` | Function/method implemented but never called |
| `dead_code` | Code that cannot be reached |

**New code introduced by this spec:**

| Item | Type | Used? | Evidence |
|------|------|-------|----------|
| [StructName] | struct | YES/NO | `instantiated at file:line` |
| [function_name] | function | YES/NO | `called at file:line` |

**NEEDS_WORK** if any new code generates unused warnings.

---

## 9. Code Quality Notes

Brief assessment of:
- [ ] Error handling appropriate
- [ ] No unwrap() on user-facing code paths
- [ ] Types are explicit (no untyped any/unknown)
- [ ] Consistent with existing patterns in codebase

---

## 10. Verdict

### APPROVED
All of the following must be true:
- All acceptance criteria pass with line-level evidence
- Integration path complete (no broken links in the trace)
- All registrations verified (invoke_handler, app.manage, builder, hooks)
- All mocks have production counterparts instantiated
- All emitted events have frontend listeners
- All deferrals reference tracked specs
- Test coverage matches spec test cases
- No unused code warnings for new code (build warning audit passed)

### NEEDS_WORK
If any above fails, provide:
1. **What failed** - specific section and item
2. **Why it failed** - missing registration, broken link, etc.
3. **How to fix** - concrete action with target file:line

---

## Review Checklist

Before submitting verdict:

- [ ] Read the spec file completely
- [ ] Read implementation notes and integration points in spec
- [ ] Traced integration path with diagram
- [ ] Verified all registrations in lib.rs
- [ ] Audited mocks vs production
- [ ] Audited event emission vs subscription
- [ ] Searched for deferrals
- [ ] Mapped test cases to actual tests
- [ ] Ran `cargo build`, no unused code warnings for new code
- [ ] Ran `bun run build`, no unused warnings for new code
- [ ] Provided line-level evidence for every claim
