//! WebAssembly module structure definitions.
//!
//! This module defines the structure of a parsed WebAssembly module, including
//! functions, tables, memories, globals, and imports/exports.
//!
//! ## Module Components
//!
//! A WebAssembly module consists of:
//! - **Types**: Function signatures shared across the module
//! - **Functions**: Code with local variables and a type reference
//! - **Tables**: Collections of function references for indirect calls
//! - **Memories**: Linear memory regions
//! - **Globals**: Global variables
//! - **Elements**: Table initialization data
//! - **Data**: Memory initialization data
//! - **Imports/Exports**: Module interface

use crate::execution::regs::Reg;
use crate::execution::vm::ProcessedInstr;
use crate::structure::instructions::*;
use crate::structure::types::*;
use std::rc::Rc;

/// Function definition within a module.
///
/// Contains the type signature, local variables, and preprocessed instruction body.
#[derive(Clone, Debug)]
pub struct Func {
    pub type_: TypeIdx,
    pub locals: Vec<(u32, ValueType)>,
    pub body: Rc<Vec<ProcessedInstr>>,
    pub reg_allocation: Option<crate::execution::regs::RegAllocation>,
    pub result_reg: Option<Reg>, // Register for return value (register mode only)
}

/// Table definition.
#[derive(Clone)]
pub struct Table {
    pub type_: TableType,
}

/// Memory definition.
#[derive(Clone)]
pub struct Mem {
    pub type_: MemType,
}

/// Global variable definition.
#[derive(Clone)]
pub struct Global {
    pub type_: GlobalType,
    pub init: Expr,
}

/// Element segment for table initialization.
pub struct Elem {
    pub type_: RefType,
    pub init: Option<Vec<Expr>>,
    pub idxes: Option<Vec<FuncIdx>>,
    pub mode: ElemMode,
    pub table_idx: Option<TableIdx>,
    pub offset: Option<Expr>,
}

/// Element segment mode.
#[derive(Debug, PartialEq)]
pub enum ElemMode {
    Passive,
    Active,
    Declarative,
}

/// Data segment for memory initialization.
pub struct Data {
    pub init: Vec<Byte>,
    pub mode: DataMode,
    pub memory: Option<MemIdx>,
    pub offset: Option<Expr>,
}

/// Data segment mode.
#[derive(Debug, PartialEq)]
pub enum DataMode {
    Passive,
    Active,
}

/// Start function specification.
pub struct Start {
    pub func: FuncIdx,
}

/// Import declaration.
pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}

/// Import descriptor specifying what is being imported.
#[derive(PartialEq, Debug)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
    WasiFunc(WasiFuncType),
}

