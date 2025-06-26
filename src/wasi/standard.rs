use super::context::*;
use super::*;
use crate::execution::mem::MemAddr;
use crate::structure::instructions::Memarg;
use getrandom::getrandom;
use std::io::{self, Read, Write};
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
        memory: &MemAddr,
        fd: Fd,
        iovs_ptr: Ptr,
        iovs_len: Size,
        nread_ptr: Ptr,
    ) -> WasiResult<i32> {
        match fd {
            0 => {
                // stdin - proceed with reading
            }
            1 | 2 => {
                // stdout and stderr - not readable
                return Err(super::error::WasiError::BadFileDescriptor);
            }
            _ => {
                // Other file descriptors - not implemented yet
                return Err(super::error::WasiError::NotImplemented);
            }
        }

        // Collect all iovec information first
        let mut iovecs = Vec::new();
        let mut total_buf_size = 0u32;

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

            if buf_len > 0 {
                iovecs.push((buf_ptr, buf_len));
                total_buf_size += buf_len;
            }
        }

        if total_buf_size == 0 {
            let nread_memarg = Memarg {
                offset: 0,
                align: 4,
            };
            memory
                .store(&nread_memarg, nread_ptr as i32, 0u32)
                .map_err(|_| super::error::WasiError::MemoryAccessError)?;
            return Ok(0);
        }

        // Read data from stdin in one operation (like POSIX readv)
        let mut input_buffer = vec![0u8; total_buf_size as usize];
        let bytes_read = io::stdin()
            .read(&mut input_buffer)
            .map_err(|_| super::error::WasiError::IoError)?;

        let mut total_written = 0u32;
        let mut data_offset = 0usize;

        for (buf_ptr, buf_len) in iovecs {
            if data_offset >= bytes_read {
                break;
            }

            let bytes_to_write = std::cmp::min(buf_len as usize, bytes_read - data_offset);

            let data_slice = &input_buffer[data_offset..data_offset + bytes_to_write];

            // Write all bytes at once using multiple store operations in chunks
            const CHUNK_SIZE: usize = 4;
            let mut written_in_chunk = 0;

            while written_in_chunk + CHUNK_SIZE <= bytes_to_write {
                let chunk_data = u32::from_le_bytes([
                    data_slice[written_in_chunk],
                    data_slice[written_in_chunk + 1],
                    data_slice[written_in_chunk + 2],
                    data_slice[written_in_chunk + 3],
                ]);

                let u32_memarg = Memarg {
                    offset: 0,
                    align: 4,
                };
                memory
                    .store(
                        &u32_memarg,
                        (buf_ptr + written_in_chunk as u32) as i32,
                        chunk_data,
                    )
                    .map_err(|_| super::error::WasiError::MemoryAccessError)?;

                written_in_chunk += CHUNK_SIZE;
            }

            // Write remaining bytes individually
            let byte_memarg = Memarg {
                offset: 0,
                align: 1,
            };
            for j in written_in_chunk..bytes_to_write {
                memory
                    .store(&byte_memarg, (buf_ptr + j as u32) as i32, data_slice[j])
                    .map_err(|_| super::error::WasiError::MemoryAccessError)?;
            }

            total_written += bytes_to_write as u32;
            data_offset += bytes_to_write;
        }

        let total_read = total_written;

        // Write the total number of bytes read to nread_ptr
        let nread_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&nread_memarg, nread_ptr as i32, total_read)
            .map_err(|_| super::error::WasiError::MemoryAccessError)?;

        Ok(0)
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
