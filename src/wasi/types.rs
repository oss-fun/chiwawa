/// WASI file descriptor type
pub type Fd = i32;

/// WASI size type
pub type Size = u32;

/// WASI pointer type
pub type Ptr = u32;

/// WASI iovec structure for scatter-gather I/O
#[repr(C)]
#[derive(Debug, Clone)]
pub struct IoVec {
    pub buf: Ptr,
    pub buf_len: Size,
}

/// WASI file size type
pub type FileSize = u64;

/// WASI timestamp type
pub type Timestamp = u64;

/// WASI exit code type
pub type ExitCode = i32; 