use super::*;
use crate::execution::mem::MemAddr;
use crate::structure::instructions::Memarg;

/// WASI iovec structure that matches wasi-libc layout
#[repr(C)]
struct WasiIovec {
    buf: *const u8,
    buf_len: u32,
}

// External links to wasi-libc functions
extern "C" {
    fn __wasi_fd_write(fd: u32, iovs: *const WasiIovec, iovs_len: u32, nwritten: *mut u32) -> u16;
    fn __wasi_args_sizes_get(argc: *mut u32, argv_buf_size: *mut u32) -> u16;
    fn __wasi_args_get(argv: *mut *mut u8, argv_buf: *mut u8) -> u16;
    fn __wasi_fd_read(fd: u32, iovs: *const WasiIovec, iovs_len: u32, nread: *mut u32) -> u16;
    fn __wasi_proc_exit(exit_code: u32) -> !;
    fn __wasi_random_get(buf: *mut u8, buf_len: u32) -> u16;
    fn __wasi_environ_sizes_get(environ_count: *mut u32, environ_buf_size: *mut u32) -> u16;
    fn __wasi_environ_get(environ: *mut *mut u8, environ_buf: *mut u8) -> u16;
    fn __wasi_clock_time_get(clock_id: u32, precision: u64, time: *mut u64) -> u16;
    fn __wasi_clock_res_get(clock_id: u32, resolution: *mut u64) -> u16;
    fn __wasi_sched_yield() -> u16;
    fn __wasi_fd_close(fd: u32) -> u16;
}

/// Passthrough WASI implementation that delegates to host runtime via wasi-libc
pub struct PassthroughWasiImpl {
    argv: Vec<String>,
}

impl PassthroughWasiImpl {
    pub fn new(argv: Vec<String>) -> Self {
        PassthroughWasiImpl { argv }
    }

    pub fn fd_write(
        &self,
        memory: &MemAddr,
        fd: Fd,
        iovs_ptr: Ptr,
        iovs_len: Size,
        nwritten_ptr: Ptr,
    ) -> WasiResult<i32> {
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();
        let memory_len = memory_guard.data.len();

        let mut iovecs = Vec::with_capacity(iovs_len as usize);

        for i in 0..iovs_len {
            // Each iovec is 8 bytes: buf_ptr (4 bytes) + buf_len (4 bytes)
            let iovec_offset = iovs_ptr as usize + (i as usize * 8);

            if iovec_offset + 8 > memory_len {
                return Err(super::error::WasiError::MemoryAccessError);
            }

            let buf_ptr = u32::from_le_bytes([
                memory_guard.data[iovec_offset],
                memory_guard.data[iovec_offset + 1],
                memory_guard.data[iovec_offset + 2],
                memory_guard.data[iovec_offset + 3],
            ]);

            let buf_len = u32::from_le_bytes([
                memory_guard.data[iovec_offset + 4],
                memory_guard.data[iovec_offset + 5],
                memory_guard.data[iovec_offset + 6],
                memory_guard.data[iovec_offset + 7],
            ]);

            if buf_len == 0 {
                iovecs.push(WasiIovec {
                    buf: std::ptr::null(),
                    buf_len: 0,
                });
                continue;
            }

            if buf_ptr as usize + buf_len as usize > memory_len {
                return Err(super::error::WasiError::MemoryAccessError);
            }

            iovecs.push(WasiIovec {
                buf: unsafe { memory_base.add(buf_ptr as usize) },
                buf_len,
            });
        }

        // Call wasi-libc fd_write function
        let mut nwritten: u32 = 0;
        let wasi_errno = unsafe {
            __wasi_fd_write(
                fd as u32,
                iovecs.as_ptr(),
                iovs_len,
                &mut nwritten as *mut u32,
            )
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                28 => Err(super::error::WasiError::IoError),          // ENOSPC -> IoError
                _ => Err(super::error::WasiError::IoError),
            };
        }

