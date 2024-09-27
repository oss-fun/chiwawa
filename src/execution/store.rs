use super::{func::FuncInst, table::TableInst, mem::MemInst, global::GlobalInst, elem::ElemInst, data::DataInst};

struct Store {
    pub funcs: Vec<FuncInst>,
    pub tables: Vec<TableInst>,
    pub mems: Vec<MemInst>,
    pub globals: Vec<GlobalInst>,
    pub elems: Vec<ElemInst>,
    pub datas: Vec<DataInst>,
}