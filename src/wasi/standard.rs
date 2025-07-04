use super::context::*;
use super::*;
use crate::execution::mem::MemAddr;
use crate::structure::instructions::Memarg;
use getrandom::getrandom;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};

/// Standard WASI implementation
pub struct StandardWasiImpl {
    context: Arc<Mutex<WasiContext>>,
    preopen_dirs: HashMap<Fd, String>,
}

impl StandardWasiImpl {
    pub fn new(preopen_paths: Vec<String>) -> Self {
        let mut preopen_dirs = HashMap::new();

        // Assign FDs starting from 3 (after stdin=0, stdout=1, stderr=2)
        for (index, path) in preopen_paths.iter().enumerate() {
            let fd = 3 + index as Fd;
            preopen_dirs.insert(fd, path.clone());
        }

        Self {
            context: Arc::new(Mutex::new(WasiContext::new())),
            preopen_dirs,
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

    pub fn fd_close(&self, fd: Fd) -> WasiResult<i32> {
        match fd {
            0 | 1 | 2 => {
                // Standard file descriptors (stdin, stdout, stderr)
                // These are always open and cannot be closed in this simple implementation
                // Return success to maintain compatibility
                Ok(0)
            }
            _ => {
                // Other file descriptors - not implemented yet
                // In a full implementation, this would close files opened with path_open
                Err(WasiError::BadFileDescriptor)
            }
        }
    }

    pub fn environ_get(
        &self,
        memory: &MemAddr,
        environ_ptr: Ptr,
        environ_buf_ptr: Ptr,
    ) -> WasiResult<i32> {
        // Collect all environment variables
        let env_vars: Vec<String> = std::env::vars()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect();

        let mut buf_offset = 0u32;
        let ptr_size = 4u32; // 32-bit pointers

        for (i, env_var) in env_vars.iter().enumerate() {
            let env_bytes = env_var.as_bytes();

            // Write pointer to environ_ptr array
            let ptr_addr = environ_ptr + (i as u32 * ptr_size);
            let string_addr = environ_buf_ptr + buf_offset;

            let ptr_memarg = Memarg {
                offset: 0,
                align: 4,
            };
            memory
                .store(&ptr_memarg, ptr_addr as i32, string_addr)
                .map_err(|_| WasiError::MemoryAccessError)?;

            // Write environment variable string to buffer
            let byte_memarg = Memarg {
                offset: 0,
                align: 1,
            };

            for (j, &byte) in env_bytes.iter().enumerate() {
                memory
                    .store(&byte_memarg, (string_addr + j as u32) as i32, byte)
                    .map_err(|_| WasiError::MemoryAccessError)?;
            }

            // Write null terminator
            memory
                .store(
                    &byte_memarg,
                    (string_addr + env_bytes.len() as u32) as i32,
                    0u8,
                )
                .map_err(|_| WasiError::MemoryAccessError)?;

            buf_offset += env_bytes.len() as u32 + 1; // +1 for null terminator
        }

        // Write null pointer at the end of environ_ptr array
        let final_ptr_addr = environ_ptr + (env_vars.len() as u32 * ptr_size);
        let ptr_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&ptr_memarg, final_ptr_addr as i32, 0u32)
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn environ_sizes_get(
        &self,
        memory: &MemAddr,
        environ_count_ptr: Ptr,
        environ_buf_size_ptr: Ptr,
    ) -> WasiResult<i32> {
        let env_vars: Vec<String> = std::env::vars()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect();

        let environ_count = env_vars.len() as u32;
        let environ_buf_size: u32 = env_vars
            .iter()
            .map(|env_var| env_var.len() as u32 + 1) // +1 for null terminator
            .sum();

        // Write environment variable count
        let count_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&count_memarg, environ_count_ptr as i32, environ_count)
            .map_err(|_| WasiError::MemoryAccessError)?;

        // Write total buffer size needed
        let size_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&size_memarg, environ_buf_size_ptr as i32, environ_buf_size)
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn args_get(&self, memory: &MemAddr, argv_ptr: Ptr, argv_buf_ptr: Ptr) -> WasiResult<i32> {
        let args: Vec<String> = std::env::args().collect();

        let mut buf_offset = 0u32;
        let ptr_size = 4u32; // 32-bit pointers

        // Write argument strings to buffer and pointers to pointer array
        for (i, arg) in args.iter().enumerate() {
            let arg_bytes = arg.as_bytes();

            // Write pointer to argv_ptr array
            let ptr_addr = argv_ptr + (i as u32 * ptr_size);
            let string_addr = argv_buf_ptr + buf_offset;

            let ptr_memarg = Memarg {
                offset: 0,
                align: 4,
            };
            memory
                .store(&ptr_memarg, ptr_addr as i32, string_addr)
                .map_err(|_| WasiError::MemoryAccessError)?;

            // Write argument string to buffer
            let byte_memarg = Memarg {
                offset: 0,
                align: 1,
            };

            for (j, &byte) in arg_bytes.iter().enumerate() {
                memory
                    .store(&byte_memarg, (string_addr + j as u32) as i32, byte)
                    .map_err(|_| WasiError::MemoryAccessError)?;
            }

            // Write null terminator
            memory
                .store(
                    &byte_memarg,
                    (string_addr + arg_bytes.len() as u32) as i32,
                    0u8,
                )
                .map_err(|_| WasiError::MemoryAccessError)?;

            buf_offset += arg_bytes.len() as u32 + 1;
        }

        // Write null pointer at the end of argv_ptr array
        let final_ptr_addr = argv_ptr + (args.len() as u32 * ptr_size);
        let ptr_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&ptr_memarg, final_ptr_addr as i32, 0u32)
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn args_sizes_get(
        &self,
        memory: &MemAddr,
        argc_ptr: Ptr,
        argv_buf_size_ptr: Ptr,
    ) -> WasiResult<i32> {
        let args: Vec<String> = std::env::args().collect();

        let argc = args.len() as u32;
        let argv_buf_size: u32 = args.iter().map(|arg| arg.len() as u32 + 1).sum();

        // Write argument count
        let count_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&count_memarg, argc_ptr as i32, argc)
            .map_err(|_| WasiError::MemoryAccessError)?;

        // Write total buffer size needed
        let size_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&size_memarg, argv_buf_size_ptr as i32, argv_buf_size)
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn clock_time_get(
        &self,
        memory: &MemAddr,
        clock_id: i32,
        _precision: i64,
        time_ptr: Ptr,
    ) -> WasiResult<i32> {
        // WASI clock IDs (from WASI specification)
        // 0: CLOCK_REALTIME - Wall clock time
        // 1: CLOCK_MONOTONIC - Monotonic time since some unspecified starting point
        // 2: CLOCK_PROCESS_CPUTIME_ID - CPU time consumed by this process
        // 3: CLOCK_THREAD_CPUTIME_ID - CPU time consumed by this thread

        let time_ns = match clock_id {
            0 | 1 => {
                // Use system time for both realtime and monotonic (simplified)
                use std::time::{SystemTime, UNIX_EPOCH};
                let duration = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| WasiError::IoError)?;

                // Convert to nanoseconds
                duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as u64
            }
            2 | 3 => {
                // Process/thread CPU time - not implemented in WASI
                return Err(WasiError::NotImplemented);
            }
            _ => {
                return Err(WasiError::InvalidArgument);
            }
        };

