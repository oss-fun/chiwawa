use super::{
    data::DataInst, elem::ElemInst, func::FuncInst, global::GlobalInst, mem::MemInst,
    table::TableInst,
};

struct Store {
    pub funcs: Vec<FuncInst>,
    pub tables: Vec<TableInst>,
    pub mems: Vec<MemInst>,
    pub globals: Vec<GlobalInst>,
    pub elems: Vec<ElemInst>,
    pub datas: Vec<DataInst>,
}
