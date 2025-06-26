use super::context::*;
use super::*;
use crate::execution::mem::MemAddr;
use crate::structure::instructions::Memarg;
use getrandom::getrandom;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// Standard WASI implementation
pub struct StandardWasiImpl {
    context: Arc<Mutex<WasiContext>>,
}

impl StandardWasiImpl {
    pub fn new() -> Self {
        Self {
            context: Arc::new(Mutex::new(WasiContext::new())),
        }
    }

    pub fn fd_write(
        &self,
        memory: &MemAddr,
        fd: Fd,
        iovs_ptr: Ptr,
        iovs_len: Size,
        nwritten_ptr: Ptr,
    ) -> WasiResult<i32> {
        // Validate file descriptor (only support stdout=1 and stderr=2 for now)
        if fd != 1 && fd != 2 {
            return Err(super::error::WasiError::BadFileDescriptor);
        }

        let mut total_written = 0u32;

        // Read each iovec structure and write the data
        for i in 0..iovs_len {
            // Each iovec is 8 bytes: buf_ptr (4 bytes) + buf_len (4 bytes)
            let iovec_offset = iovs_ptr + (i * 8);

            // Read buf_ptr (first 4 bytes of iovec)
            let buf_ptr_memarg = Memarg {
                offset: 0,
                align: 4,
            };
            let buf_ptr: u32 = memory
                .load(&buf_ptr_memarg, iovec_offset as i32)
                .map_err(|_| super::error::WasiError::MemoryAccessError)?;

            // Read buf_len (next 4 bytes of iovec)
            let buf_len_memarg = Memarg {
                offset: 0,
                align: 4,
            };
            let buf_len: u32 = memory
                .load(&buf_len_memarg, (iovec_offset + 4) as i32)
                .map_err(|_| super::error::WasiError::MemoryAccessError)?;

            if buf_len == 0 {
                continue;
            }

            // Read data from memory buffer
            let mut data = Vec::with_capacity(buf_len as usize);
            let byte_memarg = Memarg {
                offset: 0,
                align: 1,
            };

            for j in 0..buf_len {
                let byte: u8 = memory
                    .load(&byte_memarg, (buf_ptr + j) as i32)
                    .map_err(|_| super::error::WasiError::MemoryAccessError)?;
                data.push(byte);
            }

            // Write to appropriate file descriptor
            let bytes_written = match fd {
                1 => {
                    // stdout
                    io::stdout()
                        .write(&data)
                        .map_err(|_| super::error::WasiError::IoError)?
                }
                2 => {
                    // stderr
                    io::stderr()
                        .write(&data)
                        .map_err(|_| super::error::WasiError::IoError)?
                }
                _ => unreachable!(),
            };

            total_written += bytes_written as u32;
        }

        // Flush output to ensure data is written
        match fd {
            1 => io::stdout()
                .flush()
                .map_err(|_| super::error::WasiError::IoError)?,
            2 => io::stderr()
                .flush()
                .map_err(|_| super::error::WasiError::IoError)?,
            _ => unreachable!(),
        }

        // Write the total number of bytes written to nwritten_ptr
        let nwritten_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&nwritten_memarg, nwritten_ptr as i32, total_written)
            .map_err(|_| super::error::WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn fd_read(
        &self,
        _memory: &MemAddr,
        _fd: Fd,
        _iovs_ptr: Ptr,
        _iovs_len: Size,
        _nread_ptr: Ptr,
    ) -> WasiResult<i32> {
        eprintln!("WASI fd_read called - exiting for debugging");
        std::process::exit(1);
    }

    pub fn proc_exit(&self, exit_code: ExitCode) -> WasiResult<i32> {
        std::process::exit(exit_code);
    }

    pub fn random_get(&self, memory: &MemAddr, buf_ptr: Ptr, buf_len: Size) -> WasiResult<i32> {
        if buf_len == 0 {
            return Ok(0);
        }

        // Generate random bytes
        let mut random_bytes = vec![0u8; buf_len as usize];
        getrandom(&mut random_bytes).map_err(|_| WasiError::IoError)?;

        // Write random bytes to WebAssembly memory
        let byte_memarg = Memarg {
            offset: 0,
            align: 1,
        };

        for (i, &byte) in random_bytes.iter().enumerate() {
            memory
                .store(&byte_memarg, (buf_ptr + i as u32) as i32, byte)
                .map_err(|_| WasiError::MemoryAccessError)?;
        }

        Ok(0)
    }

    pub fn fd_close(&self, _fd: Fd) -> WasiResult<i32> {
        eprintln!("WASI fd_close called - exiting for debugging");
        std::process::exit(1);
    }

    pub fn environ_get(
        &self,
        _memory: &MemAddr,
        _environ_ptr: Ptr,
        _environ_buf_ptr: Ptr,
    ) -> WasiResult<i32> {
        eprintln!("WASI environ_get called - exiting for debugging");
        std::process::exit(1);
    }

    pub fn environ_sizes_get(
        &self,
        _memory: &MemAddr,
        _environ_count_ptr: Ptr,
        _environ_buf_size_ptr: Ptr,
    ) -> WasiResult<i32> {
        eprintln!("WASI environ_sizes_get called - exiting for debugging");
        std::process::exit(1);
    }
}
