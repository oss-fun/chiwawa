use std::rc::Rc;
use crate::structure::{types::*, module::*};
use crate::error::RuntimeError;
use super::value::Val;
use super::{func::FuncAddr, table::TableAddr, mem::MemAddr, global::GlobalAddr, elem::ElemAddr, data::DataAddr, export::ExportInst};

pub struct Results(Option<Vec<Val>>);

pub struct ModuleInst {
    pub types: Vec<FuncType>,
    pub func_addrs: Vec<FuncAddr>,
    pub table_addrs: Vec<TableAddr>,
    pub mem_addrs: Vec<MemAddr>,
    pub global_addrs: Vec<GlobalAddr>,
    pub elem_addrs: Vec<ElemAddr>,
    pub data_addrs: Vec<DataAddr>,
    pub exports: Vec<ExportInst>,
}

impl ModuleInst {
    pub fn new(module: Module) -> Result<Rc<ModuleInst>, RuntimeError>{
        let mut module_inst = ModuleInst {
            types: module.types.clone(),
            func_addrs: Vec::new(),
            table_addrs: Vec::new(),
            mem_addrs: Vec::new(),
            global_addrs:Vec::new(),
            elem_addrs:Vec::new(),
            data_addrs:Vec::new(),
            exports:Vec::new(),
        };    
        Ok(Rc::new(module_inst))
    }
}