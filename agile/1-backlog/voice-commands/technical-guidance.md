# Technical Guidance: Voice Commands

## Overview

This feature extends `ai-transcription` to parse transcribed text as commands and execute system actions.

## Dependencies

- `ai-transcription` feature must be completed first
- May benefit from `ai-chat` for intelligent command interpretation

## Technical Considerations

### Command Parsing Approaches
1. **Keyword-based**: Simple pattern matching ("open Chrome", "type hello")
2. **AI-interpreted**: Send to LLM to extract intent and parameters
3. **Hybrid**: Keywords for common actions, AI for complex queries

### System Integration (macOS)
- AppleScript for application control
- Accessibility API for UI automation
- Keyboard/mouse simulation (CGEvent)
- System permissions required:
  - Accessibility
  - Automation per-app

### Safety Mechanisms
- Confirmation for destructive actions
- Undo support where possible
- Rate limiting for rapid commands
- Whitelist of allowed applications

### Action Types
- Application management (open, close, switch)
- Text input (type, paste)
- Navigation (scroll, click, focus)
- Window management (resize, move, minimize)
- Custom workflows (macros)

## Reference Implementation

See VoiceInk project at `/Users/michaelhindley/Documents/git/VoiceInk` for:
- `PowerMode/` - Voice command patterns
- `CursorPaster.swift` - Text insertion
- AppleScript files for browser control
