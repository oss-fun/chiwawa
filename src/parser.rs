use std::fs::File;
use std::io::Read;
use wasmparser::{
    ExternalKind, FunctionBody, Parser, Payload::*, SectionLimited, TypeRef, ValType,
};

use crate::error::{ParserError, RuntimeError};
use crate::execution::stack::ProcessedInstr;
use crate::execution::stack::*;
use crate::structure::{instructions::*, module::*, types::*};
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::LazyLock;

// Operation type classification
#[derive(Debug, Clone, Copy, PartialEq)]
enum OperationType {
    Producer,    // 0 args, produces value (const, local.get, global.get)
    Unary,       // 1 arg, produces value (clz, ctz, popcnt, etc.)
    Binary,      // 2 args, produces value (add, sub, mul, etc.)
    MemoryLoad,  // 1 arg (address), produces value
    MemoryStore, // 2 args (address, value), no value produced
    ControlFlow, // Variable args, special handling
    Other,       // Non-optimizable
}

fn get_operation_type(op: &wasmparser::Operator) -> OperationType {
    use wasmparser::Operator;
    match op {
        // Producers
        Operator::I32Const { .. }
        | Operator::I64Const { .. }
        | Operator::F32Const { .. }
        | Operator::F64Const { .. }
        | Operator::LocalGet { .. }
        | Operator::GlobalGet { .. } => OperationType::Producer,
        // Unary operations
        Operator::I32Clz | Operator::I32Ctz | Operator::I32Popcnt |
        Operator::I64Clz | Operator::I64Ctz | Operator::I64Popcnt |
        Operator::F32Abs | Operator::F32Neg | Operator::F32Ceil |
        Operator::F32Floor | Operator::F32Trunc | Operator::F32Nearest |
        Operator::F32Sqrt | Operator::F64Abs | Operator::F64Neg |
        Operator::F64Ceil | Operator::F64Floor | Operator::F64Trunc |
        Operator::F64Nearest | Operator::F64Sqrt |
        Operator::I32Eqz | Operator::I64Eqz |
        // Type conversions (all are unary operations)
        Operator::I32WrapI64 |
        Operator::I64ExtendI32S | Operator::I64ExtendI32U |
        Operator::I32TruncF32S | Operator::I32TruncF32U |
        Operator::I32TruncF64S | Operator::I32TruncF64U |
        Operator::I64TruncF32S | Operator::I64TruncF32U |
        Operator::I64TruncF64S | Operator::I64TruncF64U |
        Operator::I32TruncSatF32S | Operator::I32TruncSatF32U |
        Operator::I32TruncSatF64S | Operator::I32TruncSatF64U |
        Operator::I64TruncSatF32S | Operator::I64TruncSatF32U |
        Operator::I64TruncSatF64S | Operator::I64TruncSatF64U |
        Operator::F32DemoteF64 | Operator::F64PromoteF32 |
        Operator::F32ConvertI32S | Operator::F32ConvertI32U |
        Operator::F32ConvertI64S | Operator::F32ConvertI64U |
        Operator::F64ConvertI32S | Operator::F64ConvertI32U |
        Operator::F64ConvertI64S | Operator::F64ConvertI64U |
        Operator::I32ReinterpretF32 | Operator::I64ReinterpretF64 |
        Operator::F32ReinterpretI32 | Operator::F64ReinterpretI64 |
        // Extend operations
        Operator::I32Extend8S | Operator::I32Extend16S |
        Operator::I64Extend8S | Operator::I64Extend16S | Operator::I64Extend32S => OperationType::Unary,

        // Memory loads
        Operator::I32Load { .. }
        | Operator::I64Load { .. }
        | Operator::F32Load { .. }
        | Operator::F64Load { .. }
        | Operator::I32Load8S { .. }
        | Operator::I32Load8U { .. }
        | Operator::I32Load16S { .. }
        | Operator::I32Load16U { .. }
        | Operator::I64Load8S { .. }
        | Operator::I64Load8U { .. }
        | Operator::I64Load16S { .. }
        | Operator::I64Load16U { .. }
        | Operator::I64Load32S { .. }
        | Operator::I64Load32U { .. } => OperationType::MemoryLoad,

        // Memory stores
        Operator::I32Store { .. }
        | Operator::I64Store { .. }
        | Operator::F32Store { .. }
        | Operator::F64Store { .. }
        | Operator::I32Store8 { .. }
        | Operator::I32Store16 { .. }
        | Operator::I64Store8 { .. }
        | Operator::I64Store16 { .. }
        | Operator::I64Store32 { .. } => OperationType::MemoryStore,

        // Binary operations
        Operator::I32Add
        | Operator::I32Sub
        | Operator::I32Mul
        | Operator::I32DivS
        | Operator::I32DivU
        | Operator::I32RemS
        | Operator::I32RemU
        | Operator::I32And
        | Operator::I32Or
        | Operator::I32Xor
        | Operator::I32Shl
        | Operator::I32ShrS
        | Operator::I32ShrU
        | Operator::I32Rotl
        | Operator::I32Rotr
        | Operator::I64Add
        | Operator::I64Sub
        | Operator::I64Mul
        | Operator::I64DivS
        | Operator::I64DivU
        | Operator::I64RemS
        | Operator::I64RemU
        | Operator::I64And
        | Operator::I64Or
        | Operator::I64Xor
        | Operator::I64Shl
        | Operator::I64ShrS
        | Operator::I64ShrU
        | Operator::I64Rotl
        | Operator::I64Rotr
        | Operator::F32Add
        | Operator::F32Sub
        | Operator::F32Mul
        | Operator::F32Div
        | Operator::F32Min
        | Operator::F32Max
        | Operator::F32Copysign
        | Operator::F64Add
        | Operator::F64Sub
        | Operator::F64Mul
        | Operator::F64Div
        | Operator::F64Min
        | Operator::F64Max
        | Operator::F64Copysign
        | Operator::I32Eq
        | Operator::I32Ne
        | Operator::I32LtS
        | Operator::I32LtU
        | Operator::I32GtS
        | Operator::I32GtU
        | Operator::I32LeS
        | Operator::I32LeU
        | Operator::I32GeS
        | Operator::I32GeU
        | Operator::I64Eq
        | Operator::I64Ne
        | Operator::I64LtS
        | Operator::I64LtU
        | Operator::I64GtS
        | Operator::I64GtU
        | Operator::I64LeS
        | Operator::I64LeU
        | Operator::I64GeS
        | Operator::I64GeU
        | Operator::F32Eq
        | Operator::F32Ne
        | Operator::F32Lt
        | Operator::F32Gt
        | Operator::F32Le
        | Operator::F32Ge
        | Operator::F64Eq
        | Operator::F64Ne
        | Operator::F64Lt
        | Operator::F64Gt
        | Operator::F64Le
        | Operator::F64Ge => OperationType::Binary,

        // Control flow
        Operator::Block { .. }
        | Operator::Loop { .. }
        | Operator::If { .. }
        | Operator::Else
        | Operator::End
        | Operator::Br { .. }
        | Operator::BrIf { .. }
        | Operator::BrTable { .. }
        | Operator::Return
        | Operator::Call { .. }
        | Operator::CallIndirect { .. } => OperationType::ControlFlow,

        // Other
        _ => OperationType::Other,
    }
}

