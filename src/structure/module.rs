use crate::execution::stack::ProcessedInstr;
use crate::structure::instructions::*;
use crate::structure::types::*;

#[derive(Clone, Debug)]
pub struct Func {
    pub type_: TypeIdx,
    pub locals: Vec<(u32, ValueType)>,
    pub body: Vec<ProcessedInstr>,
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

pub enum DataMode {
    Passive,
    Active,
}
pub struct Start {
    pub func: FuncIdx,
}

pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}

#[derive(PartialEq, Debug)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
    WasiFunc(WasiFuncType),
}

#[derive(PartialEq, Debug, Clone)]
pub enum WasiFuncType {
    ProcExit,
    FdWrite,
    FdRead,
    RandomGet,
    FdPrestatGet,
    FdPrestatDirName,
    FdClose,
    EnvironGet,
    EnvironSizesGet,
    ArgsGet,
    ArgsSizesGet,
    ClockTimeGet,
    ClockResGet,
    SchedYield,
    FdFdstatGet,
    PathOpen,
    FdSeek,
    FdTell,
}

impl WasiFuncType {
    /// WASI関数の期待される関数型を返す
    pub fn expected_func_type(&self) -> FuncType {
        match self {
            WasiFuncType::ProcExit => FuncType {
                params: vec![ValueType::NumType(NumType::I32)],
                results: vec![],
            },
            WasiFuncType::FdWrite => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::FdRead => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::RandomGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::FdPrestatGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::FdPrestatDirName => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::FdClose => FuncType {
                params: vec![ValueType::NumType(NumType::I32)],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::EnvironGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::EnvironSizesGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::ArgsGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::ArgsSizesGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::ClockTimeGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I64),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::ClockResGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::SchedYield => FuncType {
                params: vec![],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::FdFdstatGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32),
                    ValueType::NumType(NumType::I32),
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::PathOpen => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd (base directory)
                    ValueType::NumType(NumType::I32), // dirflags
                    ValueType::NumType(NumType::I32), // path ptr
                    ValueType::NumType(NumType::I32), // path len
                    ValueType::NumType(NumType::I32), // oflags
                    ValueType::NumType(NumType::I64), // fs_rights_base
                    ValueType::NumType(NumType::I64), // fs_rights_inheriting
                    ValueType::NumType(NumType::I32), // fdflags
                    ValueType::NumType(NumType::I32), // opened_fd ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)],
            },
            WasiFuncType::FdSeek => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I64), // offset
                    ValueType::NumType(NumType::I32), // whence
                ],
                results: vec![ValueType::NumType(NumType::I64)], // Returns new file position
            },
            WasiFuncType::FdTell => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // offset_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
        }
    }

    pub fn to_func_type(&self) -> FuncType {
        self.expected_func_type()
    }
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
    _name: String,
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
    pub fn new(name: &str) -> Self {
        Module {
            _name: name.to_string(),
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
