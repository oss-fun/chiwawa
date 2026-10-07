//! Passthrough WASI implementation delegating to host wasi-libc.
//!
//! This module implements WASI Preview 1 functions by calling the corresponding
//! `__wasi_*` functions from the host's wasi-libc. This avoids duplicating
//! WASI implementation logic and ensures correct behavior on any WASI-compliant host.
//!
//! Each public method on [`PassthroughWasiImpl`] corresponds to a WASI function
//! and translates between guest memory addresses and host pointers.

use super::*;
use crate::execution::mem::{MemAddr, MemInst};
use WasiError;

/// WASI iovec structure that matches wasi-libc layout.
#[repr(C)]
pub(crate) struct WasiIovec {
    buf: *const u8,
    buf_len: u32,
}

/// The guest memory range `len` bytes from `start`, unless it overflows.
pub(crate) fn guest_range(start: usize, len: usize) -> Option<std::ops::Range<usize>> {
    start.checked_add(len).map(|end| start..end)
}

/// Rebuilds a guest iovec array with its buffer pointers translated to host addresses.
pub(crate) fn collect_iovecs(
    mem: &MemInst,
    iovs_ptr: Ptr,
    iovs_len: Size,
) -> WasiResult<Vec<WasiIovec>> {
    let table = (iovs_len as usize)
        .checked_mul(8)
        .and_then(|len| guest_range(iovs_ptr as usize, len))
        .and_then(|r| mem.data.get(r))
        .ok_or(WasiError::Fault)?;

    table
        .chunks_exact(8)
        .map(|entry| {
            let buf_ptr = u32::from_le_bytes(entry[..4].try_into().unwrap()) as usize;
            let buf_len = u32::from_le_bytes(entry[4..].try_into().unwrap());
            if buf_len == 0 {
                return Ok(WasiIovec {
                    buf: std::ptr::null(),
                    buf_len: 0,
                });
            }
            let buf = guest_range(buf_ptr, buf_len as usize)
                .and_then(|r| mem.data.get(r))
                .ok_or(WasiError::Fault)?;
            Ok(WasiIovec {
                buf: buf.as_ptr(),
                buf_len,
            })
        })
        .collect()
}

fn host_ptr<T>(memory: &MemAddr, ptr: Ptr) -> *mut T {
    unsafe { memory.data_ptr().add(ptr as usize) as *mut T }
}

fn nul_terminated(bytes: &[u8]) -> Vec<u8> {
    let mut path = bytes.to_vec();
    path.push(0);
    path
}

fn guest_path(memory: &MemAddr, ptr: Ptr, len: Size) -> Vec<u8> {
    let bytes = unsafe { std::slice::from_raw_parts(host_ptr::<u8>(memory, ptr), len as usize) };
    nul_terminated(bytes)
}

fn store_u32(memory: &MemAddr, ptr: Ptr, value: u32) {
    memory.store_bytes(ptr as i32, &value.to_le_bytes());
}

fn store_u64(memory: &MemAddr, ptr: Ptr, value: u64) {
    memory.store_bytes(ptr as i32, &value.to_le_bytes());
}

