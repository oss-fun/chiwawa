//! WebAssembly bytecode parsing and instruction preprocessing.
//!
//! This module transforms WebAssembly binary modules into an optimized intermediate representation suitable for efficient interpretation.
//!
//! ## Preprocessing Pipeline
//!
//! Each function body goes through three phases:
//!
//! ### Phase 1: Decode
//! Parse WebAssembly instructions using `wasmparser`, assign operands to typed registers, 
//! apply operand folding, and record where each block ends.
//!
//! ### Phase 2: Branch Resolution
//! Resolve every branch target (`br`, `br_if`, `br_table`, `if`, `else`) to an absolute program counter in one forward walk over the stream.
//!
//! ### Phase 3: Compaction
//! Strip no-op instructions and remap every branch target to the new indices.
//!
//! ## Register-Based Execution
//!
//! Instructions are converted to use a register-based model where operands are pre-allocated registers rather than implicit stack positions. 
//! This enables more efficient execution by avoiding redundant stack operations.

use std::fs::File;
use std::io::Read;
use wasmparser::{
    ExternalKind, FunctionBody, Parser, Payload::*, SectionLimited, TypeRef, ValType,
};

use crate::error::{ParserError, RuntimeError};
use crate::execution::handlers::*;
use crate::execution::ir::*;
use crate::execution::regs::{Reg, RegAllocation, RegAllocator, RegAllocatorState};
use crate::shared::Shared;
use crate::structure::{instructions::*, module::*, types::*};
use rustc_hash::FxHashMap;
use std::sync::LazyLock;

#[cfg(feature = "call_graph")]
use crate::instrument::call_graph::{CallGraph, CallGraphBuilder};

/// Builds a function's `wide_consts`. Equal values share a slot, 
/// so an instruction only carries the slot number.
#[derive(Default)]
struct ConstPool {
    consts: Vec<u64>,
    /// Slot already holding each value.
    seen: FxHashMap<u64, u32>,
}

impl ConstPool {
    fn add(&mut self, bits: u64) -> u32 {
        if let Some(slot) = self.seen.get(&bits) {
            return *slot;
        }
        let slot = self.consts.len() as u32;
        self.consts.push(bits);
        self.seen.insert(bits, slot);
        slot
    }

    fn add_i64(&mut self, v: i64) -> u32 {
        self.add(v as u64)
    }

    fn add_f64(&mut self, v: f64) -> u32 {
        self.add(v.to_bits())
    }
}

/// If the instruction before `*at` is a `local.get` or a constant that writes `reg`, it becomes the operand and is turned into a no-op.
/// Otherwise `reg` itself is the operand.
macro_rules! take_operand {
    ($name:ident, $variant:ident, $operand:ident, $const_idx:expr) => {
        fn $name(instrs: &mut [ProcessedInstr], at: &mut usize, reg: u16) -> $operand {
            if *at > 0 {
                if let ProcessedInstr::$variant {
                    handler_index,
                    dst: $operand::Reg(d),
                    src1,
                    ..
                } = &instrs[*at - 1]
                {
                    if *d == reg
                        && (*handler_index == HANDLER_IDX_LOCAL_GET || *handler_index == $const_idx)
                    {
                        let src = *src1;
                        *at -= 1;
                        instrs[*at] = ProcessedInstr::NopReg;
                        return src;
                    }
                }
            }
            $operand::Reg(reg)
        }
    };
}

take_operand!(
    take_i32_operand,
    I32Reg,
    I32RegOperand,
    HANDLER_IDX_I32_CONST
);
take_operand!(
    take_i64_operand,
    I64Reg,
    I64RegOperand,
    HANDLER_IDX_I64_CONST
);
take_operand!(
    take_f32_operand,
    F32Reg,
    F32RegOperand,
    HANDLER_IDX_F32_CONST
);
take_operand!(
    take_f64_operand,
    F64Reg,
    F64RegOperand,
    HANDLER_IDX_F64_CONST
);

/// If the instruction before `*at` writes `src`, makes it write `local` instead, so the `local.set` needs no copy.
/// The caller makes sure that instruction's result is the top of the stack.
fn fold_dst_into_local(
    instrs: &mut [ProcessedInstr],
    at: &mut usize,
    src: Reg,
    local: Reg,
) -> bool {
    if *at == 0 {
        return false;
    }
    let idx = src.index();
    let dst: &mut u16 = match &mut instrs[*at - 1] {
        ProcessedInstr::I32Reg {
            dst: I32RegOperand::Reg(d),
            ..
        } => d,
        ProcessedInstr::I64Reg {
            dst: I64RegOperand::Reg(d),
            ..
        } => d,
        ProcessedInstr::F32Reg {
            dst: F32RegOperand::Reg(d),
            ..
        } => d,
        ProcessedInstr::F64Reg {
            dst: F64RegOperand::Reg(d),
            ..
        } => d,
        ProcessedInstr::ConversionReg {
            dst: RegOrLocal::Reg(d),
            ..
        }
        | ProcessedInstr::MemoryLoadReg {
            dst: RegOrLocal::Reg(d),
            ..
        }
        | ProcessedInstr::GlobalGetReg {
            dst: RegOrLocal::Reg(d),
            ..
        } => d,
        _ => return false,
    };
    if *dst != idx {
        return false;
    }
    *dst = local.index();
    *at -= 1;
    true
}

/// An open block, loop or `if` while its body is decoded.
#[derive(Debug, Clone)]
struct ControlBlockInfo {
    block_type: wasmparser::BlockType,
    /// A branch to a loop lands on its start, not after its end.
    is_loop: bool,
    /// Where a branch to this block leaves the block's results.
    result_regs: Vec<Reg>,
}

/// WASI imports by name, one entry per signature.
static WASI_IMPORTS: LazyLock<FxHashMap<&'static str, Vec<WasiFuncType>>> = LazyLock::new(|| {
    use SocketExt::*;
    let ext = WasiFuncType::SocketExt;
    let mut list = vec![
        // Preview 1
        ("proc_exit", WasiFuncType::ProcExit),
        ("fd_write", WasiFuncType::FdWrite),
        ("fd_read", WasiFuncType::FdRead),
        ("random_get", WasiFuncType::RandomGet),
        ("fd_prestat_get", WasiFuncType::FdPrestatGet),
        ("fd_prestat_dir_name", WasiFuncType::FdPrestatDirName),
        ("fd_close", WasiFuncType::FdClose),
        ("environ_get", WasiFuncType::EnvironGet),
        ("environ_sizes_get", WasiFuncType::EnvironSizesGet),
        ("args_get", WasiFuncType::ArgsGet),
        ("args_sizes_get", WasiFuncType::ArgsSizesGet),
        ("clock_time_get", WasiFuncType::ClockTimeGet),
        ("clock_res_get", WasiFuncType::ClockResGet),
        ("sched_yield", WasiFuncType::SchedYield),
        ("fd_fdstat_get", WasiFuncType::FdFdstatGet),
        ("path_open", WasiFuncType::PathOpen),
        ("fd_seek", WasiFuncType::FdSeek),
        ("fd_tell", WasiFuncType::FdTell),
        ("fd_sync", WasiFuncType::FdSync),
        ("fd_filestat_get", WasiFuncType::FdFilestatGet),
        ("fd_readdir", WasiFuncType::FdReaddir),
        ("fd_pread", WasiFuncType::FdPread),
        ("fd_datasync", WasiFuncType::FdDatasync),
        ("fd_fdstat_set_flags", WasiFuncType::FdFdstatSetFlags),
        ("fd_filestat_set_size", WasiFuncType::FdFilestatSetSize),
        ("fd_pwrite", WasiFuncType::FdPwrite),
        ("path_create_directory", WasiFuncType::PathCreateDirectory),
        ("path_filestat_get", WasiFuncType::PathFilestatGet),
        ("path_readlink", WasiFuncType::PathReadlink),
        ("path_remove_directory", WasiFuncType::PathRemoveDirectory),
        ("path_unlink_file", WasiFuncType::PathUnlinkFile),
        ("poll_oneoff", WasiFuncType::PollOneoff),
        ("proc_raise", WasiFuncType::ProcRaise),
        ("fd_advise", WasiFuncType::FdAdvise),
        ("fd_allocate", WasiFuncType::FdAllocate),
        ("fd_fdstat_set_rights", WasiFuncType::FdFdstatSetRights),
        ("fd_renumber", WasiFuncType::FdRenumber),
        ("fd_filestat_set_times", WasiFuncType::FdFilestatSetTimes),
        ("path_link", WasiFuncType::PathLink),
        ("path_rename", WasiFuncType::PathRename),
        ("path_symlink", WasiFuncType::PathSymlink),
        ("sock_recv", WasiFuncType::SockRecv),
        ("sock_send", WasiFuncType::SockSend),
        ("sock_shutdown", WasiFuncType::SockShutdown),
        (
            "path_filestat_set_times",
            WasiFuncType::PathFilestatSetTimes,
        ),
        // Socket extensions; WasmEdge also registers its V2 signatures under
        // explicit `_v2` names.
        ("sock_accept", WasiFuncType::SockAccept),
        ("sock_accept", ext(AcceptV1)),
        ("sock_accept_v2", WasiFuncType::SockAccept),
        ("sock_recv_v2", WasiFuncType::SockRecv),
        ("sock_send_v2", WasiFuncType::SockSend),
        ("sock_listen", ext(Listen)),
        ("sock_listen_v2", ext(Listen)),
        ("sock_open", ext(OpenWamr)),
        ("sock_open", ext(OpenWasmEdge)),
        ("sock_open_v2", ext(OpenWasmEdge)),
        ("sock_bind", ext(BindWamr)),
        ("sock_bind", ext(BindWasmEdge)),
        ("sock_bind_v2", ext(BindWasmEdge)),
        ("sock_connect", ext(ConnectWamr)),
        ("sock_connect", ext(ConnectWasmEdge)),
        ("sock_connect_v2", ext(ConnectWasmEdge)),
        ("sock_recv_from", ext(RecvFromWamr)),
        ("sock_recv_from", ext(RecvFromV1)),
        ("sock_recv_from", ext(RecvFromV2)),
        ("sock_recv_from_v2", ext(RecvFromV2)),
        ("sock_send_to", ext(SendToWamr)),
        ("sock_send_to", ext(SendToWasmEdge)),
        ("sock_send_to_v2", ext(SendToWasmEdge)),
        ("sock_addr_local", ext(AddrLocal)),
        ("sock_addr_remote", ext(AddrRemote)),
        ("sock_addr_resolve", ext(AddrResolve)),
        ("sock_close", ext(Close)),
        ("sock_getlocaladdr", ext(GetLocalAddrV1)),
        ("sock_getlocaladdr", ext(GetLocalAddrV2)),
        ("sock_getlocaladdr_v2", ext(GetLocalAddrV2)),
        ("sock_getpeeraddr", ext(GetPeerAddrV1)),
        ("sock_getpeeraddr", ext(GetPeerAddrV2)),
        ("sock_getpeeraddr_v2", ext(GetPeerAddrV2)),
        ("sock_getaddrinfo", ext(GetAddrInfo)),
        ("sock_setsockopt", ext(SetSockOpt)),
        ("sock_getsockopt", ext(GetSockOpt)),
    ];
    for (opt, set_name, get_name) in WamrSockOpt::ALL {
        list.push((set_name, ext(SetOptWamr(opt))));
        if let Some(get_name) = get_name {
            list.push((get_name, ext(GetOptWamr(opt))));
        }
    }
    let mut map = FxHashMap::default();
    for (name, func) in list {
        map.entry(name).or_insert_with(Vec::new).push(func);
    }
    map
});

/// Converts a wasmparser value type to the internal representation.
fn match_value_type(t: ValType) -> ValueType {
    match t {
        ValType::I32 => ValueType::NumType(NumType::I32),
        ValType::I64 => ValueType::NumType(NumType::I64),
        ValType::F32 => ValueType::NumType(NumType::F32),
        ValType::F64 => ValueType::NumType(NumType::F64),
        ValType::V128 => ValueType::VecType(VecType::V128),
        ValType::Ref(ref_type) => {
            if ref_type.is_func_ref() {
                ValueType::RefType(RefType::FuncRef)
            } else {
                ValueType::RefType(RefType::ExternalRef)
            }
        }
    }
}

/// Converts a slice of wasmparser value types to internal representation.
fn types_to_vec(types: &[ValType], vec: &mut Vec<ValueType>) {
    for t in types.iter() {
        vec.push(match_value_type(*t));
    }
}

/// Returns the result types for a block type.
fn get_block_result_types(block_type: &wasmparser::BlockType, module: &Module) -> Vec<ValueType> {
    match block_type {
        wasmparser::BlockType::Empty => Vec::new(),
        wasmparser::BlockType::Type(val_type) => {
            vec![match_value_type(*val_type)]
        }
        wasmparser::BlockType::FuncType(type_idx) => {
            if let Some(func_type) = module.types.get(*type_idx as usize) {
                func_type.results.clone()
            } else {
                Vec::new()
            }
        }
    }
}

/// Returns the parameter types for a block type (for multi-value blocks).
fn get_block_param_types(block_type: &wasmparser::BlockType, module: &Module) -> Vec<ValueType> {
    match block_type {
        wasmparser::BlockType::Empty => Vec::new(),
        wasmparser::BlockType::Type(_) => Vec::new(), // Single result type means no params
        wasmparser::BlockType::FuncType(type_idx) => {
            if let Some(func_type) = module.types.get(*type_idx as usize) {
                func_type.params.clone()
            } else {
                Vec::new()
            }
        }
    }
}

