//! Virtual machine state and execution loop.
//!
//! This module contains the core execution machinery including the frame stack,
//! register file, and instruction dispatch handlers.
//!
//! ## Execution State
//!
//! The VM maintains execution state through:
//! - `Stacks`: Runtime execution state
//! - `Frame`: Individual call frame with program counter and registers
//! - `RegFile`: Type-specialized register storage
//!
//! ## Handler Dispatch
//!
//! Instructions are dispatched through a handler table where each instruction
//! type maps to a specialized handler function. Branch targets are pre-resolved
//! during parsing to absolute program counter values.

use super::value::*;
use crate::error::RuntimeError;
use crate::execution::{
    func::*,
    mem::MemAddr,
    migration,
    module::*,
    regs::{Reg, RegFile},
};
use crate::structure::module::WasiFuncType;
use crate::structure::{instructions::*, types::*};
use arrayvec::ArrayVec;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::rc::{Rc, Weak};

/// Type alias for boxed register slice (ProcessedInstr use).
/// 16 bytes vs Vec's 24 bytes, no capacity overhead.
pub type RegSlice = Box<[Reg]>;

/// Instruction operand variants for DTC execution.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Operand {
    None,
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    LocalIdx(LocalIdx),
    LocalIdxI32(LocalIdx, i32),
    LocalIdxI64(LocalIdx, i64),
    LocalIdxF32(LocalIdx, f32),
    LocalIdxF64(LocalIdx, f64),
    MemArgI32(i32, Memarg),
    MemArgI64(i64, Memarg),
    GlobalIdx(GlobalIdx),
    FuncIdx(FuncIdx),
    TableIdx(TableIdx),
    TypeIdx(TypeIdx),
    RefType(RefType),
    LabelIdx {
        target_ip: usize,
        arity: usize,
        original_wasm_depth: usize,
        is_loop: bool,
        source_regs: Vec<Reg>,
        target_result_regs: Vec<Reg>,
        cond_reg: Option<Reg>, // For br_if: register containing condition value
    },
    MemArg(Memarg),
    BrTable {
        targets: Vec<Operand>,
        default: Box<Operand>,
        index_reg: Option<Reg>, // Reg containing table index
    },
    CallIndirect {
        type_idx: TypeIdx,
        table_idx: TableIdx,
        index_reg: Option<Reg>, // Reg containing table element index
    },
    Block {
        arity: usize,
        param_count: usize,
        is_loop: bool,
    },
}

/// Destination that can be either a register or a local variable.
/// Used for instructions where dst folding is applied.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum RegOrLocal {
    Reg(u16),   // Write to register
    Local(u16), // Write to local variable
}

