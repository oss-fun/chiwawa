//! Linear memory instances and load/store operations.

use crate::error::RuntimeError;
use crate::structure::{instructions::Memarg, types::*};
use byteorder::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;
use typenum::*;

/// Reference-counted handle to a memory instance.
#[derive(Clone, Debug)]
pub struct MemAddr {
    mem_inst: Rc<RefCell<MemInst>>,
}

/// Linear memory instance with bounds tracking.
#[derive(Debug, Serialize, Deserialize)]
pub struct MemInst {
    pub _type_: MemType,
    pub data: Vec<u8>,
}

impl MemAddr {
    /// Creates a new memory instance with initial size from type.
    pub fn new(type_: &MemType) -> MemAddr {
        let min = (type_.0.min * 65536) as usize;
        let max = type_.0.max.map(|max| max);
        MemAddr {
            mem_inst: Rc::new(RefCell::new(MemInst {
                _type_: MemType(Limits {
                    min: min as u32,
                    max,
                }),
                data: {
                    let mut vec = Vec::with_capacity(min);
                    vec.resize(min, 0);
                    vec
                },
            })),
        }
    }

    /// Initializes memory region from data segment.
    pub fn init(&self, offset: usize, init: &Vec<u8>) {
        let mut addr_self = self.mem_inst.borrow_mut();
        addr_self.data[offset..offset + init.len()].copy_from_slice(init);
    }
    /// Loads a typed value from memory at ptr + offset.
    pub fn load<T: ByteMem>(&self, arg: &Memarg, ptr: i32) -> Result<T, RuntimeError> {
        let pos = (ptr as usize) + (arg.offset as usize);
        let len = <T>::len();
        let raw = self.mem_inst.borrow();

        let data = unsafe {
            let slice = std::slice::from_raw_parts(raw.data.as_ptr().add(pos), len);
            slice.to_vec()
        };
        Ok(<T>::from_byte(data))
    }
    /// Stores a typed value to memory at ptr + offset.
    pub fn store<T: ByteMem>(&self, arg: &Memarg, ptr: i32, data: T) -> Result<(), RuntimeError> {
        let pos = (ptr as usize) + (arg.offset as usize);
        let buf = <T>::to_byte(data);
        let mut raw = self.mem_inst.borrow_mut();

        unsafe {
            std::ptr::copy_nonoverlapping(buf.as_ptr(), raw.data.as_mut_ptr().add(pos), buf.len());
        }

        Ok(())
    }

    /// Returns current memory size in pages (64KB each).
    pub fn mem_size(&self) -> i32 {
        (self.mem_inst.borrow().data.len() / 65536) as i32
    }

    /// Grows memory by the given number of pages. Returns previous size or -1 on failure.
    pub fn mem_grow(&self, size: i32) -> i32 {
        let prev_size = self.mem_size();
        let new = prev_size + size;

        let max_pages = {
            let guard = self.mem_inst.borrow();
            guard._type_.0.max
        };

        if let Some(max) = max_pages {
            if new > max as i32 {
                return -1;
            }
        }

        if new > 65536 {
            -1
        } else {
            self.mem_inst
                .borrow_mut()
                .data
                .resize(new as usize * 65536, 0);

            prev_size
        }
    }

    /// Returns a copy of all memory contents.
    pub fn get_data(&self) -> Result<Vec<u8>, RuntimeError> {
        let guard = self.mem_inst.borrow();
        Ok(guard.data.clone())
    }