/// Decodes the type section, building function type signatures.
fn decode_type_section(
    body: SectionLimited<'_, wasmparser::RecGroup>,
    module: &mut Module,
) -> Result<(), Box<dyn std::error::Error>> {
    for functype in body.into_iter_err_on_gc_types() {
        let functype = functype?;

        let mut params = Vec::new();
        let mut results = Vec::new();
        types_to_vec(functype.params(), &mut params);
        types_to_vec(functype.results(), &mut results);

        Shared::get_mut(&mut module.types)
            .unwrap()
            .push(crate::structure::types::FuncType { params, results });
    }
    Ok(())
}

/// Decodes the function section, creating stub entries for each function.
///
/// The function section only contains type indices;
/// the actual code bodies are decoded separately from the code section.
fn decode_func_section(
    body: SectionLimited<'_, u32>,
    module: &mut Module,
    #[cfg(feature = "call_graph")] mut cg_builder: Option<&mut CallGraphBuilder>,
) -> Result<(), Box<dyn std::error::Error>> {
    for func in body {
        let index = func?;
        let typeidx = TypeIdx(index);
        #[cfg(feature = "call_graph")]
        let func_idx = FuncIdx((module.num_imported_funcs + module.funcs.len()) as u32);
        module.funcs.push(Func {
            type_: typeidx,
            locals: Vec::new(),
            body: Shared::new(Vec::new()),
            reg_allocation: None,
            handlers: Shared::new(HandlerTable::new(Vec::new())),
            wide_consts: Box::new([]),
        });
        #[cfg(feature = "call_graph")]
        if let Some(b) = cg_builder.as_mut() {
            b.register_local_func(func_idx, typeidx);
        }
    }

    Ok(())
}

/// Decodes the import section, handling functions, tables, memories, and globals.
///
/// WASI Preview 1 function imports from `wasi_snapshot_preview1` are identified and stored with their specific `WasiFuncType` for passthrough handling.
fn decode_import_section(
    body: SectionLimited<'_, wasmparser::Import<'_>>,
    module: &mut Module,
    #[cfg(feature = "call_graph")] mut cg_builder: Option<&mut CallGraphBuilder>,
) -> Result<(), Box<dyn std::error::Error>> {
    for import in body {
        let import = import?;
        let desc = match import.ty {
            TypeRef::Func(type_index) => {
                let func_type = module
                    .types
                    .get(type_index as usize)
                    .ok_or("import refers to an unknown type")?;
                if let Some(wasi_func_type) =
                    parse_wasi_import(&import.module, &import.name, func_type)
                {
                    #[cfg(feature = "call_graph")]
                    if let Some(b) = cg_builder.as_mut() {
                        b.register_wasi_func();
                    }
                    module.num_imported_funcs += 1;
                    ImportDesc::WasiFunc(wasi_func_type)
                } else {
                    #[cfg(feature = "call_graph")]
                    if let Some(b) = cg_builder.as_mut() {
                        b.register_import_func(
                            FuncIdx(module.num_imported_funcs as u32),
                            TypeIdx(type_index),
                        );
                    }
                    module.num_imported_funcs += 1;
                    ImportDesc::Func(TypeIdx(type_index))
                }
            }
            TypeRef::Table(table_type) => {
                let max = match table_type.maximum {
                    Some(m) => Some(TryFrom::try_from(m).unwrap()),
                    None => None,
                };
                let limits = Limits {
                    min: TryFrom::try_from(table_type.initial).unwrap(),
                    max,
                };
                let reftype = if table_type.element_type.is_func_ref() {
                    RefType::FuncRef
                } else {
                    RefType::ExternalRef
                };

                ImportDesc::Table(TableType(limits, reftype))
            }
            TypeRef::Memory(memory) => {
                let max = match memory.maximum {
                    Some(m) => Some(TryFrom::try_from(m).unwrap()),
                    None => None,
                };
                let limits = Limits {
                    min: TryFrom::try_from(memory.initial).unwrap(),
                    max,
                };
                ImportDesc::Mem(MemType {
                    limits,
                    shared: memory.shared,
                })
            }
            TypeRef::Global(global) => {
                let mut_ = if global.mutable { Mut::Var } else { Mut::Const };
                let value_type = match_value_type(global.content_type);
                ImportDesc::Global(GlobalType(mut_, value_type))
            }
            TypeRef::Tag(_) => todo!(),
        };
        module.imports.push(Import {
            module: Name(import.module.to_string()),
            name: Name(import.name.to_string()),
            desc,
        });
    }
    Ok(())
}

/// Resolves an import to a WASI function handled here, if any.
fn parse_wasi_import(module: &str, name: &str, func_type: &FuncType) -> Option<WasiFuncType> {
    let candidates: &[WasiFuncType] = match module {
        "wasi_snapshot_preview1" => WASI_IMPORTS.get(name)?,
        // wasi-libc emitted `thread_spawn` before switching to the WIT-style `thread-spawn`; accept both.
        "wasi" if name == "thread-spawn" || name == "thread_spawn" => &[WasiFuncType::ThreadSpawn],
        _ => return None,
    };
    candidates
        .iter()
        .copied()
        .find(|candidate| candidate.expected_func_type().type_match(func_type))
}

/// Decodes the export section.
fn decode_export_section(
    body: SectionLimited<'_, wasmparser::Export<'_>>,
    module: &mut Module,
) -> Result<(), Box<dyn std::error::Error>> {
    for export in body {
        let export = export?;
        let index = export.index;
        let desc = match export.kind {
            ExternalKind::Func => ExportDesc::Func(FuncIdx(index)),
            ExternalKind::Table => ExportDesc::Table(TableIdx(index)),
            ExternalKind::Memory => ExportDesc::Mem(MemIdx(index)),
            ExternalKind::Global => ExportDesc::Global(GlobalIdx(index)),
            ExternalKind::Tag => todo!(),
        };
        module.exports.push(Export {
            name: Name(export.name.to_string()),
            desc,
        });
    }
    Ok(())
}

/// Decodes the memory section.
fn decode_mem_section(
    body: SectionLimited<'_, wasmparser::MemoryType>,
    module: &mut Module,
) -> Result<(), Box<dyn std::error::Error>> {
    for memory in body {
        let memory = memory?;
        let max = match memory.maximum {
            Some(m) => Some(TryFrom::try_from(m).unwrap()),
            None => None,
        };
        let limits = Limits {
            min: TryFrom::try_from(memory.initial).unwrap(),
            max,
        };
        module.mems.push(Mem {
            type_: MemType {
                limits,
                shared: memory.shared,
            },
        });
    }
    Ok(())
}

/// Decodes the table section.
fn decode_table_section(
    body: SectionLimited<'_, wasmparser::Table<'_>>,
    module: &mut Module,
) -> Result<(), Box<dyn std::error::Error>> {
    for table in body {
        let table = table?;
        let table_type = table.ty;

        let max = match table_type.maximum {
            Some(m) => Some(TryFrom::try_from(m).unwrap()),
            None => None,
        };
        let limits = Limits {
            min: TryFrom::try_from(table_type.initial).unwrap(),
            max,
        };

        let reftype = if table_type.element_type.is_func_ref() {
            RefType::FuncRef
        } else {
            RefType::ExternalRef
        };
        module.tables.push(Table {
            type_: TableType(limits, reftype),
        });
    }
    Ok(())
}

/// Decodes the global section.
fn decode_global_section(
    body: SectionLimited<'_, wasmparser::Global<'_>>,
    module: &mut Module,
) -> Result<(), Box<dyn std::error::Error>> {
    for global in body {
        let global = global?;
        let mut_ = if global.ty.mutable {
            Mut::Var
        } else {
            Mut::Const
        };
        let value_type = match_value_type(global.ty.content_type);
        let type_ = GlobalType(mut_, value_type);
        let init = parse_initexpr(global.init_expr)?;
        module.globals.push(Global { type_, init });
    }
    Ok(())
}

/// Parses a constant expression (used for global initializers, data/elem offsets).
fn parse_initexpr(expr: wasmparser::ConstExpr<'_>) -> Result<Expr, Box<dyn std::error::Error>> {
    let mut instrs = Vec::new();
    let mut ops = expr
        .get_operators_reader()
        .into_iter_with_offsets()
        .peekable();
    while let Some(res) = ops.next() {
        let (op, offset) = res?;

        if (matches!(op, wasmparser::Operator::End) && ops.peek().is_none()) {
            break;
        }

        match op {
            wasmparser::Operator::I32Const { value } => instrs.push(Instr::I32Const(value)),
            wasmparser::Operator::I64Const { value } => instrs.push(Instr::I64Const(value)),
            wasmparser::Operator::F32Const { value } => {
                instrs.push(Instr::F32Const(f32::from_bits(value.bits())))
            }
            wasmparser::Operator::F64Const { value } => {
                instrs.push(Instr::F64Const(f64::from_bits(value.bits())))
            }
            wasmparser::Operator::RefNull { .. } => {
                instrs.push(Instr::RefNull(RefType::ExternalRef))
            }
            wasmparser::Operator::RefFunc { function_index } => {
                instrs.push(Instr::RefFunc(FuncIdx(function_index)))
            }
            wasmparser::Operator::GlobalGet { global_index } => {
                instrs.push(Instr::GlobalGet(GlobalIdx(global_index)))
            }

            _ => {
                return Err(Box::new(ParserError::InitExprUnsupportedOPCodeError {
                    offset,
                }))
            }
        }
    }
    Ok(Expr(instrs))
}

/// Decodes the element section (table initialization data).
fn decode_elem_section(
    body: SectionLimited<'_, wasmparser::Element<'_>>,
    module: &mut Module,
) -> Result<(), Box<dyn std::error::Error>> {
    for (_index, entry) in body.into_iter().enumerate() {
        let entry = entry?;
        let _cnt = 0;
        let (type_, init, idxes) = match entry.items {
            wasmparser::ElementItems::Functions(funcs) => {
                let mut idxes = Vec::new();
                for func in funcs {
                    idxes.push(FuncIdx(func?));
                }
                (RefType::FuncRef, None, Some(idxes))
            }
            wasmparser::ElementItems::Expressions(type_, items) => {
                let mut exprs = Vec::new();
                for expr in items {
                    let expr = parse_initexpr(expr?)?;
                    exprs.push(expr);
                }

                if type_.is_func_ref() {
                    (RefType::FuncRef, Some(exprs), None)
                } else {
                    (RefType::ExternalRef, Some(exprs), None)
                }
            }
        };
        let (mode, table_idx, offset) = match entry.kind {
            wasmparser::ElementKind::Active {
                table_index,
                offset_expr,
            } => {
                let expr = parse_initexpr(offset_expr)?;
                let table_index = table_index.unwrap_or(0);
                (ElemMode::Active, Some(TableIdx(table_index)), Some(expr))
            }
            wasmparser::ElementKind::Passive => (ElemMode::Passive, None, None),
            wasmparser::ElementKind::Declared => (ElemMode::Declarative, None, None),
        };
        module.elems.push(Elem {
            type_,
            init,
            idxes,
            mode,
            table_idx,
            offset,
        });
    }
    Ok(())
}

/// Decodes the data section (memory initialization data).
fn decode_data_section(
    body: SectionLimited<'_, wasmparser::Data<'_>>,
    module: &mut Module,
) -> Result<(), Box<dyn std::error::Error>> {
    for (_index, entry) in body.into_iter().enumerate() {
        let entry = entry?;
        let init = entry.data.iter().map(|x| Byte(*x)).collect::<Vec<Byte>>();
        let (mode, memory, offset) = match entry.kind {
            wasmparser::DataKind::Passive => (DataMode::Passive, None, None),
            wasmparser::DataKind::Active {
                memory_index,
                offset_expr,
            } => {
                let expr = parse_initexpr(offset_expr)?;
                (DataMode::Active, Some(MemIdx(memory_index)), Some(expr))
            }
        };

        module.datas.push(Data {
            init,
            mode,
            memory,
            offset,
        })
    }
    Ok(())
}

/// Decodes one function body: registers are allocated for the operands,
/// branch targets resolved, and the handler table built.
fn decode_code_section(
    body: FunctionBody<'_>,
    module: &mut Module,
    func_index: usize,
    #[cfg(feature = "call_graph")] mut cg_builder: Option<&mut CallGraphBuilder>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut locals: Vec<(u32, ValueType)> = Vec::new();
    for pair in body.get_locals_reader()? {
        let (cnt, ty) = pair?;
        locals.push((cnt, match_value_type(ty)));
    }
    let ops_iter = body.get_operators_reader()?.into_iter_with_offsets();
    let func_type = get_func_type(module, func_index as u32);

    let DecodedBody {
        mut instrs,
        fixups,
        block_end_map,
        if_else_map,
        reg_allocation,
        const_pool,
    } = decode_processed_instrs_and_fixups(
        ops_iter,
        module,
        &locals,
        &func_type.params,
        &func_type.results,
        #[cfg(feature = "call_graph")]
        cg_builder.as_mut().map(|x| &mut **x),
        #[cfg(feature = "call_graph")]
        func_index,
    )?;
    preprocess_instructions(&mut instrs, fixups, &block_end_map, &if_else_map)?;
    let instrs = compact_instruction_stream(instrs);

    // Parallel to the body, plus a halt sentinel so an out-of-range dispatch in TCO mode terminates safely.
    let mut handlers: Vec<Handler> = instrs.iter().map(select_handler).collect();
    handlers.push(halt);

    let local_index = func_index - module.num_imported_funcs;
    let func = module
        .funcs
        .get_mut(local_index)
        .ok_or(RuntimeError::InvalidWasm(
            "Invalid function index during code decoding",
        ))?;
    func.locals = locals;
    func.body = Shared::new(instrs);
    func.reg_allocation = Some(reg_allocation);
    func.handlers = Shared::new(HandlerTable::new(handlers));
    func.wide_consts = const_pool.consts.into_boxed_slice();
    Ok(())
}

