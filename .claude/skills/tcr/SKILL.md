---
name: tcr
description: "TCR workflow automation enforcing test discipline. Auto-runs tests on todo completion, creates WIP commits on pass, tracks failures. Use for test-first development with 100% coverage enforcement."
---

# TCR (Test-Commit-Refactor) Skill

Enforces the Test-Commit-Refactor workflow through Claude Code hooks.

## How It Works

```
┌─────────────────────────────────────────────────────────────┐
│                     TCR Workflow Loop                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Write a failing test (start with red)                   │
│                    ↓                                         │
│  2. Write code to make the test pass                        │
│                    ↓                                         │
│  3. Mark todo as "completed"                                │
│                    ↓                                         │
│  ┌──────────────────────────────────────┐                   │
│  │  TCR Hook: PostToolUse on TodoWrite  │                   │
│  │  - Get changed files (git diff)      │                   │
│  │  - Find related tests                │                   │
│  │  - Run tests                         │                   │
│  └──────────────────────────────────────┘                   │
│                    ↓                                         │
│         ┌─────────┴─────────┐                               │
│         │                   │                               │
│    Tests Pass          Tests Fail                           │
│         │                   │                               │
│    Auto-commit         Increment                            │
│    WIP commit          failure count                        │
│         │                   │                               │
│         ↓                   ↓                               │
│    Next task         Agent BLOCKED                          │
│                      (must fix tests)                       │
│                                                              │
│  After 5 failures: Prompt to reconsider approach            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Pre-Commit Guardrail (Husky)

Pre-commit enforcement is handled by Husky (`.husky/pre-commit`):
- **Frontend**: Runs `bun run test:coverage` (Vitest with 100% thresholds)
- **Backend**: Runs `cargo +nightly llvm-cov --fail-under-lines 100 --fail-under-functions 100`
- Blocks commit if tests fail or coverage is insufficient

This is repository-level enforcement that applies to all contributors.

## Coverage Exclusions

Use inline comments to exclude untestable code:

**Frontend (TypeScript):**
```typescript
/* v8 ignore next */
await invoke("greet", { name }); // Tauri runtime required

/* v8 ignore start */
// Block of untestable code
/* v8 ignore stop */
```

**Backend (Rust):**
```rust
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn untestable_function() { ... }
```

## Commands

### Run Tests Manually

```bash
bun .claude/skills/tcr/tcr.ts run <files...>
```

Run tests for specific source files.

**Examples:**
```bash
bun .claude/skills/tcr/tcr.ts run src/App.tsx
bun .claude/skills/tcr/tcr.ts run src/utils/auth.ts src/utils/validation.ts
```

### Check Status

```bash
bun .claude/skills/tcr/tcr.ts status
bun .claude/skills/tcr/tcr.ts status --coverage  # Include live coverage metrics
```

Shows:
- Current step being worked on
- Failure count (visual bar)
- Last test result and timestamp
- Recent errors from `.tcr-errors.log` (if any)

Use `--coverage` or `-c` to also display current coverage metrics (runs tests).

### Reset Failure Counter

```bash
bun .claude/skills/tcr/tcr.ts reset
```

Use when you want to continue past the 5-failure threshold. Also clears the error log.

### Coverage Commands

```bash
bun .claude/skills/tcr/tcr.ts coverage          # Run both frontend and backend
bun .claude/skills/tcr/tcr.ts coverage frontend # Frontend only
bun .claude/skills/tcr/tcr.ts coverage backend  # Backend only
```

Run coverage checks and report metrics.

### Verify Configuration Sync

```bash
bun .claude/skills/tcr/tcr.ts verify-config
```

Checks that coverage thresholds are consistent across all three configuration files. Exits with code 1 if mismatches are found.

### Get Help

```bash
bun .claude/skills/tcr/tcr.ts help [command]
```

## Test Discovery

**Frontend:** Convention-based mapping

| Source File | Test File |
|-------------|-----------|
| `src/foo.ts` | `src/foo.test.ts` or `src/foo.spec.ts` |
| `src/bar.tsx` | `src/bar.test.tsx` or `src/bar.spec.tsx` |

**Backend:** Module-based filtering (tests are inline in source files)

| Source File | Test Filter |
|-------------|-------------|
| `src-tauri/src/lib.rs` | `tests::` (crate root) |
| `src-tauri/src/main.rs` | `tests::` (crate root) |
| `src-tauri/src/foo.rs` | `foo::tests::` |
| `src-tauri/src/bar/mod.rs` | `bar::tests::` |
| `src-tauri/src/bar/baz.rs` | `bar::baz::tests::` |

Backend tests must be in `#[cfg(test)] mod tests { }` blocks within each source file.

