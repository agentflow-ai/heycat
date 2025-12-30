use super::*;

#[test]
fn override_mode_defaults_to_merge() {
    assert_eq!(OverrideMode::default(), OverrideMode::Merge);
}
