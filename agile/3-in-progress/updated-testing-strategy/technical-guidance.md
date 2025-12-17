---
last-updated: 2025-12-17
status: active
---

# Technical Guidance: Updated Testing Strategy

## Architecture Overview

This feature refactors the test suite to focus on behavior-based testing rather than implementation details. The goal is to reduce test maintenance burden while maintaining confidence through smoke testing of valuable paths.

### Current State Analysis

**Test Inventory:**
- Frontend: 323 tests across 27 files (Vitest + React Testing Library)
- Backend: 401 tests across 11 files (Rust test attribute)
- Coverage threshold: 60% (lines + functions)

**Identified Anti-Patterns:**

1. **Implementation-Detail Tests** (high volume, low value)
   - Testing React internals: stable function references, listener counts, cleanup calls
   - Testing Rust guarantees: mutex thread safety, trait implementations
   - Example: `test_returns_stable_function_references` - React/useCallback guarantees this

2. **Redundant Initialization Tests**
   - Multiple tests for obvious defaults: `test_new_manager_starts_idle`, `test_default_manager_starts_idle`, `test_default_state_is_idle`
   - Testing Display/Debug trait implementations

3. **Exhaustive State Machine Coverage**
   - 59 tests in `state_test.rs` testing every transition permutation
   - Each invalid transition tested individually vs. one "invalid operations don't corrupt state" test

4. **Serialization Format Tests**
   - Testing JSON contains specific strings rather than round-trip behavior

### Target Architecture

**Behavior-Focused Testing:**

```
┌─────────────────────────────────────────────────────────┐
│                    Test Pyramid                         │
├─────────────────────────────────────────────────────────┤
│  Few integration tests (user flows, cross-component)    │
│  ───────────────────────────────────────────────────    │
│  Moderate behavior tests (one test per behavior)        │
│  ───────────────────────────────────────────────────    │
│  NO implementation-detail tests                         │
└─────────────────────────────────────────────────────────┘
```

**Test Structure Per Module:**
- 3-5 behavior tests that cover:
  - Happy path flow (e.g., complete recording cycle)
  - Error handling (user sees errors correctly)
  - Edge cases that affect users (e.g., abort discards data)

**Layers Affected:**
- `src/hooks/*.test.ts` - Frontend hook tests
- `src/components/*.test.tsx` - Component tests
- `src-tauri/src/**/*_test.rs` - Backend unit tests

**Integration with Existing Systems:**
- Vitest config unchanged (60% coverage threshold)
- Cargo test config unchanged
- TCR workflow unchanged (`tcr check` still gates commits)

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Keep 60% coverage threshold | Smoke testing philosophy - cover valuable paths, not exhaustive | 2025-12-17 |
| Remove trait implementation tests | Display/Debug/Error traits are Rust-guaranteed; testing format is brittle | 2025-12-17 |
| Consolidate state transition tests | 59 tests → ~5-10 behavior flows; invalid transitions covered by error recovery test | 2025-12-17 |
| Remove React-internal tests | stable refs, listener counts, cleanup - React/framework guarantees these | 2025-12-17 |
| Document philosophy first | TESTING.md provides consistent criteria before refactoring | 2025-12-17 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-17 | Frontend has 323 tests across 27 files | Large reduction potential in hooks |
| 2025-12-17 | Backend has 401 tests across 11 files | state_test.rs is 59 tests alone |
| 2025-12-17 | useSettings has 16 tests, useRecording has 14 | Prime candidates for consolidation |
| 2025-12-17 | state_test.rs tests every transition permutation | Can consolidate to ~5 flow tests |
| 2025-12-17 | matcher_test.rs tests actual behavior well | Good pattern to follow |

## Open Questions

- [x] What coverage threshold to maintain? → 60% (existing, appropriate for smoke testing)
- [x] Where to document philosophy? → docs/TESTING.md
- [ ] Should component tests follow same consolidation? → Evaluate after hook tests done

## References

- `vitest.config.ts` - Coverage configuration (60% lines/functions)
- `CLAUDE.md` - TCR testing philosophy documentation
- `src/hooks/useRecording.test.ts` - Example of current hook test style
- `src-tauri/src/recording/state_test.rs` - Example of exhaustive state testing
- `src-tauri/src/voice_commands/matcher_test.rs` - Good behavior-focused test example
