use std::fs::File;
use std::io::Read;
use wasmparser::{
    ExternalKind, FunctionBody, Parser, Payload::*, SectionLimited, TypeRef, ValType,
};

use crate::error::{ParserError, RuntimeError};
use crate::execution::slots::{Slot, SlotAllocator};
use crate::execution::vm::*;
use crate::structure::{instructions::*, module::*, types::*};
use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::rc::Rc;
use std::sync::LazyLock;

/// Control block information for tracking result slots
#[derive(Debug, Clone)]
struct ControlBlockInfo {
    block_type: wasmparser::BlockType,
    is_loop: bool,
    result_slots: Vec<Slot>,
    param_slots: Vec<Slot>,
}

// Cache key for block type arity calculations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum BlockTypeKey {
    Empty,
    SingleType(ValueType),
    FuncType(TypeIdx),
}

impl From<&wasmparser::BlockType> for BlockTypeKey {
    fn from(block_type: &wasmparser::BlockType) -> Self {
        match block_type {
            wasmparser::BlockType::Empty => BlockTypeKey::Empty,
            wasmparser::BlockType::Type(val_type) => {
                BlockTypeKey::SingleType(match_value_type(*val_type))
            }
            wasmparser::BlockType::FuncType(type_idx) => BlockTypeKey::FuncType(TypeIdx(*type_idx)),
        }
    }
}

// Cache for block arity calculations
struct BlockArityCache {
    block_arity_cache: FxHashMap<BlockTypeKey, usize>,
    loop_parameter_arity_cache: FxHashMap<BlockTypeKey, usize>,
}

impl BlockArityCache {
    fn new() -> Self {
        Self {
            block_arity_cache: FxHashMap::default(),
            loop_parameter_arity_cache: FxHashMap::default(),
        }
    }
}

static WASI_FUNCTION_MAP: LazyLock<FxHashMap<&'static str, WasiFuncType>> = LazyLock::new(|| {
    let mut map = FxHashMap::default();
    map.insert("proc_exit", WasiFuncType::ProcExit);
    map.insert("fd_write", WasiFuncType::FdWrite);
    map.insert("fd_read", WasiFuncType::FdRead);
    map.insert("random_get", WasiFuncType::RandomGet);
    map.insert("fd_prestat_get", WasiFuncType::FdPrestatGet);
    map.insert("fd_prestat_dir_name", WasiFuncType::FdPrestatDirName);
    map.insert("fd_close", WasiFuncType::FdClose);
    map.insert("environ_get", WasiFuncType::EnvironGet);
    map.insert("environ_sizes_get", WasiFuncType::EnvironSizesGet);
    map.insert("args_get", WasiFuncType::ArgsGet);
    map.insert("args_sizes_get", WasiFuncType::ArgsSizesGet);
    map.insert("clock_time_get", WasiFuncType::ClockTimeGet);
    map.insert("clock_res_get", WasiFuncType::ClockResGet);
    map.insert("sched_yield", WasiFuncType::SchedYield);
    map.insert("fd_fdstat_get", WasiFuncType::FdFdstatGet);
    map.insert("path_open", WasiFuncType::PathOpen);
    map.insert("fd_seek", WasiFuncType::FdSeek);
    map.insert("fd_tell", WasiFuncType::FdTell);
    map.insert("fd_sync", WasiFuncType::FdSync);
    map.insert("fd_filestat_get", WasiFuncType::FdFilestatGet);
    map.insert("fd_readdir", WasiFuncType::FdReaddir);
    map.insert("fd_pread", WasiFuncType::FdPread);
    map.insert("fd_datasync", WasiFuncType::FdDatasync);
    map.insert("fd_fdstat_set_flags", WasiFuncType::FdFdstatSetFlags);
    map.insert("fd_filestat_set_size", WasiFuncType::FdFilestatSetSize);
    map.insert("fd_pwrite", WasiFuncType::FdPwrite);
    map.insert("path_create_directory", WasiFuncType::PathCreateDirectory);
    map.insert("path_filestat_get", WasiFuncType::PathFilestatGet);
    map.insert(
        "path_filestat_set_times",
        WasiFuncType::PathFilestatSetTimes,
    );
    map.insert("path_readlink", WasiFuncType::PathReadlink);
    map.insert("path_remove_directory", WasiFuncType::PathRemoveDirectory);
    map.insert("path_unlink_file", WasiFuncType::PathUnlinkFile);
    map.insert("poll_oneoff", WasiFuncType::PollOneoff);
    map.insert("proc_raise", WasiFuncType::ProcRaise);
    map.insert("fd_advise", WasiFuncType::FdAdvise);
    map.insert("fd_allocate", WasiFuncType::FdAllocate);
    map.insert("fd_fdstat_set_rights", WasiFuncType::FdFdstatSetRights);
    map.insert("fd_renumber", WasiFuncType::FdRenumber);
    map.insert("fd_filestat_set_times", WasiFuncType::FdFilestatSetTimes);
    map.insert("path_link", WasiFuncType::PathLink);
    map.insert("path_rename", WasiFuncType::PathRename);
    map.insert("path_symlink", WasiFuncType::PathSymlink);
    map.insert("sock_accept", WasiFuncType::SockAccept);
    map.insert("sock_recv", WasiFuncType::SockRecv);
    map.insert("sock_send", WasiFuncType::SockSend);
    map.insert("sock_shutdown", WasiFuncType::SockShutdown);
    map
});

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

fn types_to_vec(types: &[ValType], vec: &mut Vec<ValueType>) {
    for t in types.iter() {
        vec.push(match_value_type(*t));
    }
}

fn calculate_block_arity(
    block_type: &wasmparser::BlockType,
    module: &Module,
    cache: &mut BlockArityCache,
) -> usize {
    let key = BlockTypeKey::from(block_type);

    if let Some(&arity) = cache.block_arity_cache.get(&key) {
        return arity;
    }

    let arity = match block_type {
        wasmparser::BlockType::Empty => 0,
        wasmparser::BlockType::Type(_) => 1,
        wasmparser::BlockType::FuncType(type_idx) => {
            if let Some(func_type) = module.types.get(*type_idx as usize) {
                func_type.results.len()
            } else {
                0 // Fallback to 0 if invalid type index
            }
        }
    };

    cache.block_arity_cache.insert(key, arity);
    arity
}

fn calculate_loop_parameter_arity(
    block_type: &wasmparser::BlockType,
    module: &Module,
    cache: &mut BlockArityCache,
) -> usize {
    let key = BlockTypeKey::from(block_type);

    if let Some(&arity) = cache.loop_parameter_arity_cache.get(&key) {
        return arity;
    }

    let arity = match block_type {
        wasmparser::BlockType::Empty => 0,
        wasmparser::BlockType::Type(_) => 0, // Single type means no parameters for loop
        wasmparser::BlockType::FuncType(type_idx) => {
            if let Some(func_type) = module.types.get(*type_idx as usize) {
                func_type.params.len() // For loops, we want parameter count, not result count
            } else {
                0 // Fallback to 0 if invalid type index
            }
        }
    };

    cache.loop_parameter_arity_cache.insert(key, arity);
    arity
}

/// Get result types for a block type
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

/// Get param types for a block type (for multi-value blocks)
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

        Rc::get_mut(&mut module.types)
            .unwrap()
            .push(crate::structure::types::FuncType { params, results });
    }
    Ok(())
}

fn decode_func_section(
    body: SectionLimited<'_, u32>,
    module: &mut Module,
) -> Result<(), Box<dyn std::error::Error>> {
    for func in body {
        let index = func?;
        let typeidx = TypeIdx(index);
        module.funcs.push(Func {
            type_: typeidx,
            locals: Vec::new(),
            body: Rc::new(Vec::new()),
            slot_allocation: None,
            result_slot: None,
        });
    }

    Ok(())
}

