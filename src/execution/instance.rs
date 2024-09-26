use crate::structure::types::*;
use std::{rc::Rc, cell::RefCell};
use super::{value::Val};


struct ModuleInst {
    pub types: Vec<FuncType>,
    pub mem_addrs: Vec<MemAddr>,
    pub global_addrs: Vec<GlobalAddr>,
    pub data_addrs: Vec<DataAddr>,
}

pub struct MemAddr(Rc<RefCell<MemInst>>);
struct MemInst {
    pub type_: MemType,
    pub data: Vec<u8>,
}

pub struct GlobalAddr(Rc<RefCell<GlobalInst>>);
struct GlobalInst {
    pub type_: GlobalType,
    pub value: Val,
}

struct DataAddr(Rc<RefCell<DataInst>>);
struct DataInst {
    pub data: Vec<u8>,    
}