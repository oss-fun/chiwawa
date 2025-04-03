use std::{rc::Rc, cell::RefCell, cell::Ref};
use crate::structure::types::*;
use super::value::Val;
use crate::error::RuntimeError;

#[derive(Clone, Debug)] // Added Debug
pub struct GlobalAddr(Rc<RefCell<GlobalInst>>);
#[derive(Debug)] // Added Debug
pub struct GlobalInst {
    pub type_: GlobalType,
    pub value: Val,
}

impl GlobalAddr {
    pub fn new(type_: &GlobalType, value: Val) -> GlobalAddr{
        GlobalAddr(Rc::new(RefCell::new(
            GlobalInst{
                type_: type_.clone(),
                value: value
            }
        )))
    }
    fn inst(&self) -> Ref<GlobalInst> {
        self.0.borrow()
    }
    pub fn get(&self) -> Val {
        self.0.borrow().value.clone()
    }
    pub fn set(&self, value: Val) -> Result<(), RuntimeError>{
        let mut self_inst = self.0.borrow_mut();
        if self_inst.value.val_type() == value.val_type() {
            self_inst.value = value;
            return Ok(());
        }
        Err(RuntimeError::InstructionFailed)
    }
}
