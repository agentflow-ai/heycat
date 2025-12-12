---
status: pending
created: 2025-12-12
completed: null
dependencies: []
---

# Spec: Download and Store Whisper Model

## Description

Implement the ability to download and store the Whisper Large v3 Turbo model from HuggingFace. Provides backend commands for downloading and checking model status, plus frontend UI for triggering the download.

## Acceptance Criteria

- [ ] Backend command `download_model` downloads Large v3 Turbo from HuggingFace
- [ ] Model stored in `{app_data_dir}/heycat/models/ggml-large-v3-turbo.bin`
- [ ] Backend command `check_model_status` returns model availability (boolean)
- [ ] Download uses reqwest with streaming for large file support
- [ ] Frontend `useModelStatus` hook tracks model availability
- [ ] `ModelDownloadButton` component shows "Download Model" / "Downloading..." / "Model Ready" states
- [ ] Event `model_download_completed` emitted when download finishes

## Test Cases

- [ ] check_model_status returns false when model doesn't exist
- [ ] check_model_status returns true when model file exists
- [ ] download_model creates models directory if not exists
- [ ] download_model fetches from correct HuggingFace URL
- [ ] Frontend hook correctly reflects backend model status
- [ ] Button transitions through states during download

## Dependencies

None

## Preconditions

- Network access available for download
- Sufficient disk space (~1.5GB)

## Implementation Notes

- Model URL: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin`
- Use `dirs::data_dir()` for cross-platform app data location
- Consider partial download resume capability (out of MVP scope per feature.md)
- No download progress UI per MVP scope - just "downloading..." state

## Related Specs

- recording-block-without-model.spec.md (depends on check_model_status)
- transcription-pipeline.spec.md (loads the downloaded model)

## Integration Points

- Production call site: `src-tauri/src/lib.rs` (command registration)
- Connects to: Frontend ModelDownloadButton, TranscriptionManager

## Integration Test

- Test location: `src-tauri/src/model/download_test.rs`
- Verification: [ ] Integration test passes
