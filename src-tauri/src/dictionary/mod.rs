// Dictionary expansion module - stores and manages dictionary entries for text expansion
//
// NOTE: This is a foundational internal module consumed by tauri-commands.spec.md.
// The #[allow(unused_imports)] will be removed when production wiring is added.

mod expander;
mod store;

#[allow(unused_imports)]
pub use expander::{DictionaryExpander, ExpansionResult};
#[allow(unused_imports)]
pub use store::{DictionaryEntry, DictionaryError, DictionaryStore};