// External declarations for wasi-libc functions
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
    fn __wasi_path_create_directory(fd: u32, path: *const u8) -> u16;
    fn __wasi_path_remove_directory(fd: u32, path: *const u8) -> u16;
    fn __wasi_path_unlink_file(fd: u32, path: *const u8) -> u16;
    fn __wasi_path_readlink(
        fd: u32,
        path: *const u8,
        buf: *mut u8,
        buf_len: u32,
        retptr0: *mut u32,
    ) -> u16;
    fn __wasi_path_filestat_get(fd: u32, flags: u32, path: *const u8, filestat: *mut u8) -> u16;
    fn __wasi_path_filestat_set_times(
        fd: u32,
        flags: u32,
        path: *const u8,
        atim: u64,
        mtim: u64,
        fst_flags: u32,
    ) -> u16;
    fn __wasi_path_open(
        fd: u32,
        dirflags: u32,
        path: *const u8,
        oflags: u16,
        fs_rights_base: u64,
        fs_rights_inheriting: u64,
        fdflags: u16,
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
    fn __wasi_fd_advise(fd: u32, offset: u64, len: u64, advice: u8) -> u16;
    fn __wasi_fd_allocate(fd: i32, offset: u64, len: u64) -> u16;
    fn __wasi_fd_fdstat_set_rights(fd: u32, fs_rights_base: u64, fs_rights_inheriting: u64) -> u16;
    fn __wasi_fd_renumber(fd: i32, to: i32) -> u16;
    fn __wasi_fd_filestat_set_times(fd: u32, atim: u64, mtim: u64, fst_flags: u32) -> u16;
    fn __wasi_path_link(
        old_fd: u32,
        old_flags: u32,
        old_path: *const u8,
        new_fd: u32,
        new_path: *const u8,
    ) -> u16;
    fn __wasi_path_rename(
        old_fd: u32,
        old_path: *const u8,
        new_fd: u32,
        new_path: *const u8,
    ) -> u16;
    fn __wasi_path_symlink(old_path: *const u8, fd: u32, new_path: *const u8) -> u16;
    fn __wasi_sock_accept(fd: u32, flags: u32, fd_ptr: *mut u32) -> u16;
    fn __wasi_sock_recv(
        fd: u32,
        ri_data: *const WasiIovec,
        ri_data_len: u32,
        ri_flags: u32,
        ro_datalen: *mut u32,
        ro_flags: *mut u32,
    ) -> u16;
    fn __wasi_sock_send(
        fd: u32,
        si_data: *const WasiIovec,
        si_data_len: u32,
        si_flags: u32,
        so_datalen: *mut u32,
    ) -> u16;
    fn __wasi_sock_shutdown(fd: u32, how: u32) -> u16;
}

/// Passthrough WASI implementation that delegates to host runtime via wasi-libc.
///
/// This struct holds state needed for WASI operations (such as command-line arguments)
/// and provides methods for each WASI Preview 1 function. Each method reads from or
/// writes to guest linear memory and calls the corresponding `__wasi_*` function.
pub struct PassthroughWasiImpl {
    argv: Vec<String>,
}

impl PassthroughWasiImpl {
    pub fn new(argv: Vec<String>) -> Self {
        PassthroughWasiImpl { argv }
    }

    /// Whether `path` exists, by a `path_filestat_get` on the current
    /// directory. Used for checkpoint trigger detection.
    pub fn check_file_exists(&self, path: &str) -> bool {
        let path = nul_terminated(path.as_bytes());
        let mut stat = [0u8; 64];
        let errno = unsafe { __wasi_path_filestat_get(3, 0, path.as_ptr(), stat.as_mut_ptr()) };
        errno == 0
    }

    pub fn fd_write(
        &self,
        memory: &MemAddr,
        fd: Fd,
        iovs_ptr: Ptr,
        iovs_len: Size,
        nwritten_ptr: Ptr,
    ) -> WasiResult<i32> {
        let iovecs = collect_iovecs(memory.get_memory_direct_access(), iovs_ptr, iovs_len)?;
        let mut nwritten: u32 = 0;
        let errno = unsafe { __wasi_fd_write(fd as u32, iovecs.as_ptr(), iovs_len, &mut nwritten) };
        if errno == 0 {
            store_u32(memory, nwritten_ptr, nwritten);
        }
        Ok(errno as i32)
    }

    pub fn fd_read(
        &self,
        memory: &MemAddr,
        fd: Fd,
        iovs_ptr: Ptr,
        iovs_len: Size,
        nread_ptr: Ptr,
    ) -> WasiResult<i32> {
        let iovecs = collect_iovecs(memory.get_memory_direct_access(), iovs_ptr, iovs_len)?;
        let mut nread: u32 = 0;
        let errno = unsafe { __wasi_fd_read(fd as u32, iovecs.as_ptr(), iovs_len, &mut nread) };
        if errno == 0 {
            store_u32(memory, nread_ptr, nread);
        }
        Ok(errno as i32)
    }

    /// Never returns.
    pub fn proc_exit(&self, exit_code: ExitCode) -> WasiResult<i32> {
        unsafe { __wasi_proc_exit(exit_code as u32) }
    }

