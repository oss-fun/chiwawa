use crate::structure::types::*;
use std::{rc::Rc, cell::RefCell};
use super::value::{Val,Externval,Ref};


struct ModuleInst {
    pub types: Vec<FuncType>,
    pub table_addrs: Vec<TableAddr>,
    pub mem_addrs: Vec<MemAddr>,
    pub global_addrs: Vec<GlobalAddr>,
    pub elem_addrs: Vec<ElemAddr>,
    pub data_addrs: Vec<DataAddr>,
    pub exports: Vec<ExportInst>,
}

pub struct TableAddr(Rc<RefCell<TableInst>>);
struct TableInst {
    pub type_: TableType,
    pub elem: Vec<Ref>,
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
struct ElemAddr(Rc<RefCell<ElemInst>>);
struct ElemInst {
    pub type_: RefType,
    pub elem: Vec<Ref>,
    
}

struct DataAddr(Rc<RefCell<DataInst>>);
struct DataInst {
    pub data: Vec<u8>,    
}

struct ExportInst {
    pub name: String,
    pub value: Externval,
}