/// Register-based operand for I32 operations
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum I32RegOperand {
    Reg(u16),   // Read from register
    Const(i32), // Constant value
    Param(u16), // Read from parameter/local
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
    /// Register-based I32 instruction
    I32Reg {
        handler_index: usize,
        dst: I32RegOperand,          // Destination: Reg(idx) or Param(local_idx)
        src1: I32RegOperand,         // First operand
        src2: Option<I32RegOperand>, // Second operand (None for unary ops)
    },
    /// Register-based I64 instruction
    I64Reg {
        handler_index: usize,
        dst: I64RegOperand, // Destination (Reg for register, Param for local variable)
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
        dst: RegOrLocal, // Destination register or local (type determined by output type)
        src: Reg,        // Source register (type determined by input type)
    },
    MemoryLoadReg {
        handler_index: usize,
        dst: RegOrLocal,     // Destination (register or local for dst folding)
        addr: I32RegOperand, // Address operand (can be folded)
        offset: u64,         // Memory offset
    },
    MemoryStoreReg {
        handler_index: usize,
        addr: I32RegOperand, // Address operand (can be folded)
        value: Reg,          // Value register to store
        offset: u64,         // Memory offset
    },

    MemoryOpsReg {
        handler_index: usize,
        dst: Option<Reg>, // Destination register (for size/grow results)
        args: RegSlice,   // Argument registers (varies by operation)
        data_index: u32,  // Data segment index (for memory.init)
    },

    SelectReg {
        handler_index: usize,
        dst: Reg,  // Destination register
        val1: Reg, // First value (selected when cond != 0)
        val2: Reg, // Second value (selected when cond == 0)
        cond: Reg, // Condition register (always I32)
    },

    GlobalGetReg {
        handler_index: usize,
        dst: RegOrLocal,   // Destination register or local
        global_index: u32, // Global variable index
    },

    GlobalSetReg {
        handler_index: usize,
        src: RegOrLocal,   // Source register or local
        global_index: u32, // Global variable index
    },

    DataDropReg {
        data_index: u32, // Data segment index to drop
    },

    /// Register-based ref local operations
    RefLocalReg {
        handler_index: usize,
        dst: u16,       // Destination register index (for get) or unused (for set)
        src: u16,       // Source register index (for set) or unused (for get)
        local_idx: u16, // Local variable index
    },

    TableRefReg {
        handler_index: usize,
        table_idx: u32,
        regs: [u16; 3],    // Operand registers (usage depends on handler)
        ref_type: RefType, // Used for RefNull
    },
    CallWasiReg {
        wasi_func_type: WasiFuncType,
        param_regs: RegSlice,    // Parameter registers
        result_reg: Option<Reg>, // Result register (most WASI functions return i32)
    },
    CallIndirectReg {
        type_idx: TypeIdx,
        table_idx: TableIdx,
        index_reg: Reg,        // Table index register
        param_regs: RegSlice,  // Parameter registers
        result_regs: RegSlice, // Result registers
    },
    CallReg {
        func_idx: FuncIdx,
        param_regs: RegSlice,  // Parameter registers
        result_regs: RegSlice, // Result registers
    },
    ReturnReg {
        result_regs: RegSlice, // Result registers to return
    },
    /// Unconditional jump (for Else)
    JumpReg { target_ip: usize },
    /// Block/Loop control structure
    BlockReg {
        arity: usize,
        param_count: usize,
        is_loop: bool,
    },
    /// If control structure
    IfReg {
        arity: usize,
        cond_reg: Reg,
        else_target_ip: usize, // Jump target if condition is false (else or end)
        has_else: bool,        // True if this if has an else branch
    },
    /// End of block/loop/if
    EndReg {
        source_regs: RegSlice,
        target_result_regs: RegSlice,
    },
    /// Unconditional branch
    BrReg {
        relative_depth: u32,
        target_ip: usize, // Target instruction pointer (set by fixup)
        source_regs: RegSlice,
        target_result_regs: RegSlice,
    },
    /// Conditional branch
    BrIfReg {
        relative_depth: u32,
        target_ip: usize, // Target instruction pointer (set by fixup)
        cond_reg: Reg,
        source_regs: RegSlice,
        target_result_regs: RegSlice,
    },
    /// Branch table
    BrTableReg {
        targets: Vec<(u32, usize, RegSlice)>, // (relative_depth, target_ip, target_result_regs) for each target
        default_target: (u32, usize, RegSlice), // (relative_depth, target_ip, target_result_regs) for default
        index_reg: Reg,                         // Index register
        source_regs: RegSlice,                  // Source registers (same for all targets)
    },
    /// No operation
    NopReg,
    /// Unreachable instruction
    UnreachableReg,
}