// Try to apply optimization for an operator based on recent instructions
fn try_apply_optimization<'a>(
    op: &wasmparser::Operator,
    recent_instrs: &mut VecDeque<(wasmparser::Operator, usize)>,
    mut processed_instr: &mut ProcessedInstr,
    initial_processed_instrs: &mut Vec<ProcessedInstr>,
    current_processed_pc: &mut usize,
    ops: &mut itertools::structs::MultiPeek<
        impl Iterator<
                Item = Result<(wasmparser::Operator<'a>, usize), wasmparser::BinaryReaderError>,
            > + 'a,
    >,
) -> bool {
    let op_type = get_operation_type(op);

    // Check if we have enough recent instructions
    match op_type {
        OperationType::Binary | OperationType::MemoryStore => {
            if recent_instrs.len() < 2 {
                return false;
            }
        }
        OperationType::MemoryLoad | OperationType::Unary => {
            if recent_instrs.is_empty() {
                return false;
            }
        }
        _ => return false,
    }

    let mut iter = recent_instrs.iter().rev();

    // Apply optimization based on operation type
    match op_type {
        OperationType::Binary => {
            let second_source = iter
                .next()
                .and_then(|(instr, _)| operator_to_value_source(instr));
            let first_source = iter
                .next()
                .and_then(|(instr, _)| operator_to_value_source(instr));

            if let (Some(first), Some(second)) = (first_source, second_source) {
                // Check if next instruction is local.set or global.set
                let mut store_target = None;
                let mut skip_next = false;

                if let Some(Ok((next_op, _))) = ops.peek() {
                    match next_op {
                        wasmparser::Operator::LocalSet { local_index } => {
                            store_target = Some(StoreTarget::Local(*local_index));
                            skip_next = true;
                        }
                        wasmparser::Operator::GlobalSet { global_index } => {
                            store_target = Some(StoreTarget::Global(*global_index));
                            skip_next = true;
                        }
                        _ => {}
                    }
                }

                if let ProcessedInstr::Legacy { operand, .. } = &mut processed_instr {
                    *operand = Operand::Optimized(OptimizedOperand::Double {
                        first: Some(first),
                        second: Some(second),
                        memarg: None,
                        store_target,
                    });
                }

                // Remove consumed instructions
                for _ in 0..2 {
                    initial_processed_instrs.pop();
                    recent_instrs.pop_back();
                    *current_processed_pc -= 1;
                }

                // If we have a store target, skip the next instruction
                if skip_next {
                    ops.next(); // Skip the set instruction
                }

                return true;
            }
        }
        OperationType::MemoryLoad => {
            if let Some(addr_source) = iter
                .next()
                .and_then(|(instr, _)| operator_to_value_source(instr))
            {
                if let ProcessedInstr::Legacy { operand, .. } = &mut processed_instr {
                    if let Operand::MemArg(memarg) = operand {
                        *operand = Operand::Optimized(OptimizedOperand::Single {
                            value: Some(addr_source),
                            memarg: Some(memarg.clone()),
                            store_target: None,
                        });

                        // Remove consumed instruction
                        initial_processed_instrs.pop();
                        recent_instrs.pop_back();
                        *current_processed_pc -= 1;
                        return true;
                    }
                }
            }
        }
        OperationType::MemoryStore => {
            let value_source = iter
                .next()
                .and_then(|(instr, _)| operator_to_value_source(instr));
            let addr_source = iter
                .next()
                .and_then(|(instr, _)| operator_to_value_source(instr));

            if let (Some(addr), Some(value)) = (addr_source, value_source) {
                if let ProcessedInstr::Legacy { operand, .. } = &mut processed_instr {
                    if let Operand::MemArg(memarg) = operand {
                        *operand = Operand::Optimized(OptimizedOperand::Double {
                            first: Some(addr),
                            second: Some(value),
                            memarg: Some(memarg.clone()),
                            store_target: None,
                        });

                        // Remove consumed instructions
                        for _ in 0..2 {
                            initial_processed_instrs.pop();
                            recent_instrs.pop_back();
                            *current_processed_pc -= 1;
                        }
                        return true;
                    }
                }
            }
        }
        OperationType::Unary => {
            // Get one value directly (same as memory load)
            if let Some(value_source) = iter
                .next()
                .and_then(|(instr, _)| operator_to_value_source(instr))
            {
                // Check if next instruction is local.set or global.set
                let mut store_target = None;
                let mut skip_next = false;

                if let Some(Ok((next_op, _))) = ops.peek() {
                    match next_op {
                        wasmparser::Operator::LocalSet { local_index } => {
                            store_target = Some(StoreTarget::Local(*local_index));
                            skip_next = true;
                        }
                        wasmparser::Operator::GlobalSet { global_index } => {
                            store_target = Some(StoreTarget::Global(*global_index));
                            skip_next = true;
                        }
                        _ => {}
                    }
                }

                if let ProcessedInstr::Legacy { operand, .. } = &mut processed_instr {
                    *operand = Operand::Optimized(OptimizedOperand::Single {
                        value: Some(value_source),
                        memarg: None,
                        store_target,
                    });
                }

                // Remove consumed instruction
                initial_processed_instrs.pop();
                recent_instrs.pop_back();
                *current_processed_pc -= 1;

                if skip_next {
                    ops.next(); // Skip the set instruction
                }

                return true;
            }
        }
        _ => {}
    }

    false
}

// Helper function to create ValueSource from an operator
fn operator_to_value_source(op: &wasmparser::Operator) -> Option<ValueSource> {
    match op {
        wasmparser::Operator::LocalGet { local_index } => Some(ValueSource::Local(*local_index)),
        wasmparser::Operator::I32Const { value } => Some(ValueSource::Const(Value::I32(*value))),
        wasmparser::Operator::I64Const { value } => Some(ValueSource::Const(Value::I64(*value))),
        wasmparser::Operator::F32Const { value } => {
            Some(ValueSource::Const(Value::F32(f32::from_bits(value.bits()))))
        }
        wasmparser::Operator::F64Const { value } => {
            Some(ValueSource::Const(Value::F64(f64::from_bits(value.bits()))))
        }
        wasmparser::Operator::GlobalGet { global_index } => {
            Some(ValueSource::Global(*global_index))
        }
        _ => None,
    }
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
    block_parameter_count_cache: FxHashMap<BlockTypeKey, usize>,
}