/// Returns true for instructions with no runtime effect, removable once all branch targets are resolved to absolute IPs:
/// - `BlockReg` (block/loop): labels are fully static, the handler is a no-op.
/// - `NopReg`: explicit nops and unreachable-code placeholders.
/// - Inner `EndReg` whose register copy does nothing (either side empty or source == target).
///   The function-level end is always kept: it collects the return registers and is the target of function-level branches.
fn is_noop_instr(instr: &ProcessedInstr) -> bool {
    match instr {
        ProcessedInstr::BlockReg { .. } | ProcessedInstr::NopReg => true,
        ProcessedInstr::EndReg {
            is_function_end: false,
            source_regs,
            target_result_regs,
        } => {
            source_regs.is_empty()
                || target_result_regs.is_empty()
                || source_regs == target_result_regs
        }
        _ => false,
    }
}

/// Phase 3: Removes no-op instructions from the stream and remaps all branch targets
/// (`BrReg`, `BrIfReg`, `BrTableReg`, `IfReg`, `JumpReg`) to the compacted indices.
///
/// A jump to a removed instruction lands on the next kept instruction at or after it, 
/// which is semantically identical because removed instructions do nothing. 
/// `remap[i]` = number of kept instructions before old index `i`, which is exactly that target.
fn compact_instruction_stream(processed: Vec<ProcessedInstr>) -> Vec<ProcessedInstr> {
    let old_len = processed.len();
    let mut remap: Vec<usize> = Vec::with_capacity(old_len + 1);
    let mut kept_count = 0usize;
    for instr in processed.iter() {
        remap.push(kept_count);
        if !is_noop_instr(instr) {
            kept_count += 1;
        }
    }
    // Defensive entry for targets one past the end.
    remap.push(kept_count);

    let mut kept: Vec<ProcessedInstr> = Vec::with_capacity(kept_count);
    for mut instr in processed.into_iter() {
        if is_noop_instr(&instr) {
            continue;
        }
        match &mut instr {
            ProcessedInstr::BrReg { target_ip, .. } => *target_ip = remap[*target_ip],
            ProcessedInstr::BrIfReg { target_ip, .. } => *target_ip = remap[*target_ip],
            ProcessedInstr::IfReg { else_target_ip, .. } => {
                *else_target_ip = remap[*else_target_ip]
            }
            ProcessedInstr::JumpReg { target_ip } => *target_ip = remap[*target_ip],
            ProcessedInstr::BrTableReg(table) => {
                for (_, target_ip, _) in table.targets.iter_mut() {
                    *target_ip = remap[*target_ip];
                }
                table.default_target.1 = remap[table.default_target.1];
            }
            _ => {}
        }
        kept.push(instr);
    }
    kept
}

/// A branch whose target is still open when it is decoded,
/// so the target's position is patched in once the whole body is known.
#[derive(Debug, Clone)]
struct FixupInfo {
    /// Program counter of the instruction to patch.
    pc: usize,
    /// Relative depth of the target label.
    depth: usize,
    /// The values a function-level branch returns.
    /// It copies them into the function end's registers, which are known only after the body.
    source_regs: Vec<Reg>,
}

/// Where a branch of `depth` lands: a loop's start,
/// or the pc after a block's `end`. `None` past the outermost block, which is a return.
fn branch_target(
    control_stack: &[(usize, bool)],
    depth: usize,
    block_end_map: &FxHashMap<usize, usize>,
) -> Result<Option<usize>, RuntimeError> {
    let Some(&(start_pc, is_loop)) = control_stack
        .len()
        .checked_sub(1 + depth)
        .map(|i| &control_stack[i])
    else {
        return Ok(None);
    };
    if is_loop {
        return Ok(Some(start_pc));
    }
    block_end_map
        .get(&start_pc)
        .map(|end| Some(*end))
        .ok_or(RuntimeError::InvalidWasm(
            "Missing EndMarker for branch target",
        ))
}

/// Phase 2: resolves every branch target. One walk rebuilds the control stack and patches each fixup at its own instruction;
/// `br_table` carries its depths itself and is patched in the same walk.
fn preprocess_instructions(
    processed: &mut [ProcessedInstr],
    fixups: Vec<FixupInfo>,
    block_end_map: &FxHashMap<usize, usize>,
    if_else_map: &FxHashMap<usize, usize>,
) -> Result<(), RuntimeError> {
    let function_end_ip = processed.len() - 1;
    let function_end_regs: RegSlice = match processed.last() {
        Some(ProcessedInstr::EndReg { source_regs, .. }) => source_regs.clone(),
        _ => {
            return Err(RuntimeError::InvalidWasm(
                "Internal Error: function body does not terminate with EndReg",
            ))
        }
    };
    // (start pc, is_loop) of each open block.
    let mut control_stack: Vec<(usize, bool)> = Vec::new();
    let mut fixups = fixups.into_iter().peekable();
    for pc in 0..processed.len() {
        match processed[pc].handler_index() {
            HANDLER_IDX_BLOCK | HANDLER_IDX_IF => control_stack.push((pc, false)),
            HANDLER_IDX_LOOP => control_stack.push((pc, true)),
            HANDLER_IDX_END => {
                control_stack.pop();
            }
            _ => {}
        }

        if let ProcessedInstr::BrTableReg(table) = &mut processed[pc] {
            let default = std::iter::once(&mut table.default_target);
            for (depth, target_ip, regs) in table.targets.iter_mut().chain(default) {
                match branch_target(&control_stack, *depth as usize, block_end_map)? {
                    Some(ip) => *target_ip = ip,
                    None => {
                        *target_ip = function_end_ip;
                        *regs = function_end_regs.clone();
                    }
                }
            }
        }

        // The instruction at `pc` is already on the control stack, which an `if` at depth 0 relies on.
        while let Some(fixup) = fixups.next_if(|fixup| fixup.pc == pc) {
            let target = branch_target(&control_stack, fixup.depth, block_end_map)?;
            match (&mut processed[pc], target) {
                // Jump on false lands after `else`, or after `end` without one.
                (ProcessedInstr::IfReg { else_target_ip, .. }, Some(end_ip)) => {
                    *else_target_ip = *if_else_map.get(&pc).unwrap_or(&end_ip);
                }
                (ProcessedInstr::JumpReg { target_ip }, Some(ip))
                | (ProcessedInstr::BrReg { target_ip, .. }, Some(ip))
                | (ProcessedInstr::BrIfReg { target_ip, .. }, Some(ip)) => *target_ip = ip,
                (
                    ProcessedInstr::BrReg {
                        target_ip,
                        result_copies,
                    }
                    | ProcessedInstr::BrIfReg {
                        target_ip,
                        result_copies,
                        ..
                    },
                    None,
                ) => {
                    *target_ip = function_end_ip;
                    *result_copies = BranchCopies::new(
                        fixup.source_regs.into_boxed_slice(),
                        function_end_regs.clone(),
                    );
                }
                _ => {
                    return Err(RuntimeError::InvalidWasm(
                        "Internal Error: fixup on an instruction without a branch target",
                    ))
                }
            }
        }
    }
    if fixups.next().is_some() {
        return Err(RuntimeError::InvalidWasm(
            "Internal Error: Unprocessed fixup after preprocessing",
        ));
    }
    Ok(())
}

/// Returns the type of a local variable by index.
///
/// WebAssembly local indices include function parameters first (indices 0..n-1),
/// followed by declared locals. The `locals` parameter uses compressed format where each entry is (count, type).
fn get_local_type(
    params: &[ValueType],
    locals: &[(u32, ValueType)],
    local_index: u32,
) -> ValueType {
    let mut index = local_index as usize;

    // First, check if the index is within the parameters range
    if index < params.len() {
        return params[index];
    }

    // Subtract parameter count to get index into locals
    index -= params.len();

    // Now search through declared locals
    for (count, vtype) in locals {
        if index < *count as usize {
            return *vtype;
        }
        index -= *count as usize;
    }
    // Should not reach here in valid wasm (wasmparser validates indices)
    ValueType::NumType(NumType::I32)
}

/// Returns the import declaring the function at `func_index`.
///
/// Function indices count only function imports, so a module that also imports a memory or table cannot index `module.imports` directly.
fn get_imported_func_desc(module: &Module, func_index: u32) -> Option<&ImportDesc> {
    module
        .imports
        .iter()
        .filter(|import| matches!(import.desc, ImportDesc::Func(_) | ImportDesc::WasiFunc(_)))
        .nth(func_index as usize)
        .map(|import| &import.desc)
}

/// Returns the value type of a global variable by index.
///
/// Searches imported globals first, then module-defined globals.
fn get_global_type(module: &Module, global_index: u32) -> ValueType {
    let mut imported_global_count = 0u32;
    for import in &module.imports {
        if let ImportDesc::Global(global_type) = &import.desc {
            if imported_global_count == global_index {
                return global_type.1;
            }
            imported_global_count += 1;
        }
    }

    let local_global_index = (global_index - imported_global_count) as usize;
    if local_global_index < module.globals.len() {
        return module.globals[local_global_index].type_.1;
    }

    ValueType::NumType(NumType::I32)
}

/// Returns the element type of a table by index.
///
/// Searches imported tables first, then module-defined tables.
fn get_table_element_type(module: &Module, table_index: u32) -> ValueType {
    // Count imported tables first
    let mut imported_table_count = 0u32;
    for import in &module.imports {
        if let ImportDesc::Table(table_type) = &import.desc {
            if imported_table_count == table_index {
                return ValueType::RefType(table_type.1);
            }
            imported_table_count += 1;
        }
    }

    // Check module-defined tables
    let local_table_index = (table_index - imported_table_count) as usize;
    if local_table_index < module.tables.len() {
        return ValueType::RefType(module.tables[local_table_index].type_.1);
    }

    // Default to funcref
    ValueType::RefType(RefType::FuncRef)
}

/// What Phase 1 produces for one function body.
struct DecodedBody {
    instrs: Vec<ProcessedInstr>,
    fixups: Vec<FixupInfo>,
    /// Start pc of each block to the pc after its `end`.
    block_end_map: FxHashMap<usize, usize>,
    /// Start pc of each `if` to the pc after its `else`, or after its `end` when it has none.
    if_else_map: FxHashMap<usize, usize>,
    reg_allocation: RegAllocation,
    const_pool: ConstPool,
}

const SELECT_HANDLERS: [usize; 4] = [
    HANDLER_IDX_SELECT_I32,
    HANDLER_IDX_SELECT_I64,
    HANDLER_IDX_SELECT_F32,
    HANDLER_IDX_SELECT_F64,
];
const GLOBAL_GET_HANDLERS: [usize; 4] = [
    HANDLER_IDX_GLOBAL_GET_I32,
    HANDLER_IDX_GLOBAL_GET_I64,
    HANDLER_IDX_GLOBAL_GET_F32,
    HANDLER_IDX_GLOBAL_GET_F64,
];
const GLOBAL_SET_HANDLERS: [usize; 4] = [
    HANDLER_IDX_GLOBAL_SET_I32,
    HANDLER_IDX_GLOBAL_SET_I64,
    HANDLER_IDX_GLOBAL_SET_F32,
    HANDLER_IDX_GLOBAL_SET_F64,
];

/// The handler for the numeric type `ty`, from `[i32, i64, f32, f64]`.
fn numeric_handler(ty: ValueType, handlers: [usize; 4]) -> usize {
    match ty {
        ValueType::I32 => handlers[0],
        ValueType::I64 => handlers[1],
        ValueType::F32 => handlers[2],
        ValueType::F64 => handlers[3],
        _ => panic!("Unsupported numeric type: {:?}", ty),
    }
}

/// A copy between registers of `ty`: `local.get`, `local.set` and `local.tee` all reduce to it.
/// References have an instruction of their own.
fn local_copy(ty: ValueType, handler_index: usize, dst: u16, src: u16) -> ProcessedInstr {
    match ty {
        ValueType::I32 => ProcessedInstr::I32Reg {
            handler_index,
            dst: I32RegOperand::Reg(dst),
            src1: I32RegOperand::Reg(src),
            src2: None,
        },
        ValueType::I64 => ProcessedInstr::I64Reg {
            handler_index,
            dst: I64RegOperand::Reg(dst),
            src1: I64RegOperand::Reg(src),
            src2: None,
        },
        ValueType::F32 => ProcessedInstr::F32Reg {
            handler_index,
            dst: F32RegOperand::Reg(dst),
            src1: F32RegOperand::Reg(src),
            src2: None,
        },
        ValueType::F64 => ProcessedInstr::F64Reg {
            handler_index,
            dst: F64RegOperand::Reg(dst),
            src1: F64RegOperand::Reg(src),
            src2: None,
        },
        ValueType::RefType(_) if handler_index == HANDLER_IDX_LOCAL_GET => {
            ProcessedInstr::RefLocalReg {
                handler_index: HANDLER_IDX_REF_LOCAL_GET,
                dst,
                src: 0,
                local_idx: src,
            }
        }
        ValueType::RefType(_) => ProcessedInstr::RefLocalReg {
            handler_index: HANDLER_IDX_REF_LOCAL_SET,
            dst: 0,
            src,
            local_idx: dst,
        },
        ValueType::VecType(_) => panic!("Unsupported type for local access: {:?}", ty),
    }
}

