use crate::structure::{types::*, module::*};
use thiserror::Error;
use std::{rc::Rc, cell::RefCell, rc::Weak};
use super::value::{Val,Externval,Ref};

#[derive(Debug, Error)]
enum RuntimeError {
    #[error("Execution Failed")]
    ExecutionFailed,
}

pub struct Results(Option<Vec<Val>>);

struct Store {
    pub funcs: Vec<FuncInst>,
    pub tables: Vec<TableInst>,
    pub mems: Vec<MemInst>,
    pub globals: Vec<GlobalInst>,
    pub elems: Vec<ElemInst>,
    pub datas: Vec<DataInst>,
}

struct ModuleInst {
    pub types: Vec<FuncType>,
    pub func_addrs: Vec<FuncAddr>,
    pub table_addrs: Vec<TableAddr>,
    pub mem_addrs: Vec<MemAddr>,
    pub global_addrs: Vec<GlobalAddr>,
    pub elem_addrs: Vec<ElemAddr>,
    pub data_addrs: Vec<DataAddr>,
    pub exports: Vec<ExportInst>,
}

pub struct FuncAddr(Rc<RefCell<FuncInst>>);
pub enum FuncInst {
    RuntimeFunc{
        ptype_: FuncType,
        module: Weak<ModuleInst>,
        code: Func,
    },
    HostFunc{
        type_: FuncType,
        host_code: Rc<dyn Fn(Vec<Val>) -> Result<Option<Val>, RuntimeError>>,
    },
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