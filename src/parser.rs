//! WebAssembly bytecode parsing and instruction preprocessing.
//!
//! This module transforms WebAssembly binary modules into an optimized intermediate
//! representation suitable for efficient interpretation.
//!
//! ## Preprocessing Pipeline
//!
//! The parser implements a multi-phase preprocessing pipeline:
//!
//! ```text
//! +----------------+     +----------------+     +----------------+
//! | Wasm Bytecode  | --> |   wasmparser   | --> |  Decode Phase  |
//! | (binary .wasm) |     |   (parsing)    |     | (build maps)   |
//! +----------------+     +----------------+     +----------------+
//!                                                       |
//!                                                       v
//! +----------------+     +----------------+     +----------------+
//! |  ProcessedInstr| <-- |  Fixup Phase   | <-- | Branch Targets |
//! | (exec ready)   |     | (resolve PCs)  |     | (Br/BrIf/etc)  |
//! +----------------+     +----------------+     +----------------+
//! ```
//!
//! ### Phase 1: Decode and Map Building
//! Parse WebAssembly instructions using `wasmparser` and build a mapping from
//! original instruction indices to processed instruction positions.
//!
//! ### Phase 2: Branch Resolution
//! Resolve branch targets (Br, BrIf, If, Else) by calculating absolute program
//! counter values from the instruction map.
//!
//! ### Phase 3: BrTable Resolution
//! Resolve BrTable targets which require special handling due to their variable
//! number of branch destinations.
//!
//! ## Register-Based Execution
//!
//! Instructions are converted to use a register-based model where operands are
//! pre-allocated registers rather than implicit stack positions. This enables
//! more efficient execution by avoiding redundant stack operations.

use std::fs::File;
use std::io::Read;
use wasmparser::{
    ExternalKind, FunctionBody, Parser, Payload::*, SectionLimited, TypeRef, ValType,
};

use crate::error::{ParserError, RuntimeError};
use crate::execution::regs::{Reg, RegAllocator};
use crate::execution::vm::*;
use crate::structure::{instructions::*, module::*, types::*};
use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::rc::Rc;
use std::sync::LazyLock;

/// Pending operand for peek-based operand folding.
/// When a const or local.get instruction is followed by a foldable consumer,
/// the operand is stored here and the source instruction is skipped.
#[derive(Clone, Copy, Debug)]
enum PendingOperand {
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
    I32Local(u16),
    I64Local(u16),
    F32Local(u16),
    F64Local(u16),
}

/// Extract I32RegOperand from pending_operands stack, falling back to register
#[inline]
fn take_i32_operand(pending: &mut Vec<PendingOperand>, reg_index: u16) -> I32RegOperand {
    if let Some(op) = pending.pop() {
        match op {
            PendingOperand::I32Const(v) => I32RegOperand::Const(v),
            PendingOperand::I32Local(idx) => I32RegOperand::Param(idx),
            _ => {
                pending.push(op);
                I32RegOperand::Reg(reg_index)
            }
        }
    } else {
        I32RegOperand::Reg(reg_index)
    }
}

/// Extract I64RegOperand from pending_operands stack, falling back to register
#[inline]
fn take_i64_operand(pending: &mut Vec<PendingOperand>, reg_index: u16) -> I64RegOperand {
    if let Some(op) = pending.pop() {
        match op {
            PendingOperand::I64Const(v) => I64RegOperand::Const(v),
            PendingOperand::I64Local(idx) => I64RegOperand::Param(idx),
            _ => {
                pending.push(op);
                I64RegOperand::Reg(reg_index)
            }
        }
    } else {
        I64RegOperand::Reg(reg_index)
    }
}

/// Extract F32RegOperand from pending_operands stack, falling back to register
#[inline]
fn take_f32_operand(pending: &mut Vec<PendingOperand>, reg_index: u16) -> F32RegOperand {
    if let Some(op) = pending.pop() {
        match op {
            PendingOperand::F32Const(v) => F32RegOperand::Const(v),
            PendingOperand::F32Local(idx) => F32RegOperand::Param(idx),
            _ => {
                pending.push(op);
                F32RegOperand::Reg(reg_index)
            }
        }
    } else {
        F32RegOperand::Reg(reg_index)
    }
}

/// Extract F64RegOperand from pending_operands stack, falling back to register
#[inline]
fn take_f64_operand(pending: &mut Vec<PendingOperand>, reg_index: u16) -> F64RegOperand {
    if let Some(op) = pending.pop() {
        match op {
            PendingOperand::F64Const(v) => F64RegOperand::Const(v),
            PendingOperand::F64Local(idx) => F64RegOperand::Param(idx),
            _ => {
                pending.push(op);
                F64RegOperand::Reg(reg_index)
            }
        }
    } else {
        F64RegOperand::Reg(reg_index)
    }
}

