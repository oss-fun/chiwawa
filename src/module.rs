use crate::types::*;
use crate::instructions::*;

pub struct Func {
    pub type_: TypeIdx,
    pub locals: Vec<ValueType>,
    pub body: Expr,
}

pub struct Table {
    pub type_: TableType,
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

pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

pub struct Export {
    pub name: Name,
    pub desc: ExportDesc,
}

pub enum ExportDesc {
    Func(FuncIdx),
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
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
    imports: Vec<Import>,
    exports: Vec<Export>,
}