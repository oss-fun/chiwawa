//! v2 dispatcher IR types.
//!
//! Defines the `Handler` function pointer type and `Outcome` enum used by
//! both the loop-style and tail-call dispatchers, plus the `ProcessedInstr`
//! representation produced by the parser.

use crate::execution::handlers::{
    HANDLER_IDX_BLOCK, HANDLER_IDX_BR, HANDLER_IDX_BR_IF, HANDLER_IDX_BR_TABLE, HANDLER_IDX_CALL,
    HANDLER_IDX_CALL_INDIRECT, HANDLER_IDX_CALL_WASI, HANDLER_IDX_DATA_DROP, HANDLER_IDX_ELSE,
    HANDLER_IDX_END, HANDLER_IDX_IF, HANDLER_IDX_LOOP, HANDLER_IDX_NOP, HANDLER_IDX_RETURN,
    HANDLER_IDX_UNREACHABLE,
};
use crate::execution::regs::Reg;
use crate::execution::state::VmState;
use crate::structure::module::WasiFuncType;
use crate::structure::types::{FuncIdx, RefType, TableIdx, TypeIdx};
use serde::{Deserialize, Serialize};

/// Type alias for boxed register slice (ProcessedInstr use).
/// 16 bytes vs Vec's 24 bytes, no capacity overhead.
pub type RegSlice = Box<[Reg]>;

/// Outcome of a handler invocation.
///
/// In the loop-style dispatcher, handlers return `Continue` to indicate the
/// outer loop should fetch the next instruction. In the tail-call dispatcher,
/// handlers tail-call the next handler directly and `Continue` is normally
/// not seen by the dispatcher driver.
///
/// Trap conditions store the error in `state.trap`. Function-level yields
/// (call/return/wasi) store the `ModuleLevelInstr` in `state.yielded`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Loop dispatcher: fetch next instruction. TCO dispatcher: chain continues.
    Continue = 0,
    /// Function body ended naturally (reached `end` at function level).
    Halt = 1,
    /// Trap occurred. Details in `state.trap`.
    Trap = 2,
    /// Yielding to runtime for frame transition. Details in `state.yielded`.
    Yield = 3,
}

/// All v2 dispatcher handlers share this signature so the function pointer
/// type matches identically — required for `return_call_indirect` in the
/// TCO dispatcher.
pub type Handler = fn(&mut VmState) -> Outcome;

/// Branch target descriptor used as a temporary holder during the parser's
/// BrTable resolution pass before being flattened into `BrTableReg`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Operand {
    LabelIdx {
        target_ip: usize,
        arity: usize,
        original_wasm_depth: usize,
        is_loop: bool,
        source_regs: Vec<Reg>,
        target_result_regs: Vec<Reg>,
        cond_reg: Option<Reg>,
    },
}

/// Destination that can be either a register or a local variable.
/// Used for instructions where dst folding is applied.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum RegOrLocal {
    Reg(u16),
    Local(u16),
}

/// Register-based operand for I32 operations
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum I32RegOperand {
    Reg(u16),
    Const(i32),
    Param(u16),
}

/// Register-based operand for I64 operations.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum I64RegOperand {
    Reg(u16),
    Const(i64),
    Param(u16),
}

/// Register-based operand for F32 operations.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum F32RegOperand {
    Reg(u16),
    Const(f32),
    Param(u16),
}

/// Register-based operand for F64 operations.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum F64RegOperand {
    Reg(u16),
    Const(f64),
    Param(u16),
}

