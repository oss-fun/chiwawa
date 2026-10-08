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
//! - [`socket`]: the socket extensions of WAMR and WasmEdge
//! - [`threads`]: wasi-threads `thread-spawn`, which interposes rather than
//!   forwarding to wasi-libc
//! - [`types`]: WASI type definitions
//! - [`error`]: WASI error codes and handling

pub mod error;
pub mod passthrough;
pub mod socket;
#[cfg(feature = "threads")]
pub mod threads;
pub mod types;

pub use error::*;
pub use types::*;

use crate::execution::value::Val;

/// The parameters of a WASI call, read by position as the WASI types.
pub struct Args<'a>(pub &'a [Val]);

impl Args<'_> {
    pub fn i32(&self, i: usize) -> WasiResult<i32> {
        self.0
            .get(i)
            .and_then(|v| v.to_i32().ok())
            .ok_or(WasiError::Inval)
    }

    pub fn u32(&self, i: usize) -> WasiResult<u32> {
        Ok(self.i32(i)? as u32)
    }

    pub fn i64(&self, i: usize) -> WasiResult<i64> {
        self.0
            .get(i)
            .and_then(|v| v.to_i64().ok())
            .ok_or(WasiError::Inval)
    }

    pub fn u64(&self, i: usize) -> WasiResult<u64> {
        Ok(self.i64(i)? as u64)
    }
}
