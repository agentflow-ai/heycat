---
status: pending
created: 2025-12-13
completed: null
dependencies: ["parakeet-module-skeleton.spec.md"]
---

# Spec: Multi-file ONNX model download

## Description

Extend the existing model download system to support multi-file ONNX models with manifest-based downloads. The current system downloads a single Whisper `.bin` file; Parakeet models require multiple ONNX files per model type (TDT vs EOU). This spec introduces a manifest structure, per-file progress events, atomic directory downloads, and model type selection.

## Acceptance Criteria

- [ ] `ModelManifest` struct created with model type, files list, and total size
- [ ] `ModelType` enum created: `ParakeetTDT`, `ParakeetEOU`
- [ ] `download_model_files()` function downloads all files in manifest
- [ ] Progress events emitted per-file: `model_file_download_progress { model_type, file_name, bytes_downloaded, total_bytes }`
- [ ] Download uses atomic temp directory + rename (follows existing pattern)
- [ ] `check_model_exists()` updated to accept `ModelType` parameter
- [ ] Model directories created at `{app_data}/heycat/models/parakeet-tdt/` and `parakeet-eou/`
- [ ] Failed download cleans up partial files/directory

## Test Cases

- [ ] Unit test: `ModelManifest::tdt()` returns correct file list (4 files)
- [ ] Unit test: `ModelManifest::eou()` returns correct file list (3 files)
- [ ] Unit test: `get_model_dir(ModelType::ParakeetTDT)` returns correct path
- [ ] Unit test: `check_model_exists(ModelType::ParakeetEOU)` returns false when directory missing
- [ ] Unit test: `check_model_exists(ModelType::ParakeetTDT)` returns true only when ALL files present
- [ ] Integration test: Download manifest validates file sizes match expected

## Dependencies

- `parakeet-module-skeleton.spec.md` - Parakeet types must exist

## Preconditions

- Network access to HuggingFace
- Existing `model/download.rs` with atomic download pattern

## Implementation Notes

### Model File Manifests

**TDT Model (parakeet-tdt-0.6b-v3-onnx)**
Base URL: `https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/`

| File | Size | Required |
|------|------|----------|
| `encoder-model.onnx` | 41.8 MB | Yes |
| `encoder-model.onnx.data` | 2.44 GB | Yes |
| `decoder_joint-model.onnx` | 72.5 MB | Yes |
| `vocab.txt` | 93.9 kB | Yes |

Total: ~2.56 GB

**EOU Model (parakeet_realtime_eou_120m-v1)**
Base URL: Needs ONNX export - use pre-converted files from parakeet-rs examples or export manually.

Expected files (from parakeet-rs documentation):
| File | Required |
|------|----------|
| `encoder.onnx` | Yes |
| `decoder_joint.onnx` | Yes |
| `tokenizer.json` | Yes |

Note: EOU ONNX files may need to be exported from the NeMo model. The spec should handle this by checking for pre-converted community models or documenting the export process.

### Key Types

```rust
/// Model type for multi-model support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    ParakeetTDT,
    ParakeetEOU,
}

/// Manifest for multi-file model downloads
#[derive(Debug, Clone)]
pub struct ModelManifest {
    pub model_type: ModelType,
    pub base_url: String,
    pub files: Vec<ModelFile>,
}

#[derive(Debug, Clone)]
pub struct ModelFile {
    pub name: String,
    pub size_bytes: u64,
}

impl ModelManifest {
    pub fn tdt() -> Self {
        Self {
            model_type: ModelType::ParakeetTDT,
            base_url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/".into(),
            files: vec![
                ModelFile { name: "encoder-model.onnx".into(), size_bytes: 43_826_176 },
                ModelFile { name: "encoder-model.onnx.data".into(), size_bytes: 2_620_162_048 },
                ModelFile { name: "decoder_joint-model.onnx".into(), size_bytes: 76_021_760 },
                ModelFile { name: "vocab.txt".into(), size_bytes: 96_154 },
            ],
        }
    }

    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.size_bytes).sum()
    }
}
```

### Download Flow

1. Create temp directory: `{models_dir}/.parakeet-tdt-{uuid}/`
2. For each file in manifest:
   - Download to temp directory using streaming (existing pattern)
   - Emit `model_file_download_progress` event
3. On success: Atomic rename temp dir to final dir
4. On failure: Delete temp directory and all contents

### Events

Add to `events.rs`:
```rust
pub mod model_events {
    // ... existing ...
    pub const MODEL_FILE_DOWNLOAD_PROGRESS: &str = "model_file_download_progress";

    #[derive(Debug, Clone, Serialize, PartialEq)]
    pub struct ModelFileDownloadProgressPayload {
        pub model_type: String,
        pub file_name: String,
        pub bytes_downloaded: u64,
        pub total_bytes: u64,
        pub file_index: usize,
        pub total_files: usize,
    }
}
```

### Files to Modify

- `src-tauri/src/model/download.rs` - Add manifest types, multi-file download
- `src-tauri/src/model/mod.rs` - Update Tauri commands
- `src-tauri/src/events.rs` - Add progress event types

## Related Specs

- `parakeet-module-skeleton.spec.md` - Prerequisite
- `tdt-batch-transcription.spec.md` - Will use downloaded TDT model
- `eou-streaming-transcription.spec.md` - Will use downloaded EOU model
- `frontend-model-settings.spec.md` - Frontend UI for download

## Integration Points

- Production call site: `src-tauri/src/model/mod.rs` - Tauri command `download_model`
- Connects to: `events.rs` (progress events), frontend download UI

## Integration Test

- Test location: Manual test via frontend (network-dependent)
- Verification: [ ] Integration test passes