        let nwritten_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&nwritten_memarg, nwritten_ptr as i32, nwritten)
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
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let mut iovecs = Vec::with_capacity(iovs_len as usize);

        for i in 0..iovs_len {
            let iovec_offset = iovs_ptr as usize + (i as usize * 8);

            let buf_ptr = u32::from_le_bytes([
                memory_guard.data[iovec_offset],
                memory_guard.data[iovec_offset + 1],
                memory_guard.data[iovec_offset + 2],
                memory_guard.data[iovec_offset + 3],
            ]);

            let buf_len = u32::from_le_bytes([
                memory_guard.data[iovec_offset + 4],
                memory_guard.data[iovec_offset + 5],
                memory_guard.data[iovec_offset + 6],
                memory_guard.data[iovec_offset + 7],
            ]);

            iovecs.push(WasiIovec {
                buf: unsafe { memory_base.add(buf_ptr as usize) as *mut u8 },
                buf_len,
            });
        }

        let mut nread: u32 = 0;
        let wasi_errno =
            unsafe { __wasi_fd_read(fd as u32, iovecs.as_ptr(), iovs_len, &mut nread as *mut u32) };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        let nread_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&nread_memarg, nread_ptr as i32, nread)
            .map_err(|_| super::error::WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn proc_exit(&self, exit_code: ExitCode) -> WasiResult<i32> {
        unsafe {
            __wasi_proc_exit(exit_code as u32);
        }
        // This function never returns
    }

    pub fn random_get(&self, memory: &MemAddr, buf_ptr: Ptr, buf_len: Size) -> WasiResult<i32> {
        if buf_len == 0 {
            return Ok(0);
        }

        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno =
            unsafe { __wasi_random_get(memory_base.add(buf_ptr as usize) as *mut u8, buf_len) };

        drop(memory_guard);

        if wasi_errno != 0 {
            return Err(super::error::WasiError::IoError);
        }

        Ok(0)
    }

    pub fn fd_close(&self, fd: Fd) -> WasiResult<i32> {
        let wasi_errno = unsafe { __wasi_fd_close(fd as u32) };

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
    }

    pub fn environ_get(
        &self,
        memory: &MemAddr,
        environ_ptr: Ptr,
        environ_buf_ptr: Ptr,
    ) -> WasiResult<i32> {
        let mut environ_count: u32 = 0;
        let mut environ_buf_size: u32 = 0;

        let wasi_errno =
            unsafe { __wasi_environ_sizes_get(&mut environ_count, &mut environ_buf_size) };

        if wasi_errno != 0 {
            return match wasi_errno {
                22 => Err(super::error::WasiError::InvalidArgument), // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        let mut environ_buf = vec![0u8; environ_buf_size as usize];
        let mut environ_ptrs = vec![std::ptr::null_mut::<u8>(); environ_count as usize];

        // Call wasi-libc environ_get function
        let wasi_errno =
            unsafe { __wasi_environ_get(environ_ptrs.as_mut_ptr(), environ_buf.as_mut_ptr()) };

        if wasi_errno != 0 {
            return match wasi_errno {
                22 => Err(super::error::WasiError::InvalidArgument), // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        // Calculate pointer offsets relative to environ_buf_ptr
        let mut ptr_data = Vec::with_capacity((environ_count as usize + 1) * 4);
        for i in 0..environ_count as usize {
            if !environ_ptrs[i].is_null() {
                // Calculate offset from the start of environ_buf
                let offset = unsafe { environ_ptrs[i].offset_from(environ_buf.as_ptr()) };
                let string_addr = environ_buf_ptr.wrapping_add(offset as u32);
                ptr_data.extend_from_slice(&string_addr.to_le_bytes());
            } else {
                ptr_data.extend_from_slice(&0u32.to_le_bytes());
            }
        }
        // Null terminator for environ array
        ptr_data.extend_from_slice(&0u32.to_le_bytes());

        // Write pointer array to WebAssembly memory
        memory
            .store_bytes(environ_ptr as i32, &ptr_data)
            .map_err(|_| super::error::WasiError::MemoryAccessError)?;

        // Write environment strings to WebAssembly memory
        memory
            .store_bytes(environ_buf_ptr as i32, &environ_buf)
            .map_err(|_| super::error::WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn environ_sizes_get(
        &self,
        memory: &MemAddr,
        environ_count_ptr: Ptr,
        environ_buf_size_ptr: Ptr,
    ) -> WasiResult<i32> {
        let mut environ_count: u32 = 0;
        let mut environ_buf_size: u32 = 0;

        let wasi_errno =
            unsafe { __wasi_environ_sizes_get(&mut environ_count, &mut environ_buf_size) };

        if wasi_errno != 0 {
            return match wasi_errno {
                22 => Err(super::error::WasiError::InvalidArgument), // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        // Write environment variable count
        let count_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&count_memarg, environ_count_ptr as i32, environ_count)
            .map_err(|_| super::error::WasiError::MemoryAccessError)?;

        // Write total buffer size needed
        let size_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&size_memarg, environ_buf_size_ptr as i32, environ_buf_size)
            .map_err(|_| super::error::WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn args_get(&self, memory: &MemAddr, argv_ptr: Ptr, argv_buf_ptr: Ptr) -> WasiResult<i32> {
        let args = &self.argv;

        // Calculate total buffer size needed for all argument strings (including null terminators)
        let total_len: usize = args.iter().map(|arg| arg.len() + 1).sum();

        // Build the argument buffer and pointer array
        let mut argv_buf = Vec::with_capacity(total_len);
        let mut ptr_data = Vec::with_capacity((args.len() + 1) * 4); // +1 for null terminator

        for arg in args {
            // Store pointer to current position in buffer (relative to argv_buf_ptr)
            let string_addr = argv_buf_ptr + argv_buf.len() as u32;
            ptr_data.extend_from_slice(&string_addr.to_le_bytes());

            // Add the string to the buffer
            argv_buf.extend_from_slice(arg.as_bytes());
            argv_buf.push(0); // null terminator
        }

        // Add null terminator for the argv array
        ptr_data.extend_from_slice(&0u32.to_le_bytes());

        // Write pointer array to WebAssembly memory
        memory
            .store_bytes(argv_ptr as i32, &ptr_data)
            .map_err(|_| super::error::WasiError::MemoryAccessError)?;

        // Write argument strings to WebAssembly memory
        memory
            .store_bytes(argv_buf_ptr as i32, &argv_buf)
            .map_err(|_| super::error::WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn args_sizes_get(
        &self,
        memory: &MemAddr,
        argc_ptr: Ptr,
        argv_buf_size_ptr: Ptr,
    ) -> WasiResult<i32> {
        let args = &self.argv;

        // Calculate argument count
        let argc = args.len() as u32;

        // Calculate total buffer size needed (sum of string lengths + null terminators)
        let argv_buf_size: u32 = args.iter().map(|arg| arg.len() + 1).sum::<usize>() as u32;

        // Write argument count to WebAssembly memory
        let argc_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(&argc_memarg, argc_ptr as i32, argc)
            .map_err(|_| super::error::WasiError::MemoryAccessError)?;

        // Write total buffer size needed to WebAssembly memory
        let argv_buf_size_memarg = Memarg {
            offset: 0,
            align: 4,
        };
        memory
            .store(
                &argv_buf_size_memarg,
                argv_buf_size_ptr as i32,
                argv_buf_size,
            )
            .map_err(|_| super::error::WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn clock_time_get(
        &self,
        memory: &MemAddr,
        clock_id: i32,
        precision: i64,
        time_ptr: Ptr,
    ) -> WasiResult<i32> {
        let mut time: u64 = 0;

        let wasi_errno =
            unsafe { __wasi_clock_time_get(clock_id as u32, precision as u64, &mut time) };

        if wasi_errno != 0 {
            return match wasi_errno {
                22 => Err(super::error::WasiError::InvalidArgument), // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        // Write timestamp (64-bit nanoseconds) to memory using store_bytes
        memory
            .store_bytes(time_ptr as i32, &time.to_le_bytes())
            .map_err(|_| super::error::WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn clock_res_get(
        &self,
        memory: &MemAddr,
        clock_id: i32,
        resolution_ptr: Ptr,
    ) -> WasiResult<i32> {
        let mut resolution: u64 = 0;

        let wasi_errno = unsafe { __wasi_clock_res_get(clock_id as u32, &mut resolution) };

        if wasi_errno != 0 {
            return match wasi_errno {
                22 => Err(super::error::WasiError::InvalidArgument), // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        // Write resolution (64-bit nanoseconds) to memory using store_bytes
        memory
            .store_bytes(resolution_ptr as i32, &resolution.to_le_bytes())
            .map_err(|_| super::error::WasiError::MemoryAccessError)?;

        Ok(0)
    }

    pub fn fd_prestat_get(&self, _memory: &MemAddr, _fd: Fd, _prestat_ptr: Ptr) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn fd_prestat_dir_name(
        &self,
        _memory: &MemAddr,
        _fd: Fd,
        _path_ptr: Ptr,
        _path_len: Size,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn sched_yield(&self) -> WasiResult<i32> {
        let wasi_errno = unsafe { __wasi_sched_yield() };

        if wasi_errno != 0 {
            return Err(super::error::WasiError::IoError);
        }

        Ok(0)
    }

    pub fn fd_fdstat_get(&self, _memory: &MemAddr, _fd: Fd, _stat_ptr: Ptr) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn path_open(
        &self,
        _memory: &MemAddr,
        _fd: Fd,
        _dirflags: u32,
        _path_ptr: Ptr,
        _path_len: Size,
        _oflags: u32,
        _fs_rights_base: u64,
        _fs_rights_inheriting: u64,
        _fdflags: u32,
        _opened_fd_ptr: Ptr,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn fd_seek(&self, _fd: Fd, _offset: i64, _whence: u32) -> WasiResult<u64> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn fd_tell(&self, _memory: &MemAddr, _fd: Fd, _offset_ptr: Ptr) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn fd_sync(&self, _fd: Fd) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn fd_filestat_get(
        &self,
        _memory: &MemAddr,
        _fd: Fd,
        _filestat_ptr: Ptr,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn fd_readdir(
        &self,
        _memory: &MemAddr,
        _fd: Fd,
        _buf_ptr: Ptr,
        _buf_len: Size,
        _cookie: u64,
        _buf_used_ptr: Ptr,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn fd_pread(
        &self,
        _memory: &MemAddr,
        _fd: Fd,
        _iovs_ptr: Ptr,
        _iovs_len: Size,
        _offset: u64,
        _nread_ptr: Ptr,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn fd_datasync(&self, _fd: Fd) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn fd_fdstat_set_flags(&self, _fd: Fd, _flags: u32) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn fd_filestat_set_size(&self, _fd: Fd, _size: u64) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn fd_pwrite(
        &self,
        _memory: &MemAddr,
        _fd: Fd,
        _iovs_ptr: Ptr,
        _iovs_len: Size,
        _offset: u64,
        _nwritten_ptr: Ptr,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn path_create_directory(
        &self,
        _memory: &MemAddr,
        _fd: Fd,
        _path_ptr: Ptr,
        _path_len: Size,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn path_filestat_get(
        &self,
        _memory: &MemAddr,
        _fd: Fd,
        _flags: u32,
        _path_ptr: Ptr,
        _path_len: Size,
        _filestat_ptr: Ptr,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn path_filestat_set_times(
        &self,
        _memory: &MemAddr,
        _fd: Fd,
        _flags: u32,
        _path_ptr: Ptr,
        _path_len: Size,
        _atim: u64,
        _mtim: u64,
        _fst_flags: u32,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn path_readlink(
        &self,
        _memory: &MemAddr,
        _fd: Fd,
        _path_ptr: Ptr,
        _path_len: Size,
        _buf_ptr: Ptr,
        _buf_len: Size,
        _buf_used_ptr: Ptr,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn path_remove_directory(
        &self,
        _memory: &MemAddr,
        _fd: Fd,
        _path_ptr: Ptr,
        _path_len: Size,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn path_unlink_file(
        &self,
        _memory: &MemAddr,
        _fd: Fd,
        _path_ptr: Ptr,
        _path_len: Size,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn poll_oneoff(
        &self,
        _memory: &MemAddr,
        _in_ptr: Ptr,
        _out_ptr: Ptr,
        _nsubscriptions: Size,
        _nevents_ptr: Ptr,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }
}