/// The type of the function at `func_index`, imported or defined.
fn get_func_type(module: &Module, func_index: u32) -> FuncType {
    let type_idx = if (func_index as usize) < module.num_imported_funcs {
        match get_imported_func_desc(module, func_index) {
            Some(ImportDesc::Func(type_idx)) => Some(*type_idx),
            _ => None,
        }
    } else {
        module
            .funcs
            .get(func_index as usize - module.num_imported_funcs)
            .map(|func| func.type_)
    };
    type_idx
        .and_then(|idx| module.types.get(idx.0 as usize))
        .cloned()
        .unwrap_or_default()
}

/// Pops the values of `types`, returning their registers.
fn pop_values(allocator: &mut RegAllocator, types: &[ValueType]) -> Vec<Reg> {
    let regs = allocator.peek_regs_for_types(types);
    for ty in types.iter().rev() {
        allocator.pop(ty);
    }
    regs
}

/// Pushes a value of each of `types`, returning their registers.
fn push_values(allocator: &mut RegAllocator, types: &[ValueType]) -> RegSlice {
    types.iter().map(|ty| allocator.push(*ty)).collect()
}

/// Pops `N` i32 values, bottom first.
fn pop_i32s<const N: usize>(allocator: &mut RegAllocator) -> [Reg; N] {
    let mut regs = [Reg::I32(0); N];
    for reg in regs.iter_mut().rev() {
        *reg = allocator.pop(&ValueType::I32);
    }
    regs
}

/// Opens a block: saves the allocator state under the block's params and reserves the block's result registers at that depth.
fn enter_block(
    allocator: &mut RegAllocator,
    allocator_state_stack: &mut Vec<RegAllocatorState>,
    control_info_stack: &mut Vec<ControlBlockInfo>,
    block_type: wasmparser::BlockType,
    module: &Module,
    is_loop: bool,
) {
    let param_types = get_block_param_types(&block_type, module);
    for ty in param_types.iter().rev() {
        allocator.pop(ty);
    }
    let saved_state = allocator.save_state();
    let mut state = saved_state.clone();
    let result_regs = get_block_result_types(&block_type, module)
        .iter()
        .map(|ty| state.next_reg_for_type(ty))
        .collect();
    allocator_state_stack.push(saved_state);
    push_values(allocator, &param_types);
    control_info_stack.push(ControlBlockInfo {
        block_type,
        is_loop,
        result_regs,
    });
}

/// `select`: `cond` is already popped, so that untyped `select` can look at the stack to learn the value type first.
fn select_instr(
    allocator: &mut RegAllocator,
    instrs: &mut [ProcessedInstr],
    at: &mut usize,
    cond: Reg,
    val_type: ValueType,
    handler_index: usize,
) -> ProcessedInstr {
    let val2 = allocator.pop(&val_type);
    let val1 = allocator.pop(&val_type);
    let mut operands = [val1, val2, cond];
    fold_local_get_args(instrs, at, &mut operands);
    let [val1, val2, cond] = operands;
    let dst = allocator.push(val_type);
    ProcessedInstr::SelectReg {
        handler_index,
        dst,
        val1,
        val2,
        cond,
    }
}

