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

/// WASI iovec structure for scatter-gather I/O.
///
/// Matches the layout expected by WASI `fd_read` and `fd_write`.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct IoVec {
    /// Pointer to the buffer in linear memory.
    pub buf: Ptr,
    /// Length of the buffer in bytes.
    pub buf_len: Size,
}

/// WASI file size type (64-bit).
pub type FileSize = u64;

/// WASI timestamp type (nanoseconds since epoch).
pub type Timestamp = u64;

/// WASI exit code type.
pub type ExitCode = i32;
