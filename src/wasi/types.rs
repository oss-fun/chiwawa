//! WASI type definitions.
//!
//! This module defines type aliases and structures that match the WASI
//! Preview 1 specification, used for interfacing with WASI functions.

/// WASI file descriptor type.
pub type Fd = i32;

/// WASI size type (32-bit unsigned).
pub type Size = u32;

/// WASI pointer type (32-bit address in linear memory).
pub type Ptr = u32;

/// WASI exit code type.
pub type ExitCode = i32;
