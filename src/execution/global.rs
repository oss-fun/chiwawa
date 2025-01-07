use std::{rc::Rc, cell::RefCell, cell::Ref};
use crate::structure::types::*;
use super::value::Val;

#[derive(Clone)]
pub struct GlobalAddr(Rc<RefCell<GlobalInst>>);
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
}