/// Processed instruction for DTC execution with pre-resolved operands.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProcessedInstr {
    I32Reg {
        handler_index: usize,
        dst: I32RegOperand,
        src1: I32RegOperand,
        src2: Option<I32RegOperand>,
    },
    I64Reg {
        handler_index: usize,
        dst: I64RegOperand,
        src1: I64RegOperand,
        src2: Option<I64RegOperand>,
    },
    F32Reg {
        handler_index: usize,
        dst: F32RegOperand,
        src1: F32RegOperand,
        src2: Option<F32RegOperand>,
    },
    F64Reg {
        handler_index: usize,
        dst: F64RegOperand,
        src1: F64RegOperand,
        src2: Option<F64RegOperand>,
    },
    ConversionReg {
        handler_index: usize,
        dst: RegOrLocal,
        src: Reg,
    },
    MemoryLoadReg {
        handler_index: usize,
        dst: RegOrLocal,
        addr: I32RegOperand,
        offset: u64,
    },
    MemoryStoreReg {
        handler_index: usize,
        addr: I32RegOperand,
        value: Reg,
        offset: u64,
    },
    MemoryOpsReg {
        handler_index: usize,
        dst: Option<Reg>,
        args: RegSlice,
        data_index: u32,
    },
    SelectReg {
        handler_index: usize,
        dst: Reg,
        val1: Reg,
        val2: Reg,
        cond: Reg,
    },
    GlobalGetReg {
        handler_index: usize,
        dst: RegOrLocal,
        global_index: u32,
    },
    GlobalSetReg {
        handler_index: usize,
        src: RegOrLocal,
        global_index: u32,
    },
    DataDropReg {
        data_index: u32,
    },
    RefLocalReg {
        handler_index: usize,
        dst: u16,
        src: u16,
        local_idx: u16,
    },
    TableRefReg {
        handler_index: usize,
        table_idx: u32,
        regs: [u16; 3],
        ref_type: RefType,
    },
    CallWasiReg {
        wasi_func_type: WasiFuncType,
        param_regs: RegSlice,
        result_reg: Option<Reg>,
    },
    CallIndirectReg {
        type_idx: TypeIdx,
        table_idx: TableIdx,
        index_reg: Reg,
        param_regs: RegSlice,
        result_regs: RegSlice,
    },
    CallReg {
        func_idx: FuncIdx,
        param_regs: RegSlice,
        result_regs: RegSlice,
    },
    ReturnReg {
        result_regs: RegSlice,
    },
    JumpReg {
        target_ip: usize,
    },
    BlockReg {
        arity: usize,
        param_count: usize,
        is_loop: bool,
    },
    IfReg {
        arity: usize,
        cond_reg: Reg,
        else_target_ip: usize,
        has_else: bool,
    },
    EndReg {
        source_regs: RegSlice,
        target_result_regs: RegSlice,
    },
    BrReg {
        relative_depth: u32,
        target_ip: usize,
        source_regs: RegSlice,
        target_result_regs: RegSlice,
    },
    BrIfReg {
        relative_depth: u32,
        target_ip: usize,
        cond_reg: Reg,
        source_regs: RegSlice,
        target_result_regs: RegSlice,
    },
    BrTableReg {
        targets: Vec<(u32, usize, RegSlice)>,
        default_target: (u32, usize, RegSlice),
        index_reg: Reg,
        source_regs: RegSlice,
    },
    NopReg,
    UnreachableReg,
}

impl ProcessedInstr {
    #[inline(always)]
    pub fn handler_index(&self) -> usize {
        match self {
            ProcessedInstr::I32Reg { handler_index, .. } => *handler_index,
            ProcessedInstr::I64Reg { handler_index, .. } => *handler_index,
            ProcessedInstr::F32Reg { handler_index, .. } => *handler_index,
            ProcessedInstr::F64Reg { handler_index, .. } => *handler_index,
            ProcessedInstr::ConversionReg { handler_index, .. } => *handler_index,
            ProcessedInstr::MemoryLoadReg { handler_index, .. } => *handler_index,
            ProcessedInstr::MemoryStoreReg { handler_index, .. } => *handler_index,
            ProcessedInstr::MemoryOpsReg { handler_index, .. } => *handler_index,
            ProcessedInstr::SelectReg { handler_index, .. } => *handler_index,
            ProcessedInstr::GlobalGetReg { handler_index, .. } => *handler_index,
            ProcessedInstr::GlobalSetReg { handler_index, .. } => *handler_index,
            ProcessedInstr::DataDropReg { .. } => HANDLER_IDX_DATA_DROP,
            ProcessedInstr::RefLocalReg { handler_index, .. } => *handler_index,
            ProcessedInstr::TableRefReg { handler_index, .. } => *handler_index,
            ProcessedInstr::CallWasiReg { .. } => HANDLER_IDX_CALL_WASI,
            ProcessedInstr::CallIndirectReg { .. } => HANDLER_IDX_CALL_INDIRECT,
            ProcessedInstr::CallReg { .. } => HANDLER_IDX_CALL,
            ProcessedInstr::ReturnReg { .. } => HANDLER_IDX_RETURN,
            ProcessedInstr::JumpReg { .. } => HANDLER_IDX_ELSE,
            ProcessedInstr::BlockReg { is_loop: false, .. } => HANDLER_IDX_BLOCK,
            ProcessedInstr::BlockReg { is_loop: true, .. } => HANDLER_IDX_LOOP,
            ProcessedInstr::IfReg { .. } => HANDLER_IDX_IF,
            ProcessedInstr::EndReg { .. } => HANDLER_IDX_END,
            ProcessedInstr::BrReg { .. } => HANDLER_IDX_BR,
            ProcessedInstr::BrIfReg { .. } => HANDLER_IDX_BR_IF,
            ProcessedInstr::BrTableReg { .. } => HANDLER_IDX_BR_TABLE,
            ProcessedInstr::NopReg => HANDLER_IDX_NOP,
            ProcessedInstr::UnreachableReg => HANDLER_IDX_UNREACHABLE,
        }
    }
}
