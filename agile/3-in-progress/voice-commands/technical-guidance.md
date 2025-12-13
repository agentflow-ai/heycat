# Technical Guidance: Voice Commands

## Overview

This feature extends `ai-transcription` to parse transcribed text as commands and execute system actions. When the user speaks a phrase that matches a registered command, the associated action executes. Unmatched text falls through to clipboard copy (existing behavior).

## Architecture Overview

### Integration Point

Voice commands intercept the transcription pipeline at `src-tauri/src/hotkey/integration.rs:249-264` where transcribed text is currently copied to clipboard. The new flow:

```
Transcription Complete (text)
    → Command Matcher (check against registry)
    → Match Found?
        → Yes: Execute Action → Emit command_executed event
        → No: Copy to Clipboard (existing behavior) → Emit transcription_completed
```

### Module Structure

```
src-tauri/src/
├── voice_commands/
│   ├── mod.rs              # Module exports, state management
│   ├── registry.rs         # Command definitions, persistence
│   ├── matcher.rs          # Fuzzy matching logic
│   ├── executor.rs         # Action execution dispatcher
│   └── actions/
│       ├── mod.rs          # Action trait, dispatcher
│       ├── app_launcher.rs # Open/close applications
│       ├── text_input.rs   # Type text, paste
│       ├── system.rs       # Volume, brightness, etc.
│       └── workflow.rs     # Multi-step sequences
└── events.rs               # Add: command_matched, command_executed, command_failed
```

### Data Flow

1. **Command Registry** (persisted in app config dir as JSON):
   ```rust
   struct CommandDefinition {
       id: Uuid,
       trigger: String,           // e.g., "open slack"
       action: ActionType,        // enum: OpenApp, TypeText, Workflow, etc.
       parameters: HashMap<String, String>,
       enabled: bool,
   }
   ```

2. **Matching** occurs after transcription:
   - Normalize input (lowercase, trim)
   - Check exact matches first
   - Fall back to fuzzy matching (Levenshtein distance) with configurable threshold
   - Handle parameterized commands: "type {text}" extracts the text portion

3. **Action Execution**:
   - Each action type implements an `Action` trait
   - Actions run in a spawned thread to avoid blocking
   - Results emitted via events

### Key Architectural Decisions

| Decision | Rationale |
|----------|-----------|
| **Backend-driven matching** | Transcription already happens in Rust; keeps latency low |
| **JSON config persistence** | Simple, human-readable, editable outside app |
| **Trait-based actions** | Extensible; easy to add new action types |
| **Fuzzy matching with threshold** | Tolerates transcription errors; threshold prevents false positives |
| **Sequential workflow execution** | Predictable ordering; simpler error handling |

### Integration with Existing Code

**Modify `hotkey/integration.rs:spawn_transcription()`:**
- After successful transcription, call `voice_commands::try_match(&text)`
- If match found, call `voice_commands::execute(command)`
- Else, proceed with clipboard copy (existing behavior)

**New Tauri Commands:**
- `get_commands()` - List all registered commands
- `add_command(definition)` - Add/update command
- `remove_command(id)` - Delete command
- `test_command(id)` - Execute command directly (for UI testing)

**New Events:**
- `command_matched` - Payload: { command_id, trigger, action_type }
- `command_executed` - Payload: { command_id, success, result }
- `command_failed` - Payload: { command_id, error }

### macOS System Integration

**Permissions Required:**
- Accessibility (for keyboard simulation, app control)
- Automation (per-app for AppleScript targets)

**Implementation Patterns (from VoiceInk reference):**

1. **App Launching:**
   ```rust
   // Use open command or NSWorkspace equivalent
   std::process::Command::new("open")
       .arg("-a")
       .arg("Slack")
       .spawn()
   ```

2. **Text Input (CGEvent):**
   ```rust
   // Via core-foundation and core-graphics crates
   // Requires Accessibility permission
   CGEvent::new_keyboard_event(source, keycode, keydown)
   ```

3. **AppleScript Execution:**
   ```rust
   std::process::Command::new("osascript")
       .arg("-e")
       .arg("tell application \"Chrome\" to open location \"https://...\"")
       .output()
   ```

4. **Clipboard + Paste:**
   ```rust
   // Already using arboard crate
   clipboard.set_text(&text)?;
   // Simulate Cmd+V via CGEvent
   ```

### Constraints

- **No LLM/AI reasoning** - Pattern matching only per scope definition
- **macOS only** - Windows/Linux implementations deferred
- **Blocking execution** - Multiple commands execute sequentially
- **Pre-configured commands** - No runtime learning or adaptation

## Dependencies

- `ai-transcription` feature must be completed ✅
- Rust crates: `strsim` (fuzzy matching), `core-foundation`, `core-graphics` (macOS APIs)

## Reference Implementation

VoiceInk project at `/Users/michaelhindley/Documents/git/VoiceInk`:
- `PowerMode/PowerModeConfig.swift` - Configuration structure pattern
- `PowerMode/ActiveWindowService.swift` - App detection via NSWorkspace
- `PowerMode/BrowserURLService.swift` - AppleScript execution pattern
- `CursorPaster.swift` - CGEvent keyboard simulation pattern

## Open Questions

*(To be resolved during implementation)*

1. Configuration UI scope - settings panel MVP or defer?
2. Workflow definition format - JSON structure for multi-step sequences
3. Error feedback - notifications vs inline vs sound?
