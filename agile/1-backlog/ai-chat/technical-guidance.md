# Technical Guidance: AI Chat

## Overview

This feature extends `ai-transcription` to send transcribed text to an AI/LLM for processing and display the response.

## Dependencies

- `ai-transcription` feature must be completed first
- API integration with AI providers (OpenAI, Anthropic, etc.)

## Technical Considerations

### AI Provider Integration
- Consider using a provider-agnostic approach
- Store API keys securely (system keychain or encrypted config)
- Handle rate limits and API errors gracefully

### Response Handling
- Streaming vs buffered responses
- Display mechanism (notification, window, clipboard)
- Token limits and cost management

### Context Management
- Conversation history for multi-turn interactions
- System prompts for behavior customization
- Memory/context window management

## Reference Implementation

See VoiceInk project at `/Users/michaelhindley/Documents/git/VoiceInk` for AI integration patterns.