        // Write timestamp (64-bit nanoseconds) to memory
        let time_memarg = Memarg {
            offset: 0,
            align: 8,
        };
        memory
            .store(&time_memarg, time_ptr as i32, time_ns as i64)
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn clock_res_get(
        &self,
        memory: &MemAddr,
        clock_id: i32,
        resolution_ptr: Ptr,
    ) -> WasiResult<i32> {
        // Clock resolution in nanoseconds
        let resolution_ns = match clock_id {
            0 | 1 => {
                // System clock resolution - typically 1 microsecond
                1_000u64 // 1 microsecond in nanoseconds
            }
            2 | 3 => {
                // Process/thread CPU time resolution - not implemented in WASI
                return Err(WasiError::NotImplemented);
            }
            _ => {
                return Err(WasiError::InvalidArgument);
            }
        };

        // Write resolution (64-bit nanoseconds) to memory
        let res_memarg = Memarg {
            offset: 0,
            align: 8,
        };
        memory
            .store(&res_memarg, resolution_ptr as i32, resolution_ns as i64)
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(0)
    }

    /// Get directory name for a preopen directory FD from our mapping
    fn get_preopen_dir_name(&self, fd: Fd) -> Result<String, WasiError> {
        if let Some(path) = self.preopen_dirs.get(&fd) {
            Ok(path.clone())
        } else {
            Err(WasiError::BadFileDescriptor)
        }
    }