fn decode_import_section(
    body: SectionLimited<'_, wasmparser::Import<'_>>,
    module: &mut Module,
) -> Result<(), Box<dyn std::error::Error>> {
    for import in body {
        let import = import?;
        let desc = match import.ty {
            TypeRef::Func(type_index) => {
                if import.module == "wasi_snapshot_preview1" {
                    if let Some(wasi_func_type) = parse_wasi_function(&import.name) {
                        module.num_imported_funcs += 1;
                        ImportDesc::WasiFunc(wasi_func_type)
                    } else {
                        module.num_imported_funcs += 1;
                        ImportDesc::Func(TypeIdx(type_index))
                    }
                } else {
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
                ImportDesc::Mem(MemType(limits))
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

fn parse_wasi_function(name: &str) -> Option<WasiFuncType> {
    WASI_FUNCTION_MAP.get(name).copied()
}

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
            type_: MemType(limits),
        });
    }
    Ok(())
}

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

fn decode_code_section(
    body: FunctionBody<'_>,
    module: &mut Module,
    func_index: usize,
    enable_superinstructions: bool,
    cache: &mut BlockArityCache,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut locals: Vec<(u32, ValueType)> = Vec::new();
    for pair in body.get_locals_reader()? {
        let (cnt, ty) = pair?;
        let ty = match_value_type(ty);
        locals.push((cnt, ty));
    }

    let ops_reader = body.get_operators_reader()?;
    let ops_iter = ops_reader.into_iter_with_offsets();

    // Get the function's parameter and result types for slot allocation
    let relative_func_index_pre = func_index - module.num_imported_funcs;
    let (param_types, result_types): (Vec<ValueType>, Vec<ValueType>) =
        if let Some(func) = module.funcs.get(relative_func_index_pre) {
            if let Some(func_type) = module.types.get(func.type_.0 as usize) {
                (func_type.params.clone(), func_type.results.clone())
            } else {
                (Vec::new(), Vec::new())
            }
        } else {
            (Vec::new(), Vec::new())
        };

    // Phase 1: Decode instructions and get necessary info for preprocessing
    let (
        mut processed_instrs,
        mut fixups,
        block_end_map,
        if_else_map,
        block_type_map,
        slot_allocation,
        result_slot,
        block_result_slots_map,
    ) = decode_processed_instrs_and_fixups(
        ops_iter,
        module,
        enable_superinstructions,
        &locals,
        &param_types,
        &result_types,
    )?;

    let relative_func_index = func_index - module.num_imported_funcs;
    if let Some(func) = module.funcs.get_mut(relative_func_index) {
        func.locals = locals;
    } else {
        return Err(Box::new(RuntimeError::InvalidWasm(
            "Invalid function index during code decoding",
        )) as Box<dyn std::error::Error>);
    }
    // Phase 2 & 3: Preprocess instructions for this function
    preprocess_instructions(
        &mut processed_instrs,
        &mut fixups,
        &block_end_map,
        &if_else_map,
        &block_type_map,
        &block_result_slots_map,
        module,
        cache,
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    let body_rc = Rc::new(processed_instrs);

    // Store function body and metadata in module
    if let Some(func) = module.funcs.get_mut(relative_func_index) {
        func.body = body_rc;
        // Store slot mode metadata (None for stack mode)
        func.slot_allocation = slot_allocation.clone();
        func.result_slot = result_slot;
    } else {
        return Err(Box::new(RuntimeError::InvalidWasm(
            "Invalid function index when storing body",
        )) as Box<dyn std::error::Error>);
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct FixupInfo {
    pc: usize,
    original_wasm_depth: usize,
    is_if_false_jump: bool,
    is_else_jump: bool,
    source_slots: Vec<Slot>,
}

// Phase 2: Resolve Br, BrIf, If, Else jumps using maps and control stack simulation
// Phase 3: Resolve BrTable jumps similarly
fn preprocess_instructions(
    processed: &mut Vec<ProcessedInstr>,
    fixups: &mut Vec<FixupInfo>,
    block_end_map: &FxHashMap<usize, usize>,
    if_else_map: &FxHashMap<usize, usize>,
    block_type_map: &FxHashMap<usize, wasmparser::BlockType>,
    block_result_slots_map: &FxHashMap<usize, (Vec<Slot>, bool)>,
    module: &Module,
    cache: &mut BlockArityCache,
) -> Result<(), RuntimeError> {
    // --- Phase 2: Resolve Br, BrIf, If, Else jumps ---

    // Control stack stores: (pc, is_loop, block_type, runtime_label_stack_idx)
    let mut current_control_stack_pass2: Vec<(usize, bool, wasmparser::BlockType)> = Vec::new();

    for fixup_index in 0..fixups.len() {
        let current_fixup_pc = fixups[fixup_index].pc;
        let current_fixup_depth = fixups[fixup_index].original_wasm_depth;
        let is_if_false_jump = fixups[fixup_index].is_if_false_jump;
        let is_else_jump = fixups[fixup_index].is_else_jump;

        let is_br_table_fixup = processed
            .get(current_fixup_pc)
            .map_or(false, |instr| instr.handler_index() == HANDLER_IDX_BR_TABLE);

        if current_fixup_depth == usize::MAX || is_br_table_fixup {
            continue;
        }

        // --- Rebuild control stack state up to the fixup instruction ---
        current_control_stack_pass2.clear();
        for (pc, instr) in processed.iter().enumerate().take(current_fixup_pc + 1) {
            match instr.handler_index() {
                HANDLER_IDX_BLOCK | HANDLER_IDX_IF => {
                    let block_type = block_type_map
                        .get(&pc)
                        .cloned()
                        .unwrap_or(wasmparser::BlockType::Empty);
                    current_control_stack_pass2.push((pc, false, block_type));
                }
                HANDLER_IDX_LOOP => {
                    let block_type = block_type_map
                        .get(&pc)
                        .cloned()
                        .unwrap_or(wasmparser::BlockType::Empty);
                    current_control_stack_pass2.push((pc, true, block_type));
                }
                HANDLER_IDX_END => {
                    if !current_control_stack_pass2.is_empty() {
                        current_control_stack_pass2.pop();
                    }
                }
                _ => {}
            }
        }

        if current_control_stack_pass2.len() <= current_fixup_depth {
            // Depth exceeds control stack - this is a branch to function level (return)
            // Set target_ip to end of function (processed.len())
            let function_end_ip = processed.len();
            if let Some(instr_to_patch) = processed.get_mut(current_fixup_pc) {
                if let ProcessedInstr::BrSlot {
                    target_ip: ref mut tip,
                    ..
                } = instr_to_patch
                {
                    *tip = function_end_ip;
                } else if let ProcessedInstr::BrIfSlot {
                    target_ip: ref mut tip,
                    ..
                } = instr_to_patch
                {
                    *tip = function_end_ip;
                } else if is_if_false_jump {
                    if !matches!(instr_to_patch, ProcessedInstr::IfSlot { .. }) {
                        fixups[fixup_index].original_wasm_depth = usize::MAX;
                        continue;
                    }
                } else if is_else_jump {
                    if !matches!(instr_to_patch, ProcessedInstr::JumpSlot { .. }) {
                        fixups[fixup_index].original_wasm_depth = usize::MAX;
                        continue;
                    }
                } else if !matches!(
                    instr_to_patch,
                    ProcessedInstr::JumpSlot { .. }
                        | ProcessedInstr::IfSlot { .. }
                        | ProcessedInstr::BrSlot { .. }
                        | ProcessedInstr::BrIfSlot { .. }
                ) {
                    fixups[fixup_index].original_wasm_depth = usize::MAX;
                    continue;
                }
            }
            fixups[fixup_index].original_wasm_depth = usize::MAX;
            continue;
        }

        let target_stack_level = current_control_stack_pass2.len() - 1 - current_fixup_depth;
        if target_stack_level >= current_control_stack_pass2.len() {
            fixups[fixup_index].original_wasm_depth = usize::MAX;
            continue;
        }

        let (target_start_pc, is_loop, _target_block_type) =
            current_control_stack_pass2[target_stack_level];

        // Calculate target IP
        // Note: block_end_map already stores End + 1 position (the instruction after EndSlot)
        let target_ip = if is_loop {
            target_start_pc
        } else {
            *block_end_map
                .get(&target_start_pc)
                .ok_or_else(|| RuntimeError::InvalidWasm("Missing EndMarker for branch target"))?
        };

        // Patch the instruction operand
        if let Some(instr_to_patch) = processed.get_mut(current_fixup_pc) {
            // Skip fixup for slot-based instructions (except those that need fixup)
            if !matches!(
                instr_to_patch,
                ProcessedInstr::JumpSlot { .. }
                    | ProcessedInstr::IfSlot { .. }
                    | ProcessedInstr::BrSlot { .. }
                    | ProcessedInstr::BrIfSlot { .. }
            ) {
                fixups[fixup_index].original_wasm_depth = usize::MAX;
                continue;
            }

            if is_if_false_jump {
                // If instruction's jump-on-false
                // Target is ElseMarker+1 or EndMarker+1
                let else_target = *if_else_map.get(&target_start_pc).unwrap_or(&target_ip);
                let has_else = else_target != target_ip;

                if let ProcessedInstr::IfSlot {
                    else_target_ip: ref mut tip,
                    has_else: ref mut he,
                    ..
                } = instr_to_patch
                {
                    *tip = else_target;
                    *he = has_else;
                }
            } else if is_else_jump {
                if let ProcessedInstr::JumpSlot {
                    target_ip: ref mut tip,
                } = instr_to_patch
                {
                    *tip = target_ip;
                }
            } else {
                // Br or BrIf instruction
                if let ProcessedInstr::BrSlot {
                    target_ip: ref mut tip,
                    ..
                } = instr_to_patch
                {
                    *tip = target_ip;
                } else if let ProcessedInstr::BrIfSlot {
                    target_ip: ref mut tip,
                    ..
                } = instr_to_patch
                {
                    *tip = target_ip;
                }
            }
        } else {
            return Err(RuntimeError::InvalidWasm(
                "Internal Error: Could not find instruction to patch",
            ));
        }
        fixups[fixup_index].original_wasm_depth = usize::MAX;
    }

    // --- Phase 3: Resolve BrTable targets ---
    // Reuse control stack simulation logic, including runtime index tracking
    let mut current_control_stack_pass3: Vec<(usize, bool, wasmparser::BlockType)> = Vec::new();

    for pc in 0..processed.len() {
        if let Some(instr) = processed.get(pc) {
            match instr.handler_index() {
                HANDLER_IDX_BLOCK | HANDLER_IDX_IF => {
                    let block_type = block_type_map
                        .get(&pc)
                        .cloned()
                        .unwrap_or(wasmparser::BlockType::Empty);
                    current_control_stack_pass3.push((pc, false, block_type));
                }
                HANDLER_IDX_LOOP => {
                    let block_type = block_type_map
                        .get(&pc)
                        .cloned()
                        .unwrap_or(wasmparser::BlockType::Empty);
                    current_control_stack_pass3.push((pc, true, block_type));
                }
                HANDLER_IDX_END => {
                    if !current_control_stack_pass3.is_empty() {
                        current_control_stack_pass3.pop();
                    }
                }
                _ => {}
            }

            // Check if it's a BrTable needing resolution *after* simulating stack for current pc
            let needs_br_table_resolution = matches!(instr, ProcessedInstr::BrTableSlot { .. });

            if needs_br_table_resolution {
                // Handle BrTableSlot directly without using fixups
                if matches!(instr, ProcessedInstr::BrTableSlot { .. }) {
                    // Clone targets to get relative depths and existing result_slots
                    let (targets_clone, default_info) = if let ProcessedInstr::BrTableSlot {
                        targets,
                        default_target,
                        ..
                    } = instr
                    {
                        (targets.clone(), default_target.clone())
                    } else {
                        continue;
                    };
                    let default_depth = default_info.0 as usize;
                    let default_result_slots = default_info.2;

                    // Compute target_ip for each target (keeping existing result_slots)
                    let mut resolved_slot_targets: Vec<(u32, usize, Vec<Slot>)> = Vec::new();
                    for (rel_depth, _, result_slots) in targets_clone.iter() {
                        let depth = *rel_depth as usize;
                        if current_control_stack_pass3.len() <= depth {
                            resolved_slot_targets.push((*rel_depth, 0, result_slots.clone())); // Invalid
                            continue;
                        }
                        let target_stack_level = current_control_stack_pass3.len() - 1 - depth;
                        let (target_start_pc, is_loop, _) =
                            current_control_stack_pass3[target_stack_level];
                        // Note: block_end_map already stores End + 1 position
                        let target_ip = if is_loop {
                            target_start_pc
                        } else {
                            *block_end_map.get(&target_start_pc).unwrap_or(&0)
                        };
                        resolved_slot_targets.push((*rel_depth, target_ip, result_slots.clone()));
                    }

                    // Compute target_ip for default target
                    // Note: block_end_map already stores End + 1 position
                    let default_target_ip = if current_control_stack_pass3.len() <= default_depth {
                        0 // Invalid
                    } else {
                        let target_stack_level =
                            current_control_stack_pass3.len() - 1 - default_depth;
                        let (target_start_pc, is_loop, _) =
                            current_control_stack_pass3[target_stack_level];
                        if is_loop {
                            target_start_pc
                        } else {
                            *block_end_map.get(&target_start_pc).unwrap_or(&0)
                        }
                    };

                    // Update BrTableSlot
                    if let Some(instr_to_patch) = processed.get_mut(pc) {
                        if let ProcessedInstr::BrTableSlot {
                            targets: ref mut slot_targets,
                            default_target: ref mut slot_default,
                            ..
                        } = instr_to_patch
                        {
                            *slot_targets = resolved_slot_targets;
                            *slot_default = (
                                default_depth as u32,
                                default_target_ip,
                                default_result_slots,
                            );
                        }
                    }
                    // Mark the fixup as processed
                    for fixup in fixups.iter_mut() {
                        if fixup.pc == pc && fixup.original_wasm_depth != usize::MAX {
                            fixup.original_wasm_depth = usize::MAX;
                        }
                    }
                    continue;
                }

                // Find fixup indices associated *only* with this BrTable pc that haven't been processed yet
                let mut fixup_indices_for_this_br_table = fixups
                    .iter()
                    .enumerate()
                    .filter(|(_, fixup)| fixup.pc == pc && fixup.original_wasm_depth != usize::MAX)
                    .map(|(idx, _)| idx)
                    .collect::<Vec<_>>();

                if fixup_indices_for_this_br_table.is_empty() {
                    continue;
                }

                // --- Default Target Resolution ---
                let default_fixup_idx = fixup_indices_for_this_br_table.pop().unwrap();
                let default_target_operand = {
                    let fixup_depth = fixups[default_fixup_idx].original_wasm_depth;
                    if current_control_stack_pass3.len() <= fixup_depth {
                        return Err(RuntimeError::InvalidWasm(
                            "Invalid relative depth for BrTable default target",
                        ));
                    }
                    let target_stack_level = current_control_stack_pass3.len() - 1 - fixup_depth;

                    // Defensive check for calculated level
                    if target_stack_level >= current_control_stack_pass3.len() {
                        return Err(RuntimeError::InvalidWasm("Internal Error: Invalid stack level calculated for BrTable default target"));
                    }

                    let (target_start_pc, is_loop, target_block_type) =
                        current_control_stack_pass3[target_stack_level];

                    // Note: block_end_map already stores End + 1 position
                    let target_ip = if is_loop {
                        target_start_pc
                    } else {
                        *block_end_map.get(&target_start_pc).ok_or_else(|| {
                            RuntimeError::InvalidWasm(
                                "Missing EndMarker for BrTable default target",
                            )
                        })?
                    };
                    let target_arity = if is_loop {
                        // For loops: Branch provides parameters (input types)
                        calculate_loop_parameter_arity(&target_block_type, module, cache)
                    } else {
                        // For blocks: Branch provides results (output types)
                        calculate_block_arity(&target_block_type, module, cache)
                    };

                    // Get source_slots from fixup (computed at BrTable creation time)
                    let source_slots = fixups[default_fixup_idx].source_slots.clone();
                    // Get target_result_slots from block_result_slots_map
                    let target_result_slots = block_result_slots_map
                        .get(&target_start_pc)
                        .map(|(slots, _)| slots.clone())
                        .unwrap_or_default();

                    fixups[default_fixup_idx].original_wasm_depth = usize::MAX;

                    Operand::LabelIdx {
                        target_ip,
                        arity: target_arity,
                        original_wasm_depth: fixup_depth,
                        is_loop: is_loop, // Use default target's loop/block information
                        source_slots,
                        target_result_slots,
                        condition_slot: None,
                    }
                };

                // --- Remaining Targets Resolution ---
                let mut resolved_targets: Vec<Operand> =
                    Vec::with_capacity(fixup_indices_for_this_br_table.len());
                for fixup_idx in fixup_indices_for_this_br_table {
                    let target_operand = {
                        let fixup_depth = fixups[fixup_idx].original_wasm_depth;
                        if current_control_stack_pass3.len() <= fixup_depth {
                            return Err(RuntimeError::InvalidWasm(
                                "Invalid relative depth for BrTable target",
                            ));
                        }
                        let target_stack_level =
                            current_control_stack_pass3.len() - 1 - fixup_depth;

                        if target_stack_level >= current_control_stack_pass3.len() {
                            return Err(RuntimeError::InvalidWasm(
                                "Internal Error: Invalid stack level calculated for BrTable target",
                            ));
                        }

                        let (target_start_pc, is_loop, target_block_type) =
                            current_control_stack_pass3[target_stack_level];
                        // Note: block_end_map already stores End + 1 position
                        let target_ip = if is_loop {
                            target_start_pc
                        } else {
                            *block_end_map.get(&target_start_pc).ok_or_else(|| {
                                RuntimeError::InvalidWasm("Missing EndMarker for BrTable target")
                            })?
                        };
                        let target_arity = if is_loop {
                            calculate_loop_parameter_arity(&target_block_type, module, cache)
                        } else {
                            calculate_block_arity(&target_block_type, module, cache)
                        };

                        // Get source_slots from fixup (computed at BrTable creation time)
                        let source_slots = fixups[fixup_idx].source_slots.clone();
                        // Get target_result_slots from block_result_slots_map
                        let target_result_slots = block_result_slots_map
                            .get(&target_start_pc)
                            .map(|(slots, _)| slots.clone())
                            .unwrap_or_default();

                        fixups[fixup_idx].original_wasm_depth = usize::MAX;

                        Operand::LabelIdx {
                            target_ip,
                            arity: target_arity,
                            original_wasm_depth: fixup_depth,
                            is_loop: is_loop,
                            source_slots,
                            target_result_slots,
                            condition_slot: None,
                        }
                    };
                    resolved_targets.push(target_operand);
                }

                // --- Patch BrTable Instruction ---
                if let Some(instr_to_patch) = processed.get_mut(pc) {
                    if let ProcessedInstr::BrTableSlot {
                        targets: ref mut slot_targets,
                        default_target: ref mut slot_default,
                        ..
                    } = instr_to_patch
                    {
                        // Extract target_ip and target_result_slots from resolved_targets (Operand::LabelIdx)
                        for (i, operand) in resolved_targets.iter().enumerate() {
                            if let Operand::LabelIdx {
                                target_ip,
                                original_wasm_depth,
                                target_result_slots,
                                ..
                            } = operand
                            {
                                if i < slot_targets.len() {
                                    slot_targets[i] = (
                                        *original_wasm_depth as u32,
                                        *target_ip,
                                        target_result_slots.clone(),
                                    );
                                }
                            }
                        }
                        // Set default target
                        if let Operand::LabelIdx {
                            target_ip,
                            original_wasm_depth,
                            target_result_slots,
                            ..
                        } = &default_target_operand
                        {
                            *slot_default = (
                                *original_wasm_depth as u32,
                                *target_ip,
                                target_result_slots.clone(),
                            );
                        }
                    }
                } else {
                    return Err(RuntimeError::InvalidWasm(
                        "Internal Error: Could not find BrTable instruction to patch",
                    ));
                }
            }
        } else {
            return Err(RuntimeError::InvalidWasm(
                "Internal Error: Invalid program counter during preprocessing",
            ));
        }
    }

    // --- Phase 4: Sanity check - Ensure all fixups were processed ---
    for (_idx, fixup) in fixups.iter().enumerate() {
        if fixup.original_wasm_depth != usize::MAX {
            return Err(RuntimeError::InvalidWasm(
                "Internal Error: Unprocessed fixup after preprocessing",
            ));
        }
    }

    Ok(())
}

/// Get the type of a local variable from its index
/// In WebAssembly, local indices include function parameters first, then declared locals.
/// params: The function parameter types
/// locals: Declared local variables in compressed format: [(count, type), ...]
fn get_local_type(
    params: &[ValueType],
    locals: &[(u32, ValueType)],
    local_index: u32,
) -> ValueType {
    let mut index = local_index as usize;

    // First, check if the index is within the parameters range
    if index < params.len() {
        return params[index].clone();
    }

    // Subtract parameter count to get index into locals
    index -= params.len();

    // Now search through declared locals
    for (count, vtype) in locals {
        if index < *count as usize {
            return vtype.clone();
        }
        index -= *count as usize;
    }
    // Should not reach here in valid wasm (wasmparser validates indices)
    ValueType::NumType(NumType::I32)
}

fn get_global_type(module: &Module, global_index: u32) -> ValueType {
    let mut imported_global_count = 0u32;
    for import in &module.imports {
        if let ImportDesc::Global(global_type) = &import.desc {
            if imported_global_count == global_index {
                return global_type.1.clone();
            }
            imported_global_count += 1;
        }
    }

    let local_global_index = (global_index - imported_global_count) as usize;
    if local_global_index < module.globals.len() {
        return module.globals[local_global_index].type_.1.clone();
    }

    ValueType::NumType(NumType::I32)
}

fn get_table_element_type(module: &Module, table_index: u32) -> ValueType {
    // Count imported tables first
    let mut imported_table_count = 0u32;
    for import in &module.imports {
        if let ImportDesc::Table(table_type) = &import.desc {
            if imported_table_count == table_index {
                return ValueType::RefType(table_type.1.clone());
            }
            imported_table_count += 1;
        }
    }

    // Check module-defined tables
    let local_table_index = (table_index - imported_table_count) as usize;
    if local_table_index < module.tables.len() {
        return ValueType::RefType(module.tables[local_table_index].type_.1.clone());
    }

    // Default to funcref
    ValueType::RefType(RefType::FuncRef)
}

fn decode_processed_instrs_and_fixups<'a>(
    ops_iter: wasmparser::OperatorsIteratorWithOffsets<'a>,
    module: &Module,
    _enable_superinstructions: bool,
    locals: &[(u32, ValueType)],
    param_types: &[ValueType],
    result_types: &[ValueType],
) -> Result<
    (
        Vec<ProcessedInstr>,
        Vec<FixupInfo>,
        FxHashMap<usize, usize>,
        FxHashMap<usize, usize>,
        FxHashMap<usize, wasmparser::BlockType>,
        Option<crate::execution::slots::SlotAllocation>,
        Option<crate::execution::slots::Slot>, // result_slot
        FxHashMap<usize, (Vec<Slot>, bool)>, // block_result_slots_map: block_start_pc -> (result_slots, is_loop)
    ),
    Box<dyn std::error::Error>,
> {
    let mut ops = ops_iter.multipeek();
    let mut initial_processed_instrs = Vec::new();
    let mut initial_fixups = Vec::new();
    let mut current_processed_pc = 0;
    // Control block stack with result slots
    let mut control_info_stack: Vec<ControlBlockInfo> = Vec::new();

    let mut block_end_map: FxHashMap<usize, usize> = FxHashMap::default();
    let mut if_else_map: FxHashMap<usize, usize> = FxHashMap::default();
    let mut block_type_map: FxHashMap<usize, wasmparser::BlockType> = FxHashMap::default();
    let mut control_stack_for_map_building: Vec<(usize, bool, Option<usize>)> = Vec::new();
    // Map from block_start_pc to (result_slots, is_loop) for BrTable resolution in Pass 3
    let mut block_result_slots_map: FxHashMap<usize, (Vec<Slot>, bool)> = FxHashMap::default();

    // Initialize slot allocator (always used since Legacy mode is removed)
    use crate::execution::slots::SlotAllocator;
    let mut slot_allocator = Some(SlotAllocator::new(locals));

    // Track allocator state at block entry for proper restoration on block exit
    let mut allocator_state_stack: Vec<crate::execution::slots::SlotAllocatorState> = Vec::new();

    // Track unreachable code depth (after br, return, unreachable, br_table)
    let mut unreachable_depth: usize = 0;

    loop {
        if ops.peek().is_none() {
            break;
        }

        let (op, _offset) = match ops.next() {
            Some(Ok(op_offset)) => op_offset,
            Some(Err(e)) => return Err(Box::new(e)),
            None => break,
        };

        // Handle unreachable code
        if unreachable_depth > 0 {
            match &op {
                wasmparser::Operator::Block { .. }
                | wasmparser::Operator::Loop { .. }
                | wasmparser::Operator::If { .. } => {
                    unreachable_depth += 1;
                }
                wasmparser::Operator::End => {
                    unreachable_depth -= 1;
                }
                wasmparser::Operator::Else => {
                    if unreachable_depth == 1 {
                        // Else at depth 1 means the then-branch was unreachable but else might be reachable
                        unreachable_depth = 0;
                    }
                }
                _ => {}
            }
            if unreachable_depth > 0 {
                initial_processed_instrs.push(ProcessedInstr::NopSlot);
                current_processed_pc += 1;
                continue;
            }
        }

        // Get the processed instruction based on execution mode
        let (processed_instr, fixup_info_opt) = if let Some(ref mut allocator) = slot_allocator {
            // Slot-based mode: convert i32 instructions to slot format
            match &op {
                wasmparser::Operator::LocalGet { local_index } => {
                    let local_type = get_local_type(param_types, locals, *local_index);
                    match local_type {
                        ValueType::NumType(NumType::I32) => {
                            let dst = allocator.push(local_type);
                            (
                                ProcessedInstr::I32Slot {
                                    handler_index: HANDLER_IDX_LOCAL_GET,
                                    dst: dst.index(),
                                    src1: I32SlotOperand::Param(*local_index as u16),
                                    src2: None,
                                },
                                None,
                            )
                        }
                        ValueType::NumType(NumType::I64) => {
                            let dst = allocator.push(local_type);
                            (
                                ProcessedInstr::I64Slot {
                                    handler_index: HANDLER_IDX_LOCAL_GET,
                                    dst,
                                    src1: I64SlotOperand::Param(*local_index as u16),
                                    src2: None,
                                },
                                None,
                            )
                        }
                        ValueType::NumType(NumType::F32) => {
                            let dst = allocator.push(local_type);
                            (
                                ProcessedInstr::F32Slot {
                                    handler_index: HANDLER_IDX_LOCAL_GET,
                                    dst,
                                    src1: F32SlotOperand::Param(*local_index as u16),
                                    src2: None,
                                },
                                None,
                            )
                        }
                        ValueType::NumType(NumType::F64) => {
                            let dst = allocator.push(local_type);
                            (
                                ProcessedInstr::F64Slot {
                                    handler_index: HANDLER_IDX_LOCAL_GET,
                                    dst,
                                    src1: F64SlotOperand::Param(*local_index as u16),
                                    src2: None,
                                },
                                None,
                            )
                        }
                        ValueType::RefType(_) => {
                            // For RefType, use RefLocalSlot
                            let dst = allocator.push(local_type);
                            (
                                ProcessedInstr::RefLocalSlot {
                                    handler_index: HANDLER_IDX_REF_LOCAL_GET_SLOT,
                                    dst: dst.index(),
                                    src: 0, // unused for get
                                    local_idx: *local_index as u16,
                                },
                                None,
                            )
                        }
                        _ => {
                            panic!("Unsupported type for LocalGet: {:?}", local_type);
                        }
                    }
                }
                wasmparser::Operator::LocalSet { local_index } => {
                    let local_type = get_local_type(param_types, locals, *local_index);
                    let local_idx = *local_index as u16;
                    let src = allocator.pop(&local_type);
                    let src_idx = src.index();
                    macro_rules! make_local_set {
                        ($instr:ident, $slot:ident, $operand:ident) => {
                            (
                                ProcessedInstr::$instr {
                                    handler_index: HANDLER_IDX_LOCAL_SET,
                                    dst: Slot::$slot(local_idx),
                                    src1: $operand::Slot(src_idx),
                                    src2: None,
                                },
                                None,
                            )
                        };
                    }
                    match local_type {
                        ValueType::NumType(NumType::I32) => (
                            ProcessedInstr::I32Slot {
                                handler_index: HANDLER_IDX_LOCAL_SET,
                                dst: local_idx,
                                src1: I32SlotOperand::Slot(src_idx),
                                src2: None,
                            },
                            None,
                        ),
                        ValueType::NumType(NumType::I64) => {
                            make_local_set!(I64Slot, I64, I64SlotOperand)
                        }
                        ValueType::NumType(NumType::F32) => {
                            make_local_set!(F32Slot, F32, F32SlotOperand)
                        }
                        ValueType::NumType(NumType::F64) => {
                            make_local_set!(F64Slot, F64, F64SlotOperand)
                        }
                        ValueType::RefType(_) => {
                            (
                                ProcessedInstr::RefLocalSlot {
                                    handler_index: HANDLER_IDX_REF_LOCAL_SET_SLOT,
                                    dst: 0, // unused for set
                                    src: src_idx,
                                    local_idx,
                                },
                                None,
                            )
                        }
                        _ => {
                            panic!("Unsupported type for LocalSet: {:?}", local_type);
                        }
                    }
                }
                wasmparser::Operator::LocalTee { local_index } => {
                    // LocalTee: copy value to local, value stays on stack
                    let local_type = get_local_type(param_types, locals, *local_index);
                    let local_idx = *local_index as u16;
                    // Peek the top slot (don't pop - value stays on stack)
                    let src_idx = allocator.peek(&local_type).unwrap().index();
                    macro_rules! make_local_tee {
                        ($instr:ident, $slot:ident, $operand:ident) => {
                            (
                                ProcessedInstr::$instr {
                                    handler_index: HANDLER_IDX_LOCAL_SET, // Reuse local.set handler
                                    dst: Slot::$slot(local_idx),
                                    src1: $operand::Slot(src_idx),
                                    src2: None,
                                },
                                None,
                            )
                        };
                    }
                    match local_type {
                        ValueType::NumType(NumType::I32) => (
                            ProcessedInstr::I32Slot {
                                handler_index: HANDLER_IDX_LOCAL_SET,
                                dst: local_idx,
                                src1: I32SlotOperand::Slot(src_idx),
                                src2: None,
                            },
                            None,
                        ),
                        ValueType::NumType(NumType::I64) => {
                            make_local_tee!(I64Slot, I64, I64SlotOperand)
                        }
                        ValueType::NumType(NumType::F32) => {
                            make_local_tee!(F32Slot, F32, F32SlotOperand)
                        }
                        ValueType::NumType(NumType::F64) => {
                            make_local_tee!(F64Slot, F64, F64SlotOperand)
                        }
                        ValueType::RefType(_) => {
                            (
                                ProcessedInstr::RefLocalSlot {
                                    handler_index: HANDLER_IDX_REF_LOCAL_SET_SLOT,
                                    dst: 0, // unused for set
                                    src: src_idx,
                                    local_idx,
                                },
                                None,
                            )
                        }
                        _ => {
                            panic!("Unsupported type for LocalTee: {:?}", local_type);
                        }
                    }
                }
                wasmparser::Operator::GlobalGet { global_index } => {
                    let global_type = get_global_type(module, *global_index);
                    match global_type {
                        ValueType::NumType(NumType::I32) => {
                            let dst = allocator.push(global_type);
                            (
                                ProcessedInstr::GlobalGetSlot {
                                    handler_index: HANDLER_IDX_GLOBAL_GET_I32,
                                    dst,
                                    global_index: *global_index,
                                },
                                None,
                            )
                        }
                        ValueType::NumType(NumType::I64) => {
                            let dst = allocator.push(global_type);
                            (
                                ProcessedInstr::GlobalGetSlot {
                                    handler_index: HANDLER_IDX_GLOBAL_GET_I64,
                                    dst,
                                    global_index: *global_index,
                                },
                                None,
                            )
                        }
                        ValueType::NumType(NumType::F32) => {
                            let dst = allocator.push(global_type);
                            (
                                ProcessedInstr::GlobalGetSlot {
                                    handler_index: HANDLER_IDX_GLOBAL_GET_F32,
                                    dst,
                                    global_index: *global_index,
                                },
                                None,
                            )
                        }
                        ValueType::NumType(NumType::F64) => {
                            let dst = allocator.push(global_type);
                            (
                                ProcessedInstr::GlobalGetSlot {
                                    handler_index: HANDLER_IDX_GLOBAL_GET_F64,
                                    dst,
                                    global_index: *global_index,
                                },
                                None,
                            )
                        }
                        _ => {
                            panic!("Unsupported type for GlobalGet: {:?}", global_type);
                        }
                    }
                }
                wasmparser::Operator::GlobalSet { global_index } => {
                    let global_type = get_global_type(module, *global_index);
                    match global_type {
                        ValueType::NumType(NumType::I32) => {
                            let src = allocator.pop(&global_type);
                            (
                                ProcessedInstr::GlobalSetSlot {
                                    handler_index: HANDLER_IDX_GLOBAL_SET_I32,
                                    src,
                                    global_index: *global_index,
                                },
                                None,
                            )
                        }
                        ValueType::NumType(NumType::I64) => {
                            let src = allocator.pop(&global_type);
                            (
                                ProcessedInstr::GlobalSetSlot {
                                    handler_index: HANDLER_IDX_GLOBAL_SET_I64,
                                    src,
                                    global_index: *global_index,
                                },
                                None,
                            )
                        }
                        ValueType::NumType(NumType::F32) => {
                            let src = allocator.pop(&global_type);
                            (
                                ProcessedInstr::GlobalSetSlot {
                                    handler_index: HANDLER_IDX_GLOBAL_SET_F32,
                                    src,
                                    global_index: *global_index,
                                },
                                None,
                            )
                        }
                        ValueType::NumType(NumType::F64) => {
                            let src = allocator.pop(&global_type);
                            (
                                ProcessedInstr::GlobalSetSlot {
                                    handler_index: HANDLER_IDX_GLOBAL_SET_F64,
                                    src,
                                    global_index: *global_index,
                                },
                                None,
                            )
                        }
                        _ => {
                            panic!("Unsupported type for GlobalSet: {:?}", global_type);
                        }
                    }
                }
                wasmparser::Operator::I32Const { value } => {
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_CONST,
                            dst: dst.index(),
                            src1: I32SlotOperand::Const(*value),
                            src2: None,
                        },
                        None,
                    )
                }
                // Binary operations - macro to reduce repetition
                wasmparser::Operator::I32Add => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_ADD,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Sub => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_SUB,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Mul => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_MUL,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32DivS => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_DIV_S,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32DivU => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_DIV_U,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32RemS => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_REM_S,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32RemU => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_REM_U,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32And => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_AND,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Or => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_OR,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Xor => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_XOR,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Shl => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_SHL,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32ShrS => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_SHR_S,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32ShrU => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_SHR_U,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Rotl => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_ROTL,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Rotr => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_ROTR,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                // Comparison operations
                wasmparser::Operator::I32Eq => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_EQ,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Ne => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_NE,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32LtS => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_LT_S,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32LtU => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_LT_U,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32LeS => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_LE_S,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32LeU => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_LE_U,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32GtS => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_GT_S,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32GtU => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_GT_U,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32GeS => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_GE_S,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32GeU => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_GE_U,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: Some(I32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                // Unary operations
                wasmparser::Operator::I32Clz => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_CLZ,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Ctz => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_CTZ,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Popcnt => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_POPCNT,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Eqz => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_EQZ,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Extend8S => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_EXTEND8_S,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Extend16S => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I32Slot {
                            handler_index: HANDLER_IDX_I32_EXTEND16_S,
                            dst: dst.index(),
                            src1: I32SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                // ============================================================================
                // I64 Slot-based instructions
                // ============================================================================
                wasmparser::Operator::I64Const { value } => {
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_CONST,
                            dst,
                            src1: I64SlotOperand::Const(*value),
                            src2: None,
                        },
                        None,
                    )
                }
                // I64 Binary arithmetic operations
                wasmparser::Operator::I64Add => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_ADD,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Sub => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_SUB,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Mul => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_MUL,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64DivS => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_DIV_S,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64DivU => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_DIV_U,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64RemS => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_REM_S,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64RemU => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_REM_U,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                // I64 Binary bitwise operations
                wasmparser::Operator::I64And => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_AND,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Or => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_OR,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Xor => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_XOR,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Shl => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_SHL,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64ShrS => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_SHR_S,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64ShrU => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_SHR_U,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Rotl => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_ROTL,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Rotr => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_ROTR,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                // I64 Unary operations
                wasmparser::Operator::I64Clz => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_CLZ,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Ctz => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_CTZ,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Popcnt => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_POPCNT,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Extend8S => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_EXTEND8_S,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Extend16S => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_EXTEND16_S,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Extend32S => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_EXTEND32_S,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                // I64 Comparison operations (return i32)
                wasmparser::Operator::I64Eqz => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_EQZ,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Eq => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_EQ,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Ne => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_NE,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64LtS => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_LT_S,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64LtU => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_LT_U,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64GtS => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_GT_S,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64GtU => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_GT_U,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64LeS => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_LE_S,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64LeU => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_LE_U,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64GeS => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_GE_S,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64GeU => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Slot {
                            handler_index: HANDLER_IDX_I64_GE_U,
                            dst,
                            src1: I64SlotOperand::Slot(src1.index()),
                            src2: Some(I64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                // F32 Const
                wasmparser::Operator::F32Const { value } => {
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_CONST,
                            dst,
                            src1: F32SlotOperand::Const(f32::from_bits(value.bits())),
                            src2: None,
                        },
                        None,
                    )
                }
                // F32 Binary arithmetic operations
                wasmparser::Operator::F32Add => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_ADD,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: Some(F32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Sub => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_SUB,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: Some(F32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Mul => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_MUL,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: Some(F32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Div => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_DIV,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: Some(F32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Min => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_MIN,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: Some(F32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Max => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_MAX,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: Some(F32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Copysign => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_COPYSIGN,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: Some(F32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                // F32 Unary operations
                wasmparser::Operator::F32Abs => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_ABS,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Neg => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_NEG,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Ceil => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_CEIL,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Floor => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_FLOOR,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Trunc => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_TRUNC,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Nearest => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_NEAREST,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Sqrt => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_SQRT,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                // F32 Comparison operations (return i32)
                wasmparser::Operator::F32Eq => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_EQ,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: Some(F32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Ne => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_NE,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: Some(F32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Lt => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_LT,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: Some(F32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Gt => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_GT,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: Some(F32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Le => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_LE,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: Some(F32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Ge => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F32Slot {
                            handler_index: HANDLER_IDX_F32_GE,
                            dst,
                            src1: F32SlotOperand::Slot(src1.index()),
                            src2: Some(F32SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                // F64 Const
                wasmparser::Operator::F64Const { value } => {
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_CONST,
                            dst,
                            src1: F64SlotOperand::Const(f64::from_bits(value.bits())),
                            src2: None,
                        },
                        None,
                    )
                }
                // F64 Binary arithmetic operations
                wasmparser::Operator::F64Add => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_ADD,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: Some(F64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Sub => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_SUB,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: Some(F64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Mul => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_MUL,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: Some(F64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Div => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_DIV,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: Some(F64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Min => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_MIN,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: Some(F64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Max => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_MAX,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: Some(F64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Copysign => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_COPYSIGN,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: Some(F64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                // F64 Unary operations
                wasmparser::Operator::F64Abs => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_ABS,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Neg => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_NEG,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Ceil => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_CEIL,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Floor => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_FLOOR,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Trunc => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_TRUNC,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Nearest => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_NEAREST,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Sqrt => {
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_SQRT,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: None,
                        },
                        None,
                    )
                }
                // F64 Comparison operations (return i32)
                wasmparser::Operator::F64Eq => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_EQ,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: Some(F64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Ne => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_NE,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: Some(F64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Lt => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_LT,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: Some(F64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Gt => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_GT,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: Some(F64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Le => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_LE,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: Some(F64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Ge => {
                    let src2 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F64Slot {
                            handler_index: HANDLER_IDX_F64_GE,
                            dst,
                            src1: F64SlotOperand::Slot(src1.index()),
                            src2: Some(F64SlotOperand::Slot(src2.index())),
                        },
                        None,
                    )
                }
                wasmparser::Operator::End => {
                    let result_type_vec = if let Some(block_info) = control_info_stack.last() {
                        get_block_result_types(&block_info.block_type, module)
                    } else {
                        result_types.to_vec()
                    };

                    // Get the top N slots based on result types
                    let source_slots = allocator.peek_slots_for_types(&result_type_vec);

                    // Get target_result_slots from ControlBlockInfo BEFORE restoring state
                    let target_result_slots = if let Some(block_info) = control_info_stack.last() {
                        block_info.result_slots.clone()
                    } else {
                        Vec::new()
                    };

                    // Get block result types before popping control_info_stack
                    let block_result_types_to_push = control_info_stack
                        .last()
                        .map(|info| get_block_result_types(&info.block_type, module))
                        .unwrap_or_default();

                    // Pop control_info_stack
                    control_info_stack.pop();

                    // Restore allocator state to block entry, then push results
                    let mut stack_to_slots = Vec::new();
                    if let Some(saved_state) = allocator_state_stack.pop() {
                        allocator.restore_state(&saved_state);
                        // Push result types to allocator (at the restored depth)
                        for vtype in block_result_types_to_push {
                            let slot = allocator.push(vtype);
                            stack_to_slots.push(slot);
                        }
                    }

                    // Use EndSlot for slot-based execution
                    let instr = ProcessedInstr::EndSlot {
                        source_slots,
                        target_result_slots,
                    };
                    (instr, None)
                }
                wasmparser::Operator::Block { blockty }
                | wasmparser::Operator::Loop { blockty } => {
                    let param_types = get_block_param_types(&blockty, module);
                    let is_loop = matches!(op, wasmparser::Operator::Loop { .. });

                    // Pop params from allocator to get state BEFORE params
                    for vtype in param_types.iter().rev() {
                        allocator.pop(&vtype);
                    }

                    // Save allocator state BEFORE params
                    let saved_state = allocator.save_state();
                    allocator_state_stack.push(saved_state.clone());

                    // Calculate result_slots at block entry depth
                    let result_types = get_block_result_types(&blockty, module);
                    let result_slots: Vec<Slot> = {
                        let mut state = saved_state;
                        result_types
                            .iter()
                            .map(|vtype| state.next_slot_for_type(vtype))
                            .collect()
                    };

                    // Push params back - they're still on the stack inside the block
                    for vtype in param_types.iter() {
                        allocator.push(vtype.clone());
                    }

                    let arity = result_types.len();
                    let param_count = param_types.len();

                    // Push to control_info_stack for End to use
                    control_info_stack.push(ControlBlockInfo {
                        block_type: *blockty,
                        is_loop,
                        result_slots,
                        param_slots: vec![],
                    });

                    let instr = ProcessedInstr::BlockSlot {
                        arity,
                        param_count,
                        is_loop,
                    };
                    (instr, None)
                }
                wasmparser::Operator::If { blockty } => {
                    let condition_slot = allocator.pop(&ValueType::NumType(NumType::I32));

                    let param_types = get_block_param_types(&blockty, module);

                    for vtype in param_types.iter().rev() {
                        allocator.pop(&vtype);
                    }

                    let saved_state = allocator.save_state();
                    allocator_state_stack.push(saved_state.clone());

                    // Calculate result_slots at block entry depth
                    let result_types = get_block_result_types(&blockty, module);
                    let result_slots: Vec<Slot> = {
                        let mut state = saved_state;
                        result_types
                            .iter()
                            .map(|vtype| state.next_slot_for_type(vtype))
                            .collect()
                    };

                    for vtype in param_types.iter() {
                        allocator.push(vtype.clone());
                    }

                    let arity = result_types.len();

                    // Push to control_info_stack for End to use
                    control_info_stack.push(ControlBlockInfo {
                        block_type: *blockty,
                        is_loop: false,
                        result_slots,
                        param_slots: vec![],
                    });

                    let instr = ProcessedInstr::IfSlot {
                        arity,
                        condition_slot,
                        else_target_ip: usize::MAX, // Will be fixed up
                        has_else: false,            // Will be updated during fixup
                    };
                    let fixup = Some(FixupInfo {
                        pc: current_processed_pc,
                        original_wasm_depth: 0,
                        is_if_false_jump: true,
                        is_else_jump: false,
                        source_slots: vec![],
                    });
                    (instr, fixup)
                }
                wasmparser::Operator::Else => {
                    if let Some(state) = allocator_state_stack.last() {
                        allocator.restore_state(state);
                    }

                    // Get the If's block type from control_info_stack and push params back
                    if let Some(block_info) = control_info_stack.last() {
                        let param_types = get_block_param_types(&block_info.block_type, module);
                        for vtype in param_types.iter() {
                            allocator.push(vtype.clone());
                        }
                    }

                    // Generate JumpSlot (target_ip will be fixed up later)
                    let instr = ProcessedInstr::JumpSlot {
                        target_ip: usize::MAX, // Will be fixed up
                    };
                    let fixup = Some(FixupInfo {
                        pc: current_processed_pc,
                        original_wasm_depth: 0,
                        is_if_false_jump: false,
                        is_else_jump: true,
                        source_slots: vec![],
                    });
                    (instr, fixup)
                }

                wasmparser::Operator::Call { function_index } => {
                    let wasi_func_type = if (*function_index as usize) < module.num_imported_funcs {
                        if let Some(import) = module.imports.get(*function_index as usize) {
                            match &import.desc {
                                crate::structure::module::ImportDesc::WasiFunc(wasi_type) => {
                                    Some(*wasi_type)
                                }
                                _ => None,
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(wasi_type) = wasi_func_type {
                        let func_type = wasi_type.expected_func_type();
                        let param_types = func_type.params;
                        let result_types = func_type.results;

                        // Get the top N slots for params based on types
                        let param_slots = allocator.peek_slots_for_types(&param_types);

                        for param_type in param_types.iter().rev() {
                            allocator.pop(&param_type);
                        }

                        let result_slot = if let Some(result_type) = result_types.first() {
                            Some(allocator.push(result_type.clone()))
                        } else {
                            None
                        };

                        (
                            ProcessedInstr::CallWasiSlot {
                                wasi_func_type: wasi_type,
                                param_slots,
                                result_slot,
                            },
                            None,
                        )
                    } else {
                        let (param_types, result_types) = if (*function_index as usize)
                            < module.num_imported_funcs
                        {
                            if let Some(import) = module.imports.get(*function_index as usize) {
                                match &import.desc {
                                    crate::structure::module::ImportDesc::Func(type_idx) => {
                                        if let Some(func_type) =
                                            module.types.get(type_idx.0 as usize)
                                        {
                                            (func_type.params.clone(), func_type.results.clone())
                                        } else {
                                            (Vec::new(), Vec::new())
                                        }
                                    }
                                    _ => (Vec::new(), Vec::new()),
                                }
                            } else {
                                (Vec::new(), Vec::new())
                            }
                        } else {
                            let local_idx = *function_index as usize - module.num_imported_funcs;
                            if let Some(func) = module.funcs.get(local_idx) {
                                if let Some(func_type) = module.types.get(func.type_.0 as usize) {
                                    (func_type.params.clone(), func_type.results.clone())
                                } else {
                                    (Vec::new(), Vec::new())
                                }
                            } else {
                                (Vec::new(), Vec::new())
                            }
                        };

                        if param_types.is_empty() && result_types.is_empty() {
                            // No params/results - still use CallSlot with empty slots
                            (
                                ProcessedInstr::CallSlot {
                                    func_idx: FuncIdx(*function_index),
                                    param_slots: vec![],
                                    result_slots: vec![],
                                },
                                None,
                            )
                        } else {
                            // Get param slots before popping
                            let mut param_slots = Vec::new();
                            for param_type in param_types.iter().rev() {
                                if let Some(slot) = allocator.peek(param_type) {
                                    param_slots.insert(0, slot);
                                }
                                allocator.pop(param_type);
                            }

                            // Push result types to allocator and collect result_slots
                            let mut result_slots = Vec::new();
                            for result_type in &result_types {
                                let slot = allocator.push(result_type.clone());
                                result_slots.push(slot);
                            }

                            // Use CallSlot for slot-based execution
                            let instr = ProcessedInstr::CallSlot {
                                func_idx: FuncIdx(*function_index),
                                param_slots,
                                result_slots,
                            };
                            (instr, None)
                        }
                    }
                }

                wasmparser::Operator::CallIndirect {
                    type_index,
                    table_index,
                    ..
                } => {
                    // Get function type from type_index
                    let (param_types, result_types_vec) =
                        if let Some(func_type) = module.types.get(*type_index as usize) {
                            (func_type.params.clone(), func_type.results.clone())
                        } else {
                            (Vec::new(), Vec::new())
                        };

                    // Pop index_slot first (i32) - this is the table index
                    let index_slot = allocator.peek(&ValueType::NumType(NumType::I32)).unwrap();
                    allocator.pop(&ValueType::NumType(NumType::I32));

                    // Get param slots (in order, from bottom to top)
                    // We need to collect the param slots before popping them
                    let param_count = param_types.len();
                    let mut param_slots = Vec::with_capacity(param_count);
                    for param_type in param_types.iter() {
                        if let Some(slot) = allocator.peek(param_type) {
                            param_slots.push(slot);
                        }
                    }
                    // Actually, we need to get the slots that will be consumed
                    // Recalculate: peek each param type from the current state
                    param_slots.clear();
                    for param_type in param_types.iter().rev() {
                        if let Some(slot) = allocator.peek(param_type) {
                            param_slots.insert(0, slot);
                        }
                        allocator.pop(param_type);
                    }

                    // Push result types to allocator and collect result_slots
                    let mut result_slots = Vec::new();
                    for result_type in &result_types_vec {
                        let slot = allocator.push(result_type.clone());
                        result_slots.push(slot);
                    }

                    // Use CallIndirectSlot for slot-based execution
                    let instr = ProcessedInstr::CallIndirectSlot {
                        type_idx: TypeIdx(*type_index),
                        table_idx: TableIdx(*table_index),
                        index_slot,
                        param_slots,
                        result_slots,
                    };
                    (instr, None)
                }

                wasmparser::Operator::Br { relative_depth } => {
                    // Compute source and target slots for branch
                    let (source_slots, target_result_slots) = compute_branch_slots(
                        &control_info_stack,
                        *relative_depth as usize,
                        slot_allocator.as_ref(),
                    );

                    let instr = ProcessedInstr::BrSlot {
                        relative_depth: *relative_depth,
                        target_ip: usize::MAX, // Will be set by fixup
                        source_slots: source_slots.clone(),
                        target_result_slots,
                    };
                    let fixup = FixupInfo {
                        pc: current_processed_pc,
                        original_wasm_depth: *relative_depth as usize,
                        is_if_false_jump: false,
                        is_else_jump: false,
                        source_slots,
                    };
                    (instr, Some(fixup))
                }
                wasmparser::Operator::BrIf { relative_depth } => {
                    // Pop condition slot
                    // In unreachable code after br/return, allocator may be empty
                    let condition_slot = allocator
                        .peek(&ValueType::NumType(NumType::I32))
                        .unwrap_or(Slot::I32(0)); // Use dummy slot in unreachable code
                    allocator.pop(&ValueType::NumType(NumType::I32));

                    // Compute source and target slots for branch
                    let (source_slots, target_result_slots) = compute_branch_slots(
                        &control_info_stack,
                        *relative_depth as usize,
                        slot_allocator.as_ref(),
                    );

                    let instr = ProcessedInstr::BrIfSlot {
                        relative_depth: *relative_depth,
                        target_ip: usize::MAX, // Will be set by fixup
                        condition_slot,
                        source_slots: source_slots.clone(),
                        target_result_slots,
                    };
                    let fixup = FixupInfo {
                        pc: current_processed_pc,
                        original_wasm_depth: *relative_depth as usize,
                        is_if_false_jump: false,
                        is_else_jump: false,
                        source_slots,
                    };
                    (instr, Some(fixup))
                }
                wasmparser::Operator::BrTable { ref targets } => {
                    // Pop index slot
                    // In unreachable code after br/return, allocator may be empty
                    let index_slot = allocator
                        .peek(&ValueType::NumType(NumType::I32))
                        .unwrap_or(Slot::I32(0)); // Use dummy slot in unreachable code
                    allocator.pop(&ValueType::NumType(NumType::I32));

                    // Collect target depths with placeholder target_ip and compute target_result_slots for each
                    let target_depths: Vec<u32> =
                        targets.targets().collect::<Result<Vec<_>, _>>()?;

                    let mut table_targets: Vec<(u32, usize, Vec<Slot>)> =
                        Vec::with_capacity(target_depths.len());
                    for depth in target_depths.iter() {
                        let (_, target_result_slots) = compute_branch_slots(
                            &control_info_stack,
                            *depth as usize,
                            slot_allocator.as_ref(),
                        );
                        table_targets.push((*depth, usize::MAX, target_result_slots));
                        // target_ip will be set by fixup
                    }

                    // Compute source and target slots for default target
                    let (source_slots, default_result_slots) = compute_branch_slots(
                        &control_info_stack,
                        targets.default() as usize,
                        slot_allocator.as_ref(),
                    );
                    let default_target = (targets.default(), usize::MAX, default_result_slots); // target_ip will be set by fixup

                    let instr = ProcessedInstr::BrTableSlot {
                        targets: table_targets.clone(),
                        default_target,
                        index_slot,
                        source_slots: source_slots.clone(),
                    };

                    // Create fixups for each target and default
                    // The fixup system will handle BrTableSlot specially
                    let fixup = FixupInfo {
                        pc: current_processed_pc,
                        original_wasm_depth: targets.default() as usize,
                        is_if_false_jump: false,
                        is_else_jump: false,
                        source_slots,
                    };
                    (instr, Some(fixup))
                }
                wasmparser::Operator::Return => {
                    // Get result slots based on function result types
                    let result_slots = allocator.peek_slots_for_types(result_types);
                    for result_type in result_types.iter().rev() {
                        allocator.pop(result_type);
                    }

                    let instr = ProcessedInstr::ReturnSlot { result_slots };
                    (instr, None)
                }
                wasmparser::Operator::Nop => (ProcessedInstr::NopSlot, None),
                wasmparser::Operator::Unreachable => (ProcessedInstr::UnreachableSlot, None),
                wasmparser::Operator::Drop => {
                    // Pop from type_stack to keep it in sync, but no runtime operation needed
                    allocator.pop_any();
                    (ProcessedInstr::NopSlot, None)
                }
                // Conversion instructions - use ConversionSlot
                wasmparser::Operator::I64ExtendI32S => {
                    let src = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I64_EXTEND_I32_S,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64ExtendI32U => {
                    let src = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I64_EXTEND_I32_U,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32WrapI64 => {
                    let src = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I32_WRAP_I64,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32TruncF32S => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I32_TRUNC_F32_S,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32TruncF32U => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I32_TRUNC_F32_U,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32TruncF64S => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I32_TRUNC_F64_S,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32TruncF64U => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I32_TRUNC_F64_U,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64TruncF32S => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I64_TRUNC_F32_S,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64TruncF32U => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I64_TRUNC_F32_U,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64TruncF64S => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I64_TRUNC_F64_S,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64TruncF64U => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I64_TRUNC_F64_U,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32TruncSatF32S => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I32_TRUNC_SAT_F32_S,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32TruncSatF32U => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I32_TRUNC_SAT_F32_U,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32TruncSatF64S => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I32_TRUNC_SAT_F64_S,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32TruncSatF64U => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I32_TRUNC_SAT_F64_U,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64TruncSatF32S => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I64_TRUNC_SAT_F32_S,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64TruncSatF32U => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I64_TRUNC_SAT_F32_U,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64TruncSatF64S => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I64_TRUNC_SAT_F64_S,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64TruncSatF64U => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I64_TRUNC_SAT_F64_U,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32ConvertI32S => {
                    let src = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_F32_CONVERT_I32_S,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32ConvertI32U => {
                    let src = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_F32_CONVERT_I32_U,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32ConvertI64S => {
                    let src = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_F32_CONVERT_I64_S,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32ConvertI64U => {
                    let src = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_F32_CONVERT_I64_U,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64ConvertI32S => {
                    let src = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_F64_CONVERT_I32_S,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64ConvertI32U => {
                    let src = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_F64_CONVERT_I32_U,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64ConvertI64S => {
                    let src = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_F64_CONVERT_I64_S,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64ConvertI64U => {
                    let src = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_F64_CONVERT_I64_U,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32DemoteF64 => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_F32_DEMOTE_F64,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64PromoteF32 => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_F64_PROMOTE_F32,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32ReinterpretF32 => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I32_REINTERPRET_F32,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64ReinterpretF64 => {
                    let src = allocator.pop(&ValueType::NumType(NumType::F64));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_I64_REINTERPRET_F64,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32ReinterpretI32 => {
                    let src = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_F32_REINTERPRET_I32,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64ReinterpretI64 => {
                    let src = allocator.pop(&ValueType::NumType(NumType::I64));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::ConversionSlot {
                            handler_index: HANDLER_IDX_F64_REINTERPRET_I64,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                // Memory Load instructions
                wasmparser::Operator::I32Load { memarg } => {
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryLoadSlot {
                            handler_index: HANDLER_IDX_I32_LOAD,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Load { memarg } => {
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::MemoryLoadSlot {
                            handler_index: HANDLER_IDX_I64_LOAD,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Load { memarg } => {
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::MemoryLoadSlot {
                            handler_index: HANDLER_IDX_F32_LOAD,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Load { memarg } => {
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::MemoryLoadSlot {
                            handler_index: HANDLER_IDX_F64_LOAD,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Load8S { memarg } => {
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryLoadSlot {
                            handler_index: HANDLER_IDX_I32_LOAD8_S,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Load8U { memarg } => {
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryLoadSlot {
                            handler_index: HANDLER_IDX_I32_LOAD8_U,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Load16S { memarg } => {
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryLoadSlot {
                            handler_index: HANDLER_IDX_I32_LOAD16_S,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Load16U { memarg } => {
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryLoadSlot {
                            handler_index: HANDLER_IDX_I32_LOAD16_U,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Load8S { memarg } => {
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::MemoryLoadSlot {
                            handler_index: HANDLER_IDX_I64_LOAD8_S,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Load8U { memarg } => {
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::MemoryLoadSlot {
                            handler_index: HANDLER_IDX_I64_LOAD8_U,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Load16S { memarg } => {
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::MemoryLoadSlot {
                            handler_index: HANDLER_IDX_I64_LOAD16_S,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Load16U { memarg } => {
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::MemoryLoadSlot {
                            handler_index: HANDLER_IDX_I64_LOAD16_U,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Load32S { memarg } => {
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::MemoryLoadSlot {
                            handler_index: HANDLER_IDX_I64_LOAD32_S,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Load32U { memarg } => {
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::MemoryLoadSlot {
                            handler_index: HANDLER_IDX_I64_LOAD32_U,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                // Memory Store instructions
                wasmparser::Operator::I32Store { memarg } => {
                    let value = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryStoreSlot {
                            handler_index: HANDLER_IDX_I32_STORE,
                            addr,
                            value,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Store { memarg } => {
                    let value = allocator.pop(&ValueType::NumType(NumType::I64));
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryStoreSlot {
                            handler_index: HANDLER_IDX_I64_STORE,
                            addr,
                            value,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Store { memarg } => {
                    let value = allocator.pop(&ValueType::NumType(NumType::F32));
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryStoreSlot {
                            handler_index: HANDLER_IDX_F32_STORE,
                            addr,
                            value,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Store { memarg } => {
                    let value = allocator.pop(&ValueType::NumType(NumType::F64));
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryStoreSlot {
                            handler_index: HANDLER_IDX_F64_STORE,
                            addr,
                            value,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Store8 { memarg } => {
                    let value = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryStoreSlot {
                            handler_index: HANDLER_IDX_I32_STORE8,
                            addr,
                            value,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Store16 { memarg } => {
                    let value = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryStoreSlot {
                            handler_index: HANDLER_IDX_I32_STORE16,
                            addr,
                            value,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Store8 { memarg } => {
                    let value = allocator.pop(&ValueType::NumType(NumType::I64));
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryStoreSlot {
                            handler_index: HANDLER_IDX_I64_STORE8,
                            addr,
                            value,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Store16 { memarg } => {
                    let value = allocator.pop(&ValueType::NumType(NumType::I64));
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryStoreSlot {
                            handler_index: HANDLER_IDX_I64_STORE16,
                            addr,
                            value,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Store32 { memarg } => {
                    let value = allocator.pop(&ValueType::NumType(NumType::I64));
                    let addr = allocator.pop(&ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryStoreSlot {
                            handler_index: HANDLER_IDX_I64_STORE32,
                            addr,
                            value,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }

                // Memory Ops instructions (size, grow, copy, init, fill)
                wasmparser::Operator::MemorySize { .. } => {
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryOpsSlot {
                            handler_index: HANDLER_IDX_MEMORY_SIZE,
                            dst: Some(dst),
                            args: vec![],
                            data_index: 0,
                        },
                        None,
                    )
                }
                wasmparser::Operator::MemoryGrow { .. } => {
                    let delta = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryOpsSlot {
                            handler_index: HANDLER_IDX_MEMORY_GROW,
                            dst: Some(dst),
                            args: vec![delta],
                            data_index: 0,
                        },
                        None,
                    )
                }
                wasmparser::Operator::MemoryCopy { .. } => {
                    let len = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dest = allocator.pop(&ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryOpsSlot {
                            handler_index: HANDLER_IDX_MEMORY_COPY,
                            dst: None,
                            args: vec![dest, src, len],
                            data_index: 0,
                        },
                        None,
                    )
                }
                wasmparser::Operator::MemoryInit { data_index, .. } => {
                    let len = allocator.pop(&ValueType::NumType(NumType::I32));
                    let offset = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dest = allocator.pop(&ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryOpsSlot {
                            handler_index: HANDLER_IDX_MEMORY_INIT,
                            dst: None,
                            args: vec![dest, offset, len],
                            data_index: *data_index,
                        },
                        None,
                    )
                }
                wasmparser::Operator::MemoryFill { .. } => {
                    let size = allocator.pop(&ValueType::NumType(NumType::I32));
                    let val = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dest = allocator.pop(&ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryOpsSlot {
                            handler_index: HANDLER_IDX_MEMORY_FILL,
                            dst: None,
                            args: vec![dest, val, size],
                            data_index: 0,
                        },
                        None,
                    )
                }
                wasmparser::Operator::DataDrop { data_index } => (
                    ProcessedInstr::DataDropSlot {
                        data_index: *data_index,
                    },
                    None,
                ),

                // Select instructions
                wasmparser::Operator::TypedSelect { ty } => {
                    // TypedSelect has explicit type
                    let val_type = match_value_type(*ty);
                    let cond = allocator.pop(&ValueType::NumType(NumType::I32));
                    let val2 = allocator.pop(&val_type);
                    let val1 = allocator.pop(&val_type);
                    let dst = allocator.push(val_type.clone());

                    let handler_index = match &val_type {
                        ValueType::NumType(NumType::I32) => HANDLER_IDX_SELECT_I32,
                        ValueType::NumType(NumType::I64) => HANDLER_IDX_SELECT_I64,
                        ValueType::NumType(NumType::F32) => HANDLER_IDX_SELECT_F32,
                        ValueType::NumType(NumType::F64) => HANDLER_IDX_SELECT_F64,
                        ValueType::RefType(_) => HANDLER_IDX_SELECT_I64,
                        ValueType::VecType(_) => panic!("VecType not supported for Select"),
                    };

                    (
                        ProcessedInstr::SelectSlot {
                            handler_index,
                            dst,
                            val1,
                            val2,
                            cond,
                        },
                        None,
                    )
                }
                wasmparser::Operator::Select => {
                    // Untyped Select: only supports i32/i64/f32/f64 (not reftype)
                    let cond = allocator.pop(&ValueType::NumType(NumType::I32));

                    // Use peek_type to determine the type of val2 (top of stack after cond)
                    let val_type = allocator
                        .peek_type()
                        .cloned()
                        .unwrap_or(ValueType::NumType(NumType::I32));
                    let handler_index = match &val_type {
                        ValueType::NumType(NumType::I32) => HANDLER_IDX_SELECT_I32,
                        ValueType::NumType(NumType::I64) => HANDLER_IDX_SELECT_I64,
                        ValueType::NumType(NumType::F32) => HANDLER_IDX_SELECT_F32,
                        ValueType::NumType(NumType::F64) => HANDLER_IDX_SELECT_F64,
                        _ => panic!("Select requires numeric values on stack"),
                    };

                    let val2 = allocator.pop(&val_type);
                    let val1 = allocator.pop(&val_type);
                    let dst = allocator.push(val_type);

                    (
                        ProcessedInstr::SelectSlot {
                            handler_index,
                            dst,
                            val1,
                            val2,
                            cond,
                        },
                        None,
                    )
                }
                wasmparser::Operator::RefNull { hty } => {
                    let ref_type = match hty {
                        wasmparser::HeapType::Func => RefType::FuncRef,
                        wasmparser::HeapType::Extern => RefType::ExternalRef,
                        _ => RefType::ExternalRef,
                    };
                    let dst = allocator.push(ValueType::RefType(ref_type.clone()));
                    (
                        ProcessedInstr::TableRefSlot {
                            handler_index: HANDLER_IDX_REF_NULL_SLOT,
                            table_idx: 0,
                            slots: [dst.index(), 0, 0],
                            ref_type,
                        },
                        None,
                    )
                }

                wasmparser::Operator::RefIsNull => {
                    // ref.is_null operates on reference types only
                    let src = allocator.pop(&ValueType::RefType(RefType::FuncRef));
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::TableRefSlot {
                            handler_index: HANDLER_IDX_REF_IS_NULL_SLOT,
                            table_idx: 0,
                            slots: [dst.index(), src.index(), 0],
                            ref_type: RefType::FuncRef, // Not used for RefIsNull
                        },
                        None,
                    )
                }

                wasmparser::Operator::TableGet { table } => {
                    // table.get: [i32] -> [ref]
                    let ref_type_vt = get_table_element_type(module, *table);
                    let idx = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ref_type_vt.clone());
                    let ref_type = match ref_type_vt {
                        ValueType::RefType(rt) => rt,
                        _ => RefType::FuncRef,
                    };
                    (
                        ProcessedInstr::TableRefSlot {
                            handler_index: HANDLER_IDX_TABLE_GET_SLOT,
                            table_idx: *table,
                            slots: [dst.index(), idx.index(), 0],
                            ref_type,
                        },
                        None,
                    )
                }

                wasmparser::Operator::TableSet { table } => {
                    // table.set: [i32, ref] -> []
                    let ref_type_vt = get_table_element_type(module, *table);
                    let val = allocator.pop(&ref_type_vt);
                    let idx = allocator.pop(&ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::TableRefSlot {
                            handler_index: HANDLER_IDX_TABLE_SET_SLOT,
                            table_idx: *table,
                            slots: [idx.index(), val.index(), 0],
                            ref_type: RefType::FuncRef, // Not used for TableSet
                        },
                        None,
                    )
                }

                wasmparser::Operator::TableFill { table } => {
                    // table.fill: [i32, ref, i32] -> []
                    let ref_type_vt = get_table_element_type(module, *table);
                    let n = allocator.pop(&ValueType::NumType(NumType::I32));
                    let val = allocator.pop(&ref_type_vt);
                    let i = allocator.pop(&ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::TableRefSlot {
                            handler_index: HANDLER_IDX_TABLE_FILL_SLOT,
                            table_idx: *table,
                            slots: [i.index(), val.index(), n.index()],
                            ref_type: RefType::FuncRef, // Not used for TableFill
                        },
                        None,
                    )
                }

                _ => {
                    panic!("Unsupported instruction: {:?}", op);
                }
            }
        } else {
            panic!("Slot allocator is required");
        };

        let processed_instr_template = processed_instr;

        // --- Update Maps and Stacks based on operator ---
        match op {
            wasmparser::Operator::Block { blockty } => {
                control_stack_for_map_building.push((current_processed_pc, false, None));
                block_type_map.insert(current_processed_pc, blockty);
            }
            wasmparser::Operator::Loop { blockty } => {
                control_stack_for_map_building.push((current_processed_pc, false, None));
                block_type_map.insert(current_processed_pc, blockty);
            }
            wasmparser::Operator::If { blockty } => {
                control_stack_for_map_building.push((current_processed_pc, true, None));
                block_type_map.insert(current_processed_pc, blockty);
            }
            wasmparser::Operator::Else => {
                if let Some((_, true, else_pc @ None)) = control_stack_for_map_building.last_mut() {
                    *else_pc = Some(current_processed_pc + 1);
                } else {
                    return Err(Box::new(RuntimeError::InvalidWasm(
                        "Else without corresponding If or If already has Else",
                    )) as Box<dyn std::error::Error>);
                }
            }
            wasmparser::Operator::End => {
                if let Some((start_pc, is_if, else_pc_opt)) = control_stack_for_map_building.pop() {
                    block_end_map.insert(start_pc, current_processed_pc + 1);
                    if is_if {
                        let else_target = else_pc_opt.unwrap_or(current_processed_pc + 1);
                        if_else_map.insert(start_pc, else_target);
                    }
                } else {
                    if ops.peek().is_none() {
                    } else {
                        return Err(Box::new(RuntimeError::InvalidWasm("Unmatched EndMarker"))
                            as Box<dyn std::error::Error>);
                    }
                }
            }
            _ => {}
        }

        // All instructions are now slot-based
        initial_processed_instrs.push(processed_instr_template);
        if let Some(fixup_info) = fixup_info_opt {
            initial_fixups.push(fixup_info);
        }

        // Update control_info_stack and block_result_slots_map
        match op {
            wasmparser::Operator::Block { .. } => {
                // Register for BrTable resolution (always needed)
                if let Some(block_info) = control_info_stack.last() {
                    block_result_slots_map.insert(
                        current_processed_pc,
                        (block_info.result_slots.clone(), false),
                    );
                }
            }
            wasmparser::Operator::Loop { .. } => {
                // Register for BrTable resolution (always needed)
                // For loops, register param_slots (used when branching to loop)
                if let Some(block_info) = control_info_stack.last() {
                    block_result_slots_map
                        .insert(current_processed_pc, (block_info.param_slots.clone(), true));
                }
            }
            wasmparser::Operator::If { .. } => {
                // Register for BrTable resolution (always needed)
                if let Some(block_info) = control_info_stack.last() {
                    block_result_slots_map.insert(
                        current_processed_pc,
                        (block_info.result_slots.clone(), false),
                    );
                }
            }
            wasmparser::Operator::End => {
                // Slot mode End already popped in its match arm above
            }
            _ => {}
        }

        // Mark following code as unreachable after unconditional control flow
        match op {
            wasmparser::Operator::Br { .. }
            | wasmparser::Operator::BrTable { .. }
            | wasmparser::Operator::Return
            | wasmparser::Operator::Unreachable => {
                unreachable_depth = 1;
            }
            _ => {}
        }

        current_processed_pc += 1;
    }

    if !control_stack_for_map_building.is_empty() {
        return Err(Box::new(RuntimeError::InvalidWasm(
            "Unclosed control block at end of function",
        )) as Box<dyn std::error::Error>);
    }

    // Get result slot before finalizing (the top of stack after all instructions)
    // Use the function's result type to peek at the correct slot type
    let result_slot = slot_allocator.as_ref().and_then(|alloc| {
        if let Some(result_type) = result_types.first() {
            // Peek at the current stack top for the result type
            alloc.peek(result_type)
        } else {
            None
        }
    });

    // Finalize slot allocation if in slot mode
    let slot_allocation = slot_allocator.map(|alloc| alloc.finalize());

    Ok((
        initial_processed_instrs,
        initial_fixups,
        block_end_map,
        if_else_map,
        block_type_map,
        slot_allocation,
        result_slot,
        block_result_slots_map,
    ))
}

/// Compute source_slots and target_result_slots for branch instructions
/// source_slots: current stack top slots that will be copied
/// target_result_slots: where to copy them (the target block's result slots, or param_slots for loops)
fn compute_branch_slots(
    control_info_stack: &[ControlBlockInfo],
    relative_depth: usize,
    slot_allocator: Option<&SlotAllocator>,
) -> (Vec<Slot>, Vec<Slot>) {
    // Get target block from control_info_stack
    let stack_len = control_info_stack.len();
    if stack_len == 0 || relative_depth >= stack_len {
        return (vec![], vec![]);
    }

    let target_idx = stack_len - 1 - relative_depth;
    let target_block = &control_info_stack[target_idx];
    // For loops, use param_slots (branch provides parameters)
    // For blocks/if, use result_slots (branch provides results)
    let target_result_slots = if target_block.is_loop {
        target_block.param_slots.clone()
    } else {
        target_block.result_slots.clone()
    };

    // Compute source_slots from current allocator state
    let source_slots = if let Some(allocator) = slot_allocator {
        // Get the slots at stack top matching result types
        let result_count = target_result_slots.len();
        if result_count == 0 {
            vec![]
        } else {
            // Get current state and compute source slots
            let state = allocator.save_state();
            target_result_slots
                .iter()
                .enumerate()
                .map(|(i, target_slot)| {
                    // Source slot is at current_depth - result_count + i
                    match target_slot {
                        Slot::I32(_) => {
                            let src_idx = state.i32_depth.saturating_sub(result_count - i);
                            Slot::I32(src_idx as u16)
                        }
                        Slot::I64(_) => {
                            let src_idx = state.i64_depth.saturating_sub(result_count - i);
                            Slot::I64(src_idx as u16)
                        }
                        Slot::F32(_) => {
                            let src_idx = state.f32_depth.saturating_sub(result_count - i);
                            Slot::F32(src_idx as u16)
                        }
                        Slot::F64(_) => {
                            let src_idx = state.f64_depth.saturating_sub(result_count - i);
                            Slot::F64(src_idx as u16)
                        }
                        Slot::Ref(_) => {
                            let src_idx = state.ref_depth.saturating_sub(result_count - i);
                            Slot::Ref(src_idx as u16)
                        }
                        Slot::V128(_) => {
                            let src_idx = state.v128_depth.saturating_sub(result_count - i);
                            Slot::V128(src_idx as u16)
                        }
                    }
                })
                .collect()
        }
    } else {
        vec![]
    };

    (source_slots, target_result_slots)
}

pub fn parse_bytecode(
    mut module: &mut Module,
    path: &str,
    enable_superinstructions: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_func_index = module.num_imported_funcs;
    let mut arity_cache = BlockArityCache::new();

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
                decode_func_section(body, &mut module)?;
            }

            ImportSection(body) => {
                decode_import_section(body, &mut module)?;
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
                    enable_superinstructions,
                    &mut arity_cache,
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

    Ok(())
}