    /// Store multiple bytes at once (bulk operation)
    pub fn store_bytes(&self, ptr: i32, data: &[u8]) -> Result<(), RuntimeError> {
        let pos = ptr as usize;
        let mut raw = self.mem_inst.borrow_mut();

        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                raw.data.as_mut_ptr().add(pos),
                data.len(),
            );
        }

        Ok(())
    }

    /// Replaces all memory contents (used during restore).
    pub fn set_data(&self, data: Vec<u8>) -> Result<(), RuntimeError> {
        let mut guard = self.mem_inst.borrow_mut();
        guard.data = data;
        Ok(())
    }

    /// Copies len bytes from src to dest within memory.
    pub fn memory_copy(&self, dest: i32, src: i32, len: i32) -> Result<(), RuntimeError> {
        let dest_pos = dest as usize;
        let src_pos = src as usize;
        let len_usize = len as usize;
        let mut raw = self.mem_inst.borrow_mut();

        if len_usize > 0 {
            unsafe {
                let src_ptr = raw.data.as_ptr().add(src_pos);
                let dest_ptr = raw.data.as_mut_ptr().add(dest_pos);
                std::ptr::copy(src_ptr, dest_ptr, len_usize);
            }
        }

        Ok(())
    }

    /// Fills len bytes starting at dest with val.
    pub fn memory_fill(&self, dest: i32, val: u8, len: i32) -> Result<(), RuntimeError> {
        let dest_pos = dest as usize;
        let len_usize = len as usize;
        let mut raw = self.mem_inst.borrow_mut();

        if len_usize > 0 {
            unsafe {
                std::ptr::write_bytes(raw.data.as_mut_ptr().add(dest_pos), val, len_usize);
            }
        }

        Ok(())
    }

    /// Returns a direct borrow of the memory instance.
    pub fn get_memory_direct_access(&self) -> std::cell::Ref<MemInst> {
        self.mem_inst.borrow()
    }
}

/// Trait for types that can be loaded/stored from memory.
pub trait ByteMem: Sized {
    /// Size in bytes.
    fn len() -> usize;
    /// Deserialize from little-endian bytes.
    fn from_byte(data: Vec<u8>) -> Self;
    /// Serialize to little-endian bytes.
    fn to_byte(self) -> Vec<u8>;
}

impl ByteMem for i8 {
    fn len() -> usize {
        consts::U1::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> i8 {
        let mut reader = Cursor::new(data.as_slice());
        reader.read_i8().unwrap()
    }
    fn to_byte(self) -> Vec<u8> {
        vec![self as u8]
    }
}
impl ByteMem for u8 {
    fn len() -> usize {
        consts::U1::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> u8 {
        let mut reader = Cursor::new(data.as_slice());
        reader.read_u8().unwrap()
    }
    fn to_byte(self) -> Vec<u8> {
        vec![self]
    }
}

impl ByteMem for i16 {
    fn len() -> usize {
        consts::U2::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> i16 {
        let mut reader = Cursor::new(data.as_slice());
        reader.read_i16::<LittleEndian>().unwrap()
    }
    fn to_byte(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ByteMem for u16 {
    fn len() -> usize {
        consts::U2::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> u16 {
        let mut reader = Cursor::new(data.as_slice());
        reader.read_u16::<LittleEndian>().unwrap()
    }
    fn to_byte(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ByteMem for i32 {
    fn len() -> usize {
        consts::U4::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> i32 {
        let mut reader = Cursor::new(data.as_slice());
        reader.read_i32::<LittleEndian>().unwrap()
    }
    fn to_byte(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ByteMem for u32 {
    fn len() -> usize {
        consts::U4::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> u32 {
        let mut reader = Cursor::new(data.as_slice());
        reader.read_u32::<LittleEndian>().unwrap()
    }
    fn to_byte(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ByteMem for i64 {
    fn len() -> usize {
        consts::U8::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> i64 {
        let mut reader = Cursor::new(data.as_slice());
        reader.read_i64::<LittleEndian>().unwrap()
    }
    fn to_byte(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ByteMem for f32 {
    fn len() -> usize {
        consts::U4::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> f32 {
        let mut reader = Cursor::new(data.as_slice());
        reader.read_f32::<LittleEndian>().unwrap()
    }
    fn to_byte(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ByteMem for f64 {
    fn len() -> usize {
        consts::U8::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> f64 {
        let mut reader = Cursor::new(data.as_slice());
        reader.read_f64::<LittleEndian>().unwrap()
    }
    fn to_byte(self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}
