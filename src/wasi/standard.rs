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
    
    /// fd_read - Read from a file descriptor
    pub fn fd_read(
        &self,
        memory: &MemAddr,
        fd: Fd,
        iovs_ptr: Ptr,
        iovs_len: Size,
        nread_ptr: Ptr,
    ) -> WasiResult<i32> {
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
    
    /// proc_exit - Exit the process
    pub fn proc_exit(&self, exit_code: ExitCode) -> WasiResult<i32> {
        Err(WasiError::ProcessExit(exit_code))
    }
    
    /// random_get - Get random bytes
    pub fn random_get(&self, memory: &MemAddr, buf_ptr: Ptr, buf_len: Size) -> WasiResult<i32> {
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
    
    /// fd_close - Close a file descriptor
    pub fn fd_close(&self, fd: Fd) -> WasiResult<i32> {
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