    pub fn random_get(&self, memory: &MemAddr, buf_ptr: Ptr, buf_len: Size) -> WasiResult<i32> {
        if buf_len == 0 {
            return Ok(0);
        }
        let errno = unsafe { __wasi_random_get(host_ptr(memory, buf_ptr), buf_len) };
        Ok(errno as i32)
    }

    pub fn fd_close(&self, fd: Fd) -> WasiResult<i32> {
        Ok(unsafe { __wasi_fd_close(fd as u32) } as i32)
    }

    pub fn environ_get(
        &self,
        memory: &MemAddr,
        environ_ptr: Ptr,
        environ_buf_ptr: Ptr,
    ) -> WasiResult<i32> {
        let mut environ_count: u32 = 0;
        let mut environ_buf_size: u32 = 0;
        let errno = unsafe { __wasi_environ_sizes_get(&mut environ_count, &mut environ_buf_size) };
        if errno != 0 {
            return Ok(errno as i32);
        }

        let mut environ_buf = vec![0u8; environ_buf_size as usize];
        let mut environ_ptrs = vec![std::ptr::null_mut::<u8>(); environ_count as usize];
        let errno =
            unsafe { __wasi_environ_get(environ_ptrs.as_mut_ptr(), environ_buf.as_mut_ptr()) };
        if errno != 0 {
            return Ok(errno as i32);
        }

        // The pointer array, rebased from the host buffer to the guest's, and
        // NUL-terminated.
        let mut ptr_data = Vec::with_capacity((environ_count as usize + 1) * 4);
        for ptr in &environ_ptrs {
            let string_addr = if ptr.is_null() {
                0
            } else {
                let offset = unsafe { ptr.offset_from(environ_buf.as_ptr()) };
                environ_buf_ptr.wrapping_add(offset as u32)
            };
            ptr_data.extend_from_slice(&string_addr.to_le_bytes());
        }
        ptr_data.extend_from_slice(&0u32.to_le_bytes());

        memory.store_bytes(environ_ptr as i32, &ptr_data);
        memory.store_bytes(environ_buf_ptr as i32, &environ_buf);
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
        let errno = unsafe { __wasi_environ_sizes_get(&mut environ_count, &mut environ_buf_size) };
        if errno != 0 {
            return Ok(errno as i32);
        }
        store_u32(memory, environ_count_ptr, environ_count);
        store_u32(memory, environ_buf_size_ptr, environ_buf_size);
        Ok(0)
    }

    pub fn args_get(&self, memory: &MemAddr, argv_ptr: Ptr, argv_buf_ptr: Ptr) -> WasiResult<i32> {
        let args = &self.argv;
        let total_len: usize = args.iter().map(|arg| arg.len() + 1).sum();

        // The strings, each NUL-terminated, and the pointer array into them,
        // itself NUL-terminated.
        let mut argv_buf = Vec::with_capacity(total_len);
        let mut ptr_data = Vec::with_capacity((args.len() + 1) * 4);
        for arg in args {
            let string_addr = argv_buf_ptr + argv_buf.len() as u32;
            ptr_data.extend_from_slice(&string_addr.to_le_bytes());
            argv_buf.extend_from_slice(arg.as_bytes());
            argv_buf.push(0);
        }
        ptr_data.extend_from_slice(&0u32.to_le_bytes());

        memory.store_bytes(argv_ptr as i32, &ptr_data);
        memory.store_bytes(argv_buf_ptr as i32, &argv_buf);
        Ok(0)
    }

    pub fn args_sizes_get(
        &self,
        memory: &MemAddr,
        argc_ptr: Ptr,
        argv_buf_size_ptr: Ptr,
    ) -> WasiResult<i32> {
        let args = &self.argv;
        let argv_buf_size: u32 = args.iter().map(|arg| arg.len() + 1).sum::<usize>() as u32;
        store_u32(memory, argc_ptr, args.len() as u32);
        store_u32(memory, argv_buf_size_ptr, argv_buf_size);
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
        let errno = unsafe { __wasi_clock_time_get(clock_id as u32, precision as u64, &mut time) };
        if errno == 0 {
            store_u64(memory, time_ptr, time);
        }
        Ok(errno as i32)
    }