/// Phase 1: decodes a function body into register-based instructions.
///
/// The operand stack is simulated to assign typed registers, `local.get` and constants are folded into their consumers,
/// and branches get placeholder targets plus the fixups Phase 2 resolves.
fn decode_processed_instrs_and_fixups<'a>(
    ops_iter: wasmparser::OperatorsIteratorWithOffsets<'a>,
    module: &Module,
    locals: &[(u32, ValueType)],
    param_types: &[ValueType],
    result_types: &[ValueType],
    #[cfg(feature = "call_graph")] mut cg_builder: Option<&mut CallGraphBuilder>,
    #[cfg(feature = "call_graph")] func_index: usize,
) -> Result<DecodedBody, Box<dyn std::error::Error>> {
    let mut ops = ops_iter.peekable();
    let mut const_pool = ConstPool::default();
    let mut instrs: Vec<ProcessedInstr> = Vec::new();
    let mut fixups: Vec<FixupInfo> = Vec::new();
    let mut control_info_stack: Vec<ControlBlockInfo> = Vec::new();
    let mut block_end_map: FxHashMap<usize, usize> = FxHashMap::default();
    let mut if_else_map: FxHashMap<usize, usize> = FxHashMap::default();
    // (start pc, is_if, pc after `else`) of each open block.
    let mut open_blocks: Vec<(usize, bool, Option<usize>)> = Vec::new();

    // The wasm local index space is params first, then declared locals;
    // `local_regs[i]` is the register slot of local `i`.
    let local_types: Vec<(u32, ValueType)> = param_types
        .iter()
        .map(|ty| (1u32, *ty))
        .chain(locals.iter().copied())
        .collect();
    let mut allocator = RegAllocator::new(&local_types);
    let local_regs: Vec<Reg> = allocator.local_regs().to_vec();
    let mut allocator_state_stack: Vec<RegAllocatorState> = Vec::new();

    // Block nesting inside code that follows br, br_table, return or unreachable.
    let mut unreachable_depth: usize = 0;
    // Set by `drop`; a `local.set` then leaves the producer's dst alone.
    let mut dropped_since_emit = false;
    // The operand folders walk back from the instruction emitted last.
    let mut look_back: usize;

    // `dst = op(src1, src2)` on `$ty` values, leaving a `$result`.
    // The sources are folded operands: a `local.get` or constant that produced them is consumed into the instruction.
    macro_rules! binop {
        ($variant:ident, $operand:ident, $take:ident, $ty:ident, $result:ident, $handler:expr) => {{
            let src2_reg = allocator.pop(&ValueType::$ty);
            let src1_reg = allocator.pop(&ValueType::$ty);
            let src2 = $take(&mut instrs, &mut look_back, src2_reg.index());
            let src1 = $take(&mut instrs, &mut look_back, src1_reg.index());
            let dst = allocator.push(ValueType::$result);
            ProcessedInstr::$variant {
                handler_index: $handler,
                dst: $operand::Reg(dst.index()),
                src1,
                src2: Some(src2),
            }
        }};
    }
    macro_rules! unop {
        ($variant:ident, $operand:ident, $take:ident, $ty:ident, $result:ident, $handler:expr) => {{
            let src1_reg = allocator.pop(&ValueType::$ty);
            let src1 = $take(&mut instrs, &mut look_back, src1_reg.index());
            let dst = allocator.push(ValueType::$result);
            ProcessedInstr::$variant {
                handler_index: $handler,
                dst: $operand::Reg(dst.index()),
                src1,
                src2: None,
            }
        }};
    }
    macro_rules! i32_binop {
        ($h:expr) => {
            binop!(I32Reg, I32RegOperand, take_i32_operand, I32, I32, $h)
        };
    }
    macro_rules! i32_unop {
        ($h:expr) => {
            unop!(I32Reg, I32RegOperand, take_i32_operand, I32, I32, $h)
        };
    }
    macro_rules! i64_binop {
        ($h:expr) => {
            binop!(I64Reg, I64RegOperand, take_i64_operand, I64, I64, $h)
        };
    }
    macro_rules! i64_cmp {
        ($h:expr) => {
            binop!(I64Reg, I64RegOperand, take_i64_operand, I64, I32, $h)
        };
    }
    macro_rules! i64_unop {
        ($h:expr) => {
            unop!(I64Reg, I64RegOperand, take_i64_operand, I64, I64, $h)
        };
    }
    macro_rules! f32_binop {
        ($h:expr) => {
            binop!(F32Reg, F32RegOperand, take_f32_operand, F32, F32, $h)
        };
    }
    macro_rules! f32_cmp {
        ($h:expr) => {
            binop!(F32Reg, F32RegOperand, take_f32_operand, F32, I32, $h)
        };
    }
    macro_rules! f32_unop {
        ($h:expr) => {
            unop!(F32Reg, F32RegOperand, take_f32_operand, F32, F32, $h)
        };
    }
    macro_rules! f64_binop {
        ($h:expr) => {
            binop!(F64Reg, F64RegOperand, take_f64_operand, F64, F64, $h)
        };
    }
    macro_rules! f64_cmp {
        ($h:expr) => {
            binop!(F64Reg, F64RegOperand, take_f64_operand, F64, I32, $h)
        };
    }
    macro_rules! f64_unop {
        ($h:expr) => {
            unop!(F64Reg, F64RegOperand, take_f64_operand, F64, F64, $h)
        };
    }
    // `dst = convert(src)` from `$from` to `$to`.
    macro_rules! conv {
        ($from:ident, $to:ident, $handler:expr) => {{
            let src = allocator.pop(&ValueType::$from);
            let dst = allocator.push(ValueType::$to);
            ProcessedInstr::ConversionReg {
                handler_index: $handler,
                dst: RegOrLocal::Reg(dst.index()),
                src,
            }
        }};
    }
    // The address operand of a memory access.
    macro_rules! pop_addr {
        () => {{
            let addr = allocator.pop(&ValueType::I32);
            take_i32_operand(&mut instrs, &mut look_back, addr.index())
        }};
    }
    macro_rules! load {
        ($to:ident, $handler:expr, $memarg:expr) => {{
            let addr = pop_addr!();
            let dst = allocator.push(ValueType::$to);
            ProcessedInstr::MemoryLoadReg {
                handler_index: $handler,
                dst: RegOrLocal::Reg(dst.index()),
                addr,
                offset: $memarg.offset,
            }
        }};
    }
    macro_rules! store {
        ($ty:ident, $handler:expr, $memarg:expr) => {{
            let value = allocator.pop(&ValueType::$ty);
            let value = fold_local_get_arg(&mut instrs, &mut look_back, value);
            let addr = pop_addr!();
            ProcessedInstr::MemoryStoreReg {
                handler_index: $handler,
                addr,
                value,
                offset: $memarg.offset,
            }
        }};
    }
    macro_rules! rmw {
        ($ty:ident, $handler:expr, $memarg:expr) => {{
            let value = allocator.pop(&ValueType::$ty);
            let addr = pop_addr!();
            let dst = allocator.push(ValueType::$ty);
            ProcessedInstr::AtomicRmwReg {
                handler_index: $handler,
                dst,
                addr,
                value,
                offset: $memarg.offset,
            }
        }};
    }
    macro_rules! cmpxchg {
        ($ty:ident, $handler:expr, $memarg:expr) => {{
            let replacement = allocator.pop(&ValueType::$ty);
            let expected = allocator.pop(&ValueType::$ty);
            let addr = allocator.pop(&ValueType::I32);
            let dst = allocator.push(ValueType::$ty);
            ProcessedInstr::AtomicCmpxchgReg {
                handler_index: $handler,
                dst,
                args: [addr, expected, replacement],
                offset: $memarg.offset,
            }
        }};
    }
    macro_rules! wait {
        ($ty:ident, $handler:expr, $memarg:expr) => {{
            let timeout = allocator.pop(&ValueType::I64);
            let expected = allocator.pop(&ValueType::$ty);
            let addr = allocator.pop(&ValueType::I32);
            let dst = allocator.push(ValueType::I32);
            ProcessedInstr::AtomicWaitReg {
                handler_index: $handler,
                dst,
                args: [addr, expected, timeout],
                offset: $memarg.offset,
            }
        }};
    }

    while let Some(op) = ops.next() {
        let (op, _offset) = op?;

        // Code after an unconditional branch is dead until its block ends;
        // nested blocks are skipped whole.
        // An `else` at depth 1 reopens an `if` whose then-branch ended in one.
        if unreachable_depth > 0 {
            match &op {
                wasmparser::Operator::Block { .. }
                | wasmparser::Operator::Loop { .. }
                | wasmparser::Operator::If { .. } => unreachable_depth += 1,
                wasmparser::Operator::End => unreachable_depth -= 1,
                wasmparser::Operator::Else if unreachable_depth == 1 => unreachable_depth = 0,
                _ => {}
            }
            if unreachable_depth > 0 {
                instrs.push(ProcessedInstr::NopReg);
                continue;
            }
        }

        let pc = instrs.len();
        look_back = pc;

        let instr: Option<ProcessedInstr> = match &op {
            wasmparser::Operator::LocalGet { local_index } => {
                let ty = get_local_type(param_types, locals, *local_index);
                let dst = allocator.push(ty);
                let local = local_regs[*local_index as usize];
                Some(local_copy(
                    ty,
                    HANDLER_IDX_LOCAL_GET,
                    dst.index(),
                    local.index(),
                ))
            }
            wasmparser::Operator::LocalSet { local_index } => {
                let ty = get_local_type(param_types, locals, *local_index);
                let local = local_regs[*local_index as usize];
                let src = allocator.pop(&ty);
                let folded = !dropped_since_emit
                    && !matches!(ty, ValueType::RefType(_))
                    && fold_dst_into_local(&mut instrs, &mut look_back, src, local);
                if folded {
                    None
                } else {
                    Some(local_copy(
                        ty,
                        HANDLER_IDX_LOCAL_SET,
                        local.index(),
                        src.index(),
                    ))
                }
            }
            wasmparser::Operator::LocalTee { local_index } => {
                // The value stays on the stack, so it is copied, not popped.
                let ty = get_local_type(param_types, locals, *local_index);
                let local = local_regs[*local_index as usize];
                let src = allocator.peek(&ty).unwrap();
                Some(local_copy(
                    ty,
                    HANDLER_IDX_LOCAL_TEE,
                    local.index(),
                    src.index(),
                ))
            }
            wasmparser::Operator::GlobalGet { global_index } => {
                let ty = get_global_type(module, *global_index);
                let dst = allocator.push(ty);
                Some(ProcessedInstr::GlobalGetReg {
                    handler_index: numeric_handler(ty, GLOBAL_GET_HANDLERS),
                    dst: RegOrLocal::Reg(dst.index()),
                    global_index: *global_index,
                })
            }
            wasmparser::Operator::GlobalSet { global_index } => {
                let ty = get_global_type(module, *global_index);
                let src = allocator.pop(&ty);
                let src = fold_local_get_arg(&mut instrs, &mut look_back, src);
                Some(ProcessedInstr::GlobalSetReg {
                    handler_index: numeric_handler(ty, GLOBAL_SET_HANDLERS),
                    src: RegOrLocal::Reg(src.index()),
                    global_index: *global_index,
                })
            }
            wasmparser::Operator::I32Const { value } => {
                let dst = allocator.push(ValueType::I32);
                Some(ProcessedInstr::I32Reg {
                    handler_index: HANDLER_IDX_I32_CONST,
                    dst: I32RegOperand::Reg(dst.index()),
                    src1: I32RegOperand::Const(*value),
                    src2: None,
                })
            }
            wasmparser::Operator::I64Const { value } => {
                let dst = allocator.push(ValueType::I64);
                Some(ProcessedInstr::I64Reg {
                    handler_index: HANDLER_IDX_I64_CONST,
                    dst: I64RegOperand::Reg(dst.index()),
                    src1: I64RegOperand::Const(const_pool.add_i64(*value)),
                    src2: None,
                })
            }
            wasmparser::Operator::F32Const { value } => {
                let dst = allocator.push(ValueType::F32);
                Some(ProcessedInstr::F32Reg {
                    handler_index: HANDLER_IDX_F32_CONST,
                    dst: F32RegOperand::Reg(dst.index()),
                    src1: F32RegOperand::Const(f32::from_bits(value.bits())),
                    src2: None,
                })
            }
            wasmparser::Operator::F64Const { value } => {
                let dst = allocator.push(ValueType::F64);
                Some(ProcessedInstr::F64Reg {
                    handler_index: HANDLER_IDX_F64_CONST,
                    dst: F64RegOperand::Reg(dst.index()),
                    src1: F64RegOperand::Const(const_pool.add_f64(f64::from_bits(value.bits()))),
                    src2: None,
                })
            }

            // i32
            wasmparser::Operator::I32Add => Some(i32_binop!(HANDLER_IDX_I32_ADD)),
            wasmparser::Operator::I32Sub => Some(i32_binop!(HANDLER_IDX_I32_SUB)),
            wasmparser::Operator::I32Mul => Some(i32_binop!(HANDLER_IDX_I32_MUL)),
            wasmparser::Operator::I32DivS => Some(i32_binop!(HANDLER_IDX_I32_DIV_S)),
            wasmparser::Operator::I32DivU => Some(i32_binop!(HANDLER_IDX_I32_DIV_U)),
            wasmparser::Operator::I32RemS => Some(i32_binop!(HANDLER_IDX_I32_REM_S)),
            wasmparser::Operator::I32RemU => Some(i32_binop!(HANDLER_IDX_I32_REM_U)),
            wasmparser::Operator::I32And => Some(i32_binop!(HANDLER_IDX_I32_AND)),
            wasmparser::Operator::I32Or => Some(i32_binop!(HANDLER_IDX_I32_OR)),
            wasmparser::Operator::I32Xor => Some(i32_binop!(HANDLER_IDX_I32_XOR)),
            wasmparser::Operator::I32Shl => Some(i32_binop!(HANDLER_IDX_I32_SHL)),
            wasmparser::Operator::I32ShrS => Some(i32_binop!(HANDLER_IDX_I32_SHR_S)),
            wasmparser::Operator::I32ShrU => Some(i32_binop!(HANDLER_IDX_I32_SHR_U)),
            wasmparser::Operator::I32Rotl => Some(i32_binop!(HANDLER_IDX_I32_ROTL)),
            wasmparser::Operator::I32Rotr => Some(i32_binop!(HANDLER_IDX_I32_ROTR)),
            wasmparser::Operator::I32Eq => Some(i32_binop!(HANDLER_IDX_I32_EQ)),
            wasmparser::Operator::I32Ne => Some(i32_binop!(HANDLER_IDX_I32_NE)),
            wasmparser::Operator::I32LtS => Some(i32_binop!(HANDLER_IDX_I32_LT_S)),
            wasmparser::Operator::I32LtU => Some(i32_binop!(HANDLER_IDX_I32_LT_U)),
            wasmparser::Operator::I32LeS => Some(i32_binop!(HANDLER_IDX_I32_LE_S)),
            wasmparser::Operator::I32LeU => Some(i32_binop!(HANDLER_IDX_I32_LE_U)),
            wasmparser::Operator::I32GtS => Some(i32_binop!(HANDLER_IDX_I32_GT_S)),
            wasmparser::Operator::I32GtU => Some(i32_binop!(HANDLER_IDX_I32_GT_U)),
            wasmparser::Operator::I32GeS => Some(i32_binop!(HANDLER_IDX_I32_GE_S)),
            wasmparser::Operator::I32GeU => Some(i32_binop!(HANDLER_IDX_I32_GE_U)),
            wasmparser::Operator::I32Clz => Some(i32_unop!(HANDLER_IDX_I32_CLZ)),
            wasmparser::Operator::I32Ctz => Some(i32_unop!(HANDLER_IDX_I32_CTZ)),
            wasmparser::Operator::I32Popcnt => Some(i32_unop!(HANDLER_IDX_I32_POPCNT)),
            wasmparser::Operator::I32Eqz => Some(i32_unop!(HANDLER_IDX_I32_EQZ)),
            wasmparser::Operator::I32Extend8S => Some(i32_unop!(HANDLER_IDX_I32_EXTEND8_S)),
            wasmparser::Operator::I32Extend16S => Some(i32_unop!(HANDLER_IDX_I32_EXTEND16_S)),

            // i64
            wasmparser::Operator::I64Add => Some(i64_binop!(HANDLER_IDX_I64_ADD)),
            wasmparser::Operator::I64Sub => Some(i64_binop!(HANDLER_IDX_I64_SUB)),
            wasmparser::Operator::I64Mul => Some(i64_binop!(HANDLER_IDX_I64_MUL)),
            wasmparser::Operator::I64DivS => Some(i64_binop!(HANDLER_IDX_I64_DIV_S)),
            wasmparser::Operator::I64DivU => Some(i64_binop!(HANDLER_IDX_I64_DIV_U)),
            wasmparser::Operator::I64RemS => Some(i64_binop!(HANDLER_IDX_I64_REM_S)),
            wasmparser::Operator::I64RemU => Some(i64_binop!(HANDLER_IDX_I64_REM_U)),
            wasmparser::Operator::I64And => Some(i64_binop!(HANDLER_IDX_I64_AND)),
            wasmparser::Operator::I64Or => Some(i64_binop!(HANDLER_IDX_I64_OR)),
            wasmparser::Operator::I64Xor => Some(i64_binop!(HANDLER_IDX_I64_XOR)),
            wasmparser::Operator::I64Shl => Some(i64_binop!(HANDLER_IDX_I64_SHL)),
            wasmparser::Operator::I64ShrS => Some(i64_binop!(HANDLER_IDX_I64_SHR_S)),
            wasmparser::Operator::I64ShrU => Some(i64_binop!(HANDLER_IDX_I64_SHR_U)),
            wasmparser::Operator::I64Rotl => Some(i64_binop!(HANDLER_IDX_I64_ROTL)),
            wasmparser::Operator::I64Rotr => Some(i64_binop!(HANDLER_IDX_I64_ROTR)),
            wasmparser::Operator::I64Clz => Some(i64_unop!(HANDLER_IDX_I64_CLZ)),
            wasmparser::Operator::I64Ctz => Some(i64_unop!(HANDLER_IDX_I64_CTZ)),
            wasmparser::Operator::I64Popcnt => Some(i64_unop!(HANDLER_IDX_I64_POPCNT)),
            wasmparser::Operator::I64Extend8S => Some(i64_unop!(HANDLER_IDX_I64_EXTEND8_S)),
            wasmparser::Operator::I64Extend16S => Some(i64_unop!(HANDLER_IDX_I64_EXTEND16_S)),
            wasmparser::Operator::I64Extend32S => Some(i64_unop!(HANDLER_IDX_I64_EXTEND32_S)),
            // Comparisons leave an i32.
            wasmparser::Operator::I64Eqz => Some(unop!(
                I64Reg,
                I64RegOperand,
                take_i64_operand,
                I64,
                I32,
                HANDLER_IDX_I64_EQZ
            )),
            wasmparser::Operator::I64Eq => Some(i64_cmp!(HANDLER_IDX_I64_EQ)),
            wasmparser::Operator::I64Ne => Some(i64_cmp!(HANDLER_IDX_I64_NE)),
            wasmparser::Operator::I64LtS => Some(i64_cmp!(HANDLER_IDX_I64_LT_S)),
            wasmparser::Operator::I64LtU => Some(i64_cmp!(HANDLER_IDX_I64_LT_U)),
            wasmparser::Operator::I64GtS => Some(i64_cmp!(HANDLER_IDX_I64_GT_S)),
            wasmparser::Operator::I64GtU => Some(i64_cmp!(HANDLER_IDX_I64_GT_U)),
            wasmparser::Operator::I64LeS => Some(i64_cmp!(HANDLER_IDX_I64_LE_S)),
            wasmparser::Operator::I64LeU => Some(i64_cmp!(HANDLER_IDX_I64_LE_U)),
            wasmparser::Operator::I64GeS => Some(i64_cmp!(HANDLER_IDX_I64_GE_S)),
            wasmparser::Operator::I64GeU => Some(i64_cmp!(HANDLER_IDX_I64_GE_U)),

            // f32
            wasmparser::Operator::F32Add => Some(f32_binop!(HANDLER_IDX_F32_ADD)),
            wasmparser::Operator::F32Sub => Some(f32_binop!(HANDLER_IDX_F32_SUB)),
            wasmparser::Operator::F32Mul => Some(f32_binop!(HANDLER_IDX_F32_MUL)),
            wasmparser::Operator::F32Div => Some(f32_binop!(HANDLER_IDX_F32_DIV)),
            wasmparser::Operator::F32Min => Some(f32_binop!(HANDLER_IDX_F32_MIN)),
            wasmparser::Operator::F32Max => Some(f32_binop!(HANDLER_IDX_F32_MAX)),
            wasmparser::Operator::F32Copysign => Some(f32_binop!(HANDLER_IDX_F32_COPYSIGN)),
            wasmparser::Operator::F32Abs => Some(f32_unop!(HANDLER_IDX_F32_ABS)),
            wasmparser::Operator::F32Neg => Some(f32_unop!(HANDLER_IDX_F32_NEG)),
            wasmparser::Operator::F32Ceil => Some(f32_unop!(HANDLER_IDX_F32_CEIL)),
            wasmparser::Operator::F32Floor => Some(f32_unop!(HANDLER_IDX_F32_FLOOR)),
            wasmparser::Operator::F32Trunc => Some(f32_unop!(HANDLER_IDX_F32_TRUNC)),
            wasmparser::Operator::F32Nearest => Some(f32_unop!(HANDLER_IDX_F32_NEAREST)),
            wasmparser::Operator::F32Sqrt => Some(f32_unop!(HANDLER_IDX_F32_SQRT)),
            wasmparser::Operator::F32Eq => Some(f32_cmp!(HANDLER_IDX_F32_EQ)),
            wasmparser::Operator::F32Ne => Some(f32_cmp!(HANDLER_IDX_F32_NE)),
            wasmparser::Operator::F32Lt => Some(f32_cmp!(HANDLER_IDX_F32_LT)),
            wasmparser::Operator::F32Gt => Some(f32_cmp!(HANDLER_IDX_F32_GT)),
            wasmparser::Operator::F32Le => Some(f32_cmp!(HANDLER_IDX_F32_LE)),
            wasmparser::Operator::F32Ge => Some(f32_cmp!(HANDLER_IDX_F32_GE)),

            // f64
            wasmparser::Operator::F64Add => Some(f64_binop!(HANDLER_IDX_F64_ADD)),
            wasmparser::Operator::F64Sub => Some(f64_binop!(HANDLER_IDX_F64_SUB)),
            wasmparser::Operator::F64Mul => Some(f64_binop!(HANDLER_IDX_F64_MUL)),
            wasmparser::Operator::F64Div => Some(f64_binop!(HANDLER_IDX_F64_DIV)),
            wasmparser::Operator::F64Min => Some(f64_binop!(HANDLER_IDX_F64_MIN)),
            wasmparser::Operator::F64Max => Some(f64_binop!(HANDLER_IDX_F64_MAX)),
            wasmparser::Operator::F64Copysign => Some(f64_binop!(HANDLER_IDX_F64_COPYSIGN)),
            wasmparser::Operator::F64Abs => Some(f64_unop!(HANDLER_IDX_F64_ABS)),
            wasmparser::Operator::F64Neg => Some(f64_unop!(HANDLER_IDX_F64_NEG)),
            wasmparser::Operator::F64Ceil => Some(f64_unop!(HANDLER_IDX_F64_CEIL)),
            wasmparser::Operator::F64Floor => Some(f64_unop!(HANDLER_IDX_F64_FLOOR)),
            wasmparser::Operator::F64Trunc => Some(f64_unop!(HANDLER_IDX_F64_TRUNC)),
            wasmparser::Operator::F64Nearest => Some(f64_unop!(HANDLER_IDX_F64_NEAREST)),
            wasmparser::Operator::F64Sqrt => Some(f64_unop!(HANDLER_IDX_F64_SQRT)),
            wasmparser::Operator::F64Eq => Some(f64_cmp!(HANDLER_IDX_F64_EQ)),
            wasmparser::Operator::F64Ne => Some(f64_cmp!(HANDLER_IDX_F64_NE)),
            wasmparser::Operator::F64Lt => Some(f64_cmp!(HANDLER_IDX_F64_LT)),
            wasmparser::Operator::F64Gt => Some(f64_cmp!(HANDLER_IDX_F64_GT)),
            wasmparser::Operator::F64Le => Some(f64_cmp!(HANDLER_IDX_F64_LE)),
            wasmparser::Operator::F64Ge => Some(f64_cmp!(HANDLER_IDX_F64_GE)),

            // Conversions
            wasmparser::Operator::I64ExtendI32S => {
                Some(conv!(I32, I64, HANDLER_IDX_I64_EXTEND_I32_S))
            }
            wasmparser::Operator::I64ExtendI32U => {
                Some(conv!(I32, I64, HANDLER_IDX_I64_EXTEND_I32_U))
            }
            wasmparser::Operator::I32WrapI64 => Some(conv!(I64, I32, HANDLER_IDX_I32_WRAP_I64)),
            wasmparser::Operator::I32TruncF32S => {
                Some(conv!(F32, I32, HANDLER_IDX_I32_TRUNC_F32_S))
            }
            wasmparser::Operator::I32TruncF32U => {
                Some(conv!(F32, I32, HANDLER_IDX_I32_TRUNC_F32_U))
            }
            wasmparser::Operator::I32TruncF64S => {
                Some(conv!(F64, I32, HANDLER_IDX_I32_TRUNC_F64_S))
            }
            wasmparser::Operator::I32TruncF64U => {
                Some(conv!(F64, I32, HANDLER_IDX_I32_TRUNC_F64_U))
            }
            wasmparser::Operator::I64TruncF32S => {
                Some(conv!(F32, I64, HANDLER_IDX_I64_TRUNC_F32_S))
            }
            wasmparser::Operator::I64TruncF32U => {
                Some(conv!(F32, I64, HANDLER_IDX_I64_TRUNC_F32_U))
            }
            wasmparser::Operator::I64TruncF64S => {
                Some(conv!(F64, I64, HANDLER_IDX_I64_TRUNC_F64_S))
            }
            wasmparser::Operator::I64TruncF64U => {
                Some(conv!(F64, I64, HANDLER_IDX_I64_TRUNC_F64_U))
            }
            wasmparser::Operator::I32TruncSatF32S => {
                Some(conv!(F32, I32, HANDLER_IDX_I32_TRUNC_SAT_F32_S))
            }
            wasmparser::Operator::I32TruncSatF32U => {
                Some(conv!(F32, I32, HANDLER_IDX_I32_TRUNC_SAT_F32_U))
            }
            wasmparser::Operator::I32TruncSatF64S => {
                Some(conv!(F64, I32, HANDLER_IDX_I32_TRUNC_SAT_F64_S))
            }
            wasmparser::Operator::I32TruncSatF64U => {
                Some(conv!(F64, I32, HANDLER_IDX_I32_TRUNC_SAT_F64_U))
            }
            wasmparser::Operator::I64TruncSatF32S => {
                Some(conv!(F32, I64, HANDLER_IDX_I64_TRUNC_SAT_F32_S))
            }
            wasmparser::Operator::I64TruncSatF32U => {
                Some(conv!(F32, I64, HANDLER_IDX_I64_TRUNC_SAT_F32_U))
            }
            wasmparser::Operator::I64TruncSatF64S => {
                Some(conv!(F64, I64, HANDLER_IDX_I64_TRUNC_SAT_F64_S))
            }
            wasmparser::Operator::I64TruncSatF64U => {
                Some(conv!(F64, I64, HANDLER_IDX_I64_TRUNC_SAT_F64_U))
            }
            wasmparser::Operator::F32ConvertI32S => {
                Some(conv!(I32, F32, HANDLER_IDX_F32_CONVERT_I32_S))
            }
            wasmparser::Operator::F32ConvertI32U => {
                Some(conv!(I32, F32, HANDLER_IDX_F32_CONVERT_I32_U))
            }
            wasmparser::Operator::F32ConvertI64S => {
                Some(conv!(I64, F32, HANDLER_IDX_F32_CONVERT_I64_S))
            }
            wasmparser::Operator::F32ConvertI64U => {
                Some(conv!(I64, F32, HANDLER_IDX_F32_CONVERT_I64_U))
            }
            wasmparser::Operator::F64ConvertI32S => {
                Some(conv!(I32, F64, HANDLER_IDX_F64_CONVERT_I32_S))
            }
            wasmparser::Operator::F64ConvertI32U => {
                Some(conv!(I32, F64, HANDLER_IDX_F64_CONVERT_I32_U))
            }
            wasmparser::Operator::F64ConvertI64S => {
                Some(conv!(I64, F64, HANDLER_IDX_F64_CONVERT_I64_S))
            }
            wasmparser::Operator::F64ConvertI64U => {
                Some(conv!(I64, F64, HANDLER_IDX_F64_CONVERT_I64_U))
            }
            wasmparser::Operator::F32DemoteF64 => Some(conv!(F64, F32, HANDLER_IDX_F32_DEMOTE_F64)),
            wasmparser::Operator::F64PromoteF32 => {
                Some(conv!(F32, F64, HANDLER_IDX_F64_PROMOTE_F32))
            }
            wasmparser::Operator::I32ReinterpretF32 => {
                Some(conv!(F32, I32, HANDLER_IDX_I32_REINTERPRET_F32))
            }
            wasmparser::Operator::I64ReinterpretF64 => {
                Some(conv!(F64, I64, HANDLER_IDX_I64_REINTERPRET_F64))
            }
            wasmparser::Operator::F32ReinterpretI32 => {
                Some(conv!(I32, F32, HANDLER_IDX_F32_REINTERPRET_I32))
            }
            wasmparser::Operator::F64ReinterpretI64 => {
                Some(conv!(I64, F64, HANDLER_IDX_F64_REINTERPRET_I64))
            }

            // Loads
            wasmparser::Operator::I32Load { memarg } => {
                Some(load!(I32, HANDLER_IDX_I32_LOAD, memarg))
            }
            wasmparser::Operator::I64Load { memarg } => {
                Some(load!(I64, HANDLER_IDX_I64_LOAD, memarg))
            }
            wasmparser::Operator::F32Load { memarg } => {
                Some(load!(F32, HANDLER_IDX_F32_LOAD, memarg))
            }
            wasmparser::Operator::F64Load { memarg } => {
                Some(load!(F64, HANDLER_IDX_F64_LOAD, memarg))
            }
            wasmparser::Operator::I32Load8S { memarg } => {
                Some(load!(I32, HANDLER_IDX_I32_LOAD8_S, memarg))
            }
            wasmparser::Operator::I32Load8U { memarg } => {
                Some(load!(I32, HANDLER_IDX_I32_LOAD8_U, memarg))
            }
            wasmparser::Operator::I32Load16S { memarg } => {
                Some(load!(I32, HANDLER_IDX_I32_LOAD16_S, memarg))
            }
            wasmparser::Operator::I32Load16U { memarg } => {
                Some(load!(I32, HANDLER_IDX_I32_LOAD16_U, memarg))
            }
            wasmparser::Operator::I64Load8S { memarg } => {
                Some(load!(I64, HANDLER_IDX_I64_LOAD8_S, memarg))
            }
            wasmparser::Operator::I64Load8U { memarg } => {
                Some(load!(I64, HANDLER_IDX_I64_LOAD8_U, memarg))
            }
            wasmparser::Operator::I64Load16S { memarg } => {
                Some(load!(I64, HANDLER_IDX_I64_LOAD16_S, memarg))
            }
            wasmparser::Operator::I64Load16U { memarg } => {
                Some(load!(I64, HANDLER_IDX_I64_LOAD16_U, memarg))
            }
            wasmparser::Operator::I64Load32S { memarg } => {
                Some(load!(I64, HANDLER_IDX_I64_LOAD32_S, memarg))
            }
            wasmparser::Operator::I64Load32U { memarg } => {
                Some(load!(I64, HANDLER_IDX_I64_LOAD32_U, memarg))
            }
            wasmparser::Operator::I32AtomicLoad { memarg } => {
                Some(load!(I32, HANDLER_IDX_I32_ATOMIC_LOAD, memarg))
            }
            wasmparser::Operator::I64AtomicLoad { memarg } => {
                Some(load!(I64, HANDLER_IDX_I64_ATOMIC_LOAD, memarg))
            }
            wasmparser::Operator::I32AtomicLoad8U { memarg } => {
                Some(load!(I32, HANDLER_IDX_I32_ATOMIC_LOAD8_U, memarg))
            }
            wasmparser::Operator::I32AtomicLoad16U { memarg } => {
                Some(load!(I32, HANDLER_IDX_I32_ATOMIC_LOAD16_U, memarg))
            }
            wasmparser::Operator::I64AtomicLoad8U { memarg } => {
                Some(load!(I64, HANDLER_IDX_I64_ATOMIC_LOAD8_U, memarg))
            }
            wasmparser::Operator::I64AtomicLoad16U { memarg } => {
                Some(load!(I64, HANDLER_IDX_I64_ATOMIC_LOAD16_U, memarg))
            }
            wasmparser::Operator::I64AtomicLoad32U { memarg } => {
                Some(load!(I64, HANDLER_IDX_I64_ATOMIC_LOAD32_U, memarg))
            }

            // Stores
            wasmparser::Operator::I32Store { memarg } => {
                Some(store!(I32, HANDLER_IDX_I32_STORE, memarg))
            }
            wasmparser::Operator::I64Store { memarg } => {
                Some(store!(I64, HANDLER_IDX_I64_STORE, memarg))
            }
            wasmparser::Operator::F32Store { memarg } => {
                Some(store!(F32, HANDLER_IDX_F32_STORE, memarg))
            }
            wasmparser::Operator::F64Store { memarg } => {
                Some(store!(F64, HANDLER_IDX_F64_STORE, memarg))
            }
            wasmparser::Operator::I32Store8 { memarg } => {
                Some(store!(I32, HANDLER_IDX_I32_STORE8, memarg))
            }
            wasmparser::Operator::I32Store16 { memarg } => {
                Some(store!(I32, HANDLER_IDX_I32_STORE16, memarg))
            }
            wasmparser::Operator::I64Store8 { memarg } => {
                Some(store!(I64, HANDLER_IDX_I64_STORE8, memarg))
            }
            wasmparser::Operator::I64Store16 { memarg } => {
                Some(store!(I64, HANDLER_IDX_I64_STORE16, memarg))
            }
            wasmparser::Operator::I64Store32 { memarg } => {
                Some(store!(I64, HANDLER_IDX_I64_STORE32, memarg))
            }
            wasmparser::Operator::I32AtomicStore { memarg } => {
                Some(store!(I32, HANDLER_IDX_I32_ATOMIC_STORE, memarg))
            }
            wasmparser::Operator::I64AtomicStore { memarg } => {
                Some(store!(I64, HANDLER_IDX_I64_ATOMIC_STORE, memarg))
            }
            wasmparser::Operator::I32AtomicStore8 { memarg } => {
                Some(store!(I32, HANDLER_IDX_I32_ATOMIC_STORE8, memarg))
            }
            wasmparser::Operator::I32AtomicStore16 { memarg } => {
                Some(store!(I32, HANDLER_IDX_I32_ATOMIC_STORE16, memarg))
            }
            wasmparser::Operator::I64AtomicStore8 { memarg } => {
                Some(store!(I64, HANDLER_IDX_I64_ATOMIC_STORE8, memarg))
            }
            wasmparser::Operator::I64AtomicStore16 { memarg } => {
                Some(store!(I64, HANDLER_IDX_I64_ATOMIC_STORE16, memarg))
            }
            wasmparser::Operator::I64AtomicStore32 { memarg } => {
                Some(store!(I64, HANDLER_IDX_I64_ATOMIC_STORE32, memarg))
            }

            // Atomic read-modify-write
            wasmparser::Operator::I32AtomicRmwAdd { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_ADD, memarg))
            }
            wasmparser::Operator::I32AtomicRmw8AddU { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_8_ADD, memarg))
            }
            wasmparser::Operator::I32AtomicRmw16AddU { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_16_ADD, memarg))
            }
            wasmparser::Operator::I64AtomicRmwAdd { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_ADD, memarg))
            }
            wasmparser::Operator::I64AtomicRmw8AddU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_8_ADD, memarg))
            }
            wasmparser::Operator::I64AtomicRmw16AddU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_16_ADD, memarg))
            }
            wasmparser::Operator::I64AtomicRmw32AddU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_32_ADD, memarg))
            }
            wasmparser::Operator::I32AtomicRmwSub { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_SUB, memarg))
            }
            wasmparser::Operator::I32AtomicRmw8SubU { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_8_SUB, memarg))
            }
            wasmparser::Operator::I32AtomicRmw16SubU { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_16_SUB, memarg))
            }
            wasmparser::Operator::I64AtomicRmwSub { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_SUB, memarg))
            }
            wasmparser::Operator::I64AtomicRmw8SubU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_8_SUB, memarg))
            }
            wasmparser::Operator::I64AtomicRmw16SubU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_16_SUB, memarg))
            }
            wasmparser::Operator::I64AtomicRmw32SubU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_32_SUB, memarg))
            }
            wasmparser::Operator::I32AtomicRmwAnd { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_AND, memarg))
            }
            wasmparser::Operator::I32AtomicRmw8AndU { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_8_AND, memarg))
            }
            wasmparser::Operator::I32AtomicRmw16AndU { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_16_AND, memarg))
            }
            wasmparser::Operator::I64AtomicRmwAnd { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_AND, memarg))
            }
            wasmparser::Operator::I64AtomicRmw8AndU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_8_AND, memarg))
            }
            wasmparser::Operator::I64AtomicRmw16AndU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_16_AND, memarg))
            }
            wasmparser::Operator::I64AtomicRmw32AndU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_32_AND, memarg))
            }
            wasmparser::Operator::I32AtomicRmwOr { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_OR, memarg))
            }
            wasmparser::Operator::I32AtomicRmw8OrU { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_8_OR, memarg))
            }
            wasmparser::Operator::I32AtomicRmw16OrU { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_16_OR, memarg))
            }
            wasmparser::Operator::I64AtomicRmwOr { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_OR, memarg))
            }
            wasmparser::Operator::I64AtomicRmw8OrU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_8_OR, memarg))
            }
            wasmparser::Operator::I64AtomicRmw16OrU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_16_OR, memarg))
            }
            wasmparser::Operator::I64AtomicRmw32OrU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_32_OR, memarg))
            }
            wasmparser::Operator::I32AtomicRmwXor { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_XOR, memarg))
            }
            wasmparser::Operator::I32AtomicRmw8XorU { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_8_XOR, memarg))
            }
            wasmparser::Operator::I32AtomicRmw16XorU { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_16_XOR, memarg))
            }
            wasmparser::Operator::I64AtomicRmwXor { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_XOR, memarg))
            }
            wasmparser::Operator::I64AtomicRmw8XorU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_8_XOR, memarg))
            }
            wasmparser::Operator::I64AtomicRmw16XorU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_16_XOR, memarg))
            }
            wasmparser::Operator::I64AtomicRmw32XorU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_32_XOR, memarg))
            }
            wasmparser::Operator::I32AtomicRmwXchg { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_XCHG, memarg))
            }
            wasmparser::Operator::I32AtomicRmw8XchgU { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_8_XCHG, memarg))
            }
            wasmparser::Operator::I32AtomicRmw16XchgU { memarg } => {
                Some(rmw!(I32, HANDLER_IDX_RMW_I32_16_XCHG, memarg))
            }
            wasmparser::Operator::I64AtomicRmwXchg { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_XCHG, memarg))
            }
            wasmparser::Operator::I64AtomicRmw8XchgU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_8_XCHG, memarg))
            }
            wasmparser::Operator::I64AtomicRmw16XchgU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_16_XCHG, memarg))
            }
            wasmparser::Operator::I64AtomicRmw32XchgU { memarg } => {
                Some(rmw!(I64, HANDLER_IDX_RMW_I64_32_XCHG, memarg))
            }
            wasmparser::Operator::I32AtomicRmwCmpxchg { memarg } => {
                Some(cmpxchg!(I32, HANDLER_IDX_CMPXCHG_I32, memarg))
            }
            wasmparser::Operator::I32AtomicRmw8CmpxchgU { memarg } => {
                Some(cmpxchg!(I32, HANDLER_IDX_CMPXCHG_I32_8, memarg))
            }
            wasmparser::Operator::I32AtomicRmw16CmpxchgU { memarg } => {
                Some(cmpxchg!(I32, HANDLER_IDX_CMPXCHG_I32_16, memarg))
            }
            wasmparser::Operator::I64AtomicRmwCmpxchg { memarg } => {
                Some(cmpxchg!(I64, HANDLER_IDX_CMPXCHG_I64, memarg))
            }
            wasmparser::Operator::I64AtomicRmw8CmpxchgU { memarg } => {
                Some(cmpxchg!(I64, HANDLER_IDX_CMPXCHG_I64_8, memarg))
            }
            wasmparser::Operator::I64AtomicRmw16CmpxchgU { memarg } => {
                Some(cmpxchg!(I64, HANDLER_IDX_CMPXCHG_I64_16, memarg))
            }
            wasmparser::Operator::I64AtomicRmw32CmpxchgU { memarg } => {
                Some(cmpxchg!(I64, HANDLER_IDX_CMPXCHG_I64_32, memarg))
            }
            wasmparser::Operator::MemoryAtomicNotify { memarg } => {
                let count = allocator.pop(&ValueType::I32);
                let addr = allocator.pop(&ValueType::I32);
                let dst = allocator.push(ValueType::I32);
                Some(ProcessedInstr::AtomicWaitReg {
                    handler_index: HANDLER_IDX_MEMORY_ATOMIC_NOTIFY,
                    dst,
                    args: [addr, count, count],
                    offset: memarg.offset,
                })
            }
            wasmparser::Operator::MemoryAtomicWait32 { memarg } => {
                Some(wait!(I32, HANDLER_IDX_MEMORY_ATOMIC_WAIT32, memarg))
            }
            wasmparser::Operator::MemoryAtomicWait64 { memarg } => {
                Some(wait!(I64, HANDLER_IDX_MEMORY_ATOMIC_WAIT64, memarg))
            }
            wasmparser::Operator::AtomicFence => Some(ProcessedInstr::MemoryOpsReg {
                handler_index: HANDLER_IDX_ATOMIC_FENCE,
                dst: None,
                args: Box::new([]),
                data_index: 0,
            }),

            // Memory management
            wasmparser::Operator::MemorySize { .. } => {
                let dst = allocator.push(ValueType::I32);
                Some(ProcessedInstr::MemoryOpsReg {
                    handler_index: HANDLER_IDX_MEMORY_SIZE,
                    dst: Some(dst),
                    args: Box::new([]),
                    data_index: 0,
                })
            }
            wasmparser::Operator::MemoryGrow { .. } => {
                let delta = allocator.pop(&ValueType::I32);
                let dst = allocator.push(ValueType::I32);
                Some(ProcessedInstr::MemoryOpsReg {
                    handler_index: HANDLER_IDX_MEMORY_GROW,
                    dst: Some(dst),
                    args: Box::new([delta]),
                    data_index: 0,
                })
            }
            wasmparser::Operator::MemoryCopy { .. } => Some(ProcessedInstr::MemoryOpsReg {
                handler_index: HANDLER_IDX_MEMORY_COPY,
                dst: None,
                args: Box::new(pop_i32s::<3>(&mut allocator)),
                data_index: 0,
            }),
            wasmparser::Operator::MemoryInit { data_index, .. } => {
                Some(ProcessedInstr::MemoryOpsReg {
                    handler_index: HANDLER_IDX_MEMORY_INIT,
                    dst: None,
                    args: Box::new(pop_i32s::<3>(&mut allocator)),
                    data_index: *data_index,
                })
            }
            wasmparser::Operator::MemoryFill { .. } => Some(ProcessedInstr::MemoryOpsReg {
                handler_index: HANDLER_IDX_MEMORY_FILL,
                dst: None,
                args: Box::new(pop_i32s::<3>(&mut allocator)),
                data_index: 0,
            }),
            wasmparser::Operator::DataDrop { data_index } => Some(ProcessedInstr::DataDropReg {
                data_index: *data_index,
            }),

            // Control
            wasmparser::Operator::Block { blockty } | wasmparser::Operator::Loop { blockty } => {
                let is_loop = matches!(op, wasmparser::Operator::Loop { .. });
                enter_block(
                    &mut allocator,
                    &mut allocator_state_stack,
                    &mut control_info_stack,
                    *blockty,
                    module,
                    is_loop,
                );
                Some(ProcessedInstr::BlockReg { is_loop })
            }
            wasmparser::Operator::If { blockty } => {
                let cond_reg = allocator.pop(&ValueType::I32);
                let cond_reg = fold_local_get_arg(&mut instrs, &mut look_back, cond_reg);
                enter_block(
                    &mut allocator,
                    &mut allocator_state_stack,
                    &mut control_info_stack,
                    *blockty,
                    module,
                    false,
                );
                fixups.push(FixupInfo {
                    pc,
                    depth: 0,
                    source_regs: Vec::new(),
                });
                Some(ProcessedInstr::IfReg {
                    cond_reg,
                    else_target_ip: usize::MAX,
                })
            }
            wasmparser::Operator::Else => {
                // The else-branch starts from the if's entry state, params included.
                if let Some(state) = allocator_state_stack.last() {
                    allocator.restore_state(state);
                }
                if let Some(block) = control_info_stack.last() {
                    push_values(
                        &mut allocator,
                        &get_block_param_types(&block.block_type, module),
                    );
                }
                fixups.push(FixupInfo {
                    pc,
                    depth: 0,
                    source_regs: Vec::new(),
                });
                Some(ProcessedInstr::JumpReg {
                    target_ip: usize::MAX,
                })
            }
            wasmparser::Operator::End => {
                let block = control_info_stack.pop();
                let is_function_end = block.is_none();
                let result_type_vec = match &block {
                    Some(block) => get_block_result_types(&block.block_type, module),
                    None => result_types.to_vec(),
                };
                let mut source_regs = allocator.peek_regs_for_types(&result_type_vec);
                fold_local_get_args(&mut instrs, &mut look_back, &mut source_regs);
                let target_result_regs = block.map(|block| block.result_regs).unwrap_or_default();
                // Back to the depth at block entry, with the results on top.
                if let Some(saved_state) = allocator_state_stack.pop() {
                    allocator.restore_state(&saved_state);
                    push_values(&mut allocator, &result_type_vec);
                }
                Some(ProcessedInstr::EndReg {
                    source_regs: source_regs.into_boxed_slice(),
                    target_result_regs: target_result_regs.into_boxed_slice(),
                    is_function_end,
                })
            }
            wasmparser::Operator::Br { relative_depth } => {
                let depth = *relative_depth as usize;
                let (mut source_regs, target_result_regs) =
                    branch_regs(&control_info_stack, depth, &allocator, result_types);
                fold_local_get_args(&mut instrs, &mut look_back, &mut source_regs);
                fixups.push(FixupInfo {
                    pc,
                    depth,
                    source_regs: source_regs.clone(),
                });
                Some(ProcessedInstr::BrReg {
                    target_ip: usize::MAX,
                    result_copies: BranchCopies::new(
                        source_regs.into_boxed_slice(),
                        target_result_regs.into_boxed_slice(),
                    ),
                })
            }
            wasmparser::Operator::BrIf { relative_depth } => {
                let depth = *relative_depth as usize;
                // Only the condition is folded: the values stay on the stack
                // when the branch is not taken.
                let cond_reg = allocator.pop(&ValueType::I32);
                let cond_reg = fold_local_get_arg(&mut instrs, &mut look_back, cond_reg);
                let (source_regs, target_result_regs) =
                    branch_regs(&control_info_stack, depth, &allocator, result_types);
                fixups.push(FixupInfo {
                    pc,
                    depth,
                    source_regs: source_regs.clone(),
                });
                Some(ProcessedInstr::BrIfReg {
                    target_ip: usize::MAX,
                    cond_reg,
                    result_copies: BranchCopies::new(
                        source_regs.into_boxed_slice(),
                        target_result_regs.into_boxed_slice(),
                    ),
                })
            }
            wasmparser::Operator::BrTable { targets } => {
                let index_reg = allocator.pop(&ValueType::I32);
                let depths: Vec<u32> = targets.targets().collect::<Result<_, _>>()?;
                let table_targets = depths
                    .iter()
                    .map(|depth| {
                        let (_, target_result_regs) = branch_regs(
                            &control_info_stack,
                            *depth as usize,
                            &allocator,
                            result_types,
                        );
                        (*depth, usize::MAX, target_result_regs.into_boxed_slice())
                    })
                    .collect();
                let (mut source_regs, default_result_regs) = branch_regs(
                    &control_info_stack,
                    targets.default() as usize,
                    &allocator,
                    result_types,
                );
                // The index sits above the values on the stack.
                source_regs.push(index_reg);
                fold_local_get_args(&mut instrs, &mut look_back, &mut source_regs);
                let index_reg = source_regs.pop().unwrap();
                Some(ProcessedInstr::BrTableReg(Box::new(BrTableData {
                    targets: table_targets,
                    default_target: (
                        targets.default(),
                        usize::MAX,
                        default_result_regs.into_boxed_slice(),
                    ),
                    index_reg,
                    source_regs: source_regs.into_boxed_slice(),
                })))
            }
            wasmparser::Operator::Return => {
                let mut result_regs = pop_values(&mut allocator, result_types);
                fold_local_get_args(&mut instrs, &mut look_back, &mut result_regs);
                Some(ProcessedInstr::ReturnReg {
                    result_regs: result_regs.into_boxed_slice(),
                })
            }
            wasmparser::Operator::Call { function_index } => {
                #[cfg(feature = "call_graph")]
                if let Some(b) = cg_builder.as_mut() {
                    b.record_call(FuncIdx(func_index as u32), FuncIdx(*function_index));
                }
                if let Some(ImportDesc::WasiFunc(wasi_type)) =
                    get_imported_func_desc(module, *function_index)
                {
                    let func_type = wasi_type.expected_func_type();
                    let mut param_regs = pop_values(&mut allocator, &func_type.params);
                    fold_local_get_args(&mut instrs, &mut look_back, &mut param_regs);
                    let result_reg = func_type.results.first().map(|ty| allocator.push(*ty));
                    Some(ProcessedInstr::CallWasiReg {
                        wasi_func_type: *wasi_type,
                        param_regs: param_regs.into_boxed_slice(),
                        result_reg,
                    })
                } else {
                    let func_type = get_func_type(module, *function_index);
                    let mut param_regs = pop_values(&mut allocator, &func_type.params);
                    fold_local_get_args(&mut instrs, &mut look_back, &mut param_regs);
                    let result_regs = push_values(&mut allocator, &func_type.results);
                    Some(ProcessedInstr::CallReg {
                        func_idx: FuncIdx(*function_index),
                        param_regs: param_regs.into_boxed_slice(),
                        result_regs,
                    })
                }
            }
            wasmparser::Operator::CallIndirect {
                type_index,
                table_index,
                ..
            } => {
                let func_type = module
                    .types
                    .get(*type_index as usize)
                    .cloned()
                    .unwrap_or_default();
                let index_reg = allocator.pop(&ValueType::I32);
                let mut regs = pop_values(&mut allocator, &func_type.params);
                // The index sits above the params on the stack.
                regs.push(index_reg);
                fold_local_get_args(&mut instrs, &mut look_back, &mut regs);
                let index_reg = regs.pop().unwrap();
                let result_regs = push_values(&mut allocator, &func_type.results);
                #[cfg(feature = "call_graph")]
                if let Some(b) = cg_builder.as_mut() {
                    b.record_call_indirect(FuncIdx(func_index as u32), TypeIdx(*type_index));
                }
                Some(ProcessedInstr::CallIndirectReg {
                    type_idx: TypeIdx(*type_index),
                    table_idx: TableIdx(*table_index),
                    index_reg,
                    param_regs: regs.into_boxed_slice(),
                    result_regs,
                })
            }
            wasmparser::Operator::Nop => None,
            wasmparser::Operator::Unreachable => Some(ProcessedInstr::UnreachableReg),
            wasmparser::Operator::Drop => {
                allocator.pop_any();
                dropped_since_emit = true;
                None
            }

            // Parametric
            wasmparser::Operator::TypedSelect { ty } => {
                let cond = allocator.pop(&ValueType::I32);
                let val_type = match_value_type(*ty);
                let handler_index = match val_type {
                    ValueType::RefType(_) => HANDLER_IDX_SELECT_I64,
                    _ => numeric_handler(val_type, SELECT_HANDLERS),
                };
                Some(select_instr(
                    &mut allocator,
                    &mut instrs,
                    &mut look_back,
                    cond,
                    val_type,
                    handler_index,
                ))
            }
            wasmparser::Operator::Select => {
                // Untyped `select` takes numeric values only; the stack says which.
                let cond = allocator.pop(&ValueType::I32);
                let val_type = allocator.peek_type().copied().unwrap_or(ValueType::I32);
                let handler_index = numeric_handler(val_type, SELECT_HANDLERS);
                Some(select_instr(
                    &mut allocator,
                    &mut instrs,
                    &mut look_back,
                    cond,
                    val_type,
                    handler_index,
                ))
            }

            // References and tables
            wasmparser::Operator::RefNull { hty } => {
                let ref_type = match hty {
                    wasmparser::HeapType::Func => RefType::FuncRef,
                    _ => RefType::ExternalRef,
                };
                let dst = allocator.push(ValueType::RefType(ref_type));
                Some(ProcessedInstr::TableRefReg {
                    handler_index: HANDLER_IDX_REF_NULL,
                    table_idx: 0,
                    regs: [dst.index(), 0, 0],
                    ref_type,
                })
            }
            wasmparser::Operator::RefIsNull => {
                let src = allocator.pop(&ValueType::RefType(RefType::FuncRef));
                let dst = allocator.push(ValueType::I32);
                Some(ProcessedInstr::TableRefReg {
                    handler_index: HANDLER_IDX_REF_IS_NULL,
                    table_idx: 0,
                    regs: [dst.index(), src.index(), 0],
                    ref_type: RefType::FuncRef,
                })
            }
            wasmparser::Operator::TableGet { table } => {
                let elem_type = get_table_element_type(module, *table);
                let idx = allocator.pop(&ValueType::I32);
                let dst = allocator.push(elem_type);
                let ref_type = match elem_type {
                    ValueType::RefType(ref_type) => ref_type,
                    _ => RefType::FuncRef,
                };
                Some(ProcessedInstr::TableRefReg {
                    handler_index: HANDLER_IDX_TABLE_GET,
                    table_idx: *table,
                    regs: [dst.index(), idx.index(), 0],
                    ref_type,
                })
            }
            wasmparser::Operator::TableSet { table } => {
                let elem_type = get_table_element_type(module, *table);
                let val = allocator.pop(&elem_type);
                let idx = allocator.pop(&ValueType::I32);
                Some(ProcessedInstr::TableRefReg {
                    handler_index: HANDLER_IDX_TABLE_SET,
                    table_idx: *table,
                    regs: [idx.index(), val.index(), 0],
                    ref_type: RefType::FuncRef,
                })
            }
            wasmparser::Operator::TableFill { table } => {
                let elem_type = get_table_element_type(module, *table);
                let n = allocator.pop(&ValueType::I32);
                let val = allocator.pop(&elem_type);
                let i = allocator.pop(&ValueType::I32);
                Some(ProcessedInstr::TableRefReg {
                    handler_index: HANDLER_IDX_TABLE_FILL,
                    table_idx: *table,
                    regs: [i.index(), val.index(), n.index()],
                    ref_type: RefType::FuncRef,
                })
            }

            _ => panic!("Unsupported instruction: {:?}", op),
        };

        // Block boundaries, for the branch resolution of Phase 2.
        match &op {
            wasmparser::Operator::Block { .. } | wasmparser::Operator::Loop { .. } => {
                open_blocks.push((pc, false, None))
            }
            wasmparser::Operator::If { .. } => open_blocks.push((pc, true, None)),
            wasmparser::Operator::Else => match open_blocks.last_mut() {
                Some((_, true, else_pc @ None)) => *else_pc = Some(pc + 1),
                _ => {
                    return Err(Box::new(RuntimeError::InvalidWasm(
                        "Else without corresponding If or If already has Else",
                    )))
                }
            },
            wasmparser::Operator::End => match open_blocks.pop() {
                Some((start_pc, is_if, else_pc)) => {
                    block_end_map.insert(start_pc, pc + 1);
                    if is_if {
                        if_else_map.insert(start_pc, else_pc.unwrap_or(pc + 1));
                    }
                }
                // The function's own end.
                None if ops.peek().is_none() => {}
                None => return Err(Box::new(RuntimeError::InvalidWasm("Unmatched EndMarker"))),
            },
            _ => {}
        }

        if let Some(instr) = instr {
            instrs.push(instr);
            dropped_since_emit = false;
            if matches!(
                op,
                wasmparser::Operator::Br { .. }
                    | wasmparser::Operator::BrTable { .. }
                    | wasmparser::Operator::Return
                    | wasmparser::Operator::Unreachable
            ) {
                unreachable_depth = 1;
            }
        }
    }

    if !open_blocks.is_empty() {
        return Err(Box::new(RuntimeError::InvalidWasm(
            "Unclosed control block at end of function",
        )));
    }

    Ok(DecodedBody {
        instrs,
        fixups,
        block_end_map,
        if_else_map,
        reg_allocation: allocator.finalize(),
        const_pool,
    })
}