    pub fn fd_prestat_get(&self, memory: &MemAddr, fd: Fd, prestat_ptr: Ptr) -> WasiResult<i32> {
        // Get the directory name to determine name length
        let dir_name = self.get_preopen_dir_name(fd)?;
        let name_len = dir_name.len() as u32;

        // Write prestat structure to memory
        // prestat structure: tag (u8) + padding (3 bytes) + union (depends on tag)
        // For PREOPENTYPE_DIR (tag=0): u32 name_len

        let tag_memarg = Memarg {
            offset: 0,
            align: 1,
        };
        // Write tag = 0 (PREOPENTYPE_DIR)
        memory
            .store(&tag_memarg, prestat_ptr as i32, 0u8)
            .map_err(|_| WasiError::MemoryAccessError)?;

        let name_len_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        // Write name length at offset 4 (after tag + 3 bytes padding)
        memory
            .store(&name_len_memarg, (prestat_ptr + 4) as i32, name_len)
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn fd_prestat_dir_name(
        &self,
        memory: &MemAddr,
        fd: Fd,
        path_ptr: Ptr,
        path_len: Size,
    ) -> WasiResult<i32> {
        // Get the directory name
        let dir_name = self.get_preopen_dir_name(fd)?;
        let dir_bytes = dir_name.as_bytes();

        // Check if provided buffer is large enough
        if path_len < dir_bytes.len() as u32 {
            return Err(WasiError::InvalidArgument);
        }

        let byte_memarg = Memarg {
            offset: 0,
            align: 1,
        };

        // Write directory name to memory
        for (i, &byte) in dir_bytes.iter().enumerate() {
            memory
                .store(&byte_memarg, (path_ptr + i as u32) as i32, byte)
                .map_err(|_| WasiError::MemoryAccessError)?;
        }
        Ok(0)
    }

    pub fn sched_yield(&self) -> WasiResult<i32> {
        std::thread::yield_now();
        Ok(0)
    }

    pub fn fd_fdstat_get(&self, memory: &MemAddr, fd: Fd, stat_ptr: Ptr) -> WasiResult<i32> {
        // File types (WASI spec)
        const FILETYPE_REGULAR_FILE: u8 = 4;
        const FILETYPE_CHARACTER_DEVICE: u8 = 2;
        const FILETYPE_UNKNOWN: u8 = 0;

        // FD flags (simplified)
        const NO_FLAGS: u16 = 0;
        const FDFLAGS_APPEND: u16 = 0x0001;
        const FDFLAGS_DSYNC: u16 = 0x0002;
        const FDFLAGS_NONBLOCK: u16 = 0x0004;
        const FDFLAGS_SYNC: u16 = 0x0010;

        // Rights (simplified set of common rights)
        const NO_INHERITING_RIGHTS: u64 = 0;
        const RIGHTS_FD_READ: u64 = 0x0000000000000002;
        const RIGHTS_FD_WRITE: u64 = 0x0000000000000040;

        let (filetype, flags, rights_base, rights_inheriting) = match fd {
            0 => {
                // stdin - character device, readable
                (
                    FILETYPE_CHARACTER_DEVICE,
                    NO_FLAGS,
                    RIGHTS_FD_READ,
                    NO_INHERITING_RIGHTS,
                )
            }
            1 | 2 => {
                // stdout/stderr - character device, writable
                (
                    FILETYPE_CHARACTER_DEVICE,
                    NO_FLAGS,
                    RIGHTS_FD_WRITE,
                    NO_INHERITING_RIGHTS,
                )
            }
            _ => {
                if self.preopen_dirs.contains_key(&fd) {
                    (
                        FILETYPE_REGULAR_FILE,
                        NO_FLAGS,
                        RIGHTS_FD_READ | RIGHTS_FD_WRITE,
                        RIGHTS_FD_READ | RIGHTS_FD_WRITE,
                    )
                } else {
                    return Err(WasiError::BadFileDescriptor);
                }
            }
        };

        // Write fdstat structure to memory
        // Structure layout: filetype(u8) + flags(u16) + rights_base(u64) + rights_inheriting(u64)
        // Total size: 1 + 2 + 8 + 8 = 19 bytes, but with alignment it's typically 24 bytes

        // Write filetype (u8)
        let filetype_memarg = Memarg {
            offset: 0,
            align: 1,
        };
        memory
            .store(&filetype_memarg, stat_ptr as i32, filetype)
            .map_err(|_| WasiError::MemoryAccessError)?;

        // Write flags (u16) at offset 2 (with padding)
        let flags_memarg = Memarg {
            offset: 0,
            align: 2,
        };
        memory
            .store(&flags_memarg, (stat_ptr + 2) as i32, flags)
            .map_err(|_| WasiError::MemoryAccessError)?;

        // Write rights_base (u64) at offset 8
        let rights_base_memarg = Memarg {
            offset: 0,
            align: 8,
        };
        memory
            .store(
                &rights_base_memarg,
                (stat_ptr + 8) as i32,
                rights_base as i64,
            )
            .map_err(|_| WasiError::MemoryAccessError)?;

        // Write rights_inheriting (u64) at offset 16
        let rights_inheriting_memarg = Memarg {
            offset: 0,
            align: 8,
        };
        memory
            .store(
                &rights_inheriting_memarg,
                (stat_ptr + 16) as i32,
                rights_inheriting as i64,
            )
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(0)
    }
}
