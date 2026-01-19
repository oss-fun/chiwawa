//! WASI Preview 1 implementation using passthrough to host wasi-libc.
//!
//! Chiwawa implements WASI by delegating system calls to the host's wasi-libc
//! implementation rather than implementing them directly. This passthrough
//! approach avoids duplicating WASI implementation logic and ensures
//! compatibility with any WASI-compliant host runtime.
//!
//! ## Architecture
//!
//! ```text
//! Guest Wasm (fd_write)
//!       |
//!       v
//! Chiwawa WASI (passthrough.rs)
//!       |
//!       v
//! Host wasi-libc
//!       |
//!       v
//! Host OS (actual write)
//! ```
//!
//! ## Module Organization
//!
//! - [`passthrough`]: WASI function implementations delegating to wasi-libc
//! - [`types`]: WASI type definitions
//! - [`error`]: WASI error codes and handling

pub mod error;
pub mod passthrough;
pub mod types;

pub use error::*;
pub use types::*;