    pub fn clock_res_get(
        &self,
        memory: &MemAddr,
        clock_id: i32,
        resolution_ptr: Ptr,
    ) -> WasiResult<i32> {
        let mut resolution: u64 = 0;
        let errno = unsafe { __wasi_clock_res_get(clock_id as u32, &mut resolution) };
        if errno == 0 {
            store_u64(memory, resolution_ptr, resolution);
        }
        Ok(errno as i32)
    }

    pub fn fd_prestat_get(&self, memory: &MemAddr, fd: Fd, prestat_ptr: Ptr) -> WasiResult<i32> {
        let errno = unsafe { __wasi_fd_prestat_get(fd as u32, host_ptr(memory, prestat_ptr)) };
        Ok(errno as i32)
    }

    pub fn fd_prestat_dir_name(
        &self,
        memory: &MemAddr,
        fd: Fd,
        path_ptr: Ptr,
        path_len: Size,
    ) -> WasiResult<i32> {
        let errno =
            unsafe { __wasi_fd_prestat_dir_name(fd as u32, host_ptr(memory, path_ptr), path_len) };
        Ok(errno as i32)
    }

    pub fn sched_yield(&self) -> WasiResult<i32> {
        Ok(unsafe { __wasi_sched_yield() } as i32)
    }

    pub fn fd_fdstat_get(&self, memory: &MemAddr, fd: Fd, stat_ptr: Ptr) -> WasiResult<i32> {
        let errno = unsafe { __wasi_fd_fdstat_get(fd as u32, host_ptr(memory, stat_ptr)) };
        Ok(errno as i32)
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
        let path = guest_path(memory, path_ptr, path_len);
        let errno = unsafe {
            __wasi_path_open(
                fd as u32,
                dirflags,
                path.as_ptr(),
                oflags as u16,
                fs_rights_base,
                fs_rights_inheriting,
                fdflags as u16,
                host_ptr(memory, opened_fd_ptr),
            )
        };
        Ok(errno as i32)
    }

    pub fn fd_seek(
        &self,
        memory: &MemAddr,
        fd: Fd,
        offset: i64,
        whence: u32,
        newoffset_ptr: Ptr,
    ) -> WasiResult<i32> {
        let errno =
            unsafe { __wasi_fd_seek(fd as u32, offset, whence, host_ptr(memory, newoffset_ptr)) };
        Ok(errno as i32)
    }

    pub fn fd_tell(&self, memory: &MemAddr, fd: Fd, offset_ptr: Ptr) -> WasiResult<i32> {
        let errno = unsafe { __wasi_fd_tell(fd as u32, host_ptr(memory, offset_ptr)) };
        Ok(errno as i32)
    }

    pub fn fd_sync(&self, fd: Fd) -> WasiResult<i32> {
        Ok(unsafe { __wasi_fd_sync(fd as u32) } as i32)
    }

    pub fn fd_filestat_get(&self, memory: &MemAddr, fd: Fd, filestat_ptr: Ptr) -> WasiResult<i32> {
        let errno = unsafe { __wasi_fd_filestat_get(fd as u32, host_ptr(memory, filestat_ptr)) };
        Ok(errno as i32)
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
        let errno = unsafe {
            __wasi_fd_readdir(
                fd as u32,
                host_ptr(memory, buf_ptr),
                buf_len,
                cookie,
                host_ptr(memory, buf_used_ptr),
            )
        };
        Ok(errno as i32)
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
        let iovecs = collect_iovecs(memory.get_memory_direct_access(), iovs_ptr, iovs_len)?;
        let mut nread: u32 = 0;
        let errno =
            unsafe { __wasi_fd_pread(fd as u32, iovecs.as_ptr(), iovs_len, offset, &mut nread) };
        if errno == 0 {
            store_u32(memory, nread_ptr, nread);
        }
        Ok(errno as i32)
    }

    pub fn fd_datasync(&self, fd: Fd) -> WasiResult<i32> {
        Ok(unsafe { __wasi_fd_datasync(fd as u32) } as i32)
    }

    pub fn fd_fdstat_set_flags(&self, fd: Fd, flags: u32) -> WasiResult<i32> {
        Ok(unsafe { __wasi_fd_fdstat_set_flags(fd as u32, flags) } as i32)
    }

