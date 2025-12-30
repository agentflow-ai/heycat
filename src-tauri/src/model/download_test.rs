use super::*;

// Tests removed per docs/TESTING.md:
// - test_model_error_display: Display trait test
// - test_model_error_is_debug: Debug trait test
// - test_model_type_debug: Debug trait test
// - test_model_type_clone_and_eq: Type system guarantee
// - test_model_type_serde: Serialization derive
// - test_model_manifest_clone: Type system guarantee
// - test_model_manifest_debug: Debug trait test
// - test_model_file_clone: Type system guarantee
// - test_model_file_debug: Debug trait test

/// Get the path to models directory in the git repo (for tests)
fn get_test_models_dir(model_type: ModelType) -> PathBuf {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("Failed to get parent of manifest dir")
        .join("models")
        .join(model_type.dir_name())
}

// ==================== Path/Directory Behavior Tests ====================

#[test]
fn test_get_models_dir_contains_expected_path() {
    let result = get_models_dir();
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.ends_with("heycat/models") || path.ends_with("heycat\\models"));
}

#[test]
fn test_ensure_models_dir_creates_directory() {
    let result = ensure_models_dir();
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.exists());
}

#[test]
fn test_model_type_dir_name() {
    assert_eq!(ModelType::ParakeetTDT.dir_name(), "parakeet-tdt");
}

#[test]
fn test_model_type_display() {
    assert_eq!(format!("{}", ModelType::ParakeetTDT), "tdt");
}

#[test]
fn test_get_model_dir_tdt_returns_correct_path() {
    let result = get_model_dir(ModelType::ParakeetTDT);
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(
        path.ends_with("heycat/models/parakeet-tdt")
            || path.ends_with("heycat\\models\\parakeet-tdt")
    );
}

// ==================== Model Manifest Tests ====================

#[test]
fn test_model_manifest_tdt_returns_correct_file_list() {
    let manifest = ModelManifest::tdt();
    assert_eq!(manifest.model_type, ModelType::ParakeetTDT);
    assert_eq!(manifest.files.len(), 4);
    assert!(manifest.base_url.contains("huggingface.co"));
    assert!(manifest.base_url.contains("parakeet-tdt"));

    let file_names: Vec<&str> = manifest.files.iter().map(|f| f.name.as_str()).collect();
    assert!(file_names.contains(&"encoder-model.onnx"));
    assert!(file_names.contains(&"encoder-model.onnx.data"));
    assert!(file_names.contains(&"decoder_joint-model.onnx"));
    assert!(file_names.contains(&"vocab.txt"));
}

// ==================== Model File Existence Tests ====================

#[test]
fn test_check_model_files_exist_in_dir_returns_false_when_directory_missing() {
    let temp_dir =
        std::env::temp_dir().join(format!("heycat-test-{}", uuid::Uuid::new_v4()));
    let manifest = ModelManifest::tdt();
    assert!(!check_model_files_exist_in_dir(&temp_dir, &manifest));
}

#[test]
fn test_check_model_files_exist_in_dir_returns_false_when_files_missing() {
    let temp_dir =
        std::env::temp_dir().join(format!("heycat-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let manifest = ModelManifest::tdt();

    let result = check_model_files_exist_in_dir(&temp_dir, &manifest);

    let _ = std::fs::remove_dir_all(&temp_dir);
    assert!(!result);
}

#[test]
fn test_check_model_files_exist_in_dir_returns_true_with_repo_models() {
    let repo_model_dir = get_test_models_dir(ModelType::ParakeetTDT);
    let manifest = ModelManifest::tdt();

    assert!(
        check_model_files_exist_in_dir(&repo_model_dir, &manifest),
        "TDT model not found in repo. Run 'git lfs pull' to fetch models. Dir: {:?}",
        repo_model_dir
    );
}

#[test]
fn test_check_model_files_exist_in_dir_returns_true_with_stub_files() {
    use std::io::Write;

    let temp_dir =
        std::env::temp_dir().join(format!("heycat-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let manifest = ModelManifest::tdt();
    for file in &manifest.files {
        let file_path = temp_dir.join(&file.name);
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(b"stub").unwrap();
    }

    let result = check_model_files_exist_in_dir(&temp_dir, &manifest);

    let _ = std::fs::remove_dir_all(&temp_dir);
    assert!(result);
}