/// Check if the next instruction(s) can fold an I32 operand.
#[inline]
fn can_fold_i32<'a>(
    ops: &mut itertools::MultiPeek<
        impl Iterator<Item = Result<(wasmparser::Operator<'a>, usize), wasmparser::BinaryReaderError>>,
    >,
) -> bool {
    let can_fold = if let Some(Ok((next_op, _))) = ops.peek() {
        if is_i32_foldable_consumer(next_op) {
            true
        } else if matches!(
            next_op,
            wasmparser::Operator::I32Const { .. } | wasmparser::Operator::LocalGet { .. }
        ) {
            if let Some(Ok((next_next_op, _))) = ops.peek() {
                is_i32_foldable_consumer(next_next_op)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    ops.reset_peek();
    can_fold
}

/// Check if the next instruction(s) can fold an I64 operand.
#[inline]
fn can_fold_i64<'a>(
    ops: &mut itertools::MultiPeek<
        impl Iterator<Item = Result<(wasmparser::Operator<'a>, usize), wasmparser::BinaryReaderError>>,
    >,
) -> bool {
    let can_fold = if let Some(Ok((next_op, _))) = ops.peek() {
        if is_i64_foldable_consumer(next_op) {
            true
        } else if matches!(
            next_op,
            wasmparser::Operator::I64Const { .. } | wasmparser::Operator::LocalGet { .. }
        ) {
            if let Some(Ok((next_next_op, _))) = ops.peek() {
                is_i64_foldable_consumer(next_next_op)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    ops.reset_peek();
    can_fold
}

/// Check if the next instruction(s) can fold an F32 operand.
#[inline]
fn can_fold_f32<'a>(
    ops: &mut itertools::MultiPeek<
        impl Iterator<Item = Result<(wasmparser::Operator<'a>, usize), wasmparser::BinaryReaderError>>,
    >,
) -> bool {
    let can_fold = if let Some(Ok((next_op, _))) = ops.peek() {
        if is_f32_foldable_consumer(next_op) {
            true
        } else if matches!(
            next_op,
            wasmparser::Operator::F32Const { .. } | wasmparser::Operator::LocalGet { .. }
        ) {
            if let Some(Ok((next_next_op, _))) = ops.peek() {
                is_f32_foldable_consumer(next_next_op)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    ops.reset_peek();
    can_fold
}

/// Check if the next instruction(s) can fold an F64 operand.
#[inline]
fn can_fold_f64<'a>(
    ops: &mut itertools::MultiPeek<
        impl Iterator<Item = Result<(wasmparser::Operator<'a>, usize), wasmparser::BinaryReaderError>>,
    >,
) -> bool {
    let can_fold = if let Some(Ok((next_op, _))) = ops.peek() {
        if is_f64_foldable_consumer(next_op) {
            true
        } else if matches!(
            next_op,
            wasmparser::Operator::F64Const { .. } | wasmparser::Operator::LocalGet { .. }
        ) {
            if let Some(Ok((next_next_op, _))) = ops.peek() {
                is_f64_foldable_consumer(next_next_op)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    ops.reset_peek();
    can_fold
}

/// Check if the instruction can consume an I32 operand (excluding br_if/if/loads/stores)
/// Note: Load/Store instructions are excluded because the 2-ahead lookahead in can_fold_i32
/// doesn't verify the type of intervening LocalGet, which can cause incorrect folding.
#[inline]
fn is_i32_foldable_consumer(op: &wasmparser::Operator) -> bool {
    matches!(
        op,
        wasmparser::Operator::I32Add
            | wasmparser::Operator::I32Sub
            | wasmparser::Operator::I32Mul
            | wasmparser::Operator::I32DivS
            | wasmparser::Operator::I32DivU
            | wasmparser::Operator::I32RemS
            | wasmparser::Operator::I32RemU
            | wasmparser::Operator::I32And
            | wasmparser::Operator::I32Or
            | wasmparser::Operator::I32Xor
            | wasmparser::Operator::I32Shl
            | wasmparser::Operator::I32ShrS
            | wasmparser::Operator::I32ShrU
            | wasmparser::Operator::I32Rotl
            | wasmparser::Operator::I32Rotr
            | wasmparser::Operator::I32Eq
            | wasmparser::Operator::I32Ne
            | wasmparser::Operator::I32LtS
            | wasmparser::Operator::I32LtU
            | wasmparser::Operator::I32LeS
            | wasmparser::Operator::I32LeU
            | wasmparser::Operator::I32GtS
            | wasmparser::Operator::I32GtU
            | wasmparser::Operator::I32GeS
            | wasmparser::Operator::I32GeU
            | wasmparser::Operator::I32Clz
            | wasmparser::Operator::I32Ctz
            | wasmparser::Operator::I32Popcnt
            | wasmparser::Operator::I32Eqz
    )
}

/// Check if the instruction is a memory load (addr is I32, single operand)
#[inline]
fn is_memory_load(op: &wasmparser::Operator) -> bool {
    matches!(
        op,
        wasmparser::Operator::I32Load { .. }
            | wasmparser::Operator::I64Load { .. }
            | wasmparser::Operator::F32Load { .. }
            | wasmparser::Operator::F64Load { .. }
            | wasmparser::Operator::I32Load8S { .. }
            | wasmparser::Operator::I32Load8U { .. }
            | wasmparser::Operator::I32Load16S { .. }
            | wasmparser::Operator::I32Load16U { .. }
            | wasmparser::Operator::I64Load8S { .. }
            | wasmparser::Operator::I64Load8U { .. }
            | wasmparser::Operator::I64Load16S { .. }
            | wasmparser::Operator::I64Load16U { .. }
            | wasmparser::Operator::I64Load32S { .. }
            | wasmparser::Operator::I64Load32U { .. }
    )
}

/// Check if the next instruction is a memory load (1-ahead only, for I32 addr folding)
#[inline]
fn can_fold_for_load<'a>(
    ops: &mut itertools::MultiPeek<
        impl Iterator<Item = Result<(wasmparser::Operator<'a>, usize), wasmparser::BinaryReaderError>>,
    >,
) -> bool {
    let can_fold = if let Some(Ok((next_op, _))) = ops.peek() {
        is_memory_load(next_op)
    } else {
        false
    };
    ops.reset_peek();
    can_fold
}

/// Check if the instruction is a memory store (for 2-ahead matching)
#[inline]
fn is_memory_store(op: &wasmparser::Operator) -> bool {
    matches!(
        op,
        wasmparser::Operator::I32Store { .. }
            | wasmparser::Operator::I64Store { .. }
            | wasmparser::Operator::F32Store { .. }
            | wasmparser::Operator::F64Store { .. }
            | wasmparser::Operator::I32Store8 { .. }
            | wasmparser::Operator::I32Store16 { .. }
            | wasmparser::Operator::I64Store8 { .. }
            | wasmparser::Operator::I64Store16 { .. }
            | wasmparser::Operator::I64Store32 { .. }
    )
}

/// Check if 2-ahead pattern matches [value_producer, store] for addr folding
/// Pattern: addr (current) -> value -> store
#[inline]
fn can_fold_for_store<'a>(
    ops: &mut itertools::MultiPeek<
        impl Iterator<Item = Result<(wasmparser::Operator<'a>, usize), wasmparser::BinaryReaderError>>,
    >,
) -> bool {
    let can_fold = if let Some(Ok((value_op, _))) = ops.peek() {
        // Check if value_op could produce a value for store
        let is_value_candidate = matches!(
            value_op,
            wasmparser::Operator::I32Const { .. }
                | wasmparser::Operator::I64Const { .. }
                | wasmparser::Operator::F32Const { .. }
                | wasmparser::Operator::F64Const { .. }
                | wasmparser::Operator::LocalGet { .. }
        );
        if is_value_candidate {
            if let Some(Ok((store_op, _))) = ops.peek() {
                is_memory_store(store_op)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    ops.reset_peek();
    can_fold
}

/// Check if the next instruction is local.set with I32 type (for dst folding)
/// Returns Some(local_idx) if folding is possible, None otherwise
#[inline]
fn try_fold_dst_i32<'a>(
    ops: &mut itertools::MultiPeek<
        impl Iterator<Item = Result<(wasmparser::Operator<'a>, usize), wasmparser::BinaryReaderError>>,
    >,
    param_types: &[ValueType],
    locals: &[(u32, ValueType)],
) -> Option<u16> {
    let result = if let Some(Ok((next_op, _))) = ops.peek() {
        if let wasmparser::Operator::LocalSet { local_index } = next_op {
            let local_type = get_local_type(param_types, locals, *local_index);
            if matches!(local_type, ValueType::NumType(NumType::I32)) {
                Some(*local_index as u16)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    ops.reset_peek();
    result
}

/// Check if the next instruction is local.set with I64 type (for dst folding)
#[inline]
fn try_fold_dst_i64<'a>(
    ops: &mut itertools::MultiPeek<
        impl Iterator<Item = Result<(wasmparser::Operator<'a>, usize), wasmparser::BinaryReaderError>>,
    >,
    param_types: &[ValueType],
    locals: &[(u32, ValueType)],
) -> Option<u16> {
    let result = if let Some(Ok((next_op, _))) = ops.peek() {
        if let wasmparser::Operator::LocalSet { local_index } = next_op {
            let local_type = get_local_type(param_types, locals, *local_index);
            if matches!(local_type, ValueType::NumType(NumType::I64)) {
                Some(*local_index as u16)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    ops.reset_peek();
    result
}

/// Check if the next instruction is local.set with F32 type (for dst folding)
#[inline]
fn try_fold_dst_f32<'a>(
    ops: &mut itertools::MultiPeek<
        impl Iterator<Item = Result<(wasmparser::Operator<'a>, usize), wasmparser::BinaryReaderError>>,
    >,
    param_types: &[ValueType],
    locals: &[(u32, ValueType)],
) -> Option<u16> {
    let result = if let Some(Ok((next_op, _))) = ops.peek() {
        if let wasmparser::Operator::LocalSet { local_index } = next_op {
            let local_type = get_local_type(param_types, locals, *local_index);
            if matches!(local_type, ValueType::NumType(NumType::F32)) {
                Some(*local_index as u16)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    ops.reset_peek();
    result
}

/// Check if the next instruction is local.set with F64 type (for dst folding)
#[inline]
fn try_fold_dst_f64<'a>(
    ops: &mut itertools::MultiPeek<
        impl Iterator<Item = Result<(wasmparser::Operator<'a>, usize), wasmparser::BinaryReaderError>>,
    >,
    param_types: &[ValueType],
    locals: &[(u32, ValueType)],
) -> Option<u16> {
    let result = if let Some(Ok((next_op, _))) = ops.peek() {
        if let wasmparser::Operator::LocalSet { local_index } = next_op {
            let local_type = get_local_type(param_types, locals, *local_index);
            if matches!(local_type, ValueType::NumType(NumType::F64)) {
                Some(*local_index as u16)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    ops.reset_peek();
    result
}

/// Check if the instruction can consume an I64 operand
#[inline]
fn is_i64_foldable_consumer(op: &wasmparser::Operator) -> bool {
    matches!(
        op,
        wasmparser::Operator::I64Add
            | wasmparser::Operator::I64Sub
            | wasmparser::Operator::I64Mul
            | wasmparser::Operator::I64DivS
            | wasmparser::Operator::I64DivU
            | wasmparser::Operator::I64RemS
            | wasmparser::Operator::I64RemU
            | wasmparser::Operator::I64And
            | wasmparser::Operator::I64Or
            | wasmparser::Operator::I64Xor
            | wasmparser::Operator::I64Shl
            | wasmparser::Operator::I64ShrS
            | wasmparser::Operator::I64ShrU
            | wasmparser::Operator::I64Rotl
            | wasmparser::Operator::I64Rotr
            | wasmparser::Operator::I64Eq
            | wasmparser::Operator::I64Ne
            | wasmparser::Operator::I64LtS
            | wasmparser::Operator::I64LtU
            | wasmparser::Operator::I64LeS
            | wasmparser::Operator::I64LeU
            | wasmparser::Operator::I64GtS
            | wasmparser::Operator::I64GtU
            | wasmparser::Operator::I64GeS
            | wasmparser::Operator::I64GeU
            | wasmparser::Operator::I64Clz
            | wasmparser::Operator::I64Ctz
            | wasmparser::Operator::I64Popcnt
            | wasmparser::Operator::I64Eqz
    )
}

/// Check if the instruction can consume an F32 operand
#[inline]
fn is_f32_foldable_consumer(op: &wasmparser::Operator) -> bool {
    matches!(
        op,
        wasmparser::Operator::F32Add
            | wasmparser::Operator::F32Sub
            | wasmparser::Operator::F32Mul
            | wasmparser::Operator::F32Div
            | wasmparser::Operator::F32Min
            | wasmparser::Operator::F32Max
            | wasmparser::Operator::F32Copysign
            | wasmparser::Operator::F32Eq
            | wasmparser::Operator::F32Ne
            | wasmparser::Operator::F32Lt
            | wasmparser::Operator::F32Le
            | wasmparser::Operator::F32Gt
            | wasmparser::Operator::F32Ge
            | wasmparser::Operator::F32Abs
            | wasmparser::Operator::F32Neg
            | wasmparser::Operator::F32Ceil
            | wasmparser::Operator::F32Floor
            | wasmparser::Operator::F32Trunc
            | wasmparser::Operator::F32Nearest
            | wasmparser::Operator::F32Sqrt
    )
}

/// Check if the instruction can consume an F64 operand
#[inline]
fn is_f64_foldable_consumer(op: &wasmparser::Operator) -> bool {
    matches!(
        op,
        wasmparser::Operator::F64Add
            | wasmparser::Operator::F64Sub
            | wasmparser::Operator::F64Mul
            | wasmparser::Operator::F64Div
            | wasmparser::Operator::F64Min
            | wasmparser::Operator::F64Max
            | wasmparser::Operator::F64Copysign
            | wasmparser::Operator::F64Eq
            | wasmparser::Operator::F64Ne
            | wasmparser::Operator::F64Lt
            | wasmparser::Operator::F64Le
            | wasmparser::Operator::F64Gt
            | wasmparser::Operator::F64Ge
            | wasmparser::Operator::F64Abs
            | wasmparser::Operator::F64Neg
            | wasmparser::Operator::F64Ceil
            | wasmparser::Operator::F64Floor
            | wasmparser::Operator::F64Trunc
            | wasmparser::Operator::F64Nearest
            | wasmparser::Operator::F64Sqrt
    )
}

/// Control block information tracked during instruction decoding.
///
/// Each control structure (block, loop, if) pushes an entry onto the control
/// stack during parsing. This tracks the block's type signature and the
/// registers allocated for its parameters and results.
#[derive(Debug, Clone)]
struct ControlBlockInfo {
    /// The block's type signature (empty, single type, or function type index).
    block_type: wasmparser::BlockType,
    /// Whether this is a loop (affects branch target calculation).
    is_loop: bool,
    /// Registers allocated for the block's result values.
    result_regs: Vec<Reg>,
    /// Registers allocated for the block's parameter values.
    param_regs: Vec<Reg>,
}

/// Cache key for block type arity calculations.
///
/// Used to memoize the number of parameters and results for block types,
/// avoiding repeated lookups in the type section.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum BlockTypeKey {
    /// Block with no parameters or results.
    Empty,
    /// Block with a single result type (no parameters).
    SingleType(ValueType),
    /// Block with a function type signature (may have multiple params/results).
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

/// Cache for block type arity calculations.
///
/// Stores precomputed result counts for blocks and parameter counts for loops,
/// reducing redundant type section lookups during instruction decoding.
struct BlockArityCache {
    /// Maps block types to their result arity (number of return values).
    block_arity_cache: FxHashMap<BlockTypeKey, usize>,
    /// Maps block types to their parameter arity (for loop continuation).
    loop_parameter_arity_cache: FxHashMap<BlockTypeKey, usize>,
}

impl BlockArityCache {
    /// Creates a new empty arity cache.
    fn new() -> Self {
        Self {
            block_arity_cache: FxHashMap::default(),
            loop_parameter_arity_cache: FxHashMap::default(),
        }
    }
}

/// Mapping from WASI function names to their internal type representations.
///
/// Used during import section parsing to identify WASI Preview 1 functions
/// and create the appropriate `WasiFuncType` entries for passthrough handling.
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

/// Calculates the result arity (number of return values) for a block type.
///
/// Uses the cache to avoid repeated lookups. For blocks and if/else, this
/// determines how many values are left on the stack when the block completes.
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

/// Calculates the parameter arity for loop continuation.
///
/// When branching to a loop, the branch target is the loop header, so we need
/// the parameter count (not result count) to know how many values to pass.
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

        Rc::get_mut(&mut module.types)
            .unwrap()
            .push(crate::structure::types::FuncType { params, results });
    }
    Ok(())
}

/// Decodes the function section, creating stub entries for each function.
///
/// The function section only contains type indices; the actual code bodies
/// are decoded separately from the code section.
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
            reg_allocation: None,
            result_reg: None,
        });
    }

    Ok(())
}

/// Decodes the import section, handling functions, tables, memories, and globals.
///
/// WASI Preview 1 function imports from `wasi_snapshot_preview1` are identified
/// and stored with their specific `WasiFuncType` for passthrough handling.
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

/// Looks up a WASI function name in the function map.
fn parse_wasi_function(name: &str) -> Option<WasiFuncType> {
    WASI_FUNCTION_MAP.get(name).copied()
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
            type_: MemType(limits),
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

/// Decodes a single function's code section entry.
///
/// This is the core of instruction preprocessing:
/// 1. Parses locals and instructions
/// 2. Allocates registers for operands
/// 3. Builds branch target maps
/// 4. Resolves all branch destinations
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

    // Get the function's parameter and result types for register allocation
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
        reg_allocation,
        result_reg,
        block_result_regs_map,
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
        &block_result_regs_map,
        module,
        cache,
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    let body_rc = Rc::new(processed_instrs);

    // Store function body and metadata in module
    if let Some(func) = module.funcs.get_mut(relative_func_index) {
        func.body = body_rc;
        // Store register mode metadata (None for stack mode)
        func.reg_allocation = reg_allocation.clone();
        func.result_reg = result_reg;
    } else {
        return Err(Box::new(RuntimeError::InvalidWasm(
            "Invalid function index when storing body",
        )) as Box<dyn std::error::Error>);
    }

    Ok(())
}

/// Information needed to fix up a branch instruction's target address.
///
/// During initial instruction decoding, branch targets are unknown because
/// the destination block's end position hasn't been seen yet. FixupInfo
/// records what needs to be patched once all blocks are parsed.
#[derive(Debug, Clone)]
struct FixupInfo {
    /// Program counter of the instruction needing fixup.
    pc: usize,
    /// Original WebAssembly branch depth (relative to control stack).
    original_wasm_depth: usize,
    /// True if this is an If instruction's false-branch jump.
    is_if_false_jump: bool,
    /// True if this is an Else instruction's end-of-then-branch jump.
    is_else_jump: bool,
    /// Source registers for multi-value branch results.
    source_regs: Vec<Reg>,
}

/// Resolves branch targets after initial instruction decoding (Phase 2 & 3).
///
/// Phase 1 is performed by [`decode_processed_instrs_and_fixups`].
///
/// - **Phase 2**: Resolves Br, BrIf, If, Else jumps using the block-end map.
/// - **Phase 3**: Resolves BrTable jumps which have multiple targets.
fn preprocess_instructions(
    processed: &mut Vec<ProcessedInstr>,
    fixups: &mut Vec<FixupInfo>,
    block_end_map: &FxHashMap<usize, usize>,
    if_else_map: &FxHashMap<usize, usize>,
    block_type_map: &FxHashMap<usize, wasmparser::BlockType>,
    block_result_regs_map: &FxHashMap<usize, (Vec<Reg>, bool)>,
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
                if let ProcessedInstr::BrReg {
                    target_ip: ref mut tip,
                    ..
                } = instr_to_patch
                {
                    *tip = function_end_ip;
                } else if let ProcessedInstr::BrIfReg {
                    target_ip: ref mut tip,
                    ..
                } = instr_to_patch
                {
                    *tip = function_end_ip;
                } else if is_if_false_jump {
                    if !matches!(instr_to_patch, ProcessedInstr::IfReg { .. }) {
                        fixups[fixup_index].original_wasm_depth = usize::MAX;
                        continue;
                    }
                } else if is_else_jump {
                    if !matches!(instr_to_patch, ProcessedInstr::JumpReg { .. }) {
                        fixups[fixup_index].original_wasm_depth = usize::MAX;
                        continue;
                    }
                } else if !matches!(
                    instr_to_patch,
                    ProcessedInstr::JumpReg { .. }
                        | ProcessedInstr::IfReg { .. }
                        | ProcessedInstr::BrReg { .. }
                        | ProcessedInstr::BrIfReg { .. }
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
        // Note: block_end_map already stores End + 1 position (the instruction after EndReg)
        let target_ip = if is_loop {
            target_start_pc
        } else {
            *block_end_map
                .get(&target_start_pc)
                .ok_or_else(|| RuntimeError::InvalidWasm("Missing EndMarker for branch target"))?
        };

        // Patch the instruction operand
        if let Some(instr_to_patch) = processed.get_mut(current_fixup_pc) {
            // Skip fixup for register-based instructions (except those that need fixup)
            if !matches!(
                instr_to_patch,
                ProcessedInstr::JumpReg { .. }
                    | ProcessedInstr::IfReg { .. }
                    | ProcessedInstr::BrReg { .. }
                    | ProcessedInstr::BrIfReg { .. }
            ) {
                fixups[fixup_index].original_wasm_depth = usize::MAX;
                continue;
            }

            if is_if_false_jump {
                // If instruction's jump-on-false
                // Target is ElseMarker+1 or EndMarker+1
                let else_target = *if_else_map.get(&target_start_pc).unwrap_or(&target_ip);
                let has_else = else_target != target_ip;

                if let ProcessedInstr::IfReg {
                    else_target_ip: ref mut tip,
                    has_else: ref mut he,
                    ..
                } = instr_to_patch
                {
                    *tip = else_target;
                    *he = has_else;
                }
            } else if is_else_jump {
                if let ProcessedInstr::JumpReg {
                    target_ip: ref mut tip,
                } = instr_to_patch
                {
                    *tip = target_ip;
                }
            } else {
                // Br or BrIf instruction
                if let ProcessedInstr::BrReg {
                    target_ip: ref mut tip,
                    ..
                } = instr_to_patch
                {
                    *tip = target_ip;
                } else if let ProcessedInstr::BrIfReg {
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
            let needs_br_table_resolution = matches!(instr, ProcessedInstr::BrTableReg { .. });

            if needs_br_table_resolution {
                // Handle BrTableReg directly without using fixups
                if matches!(instr, ProcessedInstr::BrTableReg { .. }) {
                    // Clone targets to get relative depths and existing result_regs
                    let (targets_clone, default_info) = if let ProcessedInstr::BrTableReg {
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
                    let default_result_regs = default_info.2;

                    // Compute target_ip for each target (keeping existing result_regs)
                    let mut resolved_reg_targets: Vec<(u32, usize, Vec<Reg>)> = Vec::new();
                    for (rel_depth, _, result_regs) in targets_clone.iter() {
                        let depth = *rel_depth as usize;
                        if current_control_stack_pass3.len() <= depth {
                            resolved_reg_targets.push((*rel_depth, 0, result_regs.clone())); // Invalid
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
                        resolved_reg_targets.push((*rel_depth, target_ip, result_regs.clone()));
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

                    // Update BrTableReg
                    if let Some(instr_to_patch) = processed.get_mut(pc) {
                        if let ProcessedInstr::BrTableReg {
                            targets: ref mut reg_targets,
                            default_target: ref mut reg_default,
                            ..
                        } = instr_to_patch
                        {
                            *reg_targets = resolved_reg_targets;
                            *reg_default =
                                (default_depth as u32, default_target_ip, default_result_regs);
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

                    // Get source_regs from fixup (computed at BrTable creation time)
                    let source_regs = fixups[default_fixup_idx].source_regs.clone();
                    // Get target_result_regs from block_result_regs_map
                    let target_result_regs = block_result_regs_map
                        .get(&target_start_pc)
                        .map(|(regs, _)| regs.clone())
                        .unwrap_or_default();

                    fixups[default_fixup_idx].original_wasm_depth = usize::MAX;

                    Operand::LabelIdx {
                        target_ip,
                        arity: target_arity,
                        original_wasm_depth: fixup_depth,
                        is_loop: is_loop, // Use default target's loop/block information
                        source_regs,
                        target_result_regs,
                        cond_reg: None,
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

                        // Get source_regs from fixup (computed at BrTable creation time)
                        let source_regs = fixups[fixup_idx].source_regs.clone();
                        // Get target_result_regs from block_result_regs_map
                        let target_result_regs = block_result_regs_map
                            .get(&target_start_pc)
                            .map(|(regs, _)| regs.clone())
                            .unwrap_or_default();

                        fixups[fixup_idx].original_wasm_depth = usize::MAX;

                        Operand::LabelIdx {
                            target_ip,
                            arity: target_arity,
                            original_wasm_depth: fixup_depth,
                            is_loop: is_loop,
                            source_regs,
                            target_result_regs,
                            cond_reg: None,
                        }
                    };
                    resolved_targets.push(target_operand);
                }

                // --- Patch BrTable Instruction ---
                if let Some(instr_to_patch) = processed.get_mut(pc) {
                    if let ProcessedInstr::BrTableReg {
                        targets: ref mut reg_targets,
                        default_target: ref mut reg_default,
                        ..
                    } = instr_to_patch
                    {
                        // Extract target_ip and target_result_regs from resolved_targets (Operand::LabelIdx)
                        for (i, operand) in resolved_targets.iter().enumerate() {
                            if let Operand::LabelIdx {
                                target_ip,
                                original_wasm_depth,
                                target_result_regs,
                                ..
                            } = operand
                            {
                                if i < reg_targets.len() {
                                    reg_targets[i] = (
                                        *original_wasm_depth as u32,
                                        *target_ip,
                                        target_result_regs.clone(),
                                    );
                                }
                            }
                        }
                        // Set default target
                        if let Operand::LabelIdx {
                            target_ip,
                            original_wasm_depth,
                            target_result_regs,
                            ..
                        } = &default_target_operand
                        {
                            *reg_default = (
                                *original_wasm_depth as u32,
                                *target_ip,
                                target_result_regs.clone(),
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

/// Returns the type of a local variable by index.
///
/// WebAssembly local indices include function parameters first (indices 0..n-1),
/// followed by declared locals. The `locals` parameter uses compressed format
/// where each entry is (count, type).
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

/// Returns the value type of a global variable by index.
///
/// Searches imported globals first, then module-defined globals.
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

/// Returns the element type of a table by index.
///
/// Searches imported tables first, then module-defined tables.
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

/// Decodes WebAssembly instructions into register-based processed instructions.
///
/// This is the first phase of instruction preprocessing (Phase 1). It:
/// - Converts stack-based WebAssembly instructions to register-based format
/// - Allocates registers for operands and results
/// - Builds maps for block boundaries (block_end_map, if_else_map)
/// - Records fixup information for branch instructions
///
/// # Returns
///
/// A tuple containing:
/// - Processed instructions (with placeholder branch targets)
/// - Fixup information for branch resolution
/// - Block-end position map
/// - If-else position map
/// - Block type map
/// - Register allocation metadata
/// - Function result register
/// - Block result registers map (for BrTable resolution)
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
        Option<crate::execution::regs::RegAllocation>,
        Option<crate::execution::regs::Reg>,
        FxHashMap<usize, (Vec<Reg>, bool)>,
    ),
    Box<dyn std::error::Error>,
> {
    let mut ops = ops_iter.multipeek();
    let mut initial_processed_instrs = Vec::new();
    let mut initial_fixups = Vec::new();
    let mut current_processed_pc = 0;
    // Control block stack with result registers
    let mut control_info_stack: Vec<ControlBlockInfo> = Vec::new();

    let mut block_end_map: FxHashMap<usize, usize> = FxHashMap::default();
    let mut if_else_map: FxHashMap<usize, usize> = FxHashMap::default();
    let mut block_type_map: FxHashMap<usize, wasmparser::BlockType> = FxHashMap::default();
    let mut control_stack_for_map_building: Vec<(usize, bool, Option<usize>)> = Vec::new();
    // Map from block_start_pc to (result_regs, is_loop) for BrTable resolution in Pass 3
    let mut block_result_regs_map: FxHashMap<usize, (Vec<Reg>, bool)> = FxHashMap::default();

    // Initialize register allocator (always used since Legacy mode is removed)
    use crate::execution::regs::RegAllocator;
    let mut reg_allocator = Some(RegAllocator::new(locals));

    // Track allocator state at block entry for proper restoration on block exit
    let mut allocator_state_stack: Vec<crate::execution::regs::RegAllocatorState> = Vec::new();

    // Track unreachable code depth (after br, return, unreachable, br_table)
    let mut unreachable_depth: usize = 0;

    // Pending operands for folding (stack for multiple operands)
    let mut pending_operands: Vec<PendingOperand> = Vec::new();

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
                initial_processed_instrs.push(ProcessedInstr::NopReg);
                current_processed_pc += 1;
                continue;
            }
        }

        // Get the processed instruction based on execution mode
        let (processed_instr, fixup_info_opt) = if let Some(ref mut allocator) = reg_allocator {
            // Register-based mode: convert i32 instructions to register format
            match &op {
                wasmparser::Operator::LocalGet { local_index } => {
                    let local_type = get_local_type(param_types, locals, *local_index);
                    match local_type {
                        ValueType::NumType(NumType::I32) => {
                            if can_fold_i32(&mut ops)
                                || can_fold_for_load(&mut ops)
                                || can_fold_for_store(&mut ops)
                            {
                                pending_operands
                                    .push(PendingOperand::I32Local(*local_index as u16));
                                allocator.push(local_type);
                                (ProcessedInstr::NopReg, None)
                            } else {
                                let dst = allocator.push(local_type);
                                (
                                    ProcessedInstr::I32Reg {
                                        handler_index: HANDLER_IDX_LOCAL_GET,
                                        dst: I32RegOperand::Reg(dst.index()),
                                        src1: I32RegOperand::Param(*local_index as u16),
                                        src2: None,
                                    },
                                    None,
                                )
                            }
                        }
                        ValueType::NumType(NumType::I64) => {
                            if can_fold_i64(&mut ops) {
                                pending_operands
                                    .push(PendingOperand::I64Local(*local_index as u16));
                                allocator.push(local_type);
                                (ProcessedInstr::NopReg, None)
                            } else {
                                let dst = allocator.push(local_type);
                                (
                                    ProcessedInstr::I64Reg {
                                        handler_index: HANDLER_IDX_LOCAL_GET,
                                        dst: I64RegOperand::Reg(dst.index()),
                                        src1: I64RegOperand::Param(*local_index as u16),
                                        src2: None,
                                    },
                                    None,
                                )
                            }
                        }
                        ValueType::NumType(NumType::F32) => {
                            if can_fold_f32(&mut ops) {
                                pending_operands
                                    .push(PendingOperand::F32Local(*local_index as u16));
                                allocator.push(local_type);
                                (ProcessedInstr::NopReg, None)
                            } else {
                                let dst = allocator.push(local_type);
                                (
                                    ProcessedInstr::F32Reg {
                                        handler_index: HANDLER_IDX_LOCAL_GET,
                                        dst: F32RegOperand::Reg(dst.index()),
                                        src1: F32RegOperand::Param(*local_index as u16),
                                        src2: None,
                                    },
                                    None,
                                )
                            }
                        }
                        ValueType::NumType(NumType::F64) => {
                            if can_fold_f64(&mut ops) {
                                pending_operands
                                    .push(PendingOperand::F64Local(*local_index as u16));
                                allocator.push(local_type);
                                (ProcessedInstr::NopReg, None)
                            } else {
                                let dst = allocator.push(local_type);
                                (
                                    ProcessedInstr::F64Reg {
                                        handler_index: HANDLER_IDX_LOCAL_GET,
                                        dst: F64RegOperand::Reg(dst.index()),
                                        src1: F64RegOperand::Param(*local_index as u16),
                                        src2: None,
                                    },
                                    None,
                                )
                            }
                        }
                        ValueType::RefType(_) => {
                            // For RefType, use RefLocalReg (no folding for ref types)
                            let dst = allocator.push(local_type);
                            (
                                ProcessedInstr::RefLocalReg {
                                    handler_index: HANDLER_IDX_REF_LOCAL_GET_REG,
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
                        ($instr:ident, $operand:ident) => {
                            (
                                ProcessedInstr::$instr {
                                    handler_index: HANDLER_IDX_LOCAL_SET,
                                    dst: $operand::Param(local_idx),
                                    src1: $operand::Reg(src_idx),
                                    src2: None,
                                },
                                None,
                            )
                        };
                    }
                    match local_type {
                        ValueType::NumType(NumType::I32) => (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_LOCAL_SET,
                                dst: I32RegOperand::Param(local_idx),
                                src1: I32RegOperand::Reg(src_idx),
                                src2: None,
                            },
                            None,
                        ),
                        ValueType::NumType(NumType::I64) => {
                            make_local_set!(I64Reg, I64RegOperand)
                        }
                        ValueType::NumType(NumType::F32) => {
                            make_local_set!(F32Reg, F32RegOperand)
                        }
                        ValueType::NumType(NumType::F64) => {
                            make_local_set!(F64Reg, F64RegOperand)
                        }
                        ValueType::RefType(_) => {
                            (
                                ProcessedInstr::RefLocalReg {
                                    handler_index: HANDLER_IDX_REF_LOCAL_SET_REG,
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
                    // Peek the top register (don't pop - value stays on stack)
                    let src_idx = allocator.peek(&local_type).unwrap().index();
                    macro_rules! make_local_tee {
                        ($instr:ident, $operand:ident) => {
                            (
                                ProcessedInstr::$instr {
                                    handler_index: HANDLER_IDX_LOCAL_SET, // Reuse local.set handler
                                    dst: $operand::Param(local_idx),
                                    src1: $operand::Reg(src_idx),
                                    src2: None,
                                },
                                None,
                            )
                        };
                    }
                    match local_type {
                        ValueType::NumType(NumType::I32) => (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_LOCAL_SET,
                                dst: I32RegOperand::Param(local_idx),
                                src1: I32RegOperand::Reg(src_idx),
                                src2: None,
                            },
                            None,
                        ),
                        ValueType::NumType(NumType::I64) => {
                            make_local_tee!(I64Reg, I64RegOperand)
                        }
                        ValueType::NumType(NumType::F32) => {
                            make_local_tee!(F32Reg, F32RegOperand)
                        }
                        ValueType::NumType(NumType::F64) => {
                            make_local_tee!(F64Reg, F64RegOperand)
                        }
                        ValueType::RefType(_) => {
                            (
                                ProcessedInstr::RefLocalReg {
                                    handler_index: HANDLER_IDX_REF_LOCAL_SET_REG,
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
                                ProcessedInstr::GlobalGetReg {
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
                                ProcessedInstr::GlobalGetReg {
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
                                ProcessedInstr::GlobalGetReg {
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
                                ProcessedInstr::GlobalGetReg {
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
                                ProcessedInstr::GlobalSetReg {
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
                                ProcessedInstr::GlobalSetReg {
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
                                ProcessedInstr::GlobalSetReg {
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
                                ProcessedInstr::GlobalSetReg {
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
                    if can_fold_i32(&mut ops)
                        || can_fold_for_load(&mut ops)
                        || can_fold_for_store(&mut ops)
                    {
                        pending_operands.push(PendingOperand::I32Const(*value));
                        allocator.push(ValueType::NumType(NumType::I32));
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_CONST,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1: I32RegOperand::Const(*value),
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                // Binary operations - macro to reduce repetition
                wasmparser::Operator::I32Add => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    // Check if next instruction is local.set with I32 type
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        // Consume the local.set instruction
                        let _ = ops.next();
                        // Push and immediately pop to maintain stack consistency
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        // Insert the instruction that writes directly to local
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_ADD,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        // Insert NopReg for the consumed local.set
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_ADD,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32Sub => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_SUB,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_SUB,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32Mul => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_MUL,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_MUL,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32DivS => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_DIV_S,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_DIV_S,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32DivU => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_DIV_U,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_DIV_U,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32RemS => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_REM_S,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_REM_S,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32RemU => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_REM_U,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_REM_U,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32And => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_AND,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_AND,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32Or => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_OR,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_OR,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32Xor => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_XOR,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_XOR,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32Shl => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_SHL,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_SHL,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32ShrS => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_SHR_S,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_SHR_S,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32ShrU => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_SHR_U,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_SHR_U,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32Rotl => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_ROTL,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_ROTL,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32Rotr => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_ROTR,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_ROTR,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                // Comparison operations
                wasmparser::Operator::I32Eq => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_EQ,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_EQ,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32Ne => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_NE,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_NE,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32LtS => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_LT_S,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_LT_S,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32LtU => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_LT_U,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_LT_U,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32LeS => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_LE_S,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_LE_S,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32LeU => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_LE_U,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_LE_U,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32GtS => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_GT_S,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_GT_S,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32GtU => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_GT_U,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_GT_U,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32GeS => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_GE_S,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_GE_S,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32GeU => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src2 = take_i32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_GE_U,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_GE_U,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                // Unary operations
                wasmparser::Operator::I32Clz => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_CLZ,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_CLZ,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32Ctz => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_CTZ,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_CTZ,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32Popcnt => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_POPCNT,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_POPCNT,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32Eqz => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_EQZ,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_EQZ,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32Extend8S => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_EXTEND8_S,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_EXTEND8_S,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I32Extend16S => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let src1 = take_i32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I32));
                        allocator.pop(&ValueType::NumType(NumType::I32));
                        initial_processed_instrs.push(ProcessedInstr::I32Reg {
                            handler_index: HANDLER_IDX_I32_EXTEND16_S,
                            dst: I32RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I32));
                        (
                            ProcessedInstr::I32Reg {
                                handler_index: HANDLER_IDX_I32_EXTEND16_S,
                                dst: I32RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                // ============================================================================
                // I64 Register-based instructions
                // ============================================================================
                wasmparser::Operator::I64Const { value } => {
                    if can_fold_i64(&mut ops) {
                        pending_operands.push(PendingOperand::I64Const(*value));
                        allocator.push(ValueType::NumType(NumType::I64));
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_CONST,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1: I64RegOperand::Const(*value),
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                // I64 Binary arithmetic operations
                wasmparser::Operator::I64Add => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_ADD,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_ADD,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64Sub => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_SUB,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_SUB,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64Mul => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_MUL,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_MUL,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64DivS => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_DIV_S,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_DIV_S,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64DivU => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_DIV_U,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_DIV_U,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64RemS => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_REM_S,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_REM_S,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64RemU => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_REM_U,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_REM_U,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                // I64 Binary bitwise operations
                wasmparser::Operator::I64And => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_AND,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_AND,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64Or => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_OR,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_OR,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64Xor => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_XOR,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_XOR,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64Shl => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_SHL,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_SHL,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64ShrS => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_SHR_S,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_SHR_S,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64ShrU => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_SHR_U,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_SHR_U,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64Rotl => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_ROTL,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_ROTL,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64Rotr => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_ROTR,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_ROTR,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                // I64 Unary operations
                wasmparser::Operator::I64Clz => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_CLZ,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_CLZ,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64Ctz => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_CTZ,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_CTZ,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64Popcnt => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_POPCNT,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_POPCNT,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64Extend8S => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_EXTEND8_S,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_EXTEND8_S,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64Extend16S => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_EXTEND16_S,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_EXTEND16_S,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::I64Extend32S => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_i64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::I64));
                        allocator.pop(&ValueType::NumType(NumType::I64));
                        initial_processed_instrs.push(ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_EXTEND32_S,
                            dst: I64RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::I64));
                        (
                            ProcessedInstr::I64Reg {
                                handler_index: HANDLER_IDX_I64_EXTEND32_S,
                                dst: I64RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                // I64 Comparison operations (return i32)
                wasmparser::Operator::I64Eqz => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_EQZ,
                            dst: I64RegOperand::Reg(dst.index()),
                            src1,
                            src2: None,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Eq => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_EQ,
                            dst: I64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Ne => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_NE,
                            dst: I64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64LtS => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_LT_S,
                            dst: I64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64LtU => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_LT_U,
                            dst: I64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64GtS => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_GT_S,
                            dst: I64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64GtU => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_GT_U,
                            dst: I64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64LeS => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_LE_S,
                            dst: I64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64LeU => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_LE_U,
                            dst: I64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64GeS => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_GE_S,
                            dst: I64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64GeU => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::I64));
                    let src2 = take_i64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_i64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::I64Reg {
                            handler_index: HANDLER_IDX_I64_GE_U,
                            dst: I64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                // F32 Const
                wasmparser::Operator::F32Const { value } => {
                    if can_fold_f32(&mut ops) {
                        pending_operands
                            .push(PendingOperand::F32Const(f32::from_bits(value.bits())));
                        allocator.push(ValueType::NumType(NumType::F32));
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_CONST,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1: F32RegOperand::Const(f32::from_bits(value.bits())),
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                // F32 Binary arithmetic operations
                wasmparser::Operator::F32Add => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src2 = take_f32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F32));
                        allocator.pop(&ValueType::NumType(NumType::F32));
                        initial_processed_instrs.push(ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_ADD,
                            dst: F32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_ADD,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F32Sub => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src2 = take_f32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F32));
                        allocator.pop(&ValueType::NumType(NumType::F32));
                        initial_processed_instrs.push(ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_SUB,
                            dst: F32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_SUB,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F32Mul => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src2 = take_f32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F32));
                        allocator.pop(&ValueType::NumType(NumType::F32));
                        initial_processed_instrs.push(ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_MUL,
                            dst: F32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_MUL,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F32Div => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src2 = take_f32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F32));
                        allocator.pop(&ValueType::NumType(NumType::F32));
                        initial_processed_instrs.push(ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_DIV,
                            dst: F32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_DIV,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F32Min => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src2 = take_f32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F32));
                        allocator.pop(&ValueType::NumType(NumType::F32));
                        initial_processed_instrs.push(ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_MIN,
                            dst: F32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_MIN,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F32Max => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src2 = take_f32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F32));
                        allocator.pop(&ValueType::NumType(NumType::F32));
                        initial_processed_instrs.push(ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_MAX,
                            dst: F32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_MAX,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F32Copysign => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src2 = take_f32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F32));
                        allocator.pop(&ValueType::NumType(NumType::F32));
                        initial_processed_instrs.push(ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_COPYSIGN,
                            dst: F32RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_COPYSIGN,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                // F32 Unary operations
                wasmparser::Operator::F32Abs => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F32));
                        allocator.pop(&ValueType::NumType(NumType::F32));
                        initial_processed_instrs.push(ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_ABS,
                            dst: F32RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_ABS,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F32Neg => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F32));
                        allocator.pop(&ValueType::NumType(NumType::F32));
                        initial_processed_instrs.push(ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_NEG,
                            dst: F32RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_NEG,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F32Ceil => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F32));
                        allocator.pop(&ValueType::NumType(NumType::F32));
                        initial_processed_instrs.push(ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_CEIL,
                            dst: F32RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_CEIL,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F32Floor => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F32));
                        allocator.pop(&ValueType::NumType(NumType::F32));
                        initial_processed_instrs.push(ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_FLOOR,
                            dst: F32RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_FLOOR,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F32Trunc => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F32));
                        allocator.pop(&ValueType::NumType(NumType::F32));
                        initial_processed_instrs.push(ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_TRUNC,
                            dst: F32RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_TRUNC,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F32Nearest => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F32));
                        allocator.pop(&ValueType::NumType(NumType::F32));
                        initial_processed_instrs.push(ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_NEAREST,
                            dst: F32RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_NEAREST,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F32Sqrt => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f32(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F32));
                        allocator.pop(&ValueType::NumType(NumType::F32));
                        initial_processed_instrs.push(ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_SQRT,
                            dst: F32RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F32));
                        (
                            ProcessedInstr::F32Reg {
                                handler_index: HANDLER_IDX_F32_SQRT,
                                dst: F32RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                // F32 Comparison operations (return i32)
                wasmparser::Operator::F32Eq => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src2 = take_f32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_EQ,
                            dst: F32RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Ne => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src2 = take_f32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_NE,
                            dst: F32RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Lt => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src2 = take_f32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_LT,
                            dst: F32RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Gt => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src2 = take_f32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_GT,
                            dst: F32RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Le => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src2 = take_f32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_LE,
                            dst: F32RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Ge => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F32));
                    let src2 = take_f32_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f32_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F32Reg {
                            handler_index: HANDLER_IDX_F32_GE,
                            dst: F32RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                // F64 Const
                wasmparser::Operator::F64Const { value } => {
                    if can_fold_f64(&mut ops) {
                        pending_operands
                            .push(PendingOperand::F64Const(f64::from_bits(value.bits())));
                        allocator.push(ValueType::NumType(NumType::F64));
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_CONST,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1: F64RegOperand::Const(f64::from_bits(value.bits())),
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                // F64 Binary arithmetic operations
                wasmparser::Operator::F64Add => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src2 = take_f64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F64));
                        allocator.pop(&ValueType::NumType(NumType::F64));
                        initial_processed_instrs.push(ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_ADD,
                            dst: F64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_ADD,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F64Sub => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src2 = take_f64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F64));
                        allocator.pop(&ValueType::NumType(NumType::F64));
                        initial_processed_instrs.push(ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_SUB,
                            dst: F64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_SUB,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F64Mul => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src2 = take_f64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F64));
                        allocator.pop(&ValueType::NumType(NumType::F64));
                        initial_processed_instrs.push(ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_MUL,
                            dst: F64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_MUL,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F64Div => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src2 = take_f64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F64));
                        allocator.pop(&ValueType::NumType(NumType::F64));
                        initial_processed_instrs.push(ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_DIV,
                            dst: F64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_DIV,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F64Min => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src2 = take_f64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F64));
                        allocator.pop(&ValueType::NumType(NumType::F64));
                        initial_processed_instrs.push(ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_MIN,
                            dst: F64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_MIN,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F64Max => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src2 = take_f64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F64));
                        allocator.pop(&ValueType::NumType(NumType::F64));
                        initial_processed_instrs.push(ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_MAX,
                            dst: F64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_MAX,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F64Copysign => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src2 = take_f64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F64));
                        allocator.pop(&ValueType::NumType(NumType::F64));
                        initial_processed_instrs.push(ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_COPYSIGN,
                            dst: F64RegOperand::Param(local_idx),
                            src1,
                            src2: Some(src2),
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_COPYSIGN,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1,
                                src2: Some(src2),
                            },
                            None,
                        )
                    }
                }
                // F64 Unary operations
                wasmparser::Operator::F64Abs => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F64));
                        allocator.pop(&ValueType::NumType(NumType::F64));
                        initial_processed_instrs.push(ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_ABS,
                            dst: F64RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_ABS,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F64Neg => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F64));
                        allocator.pop(&ValueType::NumType(NumType::F64));
                        initial_processed_instrs.push(ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_NEG,
                            dst: F64RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_NEG,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F64Ceil => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F64));
                        allocator.pop(&ValueType::NumType(NumType::F64));
                        initial_processed_instrs.push(ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_CEIL,
                            dst: F64RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_CEIL,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F64Floor => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F64));
                        allocator.pop(&ValueType::NumType(NumType::F64));
                        initial_processed_instrs.push(ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_FLOOR,
                            dst: F64RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_FLOOR,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F64Trunc => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F64));
                        allocator.pop(&ValueType::NumType(NumType::F64));
                        initial_processed_instrs.push(ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_TRUNC,
                            dst: F64RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_TRUNC,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F64Nearest => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F64));
                        allocator.pop(&ValueType::NumType(NumType::F64));
                        initial_processed_instrs.push(ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_NEAREST,
                            dst: F64RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_NEAREST,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                wasmparser::Operator::F64Sqrt => {
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    if let Some(local_idx) = try_fold_dst_f64(&mut ops, param_types, locals) {
                        let _ = ops.next();
                        let _dst = allocator.push(ValueType::NumType(NumType::F64));
                        allocator.pop(&ValueType::NumType(NumType::F64));
                        initial_processed_instrs.push(ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_SQRT,
                            dst: F64RegOperand::Param(local_idx),
                            src1,
                            src2: None,
                        });
                        current_processed_pc += 1;
                        (ProcessedInstr::NopReg, None)
                    } else {
                        let dst = allocator.push(ValueType::NumType(NumType::F64));
                        (
                            ProcessedInstr::F64Reg {
                                handler_index: HANDLER_IDX_F64_SQRT,
                                dst: F64RegOperand::Reg(dst.index()),
                                src1,
                                src2: None,
                            },
                            None,
                        )
                    }
                }
                // F64 Comparison operations (return i32)
                wasmparser::Operator::F64Eq => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src2 = take_f64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_EQ,
                            dst: F64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Ne => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src2 = take_f64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_NE,
                            dst: F64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Lt => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src2 = take_f64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_LT,
                            dst: F64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Gt => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src2 = take_f64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_GT,
                            dst: F64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Le => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src2 = take_f64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_LE,
                            dst: F64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Ge => {
                    let src2_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src1_reg = allocator.pop(&ValueType::NumType(NumType::F64));
                    let src2 = take_f64_operand(&mut pending_operands, src2_reg.index());
                    let src1 = take_f64_operand(&mut pending_operands, src1_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::F64Reg {
                            handler_index: HANDLER_IDX_F64_GE,
                            dst: F64RegOperand::Reg(dst.index()),
                            src1,
                            src2: Some(src2),
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

                    // Get the top N registers based on result types
                    let source_regs = allocator.peek_regs_for_types(&result_type_vec);

                    // Get target_result_regs from ControlBlockInfo BEFORE restoring state
                    let target_result_regs = if let Some(block_info) = control_info_stack.last() {
                        block_info.result_regs.clone()
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
                    let mut stack_to_regs = Vec::new();
                    if let Some(saved_state) = allocator_state_stack.pop() {
                        allocator.restore_state(&saved_state);
                        // Push result types to allocator (at the restored depth)
                        for vtype in block_result_types_to_push {
                            let reg = allocator.push(vtype);
                            stack_to_regs.push(reg);
                        }
                    }

                    // Use EndReg for register-based execution
                    let instr = ProcessedInstr::EndReg {
                        source_regs,
                        target_result_regs,
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

                    // Calculate result_regs at block entry depth
                    let result_types = get_block_result_types(&blockty, module);
                    let result_regs: Vec<Reg> = {
                        let mut state = saved_state;
                        result_types
                            .iter()
                            .map(|vtype| state.next_reg_for_type(vtype))
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
                        result_regs,
                        param_regs: vec![],
                    });

                    let instr = ProcessedInstr::BlockReg {
                        arity,
                        param_count,
                        is_loop,
                    };
                    (instr, None)
                }
                wasmparser::Operator::If { blockty } => {
                    let cond_reg = allocator.pop(&ValueType::NumType(NumType::I32));

                    let param_types = get_block_param_types(&blockty, module);

                    for vtype in param_types.iter().rev() {
                        allocator.pop(&vtype);
                    }

                    let saved_state = allocator.save_state();
                    allocator_state_stack.push(saved_state.clone());

                    // Calculate result_regs at block entry depth
                    let result_types = get_block_result_types(&blockty, module);
                    let result_regs: Vec<Reg> = {
                        let mut state = saved_state;
                        result_types
                            .iter()
                            .map(|vtype| state.next_reg_for_type(vtype))
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
                        result_regs,
                        param_regs: vec![],
                    });

                    let instr = ProcessedInstr::IfReg {
                        arity,
                        cond_reg,
                        else_target_ip: usize::MAX, // Will be fixed up
                        has_else: false,            // Will be updated during fixup
                    };
                    let fixup = Some(FixupInfo {
                        pc: current_processed_pc,
                        original_wasm_depth: 0,
                        is_if_false_jump: true,
                        is_else_jump: false,
                        source_regs: vec![],
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

                    // Generate JumpReg (target_ip will be fixed up later)
                    let instr = ProcessedInstr::JumpReg {
                        target_ip: usize::MAX, // Will be fixed up
                    };
                    let fixup = Some(FixupInfo {
                        pc: current_processed_pc,
                        original_wasm_depth: 0,
                        is_if_false_jump: false,
                        is_else_jump: true,
                        source_regs: vec![],
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

                        // Get the top N registers for params based on types
                        let param_regs = allocator.peek_regs_for_types(&param_types);

                        for param_type in param_types.iter().rev() {
                            allocator.pop(&param_type);
                        }

                        let result_reg = if let Some(result_type) = result_types.first() {
                            Some(allocator.push(result_type.clone()))
                        } else {
                            None
                        };

                        (
                            ProcessedInstr::CallWasiReg {
                                wasi_func_type: wasi_type,
                                param_regs,
                                result_reg,
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
                            // No params/results - still use CallReg with empty registers
                            (
                                ProcessedInstr::CallReg {
                                    func_idx: FuncIdx(*function_index),
                                    param_regs: vec![],
                                    result_regs: vec![],
                                },
                                None,
                            )
                        } else {
                            // Get param registers before popping
                            let mut param_regs = Vec::new();
                            for param_type in param_types.iter().rev() {
                                if let Some(reg) = allocator.peek(param_type) {
                                    param_regs.insert(0, reg);
                                }
                                allocator.pop(param_type);
                            }

                            // Push result types to allocator and collect result_regs
                            let mut result_regs = Vec::new();
                            for result_type in &result_types {
                                let reg = allocator.push(result_type.clone());
                                result_regs.push(reg);
                            }

                            // Use CallReg for register-based execution
                            let instr = ProcessedInstr::CallReg {
                                func_idx: FuncIdx(*function_index),
                                param_regs,
                                result_regs,
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

                    // Pop index_reg first (i32) - this is the table index
                    let index_reg = allocator.peek(&ValueType::NumType(NumType::I32)).unwrap();
                    allocator.pop(&ValueType::NumType(NumType::I32));

                    // Get param registers (in order, from bottom to top)
                    // We need to collect the param registers before popping them
                    let param_count = param_types.len();
                    let mut param_regs = Vec::with_capacity(param_count);
                    for param_type in param_types.iter() {
                        if let Some(reg) = allocator.peek(param_type) {
                            param_regs.push(reg);
                        }
                    }
                    // Actually, we need to get the registers that will be consumed
                    // Recalculate: peek each param type from the current state
                    param_regs.clear();
                    for param_type in param_types.iter().rev() {
                        if let Some(reg) = allocator.peek(param_type) {
                            param_regs.insert(0, reg);
                        }
                        allocator.pop(param_type);
                    }

                    // Push result types to allocator and collect result_regs
                    let mut result_regs = Vec::new();
                    for result_type in &result_types_vec {
                        let reg = allocator.push(result_type.clone());
                        result_regs.push(reg);
                    }

                    // Use CallIndirectReg for register-based execution
                    let instr = ProcessedInstr::CallIndirectReg {
                        type_idx: TypeIdx(*type_index),
                        table_idx: TableIdx(*table_index),
                        index_reg,
                        param_regs,
                        result_regs,
                    };
                    (instr, None)
                }

                wasmparser::Operator::Br { relative_depth } => {
                    // Compute source and target registers for branch
                    let (source_regs, target_result_regs) = compute_branch_regs(
                        &control_info_stack,
                        *relative_depth as usize,
                        reg_allocator.as_ref(),
                    );

                    let instr = ProcessedInstr::BrReg {
                        relative_depth: *relative_depth,
                        target_ip: usize::MAX, // Will be set by fixup
                        source_regs: source_regs.clone(),
                        target_result_regs,
                    };
                    let fixup = FixupInfo {
                        pc: current_processed_pc,
                        original_wasm_depth: *relative_depth as usize,
                        is_if_false_jump: false,
                        is_else_jump: false,
                        source_regs,
                    };
                    (instr, Some(fixup))
                }
                wasmparser::Operator::BrIf { relative_depth } => {
                    // Pop condition register
                    // In unreachable code after br/return, allocator may be empty
                    let cond_reg = allocator
                        .peek(&ValueType::NumType(NumType::I32))
                        .unwrap_or(Reg::I32(0)); // Use dummy register in unreachable code
                    allocator.pop(&ValueType::NumType(NumType::I32));

                    // Compute source and target registers for branch
                    let (source_regs, target_result_regs) = compute_branch_regs(
                        &control_info_stack,
                        *relative_depth as usize,
                        reg_allocator.as_ref(),
                    );

                    let instr = ProcessedInstr::BrIfReg {
                        relative_depth: *relative_depth,
                        target_ip: usize::MAX, // Will be set by fixup
                        cond_reg,
                        source_regs: source_regs.clone(),
                        target_result_regs,
                    };
                    let fixup = FixupInfo {
                        pc: current_processed_pc,
                        original_wasm_depth: *relative_depth as usize,
                        is_if_false_jump: false,
                        is_else_jump: false,
                        source_regs,
                    };
                    (instr, Some(fixup))
                }
                wasmparser::Operator::BrTable { ref targets } => {
                    // Pop index register
                    // In unreachable code after br/return, allocator may be empty
                    let index_reg = allocator
                        .peek(&ValueType::NumType(NumType::I32))
                        .unwrap_or(Reg::I32(0)); // Use dummy register in unreachable code
                    allocator.pop(&ValueType::NumType(NumType::I32));

                    // Collect target depths with placeholder target_ip and compute target_result_regs for each
                    let target_depths: Vec<u32> =
                        targets.targets().collect::<Result<Vec<_>, _>>()?;

                    let mut table_targets: Vec<(u32, usize, Vec<Reg>)> =
                        Vec::with_capacity(target_depths.len());
                    for depth in target_depths.iter() {
                        let (_, target_result_regs) = compute_branch_regs(
                            &control_info_stack,
                            *depth as usize,
                            reg_allocator.as_ref(),
                        );
                        table_targets.push((*depth, usize::MAX, target_result_regs));
                        // target_ip will be set by fixup
                    }

                    // Compute source and target registers for default target
                    let (source_regs, default_result_regs) = compute_branch_regs(
                        &control_info_stack,
                        targets.default() as usize,
                        reg_allocator.as_ref(),
                    );
                    let default_target = (targets.default(), usize::MAX, default_result_regs); // target_ip will be set by fixup

                    let instr = ProcessedInstr::BrTableReg {
                        targets: table_targets.clone(),
                        default_target,
                        index_reg,
                        source_regs: source_regs.clone(),
                    };

                    // Create fixups for each target and default
                    // The fixup system will handle BrTableReg specially
                    let fixup = FixupInfo {
                        pc: current_processed_pc,
                        original_wasm_depth: targets.default() as usize,
                        is_if_false_jump: false,
                        is_else_jump: false,
                        source_regs,
                    };
                    (instr, Some(fixup))
                }
                wasmparser::Operator::Return => {
                    // Get result registers based on function result types
                    let result_regs = allocator.peek_regs_for_types(result_types);
                    for result_type in result_types.iter().rev() {
                        allocator.pop(result_type);
                    }

                    let instr = ProcessedInstr::ReturnReg { result_regs };
                    (instr, None)
                }
                wasmparser::Operator::Nop => (ProcessedInstr::NopReg, None),
                wasmparser::Operator::Unreachable => (ProcessedInstr::UnreachableReg, None),
                wasmparser::Operator::Drop => {
                    // Pop from type_stack to keep it in sync, but no runtime operation needed
                    allocator.pop_any();
                    (ProcessedInstr::NopReg, None)
                }
                // Conversion instructions - use ConversionReg
                wasmparser::Operator::I64ExtendI32S => {
                    let src = allocator.pop(&ValueType::NumType(NumType::I32));
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
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
                        ProcessedInstr::ConversionReg {
                            handler_index: HANDLER_IDX_F64_REINTERPRET_I64,
                            dst,
                            src,
                        },
                        None,
                    )
                }
                // Memory Load instructions
                wasmparser::Operator::I32Load { memarg } => {
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryLoadReg {
                            handler_index: HANDLER_IDX_I32_LOAD,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Load { memarg } => {
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::MemoryLoadReg {
                            handler_index: HANDLER_IDX_I64_LOAD,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F32Load { memarg } => {
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::F32));
                    (
                        ProcessedInstr::MemoryLoadReg {
                            handler_index: HANDLER_IDX_F32_LOAD,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::F64Load { memarg } => {
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::F64));
                    (
                        ProcessedInstr::MemoryLoadReg {
                            handler_index: HANDLER_IDX_F64_LOAD,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Load8S { memarg } => {
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryLoadReg {
                            handler_index: HANDLER_IDX_I32_LOAD8_S,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Load8U { memarg } => {
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryLoadReg {
                            handler_index: HANDLER_IDX_I32_LOAD8_U,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Load16S { memarg } => {
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryLoadReg {
                            handler_index: HANDLER_IDX_I32_LOAD16_S,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I32Load16U { memarg } => {
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I32));
                    (
                        ProcessedInstr::MemoryLoadReg {
                            handler_index: HANDLER_IDX_I32_LOAD16_U,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Load8S { memarg } => {
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::MemoryLoadReg {
                            handler_index: HANDLER_IDX_I64_LOAD8_S,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Load8U { memarg } => {
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::MemoryLoadReg {
                            handler_index: HANDLER_IDX_I64_LOAD8_U,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Load16S { memarg } => {
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::MemoryLoadReg {
                            handler_index: HANDLER_IDX_I64_LOAD16_S,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Load16U { memarg } => {
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::MemoryLoadReg {
                            handler_index: HANDLER_IDX_I64_LOAD16_U,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Load32S { memarg } => {
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::MemoryLoadReg {
                            handler_index: HANDLER_IDX_I64_LOAD32_S,
                            dst,
                            addr,
                            offset: memarg.offset,
                        },
                        None,
                    )
                }
                wasmparser::Operator::I64Load32U { memarg } => {
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    let dst = allocator.push(ValueType::NumType(NumType::I64));
                    (
                        ProcessedInstr::MemoryLoadReg {
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
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    (
                        ProcessedInstr::MemoryStoreReg {
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
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    (
                        ProcessedInstr::MemoryStoreReg {
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
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    (
                        ProcessedInstr::MemoryStoreReg {
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
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    (
                        ProcessedInstr::MemoryStoreReg {
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
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    (
                        ProcessedInstr::MemoryStoreReg {
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
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    (
                        ProcessedInstr::MemoryStoreReg {
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
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    (
                        ProcessedInstr::MemoryStoreReg {
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
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    (
                        ProcessedInstr::MemoryStoreReg {
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
                    let addr_reg = allocator.pop(&ValueType::NumType(NumType::I32));
                    let addr = take_i32_operand(&mut pending_operands, addr_reg.index());
                    (
                        ProcessedInstr::MemoryStoreReg {
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
                        ProcessedInstr::MemoryOpsReg {
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
                        ProcessedInstr::MemoryOpsReg {
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
                        ProcessedInstr::MemoryOpsReg {
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
                        ProcessedInstr::MemoryOpsReg {
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
                        ProcessedInstr::MemoryOpsReg {
                            handler_index: HANDLER_IDX_MEMORY_FILL,
                            dst: None,
                            args: vec![dest, val, size],
                            data_index: 0,
                        },
                        None,
                    )
                }
                wasmparser::Operator::DataDrop { data_index } => (
                    ProcessedInstr::DataDropReg {
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
                        ProcessedInstr::SelectReg {
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
                        ProcessedInstr::SelectReg {
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
                        ProcessedInstr::TableRefReg {
                            handler_index: HANDLER_IDX_REF_NULL_REG,
                            table_idx: 0,
                            regs: [dst.index(), 0, 0],
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
                        ProcessedInstr::TableRefReg {
                            handler_index: HANDLER_IDX_REF_IS_NULL_REG,
                            table_idx: 0,
                            regs: [dst.index(), src.index(), 0],
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
                        ProcessedInstr::TableRefReg {
                            handler_index: HANDLER_IDX_TABLE_GET_REG,
                            table_idx: *table,
                            regs: [dst.index(), idx.index(), 0],
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
                        ProcessedInstr::TableRefReg {
                            handler_index: HANDLER_IDX_TABLE_SET_REG,
                            table_idx: *table,
                            regs: [idx.index(), val.index(), 0],
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
                        ProcessedInstr::TableRefReg {
                            handler_index: HANDLER_IDX_TABLE_FILL_REG,
                            table_idx: *table,
                            regs: [i.index(), val.index(), n.index()],
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
            panic!("Register allocator is required");
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

        // All instructions are now register-based
        initial_processed_instrs.push(processed_instr_template);
        if let Some(fixup_info) = fixup_info_opt {
            initial_fixups.push(fixup_info);
        }

        // Update control_info_stack and block_result_regs_map
        match op {
            wasmparser::Operator::Block { .. } => {
                // Register for BrTable resolution (always needed)
                if let Some(block_info) = control_info_stack.last() {
                    block_result_regs_map.insert(
                        current_processed_pc,
                        (block_info.result_regs.clone(), false),
                    );
                }
            }
            wasmparser::Operator::Loop { .. } => {
                // Register for BrTable resolution (always needed)
                // For loops, register param_regs (used when branching to loop)
                if let Some(block_info) = control_info_stack.last() {
                    block_result_regs_map
                        .insert(current_processed_pc, (block_info.param_regs.clone(), true));
                }
            }
            wasmparser::Operator::If { .. } => {
                // Register for BrTable resolution (always needed)
                if let Some(block_info) = control_info_stack.last() {
                    block_result_regs_map.insert(
                        current_processed_pc,
                        (block_info.result_regs.clone(), false),
                    );
                }
            }
            wasmparser::Operator::End => {
                // Register mode End already popped in its match arm above
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

    // Get result register before finalizing (the top of stack after all instructions)
    // Use the function's result type to peek at the correct register type
    let result_reg = reg_allocator.as_ref().and_then(|alloc| {
        if let Some(result_type) = result_types.first() {
            // Peek at the current stack top for the result type
            alloc.peek(result_type)
        } else {
            None
        }
    });

    // Finalize register allocation if in register mode
    let reg_allocation = reg_allocator.map(|alloc| alloc.finalize());

    Ok((
        initial_processed_instrs,
        initial_fixups,
        block_end_map,
        if_else_map,
        block_type_map,
        reg_allocation,
        result_reg,
        block_result_regs_map,
    ))
}

/// Compute source_regs and target_result_regs for branch instructions
/// source_regs: current stack top registers that will be copied
/// target_result_regs: where to copy them (the target block's result registers, or param_regs for loops)
fn compute_branch_regs(
    control_info_stack: &[ControlBlockInfo],
    relative_depth: usize,
    reg_allocator: Option<&RegAllocator>,
) -> (Vec<Reg>, Vec<Reg>) {
    // Get target block from control_info_stack
    let stack_len = control_info_stack.len();
    if stack_len == 0 || relative_depth >= stack_len {
        return (vec![], vec![]);
    }

    let target_idx = stack_len - 1 - relative_depth;
    let target_block = &control_info_stack[target_idx];
    // For loops, use param_regs (branch provides parameters)
    // For blocks/if, use result_regs (branch provides results)
    let target_result_regs = if target_block.is_loop {
        target_block.param_regs.clone()
    } else {
        target_block.result_regs.clone()
    };

    // Compute source_regs from current allocator state
    let source_regs = if let Some(allocator) = reg_allocator {
        // Get the registers at stack top matching result types
        let result_count = target_result_regs.len();
        if result_count == 0 {
            vec![]
        } else {
            // Get current state and compute source registers
            let state = allocator.save_state();
            target_result_regs
                .iter()
                .enumerate()
                .map(|(i, target_reg)| {
                    // Source register is at current_depth - result_count + i
                    match target_reg {
                        Reg::I32(_) => {
                            let src_idx = state.i32_depth.saturating_sub(result_count - i);
                            Reg::I32(src_idx as u16)
                        }
                        Reg::I64(_) => {
                            let src_idx = state.i64_depth.saturating_sub(result_count - i);
                            Reg::I64(src_idx as u16)
                        }
                        Reg::F32(_) => {
                            let src_idx = state.f32_depth.saturating_sub(result_count - i);
                            Reg::F32(src_idx as u16)
                        }
                        Reg::F64(_) => {
                            let src_idx = state.f64_depth.saturating_sub(result_count - i);
                            Reg::F64(src_idx as u16)
                        }
                        Reg::Ref(_) => {
                            let src_idx = state.ref_depth.saturating_sub(result_count - i);
                            Reg::Ref(src_idx as u16)
                        }
                        Reg::V128(_) => {
                            let src_idx = state.v128_depth.saturating_sub(result_count - i);
                            Reg::V128(src_idx as u16)
                        }
                    }
                })
                .collect()
        }
    } else {
        vec![]
    };

    (source_regs, target_result_regs)
}

/// Parses a WebAssembly binary file and populates the module structure.
///
/// This is the main entry point for loading a WebAssembly module. It reads
/// the binary file, parses all sections using wasmparser, and preprocesses
/// instructions for efficient interpretation.
///
/// # Arguments
///
/// * `module` - The module structure to populate
/// * `path` - Path to the WebAssembly binary file
/// * `enable_superinstructions` - Deprecated, no longer used
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