impl ProcessedInstr {
    /// Get handler_index
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
            ProcessedInstr::CallWasiReg { .. } => HANDLER_IDX_CALL_WASI_REG,
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

/// Module-level instructions that require runtime handling outside the DTC loop.
#[derive(Clone)]
pub enum ModuleLevelInstr {
    /// Return from function.
    Return,
    /// Invoke WASI function with register-based parameters.
    InvokeWasiReg {
        wasi_func_type: WasiFuncType,
        params: Vec<Val>,        // Parameters read from registers
        result_reg: Option<Reg>, // Register to write result to
    },
    /// Invoke WebAssembly function with register-based parameters.
    InvokeReg {
        func_addr: FuncAddr,
        params: Vec<Val>,              // Parameters read from registers
        result_regs: ArrayVec<Reg, 8>, // Registers to write results to (max 8)
    },
}

// Control Instructions
pub const HANDLER_IDX_UNREACHABLE: usize = 0x00;
pub const HANDLER_IDX_NOP: usize = 0x01;
pub const HANDLER_IDX_BLOCK: usize = 0x02;
pub const HANDLER_IDX_LOOP: usize = 0x03;
pub const HANDLER_IDX_IF: usize = 0x04;
pub const HANDLER_IDX_ELSE: usize = 0x05;
// pub const HANDLER_IDX_TRY: usize = 0x06;           // Exception handling (unsupported)
// pub const HANDLER_IDX_CATCH: usize = 0x07;         // Exception handling (unsupported)
// pub const HANDLER_IDX_THROW: usize = 0x08;         // Exception handling (unsupported)
// pub const HANDLER_IDX_RETHROW: usize = 0x09;       // Exception handling (unsupported)
// pub const HANDLER_IDX_RESERVED_0A: usize = 0x0A;   // Reserved
pub const HANDLER_IDX_END: usize = 0x0B;
pub const HANDLER_IDX_BR: usize = 0x0C;
pub const HANDLER_IDX_BR_IF: usize = 0x0D;
pub const HANDLER_IDX_BR_TABLE: usize = 0x0E;
pub const HANDLER_IDX_RETURN: usize = 0x0F;
pub const HANDLER_IDX_CALL: usize = 0x10;
pub const HANDLER_IDX_CALL_INDIRECT: usize = 0x11;
// pub const HANDLER_IDX_RESERVED_12: usize = 0x12;   // Reserved
// pub const HANDLER_IDX_RESERVED_13: usize = 0x13;   // Reserved
// pub const HANDLER_IDX_RESERVED_14: usize = 0x14;   // Reserved
// pub const HANDLER_IDX_RESERVED_15: usize = 0x15;   // Reserved
// pub const HANDLER_IDX_RESERVED_16: usize = 0x16;   // Reserved
// pub const HANDLER_IDX_RESERVED_17: usize = 0x17;   // Reserved
// pub const HANDLER_IDX_RESERVED_18: usize = 0x18;   // Reserved
// pub const HANDLER_IDX_RESERVED_19: usize = 0x19;   // Reserved

// Parametric Instructions
pub const HANDLER_IDX_DROP: usize = 0x1A;
pub const HANDLER_IDX_SELECT: usize = 0x1B;
// pub const HANDLER_IDX_SELECT_T: usize = 0x1C;      // Select with type (unsupported)
// pub const HANDLER_IDX_RESERVED_1D: usize = 0x1D;   // Reserved
// pub const HANDLER_IDX_RESERVED_1E: usize = 0x1E;   // Reserved
// pub const HANDLER_IDX_RESERVED_1F: usize = 0x1F;   // Reserved

// Variable Instructions
pub const HANDLER_IDX_LOCAL_GET: usize = 0x20;
pub const HANDLER_IDX_LOCAL_SET: usize = 0x21;
pub const HANDLER_IDX_LOCAL_TEE: usize = 0x22;
pub const HANDLER_IDX_GLOBAL_GET: usize = 0x23;
pub const HANDLER_IDX_GLOBAL_SET: usize = 0x24;
// pub const HANDLER_IDX_TABLE_GET_OLD: usize = 0x25; // Old table.get (moved to 0xD2, unsupported)
// pub const HANDLER_IDX_TABLE_SET_OLD: usize = 0x26; // Old table.set (moved to 0xD3, unsupported)
// pub const HANDLER_IDX_RESERVED_27: usize = 0x27;   // Reserved

// Memory Instructions
pub const HANDLER_IDX_I32_LOAD: usize = 0x28;
pub const HANDLER_IDX_I64_LOAD: usize = 0x29;
pub const HANDLER_IDX_F32_LOAD: usize = 0x2A;
pub const HANDLER_IDX_F64_LOAD: usize = 0x2B;
pub const HANDLER_IDX_I32_LOAD8_S: usize = 0x2C;
pub const HANDLER_IDX_I32_LOAD8_U: usize = 0x2D;
pub const HANDLER_IDX_I32_LOAD16_S: usize = 0x2E;
pub const HANDLER_IDX_I32_LOAD16_U: usize = 0x2F;
pub const HANDLER_IDX_I64_LOAD8_S: usize = 0x30;
pub const HANDLER_IDX_I64_LOAD8_U: usize = 0x31;
pub const HANDLER_IDX_I64_LOAD16_S: usize = 0x32;
pub const HANDLER_IDX_I64_LOAD16_U: usize = 0x33;
pub const HANDLER_IDX_I64_LOAD32_S: usize = 0x34;
pub const HANDLER_IDX_I64_LOAD32_U: usize = 0x35;
pub const HANDLER_IDX_I32_STORE: usize = 0x36;
pub const HANDLER_IDX_I64_STORE: usize = 0x37;
pub const HANDLER_IDX_F32_STORE: usize = 0x38;
pub const HANDLER_IDX_F64_STORE: usize = 0x39;
pub const HANDLER_IDX_I32_STORE8: usize = 0x3A;
pub const HANDLER_IDX_I32_STORE16: usize = 0x3B;
pub const HANDLER_IDX_I64_STORE8: usize = 0x3C;
pub const HANDLER_IDX_I64_STORE16: usize = 0x3D;
pub const HANDLER_IDX_I64_STORE32: usize = 0x3E;
pub const HANDLER_IDX_MEMORY_SIZE: usize = 0x3F;
pub const HANDLER_IDX_MEMORY_GROW: usize = 0x40;

// Const Instructions
pub const HANDLER_IDX_I32_CONST: usize = 0x41;
pub const HANDLER_IDX_I64_CONST: usize = 0x42;
pub const HANDLER_IDX_F32_CONST: usize = 0x43;
pub const HANDLER_IDX_F64_CONST: usize = 0x44;

// Numeric Instructions - i32
pub const HANDLER_IDX_I32_EQZ: usize = 0x45;
pub const HANDLER_IDX_I32_EQ: usize = 0x46;
pub const HANDLER_IDX_I32_NE: usize = 0x47;
pub const HANDLER_IDX_I32_LT_S: usize = 0x48;
pub const HANDLER_IDX_I32_LT_U: usize = 0x49;
pub const HANDLER_IDX_I32_GT_S: usize = 0x4A;
pub const HANDLER_IDX_I32_GT_U: usize = 0x4B;
pub const HANDLER_IDX_I32_LE_S: usize = 0x4C;
pub const HANDLER_IDX_I32_LE_U: usize = 0x4D;
pub const HANDLER_IDX_I32_GE_S: usize = 0x4E;
pub const HANDLER_IDX_I32_GE_U: usize = 0x4F;

// Numeric Instructions - i64
pub const HANDLER_IDX_I64_EQZ: usize = 0x50;
pub const HANDLER_IDX_I64_EQ: usize = 0x51;
pub const HANDLER_IDX_I64_NE: usize = 0x52;
pub const HANDLER_IDX_I64_LT_S: usize = 0x53;
pub const HANDLER_IDX_I64_LT_U: usize = 0x54;
pub const HANDLER_IDX_I64_GT_S: usize = 0x55;
pub const HANDLER_IDX_I64_GT_U: usize = 0x56;
pub const HANDLER_IDX_I64_LE_S: usize = 0x57;
pub const HANDLER_IDX_I64_LE_U: usize = 0x58;
pub const HANDLER_IDX_I64_GE_S: usize = 0x59;
pub const HANDLER_IDX_I64_GE_U: usize = 0x5A;

// Numeric Instructions - f32
pub const HANDLER_IDX_F32_EQ: usize = 0x5B;
pub const HANDLER_IDX_F32_NE: usize = 0x5C;
pub const HANDLER_IDX_F32_LT: usize = 0x5D;
pub const HANDLER_IDX_F32_GT: usize = 0x5E;
pub const HANDLER_IDX_F32_LE: usize = 0x5F;
pub const HANDLER_IDX_F32_GE: usize = 0x60;

// Numeric Instructions - f64
pub const HANDLER_IDX_F64_EQ: usize = 0x61;
pub const HANDLER_IDX_F64_NE: usize = 0x62;
pub const HANDLER_IDX_F64_LT: usize = 0x63;
pub const HANDLER_IDX_F64_GT: usize = 0x64;
pub const HANDLER_IDX_F64_LE: usize = 0x65;
pub const HANDLER_IDX_F64_GE: usize = 0x66;

pub const HANDLER_IDX_I32_CLZ: usize = 0x67;
pub const HANDLER_IDX_I32_CTZ: usize = 0x68;
pub const HANDLER_IDX_I32_POPCNT: usize = 0x69;
pub const HANDLER_IDX_I32_ADD: usize = 0x6A;
pub const HANDLER_IDX_I32_SUB: usize = 0x6B;
pub const HANDLER_IDX_I32_MUL: usize = 0x6C;
pub const HANDLER_IDX_I32_DIV_S: usize = 0x6D;
pub const HANDLER_IDX_I32_DIV_U: usize = 0x6E;
pub const HANDLER_IDX_I32_REM_S: usize = 0x6F;
pub const HANDLER_IDX_I32_REM_U: usize = 0x70;
pub const HANDLER_IDX_I32_AND: usize = 0x71;
pub const HANDLER_IDX_I32_OR: usize = 0x72;
pub const HANDLER_IDX_I32_XOR: usize = 0x73;
pub const HANDLER_IDX_I32_SHL: usize = 0x74;
pub const HANDLER_IDX_I32_SHR_S: usize = 0x75;
pub const HANDLER_IDX_I32_SHR_U: usize = 0x76;
pub const HANDLER_IDX_I32_ROTL: usize = 0x77;
pub const HANDLER_IDX_I32_ROTR: usize = 0x78;

pub const HANDLER_IDX_I64_CLZ: usize = 0x79;
pub const HANDLER_IDX_I64_CTZ: usize = 0x7A;
pub const HANDLER_IDX_I64_POPCNT: usize = 0x7B;
pub const HANDLER_IDX_I64_ADD: usize = 0x7C;
pub const HANDLER_IDX_I64_SUB: usize = 0x7D;
pub const HANDLER_IDX_I64_MUL: usize = 0x7E;
pub const HANDLER_IDX_I64_DIV_S: usize = 0x7F;
pub const HANDLER_IDX_I64_DIV_U: usize = 0x80;
pub const HANDLER_IDX_I64_REM_S: usize = 0x81;
pub const HANDLER_IDX_I64_REM_U: usize = 0x82;
pub const HANDLER_IDX_I64_AND: usize = 0x83;
pub const HANDLER_IDX_I64_OR: usize = 0x84;
pub const HANDLER_IDX_I64_XOR: usize = 0x85;
pub const HANDLER_IDX_I64_SHL: usize = 0x86;
pub const HANDLER_IDX_I64_SHR_S: usize = 0x87;
pub const HANDLER_IDX_I64_SHR_U: usize = 0x88;
pub const HANDLER_IDX_I64_ROTL: usize = 0x89;
pub const HANDLER_IDX_I64_ROTR: usize = 0x8A;

pub const HANDLER_IDX_F32_ABS: usize = 0x8B;
pub const HANDLER_IDX_F32_NEG: usize = 0x8C;
pub const HANDLER_IDX_F32_CEIL: usize = 0x8D;
pub const HANDLER_IDX_F32_FLOOR: usize = 0x8E;
pub const HANDLER_IDX_F32_TRUNC: usize = 0x8F;
pub const HANDLER_IDX_F32_NEAREST: usize = 0x90;
pub const HANDLER_IDX_F32_SQRT: usize = 0x91;
pub const HANDLER_IDX_F32_ADD: usize = 0x92;
pub const HANDLER_IDX_F32_SUB: usize = 0x93;
pub const HANDLER_IDX_F32_MUL: usize = 0x94;
pub const HANDLER_IDX_F32_DIV: usize = 0x95;
pub const HANDLER_IDX_F32_MIN: usize = 0x96;
pub const HANDLER_IDX_F32_MAX: usize = 0x97;
pub const HANDLER_IDX_F32_COPYSIGN: usize = 0x98;

// Numeric Instructions - f64 (continued)
pub const HANDLER_IDX_F64_ABS: usize = 0x99;
pub const HANDLER_IDX_F64_NEG: usize = 0x9A;
pub const HANDLER_IDX_F64_CEIL: usize = 0x9B;
pub const HANDLER_IDX_F64_FLOOR: usize = 0x9C;
pub const HANDLER_IDX_F64_TRUNC: usize = 0x9D;
pub const HANDLER_IDX_F64_NEAREST: usize = 0x9E;
pub const HANDLER_IDX_F64_SQRT: usize = 0x9F;
pub const HANDLER_IDX_F64_ADD: usize = 0xA0;
pub const HANDLER_IDX_F64_SUB: usize = 0xA1;
pub const HANDLER_IDX_F64_MUL: usize = 0xA2;
pub const HANDLER_IDX_F64_DIV: usize = 0xA3;
pub const HANDLER_IDX_F64_MIN: usize = 0xA4;
pub const HANDLER_IDX_F64_MAX: usize = 0xA5;
pub const HANDLER_IDX_F64_COPYSIGN: usize = 0xA6;

// Conversion Instructions
pub const HANDLER_IDX_I32_WRAP_I64: usize = 0xA7;
pub const HANDLER_IDX_I32_TRUNC_F32_S: usize = 0xA8;
pub const HANDLER_IDX_I32_TRUNC_F32_U: usize = 0xA9;
pub const HANDLER_IDX_I32_TRUNC_F64_S: usize = 0xAA;
pub const HANDLER_IDX_I32_TRUNC_F64_U: usize = 0xAB;
pub const HANDLER_IDX_I64_EXTEND_I32_S: usize = 0xAC;
pub const HANDLER_IDX_I64_EXTEND_I32_U: usize = 0xAD;
pub const HANDLER_IDX_I64_TRUNC_F32_S: usize = 0xAE;
pub const HANDLER_IDX_I64_TRUNC_F32_U: usize = 0xAF;
pub const HANDLER_IDX_I64_TRUNC_F64_S: usize = 0xB0;
pub const HANDLER_IDX_I64_TRUNC_F64_U: usize = 0xB1;
pub const HANDLER_IDX_F32_CONVERT_I32_S: usize = 0xB2;
pub const HANDLER_IDX_F32_CONVERT_I32_U: usize = 0xB3;
pub const HANDLER_IDX_F32_CONVERT_I64_S: usize = 0xB4;
pub const HANDLER_IDX_F32_CONVERT_I64_U: usize = 0xB5;
pub const HANDLER_IDX_F32_DEMOTE_F64: usize = 0xB6;
pub const HANDLER_IDX_F64_CONVERT_I32_S: usize = 0xB7;
pub const HANDLER_IDX_F64_CONVERT_I32_U: usize = 0xB8;
pub const HANDLER_IDX_F64_CONVERT_I64_S: usize = 0xB9;
pub const HANDLER_IDX_F64_CONVERT_I64_U: usize = 0xBA;
pub const HANDLER_IDX_F64_PROMOTE_F32: usize = 0xBB;
pub const HANDLER_IDX_I32_REINTERPRET_F32: usize = 0xBC;
pub const HANDLER_IDX_I64_REINTERPRET_F64: usize = 0xBD;
pub const HANDLER_IDX_F32_REINTERPRET_I32: usize = 0xBE;
pub const HANDLER_IDX_F64_REINTERPRET_I64: usize = 0xBF;

// Sign Extension Instructions
pub const HANDLER_IDX_I32_EXTEND8_S: usize = 0xC0;
pub const HANDLER_IDX_I32_EXTEND16_S: usize = 0xC1;
pub const HANDLER_IDX_I64_EXTEND8_S: usize = 0xC2;
pub const HANDLER_IDX_I64_EXTEND16_S: usize = 0xC3;
pub const HANDLER_IDX_I64_EXTEND32_S: usize = 0xC4;

// Bulk Memory Instructions (0xFC prefix, mapped to 0xC5-0xC7 range)
pub const HANDLER_IDX_MEMORY_COPY: usize = 0xC5;
pub const HANDLER_IDX_MEMORY_INIT: usize = 0xC6;
pub const HANDLER_IDX_MEMORY_FILL: usize = 0xC7;

// Saturating Truncation Instructions (0xFC prefix, mapped to 0xC8-0xCF range)
pub const HANDLER_IDX_I32_TRUNC_SAT_F32_S: usize = 0xC8;
pub const HANDLER_IDX_I32_TRUNC_SAT_F32_U: usize = 0xC9;
pub const HANDLER_IDX_I32_TRUNC_SAT_F64_S: usize = 0xCA;
pub const HANDLER_IDX_I32_TRUNC_SAT_F64_U: usize = 0xCB;
pub const HANDLER_IDX_I64_TRUNC_SAT_F32_S: usize = 0xCC;
pub const HANDLER_IDX_I64_TRUNC_SAT_F32_U: usize = 0xCD;
pub const HANDLER_IDX_I64_TRUNC_SAT_F64_S: usize = 0xCE;
pub const HANDLER_IDX_I64_TRUNC_SAT_F64_U: usize = 0xCF;

// Reference Instructions
pub const HANDLER_IDX_REF_NULL: usize = 0xD0;
pub const HANDLER_IDX_REF_IS_NULL: usize = 0xD1;
// pub const HANDLER_IDX_REF_FUNC: usize = 0xD2;       // Unsupported
// pub const HANDLER_IDX_REF_AS_NON_NULL: usize = 0xD3; // Unsupported
// pub const HANDLER_IDX_BR_ON_NULL: usize = 0xD4;      // Unsupported
// pub const HANDLER_IDX_BR_ON_NON_NULL: usize = 0xD5;  // Unsupported
// ... 0xD6-0xDF reserved/unsupported ...

// Table Instructions (0xFC prefix, mapped to 0xE0-0xE2 range for convenience)
pub const HANDLER_IDX_TABLE_GET: usize = 0xE0;
pub const HANDLER_IDX_TABLE_SET: usize = 0xE1;
pub const HANDLER_IDX_TABLE_FILL: usize = 0xE2;
// pub const HANDLER_IDX_TABLE_SIZE: usize = 0xE3;     // Unsupported (0xFC 0x10)
// pub const HANDLER_IDX_TABLE_GROW: usize = 0xE4;     // Unsupported (0xFC 0x0F)
// pub const HANDLER_IDX_TABLE_COPY: usize = 0xE5;     // Unsupported (0xFC 0x0E)
// pub const HANDLER_IDX_TABLE_INIT: usize = 0xE6;     // Unsupported (0xFC 0x0C)
// pub const HANDLER_IDX_ELEM_DROP: usize = 0xE7;      // Unsupported (0xFC 0x0D)
pub const HANDLER_IDX_DATA_DROP: usize = 0xE8;

// SIMD Instructions (0xFD prefix) - not implemented
// Vector instructions would start here (0xFD00 - 0xFDFF range)

// Thread Instructions (0xFE prefix) - not implemented
// Atomic operations would use 0xFE prefix

pub const MAX_HANDLER_INDEX: usize = 0x168;

/// VM execution state - holds all runtime state for WebAssembly execution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VMState {
    /// Global register file for all frames
    pub reg_file: RegFile,
    /// Activation frame stack
    pub activation_frame_stack: Vec<FrameStack>,
}

