use crate::structure::types::*;
use std::{rc::Rc, cell::RefCell};

struct ModuleInst {
    pub types: Vec<FuncType>,
    pub mem_addrs: Vec<MemAddr>,
}

struct MemAddr(Rc<RefCell<MemInst>>);
struct MemInst {
    pub type_: MemType,
    pub data: Vec<u8>,
}