use super::context::*;
use super::*;
use crate::execution::mem::MemAddr;
use crate::structure::instructions::Memarg;
use getrandom::getrandom;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Opened file information
#[derive(Debug)]
pub struct OpenFile {
    pub file: Option<File>, // None for directories
    pub path: PathBuf,
    pub flags: u32,
    pub rights_base: u64,
    pub rights_inheriting: u64,
    pub is_directory: bool,
    pub seek_position: u64, // Current seek position
}

/// WASI file types
#[derive(Debug, Clone, Copy)]
enum WasiFileType {
    Unknown = 0,
    BlockDevice = 1,
    CharacterDevice = 2, // Terminal devices like stdin/stdout/stderr
    Directory = 3,       // Directory/folder
    RegularFile = 4,     // Regular files like .txt, .exe, etc.
    SocketDgram = 5,     // UDP socket
    SocketStream = 6,    // TCP socket
    SymbolicLink = 7,    // Symbolic link
}

/// Standard WASI implementation
pub struct StandardWasiImpl {
    context: Arc<Mutex<WasiContext>>,
    preopen_dirs: HashMap<Fd, String>,
    opened_files: Arc<Mutex<HashMap<Fd, OpenFile>>>,
    next_fd: Arc<Mutex<Fd>>,
}

impl StandardWasiImpl {
    pub fn new(preopen_paths: Vec<String>) -> Self {
        let mut preopen_dirs = HashMap::new();

        // Assign FDs starting from 3 (after stdin=0, stdout=1, stderr=2)
        for (index, path) in preopen_paths.iter().enumerate() {
            let fd = 3 + index as Fd;
            preopen_dirs.insert(fd, path.clone());
        }

        // Next available FD starts after preopen directories
        let next_fd = 3 + preopen_paths.len() as Fd;

        Self {
            context: Arc::new(Mutex::new(WasiContext::new())),
            preopen_dirs,
            opened_files: Arc::new(Mutex::new(HashMap::new())),
            next_fd: Arc::new(Mutex::new(next_fd)),
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
        let is_stdout_stderr = match fd {
            0 => {
                return Err(super::error::WasiError::BadFileDescriptor);
            }
            1 | 2 => true,
            _ => {
                if self.preopen_dirs.contains_key(&fd) {
                    return Err(super::error::WasiError::BadFileDescriptor);
                }

                let opened_files = self.opened_files.lock().unwrap();
                if let Some(open_file) = opened_files.get(&fd) {
                    if open_file.is_directory {
                        return Err(super::error::WasiError::BadFileDescriptor);
                    }
                    false
                } else {
                    return Err(super::error::WasiError::BadFileDescriptor);
                }
            }
        };

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
            let bytes_written = if is_stdout_stderr {
                match fd {
                    1 => io::stdout()
                        .write(&data)
                        .map_err(|_| super::error::WasiError::IoError)?,
                    2 => io::stderr()
                        .write(&data)
                        .map_err(|_| super::error::WasiError::IoError)?,
                    _ => unreachable!(),
                }
            } else {
                let mut opened_files = self.opened_files.lock().unwrap();
                if let Some(open_file) = opened_files.get_mut(&fd) {
                    if let Some(ref mut file) = open_file.file {
                        // Set seek position if needed
                        file.seek(SeekFrom::Start(open_file.seek_position))
                            .map_err(|_| super::error::WasiError::IoError)?;

                        // Write data
                        let written = file
                            .write(&data)
                            .map_err(|_| super::error::WasiError::IoError)?;

                        // Update seek position
                        open_file.seek_position += written as u64;

                        written
                    } else {
                        return Err(super::error::WasiError::BadFileDescriptor);
                    }
                } else {
                    return Err(super::error::WasiError::BadFileDescriptor);
                }
            };

            total_written += bytes_written as u32;
        }

        // Flush output to ensure data is written
        if is_stdout_stderr {
            match fd {
                1 => io::stdout()
                    .flush()
                    .map_err(|_| super::error::WasiError::IoError)?,
                2 => io::stderr()
                    .flush()
                    .map_err(|_| super::error::WasiError::IoError)?,
                _ => unreachable!(),
            }
        } else {
            let opened_files = self.opened_files.lock().unwrap();
            if let Some(open_file) = opened_files.get(&fd) {
                if let Some(ref file) = open_file.file {
                    file.sync_all()
                        .map_err(|_| super::error::WasiError::IoError)?;
                }
            }
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
        // Validate file descriptor and determine source type
        let is_stdin = match fd {
            0 => true, // stdin
            1 | 2 => {
                // stdout and stderr - not readable
                return Err(super::error::WasiError::BadFileDescriptor);
            }
            _ => {
                if self.preopen_dirs.contains_key(&fd) {
                    return Err(super::error::WasiError::BadFileDescriptor);
                }

                // Check if it's an opened file
                let opened_files = self.opened_files.lock().unwrap();
                if let Some(open_file) = opened_files.get(&fd) {
                    if open_file.is_directory {
                        return Err(super::error::WasiError::BadFileDescriptor);
                    }
                    false
                } else {
                    return Err(super::error::WasiError::BadFileDescriptor);
                }
            }
        };

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

        // Read data from stdin or file in one operation (like POSIX readv)
        let mut input_buffer = vec![0u8; total_buf_size as usize];
        let bytes_read = if is_stdin {
            // Read from stdin
            io::stdin()
                .read(&mut input_buffer)
                .map_err(|_| super::error::WasiError::IoError)?
        } else {
            // Read from opened file
            let mut opened_files = self.opened_files.lock().unwrap();
            if let Some(open_file) = opened_files.get_mut(&fd) {
                if let Some(ref mut file) = open_file.file {
                    // Use seek position if available
                    use std::io::Seek;
                    let _ = file.seek(io::SeekFrom::Start(open_file.seek_position));

                    let bytes_read = file
                        .read(&mut input_buffer)
                        .map_err(|_| super::error::WasiError::IoError)?;

                    // Update seek position
                    open_file.seek_position += bytes_read as u64;

                    bytes_read
                } else {
                    return Err(super::error::WasiError::BadFileDescriptor);
                }
            } else {
                return Err(super::error::WasiError::BadFileDescriptor);
            }
        };

        let mut total_written = 0u32;
        let mut data_offset = 0usize;

        for (buf_ptr, buf_len) in iovecs {
            if data_offset >= bytes_read {
                break;
            }

            let bytes_to_write = std::cmp::min(buf_len as usize, bytes_read - data_offset);

            let data_slice = &input_buffer[data_offset..data_offset + bytes_to_write];

            // Use bulk write for efficiency
            memory
                .store_bytes(buf_ptr as i32, data_slice)
                .map_err(|_| super::error::WasiError::MemoryAccessError)?;

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

        // Write random bytes to WebAssembly memory in bulk
        memory
            .store_bytes(buf_ptr as i32, &random_bytes)
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn fd_close(&self, fd: Fd) -> WasiResult<i32> {
        match fd {
            0 | 1 | 2 => {
                // Standard file descriptors (stdin, stdout, stderr)
                Ok(0)
            }
            _ => {
                if self.preopen_dirs.contains_key(&fd) {
                    return Err(WasiError::BadFileDescriptor);
                }

                let mut opened_files = self.opened_files.lock().unwrap();
                if let Some(open_file) = opened_files.remove(&fd) {
                    if let Some(file) = open_file.file {
                        let _ = file.sync_all();
                    }
                    Ok(0)
                } else {
                    Err(WasiError::BadFileDescriptor)
                }
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

        let ptr_size = 4u32;

        let mut ptr_data = Vec::with_capacity((env_vars.len() + 1) * ptr_size as usize);
        let mut buf_offset = 0u32;

        // Calculate pointers and build pointer array
        for env_var in &env_vars {
            let string_addr = environ_buf_ptr + buf_offset;
            ptr_data.extend_from_slice(&string_addr.to_le_bytes());
            buf_offset += env_var.len() as u32 + 1;
        }
        ptr_data.extend_from_slice(&0u32.to_le_bytes());

        let mut buf_data = Vec::with_capacity(buf_offset as usize);
        for env_var in &env_vars {
            buf_data.extend_from_slice(env_var.as_bytes());
            buf_data.push(0);
        }

        memory
            .store_bytes(environ_ptr as i32, &ptr_data)
            .map_err(|_| WasiError::MemoryAccessError)?;

        memory
            .store_bytes(environ_buf_ptr as i32, &buf_data)
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

        let ptr_size = 4u32;

        let mut ptr_data = Vec::with_capacity((args.len() + 1) * ptr_size as usize);
        let mut buf_offset = 0u32;

        // Calculate pointers and build pointer array
        for arg in &args {
            let string_addr = argv_buf_ptr + buf_offset;
            ptr_data.extend_from_slice(&string_addr.to_le_bytes());
            buf_offset += arg.len() as u32 + 1;
        }
        ptr_data.extend_from_slice(&0u32.to_le_bytes());

        let mut buf_data = Vec::with_capacity(buf_offset as usize);
        for arg in &args {
            buf_data.extend_from_slice(arg.as_bytes());
            buf_data.push(0); // null terminator
        }

        memory
            .store_bytes(argv_ptr as i32, &ptr_data)
            .map_err(|_| WasiError::MemoryAccessError)?;

        memory
            .store_bytes(argv_buf_ptr as i32, &buf_data)
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
                    WasiFileType::CharacterDevice as u8,
                    NO_FLAGS,
                    RIGHTS_FD_READ,
                    NO_INHERITING_RIGHTS,
                )
            }
            1 | 2 => {
                // stdout/stderr - character device, writable
                (
                    WasiFileType::CharacterDevice as u8,
                    NO_FLAGS,
                    RIGHTS_FD_WRITE,
                    NO_INHERITING_RIGHTS,
                )
            }
            _ => {
                if self.preopen_dirs.contains_key(&fd) {
                    (
                        WasiFileType::Directory as u8,
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

    pub fn path_open(
        &self,
        memory: &MemAddr,
        fd: Fd,
        dirflags: u32,
        path_ptr: Ptr,
        path_len: Size,
        oflags: u32,
        fs_rights_base: u64,
        fs_rights_inheriting: u64,
        fdflags: u32,
        opened_fd_ptr: Ptr,
    ) -> WasiResult<i32> {
        // WASI dirflags constants
        const LOOKUPFLAGS_SYMLINK_FOLLOW: u32 = 0x0001;

        // WASI oflags constants
        const OFLAGS_CREAT: u32 = 0x0001;
        const OFLAGS_DIRECTORY: u32 = 0x0002;
        const OFLAGS_EXCL: u32 = 0x0004;
        const OFLAGS_TRUNC: u32 = 0x0008;

        // Read path string from memory
        let mut path_bytes = vec![0u8; path_len as usize];
        for i in 0..path_len {
            let byte: u8 = memory
                .load(
                    &Memarg {
                        offset: 0,
                        align: 1,
                    },
                    (path_ptr + i) as i32,
                )
                .map_err(|_| WasiError::MemoryAccessError)?;
            path_bytes[i as usize] = byte;
        }
        let path_str = String::from_utf8(path_bytes).map_err(|_| WasiError::InvalidArgument)?;

        // Resolve base directory
        let base_dir = if let Some(preopen_path) = self.preopen_dirs.get(&fd) {
            PathBuf::from(preopen_path)
        } else {
            return Err(WasiError::BadFileDescriptor);
        };

        // Construct full path
        let mut full_path = base_dir.join(&path_str);

        // Handle symbolic link resolution based on dirflags
        let follow_symlinks = dirflags & LOOKUPFLAGS_SYMLINK_FOLLOW != 0;

        if follow_symlinks {
            full_path = full_path.canonicalize().map_err(|e| match e.kind() {
                io::ErrorKind::NotFound => WasiError::NoSuchFileOrDirectory,
                io::ErrorKind::PermissionDenied => WasiError::NotPermitted,
                _ => WasiError::Io,
            })?;
        } else {
            if full_path.is_symlink() {
                if !full_path.exists() {
                    return Err(WasiError::NoSuchFileOrDirectory);
                }
            }
        }

        // Validate path is within base directory
        if !full_path.starts_with(&base_dir) {
            return Err(WasiError::NotPermitted);
        }

        let is_directory_request = oflags & OFLAGS_DIRECTORY != 0;

        // Validate path exists
        if !follow_symlinks && !full_path.exists() {
            return Err(WasiError::NoSuchFileOrDirectory);
        }

        // Validate it's actually a directory
        if is_directory_request {
            if !full_path.is_dir() {
                return Err(WasiError::NotDirectory);
            }
        }

        let (file_handle, is_directory) = if is_directory_request {
            (None, true)
        } else {
            let mut open_options = OpenOptions::new();

            // Read/write permissions based on rights
            const RIGHTS_FD_READ: u64 = 0x0000000000000002;
            const RIGHTS_FD_WRITE: u64 = 0x0000000000000040;

            if fs_rights_base & RIGHTS_FD_READ != 0 {
                open_options.read(true);
            }
            if fs_rights_base & RIGHTS_FD_WRITE != 0 {
                open_options.write(true);
            }

            if oflags & OFLAGS_CREAT != 0 {
                open_options.create(true);
            }
            if oflags & OFLAGS_EXCL != 0 {
                open_options.create_new(true);
            }
            if oflags & OFLAGS_TRUNC != 0 {
                open_options.truncate(true);
            }

            // Append mode based on fdflags
            const FDFLAGS_APPEND: u32 = 0x0001;
            if fdflags & FDFLAGS_APPEND != 0 {
                open_options.append(true);
            }

            // Open the file
            let file = open_options.open(&full_path).map_err(|e| match e.kind() {
                io::ErrorKind::NotFound => WasiError::NoSuchFileOrDirectory,
                io::ErrorKind::PermissionDenied => WasiError::NotPermitted,
                io::ErrorKind::AlreadyExists => WasiError::Exist,
                _ => WasiError::Io,
            })?;

            (Some(file), false)
        };

        // Allocate new file descriptor
        let mut next_fd_guard = self.next_fd.lock().unwrap();
        let new_fd = *next_fd_guard;
        *next_fd_guard += 1;
        drop(next_fd_guard);

        // Store opened file
        let open_file = OpenFile {
            file: file_handle,
            path: full_path,
            flags: fdflags,
            rights_base: fs_rights_base,
            rights_inheriting: fs_rights_inheriting,
            is_directory,
            seek_position: 0, // Start at beginning of file
        };

        let mut opened_files = self.opened_files.lock().unwrap();
        opened_files.insert(new_fd, open_file);
        drop(opened_files);

        // Write result FD to memory
        let fd_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&fd_memarg, opened_fd_ptr as i32, new_fd)
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn fd_seek(&self, fd: Fd, offset: i64, whence: u32) -> WasiResult<u64> {
        // WASI whence constants
        const WHENCE_SET: u32 = 0; // Seek from beginning of file
        const WHENCE_CUR: u32 = 1; // Seek from current position
        const WHENCE_END: u32 = 2; // Seek from end of file

        // Check if fd is a standard stream (not seekable)
        if fd <= 2 {
            return Err(WasiError::InvalidArgument);
        }

        // Get the opened file
        let mut opened_files = self.opened_files.lock().unwrap();
        let open_file = opened_files
            .get_mut(&fd)
            .ok_or(WasiError::BadFileDescriptor)?;

        // Check if it's a directory (not seekable)
        if open_file.is_directory {
            return Err(WasiError::InvalidArgument);
        }

        // Get the file handle
        let file = open_file
            .file
            .as_mut()
            .ok_or(WasiError::BadFileDescriptor)?;

        // Calculate new position based on whence
        let new_position = match whence {
            WHENCE_SET => {
                // Seek from beginning
                if offset < 0 {
                    return Err(WasiError::InvalidArgument);
                }
                offset as u64
            }
            WHENCE_CUR => {
                // Seek from current position
                let current_pos = open_file.seek_position;
                current_pos
                    .checked_add_signed(offset)
                    .ok_or(WasiError::InvalidArgument)?
            }
            WHENCE_END => {
                // Seek from end of file
                let file_size = file.metadata().map_err(|_| WasiError::Io)?.len();
                file_size
                    .checked_add_signed(offset)
                    .ok_or(WasiError::InvalidArgument)?
            }
            _ => return Err(WasiError::InvalidArgument),
        };

        // Perform the seek operation
        file.seek(SeekFrom::Start(new_position))
            .map_err(|_| WasiError::Io)?;

        // Update our tracked position
        open_file.seek_position = new_position;

        // Return the new position
        Ok(new_position)
    }

    pub fn fd_tell(&self, memory: &MemAddr, fd: Fd, offset_ptr: Ptr) -> WasiResult<i32> {
        if fd <= 2 {
            return Err(WasiError::InvalidArgument);
        }

        let opened_files = self.opened_files.lock().unwrap();
        let open_file = opened_files.get(&fd).ok_or(WasiError::BadFileDescriptor)?;

        if open_file.is_directory {
            return Err(WasiError::InvalidArgument);
        }

        let current_position = open_file.seek_position;

        let offset_memarg = Memarg {
            offset: 0,
            align: 8,
        };
        memory
            .store(&offset_memarg, offset_ptr as i32, current_position as i64)
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn fd_sync(&self, fd: Fd) -> WasiResult<i32> {
        match fd {
            0 => {
                return Err(WasiError::BadFileDescriptor);
            }
            1 => {
                io::stdout().flush().map_err(|_| WasiError::IoError)?;
                return Ok(0);
            }
            2 => {
                io::stderr().flush().map_err(|_| WasiError::IoError)?;
                return Ok(0);
            }
            _ => {}
        }

        if self.preopen_dirs.contains_key(&fd) {
            return Err(WasiError::BadFileDescriptor);
        }

        let opened_files = self.opened_files.lock().unwrap();
        let open_file = opened_files.get(&fd).ok_or(WasiError::BadFileDescriptor)?;

        if open_file.is_directory {
            return Err(WasiError::BadFileDescriptor);
        }

        if let Some(ref file) = open_file.file {
            file.sync_all().map_err(|_| WasiError::IoError)?;
            Ok(0)
        } else {
            Err(WasiError::BadFileDescriptor)
        }
    }

    // Helper function to write filestat structure to memory
    fn write_filestat_to_memory(
        memory: &MemAddr,
        filestat_ptr: Ptr,
        device: u64,
        inode: u64,
        filetype: u8,
        linkcount: u64,
        size: u64,
        atim: u64,
        mtim: u64,
        ctim: u64,
    ) -> WasiResult<()> {
        // Build complete filestat structure in memory (64 bytes)
        let mut filestat_data = Vec::with_capacity(64);

        // filestat layout (64 bytes total):
        // device (u64) - offset 0
        filestat_data.extend_from_slice(&device.to_le_bytes());
        // inode (u64) - offset 8
        filestat_data.extend_from_slice(&inode.to_le_bytes());
        // filetype (u8) - offset 16, stored as u32 for alignment
        filestat_data.extend_from_slice(&(filetype as u32).to_le_bytes());
        // padding for alignment - offset 20
        filestat_data.extend_from_slice(&0u32.to_le_bytes());
        // linkcount (u64) - offset 24
        filestat_data.extend_from_slice(&linkcount.to_le_bytes());
        // size (u64) - offset 32
        filestat_data.extend_from_slice(&size.to_le_bytes());
        // atim (u64) - offset 40
        filestat_data.extend_from_slice(&atim.to_le_bytes());
        // mtim (u64) - offset 48
        filestat_data.extend_from_slice(&mtim.to_le_bytes());
        // ctim (u64) - offset 56
        filestat_data.extend_from_slice(&ctim.to_le_bytes());

        memory
            .store_bytes(filestat_ptr as i32, &filestat_data)
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(())
    }

    pub fn fd_filestat_get(&self, memory: &MemAddr, fd: Fd, filestat_ptr: Ptr) -> WasiResult<i32> {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Handle standard streams
        match fd {
            0 | 1 | 2 => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;

                Self::write_filestat_to_memory(
                    memory,
                    filestat_ptr,
                    0,                                   // device
                    fd as u64,                           // inode (use fd as inode)
                    WasiFileType::CharacterDevice as u8, // filetype
                    1,                                   // linkcount
                    0,                                   // size (0 for streams)
                    now,                                 // atim
                    now,                                 // mtim
                    now,                                 // ctim
                )?;
                return Ok(0);
            }
            _ => {}
        }

        // Check if it's a preopen directory
        if let Some(_) = self.preopen_dirs.get(&fd) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;

            Self::write_filestat_to_memory(
                memory,
                filestat_ptr,
                0,
                fd as u64,
                WasiFileType::Directory as u8,
                1,
                0,
                now,
                now,
                now,
            )?;
            return Ok(0);
        }

        // Get the opened file
        let opened_files = self.opened_files.lock().unwrap();
        let open_file = opened_files.get(&fd).ok_or(WasiError::BadFileDescriptor)?;

        if open_file.is_directory {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;

            Self::write_filestat_to_memory(
                memory,
                filestat_ptr,
                0,
                fd as u64,
                WasiFileType::Directory as u8,
                1,
                0,
                now,
                now,
                now,
            )?;
            return Ok(0);
        }

        // Regular file - get metadata
        if let Some(ref file) = open_file.file {
            let metadata = file.metadata().map_err(|_| WasiError::IoError)?;

            // Get timestamps
            let accessed = metadata
                .accessed()
                .unwrap_or(SystemTime::UNIX_EPOCH)
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            let modified = metadata
                .modified()
                .unwrap_or(SystemTime::UNIX_EPOCH)
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            let created = metadata
                .created()
                .unwrap_or(SystemTime::UNIX_EPOCH)
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;

            Self::write_filestat_to_memory(
                memory,
                filestat_ptr,
                0,
                fd as u64,
                WasiFileType::RegularFile as u8,
                1,
                metadata.len(),
                accessed,
                modified,
                created,
            )?;
            Ok(0)
        } else {
            Err(WasiError::BadFileDescriptor)
        }
    }

    pub fn fd_readdir(
        &self,
        memory: &MemAddr,
        fd: Fd,
        buf_ptr: Ptr,
        buf_len: Size,
        cookie: u64,
        buf_used_ptr: Ptr,
    ) -> WasiResult<i32> {
        use std::fs;
        if fd <= 2 {
            return Err(WasiError::BadFileDescriptor);
        }

        let dir_path = if let Some(preopen_path) = self.preopen_dirs.get(&fd) {
            PathBuf::from(preopen_path)
        } else {
            let opened_files = self.opened_files.lock().unwrap();
            if let Some(open_file) = opened_files.get(&fd) {
                if !open_file.is_directory {
                    return Err(WasiError::NotDirectory);
                }
                open_file.path.clone()
            } else {
                return Err(WasiError::BadFileDescriptor);
            }
        };

        let entries = fs::read_dir(&dir_path).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => WasiError::NoSuchFileOrDirectory,
            io::ErrorKind::PermissionDenied => WasiError::NotPermitted,
            _ => WasiError::IoError,
        })?;

        let mut total_written = 0u32;
        let mut entry_index = 0u64;

        for entry_result in entries {
            let entry = entry_result.map_err(|_| WasiError::IoError)?;

            if entry_index < cookie {
                entry_index += 1;
                continue;
            }

            let file_name = entry.file_name();
            let name_string = file_name.to_string_lossy().into_owned();
            let name_bytes = name_string.as_bytes();
            let name_len = name_bytes.len() as u32;

            // Calculate dirent size: 24 bytes (header) + name length
            let dirent_size = 24 + name_len;

            // Check if there's enough space in the buffer
            if total_written + dirent_size > buf_len {
                break;
            }

            // Get file type
            let metadata = entry.metadata().map_err(|_| WasiError::IoError)?;
            let file_type = if metadata.is_dir() {
                WasiFileType::Directory as u8
            } else if metadata.is_file() {
                WasiFileType::RegularFile as u8
            } else {
                WasiFileType::Unknown as u8
            };

            // Build complete dirent entry in memory
            let mut dirent_data = Vec::with_capacity(24 + name_len as usize);
            dirent_data.extend_from_slice(&(entry_index + 1).to_le_bytes());
            dirent_data.extend_from_slice(&entry_index.to_le_bytes());
            dirent_data.extend_from_slice(&name_len.to_le_bytes());
            dirent_data.push(file_type);
            dirent_data.extend_from_slice(&[0u8; 3]);
            dirent_data.extend_from_slice(name_bytes);

            let dirent_start = buf_ptr + total_written;
            memory
                .store_bytes(dirent_start as i32, &dirent_data)
                .map_err(|_| WasiError::MemoryAccessError)?;

            total_written += dirent_size;
            entry_index += 1;
        }

        let used_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&used_memarg, buf_used_ptr as i32, total_written)
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn fd_pread(
        &self,
        memory: &MemAddr,
        fd: Fd,
        iovs_ptr: Ptr,
        iovs_len: Size,
        offset: u64,
        nread_ptr: Ptr,
    ) -> WasiResult<i32> {
        if fd <= 2 {
            return Err(WasiError::BadFileDescriptor);
        }

        if self.preopen_dirs.contains_key(&fd) {
            return Err(WasiError::BadFileDescriptor);
        }

        let opened_files = self.opened_files.lock().unwrap();
        let open_file = opened_files.get(&fd).ok_or(WasiError::BadFileDescriptor)?;

        if open_file.is_directory {
            return Err(WasiError::BadFileDescriptor);
        }

        let file = open_file
            .file
            .as_ref()
            .ok_or(WasiError::BadFileDescriptor)?;

        // Clone the file handle for positioned read (doesn't change original position)
        let mut file_clone = file.try_clone().map_err(|_| WasiError::IoError)?;

        // Seek to the specified offset
        file_clone
            .seek(SeekFrom::Start(offset))
            .map_err(|_| WasiError::IoError)?;

        let mut total_read = 0u32;

        for i in 0..iovs_len {
            let iov_addr = iovs_ptr + (i * 8);

            let buf_ptr: u32 = memory
                .load(
                    &Memarg {
                        offset: 0,
                        align: 4,
                    },
                    iov_addr as i32,
                )
                .map_err(|_| WasiError::MemoryAccessError)?;

            let buf_len: u32 = memory
                .load(
                    &Memarg {
                        offset: 4,
                        align: 4,
                    },
                    iov_addr as i32,
                )
                .map_err(|_| WasiError::MemoryAccessError)?;

            let mut buffer = vec![0u8; buf_len as usize];
            let bytes_read = file_clone
                .read(&mut buffer)
                .map_err(|_| WasiError::IoError)?;

            // Truncate buffer to actual bytes read
            buffer.truncate(bytes_read);

            memory
                .store_bytes(buf_ptr as i32, &buffer)
                .map_err(|_| WasiError::MemoryAccessError)?;

            total_read += bytes_read as u32;

            if bytes_read < buf_len as usize {
                break;
            }
        }

        let nread_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&nread_memarg, nread_ptr as i32, total_read)
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn fd_datasync(&self, fd: Fd) -> WasiResult<i32> {
        // TODO: Implement fd_datasync
        // For now, return not implemented
        Err(WasiError::NotImplemented)
    }

    pub fn fd_fdstat_set_flags(&self, fd: Fd, flags: u32) -> WasiResult<i32> {
        // TODO: Implement fd_fdstat_set_flags
        // For now, return not implemented
        Err(WasiError::NotImplemented)
    }

    pub fn fd_filestat_set_size(&self, fd: Fd, size: u64) -> WasiResult<i32> {
        // TODO: Implement fd_filestat_set_size (truncate)
        // For now, return not implemented
        Err(WasiError::NotImplemented)
    }

    pub fn fd_pwrite(
        &self,
        memory: &MemAddr,
        fd: Fd,
        iovs_ptr: Ptr,
        iovs_len: Size,
        offset: u64,
        nwritten_ptr: Ptr,
    ) -> WasiResult<i32> {
        if fd <= 2 {
            return Err(WasiError::BadFileDescriptor);
        }

        if self.preopen_dirs.contains_key(&fd) {
            return Err(WasiError::BadFileDescriptor);
        }

        // Get the opened file
        let opened_files = self.opened_files.lock().unwrap();
        let open_file = opened_files.get(&fd).ok_or(WasiError::BadFileDescriptor)?;

        if open_file.is_directory {
            return Err(WasiError::BadFileDescriptor);
        }

        let file = open_file
            .file
            .as_ref()
            .ok_or(WasiError::BadFileDescriptor)?;

        let mut file_clone = file.try_clone().map_err(|_| WasiError::IoError)?;

        file_clone
            .seek(SeekFrom::Start(offset))
            .map_err(|_| WasiError::IoError)?;

        let mut total_written = 0u32;

        for i in 0..iovs_len {
            let iovec_offset = iovs_ptr + (i * 8);

            let buf_ptr_memarg = Memarg {
                offset: 0,
                align: 4,
            };
            let buf_ptr: u32 = memory
                .load(&buf_ptr_memarg, iovec_offset as i32)
                .map_err(|_| WasiError::MemoryAccessError)?;

            // Read buf_len (next 4 bytes of iovec)
            let buf_len_memarg = Memarg {
                offset: 0,
                align: 4,
            };
            let buf_len: u32 = memory
                .load(&buf_len_memarg, (iovec_offset + 4) as i32)
                .map_err(|_| WasiError::MemoryAccessError)?;

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
                    .map_err(|_| WasiError::MemoryAccessError)?;
                data.push(byte);
            }

            // Write data to file at current position
            let bytes_written = file_clone.write(&data).map_err(|_| WasiError::IoError)?;

            total_written += bytes_written as u32;

            // If we couldn't write all data, stop
            if bytes_written < data.len() {
                break;
            }
        }
        file_clone.flush().map_err(|_| WasiError::IoError)?;

        let nwritten_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&nwritten_memarg, nwritten_ptr as i32, total_written)
            .map_err(|_| WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn path_create_directory(
        &self,
        memory: &MemAddr,
        fd: Fd,
        path_ptr: Ptr,
        path_len: Size,
    ) -> WasiResult<i32> {
        // TODO: Implement path_create_directory
        // For now, return not implemented
        Err(WasiError::NotImplemented)
    }

    pub fn path_filestat_get(
        &self,
        memory: &MemAddr,
        fd: Fd,
        flags: u32,
        path_ptr: Ptr,
        path_len: Size,
        filestat_ptr: Ptr,
    ) -> WasiResult<i32> {
        // TODO: Implement path_filestat_get
        // For now, return not implemented
        Err(WasiError::NotImplemented)
    }

    pub fn path_filestat_set_times(
        &self,
        memory: &MemAddr,
        fd: Fd,
        flags: u32,
        path_ptr: Ptr,
        path_len: Size,
        atim: u64,
        mtim: u64,
        fst_flags: u32,
    ) -> WasiResult<i32> {
        // TODO: Implement path_filestat_set_times
        // For now, return not implemented
        Err(WasiError::NotImplemented)
    }

    pub fn path_readlink(
        &self,
        memory: &MemAddr,
        fd: Fd,
        path_ptr: Ptr,
        path_len: Size,
        buf_ptr: Ptr,
        buf_len: Size,
        buf_used_ptr: Ptr,
    ) -> WasiResult<i32> {
        // TODO: Implement path_readlink
        // For now, return not implemented
        Err(WasiError::NotImplemented)
    }

    pub fn path_remove_directory(
        &self,
        memory: &MemAddr,
        fd: Fd,
        path_ptr: Ptr,
        path_len: Size,
    ) -> WasiResult<i32> {
        // TODO: Implement path_remove_directory
        // For now, return not implemented
        Err(WasiError::NotImplemented)
    }

    pub fn path_unlink_file(
        &self,
        memory: &MemAddr,
        fd: Fd,
        path_ptr: Ptr,
        path_len: Size,
    ) -> WasiResult<i32> {
        // TODO: Implement path_unlink_file
        // For now, return not implemented
        Err(WasiError::NotImplemented)
    }

    pub fn poll_oneoff(
        &self,
        memory: &MemAddr,
        in_ptr: Ptr,
        out_ptr: Ptr,
        nsubscriptions: Size,
        nevents_ptr: Ptr,
    ) -> WasiResult<i32> {
        // TODO: Implement poll_oneoff
        // For now, return not implemented
        Err(WasiError::NotImplemented)
    }
}
