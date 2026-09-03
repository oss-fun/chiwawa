//! Linear memory instances and load/store operations.

use crate::shared::Shared;
use crate::structure::types::*;
use serde::{Deserialize, Serialize};
use std::cell::UnsafeCell;
#[cfg(feature = "threads")]
use std::sync::Mutex;

/// Reference-counted handle to a memory instance.
/// Uses UnsafeCell for zero-cost memory access in the interpreter hot path.
/// Safety: accesses within one interpreter do not overlap. A memory shared
/// between wasi-threads threads is the one exception
#[derive(Clone, Debug)]
pub struct MemAddr {
    mem_inst: Shared<UnsafeCell<MemInst>>,
    /// Serializes `mem_grow` between threads
    #[cfg(feature = "threads")]
    grow_lock: Shared<Mutex<()>>,
}

#[cfg(feature = "threads")]
unsafe impl Send for MemAddr {}
#[cfg(feature = "threads")]
unsafe impl Sync for MemAddr {}

/// Linear memory instance with bounds tracking.
#[derive(Debug, Serialize, Deserialize)]
pub struct MemInst {
    pub _type_: MemType,
    pub data: Vec<u8>,
}

impl MemAddr {
    /// Creates a new memory instance with initial size from type.
    pub fn new(type_: &MemType) -> MemAddr {
        let min = (type_.limits.min * 65536) as usize;
        let max = type_.limits.max.map(|max| max);
        let shared = type_.shared;
        // Threads cache raw pointers into this buffer, so a shared memory
        // reserves its declared maximum here and never reallocates on growth.
        let capacity = if shared {
            max.map_or(min, |max| (max as usize) * 65536)
        } else {
            min
        };
        MemAddr {
            mem_inst: Shared::new(UnsafeCell::new(MemInst {
                _type_: MemType {
                    limits: Limits {
                        min: min as u32,
                        max,
                    },
                    shared,
                },
                data: {
                    let mut vec = Vec::with_capacity(capacity);
                    vec.resize(min, 0);
                    vec
                },
            })),
            #[cfg(feature = "threads")]
            grow_lock: Shared::new(Mutex::new(())),
        }
    }

    /// Initializes memory region from data segment.
    #[inline]
    pub fn init(&self, offset: usize, init: &[u8]) {
        // Safety: Single-threaded access, no overlapping borrows
        let mem = unsafe { &mut *self.mem_inst.get() };
        mem.data[offset..offset + init.len()].copy_from_slice(init);
    }

    /// Loads a typed value from memory at ptr + offset.
    /// No bounds checking - relies on host runtime for memory safety.
    /// No heap allocation - reads directly from memory pointer.
    #[inline(always)]
    pub fn load<T: ByteMem>(&self, offset: u64, ptr: i32) -> T {
        let pos = (ptr as usize) + (offset as usize);
        // Safety: Single-threaded access, no overlapping borrows
        let mem = unsafe { &*self.mem_inst.get() };
        unsafe { T::read_from_ptr(mem.data.as_ptr().add(pos)) }
    }

    /// Stores a typed value to memory at ptr + offset.
    /// No bounds checking - relies on host runtime for memory safety.
    /// No heap allocation - writes directly to memory pointer.
    #[inline(always)]
    pub fn store<T: ByteMem>(&self, offset: u64, ptr: i32, data: T) {
        let pos = (ptr as usize) + (offset as usize);
        // Safety: Single-threaded access, no overlapping borrows
        let mem = unsafe { &mut *self.mem_inst.get() };
        unsafe { data.write_to_ptr(mem.data.as_mut_ptr().add(pos)) }
    }

    /// Returns raw mutable pointer to memory data for caching.
    /// Safety: Caller must ensure pointer is not used after memory grows.
    #[inline(always)]
    pub fn data_ptr(&self) -> *mut u8 {
        // Safety: Single-threaded access
        let mem = unsafe { &mut *self.mem_inst.get() };
        mem.data.as_mut_ptr()
    }

    /// Returns current memory size in pages (64KB each).
    #[inline]
    pub fn mem_size(&self) -> i32 {
        // Safety: Single-threaded access
        let mem = unsafe { &*self.mem_inst.get() };
        (mem.data.len() / 65536) as i32
    }