    pub fn fd_filestat_set_size(&self, fd: Fd, size: u64) -> WasiResult<i32> {
        Ok(unsafe { __wasi_fd_filestat_set_size(fd as u32, size) } as i32)
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
        let iovecs = collect_iovecs(memory.get_memory_direct_access(), iovs_ptr, iovs_len)?;
        let mut nwritten: u32 = 0;
        let errno = unsafe {
            __wasi_fd_pwrite(fd as u32, iovecs.as_ptr(), iovs_len, offset, &mut nwritten)
        };
        if errno == 0 {
            store_u32(memory, nwritten_ptr, nwritten);
        }
        Ok(errno as i32)
    }

    pub fn path_create_directory(
        &self,
        memory: &MemAddr,
        fd: Fd,
        path_ptr: Ptr,
        path_len: Size,
    ) -> WasiResult<i32> {
        let path = guest_path(memory, path_ptr, path_len);
        Ok(unsafe { __wasi_path_create_directory(fd as u32, path.as_ptr()) } as i32)
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
        let path = guest_path(memory, path_ptr, path_len);
        let errno = unsafe {
            __wasi_path_filestat_get(
                fd as u32,
                flags,
                path.as_ptr(),
                host_ptr(memory, filestat_ptr),
            )
        };
        Ok(errno as i32)
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
        let path = guest_path(memory, path_ptr, path_len);
        let errno = unsafe {
            __wasi_path_filestat_set_times(fd as u32, flags, path.as_ptr(), atim, mtim, fst_flags)
        };
        Ok(errno as i32)
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
        let path = guest_path(memory, path_ptr, path_len);
        let errno = unsafe {
            __wasi_path_readlink(
                fd as u32,
                path.as_ptr(),
                host_ptr(memory, buf_ptr),
                buf_len,
                host_ptr(memory, buf_used_ptr),
            )
        };
        Ok(errno as i32)
    }

    pub fn path_remove_directory(
        &self,
        memory: &MemAddr,
        fd: Fd,
        path_ptr: Ptr,
        path_len: Size,
    ) -> WasiResult<i32> {
        let path = guest_path(memory, path_ptr, path_len);
        Ok(unsafe { __wasi_path_remove_directory(fd as u32, path.as_ptr()) } as i32)
    }

    pub fn path_unlink_file(
        &self,
        memory: &MemAddr,
        fd: Fd,
        path_ptr: Ptr,
        path_len: Size,
    ) -> WasiResult<i32> {
        let path = guest_path(memory, path_ptr, path_len);
        Ok(unsafe { __wasi_path_unlink_file(fd as u32, path.as_ptr()) } as i32)
    }

    pub fn poll_oneoff(
        &self,
        memory: &MemAddr,
        in_ptr: Ptr,
        out_ptr: Ptr,
        nsubscriptions: Size,
        nevents_ptr: Ptr,
    ) -> WasiResult<i32> {
        let errno = unsafe {
            __wasi_poll_oneoff(
                host_ptr(memory, in_ptr),
                host_ptr(memory, out_ptr),
                nsubscriptions,
                host_ptr(memory, nevents_ptr),
            )
        };
        Ok(errno as i32)
    }

    pub fn proc_raise(&self, signal: u32) -> WasiResult<i32> {
        Ok(unsafe { __wasi_proc_raise(signal) } as i32)
    }

    pub fn fd_advise(&self, fd: u32, offset: u64, len: u64, advice: u32) -> WasiResult<i32> {
        Ok(unsafe { __wasi_fd_advise(fd, offset, len, advice as u8) } as i32)
    }

    pub fn fd_allocate(&self, fd: u32, offset: u64, len: u64) -> WasiResult<i32> {
        Ok(unsafe { __wasi_fd_allocate(fd as i32, offset, len) } as i32)
    }

    pub fn fd_fdstat_set_rights(
        &self,
        fd: u32,
        fs_rights_base: u64,
        fs_rights_inheriting: u64,
    ) -> WasiResult<i32> {
        let errno =
            unsafe { __wasi_fd_fdstat_set_rights(fd, fs_rights_base, fs_rights_inheriting) };
        Ok(errno as i32)
    }

    pub fn fd_renumber(&self, fd: u32, to: u32) -> WasiResult<i32> {
        Ok(unsafe { __wasi_fd_renumber(fd as i32, to as i32) } as i32)
    }

