use super::error::*;
use std::collections::HashMap;
use std::io::{self, Read, Write};

/// WASI context for managing file descriptors and state
pub struct WasiContext {
    pub file_descriptors: HashMap<i32, Box<dyn FileDescriptor>>,
    pub next_fd: i32,
}

impl WasiContext {
    pub fn new() -> Self {
        let mut ctx = WasiContext {
            file_descriptors: HashMap::new(),
            next_fd: 3, // Start from 3, as 0, 1, 2 are reserved
        };

        // Initialize standard file descriptors
        ctx.file_descriptors
            .insert(0, Box::new(StdinWrapper::new()));
        ctx.file_descriptors
            .insert(1, Box::new(StdoutWrapper::new()));
        ctx.file_descriptors
            .insert(2, Box::new(StderrWrapper::new()));

        ctx
    }

    pub fn get_fd(&mut self, fd: i32) -> WasiResult<&mut Box<dyn FileDescriptor>> {
        self.file_descriptors.get_mut(&fd).ok_or(WasiError::BadF)
    }
}

/// Trait for file descriptor operations
pub trait FileDescriptor: Send + Sync {
    fn read(&mut self, buf: &mut [u8]) -> WasiResult<usize>;
    fn write(&mut self, buf: &[u8]) -> WasiResult<usize>;
    fn seek(&mut self, offset: i64, whence: i32) -> WasiResult<i64>;
    fn close(&mut self) -> WasiResult<()>;
}

/// Wrapper for stdin
pub struct StdinWrapper {
    stdin: io::Stdin,
}

impl StdinWrapper {
    pub fn new() -> Self {
        Self { stdin: io::stdin() }
    }
}

impl FileDescriptor for StdinWrapper {
    fn read(&mut self, buf: &mut [u8]) -> WasiResult<usize> {
        self.stdin.read(buf).map_err(|_| WasiError::Io)
    }

    fn write(&mut self, _buf: &[u8]) -> WasiResult<usize> {
        Err(WasiError::BadF)
    }

    fn seek(&mut self, _offset: i64, _whence: i32) -> WasiResult<i64> {
        Err(WasiError::SPipe)
    }

    fn close(&mut self) -> WasiResult<()> {
        Ok(())
    }
}

/// Wrapper for stdout
pub struct StdoutWrapper {
    stdout: io::Stdout,
}

impl StdoutWrapper {
    pub fn new() -> Self {
        Self {
            stdout: io::stdout(),
        }
    }
}

impl FileDescriptor for StdoutWrapper {
    fn read(&mut self, _buf: &mut [u8]) -> WasiResult<usize> {
        Err(WasiError::BadF)
    }

    fn write(&mut self, buf: &[u8]) -> WasiResult<usize> {
        self.stdout.write(buf).map_err(|_| WasiError::Io)
    }

    fn seek(&mut self, _offset: i64, _whence: i32) -> WasiResult<i64> {
        Err(WasiError::SPipe)
    }

    fn close(&mut self) -> WasiResult<()> {
        Ok(())
    }
}

/// Wrapper for stderr
pub struct StderrWrapper {
    stderr: io::Stderr,
}

impl StderrWrapper {
    pub fn new() -> Self {
        Self {
            stderr: io::stderr(),
        }
    }
}

impl FileDescriptor for StderrWrapper {
    fn read(&mut self, _buf: &mut [u8]) -> WasiResult<usize> {
        Err(WasiError::BadF)
    }

    fn write(&mut self, buf: &[u8]) -> WasiResult<usize> {
        self.stderr.write(buf).map_err(|_| WasiError::Io)
    }

    fn seek(&mut self, _offset: i64, _whence: i32) -> WasiResult<i64> {
        Err(WasiError::SPipe)
    }

    fn close(&mut self) -> WasiResult<()> {
        Ok(())
    }
}
