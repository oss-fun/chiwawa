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
    fn __wasi_fd_write(fd: u32, iovs: *const WasiIovec, iovs_len: u32, nwritten: *mut u32) -> u16; // WASI errno_t

    fn __wasi_args_sizes_get(argc: *mut u32, argv_buf_size: *mut u32) -> u16; // WASI errno_t

    fn __wasi_args_get(argv: *mut *mut u8, argv_buf: *mut u8) -> u16; // WASI errno_t
}

/// Passthrough WASI implementation that delegates to host runtime via wasi-libc
pub struct PassthroughWasiImpl;

impl PassthroughWasiImpl {
    pub fn new() -> Self {
        PassthroughWasiImpl
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
        _memory: &MemAddr,
        _fd: Fd,
        _iovs_ptr: Ptr,
        _iovs_len: Size,
        _nread_ptr: Ptr,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn proc_exit(&self, _exit_code: ExitCode) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn random_get(&self, _memory: &MemAddr, _buf_ptr: Ptr, _buf_len: Size) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn fd_close(&self, _fd: Fd) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn environ_get(
        &self,
        _memory: &MemAddr,
        _environ_ptr: Ptr,
        _environ_buf_ptr: Ptr,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn environ_sizes_get(
        &self,
        _memory: &MemAddr,
        _environ_count_ptr: Ptr,
        _environ_buf_size_ptr: Ptr,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn args_get(&self, memory: &MemAddr, argv_ptr: Ptr, argv_buf_ptr: Ptr) -> WasiResult<i32> {
        // First, get the argument count and buffer size using args_sizes_get
        let mut argc: u32 = 0;
        let mut argv_buf_size: u32 = 0;

        let wasi_errno =
            unsafe { __wasi_args_sizes_get(&mut argc as *mut u32, &mut argv_buf_size as *mut u32) };

        if wasi_errno != 0 {
            return match wasi_errno {
                22 => Err(super::error::WasiError::InvalidArgument), // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        // Allocate buffers for the arguments
        let mut argv_buf = vec![0u8; argv_buf_size as usize];
        let mut argv_ptrs = vec![std::ptr::null_mut::<u8>(); argc as usize];

        // Call wasi-libc args_get function
        let wasi_errno = unsafe { __wasi_args_get(argv_ptrs.as_mut_ptr(), argv_buf.as_mut_ptr()) };

        if wasi_errno != 0 {
            return match wasi_errno {
                22 => Err(super::error::WasiError::InvalidArgument), // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

        // Calculate pointer offsets relative to argv_buf_ptr
        let mut ptr_data = Vec::with_capacity((argc as usize + 1) * 4);
        for i in 0..argc as usize {
            if !argv_ptrs[i].is_null() {
                // Calculate offset from the start of argv_buf
                let offset = unsafe { argv_ptrs[i].offset_from(argv_buf.as_ptr()) };
                let string_addr = argv_buf_ptr.wrapping_add(offset as u32);
                ptr_data.extend_from_slice(&string_addr.to_le_bytes());
            } else {
                ptr_data.extend_from_slice(&0u32.to_le_bytes());
            }
        }
        // Null terminator for argv array
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
        // Call wasi-libc args_sizes_get function
        let mut argc: u32 = 0;
        let mut argv_buf_size: u32 = 0;

        let wasi_errno =
            unsafe { __wasi_args_sizes_get(&mut argc as *mut u32, &mut argv_buf_size as *mut u32) };

        // Convert WASI errno to our error type
        if wasi_errno != 0 {
            return match wasi_errno {
                22 => Err(super::error::WasiError::InvalidArgument), // EINVAL
                _ => Err(super::error::WasiError::IoError),
            };
        }

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
        _memory: &MemAddr,
        _clock_id: i32,
        _precision: i64,
        _time_ptr: Ptr,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
    }

    pub fn clock_res_get(
        &self,
        _memory: &MemAddr,
        _clock_id: i32,
        _resolution_ptr: Ptr,
    ) -> WasiResult<i32> {
        Err(super::error::WasiError::NotImplemented)
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
        Err(super::error::WasiError::NotImplemented)
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