**Note:** If no test files are found for changed frontend files, the hook warns and exits without auto-committing. Write tests first, or commit manually.

## Test Runners

- **Frontend**: Vitest with v8 coverage (via `bun run test:coverage`)
- **Backend**: cargo-llvm-cov with nightly toolchain (via `cargo +nightly llvm-cov`)

The target is automatically detected based on which files changed.

## State Files

TCR stores state in two files at project root:

### `.tcr-state.json` - Main State

```json
{
  "currentStep": "add-user-authentication",
  "failureCount": 2,
  "lastTestResult": {
    "passed": false,
    "timestamp": "2025-11-25T10:15:00Z",
    "error": "Expected true, got false",
    "filesRun": ["src/auth.test.ts"],
    "target": "frontend"
  }
}
```

### `.tcr-errors.log` - Error Log

Persists hook errors that would otherwise only appear in console. Shown by `tcr status` and cleared by `tcr reset`.

**Add to `.gitignore`:**
```
.tcr-state.json
.tcr-errors.log
```

## Failure Threshold

After 5 consecutive failures on the same task, TCR prompts you to:

1. Break down the task into smaller pieces
2. Review the test expectations
3. Take a different approach
4. Run `tcr reset` to continue

## Hook Configuration

The skill requires a Claude Code hook in `.claude/settings.json`:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "TodoWrite",
        "hooks": [{
          "type": "command",
          "command": "bun .claude/skills/tcr/tcr.ts hook-todo-complete"
        }]
      }
    ]
  }
}
```

Pre-commit enforcement is handled separately by Husky (see above).

## Prerequisites

**Frontend:**
- Vitest configured with coverage (`vitest.config.ts`)
- Tests follow naming convention (`*.test.ts` or `*.spec.ts`)

**Backend:**
- Rust nightly toolchain: `rustup toolchain install nightly`
- cargo-llvm-cov: `cargo install cargo-llvm-cov`
- Tests in `#[cfg(test)] mod tests { }` blocks

## Tips

1. **Start with a failing test** - Write the test first, watch it fail
2. **Small steps** - Each todo should be a small, testable change
3. **Trust the loop** - Let TCR handle commits; focus on making tests pass
4. **Reset wisely** - If you hit 5 failures, consider if the approach needs rethinking

## Maintenance Notes

### Coverage Configuration Sync

Coverage thresholds are enforced in **three separate locations** that must stay in sync:

1. `.claude/skills/tcr/lib/coverage/config.ts` - TCR status display and reporting
2. `vitest.config.ts` - Frontend thresholds (coverage.thresholds)
3. `.husky/pre-commit` - Backend thresholds (--fail-under-lines/--fail-under-functions)

**If you change coverage thresholds, update all three files!**

### Exit Code Behavior

The TCR hook uses Claude Code's exit code system to enforce test discipline:

| Scenario | Exit Code | Effect |
|----------|-----------|--------|
| Tests pass | 0 | Agent continues, WIP commit created |
| Tests fail | 2 | Agent BLOCKED, must fix tests |
| No test files found | 0 | Agent continues, no auto-commit (warning shown) |
| Hook runtime error | 0 | Agent continues (fail-open), error logged |

**Test failures block the agent** - Claude receives the error via stderr and must respond to it before continuing. This enforces TCR discipline.

**Hook runtime errors are fail-open** - If the hook itself crashes (can't parse input, can't run tests), it exits with code 0 so bugs in the hook don't block all work. Errors are logged to `.tcr-errors.log` for inspection via `tcr status`.

### Known Limitations

- **Frontend test discovery**: Uses convention-based mapping (foo.ts → foo.test.ts). Non-standard test file locations won't be auto-discovered.

- **Backend test filtering**: Uses module-based filtering derived from file paths. Tests must be in `#[cfg(test)] mod tests { }` blocks within each source file.

- **Coverage configuration sync**: Thresholds are defined in three places. Use `tcr verify-config` to check they're in sync.
