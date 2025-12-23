// SharedDenoiser for thread-safe DTLN denoiser sharing
// Eliminates 2s model loading delay by loading once at app startup

use std::sync::{Arc, Mutex};

use super::{load_embedded_models, DenoiserError, DtlnDenoiser};

/// Shared DTLN denoiser wrapper for thread-safe sharing across recordings
///
/// This struct provides thread-safe access to a single DTLN denoiser instance
/// that can be shared between all recording sessions. Previously, each recording
/// loaded the ONNX models (~2s delay), making quick recordings unusable.
///
/// ## Usage
///
/// ```ignore
/// // Load once at app startup
/// let shared_denoiser = SharedDenoiser::try_load()?;
///
/// // For each recording:
/// shared_denoiser.reset();  // Clear LSTM states
/// // Pass to audio capture...
/// ```
///
/// ## LSTM State Reset
///
/// The DTLN model maintains LSTM hidden states between frames for temporal
/// continuity. These states MUST be reset at the start of each recording
/// via `reset()` to prevent audio artifacts from previous sessions.
#[derive(Clone)]
pub struct SharedDenoiser {
    /// The DTLN denoiser wrapped in thread-safe primitives
    inner: Arc<Mutex<DtlnDenoiser>>,
}

impl SharedDenoiser {
    /// Try to load the shared denoiser from embedded ONNX models
    ///
    /// This loads and optimizes the DTLN models, which takes ~2 seconds.
    /// Should be called once at app startup.
    ///
    /// # Returns
    /// * `Ok(SharedDenoiser)` - Successfully loaded denoiser ready for use
    /// * `Err(DenoiserError)` - If model loading or optimization fails
    pub fn try_load() -> Result<Self, DenoiserError> {
        crate::info!("Loading shared DTLN denoiser models...");
        let models = load_embedded_models()?;
        let denoiser = DtlnDenoiser::new(models);
        crate::info!("Shared DTLN denoiser loaded successfully");

        Ok(Self {
            inner: Arc::new(Mutex::new(denoiser)),
        })
    }

    /// Reset the denoiser state for a new recording
    ///
    /// Clears LSTM hidden states and buffers to prevent audio artifacts
    /// from previous recordings bleeding into the new one.
    ///
    /// **MUST be called at the start of each new recording.**
    pub fn reset(&self) {
        if let Ok(mut denoiser) = self.inner.lock() {
            denoiser.reset();
        } else {
            crate::warn!("Failed to lock denoiser for reset - lock poisoned");
        }
    }

    /// Get a clone of the inner Arc<Mutex<DtlnDenoiser>>
    ///
    /// This is used to pass the denoiser to the audio callback, which needs
    /// its own reference to process samples.
    pub fn inner(&self) -> Arc<Mutex<DtlnDenoiser>> {
        self.inner.clone()
    }
}

impl std::fmt::Debug for SharedDenoiser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedDenoiser")
            .field("loaded", &true)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::LazyLock;

    /// Cached SharedDenoiser for tests - avoids repeated model loads
    static CACHED_SHARED_DENOISER: LazyLock<SharedDenoiser> = LazyLock::new(|| {
        SharedDenoiser::try_load().expect("Models should load for tests")
    });

    // ==================== Behavior Tests ====================

    #[test]
    fn test_try_load_succeeds_with_embedded_models() {
        // This test verifies that the embedded models can be loaded
        // It's a critical behavior test since the app won't function without models
        // Note: This test intentionally calls try_load() directly to verify the load path
        let result = SharedDenoiser::try_load();
        assert!(result.is_ok(), "Failed to load embedded models: {:?}", result.err());
    }

    #[test]
    fn test_reset_does_not_panic() {
        // Reset should always succeed, even if called multiple times
        let denoiser = CACHED_SHARED_DENOISER.clone();
        denoiser.reset();
        denoiser.reset(); // Multiple resets should be safe
    }

    #[test]
    fn test_inner_returns_same_instance() {
        // Verify that inner() returns clones pointing to the same denoiser
        let denoiser = CACHED_SHARED_DENOISER.clone();
        let inner1 = denoiser.inner();
        let inner2 = denoiser.inner();

        // Both should point to the same underlying data
        assert!(Arc::ptr_eq(&inner1, &inner2));
    }

    #[test]
    fn test_clone_shares_same_denoiser() {
        // Verify that cloning SharedDenoiser shares the same inner instance
        let denoiser1 = CACHED_SHARED_DENOISER.clone();
        let denoiser2 = denoiser1.clone();

        // Both clones should share the same inner Arc
        assert!(Arc::ptr_eq(&denoiser1.inner, &denoiser2.inner));
    }

    #[test]
    fn test_shared_denoiser_is_send_sync() {
        // SharedDenoiser must be Send + Sync for cross-thread sharing
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SharedDenoiser>();
    }
}
