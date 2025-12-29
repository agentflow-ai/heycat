//! SpacetimeDB integration module for heycat
//!
//! This module manages the SpacetimeDB sidecar process lifecycle:
//! - Spawning the standalone server on app startup
//! - Managing the WebSocket connection via the SDK client
//! - Graceful shutdown on app close
//! - Subscription handling for real-time updates
//!
//! Architecture:
//! ```text
//! Tauri App
//!    |
//!    +-- SpacetimeDB Sidecar (standalone server)
//!    |      |
//!    |      +-- WebSocket (localhost:3055)
//!    |             |
//!    +-------------+-- SpacetimeDB SDK Client (client.rs)
//!                        |
//!                        +-- Subscription Callbacks
//!                               |
//!                               +-- SubscriptionHandler (subscriptions.rs)
//!                                      |
//!                                      +-- Tauri Events → Event Bridge
//! ```
//!
//! ## Module Organization
//!
//! - `sidecar.rs` - Manages the SpacetimeDB standalone server process
//! - `subscriptions.rs` - Event emission traits and handlers
//! - `client.rs` - SDK client connection and subscription management
//!
//! ## Binding Generation
//!
//! The SDK client requires generated bindings. To generate them:
//! ```bash
//! mkdir -p src-tauri/src/spacetimedb/module_bindings
//! spacetime generate --lang rust \
//!     --out-dir src-tauri/src/spacetimedb/module_bindings \
//!     --project-path spacetimedb
//! ```

pub mod client;
mod sidecar;
mod subscriptions;

pub use client::{
    ClientError, ConnectionState, RecordingRecord, RecordingStoreError, SpacetimeClient,
    TranscriptionRecord, TranscriptionStoreError,
};
pub use sidecar::{SidecarHandle, SidecarManager};

// Subscription types used by the SDK client callbacks
#[allow(unused_imports)]
pub use subscriptions::{
    ConnectionStatusPayload, RecordingsUpdatedPayload, SubscriptionEventEmitter,
    SubscriptionHandler, TauriSubscriptionEmitter, TranscriptionsUpdatedPayload,
};
