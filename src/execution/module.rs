use crate::structure::types::*;
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