    /// Grows memory by the given number of pages. Returns previous size or -1 on failure.
    pub fn mem_grow(&self, size: i32) -> i32 {
        // Only one thread may extend the memory at a time.
        #[cfg(feature = "threads")]
        let _grow_guard = self
            .grow_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let prev_size = self.mem_size();
        let new = prev_size + size;

        // Safety: Single-threaded access
        let mem = unsafe { &*self.mem_inst.get() };
        let max_pages = mem._type_.limits.max;

        if let Some(max) = max_pages {
            if new > max as i32 {
                return -1;
            }
        }

        // Growing a shared memory past its reservation would move the buffer
        // out from under the pointers other threads have cached.
        if mem._type_.shared && (new as usize) * 65536 > mem.data.capacity() {
            return -1;
        }

        if new > 65536 {
            -1
        } else {
            // Safety: Single-threaded access, no overlapping borrows
            let mem = unsafe { &mut *self.mem_inst.get() };
            mem.data.resize(new as usize * 65536, 0);
            prev_size
        }
    }

    /// Returns a copy of all memory contents.
    #[inline]
    pub fn get_data(&self) -> Vec<u8> {
        // Safety: Single-threaded access
        let mem = unsafe { &*self.mem_inst.get() };
        mem.data.clone()
    }

    /// Store multiple bytes at once (bulk operation)
    /// No bounds checking - relies on host runtime for memory safety.
    #[inline]
    pub fn store_bytes(&self, ptr: i32, data: &[u8]) {
        let pos = ptr as usize;
        // Safety: Single-threaded access, no overlapping borrows
        let mem = unsafe { &mut *self.mem_inst.get() };

        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                mem.data.as_mut_ptr().add(pos),
                data.len(),
            );
        }
    }

    /// Replaces all memory contents (used during restore).
    #[inline]
    pub fn set_data(&self, data: Vec<u8>) {
        // Safety: Single-threaded access, no overlapping borrows
        let mem = unsafe { &mut *self.mem_inst.get() };
        mem.data = data;
    }

    /// Copies len bytes from src to dest within memory.
    /// No bounds checking - relies on host runtime for memory safety.
    #[inline]
    pub fn memory_copy(&self, dest: i32, src: i32, len: i32) {
        let dest_pos = dest as usize;
        let src_pos = src as usize;
        let len_usize = len as usize;
        // Safety: Single-threaded access, no overlapping borrows
        let mem = unsafe { &mut *self.mem_inst.get() };

        unsafe {
            let src_ptr = mem.data.as_ptr().add(src_pos);
            let dest_ptr = mem.data.as_mut_ptr().add(dest_pos);
            std::ptr::copy(src_ptr, dest_ptr, len_usize);
        }
    }

    /// Fills len bytes starting at dest with val.
    /// No bounds checking - relies on host runtime for memory safety.
    #[inline]
    pub fn memory_fill(&self, dest: i32, val: u8, len: i32) {
        let dest_pos = dest as usize;
        let len_usize = len as usize;
        // Safety: Single-threaded access, no overlapping borrows
        let mem = unsafe { &mut *self.mem_inst.get() };

        unsafe {
            std::ptr::write_bytes(mem.data.as_mut_ptr().add(dest_pos), val, len_usize);
        }
    }

    /// Returns a raw pointer to the memory data for direct access.
    /// # Safety
    /// Caller must ensure no mutable aliases exist during use.
    #[inline]
    pub unsafe fn get_data_ptr(&self) -> *const u8 {
        (*self.mem_inst.get()).data.as_ptr()
    }

    /// Returns a mutable raw pointer to the memory data for direct access.
    /// # Safety
    /// Caller must ensure no other references exist during use.
    #[inline]
    pub unsafe fn get_data_mut_ptr(&self) -> *mut u8 {
        (*self.mem_inst.get()).data.as_mut_ptr()
    }

    /// Returns the length of the memory data.
    #[inline]
    pub fn data_len(&self) -> usize {
        // Safety: Single-threaded access
        let mem = unsafe { &*self.mem_inst.get() };
        mem.data.len()
    }

    /// Returns a reference to the memory instance for direct access.
    /// This is used by WASI functions that need to read memory directly.
    /// # Safety
    /// The returned reference must not be held across operations that could
    /// mutate memory. Caller is responsible for ensuring no aliasing violations.
    #[inline]
    pub fn get_memory_direct_access(&self) -> &MemInst {
        // Safety: Single-threaded access, caller must ensure no overlapping mutable access
        unsafe { &*self.mem_inst.get() }
    }
}

