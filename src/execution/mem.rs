use std::{rc::Rc, cell::RefCell};
use crate::structure::{types::*,instructions::Memarg};
use crate::error::RuntimeError;
use typenum::*;
use std::io::Cursor;
use byteorder::*;

#[derive(Clone)]
pub struct MemAddr(Rc<RefCell<MemInst>>);
pub struct MemInst {
    pub type_: MemType,
    pub data: Vec<u8>,
}

impl MemAddr {
    pub fn new(type_: &MemType) -> MemAddr{
        MemAddr(Rc::new(RefCell::new(
            MemInst{
                type_: type_.clone(),
                data: Vec::new(),
            }
        )))
    }

    pub fn init(&self, offset: usize, init: &Vec<u8>){
        let addr_self = &mut self.0.borrow_mut();
        for(index, data) in init.iter().enumerate(){
            addr_self.data[index + offset] = *data;
        }
    }
    pub fn load<T: ByteMem>(&self, arg: &Memarg, ptr: i32) -> Result<T, RuntimeError>{
        let pos = ptr.checked_add(i32::try_from(arg.offset).ok().unwrap()).ok_or_else(|| RuntimeError::InstructionFailed)? as usize;
        let len =  <T>::len();
        let raw = &self.0.borrow().data;
    
        if pos + len < raw.len(){
            return Err(RuntimeError::InstructionFailed);
        }

        let data = Vec::from(&raw[pos..pos + len]);
        Ok(<T>::from_byte(data))
    }
    pub fn store<T:ByteMem>(&self, arg: &Memarg, ptr: i32, data: T)-> Result<(), RuntimeError>{
        let pos = ptr.checked_add(i32::try_from(arg.offset).ok().unwrap()).ok_or_else(|| RuntimeError::InstructionFailed)? as usize;
        let data = <T>::to_byte(data);
        let len =  <T>::len();
        let mut raw = &mut self.0.borrow_mut().data;
    
        if pos + len < raw.len(){
            return Err(RuntimeError::InstructionFailed);
        }
        for (i, x) in data.into_iter().enumerate(){
            raw[pos + i] = x;
        }

        Ok(())
    }
}

pub trait ByteMem: Sized{
    fn len() -> usize;
    fn from_byte(data: Vec<u8>) -> Self;
    fn to_byte(self) -> Vec<u8>;
}

impl ByteMem for i8 {
    fn len() -> usize{
        consts::U1::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> i8{
        let mut reader = Cursor::new(data.as_slice());
        reader.read_i8().unwrap()
    }
    fn to_byte(self) -> Vec<u8>{
        let mut buf: Vec<u8> =  Vec::with_capacity(Self::len());
        buf.write_i8(self).unwrap();
        buf[..].to_vec()
    }
}
impl ByteMem for u8 {
    fn len() -> usize{
        consts::U1::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> u8{
        let mut reader = Cursor::new(data.as_slice());
        reader.read_u8().unwrap()
    }
    fn to_byte(self) -> Vec<u8>{
        let mut buf: Vec<u8> =  Vec::with_capacity(Self::len());
        buf.write_u8(self).unwrap();
        buf[..].to_vec()
    }
}

impl ByteMem for i32 {
    fn len() -> usize{
        consts::U4::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> i32{
        let mut reader = Cursor::new(data.as_slice());
        reader.read_i32::<LittleEndian>().unwrap()
    }
    fn to_byte(self) -> Vec<u8>{
        let mut buf: Vec<u8> =  Vec::with_capacity(Self::len());
        buf.write_i32::<LittleEndian>(self).unwrap();
        buf[..].to_vec()
    }
}

impl ByteMem for i64 {
    fn len() -> usize{
        consts::U8::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> i64{
        let mut reader = Cursor::new(data.as_slice());
        reader.read_i64::<LittleEndian>().unwrap()
    }
    fn to_byte(self) -> Vec<u8>{
        let mut buf: Vec<u8> =  Vec::with_capacity(Self::len());
        buf.write_i64::<LittleEndian>(self).unwrap();
        buf[..].to_vec()
    }
}

impl ByteMem for f32 {
    fn len() -> usize{
        consts::U4::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> f32{
        let mut reader = Cursor::new(data.as_slice());
        reader.read_f32::<LittleEndian>().unwrap()
    }
    fn to_byte(self) -> Vec<u8>{
        let mut buf: Vec<u8> =  Vec::with_capacity(Self::len());
        buf.write_f32::<LittleEndian>(self).unwrap();
        buf[..].to_vec()
    }
}

impl ByteMem for f64 {
    fn len() -> usize{
        consts::U8::to_usize()
    }
    fn from_byte(data: Vec<u8>) -> f64{
        let mut reader = Cursor::new(data.as_slice());
        reader.read_f64::<LittleEndian>().unwrap()
    }
    fn to_byte(self) -> Vec<u8>{
        let mut buf: Vec<u8> =  Vec::with_capacity(Self::len());
        buf.write_f64::<LittleEndian>(self).unwrap();
        buf[..].to_vec()
    }
}