/// Type alias for backward compatibility
pub type Stacks = VMState;

impl VMState {
    /// Creates a new VM state for executing a function with given parameters.
    pub fn new(funcaddr: &FuncAddr, params: Vec<Val>) -> Result<VMState, RuntimeError> {
        let func_inst_guard = funcaddr.read_lock();
        match &*func_inst_guard {
            FuncInst::RuntimeFunc {
                type_,
                module,
                code,
            } => {
                if params.len() != type_.params.len() {
                    return Err(RuntimeError::InvalidParameterCount);
                }

                let mut locals = params;
                for v in code.locals.iter() {
                    for _ in 0..(v.0) {
                        locals.push(match v.1 {
                            ValueType::NumType(NumType::I32) => Val::Num(Num::I32(0)),
                            ValueType::NumType(NumType::I64) => Val::Num(Num::I64(0)),
                            ValueType::NumType(NumType::F32) => Val::Num(Num::F32(0.0)),
                            ValueType::NumType(NumType::F64) => Val::Num(Num::F64(0.0)),
                            ValueType::VecType(VecType::V128) => Val::Vec_(Vec_::V128(0)),
                            ValueType::RefType(_) => Val::Ref(Ref::RefNull),
                        });
                    }
                }

                // Initialize global RegFile
                let mut reg_file = RegFile::new_global();

                // Push initial frame allocation if present
                if let Some(alloc) = code.reg_allocation.as_ref() {
                    reg_file.save_offsets(alloc);
                }

                // Cache primary memory address and raw pointer
                let primary_mem = module.upgrade().and_then(|m| m.mem_addrs.first().cloned());
                let cached_mem_ptr = primary_mem.as_ref().map(|m| m.data_ptr());

                let initial_frame = FrameStack {
                    frame: Frame {
                        locals,
                        module: module.clone(),
                        n: type_.results.len(),
                        result_reg: code.result_reg,
                    },
                    label_stack: vec![LabelStack {
                        label: Label {
                            locals_num: type_.results.len(),
                            arity: type_.results.len(),
                            is_loop: false,
                            stack_height: 0, // Function level starts with empty stack
                            return_ip: 0,    // No return needed for function level
                        },
                        processed_instrs: code.body.clone(),
                        ip: 0,
                    }],
                    void: type_.results.is_empty(),
                    instruction_count: 0,
                    enable_checkpoint: false,
                    result_regs: ArrayVec::new(),
                    return_result_regs: ArrayVec::new(),
                    primary_mem,
                    cached_mem_ptr,
                    handlers: code.handlers.clone(),
                };

                Ok(VMState {
                    reg_file,
                    activation_frame_stack: vec![initial_frame],
                })
            }
            FuncInst::HostFunc { .. } => {
                // TODO: Handle host function invocation setup if needed,
                // or rely on FuncAddr::call to handle it directly.
                Err(RuntimeError::UnimplementedHostFunction)
            }
            FuncInst::WasiFunc { .. } => {
                // WASI functions are handled directly in Runtime, not through Stacks
                Err(RuntimeError::UnimplementedHostFunction)
            }
        }
    }