/// Trait for types that can be loaded/stored from memory.
/// Uses direct pointer access to avoid heap allocations.
pub trait ByteMem: Sized {
    /// Read value directly from memory pointer (little-endian).
    /// # Safety
    /// Caller must ensure ptr is valid and properly aligned for the type.
    unsafe fn read_from_ptr(ptr: *const u8) -> Self;

    /// Write value directly to memory pointer (little-endian).
    /// # Safety
    /// Caller must ensure ptr is valid and has enough space.
    unsafe fn write_to_ptr(self, ptr: *mut u8);
}

impl ByteMem for i8 {
    #[inline]
    unsafe fn read_from_ptr(ptr: *const u8) -> i8 {
        *ptr as i8
    }
    #[inline]
    unsafe fn write_to_ptr(self, ptr: *mut u8) {
        *ptr = self as u8;
    }
}

impl ByteMem for u8 {
    #[inline]
    unsafe fn read_from_ptr(ptr: *const u8) -> u8 {
        *ptr
    }
    #[inline]
    unsafe fn write_to_ptr(self, ptr: *mut u8) {
        *ptr = self;
    }
}

impl ByteMem for i16 {
    #[inline]
    unsafe fn read_from_ptr(ptr: *const u8) -> i16 {
        std::ptr::read_unaligned(ptr as *const i16).to_le()
    }
    #[inline]
    unsafe fn write_to_ptr(self, ptr: *mut u8) {
        std::ptr::write_unaligned(ptr as *mut i16, self.to_le());
    }
}

impl ByteMem for u16 {
    #[inline]
    unsafe fn read_from_ptr(ptr: *const u8) -> u16 {
        std::ptr::read_unaligned(ptr as *const u16).to_le()
    }
    #[inline]
    unsafe fn write_to_ptr(self, ptr: *mut u8) {
        std::ptr::write_unaligned(ptr as *mut u16, self.to_le());
    }
}

impl ByteMem for i32 {
    #[inline]
    unsafe fn read_from_ptr(ptr: *const u8) -> i32 {
        std::ptr::read_unaligned(ptr as *const i32).to_le()
    }
    #[inline]
    unsafe fn write_to_ptr(self, ptr: *mut u8) {
        std::ptr::write_unaligned(ptr as *mut i32, self.to_le());
    }
}

impl ByteMem for u32 {
    #[inline]
    unsafe fn read_from_ptr(ptr: *const u8) -> u32 {
        std::ptr::read_unaligned(ptr as *const u32).to_le()
    }
    #[inline]
    unsafe fn write_to_ptr(self, ptr: *mut u8) {
        std::ptr::write_unaligned(ptr as *mut u32, self.to_le());
    }
}

impl ByteMem for i64 {
    #[inline]
    unsafe fn read_from_ptr(ptr: *const u8) -> i64 {
        std::ptr::read_unaligned(ptr as *const i64).to_le()
    }
    #[inline]
    unsafe fn write_to_ptr(self, ptr: *mut u8) {
        std::ptr::write_unaligned(ptr as *mut i64, self.to_le());
    }
}

impl ByteMem for f32 {
    #[inline]
    unsafe fn read_from_ptr(ptr: *const u8) -> f32 {
        f32::from_bits(std::ptr::read_unaligned(ptr as *const u32).to_le())
    }
    #[inline]
    unsafe fn write_to_ptr(self, ptr: *mut u8) {
        std::ptr::write_unaligned(ptr as *mut u32, self.to_bits().to_le());
    }
}

impl ByteMem for f64 {
    #[inline]
    unsafe fn read_from_ptr(ptr: *const u8) -> f64 {
        f64::from_bits(std::ptr::read_unaligned(ptr as *const u64).to_le())
    }
    #[inline]
    unsafe fn write_to_ptr(self, ptr: *mut u8) {
        std::ptr::write_unaligned(ptr as *mut u64, self.to_bits().to_le());
    }
}
