use thiserror::Error;

pub type WasiResult<T> = Result<T, WasiError>;

#[derive(Error, Debug, Clone)]
pub enum WasiError {
    #[error("Success")]
    Success,

    #[error("I/O error")]
    IoError,

    #[error("Bad file descriptor")]
    BadFileDescriptor,

    #[error("Memory access error")]
    MemoryAccessError,

    #[error("Invalid argument")]
    InvalidArgument,

    #[error("Invalid seek")]
    InvalidSeek,

    #[error("Function not implemented")]
    NotImplemented,

    #[error("Process exit requested with code {0}")]
    ProcessExit(i32),

    #[error("Permission denied")]
    NotPermitted,

    #[error("No such file or directory")]
    NoSuchFileOrDirectory,

    #[error("File exists")]
    Exist,

    #[error("I/O error")]
    Io,

    #[error("Not a directory")]
    NotDirectory,

    #[error("No space left on device")]
    NoSpace,

    #[error("Permission denied")]
    PermissionDenied,
}

impl WasiError {
    /// Convert WASI error to errno value
    pub fn to_errno(&self) -> i32 {
        match self {
            WasiError::Success => 0,               // Success
            WasiError::ProcessExit(_) => 0,        // Success (process exit is not an error)
            WasiError::NotPermitted => 1,          // EPERM
            WasiError::NoSuchFileOrDirectory => 2, // ENOENT
            WasiError::IoError => 5,               // EIO
            WasiError::Io => 5,                    // EIO
            WasiError::BadFileDescriptor => 9,     // EBADF
            WasiError::MemoryAccessError => 14,    // EFAULT
            WasiError::Exist => 17,                // EEXIST
            WasiError::NotDirectory => 20,         // ENOTDIR
            WasiError::InvalidArgument => 22,      // EINVAL
            WasiError::NoSpace => 28,              // ENOSPC
            WasiError::InvalidSeek => 29,          // ESPIPE
            WasiError::NotImplemented => 38,       // ENOSYS
            WasiError::PermissionDenied => 1,      // EPERM
        }
    }
}