    /// Get mutable references to both reg_file and activation_frame_stack
    /// This allows split borrowing of VMState fields
    pub fn get_reg_file_and_frames(&mut self) -> (&mut RegFile, &mut Vec<FrameStack>) {
        (&mut self.reg_file, &mut self.activation_frame_stack)
    }
}

/// Call frame containing locals and module reference.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub locals: Vec<Val>,
    #[serde(skip)]
    pub module: Weak<ModuleInst>,
    pub n: usize,
    pub result_reg: Option<crate::execution::regs::Reg>, // Register for return value (register mode only)
}

/// Activation frame stack with label stacks and execution state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameStack {
    pub frame: Frame,
    pub label_stack: Vec<LabelStack>,
    pub void: bool,
    pub instruction_count: u64,
    #[serde(skip)]
    pub enable_checkpoint: bool,
    /// Registers where caller expects results to be written (max 8 for multi-value)
    pub result_regs: ArrayVec<Reg, 8>,
    /// Registers containing this function's return values (set on return, max 8)
    pub return_result_regs: ArrayVec<Reg, 8>,
    /// Cached primary memory address (avoids module.upgrade() on every memory op)
    #[serde(skip)]
    pub primary_mem: Option<MemAddr>,
    /// Cached raw pointer to memory data (avoids Rc/UnsafeCell indirection on every memory op)
    /// Must be updated after memory.grow
    #[serde(skip)]
    pub cached_mem_ptr: Option<*mut u8>,
    /// v2 dispatcher handler array (parallel to label_stack[*].processed_instrs).
    /// Cloned (Rc bump) from `Func.handlers` at frame entry.
    #[serde(skip)]
    pub handlers: Rc<Vec<super::ir::Handler>>,
}

