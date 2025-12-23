---
status: pending
created: 2025-12-23
completed: null
dependencies:
  - context-resolver
---

# Spec: Wire Context Resolver into Recording Pipeline

## Description

Integrate the ContextResolver into the RecordingTranscriptionService so that command matching and dictionary expansion use context-aware commands/entries based on the active window.

**Data Flow Reference:** See `technical-guidance.md` → "DF-3: Context-Aware Command Execution Flow" (full diagram) and "DF-4: App Initialization Flow"

**THIS IS THE WIRING SPEC** - connects the feature to the production code path.

## Acceptance Criteria

- [ ] `RecordingTranscriptionService` accepts optional `ContextResolver`
- [ ] Builder method: `with_context_resolver(resolver: Arc<ContextResolver>)`
- [ ] `try_command_matching()` uses `resolver.get_effective_commands()` when resolver present
- [ ] Dictionary expansion uses `resolver.get_effective_dictionary()` when resolver present
- [ ] Falls back to global registry/store when resolver not set (backwards compatible)
- [ ] Falls back to global on any resolver error (graceful degradation)
- [ ] App initialization (`lib.rs`) creates and wires the resolver
- [ ] All entry points (UI, hotkey, wake word) benefit from context resolution

## Test Cases

- [ ] Without resolver: behavior unchanged (uses global)
- [ ] With resolver + no context: uses global commands
- [ ] With resolver + context: uses context-resolved commands
- [ ] Resolver error: falls back to global (no panic)
- [ ] Hotkey-triggered recording uses context resolution
- [ ] Wake-word-triggered recording uses context resolution
- [ ] UI-triggered recording uses context resolution

## Dependencies

- `context-resolver` - provides ContextResolver

## Preconditions

- ContextResolver fully implemented
- WindowMonitor running
- WindowContextStore loaded

## Implementation Notes

**Files to modify:**
- `src-tauri/src/transcription/service.rs` - add context_resolver field, modify matching
- `src-tauri/src/lib.rs` - wire resolver during app initialization

**Service modification:**
```rust
pub struct RecordingTranscriptionService<T, C> {
    // ... existing fields ...
    context_resolver: Option<Arc<ContextResolver>>,
}

impl<T, C> RecordingTranscriptionService<T, C> {
    pub fn with_context_resolver(mut self, resolver: Arc<ContextResolver>) -> Self {
        self.context_resolver = Some(resolver);
        self
    }

    async fn try_command_matching(&self, text: &str) -> bool {
        let commands = match &self.context_resolver {
            Some(resolver) => resolver.get_effective_commands(&self.registry),
            None => self.registry.all_commands(),
        };
        // ... matching logic using commands ...
    }
}
```

**Architecture reference:** See `docs/ARCHITECTURE.md` Section 3 (Multiple Entry Points Pattern) - all paths must converge.

## Related Specs

All previous specs feed into this integration point.

## Integration Points

- Production call site: `src-tauri/src/lib.rs:setup()` - creates and injects resolver
- Connects to: All recording entry points (UI, hotkey, wake word)

## Integration Test

- Test location: End-to-end test with window context active
- Verification: [ ] Integration test passes
- Manual test: Create context for an app, speak command, verify context-specific execution
