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
    fn __wasi_fd_sync(fd: u32) -> u16;
    fn __wasi_fd_datasync(fd: u32) -> u16;
    fn __wasi_fd_prestat_get(fd: u32, prestat: *mut u8) -> u16;
    fn __wasi_fd_prestat_dir_name(fd: u32, path: *mut u8, path_len: u32) -> u16;
    fn __wasi_fd_fdstat_get(fd: u32, stat: *mut u8) -> u16;
    fn __wasi_fd_seek(fd: u32, offset: i64, whence: u32, newoffset: *mut u64) -> u16;
    fn __wasi_fd_tell(fd: u32, offset: *mut u64) -> u16;
    fn __wasi_fd_fdstat_set_flags(fd: u32, flags: u32) -> u16;
    fn __wasi_fd_filestat_set_size(fd: u32, size: u64) -> u16;
    fn __wasi_fd_filestat_get(fd: u32, filestat: *mut u8) -> u16;
    fn __wasi_path_create_directory(fd: u32, path: *const u8, path_len: u32) -> u16;
    fn __wasi_path_remove_directory(fd: u32, path: *const u8, path_len: u32) -> u16;
    fn __wasi_path_unlink_file(fd: u32, path: *const u8, path_len: u32) -> u16;
    fn __wasi_path_readlink(
        fd: u32,
        path: *const u8,
        path_len: u32,
        buf: *mut u8,
        buf_len: u32,
        buf_used: *mut u32,
    ) -> u16;
    fn __wasi_path_filestat_get(
        fd: u32,
        flags: u32,
        path: *const u8,
        path_len: u32,
        filestat: *mut u8,
    ) -> u16;
    fn __wasi_path_filestat_set_times(
        fd: u32,
        flags: u32,
        path: *const u8,
        path_len: u32,
        atim: u64,
        mtim: u64,
        fst_flags: u32,
    ) -> u16;
    fn __wasi_path_open(
        fd: u32,
        dirflags: u32,
        path: *const u8,
        path_len: u32,
        oflags: u32,
        fs_rights_base: u64,
        fs_rights_inheriting: u64,
        fdflags: u32,
        opened_fd: *mut u32,
    ) -> u16;
    fn __wasi_poll_oneoff(
        in_ptr: *const u8,
        out_ptr: *mut u8,
        nsubscriptions: u32,
        nevents: *mut u32,
    ) -> u16;
    fn __wasi_fd_readdir(
        fd: u32,
        buf: *mut u8,
        buf_len: u32,
        cookie: u64,
        buf_used: *mut u32,
    ) -> u16;
    fn __wasi_fd_pread(
        fd: u32,
        iovs: *const WasiIovec,
        iovs_len: u32,
        offset: u64,
        nread: *mut u32,
    ) -> u16;
    fn __wasi_fd_pwrite(
        fd: u32,
        iovs: *const WasiIovec,
        iovs_len: u32,
        offset: u64,
        nwritten: *mut u32,
    ) -> u16;
    fn __wasi_proc_raise(signal: u32) -> u16;
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

            if buf_len == 0 {
                iovecs.push(WasiIovec {
                    buf: std::ptr::null(),
                    buf_len: 0,
                });
            } else {
                iovecs.push(WasiIovec {
                    buf: unsafe { memory_base.add(buf_ptr as usize) },
                    buf_len,
                });
            }
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

    pub fn fd_prestat_get(&self, memory: &MemAddr, fd: Fd, prestat_ptr: Ptr) -> WasiResult<i32> {
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno = unsafe {
            __wasi_fd_prestat_get(fd as u32, memory_base.add(prestat_ptr as usize) as *mut u8)
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
    }

    pub fn fd_prestat_dir_name(
        &self,
        memory: &MemAddr,
        fd: Fd,
        path_ptr: Ptr,
        path_len: Size,
    ) -> WasiResult<i32> {
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno = unsafe {
            __wasi_fd_prestat_dir_name(
                fd as u32,
                memory_base.add(path_ptr as usize) as *mut u8,
                path_len,
            )
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
    }

    pub fn sched_yield(&self) -> WasiResult<i32> {
        let wasi_errno = unsafe { __wasi_sched_yield() };

        if wasi_errno != 0 {
            return Err(super::error::WasiError::IoError);
        }

        Ok(0)
    }

    pub fn fd_fdstat_get(&self, memory: &MemAddr, fd: Fd, stat_ptr: Ptr) -> WasiResult<i32> {
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno = unsafe {
            __wasi_fd_fdstat_get(fd as u32, memory_base.add(stat_ptr as usize) as *mut u8)
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

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
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno = unsafe {
            __wasi_path_open(
                fd as u32,
                dirflags,
                memory_base.add(path_ptr as usize),
                path_len,
                oflags,
                fs_rights_base,
                fs_rights_inheriting,
                fdflags,
                memory_base.add(opened_fd_ptr as usize) as *mut u32,
            )
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                2 => Err(super::error::WasiError::IoError),           // ENOENT
                13 => Err(super::error::WasiError::IoError),          // EACCES
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
    }

    pub fn fd_seek(&self, fd: Fd, offset: i64, whence: u32) -> WasiResult<u64> {
        let mut newoffset: u64 = 0;

        let wasi_errno =
            unsafe { __wasi_fd_seek(fd as u32, offset, whence, &mut newoffset as *mut u64) };

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(newoffset)
    }

    pub fn fd_tell(&self, memory: &MemAddr, fd: Fd, offset_ptr: Ptr) -> WasiResult<i32> {
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno =
            unsafe { __wasi_fd_tell(fd as u32, memory_base.add(offset_ptr as usize) as *mut u64) };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
    }

    pub fn fd_sync(&self, fd: Fd) -> WasiResult<i32> {
        let wasi_errno = unsafe { __wasi_fd_sync(fd as u32) };

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
    }

    pub fn fd_filestat_get(&self, memory: &MemAddr, fd: Fd, filestat_ptr: Ptr) -> WasiResult<i32> {
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno = unsafe {
            __wasi_fd_filestat_get(fd as u32, memory_base.add(filestat_ptr as usize) as *mut u8)
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
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
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno = unsafe {
            __wasi_fd_readdir(
                fd as u32,
                memory_base.add(buf_ptr as usize) as *mut u8,
                buf_len,
                cookie,
                memory_base.add(buf_used_ptr as usize) as *mut u32,
            )
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                28 => Err(super::error::WasiError::IoError),          // ENOSPC
                _ => Err(super::error::WasiError::IoError),
            };
        }

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
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let mut iovecs = Vec::with_capacity(iovs_len as usize);

        for i in 0..iovs_len {
            // Each iovec is 8 bytes: buf_ptr (4 bytes) + buf_len (4 bytes)
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

            if buf_len == 0 {
                iovecs.push(WasiIovec {
                    buf: std::ptr::null(),
                    buf_len: 0,
                });
            } else {
                iovecs.push(WasiIovec {
                    buf: unsafe { memory_base.add(buf_ptr as usize) },
                    buf_len,
                });
            }
        }

        let mut nread: u32 = 0;
        let wasi_errno = unsafe {
            __wasi_fd_pread(
                fd as u32,
                iovecs.as_ptr(),
                iovs_len,
                offset,
                &mut nread as *mut u32,
            )
        };

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

    pub fn fd_datasync(&self, fd: Fd) -> WasiResult<i32> {
        let wasi_errno = unsafe { __wasi_fd_datasync(fd as u32) };

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
    }

    pub fn fd_fdstat_set_flags(&self, fd: Fd, flags: u32) -> WasiResult<i32> {
        let wasi_errno = unsafe { __wasi_fd_fdstat_set_flags(fd as u32, flags) };

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
    }

    pub fn fd_filestat_set_size(&self, fd: Fd, size: u64) -> WasiResult<i32> {
        let wasi_errno = unsafe { __wasi_fd_filestat_set_size(fd as u32, size) };

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
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
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let mut iovecs = Vec::with_capacity(iovs_len as usize);

        for i in 0..iovs_len {
            // Each iovec is 8 bytes: buf_ptr (4 bytes) + buf_len (4 bytes)
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

            if buf_len == 0 {
                iovecs.push(WasiIovec {
                    buf: std::ptr::null(),
                    buf_len: 0,
                });
            } else {
                iovecs.push(WasiIovec {
                    buf: unsafe { memory_base.add(buf_ptr as usize) },
                    buf_len,
                });
            }
        }

        let mut nwritten: u32 = 0;
        let wasi_errno = unsafe {
            __wasi_fd_pwrite(
                fd as u32,
                iovecs.as_ptr(),
                iovs_len,
                offset,
                &mut nwritten as *mut u32,
            )
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                28 => Err(super::error::WasiError::IoError),          // ENOSPC
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

    pub fn path_create_directory(
        &self,
        memory: &MemAddr,
        fd: Fd,
        path_ptr: Ptr,
        path_len: Size,
    ) -> WasiResult<i32> {
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno = unsafe {
            __wasi_path_create_directory(fd as u32, memory_base.add(path_ptr as usize), path_len)
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
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
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno = unsafe {
            __wasi_path_filestat_get(
                fd as u32,
                flags,
                memory_base.add(path_ptr as usize),
                path_len,
                memory_base.add(filestat_ptr as usize) as *mut u8,
            )
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
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
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno = unsafe {
            __wasi_path_filestat_set_times(
                fd as u32,
                flags,
                memory_base.add(path_ptr as usize),
                path_len,
                atim,
                mtim,
                fst_flags,
            )
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
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
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno = unsafe {
            __wasi_path_readlink(
                fd as u32,
                memory_base.add(path_ptr as usize),
                path_len,
                memory_base.add(buf_ptr as usize) as *mut u8,
                buf_len,
                memory_base.add(buf_used_ptr as usize) as *mut u32,
            )
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
    }

    pub fn path_remove_directory(
        &self,
        memory: &MemAddr,
        fd: Fd,
        path_ptr: Ptr,
        path_len: Size,
    ) -> WasiResult<i32> {
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno = unsafe {
            __wasi_path_remove_directory(fd as u32, memory_base.add(path_ptr as usize), path_len)
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
    }

    pub fn path_unlink_file(
        &self,
        memory: &MemAddr,
        fd: Fd,
        path_ptr: Ptr,
        path_len: Size,
    ) -> WasiResult<i32> {
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno = unsafe {
            __wasi_path_unlink_file(fd as u32, memory_base.add(path_ptr as usize), path_len)
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                8 => Err(super::error::WasiError::BadFileDescriptor), // EBADF
                22 => Err(super::error::WasiError::InvalidArgument),  // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
    }

    pub fn poll_oneoff(
        &self,
        memory: &MemAddr,
        in_ptr: Ptr,
        out_ptr: Ptr,
        nsubscriptions: Size,
        nevents_ptr: Ptr,
    ) -> WasiResult<i32> {
        let memory_guard = memory.get_memory_direct_access();
        let memory_base = memory_guard.data.as_ptr();

        let wasi_errno = unsafe {
            __wasi_poll_oneoff(
                memory_base.add(in_ptr as usize),
                memory_base.add(out_ptr as usize) as *mut u8,
                nsubscriptions,
                memory_base.add(nevents_ptr as usize) as *mut u32,
            )
        };

        drop(memory_guard);

        if wasi_errno != 0 {
            return match wasi_errno {
                22 => Err(super::error::WasiError::InvalidArgument), // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
    }

    pub fn proc_raise(&self, _memory: &MemAddr, signal: u32) -> WasiResult<i32> {
        let wasi_errno = unsafe { __wasi_proc_raise(signal) };

        if wasi_errno != 0 {
            return match wasi_errno {
                22 => Err(super::error::WasiError::InvalidArgument), // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        Ok(0)
    }
}