impl FrameStack {
    /// Push a new label stack for a block or loop
    pub fn push_label_stack(&mut self, label: Label, instructions: Vec<ProcessedInstr>) {
        let new_label_stack = LabelStack {
            label,
            processed_instrs: Rc::new(instructions),
            ip: 0,
        };
        self.label_stack.push(new_label_stack);
    }

    /// Pop the top label stack when exiting a block
    pub fn pop_label_stack(&mut self) -> Option<LabelStack> {
        if self.label_stack.len() > 1 {
            self.label_stack.pop()
        } else {
            None
        }
    }
}

/// Block/loop label with arity and return information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Label {
    pub locals_num: usize,
    pub arity: usize,
    pub is_loop: bool,
    pub stack_height: usize,
    pub return_ip: usize,
}

/// Label stack containing instructions and program counter.
#[derive(Clone, Debug)]
pub struct LabelStack {
    pub label: Label,
    pub processed_instrs: Rc<Vec<ProcessedInstr>>,
    pub ip: usize,
}

impl Serialize for LabelStack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        // Skip processed_instrs: it is deterministically derived from the Wasm binary
        // and will be reconstructed from the module during restore.
        let mut state = serializer.serialize_struct("LabelStack", 2)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("ip", &self.ip)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for LabelStack {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct LabelStackData {
            label: Label,
            ip: usize,
        }

        let data = LabelStackData::deserialize(deserializer)?;
        Ok(LabelStack {
            label: data.label,
            // Placeholder: reconstructed from module during restore
            processed_instrs: Rc::new(Vec::new()),
            ip: data.ip,
        })
    }
}

