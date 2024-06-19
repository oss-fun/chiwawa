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

pub struct Global {
    type_: GlobalType,
    init: Expr,
}

pub struct Elem {
    type_: RefType,
    init: Vec<Expr>,
    mode: ElemMode,
    tableIdx: Option<TableIdx>,
    offset: Option<Expr>,
}
pub enum ElemMode {
    Passive,
    Active,
    Declarative,
}

pub struct Data {
    init: Vec<Byte>,
    mode: DataMode,
    memory: Option<MemIdx>,
    offset: Option<Expr>,
}

pub enum DataMode{
    Passive,
    Active,
}
pub struct Start {
    func: FuncIdx
}

pub struct Module {
    name: String,
    types: Vec<FuncType>,
    funcs: Vec<Func>,
    tables: Vec<Table>,
    mems: Vec<Mem>,
    globals: Vec<Global>,
    elems: Vec<Elem>,
    datas: Vec<Data>,
    start: Option<Start>,
}