impl BlockArityCache {
    fn new() -> Self {
        Self {
            block_arity_cache: FxHashMap::default(),
            loop_parameter_arity_cache: FxHashMap::default(),
            block_parameter_count_cache: FxHashMap::default(),
        }
    }
}

struct ConservativePurityChecker {
    safe_arithmetic_ops: FxHashSet<usize>,
    safe_const_ops: FxHashSet<usize>,
    safe_comparison_ops: FxHashSet<usize>,
    safe_stack_ops: FxHashSet<usize>,
    safe_local_read_ops: FxHashSet<usize>,
    safe_control_ops: FxHashSet<usize>,
}

impl ConservativePurityChecker {
    fn new() -> Self {
        let mut safe_arithmetic_ops = FxHashSet::default();
        safe_arithmetic_ops.insert(HANDLER_IDX_I32_ADD);
        safe_arithmetic_ops.insert(HANDLER_IDX_I32_SUB);
        safe_arithmetic_ops.insert(HANDLER_IDX_I32_MUL);
        safe_arithmetic_ops.insert(HANDLER_IDX_I64_ADD);
        safe_arithmetic_ops.insert(HANDLER_IDX_I64_SUB);
        safe_arithmetic_ops.insert(HANDLER_IDX_I64_MUL);
        safe_arithmetic_ops.insert(HANDLER_IDX_F32_ADD);
        safe_arithmetic_ops.insert(HANDLER_IDX_F32_SUB);
        safe_arithmetic_ops.insert(HANDLER_IDX_F32_MUL);
        safe_arithmetic_ops.insert(HANDLER_IDX_F64_ADD);
        safe_arithmetic_ops.insert(HANDLER_IDX_F64_SUB);
        safe_arithmetic_ops.insert(HANDLER_IDX_F64_MUL);

        let mut safe_const_ops = FxHashSet::default();
        safe_const_ops.insert(HANDLER_IDX_I32_CONST);
        safe_const_ops.insert(HANDLER_IDX_I64_CONST);
        safe_const_ops.insert(HANDLER_IDX_F32_CONST);
        safe_const_ops.insert(HANDLER_IDX_F64_CONST);

        let mut safe_comparison_ops = FxHashSet::default();
        safe_comparison_ops.insert(HANDLER_IDX_I32_EQ);
        safe_comparison_ops.insert(HANDLER_IDX_I32_NE);
        safe_comparison_ops.insert(HANDLER_IDX_I32_LT_S);
        safe_comparison_ops.insert(HANDLER_IDX_I32_LT_U);
        safe_comparison_ops.insert(HANDLER_IDX_I32_GT_S);
        safe_comparison_ops.insert(HANDLER_IDX_I32_GT_U);
        safe_comparison_ops.insert(HANDLER_IDX_I32_LE_S);
        safe_comparison_ops.insert(HANDLER_IDX_I32_LE_U);
        safe_comparison_ops.insert(HANDLER_IDX_I32_GE_S);
        safe_comparison_ops.insert(HANDLER_IDX_I32_GE_U);
        safe_comparison_ops.insert(HANDLER_IDX_I64_EQ);
        safe_comparison_ops.insert(HANDLER_IDX_I64_NE);
        safe_comparison_ops.insert(HANDLER_IDX_I64_LT_S);
        safe_comparison_ops.insert(HANDLER_IDX_I64_LT_U);
        safe_comparison_ops.insert(HANDLER_IDX_I64_GT_S);
        safe_comparison_ops.insert(HANDLER_IDX_I64_GT_U);
        safe_comparison_ops.insert(HANDLER_IDX_I64_LE_S);
        safe_comparison_ops.insert(HANDLER_IDX_I64_LE_U);
        safe_comparison_ops.insert(HANDLER_IDX_I64_GE_S);
        safe_comparison_ops.insert(HANDLER_IDX_I64_GE_U);
        safe_comparison_ops.insert(HANDLER_IDX_F32_EQ);
        safe_comparison_ops.insert(HANDLER_IDX_F32_NE);
        safe_comparison_ops.insert(HANDLER_IDX_F32_LT);
        safe_comparison_ops.insert(HANDLER_IDX_F32_GT);
        safe_comparison_ops.insert(HANDLER_IDX_F32_LE);
        safe_comparison_ops.insert(HANDLER_IDX_F32_GE);
        safe_comparison_ops.insert(HANDLER_IDX_F64_EQ);
        safe_comparison_ops.insert(HANDLER_IDX_F64_NE);
        safe_comparison_ops.insert(HANDLER_IDX_F64_LT);
        safe_comparison_ops.insert(HANDLER_IDX_F64_GT);
        safe_comparison_ops.insert(HANDLER_IDX_F64_LE);
        safe_comparison_ops.insert(HANDLER_IDX_F64_GE);

        let mut safe_stack_ops = FxHashSet::default();
        safe_stack_ops.insert(HANDLER_IDX_DROP);
        safe_stack_ops.insert(HANDLER_IDX_SELECT);

        let mut safe_local_read_ops = FxHashSet::default();
        safe_local_read_ops.insert(HANDLER_IDX_LOCAL_GET);

        let mut safe_control_ops = FxHashSet::default();
        safe_control_ops.insert(HANDLER_IDX_BLOCK);
        safe_control_ops.insert(HANDLER_IDX_LOOP);
        safe_control_ops.insert(HANDLER_IDX_IF);
        safe_control_ops.insert(HANDLER_IDX_ELSE);
        safe_control_ops.insert(HANDLER_IDX_END);

        Self {
            safe_arithmetic_ops,
            safe_const_ops,
            safe_comparison_ops,
            safe_stack_ops,
            safe_local_read_ops,
            safe_control_ops,
        }
    }

    fn is_instruction_safe(&self, handler_idx: usize) -> bool {
        self.safe_arithmetic_ops.contains(&handler_idx)
            || self.safe_const_ops.contains(&handler_idx)
            || self.safe_comparison_ops.contains(&handler_idx)
            || self.safe_stack_ops.contains(&handler_idx)
            || self.safe_local_read_ops.contains(&handler_idx)
            || self.safe_control_ops.contains(&handler_idx)
    }

    fn is_block_memoizable(&self, instructions: &[ProcessedInstr]) -> bool {
        for instr in instructions {
            if !self.is_instruction_safe(instr.handler_index()) {
                return false;
            }
        }
        true
    }

