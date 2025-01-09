use std::{rc::Rc, cell::RefCell};
use crate::structure::{types::*,instructions::Memarg};
use crate::error::RuntimeError;

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
    pub fn Load(&self, arg: &Memarg, ptr: i32) -> Result<i32, RuntimeError>{
        let pos = ptr.checked_add(i32::try_from(arg.offset).ok().unwrap());
        Ok(1)
    }
}