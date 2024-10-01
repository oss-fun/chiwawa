use std::{rc::Rc, cell::RefCell};
use crate::structure::types::*;

#[derive(Clone)]
pub struct MemAddr(Rc<RefCell<MemInst>>);
pub struct MemInst {
    pub type_: MemType,
    pub data: Vec<u8>,
}