/// WASI function types for passthrough implementation.
#[derive(PartialEq, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
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
    FdSync,
    FdFilestatGet,
    FdReaddir,
    FdPread,
    FdDatasync,
    FdFdstatSetFlags,
    FdFilestatSetSize,
    FdPwrite,
    PathCreateDirectory,
    PathFilestatGet,
    PathFilestatSetTimes,
    PathReadlink,
    PathRemoveDirectory,
    PathUnlinkFile,
    PollOneoff,
    ProcRaise,
    FdAdvise,
    FdAllocate,
    FdFdstatSetRights,
    FdRenumber,
    FdFilestatSetTimes,
    PathLink,
    PathRename,
    PathSymlink,
    SockAccept,
    SockRecv,
    SockSend,
    SockShutdown,
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
                    ValueType::NumType(NumType::I32), // newoffset_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns errno
            },
            WasiFuncType::FdTell => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // offset_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdSync => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdFilestatGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // filestat_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdReaddir => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // buf_ptr
                    ValueType::NumType(NumType::I32), // buf_len
                    ValueType::NumType(NumType::I64), // cookie
                    ValueType::NumType(NumType::I32), // buf_used_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdPread => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // iovs_ptr
                    ValueType::NumType(NumType::I32), // iovs_len
                    ValueType::NumType(NumType::I64), // offset
                    ValueType::NumType(NumType::I32), // nread_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdDatasync => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdFdstatSetFlags => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // flags
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdFilestatSetSize => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I64), // size
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdPwrite => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // iovs_ptr
                    ValueType::NumType(NumType::I32), // iovs_len
                    ValueType::NumType(NumType::I64), // offset
                    ValueType::NumType(NumType::I32), // nwritten_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathCreateDirectory => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // path_ptr
                    ValueType::NumType(NumType::I32), // path_len
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathFilestatGet => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // flags
                    ValueType::NumType(NumType::I32), // path_ptr
                    ValueType::NumType(NumType::I32), // path_len
                    ValueType::NumType(NumType::I32), // filestat_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathFilestatSetTimes => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // flags
                    ValueType::NumType(NumType::I32), // path_ptr
                    ValueType::NumType(NumType::I32), // path_len
                    ValueType::NumType(NumType::I64), // atim
                    ValueType::NumType(NumType::I64), // mtim
                    ValueType::NumType(NumType::I32), // fst_flags
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathReadlink => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // path_ptr
                    ValueType::NumType(NumType::I32), // path_len
                    ValueType::NumType(NumType::I32), // buf_ptr
                    ValueType::NumType(NumType::I32), // buf_len
                    ValueType::NumType(NumType::I32), // buf_used_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathRemoveDirectory => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // path_ptr
                    ValueType::NumType(NumType::I32), // path_len
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathUnlinkFile => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // path_ptr
                    ValueType::NumType(NumType::I32), // path_len
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PollOneoff => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // in_ptr
                    ValueType::NumType(NumType::I32), // out_ptr
                    ValueType::NumType(NumType::I32), // nsubscriptions
                    ValueType::NumType(NumType::I32), // nevents_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::ProcRaise => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // signal
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdAdvise => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I64), // offset
                    ValueType::NumType(NumType::I64), // len
                    ValueType::NumType(NumType::I32), // advice
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdAllocate => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I64), // offset
                    ValueType::NumType(NumType::I64), // len
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdFdstatSetRights => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I64), // fs_rights_base
                    ValueType::NumType(NumType::I64), // fs_rights_inheriting
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdRenumber => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // to
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::FdFilestatSetTimes => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I64), // atim
                    ValueType::NumType(NumType::I64), // mtim
                    ValueType::NumType(NumType::I32), // fst_flags
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathLink => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // old_fd
                    ValueType::NumType(NumType::I32), // old_flags
                    ValueType::NumType(NumType::I32), // old_path_ptr
                    ValueType::NumType(NumType::I32), // old_path_len
                    ValueType::NumType(NumType::I32), // new_fd
                    ValueType::NumType(NumType::I32), // new_path_ptr
                    ValueType::NumType(NumType::I32), // new_path_len
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathRename => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // old_fd
                    ValueType::NumType(NumType::I32), // old_path_ptr
                    ValueType::NumType(NumType::I32), // old_path_len
                    ValueType::NumType(NumType::I32), // new_fd
                    ValueType::NumType(NumType::I32), // new_path_ptr
                    ValueType::NumType(NumType::I32), // new_path_len
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::PathSymlink => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // old_path_ptr
                    ValueType::NumType(NumType::I32), // old_path_len
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // new_path_ptr
                    ValueType::NumType(NumType::I32), // new_path_len
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::SockAccept => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // flags
                    ValueType::NumType(NumType::I32), // fd_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::SockRecv => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // ri_data_ptr
                    ValueType::NumType(NumType::I32), // ri_data_len
                    ValueType::NumType(NumType::I32), // ri_flags
                    ValueType::NumType(NumType::I32), // ro_datalen_ptr
                    ValueType::NumType(NumType::I32), // ro_flags_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::SockSend => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // si_data_ptr
                    ValueType::NumType(NumType::I32), // si_data_len
                    ValueType::NumType(NumType::I32), // si_flags
                    ValueType::NumType(NumType::I32), // so_datalen_ptr
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
            WasiFuncType::SockShutdown => FuncType {
                params: vec![
                    ValueType::NumType(NumType::I32), // fd
                    ValueType::NumType(NumType::I32), // how
                ],
                results: vec![ValueType::NumType(NumType::I32)], // Returns error code
            },
        }
    }

    pub fn to_func_type(&self) -> FuncType {
        self.expected_func_type()
    }
}

/// Export declaration.
pub struct Export {
    pub name: Name,
    pub desc: ExportDesc,
}

/// Export descriptor specifying what is being exported.
#[derive(Clone)]
pub enum ExportDesc {
    Func(FuncIdx),
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
}

/// A parsed WebAssembly module.
///
/// Contains all sections of a WebAssembly module after parsing and preprocessing.
pub struct Module {
    _name: String,
    /// Function type signatures.
    pub types: Rc<Vec<FuncType>>,
    /// Function definitions (including imported functions).
    pub funcs: Vec<Func>,
    /// Table definitions.
    pub tables: Vec<Table>,
    /// Memory definitions.
    pub mems: Vec<Mem>,
    /// Global variable definitions.
    pub globals: Vec<Global>,
    /// Element segments.
    pub elems: Vec<Elem>,
    /// Data segments.
    pub datas: Vec<Data>,
    /// Optional start function.
    pub start: Option<Start>,
    /// Import declarations.
    pub imports: Vec<Import>,
    /// Number of imported functions.
    pub num_imported_funcs: usize,
    /// Current code section index during parsing.
    pub code_index: usize,
    /// Export declarations.
    pub exports: Vec<Export>,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Module {
            _name: name.to_string(),
            types: Rc::new(Vec::new()),
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
