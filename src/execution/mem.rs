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
    pub fn i32_load(&self, arg: &Memarg, ptr: i32) -> Result<i32, RuntimeError>{
        let pos = ptr.checked_add(i32::try_from(arg.offset).ok().unwrap()).ok_or_else(|| RuntimeError::InstructionFailed)? as usize;
        let len = consts::U4::to_usize();
        let raw = &self.0.borrow().data;
        if pos + len < raw.len(){
            return Err(RuntimeError::InstantiateFailed);
        }
        let data = Vec::from(&raw[pos..pos + len]);
        let mut reader = Cursor::new(data.as_slice());
        Ok(reader.read_i32::<LittleEndian>().unwrap())
    }
}