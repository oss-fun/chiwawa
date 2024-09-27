use std::{rc::Rc, cell::RefCell};
use crate::structure::types::*;
use super::value::Val;

pub struct GlobalAddr(Rc<RefCell<GlobalInst>>);
pub struct GlobalInst {
    pub type_: GlobalType,
    pub value: Val,
}