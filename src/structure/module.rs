use crate::structure::types::*;
use crate::structure::instructions::*;
use crate::execution::stack::*;

#[derive(Clone, Debug)]
pub struct Func {
    pub type_: TypeIdx,
    pub locals: Vec<(u32,ValueType)>,
    pub body: Vec<ProcessedInstr>,
    pub fixups: Vec<FixupInfo>,
}

#[derive(Clone)]
pub struct Table {
    pub type_: TableType,
}

#[derive(Clone)]
pub struct Mem {
    pub type_: MemType,
}

#[derive(Clone)]
pub struct Global {
    pub type_: GlobalType,
    pub init: Expr,
}

pub struct Elem {
    pub type_: RefType,
    pub init: Option<Vec<Expr>>,
    pub idxes: Option<Vec<FuncIdx>>,
    pub mode: ElemMode,
    pub table_idx: Option<TableIdx>,
    pub offset: Option<Expr>,
}

#[derive(Debug, PartialEq)]
pub enum ElemMode {
    Passive,
    Active,
    Declarative,
}

pub struct Data {
    pub init: Vec<Byte>,
    pub mode: DataMode,
    pub memory: Option<MemIdx>,
    pub offset: Option<Expr>,
}

pub enum DataMode{
    Passive,
    Active,
}
pub struct Start {
    pub func: FuncIdx
}

pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}

#[derive(PartialEq)]
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

#[derive(Clone)]
pub enum ExportDesc {
    Func(FuncIdx),
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
}

pub struct Module {
    name: String,
    pub types: Vec<FuncType>,
    pub funcs: Vec<Func>,
    pub tables: Vec<Table>,
    pub mems: Vec<Mem>,
    pub globals: Vec<Global>,
    pub elems: Vec<Elem>,
    pub datas: Vec<Data>,
    pub start: Option<Start>,
    pub imports: Vec<Import>,
    pub num_imported_funcs: usize,
    pub code_index: usize,
    pub exports: Vec<Export>,
}

impl Module {
    pub fn new(name: &str) -> Self{
        Module{
            name: name.to_string(),
            types: Vec::new(),
            funcs: Vec::new(),
            tables: Vec::new(),
            mems: Vec::new(),
            globals: Vec::new(),
            elems: Vec::new(),
            datas: Vec::new(),
            start: None,
            imports: Vec::new(),
            num_imported_funcs: 0,
            code_index: 0,
            exports: Vec::new(),
        }
    }
}
