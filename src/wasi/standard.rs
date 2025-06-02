use super::*;
use super::context::*;
use crate::execution::mem::MemAddr;
use crate::structure::instructions::Memarg;
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
    
    /// fd_write - Write to a file descriptor
    pub fn fd_write(
        &self,
        memory: &MemAddr,
        fd: Fd,
        iovs_ptr: Ptr,
        iovs_len: Size,
        nwritten_ptr: Ptr,
    ) -> WasiResult<i32> {
        // Use inline WebAssembly to call fd_write
        #[cfg(target_arch = "wasm32")] // Re-enable for testing
        {
            let result: i32;
            unsafe {
                std::arch::asm!(
                    "local.get {fd}",
                    "local.get {iovs_ptr}",
                    "local.get {iovs_len}",
                    "local.get {nwritten_ptr}",
                    "call 0", // call fd_write (import index 0)
                    "local.set {result}",
                    fd = in(local) fd,
                    iovs_ptr = in(local) iovs_ptr,
                    iovs_len = in(local) iovs_len,
                    nwritten_ptr = in(local) nwritten_ptr,
                    result = out(local) result,
                );
            }
            Ok(result)
        }
        
        #[cfg(not(target_arch = "wasm32"))] // Host implementation for native builds
        {
            let mut ctx = self.context.lock().unwrap();
            let mut total_written = 0usize;
            
            // Read iovec array from memory
            for i in 0..iovs_len {
                let iov_offset = iovs_ptr as usize + (i as usize * 8); // Each iovec is 8 bytes
                
                // Read buf pointer (4 bytes)
                let buf_ptr = memory.load::<i32>(&Memarg { offset: 0, align: 4 }, iov_offset as i32)
                    .map_err(|_| WasiError::MemoryAccessError)?;
                // Read buf_len (4 bytes)
                let buf_len = memory.load::<i32>(&Memarg { offset: 4, align: 4 }, iov_offset as i32)
                    .map_err(|_| WasiError::MemoryAccessError)? as u32;
                
                if buf_len > 0 {
                    // Read data from memory
                    let mut data = vec![0u8; buf_len as usize];
                    for j in 0..buf_len {
                        let byte = memory.load::<u8>(&Memarg { offset: 0, align: 1 }, (buf_ptr as usize + j as usize) as i32)
                            .map_err(|_| WasiError::MemoryAccessError)?;
                        data[j as usize] = byte;
                    }
                    
                    // Write to file descriptor
                    let fd_ref = ctx.get_fd(fd)?;
                    let written = fd_ref.write(&data)?;
                    total_written += written;
                }
            }
            
            // Write total bytes written to memory
            memory.store::<i32>(&Memarg { offset: 0, align: 4 }, nwritten_ptr as i32, total_written as i32)
                .map_err(|_| WasiError::MemoryAccessError)?;
            
            Ok(0) // Success
        }
    }
    
    /// fd_read - Read from a file descriptor
    pub fn fd_read(
        &self,
        memory: &MemAddr,
        fd: Fd,
        iovs_ptr: Ptr,
        iovs_len: Size,
        nread_ptr: Ptr,
    ) -> WasiResult<i32> {
        // Use inline WebAssembly to call fd_read
        #[cfg(never)] // Temporarily disable inline WASM
        {
            let result: i32;
            unsafe {
                std::arch::asm!(
                    "local.get {fd}",
                    "local.get {iovs_ptr}",
                    "local.get {iovs_len}",
                    "local.get {nread_ptr}",
                    "call 2", // call fd_read (import index 2)
                    "local.set {result}",
                    fd = in(local) fd,
                    iovs_ptr = in(local) iovs_ptr,
                    iovs_len = in(local) iovs_len,
                    nread_ptr = in(local) nread_ptr,
                    result = out(local) result,
                );
            }
            Ok(result)
        }
        
        #[cfg(not(never))] // Always use host implementation for now
        {
            let mut ctx = self.context.lock().unwrap();
            let mut total_read = 0usize;
            
            // Read iovec array from memory
            for i in 0..iovs_len {
                let iov_offset = iovs_ptr as usize + (i as usize * 8);
                
                // Read buf pointer and length
                let buf_ptr = memory.load::<i32>(&Memarg { offset: 0, align: 4 }, iov_offset as i32)
                    .map_err(|_| WasiError::MemoryAccessError)? as u32;
                let buf_len = memory.load::<i32>(&Memarg { offset: 4, align: 4 }, iov_offset as i32)
                    .map_err(|_| WasiError::MemoryAccessError)? as u32;
                
                if buf_len > 0 {
                    // Read data into buffer
                    let mut data = vec![0u8; buf_len as usize];
                    let fd_ref = ctx.get_fd(fd)?;
                    let read_bytes = fd_ref.read(&mut data)?;
                    
                    // Write data to memory
                    for j in 0..read_bytes {
                        memory.store::<u8>(&Memarg { offset: 0, align: 1 }, (buf_ptr as usize + j) as i32, data[j])
                            .map_err(|_| WasiError::MemoryAccessError)?;
                    }
                    
                    total_read += read_bytes;
                    
                    // If we read less than requested, we're done
                    if read_bytes < buf_len as usize {
                        break;
                    }
                }
            }
            
            // Write total bytes read to memory
            memory.store::<i32>(&Memarg { offset: 0, align: 4 }, nread_ptr as i32, total_read as i32)
                .map_err(|_| WasiError::MemoryAccessError)?;
            
            Ok(0) // Success
        }
    }
    
    /// proc_exit - Exit the process
    pub fn proc_exit(&self, exit_code: ExitCode) -> WasiResult<i32> {
        // Use inline WebAssembly to call proc_exit
        #[cfg(never)] // Temporarily disable inline WASM
        {
            unsafe {
                std::arch::asm!(
                    "local.get {exit_code}",
                    "call 3", // call proc_exit (import index 3)
                    "unreachable", // proc_exit never returns
                    exit_code = in(local) exit_code,
                );
            }
            // This should never be reached
            unreachable!()
        }
        
        #[cfg(not(never))] // Always use host implementation for now
        {
            Err(WasiError::ProcessExit(exit_code))
        }
    }
    
    /// random_get - Get random bytes
    pub fn random_get(&self, memory: &MemAddr, buf_ptr: Ptr, buf_len: Size) -> WasiResult<i32> {
        // Use inline WebAssembly to call random_get
        #[cfg(never)] // Temporarily disable inline WASM
        {
            let result: i32;
            unsafe {
                std::arch::asm!(
                    "local.get {buf_ptr}",
                    "local.get {buf_len}",
                    "call 4", // call random_get (import index 4)
                    "local.set {result}",
                    buf_ptr = in(local) buf_ptr,
                    buf_len = in(local) buf_len,
                    result = out(local) result,
                );
            }
            Ok(result)
        }
        
        #[cfg(not(never))] // Always use host implementation for now
        {
            use getrandom::getrandom;
            
            let mut buffer = vec![0u8; buf_len as usize];
            getrandom(&mut buffer).map_err(|_| WasiError::IoError)?;
            
            // Write random data to memory
            for (i, &byte) in buffer.iter().enumerate() {
                memory.store::<u8>(&Memarg { offset: 0, align: 1 }, (buf_ptr as usize + i) as i32, byte)
                    .map_err(|_| WasiError::MemoryAccessError)?;
            }
            
            Ok(0) // Success
        }
    }
    
    /// fd_close - Close a file descriptor
    pub fn fd_close(&self, fd: Fd) -> WasiResult<i32> {
        // Use inline WebAssembly to call fd_close
        #[cfg(never)] // Temporarily disable inline WASM
        {
            let result: i32;
            unsafe {
                std::arch::asm!(
                    "local.get {fd}",
                    "call 6", // call fd_close (import index 6)
                    "local.set {result}",
                    fd = in(local) fd,
                    result = out(local) result,
                );
            }
            Ok(result)
        }
        
        #[cfg(not(never))] // Always use host implementation for now
        {
            let mut ctx = self.context.lock().unwrap();
            
            if fd <= 2 {
                // Don't actually close stdin/stdout/stderr
                return Ok(0);
            }
            
            if let Some(mut fd_obj) = ctx.file_descriptors.remove(&fd) {
                fd_obj.close()?;
            }
            
            Ok(0) // Success
        }
    }

    /// environ_get - Get environment variables
    pub fn environ_get(&self, memory: &MemAddr, environ_ptr: Ptr, environ_buf_ptr: Ptr) -> WasiResult<i32> {
        // Use inline WebAssembly to call environ_get
        #[cfg(never)] // Temporarily disable inline WASM
        {
            let result: i32;
            unsafe {
                std::arch::asm!(
                    "local.get {environ_ptr}",
                    "local.get {environ_buf_ptr}",
                    "call 1", // call environ_get (import index 1)
                    "local.set {result}",
                    environ_ptr = in(local) environ_ptr,
                    environ_buf_ptr = in(local) environ_buf_ptr,
                    result = out(local) result,
                );
            }
            Ok(result)
        }
        
        #[cfg(not(never))] // Always use host implementation for now
        {
            // For simplicity, we return an empty environment
            // In a real implementation, you would populate this with actual environment variables
            
            // Write 0 (null pointer) to indicate no environment variables
            memory.store::<i32>(&Memarg { offset: 0, align: 4 }, environ_ptr as i32, 0)
                .map_err(|_| WasiError::MemoryAccessError)?;

            Ok(0) // Success
        }
    }

    /// environ_sizes_get - Get environment variable sizes
    pub fn environ_sizes_get(&self, memory: &MemAddr, environ_count_ptr: Ptr, environ_buf_size_ptr: Ptr) -> WasiResult<i32> {
        // Use inline WebAssembly to call environ_sizes_get
        #[cfg(never)] // Temporarily disable inline WASM
        {
            let result: i32;
            unsafe {
                std::arch::asm!(
                    "local.get {environ_count_ptr}",
                    "local.get {environ_buf_size_ptr}",
                    "call 5", // call environ_sizes_get (import index 5)
                    "local.set {result}",
                    environ_count_ptr = in(local) environ_count_ptr,
                    environ_buf_size_ptr = in(local) environ_buf_size_ptr,
                    result = out(local) result,
                );
            }
            Ok(result)
        }
        
        #[cfg(not(never))] // Always use host implementation for now
        {
            // Return 0 environment variables and 0 buffer size
            memory.store::<i32>(&Memarg { offset: 0, align: 4 }, environ_count_ptr as i32, 0)
                .map_err(|_| WasiError::MemoryAccessError)?;
            memory.store::<i32>(&Memarg { offset: 0, align: 4 }, environ_buf_size_ptr as i32, 0)
                .map_err(|_| WasiError::MemoryAccessError)?;
            
            Ok(0) // Success
        }
    }
} 