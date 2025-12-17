---
last-updated: 2025-12-17
status: draft
---

# Technical Guidance: Tauri Code Review Improvements

## Architecture Overview

> **Required before moving to 3-in-progress**
> Document high-level design decisions that guide all specs in this feature.

This feature implements low-risk improvements identified from a comprehensive Tauri code review. All changes follow existing patterns in the codebase and do not modify runtime behavior.

**Layers/Components Involved:**
- Build configuration (`src-tauri/Cargo.toml`)
- Event system (`src-tauri/src/events.rs`, `src-tauri/src/commands/mod.rs`)
- Frontend type safety (`src/hooks/useRecording.ts`)

**Patterns Used:**
- Event name constants pattern (already established in `events.rs`)
- TypeScript generic type parameters for Tauri invoke calls (already used elsewhere)
- Cargo profile configuration (standard Rust pattern)

**Integration:**
- All changes are additive or follow existing patterns
- No new dependencies introduced
- No API changes

**Constraints:**
- Must not change any runtime behavior
- Must pass existing test suite
- Must maintain backwards compatibility

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Use `opt-level = "s"` | Optimize for size over speed for desktop app | 2025-12-17 |
| Add constant to `event_names` module | Consistent with existing pattern | 2025-12-17 |
| Defer setup function extraction | Low priority, would require more extensive changes | 2025-12-17 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-17 | No [profile.release] in Cargo.toml | Suboptimal release builds |
| 2025-12-17 | "audio-level" event uses string literal | Inconsistent with event_names pattern |
| 2025-12-17 | stop_recording invoke missing type param | Reduced TypeScript type safety |

## Open Questions

- [x] Should we use `opt-level = "s"` or `opt-level = 3`? -> "s" for size optimization
- [x] Should we also add `strip = true`? -> Defer to separate feature

## References

- [Tauri v2 Documentation](https://v2.tauri.app/)
- [Cargo Profile Reference](https://doc.rust-lang.org/cargo/reference/profiles.html)
- Existing event pattern: `src-tauri/src/events.rs:9-17`
