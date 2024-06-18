use crate::types::*;
use crate::instructions::*;

pub struct Func {
    type_: TypeIdx,
    locals: Vec<ValueType>,
    body: Expr,
}

pub struct Table {
    type_: TableType,
}

pub struct Mem {
    type_: MemType,
}

pub struct Module {
    name: String,
    types: Vec<FuncType>,
    funcs: Vec<Func>,
    tables: Vec<Table>,
    mems: Vec<Mem>,
}