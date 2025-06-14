use crate::error::RuntimeError;
use crate::structure::{instructions::Memarg, types::*};
use byteorder::*;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::sync::{Arc, RwLock};
use typenum::*;

#[derive(Clone, Debug)]
pub struct MemAddr(Arc<RwLock<MemInst>>);
#[derive(Debug, Serialize, Deserialize)]
pub struct MemInst {
    pub _type_: MemType,
    pub data: Vec<u8>,
}

impl MemAddr {
    pub fn new(type_: &MemType) -> MemAddr {
        let min = (type_.0.min * 65536) as usize;
        let max = type_.0.max.map(|max| max);
        MemAddr(Arc::new(RwLock::new(MemInst {
            _type_: MemType(Limits {
                min: min as u32,
                max,
            }),
            data: {
                let mut vec = Vec::with_capacity(min);
                vec.resize(min, 0);
                vec
            },
        })))
    }

    pub fn init(&self, offset: usize, init: &Vec<u8>) {
        let mut addr_self = self.0.write().expect("RwLock poisoned");
        for (index, data) in init.iter().enumerate() {
            addr_self.data[index + offset] = *data;
        }
    }
    pub fn load<T: ByteMem>(&self, arg: &Memarg, ptr: i32) -> Result<T, RuntimeError> {
        let pos = ptr
            .checked_add(i32::try_from(arg.offset).ok().unwrap())
            .ok_or_else(|| RuntimeError::InstructionFailed)? as usize;
        let len = <T>::len();
        let raw = &self.0.read().expect("RwLock poisoned").data;
        let data = Vec::from(&raw[pos..pos + len]);
        Ok(<T>::from_byte(data))
    }
    pub fn store<T: ByteMem>(&self, arg: &Memarg, ptr: i32, data: T) -> Result<(), RuntimeError> {
        let pos = ptr
            .checked_add(i32::try_from(arg.offset).ok().unwrap())
            .ok_or_else(|| RuntimeError::InstructionFailed)? as usize;
        let buf = <T>::to_byte(data);
        let mut raw = self.0.write().expect("RwLock poisoned");
        for (i, x) in buf.into_iter().enumerate() {
            raw.data[pos + i] = x;
        }

        Ok(())
    }

    pub fn mem_size(&self) -> i32 {
        (self.0.read().expect("RwLock poisoned").data.len() / 65536) as i32
    }

    pub fn mem_grow(&self, size: i32) -> i32 {
        let prev_size = self.mem_size();
        let new = prev_size + size;
        if new > 65536 {
            -1
        } else {
            self.0
                .write()
                .expect("RwLock poisoned")
                .data
                .resize(new as usize * 65536, 0);
            prev_size
        }
    }

    pub fn get_data(&self) -> Result<Vec<u8>, RuntimeError> {
        let guard = self
            .0
            .read()
            .map_err(|_| RuntimeError::ExecutionFailed("Memory RwLock poisoned"))?;
        Ok(guard.data.clone())
    }

    pub fn set_data(&self, data: Vec<u8>) -> Result<(), RuntimeError> {
        let mut guard = self
            .0
            .write()
            .map_err(|_| RuntimeError::ExecutionFailed("Memory RwLock poisoned"))?;
        guard.data = data;
        Ok(())
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