    /// Analyzes a function to identify memoizable blocks
    ///
    /// This function performs static analysis to find top-level blocks that are safe
    /// for memoization (i.e., pure functions without side effects).
    ///
    /// # Algorithm:
    /// 1. Tracks block nesting depth using a counter
    /// 2. Records start positions of top-level blocks (depth 0)
    /// 3. When a top-level block ends, checks if entire block is pure
    /// 4. Only includes blocks that contain exclusively safe operations
    ///
    /// # Conservative Approach:
    /// - Only analyzes top-level blocks (ignores nested blocks)
    /// - Any unsafe instruction disqualifies the entire block
    /// - Nested blocks are considered part of their parent block
    ///
    /// # Example:
    /// ```wasm
    /// block (result i32)     ; <- Position 5: memoizable (pure arithmetic)
    ///   i32.const 10
    ///   i32.const 20
    ///   i32.add
    /// end
    ///
    /// block (result i32)     ; <- Position 12: not memoizable (has call)
    ///   i32.const 5
    ///   call $function       ; <- Unsafe operation
    /// end
    /// ```
    ///
    /// # Returns:
    /// HashSet containing start positions of memoizable blocks (e.g., {5} for above example)
    fn analyze_function(&self, instructions: &[ProcessedInstr]) -> FxHashSet<usize> {
        let mut memoizable_blocks = FxHashSet::default();
        let mut current_block_start = 0;
        let mut block_depth = 0; // Track nesting level

        for (i, instr) in instructions.iter().enumerate() {
            match instr.handler_index() {
                HANDLER_IDX_BLOCK | HANDLER_IDX_LOOP | HANDLER_IDX_IF => {
                    // Record start position only for top-level blocks
                    if block_depth == 0 {
                        current_block_start = i;
                    }
                    block_depth += 1; // Enter nested block
                }
                HANDLER_IDX_END => {
                    if block_depth > 0 {
                        block_depth -= 1; // Exit nested block

                        // Analyze block only when returning to top level
                        if block_depth == 0 {
                            // Extract all instructions in this top-level block
                            let block_instructions = &instructions[current_block_start..=i];

                            if self.is_block_memoizable(block_instructions) {
                                memoizable_blocks.insert(current_block_start);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        memoizable_blocks
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

fn calculate_block_parameter_count(
    block_type: &wasmparser::BlockType,
    module: &Module,
    cache: &mut BlockArityCache,
) -> usize {
    let key = BlockTypeKey::from(block_type);

    if let Some(&count) = cache.block_parameter_count_cache.get(&key) {
        return count;
    }

    let count = match block_type {
        wasmparser::BlockType::Empty => 0,
        wasmparser::BlockType::Type(_) => 0, // Single type means no parameters for block
        wasmparser::BlockType::FuncType(type_idx) => {
            if let Some(func_type) = module.types.get(*type_idx as usize) {
                func_type.params.len()
            } else {
                0 // Fallback to 0 if invalid type index
            }
        }
    };

    cache.block_parameter_count_cache.insert(key, count);
    count
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
    execution_mode: &str,
    cache: &mut BlockArityCache,
    purity_checker: &ConservativePurityChecker,
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
    ) = decode_processed_instrs_and_fixups(
        ops_iter,
        module,
        enable_superinstructions,
        execution_mode,
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
        module,
        cache,
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    let body_rc = Rc::new(processed_instrs);

    // Store function body and metadata in module
    if let Some(func) = module.funcs.get_mut(relative_func_index) {
        // Analyze which blocks are pure (side-effect free) for memoization optimization
        let memoizable_blocks = purity_checker.analyze_function(&body_rc);

        func.body = body_rc;
        // Store slot mode metadata (None for stack mode)
        func.slot_allocation = slot_allocation.clone();
        func.result_slot = result_slot;

        // Ensure memoizable_blocks array has enough capacity
        while module.memoizable_blocks.len() <= relative_func_index {
            module.memoizable_blocks.push(FxHashSet::default());
        }

        // Store memoizable block information for this function
        if let Some(blocks) = module.memoizable_blocks.get_mut(relative_func_index) {
            *blocks = memoizable_blocks;
        }
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
    block_end_map: &FxHashMap<usize, usize>,
    if_else_map: &FxHashMap<usize, usize>,
    block_type_map: &FxHashMap<usize, wasmparser::BlockType>,
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
            if let Some(instr_to_patch) = processed.get_mut(current_fixup_pc) {
                // Skip fixup for slot-based instructions
                if !matches!(instr_to_patch, ProcessedInstr::Legacy { .. }) {
                    fixups[fixup_index].original_wasm_depth = usize::MAX;
                    continue;
                }
                if is_if_false_jump {
                    fixups[fixup_index].original_wasm_depth = usize::MAX;
                    continue;
                } else if is_else_jump {
                    fixups[fixup_index].original_wasm_depth = usize::MAX;
                    continue;
                } else {
                    // Skip fixup for slot-based instructions
                    if !matches!(instr_to_patch, ProcessedInstr::Legacy { .. }) {
                        continue;
                    }
                    instr_to_patch.set_handler_index(HANDLER_IDX_RETURN);
                    *instr_to_patch.operand_mut() = Operand::None;
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

        let (target_start_pc, is_loop, target_block_type) =
            current_control_stack_pass2[target_stack_level];

        // Calculate target IP
        let target_ip = if is_loop {
            target_start_pc
        } else {
            *block_end_map
                .get(&target_start_pc)
                .ok_or_else(|| RuntimeError::InvalidWasm("Missing EndMarker for branch target"))?
        };

        let target_arity = if is_loop {
            calculate_loop_parameter_arity(&target_block_type, module, cache)
        } else {
            calculate_block_arity(&target_block_type, module, cache)
        };

        // Patch the instruction operand
        if let Some(instr_to_patch) = processed.get_mut(current_fixup_pc) {
            // Skip fixup for slot-based instructions
            if !matches!(instr_to_patch, ProcessedInstr::Legacy { .. }) {
                fixups[fixup_index].original_wasm_depth = usize::MAX;
                continue;
            }
            if is_if_false_jump {
                // If instruction's jump-on-false
                // Target is ElseMarker+1 or EndMarker+1
                let else_target = *if_else_map.get(&target_start_pc).unwrap_or(&target_ip);
                // For if statements, get the block type of the current if instruction
                let if_block_type = block_type_map
                    .get(&current_fixup_pc)
                    .cloned()
                    .unwrap_or(wasmparser::BlockType::Empty);
                let if_arity = calculate_block_arity(&if_block_type, module, cache);

                *instr_to_patch.operand_mut() = Operand::LabelIdx {
                    target_ip: else_target,
                    arity: if_arity,
                    original_wasm_depth: current_fixup_depth,
                    is_loop: false,
                };
            } else if is_else_jump {
                // Else instruction's jump-to-end
                let else_block_type = block_type_map
                    .get(&current_fixup_pc)
                    .cloned()
                    .unwrap_or(wasmparser::BlockType::Empty);
                let else_arity = calculate_block_arity(&else_block_type, module, cache);
                *instr_to_patch.operand_mut() = Operand::LabelIdx {
                    target_ip: target_ip,
                    arity: else_arity,
                    original_wasm_depth: current_fixup_depth,
                    is_loop: false,
                };
            } else {
                // Br or BrIf instruction
                *instr_to_patch.operand_mut() = Operand::LabelIdx {
                    target_ip,
                    arity: target_arity,
                    original_wasm_depth: current_fixup_depth,
                    is_loop: is_loop,
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
            let needs_br_table_resolution = matches!(instr, ProcessedInstr::Legacy { .. })
                && instr.handler_index() == HANDLER_IDX_BR_TABLE
                && *instr.operand() == Operand::None;

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
                        // Skip fixup for slot-based instructions
                        if matches!(instr_to_patch, ProcessedInstr::Legacy { .. }) {
                            *instr_to_patch.operand_mut() = Operand::BrTable {
                                targets: vec![],
                                default: Box::new(Operand::None),
                            };
                        }
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

                    let (target_start_pc, is_loop, target_block_type) =
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
                    let target_arity = if is_loop {
                        // For loops: Branch provides parameters (input types)
                        calculate_loop_parameter_arity(&target_block_type, module, cache)
                    } else {
                        // For blocks: Branch provides results (output types)
                        calculate_block_arity(&target_block_type, module, cache)
                    };

                    fixups[default_fixup_idx].original_wasm_depth = usize::MAX;

                    Operand::LabelIdx {
                        target_ip,
                        arity: target_arity,
                        original_wasm_depth: fixup_depth,
                        is_loop: is_loop, // Use default target's loop/block information
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
                        let target_ip = if is_loop {
                            target_start_pc
                        } else {
                            let end_ip = *block_end_map.get(&target_start_pc).ok_or_else(|| {
                                RuntimeError::InvalidWasm("Missing EndMarker for BrTable target")
                            })?;
                            end_ip
                        };
                        let target_arity = if is_loop {
                            calculate_loop_parameter_arity(&target_block_type, module, cache)
                        } else {
                            calculate_block_arity(&target_block_type, module, cache)
                        };
                        // Branch to parent level: if target block is at index N, unwind to N-1

                        fixups[fixup_idx].original_wasm_depth = usize::MAX;

                        Operand::LabelIdx {
                            target_ip,
                            arity: target_arity,
                            original_wasm_depth: fixup_depth,
                            is_loop: is_loop,
                        }
                    };
                    resolved_targets.push(target_operand);
                }

                // --- Patch BrTable Instruction ---
                if let Some(instr_to_patch) = processed.get_mut(pc) {
                    // Skip fixup for slot-based instructions
                    if matches!(instr_to_patch, ProcessedInstr::Legacy { .. }) {
                        *instr_to_patch.operand_mut() = Operand::BrTable {
                            targets: resolved_targets,
                            default: Box::new(default_target_operand),
                        };
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

fn decode_processed_instrs_and_fixups<'a>(
    ops_iter: wasmparser::OperatorsIteratorWithOffsets<'a>,
    module: &Module,
    enable_superinstructions: bool,
    execution_mode: &str,
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
    ),
    Box<dyn std::error::Error>,
> {
    let mut ops = ops_iter.multipeek();
    let mut initial_processed_instrs = Vec::new();
    let mut initial_fixups = Vec::new();
    let mut current_processed_pc = 0;
    let mut control_info_stack: Vec<(wasmparser::BlockType, usize)> = Vec::new();

    let mut block_end_map: FxHashMap<usize, usize> = FxHashMap::default();
    let mut if_else_map: FxHashMap<usize, usize> = FxHashMap::default();
    let mut block_type_map: FxHashMap<usize, wasmparser::BlockType> = FxHashMap::default();
    let mut control_stack_for_map_building: Vec<(usize, bool, Option<usize>)> = Vec::new();
    // Track recent instructions for value source optimization
    let mut recent_instrs: VecDeque<(wasmparser::Operator, usize)> = VecDeque::new();
    const RECENT_INSTRS_WINDOW: usize = 3; // Track last 3 instructions

    // Initialize slot allocator for slot-based execution mode
    let mut slot_allocator = if execution_mode == "slot" {
        use crate::execution::slots::SlotAllocator;
        Some(SlotAllocator::new(locals))
    } else {
        None
    };

    // Track allocator state at block entry for proper restoration on block exit
    let mut allocator_state_stack: Vec<crate::execution::slots::SlotAllocatorState> = Vec::new();

    loop {
        if ops.peek().is_none() {
            break;
        }

        let (op, _offset) = match ops.next() {
            Some(Ok(op_offset)) => op_offset,
            Some(Err(e)) => return Err(Box::new(e)),
            None => break,
        };

        // Get the processed instruction based on execution mode
        let (mut processed_instr, fixup_info_opt) = if let Some(ref mut allocator) = slot_allocator
        {
            // Slot-based mode: convert i32 instructions to slot format
            use crate::execution::stack::{I32SlotOperand, ProcessedInstr};
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
                        _ => {
                            // For other types, fall back to Legacy mode
                            map_operator_to_initial_instr_and_fixup(
                                &op,
                                current_processed_pc,
                                &control_info_stack,
                                module,
                                &mut BlockArityCache::new(),
                            )?
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::I32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::I64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::I64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F32));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F32));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let src2 = allocator.pop(ValueType::NumType(NumType::F64));
                    let src1 = allocator.pop(ValueType::NumType(NumType::F64));
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
                    let result_arity = if let Some((blockty, _)) = control_info_stack.last() {
                        get_block_result_types(blockty, module).len()
                    } else {
                        result_types.len()
                    };

                    // Only sync the TOP N slots (where N is result arity), not all active slots
                    let all_slots = allocator.get_active_slots();
                    let slots_to_stack = if all_slots.len() >= result_arity {
                        all_slots[all_slots.len() - result_arity..].to_vec()
                    } else {
                        all_slots
                    };

                    // Restore allocator state to block entry, then push results
                    // This ensures result slots are allocated at the correct depth
                    let mut stack_to_slots = Vec::new();
                    if let Some(saved_state) = allocator_state_stack.pop() {
                        allocator.restore_state(&saved_state);
                        // Push result types to allocator (at the restored depth)
                        if let Some((blockty, _)) = control_info_stack.last() {
                            let result_types = get_block_result_types(blockty, module);
                            for vtype in result_types {
                                let slot = allocator.push(vtype);
                                stack_to_slots.push(slot);
                            }
                        }
                    }

                    let (mut instr, fixup) = map_operator_to_initial_instr_and_fixup(
                        &op,
                        current_processed_pc,
                        &control_info_stack,
                        module,
                        &mut BlockArityCache::new(),
                    )?;
                    // Set slots_to_stack and stack_to_slots on the legacy instruction
                    if let ProcessedInstr::Legacy {
                        slots_to_stack: ref mut ss,
                        stack_to_slots: ref mut rs,
                        ..
                    } = instr
                    {
                        *ss = slots_to_stack;
                        *rs = stack_to_slots;
                    }
                    (instr, fixup)
                }
                wasmparser::Operator::Block { blockty }
                | wasmparser::Operator::Loop { blockty } => {
                    let param_types = get_block_param_types(&blockty, module);

                    // Pop params from allocator to get state BEFORE params
                    for vtype in param_types.iter().rev() {
                        allocator.pop(vtype.clone());
                    }

                    // Save allocator state BEFORE params
                    allocator_state_stack.push(allocator.save_state());

                    // Push params back - they're still on the stack inside the block
                    for vtype in param_types.iter() {
                        allocator.push(vtype.clone());
                    }

                    map_operator_to_initial_instr_and_fixup(
                        &op,
                        current_processed_pc,
                        &control_info_stack,
                        module,
                        &mut BlockArityCache::new(),
                    )?
                }
                wasmparser::Operator::If { blockty } => {
                    let condition_slot = allocator.pop(ValueType::NumType(NumType::I32));

                    let param_types = get_block_param_types(&blockty, module);

                    for vtype in param_types.iter().rev() {
                        allocator.pop(vtype.clone());
                    }

                    let slots_to_stack = allocator.get_active_slots();

                    allocator_state_stack.push(allocator.save_state());

                    for vtype in param_types.iter() {
                        allocator.push(vtype.clone());
                    }

                    let (mut instr, fixup) = map_operator_to_initial_instr_and_fixup(
                        &op,
                        current_processed_pc,
                        &control_info_stack,
                        module,
                        &mut BlockArityCache::new(),
                    )?;

                    let mut full_slots_to_stack = slots_to_stack;
                    full_slots_to_stack.push(condition_slot);
                    if let ProcessedInstr::Legacy {
                        slots_to_stack: ss, ..
                    } = &mut instr
                    {
                        *ss = full_slots_to_stack;
                    }
                    (instr, fixup)
                }
                wasmparser::Operator::Else => {
                    if let Some(state) = allocator_state_stack.last() {
                        allocator.restore_state(state);
                    }

                    // Get the If's block type from control_info_stack and push params back
                    if let Some((blockty, _)) = control_info_stack.last() {
                        let param_types = get_block_param_types(blockty, module);
                        for vtype in param_types.iter() {
                            allocator.push(vtype.clone());
                        }
                    }

                    map_operator_to_initial_instr_and_fixup(
                        &op,
                        current_processed_pc,
                        &control_info_stack,
                        module,
                        &mut BlockArityCache::new(),
                    )?
                }

                wasmparser::Operator::Call { function_index } => {
                    let func_type_idx = if (*function_index as usize) < module.num_imported_funcs {
                        if let Some(import) = module.imports.get(*function_index as usize) {
                            match &import.desc {
                                crate::structure::module::ImportDesc::Func(type_idx) => {
                                    Some(type_idx.0 as usize)
                                }
                                _ => None,
                            }
                        } else {
                            None
                        }
                    } else {
                        let local_idx = *function_index as usize - module.num_imported_funcs;
                        if let Some(func) = module.funcs.get(local_idx) {
                            Some(func.type_.0 as usize)
                        } else {
                            None
                        }
                    };

                    let (param_types, result_types) = if let Some(type_idx) = func_type_idx {
                        if let Some(func_type) = module.types.get(type_idx) {
                            (func_type.params.clone(), func_type.results.clone())
                        } else {
                            (Vec::new(), Vec::new())
                        }
                    } else {
                        (Vec::new(), Vec::new())
                    };

                    if param_types.is_empty() && result_types.is_empty() {
                        map_operator_to_initial_instr_and_fixup(
                            &op,
                            current_processed_pc,
                            &control_info_stack,
                            module,
                            &mut BlockArityCache::new(),
                        )?
                    } else {
                        // Sync all active slots to stack
                        let slots_to_stack = allocator.get_active_slots();

                        // Pop consumed params in reverse order
                        for param_type in param_types.iter().rev() {
                            allocator.pop(param_type.clone());
                        }

                        // Push result types and collect for stack_to_slots
                        let mut stack_to_slots = Vec::new();
                        for result_type in &result_types {
                            let slot = allocator.push(result_type.clone());
                            stack_to_slots.push(slot);
                        }

                        let (mut instr, fixup) = map_operator_to_initial_instr_and_fixup(
                            &op,
                            current_processed_pc,
                            &control_info_stack,
                            module,
                            &mut BlockArityCache::new(),
                        )?;
                        if let ProcessedInstr::Legacy {
                            slots_to_stack: ref mut ss,
                            stack_to_slots: ref mut rs,
                            ..
                        } = instr
                        {
                            *ss = slots_to_stack;
                            *rs = stack_to_slots;
                        }
                        (instr, fixup)
                    }
                }

                wasmparser::Operator::CallIndirect { type_index, .. } => {
                    // Get function type from type_index
                    let (param_types, result_types_vec) =
                        if let Some(func_type) = module.types.get(*type_index as usize) {
                            (func_type.params.clone(), func_type.results.clone())
                        } else {
                            (Vec::new(), Vec::new())
                        };

                    // Sync all active slots to stack
                    let slots_to_stack = allocator.get_active_slots();

                    // Pop consumed values: params + 1 (table index)
                    // Table index is always i32
                    allocator.pop(ValueType::NumType(NumType::I32));
                    // Pop params in reverse order
                    for param_type in param_types.iter().rev() {
                        allocator.pop(param_type.clone());
                    }

                    // Push result types to allocator and collect for stack_to_slots
                    let mut stack_to_slots = Vec::new();
                    for result_type in &result_types_vec {
                        let slot = allocator.push(result_type.clone());
                        stack_to_slots.push(slot);
                    }

                    let (mut instr, fixup) = map_operator_to_initial_instr_and_fixup(
                        &op,
                        current_processed_pc,
                        &control_info_stack,
                        module,
                        &mut BlockArityCache::new(),
                    )?;
                    if let ProcessedInstr::Legacy {
                        slots_to_stack: ref mut ss,
                        stack_to_slots: ref mut rs,
                        ..
                    } = instr
                    {
                        *ss = slots_to_stack;
                        *rs = stack_to_slots;
                    }
                    (instr, fixup)
                }

                wasmparser::Operator::Br { relative_depth } => {
                    // Get target block's result arity to sync only TOP N slots
                    let target_arity = if *relative_depth as usize >= control_info_stack.len() {
                        // Branching to function level - use function result types
                        result_types.len()
                    } else {
                        let target_idx = control_info_stack.len() - 1 - *relative_depth as usize;
                        let (blockty, _) = &control_info_stack[target_idx];
                        get_block_result_types(blockty, module).len()
                    };

                    // Only sync TOP N slots (where N is target block's result arity)
                    let all_slots = allocator.get_active_slots();
                    let slots_to_stack = if all_slots.len() >= target_arity {
                        all_slots[all_slots.len() - target_arity..].to_vec()
                    } else {
                        all_slots
                    };

                    let (mut instr, fixup) = map_operator_to_initial_instr_and_fixup(
                        &op,
                        current_processed_pc,
                        &control_info_stack,
                        module,
                        &mut BlockArityCache::new(),
                    )?;
                    if let ProcessedInstr::Legacy {
                        slots_to_stack: ref mut ss,
                        ..
                    } = instr
                    {
                        *ss = slots_to_stack;
                    }
                    (instr, fixup)
                }
                wasmparser::Operator::BrIf { relative_depth } => {
                    //pop condition slot
                    let condition_slot = allocator.peek(ValueType::NumType(NumType::I32));
                    allocator.pop(ValueType::NumType(NumType::I32));

                    // Get target block's result arity to sync only TOP N slots
                    let target_arity = if *relative_depth as usize >= control_info_stack.len() {
                        result_types.len()
                    } else {
                        let target_idx = control_info_stack.len() - 1 - *relative_depth as usize;
                        let (blockty, _) = &control_info_stack[target_idx];
                        get_block_result_types(blockty, module).len()
                    };

                    let all_slots = allocator.get_active_slots();
                    let mut slots_to_stack = if all_slots.len() >= target_arity {
                        all_slots[all_slots.len() - target_arity..].to_vec()
                    } else {
                        all_slots
                    };
                    // Add condition slot at the end if available
                    if let Some(cond) = condition_slot {
                        slots_to_stack.push(cond);
                    }

                    let (mut instr, fixup) = map_operator_to_initial_instr_and_fixup(
                        &op,
                        current_processed_pc,
                        &control_info_stack,
                        module,
                        &mut BlockArityCache::new(),
                    )?;
                    if let ProcessedInstr::Legacy {
                        slots_to_stack: ref mut ss,
                        ..
                    } = instr
                    {
                        *ss = slots_to_stack;
                    }
                    (instr, fixup)
                }
                wasmparser::Operator::BrTable { .. } | wasmparser::Operator::Return => {
                    // For BrTable and Return, sync all slots (conservative approach)
                    let slots_to_stack = allocator.get_active_slots();
                    let (mut instr, fixup) = map_operator_to_initial_instr_and_fixup(
                        &op,
                        current_processed_pc,
                        &control_info_stack,
                        module,
                        &mut BlockArityCache::new(),
                    )?;
                    if let ProcessedInstr::Legacy {
                        slots_to_stack: ref mut ss,
                        ..
                    } = instr
                    {
                        *ss = slots_to_stack;
                    }
                    (instr, fixup)
                }
                wasmparser::Operator::Drop => {
                    // Drop consumes one value - pop from allocator and sync just that slot
                    // We need to determine the type of the value being dropped
                    // For now, try to pop from each type stack starting with I32
                    let slot_to_drop = allocator.pop_any_type();
                    let slots_to_stack = if let Some(slot) = slot_to_drop {
                        vec![slot]
                    } else {
                        Vec::new()
                    };
                    let (mut instr, fixup) = map_operator_to_initial_instr_and_fixup(
                        &op,
                        current_processed_pc,
                        &control_info_stack,
                        module,
                        &mut BlockArityCache::new(),
                    )?;
                    if let ProcessedInstr::Legacy {
                        slots_to_stack: ref mut ss,
                        ..
                    } = instr
                    {
                        *ss = slots_to_stack;
                    }
                    (instr, fixup)
                }
                _ => {
                    // Fall back to Legacy mode for unsupported instructions
                    // Sync all active slots to value_stack before executing
                    let slots_to_stack = allocator.get_active_slots();
                    allocator.clear_stack();
                    let (mut instr, fixup) = map_operator_to_initial_instr_and_fixup(
                        &op,
                        current_processed_pc,
                        &control_info_stack,
                        module,
                        &mut BlockArityCache::new(),
                    )?;
                    // Set slots_to_stack on the legacy instruction
                    if let ProcessedInstr::Legacy {
                        slots_to_stack: ref mut ss,
                        ..
                    } = instr
                    {
                        *ss = slots_to_stack;
                    }
                    (instr, fixup)
                }
            }
        } else {
            // Stack-based mode: use existing logic
            map_operator_to_initial_instr_and_fixup(
                &op,
                current_processed_pc,
                &control_info_stack,
                module,
                &mut BlockArityCache::new(),
            )?
        };

        // Try to apply optimization if enabled (but not for slot instructions)
        if enable_superinstructions && matches!(processed_instr, ProcessedInstr::Legacy { .. }) {
            try_apply_optimization(
                &op,
                &mut recent_instrs,
                &mut processed_instr,
                &mut initial_processed_instrs,
                &mut current_processed_pc,
                &mut ops,
            );
        }

        let processed_instr_template = processed_instr;
        let is_legacy = matches!(processed_instr_template, ProcessedInstr::Legacy { .. });

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

        if let wasmparser::Operator::BrTable { ref targets } = op {
            // Use slots_to_stack from processed_instr_template (set by slot mode default handler)
            let slots_to_stack =
                if let ProcessedInstr::Legacy { slots_to_stack, .. } = &processed_instr_template {
                    slots_to_stack.clone()
                } else {
                    Vec::new()
                };
            let processed_instr = ProcessedInstr::Legacy {
                handler_index: HANDLER_IDX_BR_TABLE,
                operand: Operand::None,
                slots_to_stack,
                stack_to_slots: Vec::new(),
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

        // Track recent instructions for optimization (only for Legacy instructions)
        if is_legacy {
            recent_instrs.push_back((op.clone(), current_processed_pc));
            // Keep only the last RECENT_INSTRS_WINDOW instructions
            if recent_instrs.len() > RECENT_INSTRS_WINDOW {
                recent_instrs.pop_front();
            }
        }

        current_processed_pc += 1;
    }

    if !control_stack_for_map_building.is_empty() {
        return Err(Box::new(RuntimeError::InvalidWasm(
            "Unclosed control block at end of function",
        )) as Box<dyn std::error::Error>);
    }

    // Update block operands with range information
    update_block_operands_with_ranges(&mut initial_processed_instrs, &block_end_map);

    // Get result slot before finalizing (the top of stack after all instructions)
    // Use the function's result type to peek at the correct slot type
    let result_slot = slot_allocator.as_ref().and_then(|alloc| {
        if let Some(result_type) = result_types.first() {
            // Peek at the current stack top for the result type
            alloc.peek(result_type.clone())
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
    ))
}

fn update_block_operands_with_ranges(
    processed_instrs: &mut Vec<ProcessedInstr>,
    block_end_map: &FxHashMap<usize, usize>,
) {
    for (pc, instr) in processed_instrs.iter_mut().enumerate() {
        // Skip slot-based instructions
        if !matches!(instr, ProcessedInstr::Legacy { .. }) {
            continue;
        }
        if let Operand::Block {
            arity: _,
            param_count: _,
            is_loop: _,
            start_ip,
            end_ip,
        } = instr.operand_mut()
        {
            if let Some(&actual_end_ip) = block_end_map.get(&pc) {
                // Block content starts from the instruction after the block/loop
                *start_ip = pc + 1;
                // Block content ends before the corresponding end instruction (exclude the end)
                *end_ip = actual_end_ip;
            }
        }
    }
}

fn map_operator_to_initial_instr_and_fixup(
    op: &wasmparser::Operator,
    current_processed_pc: usize,
    _control_info_stack: &[(wasmparser::BlockType, usize)],
    module: &Module,
    cache: &mut BlockArityCache,
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
        wasmparser::Operator::Block { blockty } => {
            handler_index = HANDLER_IDX_BLOCK;
            let arity = calculate_block_arity(&blockty, module, cache);
            let param_count = calculate_block_parameter_count(&blockty, module, cache);
            operand = Operand::Block {
                arity,
                param_count,
                is_loop: false,
                start_ip: 0, // Will be updated in post-processing
                end_ip: 0,   // Will be updated in post-processing
            };
        }
        wasmparser::Operator::Loop { blockty } => {
            handler_index = HANDLER_IDX_LOOP;
            let arity = calculate_block_arity(&blockty, module, cache); // Use block arity for loop results
            let param_count = calculate_block_parameter_count(&blockty, module, cache);
            operand = Operand::Block {
                arity,
                param_count,
                is_loop: true,
                start_ip: 0, // Will be updated in post-processing
                end_ip: 0,   // Will be updated in post-processing
            };
        }
        wasmparser::Operator::If { blockty } => {
            handler_index = HANDLER_IDX_IF;
            let arity = calculate_block_arity(&blockty, module, cache);

            fixup_info = Some(FixupInfo {
                pc: current_processed_pc,
                original_wasm_depth: 0,
                is_if_false_jump: true,
                is_else_jump: false,
            });
            operand = Operand::LabelIdx {
                target_ip: usize::MAX,
                arity,
                original_wasm_depth: 0,
                is_loop: false,
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
                original_wasm_depth: 0,
                is_loop: false,
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
                original_wasm_depth: relative_depth as usize,
                is_loop: false,
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
                original_wasm_depth: relative_depth as usize,
                is_loop: false,
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
            operand = Operand::CallIndirect {
                type_idx: TypeIdx(type_index),
                table_idx: TableIdx(table_index),
            };
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
        wasmparser::Operator::MemoryCopy { .. } => {
            handler_index = HANDLER_IDX_MEMORY_COPY;
        }
        wasmparser::Operator::MemoryInit { data_index, .. } => {
            handler_index = HANDLER_IDX_MEMORY_INIT;
            operand = Operand::I32(data_index as i32);
        }
        wasmparser::Operator::MemoryFill { .. } => {
            handler_index = HANDLER_IDX_MEMORY_FILL;
        }
        // TODO: DataDrop

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
            handler_index = HANDLER_IDX_I32_TRUNC_SAT_F32_S;
        }
        wasmparser::Operator::I32TruncSatF32U => {
            handler_index = HANDLER_IDX_I32_TRUNC_SAT_F32_U;
        }
        wasmparser::Operator::I32TruncSatF64S => {
            handler_index = HANDLER_IDX_I32_TRUNC_SAT_F64_S;
        }
        wasmparser::Operator::I32TruncSatF64U => {
            handler_index = HANDLER_IDX_I32_TRUNC_SAT_F64_U;
        }
        wasmparser::Operator::I64TruncSatF32S => {
            handler_index = HANDLER_IDX_I64_TRUNC_SAT_F32_S;
        }
        wasmparser::Operator::I64TruncSatF32U => {
            handler_index = HANDLER_IDX_I64_TRUNC_SAT_F32_U;
        }
        wasmparser::Operator::I64TruncSatF64S => {
            handler_index = HANDLER_IDX_I64_TRUNC_SAT_F64_S;
        }
        wasmparser::Operator::I64TruncSatF64U => {
            handler_index = HANDLER_IDX_I64_TRUNC_SAT_F64_U;
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
        wasmparser::Operator::RefNull { hty } => {
            handler_index = HANDLER_IDX_REF_NULL;
            operand = Operand::RefType(match hty {
                wasmparser::HeapType::Func => RefType::FuncRef,
                wasmparser::HeapType::Extern => RefType::ExternalRef,
                _ => RefType::ExternalRef, // Default fallback
            });
        }
        wasmparser::Operator::RefIsNull => {
            handler_index = HANDLER_IDX_REF_IS_NULL;
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
            handler_index = HANDLER_IDX_TABLE_GET;
            operand = Operand::TableIdx(TableIdx(table));
        }
        wasmparser::Operator::TableSet { table } => {
            handler_index = HANDLER_IDX_TABLE_SET;
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
            handler_index = HANDLER_IDX_TABLE_FILL;
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

    let processed_instr = ProcessedInstr::Legacy {
        handler_index,
        operand,
        slots_to_stack: Vec::new(),
        stack_to_slots: Vec::new(),
    };
    Ok((processed_instr, fixup_info))
}

pub fn parse_bytecode(
    mut module: &mut Module,
    path: &str,
    enable_superinstructions: bool,
    execution_mode: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_func_index = module.num_imported_funcs;
    let mut arity_cache = BlockArityCache::new();
    let purity_checker = ConservativePurityChecker::new();

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
                decode_code_section(
                    body,
                    &mut module,
                    current_func_index,
                    enable_superinstructions,
                    execution_mode,
                    &mut arity_cache,
                    &purity_checker,
                )?;
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