/// `fold_local_get_args` for one register.
fn fold_local_get_arg(instrs: &mut [ProcessedInstr], at: &mut usize, reg: Reg) -> Reg {
    let mut regs = [reg];
    fold_local_get_args(instrs, at, &mut regs);
    regs[0]
}

/// Lets an instruction that takes plain registers read them from the locals.
/// Each `local.get` copy before `*at` that feeds a register in `regs` becomes a no-op, and that register is replaced by the local's own.
/// Any other instruction that wrote the register is stepped over:
/// it wrote no local, so the copies before it still hold.
fn fold_local_get_args(instrs: &mut [ProcessedInstr], at: &mut usize, regs: &mut [Reg]) {
    macro_rules! copied_local {
        ($handler:expr, $src1:expr, $operand:ident, $variant:ident) => {
            match ($handler, $src1) {
                (&HANDLER_IDX_LOCAL_GET, $operand::Reg(l)) => Some(Reg::$variant(*l)),
                _ => None,
            }
        };
    }
    for reg in regs.iter_mut().rev() {
        if *at == 0 {
            break;
        }
        let local = match (&instrs[*at - 1], *reg) {
            (
                ProcessedInstr::I32Reg {
                    handler_index,
                    dst: I32RegOperand::Reg(d),
                    src1,
                    ..
                },
                Reg::I32(r),
            ) if *d == r => copied_local!(handler_index, src1, I32RegOperand, I32),
            (
                ProcessedInstr::I64Reg {
                    handler_index,
                    dst: I64RegOperand::Reg(d),
                    src1,
                    ..
                },
                Reg::I64(r),
            ) if *d == r => copied_local!(handler_index, src1, I64RegOperand, I64),
            (
                ProcessedInstr::F32Reg {
                    handler_index,
                    dst: F32RegOperand::Reg(d),
                    src1,
                    ..
                },
                Reg::F32(r),
            ) if *d == r => copied_local!(handler_index, src1, F32RegOperand, F32),
            (
                ProcessedInstr::F64Reg {
                    handler_index,
                    dst: F64RegOperand::Reg(d),
                    src1,
                    ..
                },
                Reg::F64(r),
            ) if *d == r => copied_local!(handler_index, src1, F64RegOperand, F64),
            _ => break,
        };
        *at -= 1;
        if let Some(local) = local {
            *reg = local;
            instrs[*at] = ProcessedInstr::NopReg;
        }
    }
}

