use std::{rc::Rc, cell::RefCell};
use crate::structure::types::*;
use super::value::Val;

#[derive(Clone)]
pub struct GlobalAddr(Rc<RefCell<GlobalInst>>);
pub struct GlobalInst {
    pub type_: GlobalType,
    pub value: Val,
}

impl GlobalAddr {
    pub fn new(type_: GlobalType, value: Val) -> GlobalAddr{
        GlobalAddr(Rc::new(RefCell::new(
            GlobalInst{type_, value}
        )))
    }
}