use std::fs::File;
use std::io::Read;
use std::iter::Peekable;
use wasmparser::{
    ExternalKind, FunctionBody, OperatorsIteratorWithOffsets, Parser, Payload::*, SectionLimited,
    TypeRef, ValType,
};

use crate::error::{ParserError, RuntimeError};
use crate::execution::stack::*;
use crate::structure::{instructions::*, module::*, types::*};
use std::collections::HashMap;

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

        module
            .types
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
            body: Vec::new(),
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
                module.num_imported_funcs += 1;
                ImportDesc::Func(TypeIdx(type_index))
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
) -> Result<(), Box<dyn std::error::Error>> {
    let mut locals: Vec<(u32, ValueType)> = Vec::new();
    for pair in body.get_locals_reader()? {
        let (cnt, ty) = pair?;
        let ty = match_value_type(ty);
        locals.push((cnt, ty));
    }

    if let Some(func) = module.funcs.get_mut(func_index) {
        func.locals = locals;
    } else {
        return Err(Box::new(RuntimeError::InvalidWasm(
            "Invalid function index during code decoding",
        )) as Box<dyn std::error::Error>);
    }

    let ops_reader = body.get_operators_reader()?;
    let mut ops_iter = ops_reader.into_iter_with_offsets().peekable();

    // Phase 1: Decode instructions and get necessary info for preprocessing
    let (mut processed_instrs, mut fixups, block_end_map, if_else_map) =
        decode_processed_instrs_and_fixups(&mut ops_iter)?;

    // Phase 2 & 3: Preprocess instructions for this function
    preprocess_instructions(
        &mut processed_instrs,
        &mut fixups,
        &block_end_map,
        &if_else_map,
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    if let Some(func) = module.funcs.get_mut(func_index) {
        func.body = processed_instrs;
    } else {
        return Err(Box::new(RuntimeError::InvalidWasm(
            "Invalid function index when storing body",
        )) as Box<dyn std::error::Error>);
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct FixupInfo {
    pc: usize,
    original_wasm_depth: usize,
    is_if_false_jump: bool,
    is_else_jump: bool,
}

// Phase 2: Resolve Br, BrIf, If, Else jumps using maps and control stack simulation
// Phase 3: Resolve BrTable jumps similarly
fn preprocess_instructions(
    processed: &mut Vec<ProcessedInstr>,
    fixups: &mut Vec<FixupInfo>,
    block_end_map: &HashMap<usize, usize>,
    if_else_map: &HashMap<usize, usize>,
) -> Result<(), RuntimeError> {
    // --- Phase 2: Resolve Br, BrIf, If, Else jumps ---

    // Control stack stores: (pc, is_loop, block_type, runtime_label_stack_idx)
    let mut current_control_stack_pass2: Vec<(usize, bool, wasmparser::BlockType, usize)> =
        Vec::new();
    let mut runtime_label_stack_idx_counter;

    for fixup_index in 0..fixups.len() {
        let current_fixup_pc = fixups[fixup_index].pc;
        let current_fixup_depth = fixups[fixup_index].original_wasm_depth;
        let is_if_false_jump = fixups[fixup_index].is_if_false_jump;
        let is_else_jump = fixups[fixup_index].is_else_jump;

        let is_br_table_fixup = processed
            .get(current_fixup_pc)
            .map_or(false, |instr| instr.handler_index == HANDLER_IDX_BR_TABLE);

        if current_fixup_depth == usize::MAX || is_br_table_fixup {
            continue;
        }

        // --- Rebuild control stack state up to the fixup instruction ---
        current_control_stack_pass2.clear();
        let mut runtime_label_stack_idx_counter = 0;
        for (pc, instr) in processed.iter().enumerate().take(current_fixup_pc + 1) {
            match instr.handler_index {
                HANDLER_IDX_BLOCK | HANDLER_IDX_IF => {
                    let block_type = wasmparser::BlockType::Empty;
                    current_control_stack_pass2.push((
                        pc,
                        false,
                        block_type,
                        runtime_label_stack_idx_counter,
                    ));
                    runtime_label_stack_idx_counter += 1;
                }
                HANDLER_IDX_LOOP => {
                    let block_type = wasmparser::BlockType::Empty;
                    current_control_stack_pass2.push((
                        pc,
                        true,
                        block_type,
                        runtime_label_stack_idx_counter,
                    ));
                    runtime_label_stack_idx_counter += 1;
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
            fixups[fixup_index].original_wasm_depth = usize::MAX;
            continue;
        }

        let target_stack_level = current_control_stack_pass2.len() - 1 - current_fixup_depth;
        if target_stack_level >= current_control_stack_pass2.len() {
            fixups[fixup_index].original_wasm_depth = usize::MAX;
            continue;
        }

        let (target_start_pc, is_loop, _target_block_type, target_runtime_idx) =
            current_control_stack_pass2[target_stack_level];

        // Calculate target IP
        let target_ip = if is_loop {
            target_start_pc
        } else {
            *block_end_map
                .get(&target_start_pc)
                .ok_or_else(|| RuntimeError::InvalidWasm("Missing EndMarker for branch target"))?
        };

        let target_arity = 0;
        let target_label_stack_idx = target_runtime_idx;

        // Patch the instruction operand
        if let Some(instr_to_patch) = processed.get_mut(current_fixup_pc) {
            if is_if_false_jump {
                // If instruction's jump-on-false
                // Target is ElseMarker+1 or EndMarker+1
                let else_target = *if_else_map.get(&target_start_pc).unwrap_or(&target_ip);
                instr_to_patch.operand = Operand::LabelIdx {
                    target_ip: else_target,
                    arity: 0,
                    target_label_stack_idx: 0,
                };
            } else if is_else_jump {
                // Else instruction's jump-to-end
                instr_to_patch.operand = Operand::LabelIdx {
                    target_ip: target_ip,
                    arity: 0,
                    target_label_stack_idx: 0,
                };
            } else {
                // Br or BrIf instruction
                instr_to_patch.operand = Operand::LabelIdx {
                    target_ip,
                    arity: target_arity,
                    target_label_stack_idx,
                };
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
    let mut current_control_stack_pass3: Vec<(usize, bool, wasmparser::BlockType, usize)> =
        Vec::new();
    runtime_label_stack_idx_counter = 0; // Reset counter

    for pc in 0..processed.len() {
        if let Some(instr) = processed.get(pc) {
            match instr.handler_index {
                HANDLER_IDX_BLOCK | HANDLER_IDX_IF => {
                    let block_type = wasmparser::BlockType::Empty;
                    current_control_stack_pass3.push((
                        pc,
                        false,
                        block_type,
                        runtime_label_stack_idx_counter,
                    ));
                    runtime_label_stack_idx_counter += 1;
                }
                HANDLER_IDX_LOOP => {
                    let block_type = wasmparser::BlockType::Empty;
                    current_control_stack_pass3.push((
                        pc,
                        true,
                        block_type,
                        runtime_label_stack_idx_counter,
                    ));
                    runtime_label_stack_idx_counter += 1;
                }
                HANDLER_IDX_END => {
                    if !current_control_stack_pass3.is_empty() {
                        current_control_stack_pass3.pop();
                    }
                }
                _ => {}
            }

            // Check if it's a BrTable needing resolution *after* simulating stack for current pc
            let needs_br_table_resolution =
                instr.handler_index == HANDLER_IDX_BR_TABLE && instr.operand == Operand::None;

            if needs_br_table_resolution {
                // Find fixup indices associated *only* with this BrTable pc that haven't been processed yet
                let mut fixup_indices_for_this_br_table = fixups
                    .iter()
                    .enumerate()
                    .filter(|(_, fixup)| fixup.pc == pc && fixup.original_wasm_depth != usize::MAX)
                    .map(|(idx, _)| idx)
                    .collect::<Vec<_>>();

                if fixup_indices_for_this_br_table.is_empty() {
                    if let Some(instr_to_patch) = processed.get_mut(pc) {
                        instr_to_patch.operand = Operand::BrTable {
                            targets: vec![],
                            default: Box::new(Operand::None),
                        };
                    }
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

                    let (target_start_pc, is_loop, _target_block_type, target_runtime_idx) =
                        current_control_stack_pass3[target_stack_level];

                    let target_ip = if is_loop {
                        target_start_pc
                    } else {
                        *block_end_map.get(&target_start_pc).ok_or_else(|| {
                            RuntimeError::InvalidWasm(
                                "Missing EndMarker for BrTable default target",
                            )
                        })?
                    };
                    let target_arity = 0;
                    let target_label_stack_idx = target_runtime_idx;

                    fixups[default_fixup_idx].original_wasm_depth = usize::MAX;

                    Operand::LabelIdx {
                        target_ip,
                        arity: target_arity,
                        target_label_stack_idx,
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

                        let (target_start_pc, is_loop, _target_block_type, target_runtime_idx) =
                            current_control_stack_pass3[target_stack_level];
                        let target_ip = if is_loop {
                            target_start_pc
                        } else {
                            *block_end_map.get(&target_start_pc).ok_or_else(|| {
                                RuntimeError::InvalidWasm("Missing EndMarker for BrTable target")
                            })?
                        };
                        let target_arity = 0;
                        let target_label_stack_idx = target_runtime_idx;

                        fixups[fixup_idx].original_wasm_depth = usize::MAX;

                        Operand::LabelIdx {
                            target_ip,
                            arity: target_arity,
                            target_label_stack_idx,
                        }
                    };
                    resolved_targets.push(target_operand);
                }

                // --- Patch BrTable Instruction ---
                if let Some(instr_to_patch) = processed.get_mut(pc) {
                    instr_to_patch.operand = Operand::BrTable {
                        targets: resolved_targets,
                        default: Box::new(default_target_operand),
                    };
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

fn decode_processed_instrs_and_fixups(
    ops: &mut Peekable<OperatorsIteratorWithOffsets<'_>>,
) -> Result<
    (
        Vec<ProcessedInstr>,
        Vec<FixupInfo>,
        HashMap<usize, usize>,
        HashMap<usize, usize>,
    ),
    Box<dyn std::error::Error>,
> {
    let mut initial_processed_instrs = Vec::new();
    let mut initial_fixups = Vec::new();
    let mut current_processed_pc = 0;
    let mut control_info_stack: Vec<(wasmparser::BlockType, usize)> = Vec::new();

    let mut block_end_map: HashMap<usize, usize> = HashMap::new();
    let mut if_else_map: HashMap<usize, usize> = HashMap::new();
    let mut control_stack_for_map_building: Vec<(usize, bool, Option<usize>)> = Vec::new();

    loop {
        if ops.peek().is_none() {
            break;
        }

        let (op, _offset) = match ops.next() {
            Some(Ok(op_offset)) => op_offset,
            Some(Err(e)) => return Err(Box::new(e)),
            None => break,
        };

        let (processed_instr_template, fixup_info_opt) = map_operator_to_initial_instr_and_fixup(
            &op,
            current_processed_pc,
            &control_info_stack,
        )?;

        // --- Update Maps and Stacks based on operator ---
        match op {
            wasmparser::Operator::Block { blockty: _ } => {
                control_stack_for_map_building.push((current_processed_pc, false, None));
            }
            wasmparser::Operator::Loop { blockty: _ } => {
                control_stack_for_map_building.push((current_processed_pc, false, None));
            }
            wasmparser::Operator::If { blockty: _ } => {
                control_stack_for_map_building.push((current_processed_pc, true, None));
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

        if let wasmparser::Operator::BrTable { ref targets } = op {
            let processed_instr = ProcessedInstr {
                handler_index: HANDLER_IDX_BR_TABLE,
                operand: Operand::None,
            };
            initial_processed_instrs.push(processed_instr);

            let table_targets = targets.targets().collect::<Result<Vec<_>, _>>()?;
            for target_depth in table_targets.iter() {
                let fixup = FixupInfo {
                    pc: current_processed_pc,
                    original_wasm_depth: *target_depth as usize,
                    is_if_false_jump: false,
                    is_else_jump: false,
                };
                initial_fixups.push(fixup);
            }
            let default_fixup = FixupInfo {
                pc: current_processed_pc,
                original_wasm_depth: targets.default() as usize,
                is_if_false_jump: false,
                is_else_jump: false,
            };
            initial_fixups.push(default_fixup);
        } else {
            initial_processed_instrs.push(processed_instr_template);
            if let Some(fixup_info) = fixup_info_opt {
                initial_fixups.push(fixup_info);
            }

            match op {
                wasmparser::Operator::Block { blockty } => {
                    control_info_stack.push((blockty, current_processed_pc));
                }
                wasmparser::Operator::Loop { blockty } => {
                    control_info_stack.push((blockty, current_processed_pc));
                }
                wasmparser::Operator::If { blockty } => {
                    control_info_stack.push((blockty, current_processed_pc));
                }
                wasmparser::Operator::End => {
                    if !control_info_stack.is_empty() {
                        control_info_stack.pop();
                    }
                }
                _ => {}
            }
        }

        current_processed_pc += 1;
    }

    if !control_stack_for_map_building.is_empty() {
        return Err(Box::new(RuntimeError::InvalidWasm(
            "Unclosed control block at end of function",
        )) as Box<dyn std::error::Error>);
    }

    Ok((
        initial_processed_instrs,
        initial_fixups,
        block_end_map,
        if_else_map,
    ))
}

fn map_operator_to_initial_instr_and_fixup(
    op: &wasmparser::Operator,
    current_processed_pc: usize,
    _control_info_stack: &[(wasmparser::BlockType, usize)],
) -> Result<(ProcessedInstr, Option<FixupInfo>), Box<dyn std::error::Error>> {
    let handler_index;
    let mut operand = Operand::None;
    let mut fixup_info = None;

    match *op {
        wasmparser::Operator::Unreachable => {
            handler_index = HANDLER_IDX_UNREACHABLE;
        }
        wasmparser::Operator::Nop => {
            handler_index = HANDLER_IDX_NOP;
        }
        wasmparser::Operator::Block { blockty: _ } => {
            handler_index = HANDLER_IDX_BLOCK;
        }
        wasmparser::Operator::Loop { blockty: _ } => {
            handler_index = HANDLER_IDX_LOOP;
        }
        wasmparser::Operator::If { blockty: _ } => {
            handler_index = HANDLER_IDX_IF;
            fixup_info = Some(FixupInfo {
                pc: current_processed_pc,
                original_wasm_depth: 0,
                is_if_false_jump: true,
                is_else_jump: false,
            });
            operand = Operand::LabelIdx {
                target_ip: usize::MAX,
                arity: 0,
                target_label_stack_idx: 0,
            };
        }
        wasmparser::Operator::Else => {
            handler_index = HANDLER_IDX_ELSE;
            fixup_info = Some(FixupInfo {
                pc: current_processed_pc,
                original_wasm_depth: 0,
                is_if_false_jump: false,
                is_else_jump: true,
            });
            operand = Operand::LabelIdx {
                target_ip: usize::MAX,
                arity: 0,
                target_label_stack_idx: 0,
            };
        }
        wasmparser::Operator::End => {
            handler_index = HANDLER_IDX_END;
        }
        wasmparser::Operator::Br { relative_depth } => {
            handler_index = HANDLER_IDX_BR;
            fixup_info = Some(FixupInfo {
                pc: current_processed_pc,
                original_wasm_depth: relative_depth as usize,
                is_if_false_jump: false,
                is_else_jump: false,
            });
            operand = Operand::LabelIdx {
                target_ip: usize::MAX,
                arity: 0,
                target_label_stack_idx: 0,
            };
        }
        wasmparser::Operator::BrIf { relative_depth } => {
            handler_index = HANDLER_IDX_BR_IF;

            fixup_info = Some(FixupInfo {
                pc: current_processed_pc,
                original_wasm_depth: relative_depth as usize,
                is_if_false_jump: false,
                is_else_jump: false,
            });
            operand = Operand::LabelIdx {
                target_ip: usize::MAX,
                arity: 0,
                target_label_stack_idx: 0,
            };
        }
        wasmparser::Operator::BrTable { targets: _ } => {
            handler_index = HANDLER_IDX_BR_TABLE;
            operand = Operand::None;
            fixup_info = None;
        }
        wasmparser::Operator::Return => {
            handler_index = HANDLER_IDX_RETURN;
        }
        wasmparser::Operator::Call { function_index } => {
            handler_index = HANDLER_IDX_CALL;
            operand = Operand::FuncIdx(FuncIdx(function_index));
        }
        wasmparser::Operator::CallIndirect {
            type_index,
            table_index,
            ..
        } => {
            handler_index = HANDLER_IDX_CALL_INDIRECT;
            if table_index != 0 {
                return Err(
                    Box::new(RuntimeError::InvalidWasm("Only table index 0 is supported"))
                        as Box<dyn std::error::Error>,
                );
            }
            operand = Operand::TypeIdx(TypeIdx(type_index));
        }

        /* Parametric Instructions */
        wasmparser::Operator::Drop => {
            handler_index = HANDLER_IDX_DROP;
        }
        wasmparser::Operator::Select => {
            handler_index = HANDLER_IDX_SELECT;
        }
        wasmparser::Operator::TypedSelect { .. } => {
            handler_index = HANDLER_IDX_SELECT;
        }

        /* Variable Instructions */
        wasmparser::Operator::LocalGet { local_index } => {
            handler_index = HANDLER_IDX_LOCAL_GET;
            operand = Operand::LocalIdx(LocalIdx(local_index));
        }
        wasmparser::Operator::LocalSet { local_index } => {
            handler_index = HANDLER_IDX_LOCAL_SET;
            operand = Operand::LocalIdx(LocalIdx(local_index));
        }
        wasmparser::Operator::LocalTee { local_index } => {
            handler_index = HANDLER_IDX_LOCAL_TEE;
            operand = Operand::LocalIdx(LocalIdx(local_index));
        }
        wasmparser::Operator::GlobalGet { global_index } => {
            handler_index = HANDLER_IDX_GLOBAL_GET;
            operand = Operand::GlobalIdx(GlobalIdx(global_index));
        }
        wasmparser::Operator::GlobalSet { global_index } => {
            handler_index = HANDLER_IDX_GLOBAL_SET;
            operand = Operand::GlobalIdx(GlobalIdx(global_index));
        }

        /* Memory Instructions */
        wasmparser::Operator::I32Load { memarg } => {
            handler_index = HANDLER_IDX_I32_LOAD;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I64Load { memarg } => {
            handler_index = HANDLER_IDX_I64_LOAD;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::F32Load { memarg } => {
            handler_index = HANDLER_IDX_F32_LOAD;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::F64Load { memarg } => {
            handler_index = HANDLER_IDX_F64_LOAD;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I32Load8S { memarg } => {
            handler_index = HANDLER_IDX_I32_LOAD8_S;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I32Load8U { memarg } => {
            handler_index = HANDLER_IDX_I32_LOAD8_U;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I32Load16S { memarg } => {
            handler_index = HANDLER_IDX_I32_LOAD16_S;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I32Load16U { memarg } => {
            handler_index = HANDLER_IDX_I32_LOAD16_U;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I64Load8S { memarg } => {
            handler_index = HANDLER_IDX_I64_LOAD8_S;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I64Load8U { memarg } => {
            handler_index = HANDLER_IDX_I64_LOAD8_U;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I64Load16S { memarg } => {
            handler_index = HANDLER_IDX_I64_LOAD16_S;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I64Load16U { memarg } => {
            handler_index = HANDLER_IDX_I64_LOAD16_U;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I64Load32S { memarg } => {
            handler_index = HANDLER_IDX_I64_LOAD32_S;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I64Load32U { memarg } => {
            handler_index = HANDLER_IDX_I64_LOAD32_U;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I32Store { memarg } => {
            handler_index = HANDLER_IDX_I32_STORE;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I64Store { memarg } => {
            handler_index = HANDLER_IDX_I64_STORE;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::F32Store { memarg } => {
            handler_index = HANDLER_IDX_F32_STORE;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::F64Store { memarg } => {
            handler_index = HANDLER_IDX_F64_STORE;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I32Store8 { memarg } => {
            handler_index = HANDLER_IDX_I32_STORE8;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I32Store16 { memarg } => {
            handler_index = HANDLER_IDX_I32_STORE16;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I64Store8 { memarg } => {
            handler_index = HANDLER_IDX_I64_STORE8;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I64Store16 { memarg } => {
            handler_index = HANDLER_IDX_I64_STORE16;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::I64Store32 { memarg } => {
            handler_index = HANDLER_IDX_I64_STORE32;
            operand = Operand::MemArg(Memarg {
                offset: memarg.offset as u32,
                align: memarg.align as u32,
            });
        }
        wasmparser::Operator::MemorySize { .. } => {
            handler_index = HANDLER_IDX_MEMORY_SIZE;
        }
        wasmparser::Operator::MemoryGrow { .. } => {
            handler_index = HANDLER_IDX_MEMORY_GROW;
        }
        // TODO: MemoryFill, MemoryCopy, MemoryInit, DataDrop

        /* Numeric Instructions */
        wasmparser::Operator::I32Const { value } => {
            handler_index = HANDLER_IDX_I32_CONST;
            operand = Operand::I32(value);
        }
        wasmparser::Operator::I64Const { value } => {
            handler_index = HANDLER_IDX_I64_CONST;
            operand = Operand::I64(value);
        }
        wasmparser::Operator::F32Const { value } => {
            handler_index = HANDLER_IDX_F32_CONST;
            operand = Operand::F32(f32::from_bits(value.bits()));
        }
        wasmparser::Operator::F64Const { value } => {
            handler_index = HANDLER_IDX_F64_CONST;
            operand = Operand::F64(f64::from_bits(value.bits()));
        }
        wasmparser::Operator::I32Clz => {
            handler_index = HANDLER_IDX_I32_CLZ;
        }
        wasmparser::Operator::I32Ctz => {
            handler_index = HANDLER_IDX_I32_CTZ;
        }
        wasmparser::Operator::I32Popcnt => {
            handler_index = HANDLER_IDX_I32_POPCNT;
        }
        wasmparser::Operator::I64Clz => {
            handler_index = HANDLER_IDX_I64_CLZ;
        }
        wasmparser::Operator::I64Ctz => {
            handler_index = HANDLER_IDX_I64_CTZ;
        }
        wasmparser::Operator::I64Popcnt => {
            handler_index = HANDLER_IDX_I64_POPCNT;
        }
        wasmparser::Operator::F32Abs => {
            handler_index = HANDLER_IDX_F32_ABS;
        }
        wasmparser::Operator::F32Neg => {
            handler_index = HANDLER_IDX_F32_NEG;
        }
        wasmparser::Operator::F32Sqrt => {
            handler_index = HANDLER_IDX_F32_SQRT;
        }
        wasmparser::Operator::F32Ceil => {
            handler_index = HANDLER_IDX_F32_CEIL;
        }
        wasmparser::Operator::F32Floor => {
            handler_index = HANDLER_IDX_F32_FLOOR;
        }
        wasmparser::Operator::F32Trunc => {
            handler_index = HANDLER_IDX_F32_TRUNC;
        }
        wasmparser::Operator::F32Nearest => {
            handler_index = HANDLER_IDX_F32_NEAREST;
        }
        wasmparser::Operator::F64Abs => {
            handler_index = HANDLER_IDX_F64_ABS;
        }
        wasmparser::Operator::F64Neg => {
            handler_index = HANDLER_IDX_F64_NEG;
        }
        wasmparser::Operator::F64Sqrt => {
            handler_index = HANDLER_IDX_F64_SQRT;
        }
        wasmparser::Operator::F64Ceil => {
            handler_index = HANDLER_IDX_F64_CEIL;
        }
        wasmparser::Operator::F64Floor => {
            handler_index = HANDLER_IDX_F64_FLOOR;
        }
        wasmparser::Operator::F64Trunc => {
            handler_index = HANDLER_IDX_F64_TRUNC;
        }
        wasmparser::Operator::F64Nearest => {
            handler_index = HANDLER_IDX_F64_NEAREST;
        }
        wasmparser::Operator::I32Add => {
            handler_index = HANDLER_IDX_I32_ADD;
        }
        wasmparser::Operator::I32Sub => {
            handler_index = HANDLER_IDX_I32_SUB;
        }
        wasmparser::Operator::I32Mul => {
            handler_index = HANDLER_IDX_I32_MUL;
        }
        wasmparser::Operator::I32DivS => {
            handler_index = HANDLER_IDX_I32_DIV_S;
        }
        wasmparser::Operator::I32DivU => {
            handler_index = HANDLER_IDX_I32_DIV_U;
        }
        wasmparser::Operator::I32RemS => {
            handler_index = HANDLER_IDX_I32_REM_S;
        }
        wasmparser::Operator::I32RemU => {
            handler_index = HANDLER_IDX_I32_REM_U;
        }
        wasmparser::Operator::I32And => {
            handler_index = HANDLER_IDX_I32_AND;
        }
        wasmparser::Operator::I32Or => {
            handler_index = HANDLER_IDX_I32_OR;
        }
        wasmparser::Operator::I32Xor => {
            handler_index = HANDLER_IDX_I32_XOR;
        }
        wasmparser::Operator::I32Shl => {
            handler_index = HANDLER_IDX_I32_SHL;
        }
        wasmparser::Operator::I32ShrS => {
            handler_index = HANDLER_IDX_I32_SHR_S;
        }
        wasmparser::Operator::I32ShrU => {
            handler_index = HANDLER_IDX_I32_SHR_U;
        }
        wasmparser::Operator::I32Rotl => {
            handler_index = HANDLER_IDX_I32_ROTL;
        }
        wasmparser::Operator::I32Rotr => {
            handler_index = HANDLER_IDX_I32_ROTR;
        }
        wasmparser::Operator::I64Add => {
            handler_index = HANDLER_IDX_I64_ADD;
        }
        wasmparser::Operator::I64Sub => {
            handler_index = HANDLER_IDX_I64_SUB;
        }
        wasmparser::Operator::I64Mul => {
            handler_index = HANDLER_IDX_I64_MUL;
        }
        wasmparser::Operator::I64DivS => {
            handler_index = HANDLER_IDX_I64_DIV_S;
        }
        wasmparser::Operator::I64DivU => {
            handler_index = HANDLER_IDX_I64_DIV_U;
        }
        wasmparser::Operator::I64RemS => {
            handler_index = HANDLER_IDX_I64_REM_S;
        }
        wasmparser::Operator::I64RemU => {
            handler_index = HANDLER_IDX_I64_REM_U;
        }
        wasmparser::Operator::I64And => {
            handler_index = HANDLER_IDX_I64_AND;
        }
        wasmparser::Operator::I64Or => {
            handler_index = HANDLER_IDX_I64_OR;
        }
        wasmparser::Operator::I64Xor => {
            handler_index = HANDLER_IDX_I64_XOR;
        }
        wasmparser::Operator::I64Shl => {
            handler_index = HANDLER_IDX_I64_SHL;
        }
        wasmparser::Operator::I64ShrS => {
            handler_index = HANDLER_IDX_I64_SHR_S;
        }
        wasmparser::Operator::I64ShrU => {
            handler_index = HANDLER_IDX_I64_SHR_U;
        }
        wasmparser::Operator::I64Rotl => {
            handler_index = HANDLER_IDX_I64_ROTL;
        }
        wasmparser::Operator::I64Rotr => {
            handler_index = HANDLER_IDX_I64_ROTR;
        }
        wasmparser::Operator::F32Add => {
            handler_index = HANDLER_IDX_F32_ADD;
        }
        wasmparser::Operator::F32Sub => {
            handler_index = HANDLER_IDX_F32_SUB;
        }
        wasmparser::Operator::F32Mul => {
            handler_index = HANDLER_IDX_F32_MUL;
        }
        wasmparser::Operator::F32Div => {
            handler_index = HANDLER_IDX_F32_DIV;
        }
        wasmparser::Operator::F32Min => {
            handler_index = HANDLER_IDX_F32_MIN;
        }
        wasmparser::Operator::F32Max => {
            handler_index = HANDLER_IDX_F32_MAX;
        }
        wasmparser::Operator::F32Copysign => {
            handler_index = HANDLER_IDX_F32_COPYSIGN;
        }
        wasmparser::Operator::F64Add => {
            handler_index = HANDLER_IDX_F64_ADD;
        }
        wasmparser::Operator::F64Sub => {
            handler_index = HANDLER_IDX_F64_SUB;
        }
        wasmparser::Operator::F64Mul => {
            handler_index = HANDLER_IDX_F64_MUL;
        }
        wasmparser::Operator::F64Div => {
            handler_index = HANDLER_IDX_F64_DIV;
        }
        wasmparser::Operator::F64Min => {
            handler_index = HANDLER_IDX_F64_MIN;
        }
        wasmparser::Operator::F64Max => {
            handler_index = HANDLER_IDX_F64_MAX;
        }
        wasmparser::Operator::F64Copysign => {
            handler_index = HANDLER_IDX_F64_COPYSIGN;
        }
        wasmparser::Operator::I32Eqz => {
            handler_index = HANDLER_IDX_I32_EQZ;
        }
        wasmparser::Operator::I64Eqz => {
            handler_index = HANDLER_IDX_I64_EQZ;
        }
        wasmparser::Operator::I32Eq => {
            handler_index = HANDLER_IDX_I32_EQ;
        }
        wasmparser::Operator::I32Ne => {
            handler_index = HANDLER_IDX_I32_NE;
        }
        wasmparser::Operator::I32LtS => {
            handler_index = HANDLER_IDX_I32_LT_S;
        }
        wasmparser::Operator::I32LtU => {
            handler_index = HANDLER_IDX_I32_LT_U;
        }
        wasmparser::Operator::I32GtS => {
            handler_index = HANDLER_IDX_I32_GT_S;
        }
        wasmparser::Operator::I32GtU => {
            handler_index = HANDLER_IDX_I32_GT_U;
        }
        wasmparser::Operator::I32LeS => {
            handler_index = HANDLER_IDX_I32_LE_S;
        }
        wasmparser::Operator::I32LeU => {
            handler_index = HANDLER_IDX_I32_LE_U;
        }
        wasmparser::Operator::I32GeS => {
            handler_index = HANDLER_IDX_I32_GE_S;
        }
        wasmparser::Operator::I32GeU => {
            handler_index = HANDLER_IDX_I32_GE_U;
        }
        wasmparser::Operator::I64Eq => {
            handler_index = HANDLER_IDX_I64_EQ;
        }
        wasmparser::Operator::I64Ne => {
            handler_index = HANDLER_IDX_I64_NE;
        }
        wasmparser::Operator::I64LtS => {
            handler_index = HANDLER_IDX_I64_LT_S;
        }
        wasmparser::Operator::I64LtU => {
            handler_index = HANDLER_IDX_I64_LT_U;
        }
        wasmparser::Operator::I64GtS => {
            handler_index = HANDLER_IDX_I64_GT_S;
        }
        wasmparser::Operator::I64GtU => {
            handler_index = HANDLER_IDX_I64_GT_U;
        }
        wasmparser::Operator::I64LeS => {
            handler_index = HANDLER_IDX_I64_LE_S;
        }
        wasmparser::Operator::I64LeU => {
            handler_index = HANDLER_IDX_I64_LE_U;
        }
        wasmparser::Operator::I64GeS => {
            handler_index = HANDLER_IDX_I64_GE_S;
        }
        wasmparser::Operator::I64GeU => {
            handler_index = HANDLER_IDX_I64_GE_U;
        }
        wasmparser::Operator::F32Eq => {
            handler_index = HANDLER_IDX_F32_EQ;
        }
        wasmparser::Operator::F32Ne => {
            handler_index = HANDLER_IDX_F32_NE;
        }
        wasmparser::Operator::F32Lt => {
            handler_index = HANDLER_IDX_F32_LT;
        }
        wasmparser::Operator::F32Gt => {
            handler_index = HANDLER_IDX_F32_GT;
        }
        wasmparser::Operator::F32Le => {
            handler_index = HANDLER_IDX_F32_LE;
        }
        wasmparser::Operator::F32Ge => {
            handler_index = HANDLER_IDX_F32_GE;
        }
        wasmparser::Operator::F64Eq => {
            handler_index = HANDLER_IDX_F64_EQ;
        }
        wasmparser::Operator::F64Ne => {
            handler_index = HANDLER_IDX_F64_NE;
        }
        wasmparser::Operator::F64Lt => {
            handler_index = HANDLER_IDX_F64_LT;
        }
        wasmparser::Operator::F64Gt => {
            handler_index = HANDLER_IDX_F64_GT;
        }
        wasmparser::Operator::F64Le => {
            handler_index = HANDLER_IDX_F64_LE;
        }
        wasmparser::Operator::F64Ge => {
            handler_index = HANDLER_IDX_F64_GE;
        }
        wasmparser::Operator::I32WrapI64 => {
            handler_index = HANDLER_IDX_I32_WRAP_I64;
        }
        wasmparser::Operator::I64ExtendI32U => {
            handler_index = HANDLER_IDX_I64_EXTEND_I32_U;
        }
        wasmparser::Operator::I64ExtendI32S => {
            handler_index = HANDLER_IDX_I64_EXTEND_I32_S;
        }
        wasmparser::Operator::I32TruncF32S => {
            handler_index = HANDLER_IDX_I32_TRUNC_F32_S;
        }
        wasmparser::Operator::I32TruncF32U => {
            handler_index = HANDLER_IDX_I32_TRUNC_F32_U;
        }
        wasmparser::Operator::I32TruncF64S => {
            handler_index = HANDLER_IDX_I32_TRUNC_F64_S;
        }
        wasmparser::Operator::I32TruncF64U => {
            handler_index = HANDLER_IDX_I32_TRUNC_F64_U;
        }
        wasmparser::Operator::I64TruncF32S => {
            handler_index = HANDLER_IDX_I64_TRUNC_F32_S;
        }
        wasmparser::Operator::I64TruncF32U => {
            handler_index = HANDLER_IDX_I64_TRUNC_F32_U;
        }
        wasmparser::Operator::I64TruncF64S => {
            handler_index = HANDLER_IDX_I64_TRUNC_F64_S;
        }
        wasmparser::Operator::I64TruncF64U => {
            handler_index = HANDLER_IDX_I64_TRUNC_F64_U;
        }
        wasmparser::Operator::I32TruncSatF32S => {
            handler_index = HANDLER_IDX_NOP;
        }
        wasmparser::Operator::I32TruncSatF32U => {
            handler_index = HANDLER_IDX_NOP;
        }
        wasmparser::Operator::I32TruncSatF64S => {
            handler_index = HANDLER_IDX_NOP;
        }
        wasmparser::Operator::I32TruncSatF64U => {
            handler_index = HANDLER_IDX_NOP;
        }
        wasmparser::Operator::I64TruncSatF32S => {
            handler_index = HANDLER_IDX_NOP;
        }
        wasmparser::Operator::I64TruncSatF32U => {
            handler_index = HANDLER_IDX_NOP;
        }
        wasmparser::Operator::I64TruncSatF64S => {
            handler_index = HANDLER_IDX_NOP;
        }
        wasmparser::Operator::I64TruncSatF64U => {
            handler_index = HANDLER_IDX_NOP;
        }
        wasmparser::Operator::F32DemoteF64 => {
            handler_index = HANDLER_IDX_F32_DEMOTE_F64;
        }
        wasmparser::Operator::F64PromoteF32 => {
            handler_index = HANDLER_IDX_F64_PROMOTE_F32;
        }
        wasmparser::Operator::F32ConvertI32S => {
            handler_index = HANDLER_IDX_F32_CONVERT_I32_S;
        }
        wasmparser::Operator::F32ConvertI32U => {
            handler_index = HANDLER_IDX_F32_CONVERT_I32_U;
        }
        wasmparser::Operator::F32ConvertI64S => {
            handler_index = HANDLER_IDX_F32_CONVERT_I64_S;
        }
        wasmparser::Operator::F32ConvertI64U => {
            handler_index = HANDLER_IDX_F32_CONVERT_I64_U;
        }
        wasmparser::Operator::F64ConvertI32S => {
            handler_index = HANDLER_IDX_F64_CONVERT_I32_S;
        }
        wasmparser::Operator::F64ConvertI32U => {
            handler_index = HANDLER_IDX_F64_CONVERT_I32_U;
        }
        wasmparser::Operator::F64ConvertI64S => {
            handler_index = HANDLER_IDX_F64_CONVERT_I64_S;
        }
        wasmparser::Operator::F64ConvertI64U => {
            handler_index = HANDLER_IDX_F64_CONVERT_I64_U;
        }
        wasmparser::Operator::I32ReinterpretF32 => {
            handler_index = HANDLER_IDX_I32_REINTERPRET_F32;
        }
        wasmparser::Operator::I64ReinterpretF64 => {
            handler_index = HANDLER_IDX_I64_REINTERPRET_F64;
        }
        wasmparser::Operator::F32ReinterpretI32 => {
            handler_index = HANDLER_IDX_F32_REINTERPRET_I32;
        }
        wasmparser::Operator::F64ReinterpretI64 => {
            handler_index = HANDLER_IDX_F64_REINTERPRET_I64;
        }
        wasmparser::Operator::I32Extend8S => {
            handler_index = HANDLER_IDX_I32_EXTEND8_S;
        }
        wasmparser::Operator::I32Extend16S => {
            handler_index = HANDLER_IDX_I32_EXTEND16_S;
        }
        wasmparser::Operator::I64Extend8S => {
            handler_index = HANDLER_IDX_I64_EXTEND8_S;
        }
        wasmparser::Operator::I64Extend16S => {
            handler_index = HANDLER_IDX_I64_EXTEND16_S;
        }
        wasmparser::Operator::I64Extend32S => {
            handler_index = HANDLER_IDX_I64_EXTEND32_S;
        }

        /* Reference Instructions */
        wasmparser::Operator::RefNull { hty: _ } => {
            handler_index = HANDLER_IDX_NOP;
            operand = Operand::None;
        }
        wasmparser::Operator::RefIsNull => {
            handler_index = HANDLER_IDX_NOP;
        }
        wasmparser::Operator::RefFunc { function_index } => {
            handler_index = HANDLER_IDX_NOP;
            operand = Operand::FuncIdx(FuncIdx(function_index));
        }
        wasmparser::Operator::RefEq => {
            handler_index = HANDLER_IDX_NOP;
            println!("Warning: Unhandled RefEq");
        }

        /* Table Instructions */
        wasmparser::Operator::TableGet { table } => {
            handler_index = HANDLER_IDX_NOP;
            operand = Operand::TableIdx(TableIdx(table));
        }
        wasmparser::Operator::TableSet { table } => {
            handler_index = HANDLER_IDX_NOP;
            operand = Operand::TableIdx(TableIdx(table));
        }
        wasmparser::Operator::TableSize { table } => {
            handler_index = HANDLER_IDX_NOP;
            operand = Operand::TableIdx(TableIdx(table));
        }
        wasmparser::Operator::TableGrow { table } => {
            handler_index = HANDLER_IDX_NOP;
            operand = Operand::TableIdx(TableIdx(table));
        }
        wasmparser::Operator::TableFill { table } => {
            handler_index = HANDLER_IDX_NOP;
            operand = Operand::TableIdx(TableIdx(table));
        }
        wasmparser::Operator::TableCopy {
            dst_table: _,
            src_table: _,
        } => {
            handler_index = HANDLER_IDX_NOP;
        }
        wasmparser::Operator::TableInit {
            elem_index: _,
            table: _,
        } => {
            handler_index = HANDLER_IDX_NOP;
        }
        wasmparser::Operator::ElemDrop { elem_index: _ } => {
            handler_index = HANDLER_IDX_NOP;
        }

        _ => {
            handler_index = HANDLER_IDX_NOP;
        }
    };

    let processed_instr = ProcessedInstr {
        handler_index,
        operand,
    };
    Ok((processed_instr, fixup_info))
}

pub fn parse_bytecode(
    mut module: &mut Module,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_func_index = module.num_imported_funcs;

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
                decode_code_section(body, &mut module, current_func_index)?;
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

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use std::iter::Peekable;
    use wasmparser::{
        ExternalKind, FunctionBody, OperatorsIteratorWithOffsets, Parser, Payload::*,
        SectionLimited, TypeRef, ValType,
    };

    use crate::error::ParserError;
    use crate::parser;
    use crate::structure::{instructions::*, module::*, types::*};
    use wasmparser::Payload::*;
    #[test]
    fn decode_type_section() {
        let wat = r#"
        (module
            (func (param i32 i32 i64))
            (func (result i32 i32))
            (func (param i32 i32) (result i32))
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary) {
            match payload.unwrap() {
                TypeSection(body) => {
                    parser::decode_type_section(body, &mut module).unwrap();
                }
                _ => {}
            }
        }
        let len = module.types.len();
        let exptect_param = [3, 0, 2];
        let exptect_result = [0, 2, 1];
        for i in 0..len {
            let params = &module.types[i].params;
            let results = &module.types[i].results;
            assert_eq!(params.len(), exptect_param[i]);
            assert_eq!(results.len(), exptect_result[i]);
        }
        assert_eq!(len, 3);
    }

    #[test]
    fn decode_func_section() {
        let wat = r#"
        (module
            (import "test" "test" (func (param i32))) 
            (import "test" "test" (func (param i32 i32) (result i32))) 
            (func (param i32 i32 i64))
            (func (result i32 i32)) 
            (func (param i32 i32) (result i32))
        )"#;
        /*
        Type Idx
        0: (param i32)
        1: (param i32 i32) (result i32)
        2: (param i32 i32 i64)
        3: (result i32 i32)
        */

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary) {
            match payload.unwrap() {
                FunctionSection(body) => {
                    parser::decode_func_section(body, &mut module).unwrap();
                }
                _ => {}
            }
        }
        let funcs_num = module.funcs.len();
        assert_eq!(funcs_num, 3);

        let exptect_idx = [2, 3, 1];
        for i in 0..funcs_num {
            let idx = &module.funcs[i].type_;
            assert_eq!(idx.0, exptect_idx[i]);
        }
    }

    #[test]
    fn decode_import_section() {
        let wat = r#"
        (module
            (import "module1" "func1" (func (param i32))) 
            (import "module2" "func1" (func (param i32 i32) (result i32)))
            (import "module2" "func2" (func (param i32 i64) (result f32))) 
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary) {
            match payload.unwrap() {
                ImportSection(body) => {
                    parser::decode_import_section(body, &mut module).unwrap();
                }
                _ => {}
            }
        }
        let imports_num = module.imports.len();
        assert_eq!(imports_num, 3);

        let module_names = ["module1", "module2", "module2"];
        let names = ["func1", "func1", "func2"];
        for i in 0..imports_num {
            let module_name = &module.imports[i].module.0;
            let name = &module.imports[i].name.0;
            let desc = &module.imports[i].desc;
            assert_eq!(module_name, module_names[i]);
            assert_eq!(name, names[i]);
            assert!(matches!(desc, ImportDesc::Func(TypeIdx(i))));
        }
    }

    #[test]
    fn decode_export_section() {
        let wat = r#"
        (module
            (memory (export "memory") 2 3)
            (func $add (export "add") (param $a i32) (param $b i32) (result i32)
                (i32.add (local.get $a) (local.get $b))
            )
            (func $sub (export "sub") (param $a i32) (param $b i32) (result i32)
                (i32.sub (local.get $a) (local.get $b))
            )
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary) {
            match payload.unwrap() {
                ExportSection(body) => {
                    parser::decode_export_section(body, &mut module).unwrap();
                }
                _ => {}
            }
        }
        let exports_num = module.exports.len();
        assert_eq!(exports_num, 3);

        let names = ["memory", "add", "sub"];
        for i in 0..exports_num {
            let name = &module.exports[i].name.0;
            let desc = &module.exports[i].desc;
            assert_eq!(name, names[i]);
            let expect = if i == 0 {
                ExportDesc::Mem(MemIdx(i as u32))
            } else {
                ExportDesc::Func(FuncIdx((i - 1) as u32))
            };
            assert!(matches!(desc, expect));
        }
    }

    #[test]
    fn decode_mem_section() {
        let wat = r#"
        (module
            (memory (export "memory") 2 3)
            (memory (export "mem") 1)

        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary) {
            match payload.unwrap() {
                MemorySection(body) => {
                    parser::decode_mem_section(body, &mut module).unwrap();
                }
                _ => {}
            }
        }
        let memory_num = module.mems.len();
        assert_eq!(memory_num, 2);

        let expects_min = [2, 1];
        for i in 0..memory_num {
            let limits = &module.mems[i].type_.0;
            let min = limits.min;
            let max = limits.max;
            assert_eq!(min, expects_min[i]);
            if i == 0 {
                assert_eq!(max, Some(3));
            } else {
                assert_eq!(max, None);
            }
        }
    }

    #[test]
    fn decode_table_section() {
        let wat = r#"
        (module
            (table 2 externref)
            (table 3 funcref)

        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary) {
            match payload.unwrap() {
                TableSection(body) => {
                    parser::decode_table_section(body, &mut module).unwrap();
                }
                _ => {}
            }
        }
        let table_num = module.tables.len();
        assert_eq!(table_num, 2);

        let mut limits = &module.tables[0].type_.0;
        let mut reftype = &module.tables[0].type_.1;
        let mut min = limits.min;
        let mut max = limits.max;
        assert_eq!(min, 2);
        assert_eq!(max, None);
        assert!(matches!(reftype, RefType::ExternalRef));

        limits = &module.tables[1].type_.0;
        reftype = &module.tables[1].type_.1;
        min = limits.min;
        max = limits.max;
        assert_eq!(min, 3);
        assert_eq!(max, None);
        assert!(matches!(reftype, RefType::FuncRef));
    }

    #[test]
    fn decode_global_section() {
        let wat = r#"
        (module
            (global $f32 f32)
            (global $f64 (mut i64)(i64.const 2024))
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary) {
            match payload.unwrap() {
                GlobalSection(body) => {
                    parser::decode_global_section(body, &mut module).unwrap();
                }
                _ => {}
            }
        }
        let global_num = module.globals.len();
        assert_eq!(global_num, 2);

        let mut type_ = &module.globals[0].type_;
        let mut init = &module.globals[0].init;
        assert!(matches!(type_.0, Mut::Const));
        assert!(matches!(type_.1, ValueType::NumType(NumType::F32)));
        assert_eq!(init.0.len(), 0);

        type_ = &module.globals[1].type_;
        init = &module.globals[1].init;
        assert!(matches!(type_.0, Mut::Var));
        assert!(matches!(type_.1, ValueType::NumType(NumType::I64)));
        assert_eq!(init.0.len(), 1);
        assert!(matches!(init.0[0], Instr::I64Const(2024)));
    }

    #[test]
    fn decode_elem_section() {
        let wat = r#"
        (module
            (table $t0 (export "table1") 2 funcref)
       
            (func $f1 (result i32)
                i32.const 1
            )
            (func $f2 (result i32)
                i32.const 2
            )
            (func $f3 (result i32)
                i32.const 3
            )
            (elem $t0(i32.const 0) $f1 $f2)
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary) {
            match payload.unwrap() {
                TableSection(body) => {
                    parser::decode_table_section(body, &mut module).unwrap();
                }
                ElementSection(body) => {
                    parser::decode_elem_section(body, &mut module).unwrap();
                }
                _ => {}
            }
        }
        let elem_num = module.elems.len();
        assert_eq!(elem_num, 1);

        let mut elem = &module.elems[0];
        assert!(matches!(elem.type_, RefType::FuncRef));
        assert!(matches!(elem.mode, ElemMode::Active));
        assert_eq!(elem.table_idx, Some(TableIdx(0)));
    }

    #[test]
    fn decode_data_section() {
        //Test Code: https://github.com/eliben/wasm-wat-samples
        let wat = r#"
        (module
            (memory (export "memory") 1 100)
                (data (i32.const 0x0000)
                    "\67\68\69\70\AA\FF\DF\CB"
                    "\12\A1\32\B3\A5\1F\01\02"
                )
                (data (i32.const 0x020)
                    "\01\03\05\07\09\0B\0D\0F"
                )
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");

        for payload in parser.parse_all(&binary) {
            match payload.unwrap() {
                MemorySection(body) => {
                    parser::decode_mem_section(body, &mut module).unwrap();
                }
                DataSection(body) => {
                    parser::decode_data_section(body, &mut module).unwrap();
                }
                _ => {}
            }
        }
        let data_num = module.datas.len();
        assert_eq!(data_num, 2);

        let mut init = &module.datas[0].init;
        let mut mode = &module.datas[0].mode;
        let mut memory = &module.datas[0].memory;
        let mut offset = &module.datas[0].offset;
        assert_eq!(init[0].0, 0x67);
        assert_eq!(init[7].0, 0xCB);
        assert!(matches!(mode, DataMode::Active));
        assert!(matches!(memory, Some(MemIdx(0))));
        let expected = Expr(vec![Instr::I32Const(0)]);
        assert_eq!(*offset, Some(expected));

        init = &module.datas[1].init;
        mode = &module.datas[1].mode;
        memory = &module.datas[1].memory;
        offset = &module.datas[1].offset;
        assert_eq!(init[0].0, 0x01);
        assert_eq!(init[7].0, 0x0F);
        assert!(matches!(mode, DataMode::Active));
        assert!(matches!(memory, Some(MemIdx(0))));
        let expected = Expr(vec![Instr::I32Const(0x020)]);
        assert_eq!(*offset, Some(expected));
    }

    #[test]
    fn decode_code_section_if_else() {
        //Test Code: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Control_flow/if...else
        let wat = r#"
        (module
            (func $ifexpr (result i32)
                i32.const 0
                (if (result i32)
                (then
                    ;; do something
                    (i32.const 1)
                )
                (else
                    ;; do something else
                    (i32.const 2)
                )
                )
            )
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");
        let mut current_func_index = module.num_imported_funcs;
        for payload in parser.parse_all(&binary) {
            match payload.unwrap() {
                FunctionSection(body) => {
                    parser::decode_func_section(body, &mut module).unwrap();
                }
                CodeSectionEntry(body) => {
                    parser::decode_code_section(body, &mut module, current_func_index).unwrap();
                    current_func_index += 1;
                }
                _ => {}
            }
        }
        let func_num = module.funcs.len();
        assert_eq!(func_num, 1);
    }

    #[test]
    fn decode_code_section_loop() {
        //Test Code: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Control_flow/loop
        let wat = r#"
       (module
            (func
                (local $i i32)
                (loop $my_loop
                    ;; add one to $i
                    local.get $i
                    i32.const 1
                    i32.add
                    local.set $i
                    ;; if $i is less than 10 branch to loop
                    local.get $i
                    i32.const 10
                    i32.lt_s
                    br_if $my_loop
                )
            )
       )"#;

        let binary = wat::parse_str(wat).unwrap();
        let parser = wasmparser::Parser::new(0);
        let mut module = Module::new("test");
        let mut current_func_index = module.num_imported_funcs;

        for payload in parser.parse_all(&binary) {
            match payload.unwrap() {
                FunctionSection(body) => {
                    parser::decode_func_section(body, &mut module).unwrap();
                }
                CodeSectionEntry(body) => {
                    parser::decode_code_section(body, &mut module, current_func_index).unwrap();
                    current_func_index += 1;
                }
                _ => {}
            }
        }
        let func_num = module.funcs.len();
        assert_eq!(func_num, 1);
    }
}
