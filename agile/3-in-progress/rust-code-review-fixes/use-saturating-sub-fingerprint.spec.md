---
status: pending
created: 2025-12-21
completed: null
dependencies: []
---

# Spec: Use saturating_sub in AudioFingerprint

## Description

The `AudioFingerprint::overlap_ratio` method uses regular subtraction which could theoretically wrap on u64 underflow (though guarded by a condition). Using `saturating_sub` makes the intent clearer and is more defensive.

## Acceptance Criteria

- [ ] `overlap_ratio` uses `saturating_sub` for all subtractions
- [ ] Guard condition can be simplified or made more explicit
- [ ] Behavior remains identical (no functional change)
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

## Test Cases

- [ ] Existing wake word detector tests continue to pass
- [ ] Edge case: overlapping fingerprints produce correct ratio

## Dependencies

None

## Preconditions

None

## Implementation Notes

**File to modify:** `src-tauri/src/listening/detector.rs:96-116`

**Current code:**
```rust
fn overlap_ratio(&self, other: &AudioFingerprint) -> f32 {
    let overlap_start = self.start_idx.max(other.start_idx);
    let overlap_end = self.end_idx.min(other.end_idx);

    if overlap_start >= overlap_end {
        return 0.0; // No overlap
    }

    let overlap_len = (overlap_end - overlap_start) as f32;  // Could wrap if guard missed
    let self_len = (self.end_idx - self.start_idx) as f32;   // Could wrap if end < start

    if self_len == 0.0 {
        return 0.0;
    }

    overlap_len / self_len
}
```

**Suggested fix:**
```rust
fn overlap_ratio(&self, other: &AudioFingerprint) -> f32 {
    let overlap_start = self.start_idx.max(other.start_idx);
    let overlap_end = self.end_idx.min(other.end_idx);

    // Use saturating_sub to make underflow handling explicit
    let overlap_len = overlap_end.saturating_sub(overlap_start);
    if overlap_len == 0 {
        return 0.0; // No overlap
    }

    let self_len = self.end_idx.saturating_sub(self.start_idx);
    if self_len == 0 {
        return 0.0;
    }

    overlap_len as f32 / self_len as f32
}
```

## Related Specs

None

## Integration Points

- Production call site: `WakeWordDetector::analyze()` at `detector.rs:353-362`
- Connects to: Wake word duplicate detection

## Integration Test

- Test location: N/A (defensive code improvement, no functional change)
- Verification: [x] N/A
