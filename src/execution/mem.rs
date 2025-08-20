use crate::error::RuntimeError;
use crate::structure::{instructions::Memarg, types::*};
use byteorder::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, RwLock};
use typenum::*;

#[derive(Clone, Debug)]
pub struct MemAddr {
    mem_inst: Arc<RwLock<MemInst>>,
    page_versions: Arc<RwLock<HashMap<u32, u64>>>,
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
            mem_inst: Arc::new(RwLock::new(MemInst {
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
            page_versions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn update_page_versions(&self, start_pos: usize, len: usize) {
        if len > 0 {
            let start_page = start_pos / 65536;
            let end_page = (start_pos + len - 1) / 65536;
            let mut versions = self.page_versions.write().expect("RwLock poisoned");
            for page in start_page..=end_page {
                *versions.entry(page as u32).or_insert(0) += 1;
            }
        }
    }

    pub fn init(&self, offset: usize, init: &Vec<u8>) {
        let mut addr_self = self.mem_inst.write().expect("RwLock poisoned");
        for (index, data) in init.iter().enumerate() {
            addr_self.data[index + offset] = *data;
        }
        self.update_page_versions(offset, init.len());
    }
    pub fn load<T: ByteMem>(&self, arg: &Memarg, ptr: i32) -> Result<T, RuntimeError> {
        let pos = ptr
            .checked_add(i32::try_from(arg.offset).ok().unwrap())
            .ok_or_else(|| RuntimeError::InstructionFailed)? as usize;
        let len = <T>::len();
        let raw = &self.mem_inst.read().expect("RwLock poisoned").data;
        let data = Vec::from(&raw[pos..pos + len]);
        Ok(<T>::from_byte(data))
    }
    pub fn store<T: ByteMem>(&self, arg: &Memarg, ptr: i32, data: T) -> Result<(), RuntimeError> {
        let pos = ptr
            .checked_add(i32::try_from(arg.offset).ok().unwrap())
            .ok_or_else(|| RuntimeError::InstructionFailed)? as usize;
        let buf = <T>::to_byte(data);
        let mut raw = self.mem_inst.write().expect("RwLock poisoned");
        for (i, x) in buf.into_iter().enumerate() {
            raw.data[pos + i] = x;
        }
        let len = <T>::len();
        drop(raw);
        self.update_page_versions(pos, len);

        Ok(())
    }

    pub fn mem_size(&self) -> i32 {
        (self.mem_inst.read().expect("RwLock poisoned").data.len() / 65536) as i32
    }

    pub fn mem_grow(&self, size: i32) -> i32 {
        let prev_size = self.mem_size();
        let new = prev_size + size;

        let max_pages = {
            let guard = self.mem_inst.read().expect("RwLock poisoned");
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
                .write()
                .expect("RwLock poisoned")
                .data
                .resize(new as usize * 65536, 0);
            prev_size
        }
    }

    pub fn get_data(&self) -> Result<Vec<u8>, RuntimeError> {
        let guard = self
            .mem_inst
            .read()
            .map_err(|_| RuntimeError::ExecutionFailed("Memory RwLock poisoned"))?;
        Ok(guard.data.clone())
    }

    /// Store multiple bytes at once (bulk operation)
    pub fn store_bytes(&self, ptr: i32, data: &[u8]) -> Result<(), RuntimeError> {
        let pos = ptr as usize;
        let mut raw = self.mem_inst.write().expect("RwLock poisoned");

        // Bounds check
        if pos.checked_add(data.len()).unwrap_or(usize::MAX) > raw.data.len() {
            return Err(RuntimeError::InstructionFailed);
        }

        // Bulk copy
        raw.data[pos..pos + data.len()].copy_from_slice(data);

        drop(raw);
        self.update_page_versions(pos, data.len());

        Ok(())
    }

    pub fn set_data(&self, data: Vec<u8>) -> Result<(), RuntimeError> {
        let mut guard = self
            .mem_inst
            .write()
            .map_err(|_| RuntimeError::ExecutionFailed("Memory RwLock poisoned"))?;
        guard.data = data;
        Ok(())
    }

    pub fn memory_copy(&self, dest: i32, src: i32, len: i32) -> Result<(), RuntimeError> {
        let dest_pos = dest as usize;
        let src_pos = src as usize;
        let len_usize = len as usize;
        let mut raw = self.mem_inst.write().expect("RwLock poisoned");

        if len_usize > 0 {
            raw.data.copy_within(src_pos..src_pos + len_usize, dest_pos);

            drop(raw);
            self.update_page_versions(dest_pos, len_usize);
        }

        Ok(())
    }

    pub fn memory_fill(&self, dest: i32, val: u8, len: i32) -> Result<(), RuntimeError> {
        let dest_pos = dest as usize;
        let len_usize = len as usize;
        let mut raw = self.mem_inst.write().expect("RwLock poisoned");

        // Bounds check
        if dest_pos.checked_add(len_usize).unwrap_or(usize::MAX) > raw.data.len() {
            return Err(RuntimeError::InstructionFailed);
        }

        if len_usize > 0 {
            for i in 0..len_usize {
                raw.data[dest_pos + i] = val;
            }

            drop(raw);
            self.update_page_versions(dest_pos, len_usize);
        }

        Ok(())
    }

    pub fn get_memory_direct_access(&self) -> std::sync::RwLockReadGuard<MemInst> {
        self.mem_inst.read().expect("RwLock poisoned")
    }

    pub fn get_all_page_versions(&self) -> Vec<(u32, u64)> {
        let versions = self.page_versions.read().expect("RwLock poisoned");
        let mut result: Vec<(u32, u64)> = versions
            .iter()
            .map(|(&page, &version)| (page, version))
            .collect();
        result.sort_by_key(|&(page, _)| page);
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
        let mut buf: Vec<u8> = Vec::with_capacity(Self::len());
        buf.write_i8(self).unwrap();
        buf[..].to_vec()
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
        let mut buf: Vec<u8> = Vec::with_capacity(Self::len());
        buf.write_u8(self).unwrap();
        buf[..].to_vec()
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
        let mut buf: Vec<u8> = Vec::with_capacity(Self::len());
        buf.write_i16::<LittleEndian>(self).unwrap();
        buf[..].to_vec()
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
        let mut buf: Vec<u8> = Vec::with_capacity(Self::len());
        buf.write_u16::<LittleEndian>(self).unwrap();
        buf[..].to_vec()
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
        let mut buf: Vec<u8> = Vec::with_capacity(Self::len());
        buf.write_i32::<LittleEndian>(self).unwrap();
        buf[..].to_vec()
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
        let mut buf: Vec<u8> = Vec::with_capacity(Self::len());
        buf.write_u32::<LittleEndian>(self).unwrap();
        buf[..].to_vec()
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
        let mut buf: Vec<u8> = Vec::with_capacity(Self::len());
        buf.write_i64::<LittleEndian>(self).unwrap();
        buf[..].to_vec()
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
        let mut buf: Vec<u8> = Vec::with_capacity(Self::len());
        buf.write_f32::<LittleEndian>(self).unwrap();
        buf[..].to_vec()
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
        let mut buf: Vec<u8> = Vec::with_capacity(Self::len());
        buf.write_f64::<LittleEndian>(self).unwrap();
        buf[..].to_vec()
    }
}