    pub fn fd_filestat_set_times(
        &self,
        fd: u32,
        atim: u64,
        mtim: u64,
        fst_flags: u32,
    ) -> WasiResult<i32> {
        Ok(unsafe { __wasi_fd_filestat_set_times(fd, atim, mtim, fst_flags) } as i32)
    }

    pub fn path_link(
        &self,
        memory: &MemAddr,
        old_fd: u32,
        old_flags: u32,
        old_path_ptr: Ptr,
        old_path_len: Size,
        new_fd: u32,
        new_path_ptr: Ptr,
        new_path_len: Size,
    ) -> WasiResult<i32> {
        let old_path = guest_path(memory, old_path_ptr, old_path_len);
        let new_path = guest_path(memory, new_path_ptr, new_path_len);
        let errno = unsafe {
            __wasi_path_link(
                old_fd,
                old_flags,
                old_path.as_ptr(),
                new_fd,
                new_path.as_ptr(),
            )
        };
        Ok(errno as i32)
    }

    pub fn path_rename(
        &self,
        memory: &MemAddr,
        old_fd: u32,
        old_path_ptr: Ptr,
        old_path_len: Size,
        new_fd: u32,
        new_path_ptr: Ptr,
        new_path_len: Size,
    ) -> WasiResult<i32> {
        let old_path = guest_path(memory, old_path_ptr, old_path_len);
        let new_path = guest_path(memory, new_path_ptr, new_path_len);
        let errno =
            unsafe { __wasi_path_rename(old_fd, old_path.as_ptr(), new_fd, new_path.as_ptr()) };
        Ok(errno as i32)
    }

    pub fn path_symlink(
        &self,
        memory: &MemAddr,
        old_path_ptr: Ptr,
        old_path_len: Size,
        fd: u32,
        new_path_ptr: Ptr,
        new_path_len: Size,
    ) -> WasiResult<i32> {
        let old_path = guest_path(memory, old_path_ptr, old_path_len);
        let new_path = guest_path(memory, new_path_ptr, new_path_len);
        let errno = unsafe { __wasi_path_symlink(old_path.as_ptr(), fd, new_path.as_ptr()) };
        Ok(errno as i32)
    }

    pub fn sock_accept(
        &self,
        memory: &MemAddr,
        fd: u32,
        flags: u32,
        fd_ptr: Ptr,
    ) -> WasiResult<i32> {
        let errno = unsafe { __wasi_sock_accept(fd, flags, host_ptr(memory, fd_ptr)) };
        Ok(errno as i32)
    }

    pub fn sock_recv(
        &self,
        memory: &MemAddr,
        fd: u32,
        ri_data_ptr: Ptr,
        ri_data_len: Size,
        ri_flags: u32,
        ro_datalen_ptr: Ptr,
        ro_flags_ptr: Ptr,
    ) -> WasiResult<i32> {
        let iovecs = collect_iovecs(memory.get_memory_direct_access(), ri_data_ptr, ri_data_len)?;
        let errno = unsafe {
            __wasi_sock_recv(
                fd,
                iovecs.as_ptr(),
                ri_data_len,
                ri_flags,
                host_ptr(memory, ro_datalen_ptr),
                host_ptr(memory, ro_flags_ptr),
            )
        };
        Ok(errno as i32)
    }

    pub fn sock_send(
        &self,
        memory: &MemAddr,
        fd: u32,
        si_data_ptr: Ptr,
        si_data_len: Size,
        si_flags: u32,
        so_datalen_ptr: Ptr,
    ) -> WasiResult<i32> {
        let iovecs = collect_iovecs(memory.get_memory_direct_access(), si_data_ptr, si_data_len)?;
        let errno = unsafe {
            __wasi_sock_send(
                fd,
                iovecs.as_ptr(),
                si_data_len,
                si_flags,
                host_ptr(memory, so_datalen_ptr),
            )
        };
        Ok(errno as i32)
    }

    pub fn sock_shutdown(&self, fd: u32, how: u32) -> WasiResult<i32> {
        Ok(unsafe { __wasi_sock_shutdown(fd, how) } as i32)
    }
}