pub const HANDLER_IDX_SELECT_I32: usize = 0xF0;
pub const HANDLER_IDX_SELECT_I64: usize = 0xF1;
pub const HANDLER_IDX_SELECT_F32: usize = 0xF2;
pub const HANDLER_IDX_SELECT_F64: usize = 0xF3;

// GlobalGetReg handler constants
pub const HANDLER_IDX_GLOBAL_GET_I32: usize = 0xF4;
pub const HANDLER_IDX_GLOBAL_GET_I64: usize = 0xF5;
pub const HANDLER_IDX_GLOBAL_GET_F32: usize = 0xF6;
pub const HANDLER_IDX_GLOBAL_GET_F64: usize = 0xF7;

// GlobalSetReg handler constants
pub const HANDLER_IDX_GLOBAL_SET_I32: usize = 0xF8;
pub const HANDLER_IDX_GLOBAL_SET_I64: usize = 0xF9;
pub const HANDLER_IDX_GLOBAL_SET_F32: usize = 0xFA;
pub const HANDLER_IDX_GLOBAL_SET_F64: usize = 0xFB;

// TableRefReg handler constants
pub const HANDLER_IDX_REF_NULL_REG: usize = 0xFC;
pub const HANDLER_IDX_REF_IS_NULL_REG: usize = 0xFD;
pub const HANDLER_IDX_TABLE_GET_REG: usize = 0xFE;
pub const HANDLER_IDX_TABLE_SET_REG: usize = 0xFF;
pub const HANDLER_IDX_TABLE_FILL_REG: usize = 0x100;

// RefLocalReg handler constants
pub const HANDLER_IDX_REF_LOCAL_GET_REG: usize = 0x101;
pub const HANDLER_IDX_REF_LOCAL_SET_REG: usize = 0x102;

// CallWasiReg handler constant
pub const HANDLER_IDX_CALL_WASI_REG: usize = 0x103;

