use crate::error::RuntimeError;
use crate::execution::memoization::MemoryChunkTracker;
use crate::structure::{instructions::Memarg, types::*};
use byteorder::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashSet;
use std::io::Cursor;
use std::rc::Rc;
use typenum::*;

// Chunk size for memory version tracking (4KB chunks for fine-grained invalidation)
// Note: This is different from WebAssembly page size (64KB)
pub const CHUNK_SIZE: usize = 4096;

#[derive(Clone, Debug)]
pub struct MemAddr {
    mem_inst: Rc<RefCell<MemInst>>,
    chunk_versions: Rc<RefCell<Vec<u64>>>,
    current_block_written_chunks: RefCell<Option<MemoryChunkTracker>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct MemInst {
    pub _type_: MemType,
    pub data: Vec<u8>,
}

impl MemAddr {
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
            chunk_versions: Rc::new(RefCell::new(Vec::new())),
            current_block_written_chunks: RefCell::new(None),
        }
    }

    fn update_chunk_versions(&self, start_pos: usize, len: usize) {
        if len > 0 {
            let start_chunk = start_pos / CHUNK_SIZE;
            let end_chunk = (start_pos + len - 1) / CHUNK_SIZE;
            let mut versions = self.chunk_versions.borrow_mut();

            // Ensure vector is large enough
            if end_chunk >= versions.len() {
                versions.resize(end_chunk + 1, 0);
            }

            // Update chunk versions
            for chunk in start_chunk..=end_chunk {
                versions[chunk] += 1;
            }

            // Try to record in current_block_written_chunks if tracking
            if let Ok(mut access_lock) = self.current_block_written_chunks.try_borrow_mut() {
                if let Some(ref mut tracker) = *access_lock {
                    for chunk in start_chunk..=end_chunk {
                        tracker.track_access(chunk as u32);
                    }
                }
            }
        }
    }

    pub fn init(&self, offset: usize, init: &Vec<u8>) {
        let mut addr_self = self.mem_inst.borrow_mut();
        addr_self.data[offset..offset + init.len()].copy_from_slice(init);
        drop(addr_self);
        self.update_chunk_versions(offset, init.len());
    }
    pub fn load<T: ByteMem>(&self, arg: &Memarg, ptr: i32) -> Result<T, RuntimeError> {
        let pos = ptr
            .checked_add(i32::try_from(arg.offset).ok().unwrap())
            .ok_or_else(|| RuntimeError::InstructionFailed)? as usize;
        let len = <T>::len();
        let raw = &self.mem_inst.borrow().data;
        let data = Vec::from(&raw[pos..pos + len]);
        Ok(<T>::from_byte(data))
    }
    pub fn store<T: ByteMem>(&self, arg: &Memarg, ptr: i32, data: T) -> Result<(), RuntimeError> {
        let pos = ptr
            .checked_add(i32::try_from(arg.offset).ok().unwrap())
            .ok_or_else(|| RuntimeError::InstructionFailed)? as usize;
        let buf = <T>::to_byte(data);
        let mut raw = self.mem_inst.borrow_mut();

        raw.data[pos..pos + buf.len()].copy_from_slice(&buf);

        let len = <T>::len();
        drop(raw);
        self.update_chunk_versions(pos, len);

        Ok(())
    }

    pub fn mem_size(&self) -> i32 {
        (self.mem_inst.borrow().data.len() / 65536) as i32
    }

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

            // Ensure chunk versions vector is large enough for new memory size
            // Convert from WASM pages (64KB) to internal tracking chunks (4KB)
            if new > 0 {
                let mut versions = self.chunk_versions.borrow_mut();
                let total_chunks = (new as usize * 65536) / CHUNK_SIZE;
                if total_chunks > versions.len() {
                    versions.resize(total_chunks, 0);
                }
            }

            prev_size
        }
    }

    pub fn get_data(&self) -> Result<Vec<u8>, RuntimeError> {
        let guard = self.mem_inst.borrow();
        Ok(guard.data.clone())
    }

    /// Store multiple bytes at once (bulk operation)
    pub fn store_bytes(&self, ptr: i32, data: &[u8]) -> Result<(), RuntimeError> {
        let pos = ptr as usize;
        let mut raw = self.mem_inst.borrow_mut();

        // Bounds check
        if pos.checked_add(data.len()).unwrap_or(usize::MAX) > raw.data.len() {
            return Err(RuntimeError::InstructionFailed);
        }

        // Bulk copy
        raw.data[pos..pos + data.len()].copy_from_slice(data);

        drop(raw);
        self.update_chunk_versions(pos, data.len());

        Ok(())
    }

    pub fn set_data(&self, data: Vec<u8>) -> Result<(), RuntimeError> {
        let mut guard = self.mem_inst.borrow_mut();
        guard.data = data;
        Ok(())
    }

    pub fn memory_copy(&self, dest: i32, src: i32, len: i32) -> Result<(), RuntimeError> {
        let dest_pos = dest as usize;
        let src_pos = src as usize;
        let len_usize = len as usize;
        let mut raw = self.mem_inst.borrow_mut();

        if len_usize > 0 {
            raw.data.copy_within(src_pos..src_pos + len_usize, dest_pos);

            drop(raw);
            self.update_chunk_versions(dest_pos, len_usize);
        }

        Ok(())
    }

    pub fn memory_fill(&self, dest: i32, val: u8, len: i32) -> Result<(), RuntimeError> {
        let dest_pos = dest as usize;
        let len_usize = len as usize;
        let mut raw = self.mem_inst.borrow_mut();

        // Bounds check
        if dest_pos.checked_add(len_usize).unwrap_or(usize::MAX) > raw.data.len() {
            return Err(RuntimeError::MemoryOutOfBounds);
        }

        if len_usize > 0 {
            // Use fill for efficient bulk operation
            raw.data[dest_pos..dest_pos + len_usize].fill(val);

            drop(raw);
            self.update_chunk_versions(dest_pos, len_usize);
        }

        Ok(())
    }

    pub fn get_memory_direct_access(&self) -> std::cell::Ref<MemInst> {
        self.mem_inst.borrow()
    }

    pub fn get_all_chunk_versions(&self) -> Vec<(u32, u64)> {
        let versions = self.chunk_versions.borrow();
        versions
            .iter()
            .enumerate()
            .filter_map(|(idx, &version)| {
                if version > 0 {
                    Some((idx as u32, version))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn start_tracking_access(&self) {
        let mut access_lock = self.current_block_written_chunks.borrow_mut();
        *access_lock = Some(MemoryChunkTracker::new());
    }

    pub fn get_and_stop_tracking_access(&self) -> Option<MemoryChunkTracker> {
        let mut access_lock = self.current_block_written_chunks.borrow_mut();
        access_lock.take()
    }

    pub fn get_chunk_versions_for_chunks(&self, chunks: &HashSet<u32>) -> Vec<(u32, u64)> {
        let versions = self.chunk_versions.borrow();
        let mut result: Vec<(u32, u64)> = chunks
            .iter()
            .filter_map(|&chunk| {
                if (chunk as usize) < versions.len() && versions[chunk as usize] > 0 {
                    Some((chunk, versions[chunk as usize]))
                } else {
                    None
                }
            })
            .collect();
        result.sort_by_key(|&(chunk, _)| chunk);
        result
    }

    pub fn get_chunk_versions_for_tracker(
        &self,
        tracker: &crate::execution::memoization::MemoryChunkTracker,
    ) -> Vec<(u32, u64)> {
        let versions = self.chunk_versions.borrow();
        let mut result: Vec<(u32, u64)> = tracker
            .iter()
            .filter_map(|chunk| {
                if (chunk as usize) < versions.len() && versions[chunk as usize] > 0 {
                    Some((chunk, versions[chunk as usize]))
                } else {
                    None
                }
            })
            .collect();
        result.sort_by_key(|&(chunk, _)| chunk);
        result
    }
}

pub trait ByteMem: Sized {
    fn len() -> usize;
    fn from_byte(data: Vec<u8>) -> Self;
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