/// The registers a branch of `depth` copies: the values on top of the operand stack, 
/// and the target's result registers.
/// A loop takes its parameters where they already are, so it needs no copy.
/// Past the outermost block the target is the function end, whose registers the fixup fills in.
fn branch_regs(
    control_info_stack: &[ControlBlockInfo],
    depth: usize,
    allocator: &RegAllocator,
    result_types: &[ValueType],
) -> (Vec<Reg>, Vec<Reg>) {
    let Some(target) = control_info_stack
        .len()
        .checked_sub(1 + depth)
        .map(|i| &control_info_stack[i])
    else {
        return (allocator.peek_regs_for_types(result_types), Vec::new());
    };
    if target.is_loop {
        return (Vec::new(), Vec::new());
    }
    let types: Vec<ValueType> = target.result_regs.iter().map(Reg::value_type).collect();
    (
        allocator.peek_regs_for_types(&types),
        target.result_regs.clone(),
    )
}

/// Parses a WebAssembly binary file and populates the module structure.
///
/// This is the main entry point for loading a WebAssembly module.
/// It reads the binary file, parses all sections using wasmparser, and preprocesses instructions for efficient interpretation.
///
/// Output returned by [`parse_bytecode`].
pub struct ParseOutput {
    /// Static call graph built during parsing (only present with the `call_graph` feature).
    #[cfg(feature = "call_graph")]
    pub call_graph: CallGraph,
}

