# Integration-Focused Review

You are reviewing a spec for a **Tauri v2 desktop application** with React frontend and Rust backend. One of the primary failure modes is code that exists but is not wired up end-to-end.

## Pre-Review Gates (Automated)

Run these checks FIRST. **STOP if any fails.**

### 1. Build Warning Check
```bash
cd src-tauri && cargo check 2>&1 | grep -E "(warning|unused|dead_code|never)"
```
**FAIL if new code has unused warnings.**

### 2. Command Registration Check (if spec adds commands)
```bash
# Commands defined but not registered
comm -23 \
  <(grep -rn "#\[tauri::command\]" src-tauri/src -A1 | grep "fn " | sed 's/.*fn \([a-z_]*\).*/\1/' | sort -u) \
  <(grep -A50 "invoke_handler" src-tauri/src/lib.rs | grep "commands::" | sed 's/.*::\([a-z_]*\).*/\1/' | sort -u)
```
**FAIL if output is not empty.**

### 3. Event Subscription Check (if spec adds events)
```bash
# Events defined in backend
grep "pub const.*: &str = " src-tauri/src/events.rs | grep -oP '"\K[^"]+'

# Events listened to in frontend
grep -rn "listen<" src/hooks --include="*.ts" | grep -oP '"[a-z_]+"'
```
**FAIL if new event has no listener.**

---

## Manual Review (6 Questions)

IMPORTANT: While going through the manual review, ensure to understand how to review the test implementation, it must adhere to the docs/TESTING.md instructions.

### 1. Is the code wired up end-to-end?

Verify the new code is actually connected to production execution paths:
- [ ] New functions are called from production code (not just tests)
- [ ] New structs are instantiated in production code (not just tests)
- [ ] New events are both emitted AND listened to
- [ ] New commands are registered in invoke_handler AND called from frontend

**NEEDS_WORK if any new code is orphaned (exists but not connected).**

### 2. What would break if this code was deleted?

For each new function/struct/event introduced:

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| [name] | fn/struct/event | file:line | YES/NO/TEST-ONLY |

**TEST-ONLY = NEEDS_WORK** - Code only used in tests must be moved to test module.
**NO = NEEDS_WORK** - Code not reachable from production is dead code.

### 3. Where does the data flow?

For features with frontend-backend interaction, trace the complete path:

```
[UI Action]
     |
     v
[Hook] src/hooks/useX.ts:NN
     | invoke("command_name")
     v
[Command] src-tauri/src/commands/mod.rs:NN
     |
     v
[Logic] src-tauri/src/module/file.rs:NN
     |
     v
[Event] emit!("event_name") at file:NN
     |
     v
[Listener] src/hooks/useX.ts:NN listen()
     |
     v
[State Update] useState/store at file:NN
     |
     v
[UI Re-render]
```

**NEEDS_WORK if any link is missing or broken.**

### 4. Are there any deferrals?

Search implementation for deferred work:
```bash
grep -rn "TODO\|FIXME\|XXX\|HACK\|handled separately\|will be implemented\|for now" src-tauri/src src/
```

| Deferral Text | Location | Tracking Spec |
|---------------|----------|---------------|
| [quote] | file:line | spec.md or **MISSING** |

**MISSING = NEEDS_WORK** - Every deferral must reference a tracking spec.

### 5. Automated check results

Paste the output from Pre-Review Gates above:
```
[paste output]
```

### 6. Frontend-Only Integration Check (for UI specs without backend changes)

When the spec creates hooks or components but no backend commands/events:

#### App Entry Point Verification
```bash
# Find where the component's parent is rendered
grep -rn "AppShell\|<App" src/ --include="*.tsx" | head -5
```

Check the app entry point (usually `src/App.tsx`):
- [ ] New hooks are called here (not just in intermediate components)
- [ ] New state is passed to child components (not hardcoded values)
- [ ] Dynamic data flows from hooks → props → component

#### Hardcoded Value Check
```bash
# Look for hardcoded status/state props where dynamic should be
grep -rn 'status="idle"\|status="ready"\|isRecording={false}' src/App.tsx src/components/
```
**NEEDS_WORK if state that should come from a hook is hardcoded.**

#### Hook Usage Check
For each new hook created:
| Hook | Created In | Called In | Passes Data To |
|------|------------|-----------|----------------|
| [name] | hooks/useX.ts | App.tsx:NN | Component.prop |

**NEEDS_WORK if "Called In" is only test files or component files (not app entry point).**

---

## Verdict

### APPROVED
All of the following must be true:
- [ ] All automated checks pass (no warnings, all registrations verified)
- [ ] All new code is reachable from production (not test-only)
- [ ] Data flow is complete with no broken links (backend-frontend AND frontend-only)
- [ ] All deferrals reference tracking specs

### NEEDS_WORK
Provide:
1. **What failed** - specific question number and item
2. **Why it failed** - missing registration, broken link, no evidence, etc.
3. **How to fix** - concrete action with target file:line
