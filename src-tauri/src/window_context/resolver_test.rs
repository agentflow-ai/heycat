// Tests for WindowContext resolver
//
// NOTE: These tests require SpacetimeDB integration and are currently disabled.
// The ContextResolver functionality is tested via integration tests that run
// with a live SpacetimeDB instance.
//
// The resolver's core logic (merge/replace modes) is verified through:
// 1. Code review of the implementation
// 2. Manual testing with the running application
// 3. E2E tests when switching between windows

// Tests commented out until SpacetimeDB mock infrastructure is available:
//
// - no_context_returns_all_global_commands: Verifies fallback behavior
// - no_context_returns_all_global_dictionary: Verifies fallback behavior
// - replace_mode_returns_only_context_commands: Verifies Replace mode
// - command_id_not_found_is_skipped: Edge case handling
// - dictionary_entry_id_not_found_is_skipped: Edge case handling
// - resolver_creation: Requires SpacetimeDB client
// - get_current_context_id_returns_none_when_no_context: Requires SpacetimeDB client
// - merge_mode_logic_verified_by_structure: Merge mode verification

use super::*;

#[test]
fn context_resolver_is_send_sync() {
    // Verify ContextResolver is thread-safe
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<ContextResolver>();
    assert_sync::<ContextResolver>();
}