/// # Arguments
///
/// * `module` - The module structure to populate
/// * `path` - Path to the WebAssembly binary file
pub fn parse_bytecode(
    mut module: &mut Module,
    path: &str,
) -> Result<ParseOutput, Box<dyn std::error::Error>> {
    let mut current_func_index = module.num_imported_funcs;
    #[cfg(feature = "call_graph")]
    let mut cg_builder = CallGraphBuilder::new();

    let mut buf = Vec::new();
    let parser = Parser::new(0);

    let mut file = File::open(path)?;
    file.read_to_end(&mut buf)?;

    for payload in parser.parse_all(&buf) {
        match payload? {
            Version {
                num,
                encoding: _,
                range: _,
            } => {
                if num != 0x01 {
                    return Err(Box::new(ParserError::VersionError));
                }
            }

            TypeSection(body) => {
                decode_type_section(body, &mut module)?;
            }

            FunctionSection(body) => {
                decode_func_section(
                    body,
                    &mut module,
                    #[cfg(feature = "call_graph")]
                    Some(&mut cg_builder),
                )?;
            }

            ImportSection(body) => {
                decode_import_section(
                    body,
                    &mut module,
                    #[cfg(feature = "call_graph")]
                    Some(&mut cg_builder),
                )?;
                current_func_index = module.num_imported_funcs;
            }
            ExportSection(body) => {
                decode_export_section(body, &mut module)?;
            }

            TableSection(body) => {
                decode_table_section(body, &mut module)?;
            }

            MemorySection(body) => {
                decode_mem_section(body, &mut module)?;
            }

            TagSection(_) => { /* ... */ }

            GlobalSection(body) => {
                decode_global_section(body, &mut module)?;
            }

            StartSection { func, .. } => {
                module.start = Some(Start {
                    func: FuncIdx(func),
                });
            }

            ElementSection(body) => {
                decode_elem_section(body, &mut module)?;
            }

            DataCountSection { .. } => { /* ... */ }

            DataSection(body) => {
                decode_data_section(body, &mut module)?;
            }

            CodeSectionStart { .. } => { /* ... */ }
            CodeSectionEntry(body) => {
                let result = decode_code_section(
                    body,
                    &mut module,
                    current_func_index,
                    #[cfg(feature = "call_graph")]
                    Some(&mut cg_builder),
                );
                result?;
                current_func_index += 1;
            }

            ModuleSection { .. } => { /* ... */ }
            InstanceSection(_) => { /* ... */ }
            CoreTypeSection(_) => { /* ... */ }
            ComponentSection { .. } => { /* ... */ }
            ComponentInstanceSection(_) => { /* ... */ }
            ComponentAliasSection(_) => { /* ... */ }
            ComponentTypeSection(_) => { /* ... */ }
            ComponentCanonicalSection(_) => { /* ... */ }
            ComponentStartSection { .. } => { /* ... */ }
            ComponentImportSection(_) => { /* ... */ }
            ComponentExportSection(_) => { /* ... */ }

            CustomSection(_) => { /* ... */ }

            UnknownSection { .. } => { /* ... */ }

            End(_) => {}
        }
    }

    Ok(ParseOutput {
        #[cfg(feature = "call_graph")]
        call_graph: cg_builder.finish(),
    })
}
