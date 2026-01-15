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
use crate::structure::types::LabelIdx as StructureLabelIdx;
use crate::structure::{instructions::*, types::*};
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::rc::{Rc, Weak};

// Value source for hybrid approach
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValueSource {
    Stack,        // Value from stack (default)
    Const(Value), // Constant value
    Local(u32),   // Local variable
    Global(u32),  // Global variable
}

// Value type for direct operands
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Value {
    pub fn to_val(&self) -> Val {
        match self {
            Value::I32(v) => Val::Num(Num::I32(*v)),
            Value::I64(v) => Val::Num(Num::I64(*v)),
            Value::F32(v) => Val::Num(Num::F32(*v)),
            Value::F64(v) => Val::Num(Num::F64(*v)),
        }
    }
}

// Store target for optimized operations
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StoreTarget {
    Local(u32),
    Global(u32),
}

// Unified optimized operand type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum OptimizedOperand {
    // For operations that need 1 value (load, unary operations)
    Single {
        value: Option<ValueSource>,
        memarg: Option<Memarg>,            // For memory operations
        store_target: Option<StoreTarget>, // Where to store the result
    },
    // For operations that need 2 values (binary ops, store)
    Double {
        first: Option<ValueSource>,        // binary op's left, or store's addr
        second: Option<ValueSource>,       // binary op's right, or store's value
        memarg: Option<Memarg>,            // For store operations
        store_target: Option<StoreTarget>, // Where to store the result
    },
}

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

    // Unified optimized operand
    Optimized(OptimizedOperand),
}

/// Register-based operand for I32 operations
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum I32RegOperand {
    Reg(u16),   // Read from register
    Const(i32), // Constant value
    Param(u16), // Read from parameter/local
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum I64RegOperand {
    Reg(u16),
    Const(i64),
    Param(u16),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum F32RegOperand {
    Reg(u16),
    Const(f32),
    Param(u16),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum F64RegOperand {
    Reg(u16),
    Const(f64),
    Param(u16),
}

/// I32 operations for register-based execution
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum I32Op {
    // Constants and locals
    Const,
    GetParam,
    SetLocal,
    // Binary arithmetic operations
    Add,
    Sub,
    Mul,
    DivS,
    DivU,
    RemS,
    RemU,
    // Binary bitwise operations
    And,
    Or,
    Xor,
    Shl,
    ShrS,
    ShrU,
    Rotl,
    Rotr,
    // Comparison operations
    Eq,
    Ne,
    LtS,
    LtU,
    LeS,
    LeU,
    GtS,
    GtU,
    GeS,
    GeU,
    // Unary operations
    Clz,
    Ctz,
    Popcnt,
    Eqz,
    Extend8S,
    Extend16S,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProcessedInstr {
    /// Register-based I32 instruction
    I32Reg {
        handler_index: usize,
        dst: u16,                    // Destination register index
        src1: I32RegOperand,         // First operand
        src2: Option<I32RegOperand>, // Second operand (None for unary ops)
    },
    /// Register-based I64 instruction
    I64Reg {
        handler_index: usize,
        dst: Reg, // Destination register (I64 for arithmetic, I32 for comparisons)
        src1: I64RegOperand,
        src2: Option<I64RegOperand>,
    },

    F32Reg {
        handler_index: usize,
        dst: Reg,
        src1: F32RegOperand,
        src2: Option<F32RegOperand>,
    },

    F64Reg {
        handler_index: usize,
        dst: Reg,
        src1: F64RegOperand,
        src2: Option<F64RegOperand>,
    },
    ConversionReg {
        handler_index: usize,
        dst: Reg, // Destination register (type determined by output type)
        src: Reg, // Source register (type determined by input type)
    },
    MemoryLoadReg {
        handler_index: usize,
        dst: Reg,    // Destination register for loaded value
        addr: Reg,   // Address register (always I32)
        offset: u64, // Memory offset
    },
    MemoryStoreReg {
        handler_index: usize,
        addr: Reg,   // Address register (always I32)
        value: Reg,  // Value register to store
        offset: u64, // Memory offset
    },

    MemoryOpsReg {
        handler_index: usize,
        dst: Option<Reg>, // Destination register (for size/grow results)
        args: Vec<Reg>,   // Argument registers (varies by operation)
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
        dst: Reg,          // Destination register
        global_index: u32, // Global variable index
    },

    GlobalSetReg {
        handler_index: usize,
        src: Reg,          // Source register
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
        param_regs: Vec<Reg>,    // Parameter registers
        result_reg: Option<Reg>, // Result register (most WASI functions return i32)
    },
    CallIndirectReg {
        type_idx: TypeIdx,
        table_idx: TableIdx,
        index_reg: Reg,        // Table index register
        param_regs: Vec<Reg>,  // Parameter registers
        result_regs: Vec<Reg>, // Result registers
    },
    CallReg {
        func_idx: FuncIdx,
        param_regs: Vec<Reg>,  // Parameter registers
        result_regs: Vec<Reg>, // Result registers
    },
    ReturnReg {
        result_regs: Vec<Reg>, // Result registers to return
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
        source_regs: Vec<Reg>,
        target_result_regs: Vec<Reg>,
    },
    /// Unconditional branch
    BrReg {
        relative_depth: u32,
        target_ip: usize, // Target instruction pointer (set by fixup)
        source_regs: Vec<Reg>,
        target_result_regs: Vec<Reg>,
    },
    /// Conditional branch
    BrIfReg {
        relative_depth: u32,
        target_ip: usize, // Target instruction pointer (set by fixup)
        cond_reg: Reg,
        source_regs: Vec<Reg>,
        target_result_regs: Vec<Reg>,
    },
    /// Branch table
    BrTableReg {
        targets: Vec<(u32, usize, Vec<Reg>)>, // (relative_depth, target_ip, target_result_regs) for each target
        default_target: (u32, usize, Vec<Reg>), // (relative_depth, target_ip, target_result_regs) for default
        index_reg: Reg,                         // Index register
        source_regs: Vec<Reg>,                  // Source registers (same for all targets)
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

#[derive(Clone)]
pub enum ModuleLevelInstr {
    Return,
    InvokeWasiReg {
        wasi_func_type: WasiFuncType,
        params: Vec<Val>,        // Parameters read from registers
        result_reg: Option<Reg>, // Register to write result to
    },
    InvokeReg {
        func_addr: FuncAddr,
        params: Vec<Val>,      // Parameters read from registers
        result_regs: Vec<Reg>, // Registers to write results to
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
                    result_regs: vec![],
                    return_result_regs: vec![],
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub locals: Vec<Val>,
    #[serde(skip)]
    pub module: Weak<ModuleInst>,
    pub n: usize,
    pub result_reg: Option<crate::execution::regs::Reg>, // Register for return value (register mode only)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameStack {
    pub frame: Frame,
    pub label_stack: Vec<LabelStack>,
    pub void: bool,
    pub instruction_count: u64,
    #[serde(skip)]
    pub enable_checkpoint: bool,
    /// Registers where caller expects results to be written
    pub result_regs: Vec<Reg>,
    /// Registers containing this function's return values (set on return)
    pub return_result_regs: Vec<Reg>,
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

    /// DTC execution loop
    pub fn run_dtc_loop(
        &mut self,
        reg_file: &mut RegFile,
        _called_func_addr_out: &mut Option<FuncAddr>,
        _execution_stats: Option<&mut super::stats::ExecutionStats>,
        _tracer: Option<&mut super::trace::Tracer>,
    ) -> Result<Result<Option<ModuleLevelInstr>, RuntimeError>, RuntimeError> {
        let mut current_label_stack_idx = self
            .label_stack
            .len()
            .checked_sub(1)
            .ok_or(RuntimeError::StackError("Initial label stack empty"))?;

        loop {
            if self.enable_checkpoint {
                #[cfg(all(
                    target_arch = "wasm32",
                    target_os = "wasi",
                    target_env = "p1",
                    target_feature = "atomics"
                ))]
                {
                    if migration::check_checkpoint_flag() {
                        return Ok(Err(RuntimeError::CheckpointRequested));
                    }
                }
            }

            self.instruction_count += 1;

            if current_label_stack_idx >= self.label_stack.len() {
                break;
            }

            let current_label_stack = &mut self.label_stack[current_label_stack_idx];
            let processed_code = &current_label_stack.processed_instrs;
            let ip = current_label_stack.ip;

            if ip >= processed_code.len() {
                current_label_stack.ip = ip;

                // Label stack index indicates nesting level:
                // Index 0: Function level (function body)
                // Index 1+: Block/loop/if levels (nested blocks)
                if current_label_stack_idx > 0 {
                    let return_ip = self.label_stack[current_label_stack_idx].label.return_ip;
                    let current_label = &self.label_stack[current_label_stack_idx].label;
                    let is_loop = current_label.is_loop;

                    // Check if returning to parent would cause re-execution of already processed code
                    // This handles cases where break jumps out of loops and shouldn't re-enter them
                    // Exception: For loops, it's normal for return_ip to be smaller than current_ip
                    if return_ip <= ip && !is_loop {
                        // This would cause re-execution - end the function instead
                        break;
                    }

                    self.label_stack.pop();
                    current_label_stack_idx -= 1;

                    // Continue execution at return point in parent level
                    if current_label_stack_idx < self.label_stack.len() {
                        let parent_label_stack = &mut self.label_stack[current_label_stack_idx];
                        if is_loop {
                            parent_label_stack.ip = ip + 1;
                        } else {
                            parent_label_stack.ip = return_ip;
                        }
                    }
                    continue;
                } else {
                    // Function level (index 0): end of function execution
                    break;
                }
            }

            let instruction_ref = &processed_code[ip];

            // Match on instruction type
            match instruction_ref {
                ProcessedInstr::I32Reg {
                    handler_index,
                    dst,
                    src1,
                    src2,
                } => {
                    // Create context for handler
                    let ctx = I32RegContext {
                        reg_file: reg_file.get_i32_regs(),
                        locals: &mut self.frame.locals,
                        src1: src1.clone(),
                        src2: src2.clone(),
                    };

                    let handler = I32_REG_HANDLER_TABLE[*handler_index];
                    handler(ctx, *dst)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::I64Reg {
                    handler_index,
                    dst,
                    src1,
                    src2,
                } => {
                    // Get both i32 and i64 registers for comparison operations
                    let (i32_regs, i64_regs) = reg_file.get_i32_and_i64_regs();
                    let ctx = I64RegContext {
                        i64_regs,
                        i32_regs,
                        locals: &mut self.frame.locals,
                        src1: src1.clone(),
                        src2: src2.clone(),
                        dst: dst.clone(),
                    };

                    let handler = I64_REG_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::F32Reg {
                    handler_index,
                    dst,
                    src1,
                    src2,
                } => {
                    // Get both i32 and f32 registers for comparison operations
                    let (i32_regs, f32_regs) = reg_file.get_i32_and_f32_regs();
                    let ctx = F32RegContext {
                        f32_regs,
                        i32_regs,
                        locals: &mut self.frame.locals,
                        src1: src1.clone(),
                        src2: src2.clone(),
                        dst: dst.clone(),
                    };

                    let handler = F32_REG_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::F64Reg {
                    handler_index,
                    dst,
                    src1,
                    src2,
                } => {
                    // Get both i32 and f64 registers for comparison operations
                    let (i32_regs, f64_regs) = reg_file.get_i32_and_f64_regs();
                    let ctx = F64RegContext {
                        f64_regs,
                        i32_regs,
                        locals: &mut self.frame.locals,
                        src1: src1.clone(),
                        src2: src2.clone(),
                        dst: dst.clone(),
                    };

                    let handler = F64_REG_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::ConversionReg {
                    handler_index,
                    dst,
                    src,
                } => {
                    let ctx = ConversionRegContext {
                        reg_file,
                        src: src.clone(),
                        dst: dst.clone(),
                    };

                    let handler = CONVERSION_REG_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::MemoryLoadReg {
                    handler_index,
                    dst,
                    addr,
                    offset,
                } => {
                    let module_inst = self
                        .frame
                        .module
                        .upgrade()
                        .ok_or(RuntimeError::ModuleInstanceGone)?;
                    let mem_addr = module_inst
                        .mem_addrs
                        .first()
                        .ok_or(RuntimeError::MemoryNotFound)?;

                    let ctx = MemoryLoadRegContext {
                        reg_file,
                        mem_addr,
                        addr: addr.clone(),
                        dst: dst.clone(),
                        offset: *offset,
                    };

                    let handler = MEMORY_LOAD_REG_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::MemoryStoreReg {
                    handler_index,
                    addr,
                    value,
                    offset,
                } => {
                    let module_inst = self
                        .frame
                        .module
                        .upgrade()
                        .ok_or(RuntimeError::ModuleInstanceGone)?;
                    let mem_addr = module_inst
                        .mem_addrs
                        .first()
                        .ok_or(RuntimeError::MemoryNotFound)?;

                    let ctx = MemoryStoreRegContext {
                        reg_file,
                        mem_addr,
                        addr: addr.clone(),
                        value: value.clone(),
                        offset: *offset,
                    };

                    let handler = MEMORY_STORE_REG_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::MemoryOpsReg {
                    handler_index,
                    dst,
                    args,
                    data_index,
                } => {
                    let module_inst = self
                        .frame
                        .module
                        .upgrade()
                        .ok_or(RuntimeError::ModuleInstanceGone)?;
                    let mem_addr = module_inst
                        .mem_addrs
                        .first()
                        .ok_or(RuntimeError::MemoryNotFound)?;

                    let ctx = MemoryOpsRegContext {
                        reg_file,
                        mem_addr,
                        module_inst: &module_inst,
                        dst: dst.clone(),
                        args: args.clone(),
                        data_index: *data_index,
                    };

                    let handler = MEMORY_OPS_REG_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::SelectReg {
                    handler_index,
                    dst,
                    val1,
                    val2,
                    cond,
                } => {
                    let ctx = SelectRegContext {
                        reg_file,
                        dst: dst.clone(),
                        val1: val1.clone(),
                        val2: val2.clone(),
                        cond: cond.clone(),
                    };

                    let handler = SELECT_REG_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::GlobalGetReg {
                    handler_index,
                    dst,
                    global_index,
                } => {
                    let module_inst = self
                        .frame
                        .module
                        .upgrade()
                        .ok_or(RuntimeError::ModuleInstanceGone)?;
                    let global_addr = module_inst
                        .global_addrs
                        .get_by_idx(GlobalIdx(*global_index))
                        .clone();
                    let val = global_addr.get();

                    match *handler_index {
                        HANDLER_IDX_GLOBAL_GET_I32 => {
                            reg_file.set_i32(dst.index(), val.to_i32().unwrap());
                        }
                        HANDLER_IDX_GLOBAL_GET_I64 => {
                            reg_file.set_i64(dst.index(), val.to_i64().unwrap());
                        }
                        HANDLER_IDX_GLOBAL_GET_F32 => {
                            reg_file.set_f32(dst.index(), val.to_f32().unwrap());
                        }
                        HANDLER_IDX_GLOBAL_GET_F64 => {
                            reg_file.set_f64(dst.index(), val.to_f64().unwrap());
                        }
                        _ => return Err(RuntimeError::InvalidHandlerIndex),
                    }

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::GlobalSetReg {
                    handler_index,
                    src,
                    global_index,
                } => {
                    let val = match *handler_index {
                        HANDLER_IDX_GLOBAL_SET_I32 => {
                            Val::Num(Num::I32(reg_file.get_i32(src.index())))
                        }
                        HANDLER_IDX_GLOBAL_SET_I64 => {
                            Val::Num(Num::I64(reg_file.get_i64(src.index())))
                        }
                        HANDLER_IDX_GLOBAL_SET_F32 => {
                            Val::Num(Num::F32(reg_file.get_f32(src.index())))
                        }
                        HANDLER_IDX_GLOBAL_SET_F64 => {
                            Val::Num(Num::F64(reg_file.get_f64(src.index())))
                        }
                        _ => return Err(RuntimeError::InvalidHandlerIndex),
                    };

                    let module_inst = self
                        .frame
                        .module
                        .upgrade()
                        .ok_or(RuntimeError::ModuleInstanceGone)?;
                    let global_addr = module_inst
                        .global_addrs
                        .get_by_idx(GlobalIdx(*global_index))
                        .clone();
                    global_addr.set(val)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::DataDropReg { data_index } => {
                    let module_inst = self
                        .frame
                        .module
                        .upgrade()
                        .ok_or(RuntimeError::ModuleInstanceGone)?;

                    if (*data_index as usize) < module_inst.data_addrs.len() {
                        module_inst.data_addrs[*data_index as usize].drop_data();
                    }

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::RefLocalReg {
                    handler_index,
                    dst,
                    src,
                    local_idx,
                } => {
                    let handler_fn = REF_LOCAL_REG_HANDLER_TABLE
                        .get(*handler_index)
                        .ok_or(RuntimeError::InvalidHandlerIndex)?;

                    handler_fn(RefLocalRegContext {
                        reg_file,
                        locals: &mut self.frame.locals,
                        dst: *dst,
                        src: *src,
                        local_idx: *local_idx,
                    })?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::TableRefReg {
                    handler_index,
                    table_idx,
                    regs,
                    ref_type,
                } => {
                    let handler_fn = TABLE_REF_REG_HANDLER_TABLE
                        .get(*handler_index)
                        .ok_or(RuntimeError::InvalidHandlerIndex)?;

                    let module_inst = self
                        .frame
                        .module
                        .upgrade()
                        .ok_or(RuntimeError::ModuleInstanceGone)?;

                    handler_fn(TableRefRegContext {
                        reg_file,
                        module_inst: &module_inst,
                        table_idx: *table_idx,
                        regs: *regs,
                        ref_type: *ref_type,
                    })?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::CallWasiReg {
                    wasi_func_type,
                    param_regs,
                    result_reg,
                } => {
                    let wasi_func_type_copy = *wasi_func_type;
                    let param_regs_copy = param_regs.clone();
                    let result_reg_copy = *result_reg;

                    // Read parameters from registers
                    let params: Vec<Val> = param_regs_copy
                        .iter()
                        .map(|reg| reg_file.get_val(reg))
                        .collect();

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    return Ok(Ok(Some(ModuleLevelInstr::InvokeWasiReg {
                        wasi_func_type: wasi_func_type_copy,
                        params,
                        result_reg: result_reg_copy,
                    })));
                }
                ProcessedInstr::CallIndirectReg {
                    type_idx,
                    table_idx,
                    index_reg,
                    param_regs,
                    result_regs,
                } => {
                    let type_idx = *type_idx;
                    let table_idx = *table_idx;
                    let index_reg = *index_reg;
                    let param_regs = param_regs.clone();
                    let result_regs = result_regs.clone();

                    // Read table index from register
                    let i = reg_file.get_i32(index_reg.index());

                    let module_inst = self
                        .frame
                        .module
                        .upgrade()
                        .ok_or(RuntimeError::ModuleInstanceGone)?;
                    let table_addr = module_inst
                        .table_addrs
                        .get(table_idx.0 as usize)
                        .ok_or(RuntimeError::TableNotFound)?;

                    let func_ref_option = table_addr.get_func_addr(i as usize);

                    if let Some(func_addr) = func_ref_option {
                        let actual_type = func_addr.func_type();
                        let expected_type = &module_inst.types[type_idx.0 as usize];

                        if actual_type != *expected_type {
                            return Err(RuntimeError::IndirectCallTypeMismatch);
                        }

                        if self.enable_checkpoint {
                            #[cfg(all(
                                target_arch = "wasm32",
                                target_os = "wasi",
                                target_env = "p1",
                                not(target_feature = "atomics")
                            ))]
                            {
                                if migration::check_checkpoint_trigger(&self.frame)? {
                                    return Ok(Err(RuntimeError::CheckpointRequested));
                                }
                            }
                        }

                        let params: Vec<Val> =
                            param_regs.iter().map(|reg| reg_file.get_val(reg)).collect();

                        self.label_stack[current_label_stack_idx].ip = ip + 1;
                        return Ok(Ok(Some(ModuleLevelInstr::InvokeReg {
                            func_addr,
                            params,
                            result_regs,
                        })));
                    } else {
                        return Err(RuntimeError::UninitializedElement);
                    }
                }
                ProcessedInstr::CallReg {
                    func_idx,
                    param_regs,
                    result_regs,
                } => {
                    let func_idx = *func_idx;
                    let param_regs = param_regs.clone();
                    let result_regs = result_regs.clone();

                    let module_inst = self
                        .frame
                        .module
                        .upgrade()
                        .ok_or(RuntimeError::ModuleInstanceGone)?;

                    let func_addr = module_inst
                        .func_addrs
                        .get(func_idx.0 as usize)
                        .ok_or(RuntimeError::ExportFuncNotFound)?
                        .clone();

                    if self.enable_checkpoint {
                        #[cfg(all(
                            target_arch = "wasm32",
                            target_os = "wasi",
                            target_env = "p1",
                            not(target_feature = "atomics")
                        ))]
                        {
                            if migration::check_checkpoint_trigger(&self.frame)? {
                                return Ok(Err(RuntimeError::CheckpointRequested));
                            }
                        }
                    }

                    // Read parameters from registers
                    let params: Vec<Val> =
                        param_regs.iter().map(|reg| reg_file.get_val(reg)).collect();

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    return Ok(Ok(Some(ModuleLevelInstr::InvokeReg {
                        func_addr,
                        params,
                        result_regs,
                    })));
                }
                ProcessedInstr::ReturnReg { result_regs } => {
                    // Store result registers for caller to read
                    self.return_result_regs = result_regs.clone();

                    return Ok(Ok(Some(ModuleLevelInstr::Return)));
                }
                ProcessedInstr::JumpReg { target_ip } => {
                    let target_ip = *target_ip;
                    if self.label_stack.len() > 1 {
                        self.label_stack.pop();
                        current_label_stack_idx = self.label_stack.len() - 1;
                    }
                    self.label_stack[current_label_stack_idx].ip = target_ip;
                    continue;
                }
                ProcessedInstr::BlockReg {
                    arity,
                    param_count,
                    is_loop,
                } => {
                    // Push a new label for Block/Loop
                    let label = Label {
                        locals_num: *param_count,
                        arity: *arity,
                        is_loop: *is_loop,
                        stack_height: 0, // Not used in register mode
                        return_ip: ip + 1,
                    };
                    let current_instrs = self.label_stack[current_label_stack_idx]
                        .processed_instrs
                        .clone();
                    self.label_stack.push(LabelStack {
                        label,
                        processed_instrs: current_instrs,
                        ip: ip + 1,
                    });
                    current_label_stack_idx = self.label_stack.len() - 1;
                    continue;
                }
                ProcessedInstr::IfReg {
                    arity,
                    cond_reg,
                    else_target_ip,
                    has_else,
                } => {
                    let arity = *arity;
                    let cond_reg = *cond_reg;
                    let else_target_ip = *else_target_ip;
                    let has_else = *has_else;

                    // Read condition from register
                    let cond = reg_file.get_i32(cond_reg.index());

                    let current_instrs = self.label_stack[current_label_stack_idx]
                        .processed_instrs
                        .clone();

                    if cond != 0 {
                        // Condition true: execute then branch
                        // Push a new label for If
                        let label = Label {
                            locals_num: 0,
                            arity,
                            is_loop: false,
                            stack_height: 0, // Not used in register mode
                            return_ip: else_target_ip,
                        };

                        self.label_stack.push(LabelStack {
                            label,
                            processed_instrs: current_instrs,
                            ip: ip + 1,
                        });
                        current_label_stack_idx = self.label_stack.len() - 1;
                    } else if has_else {
                        // Condition false with else: push label stack and execute else branch
                        let label = Label {
                            locals_num: 0,
                            arity,
                            is_loop: false,
                            stack_height: 0,
                            return_ip: else_target_ip,
                        };

                        self.label_stack.push(LabelStack {
                            label,
                            processed_instrs: current_instrs,
                            ip: else_target_ip,
                        });
                        current_label_stack_idx = self.label_stack.len() - 1;
                    } else {
                        // Condition false without else: skip the entire if, no label stack push
                        self.label_stack[current_label_stack_idx].ip = else_target_ip;
                    }
                    continue;
                }
                ProcessedInstr::EndReg {
                    source_regs,
                    target_result_regs,
                } => {
                    let source_regs = source_regs.clone();
                    let target_result_regs = target_result_regs.clone();

                    // Pop label stack
                    if self.label_stack.len() > 1 {
                        // Block/Loop/If level end: copy results to target registers
                        reg_file.copy_regs(&source_regs, &target_result_regs);
                        self.label_stack.pop();
                        current_label_stack_idx = self.label_stack.len() - 1;
                        // Set parent's ip to continue after the End instruction
                        self.label_stack[current_label_stack_idx].ip = ip + 1;

                        // If next ip is beyond code length and we're at function level, store result registers and break
                        let parent_processed_code =
                            &self.label_stack[current_label_stack_idx].processed_instrs;
                        if ip + 1 >= parent_processed_code.len() && current_label_stack_idx == 0 {
                            self.return_result_regs = source_regs.clone();
                            break;
                        }
                    } else {
                        // Function level end: store result registers for return
                        self.return_result_regs = source_regs.clone();
                        break;
                    }
                    continue;
                }
                ProcessedInstr::BrReg {
                    relative_depth,
                    target_ip,
                    source_regs,
                    target_result_regs,
                } => {
                    let relative_depth = *relative_depth as usize;
                    let target_ip = *target_ip;
                    let source_regs = source_regs.clone();
                    let target_result_regs = target_result_regs.clone();

                    if !source_regs.is_empty() && !target_result_regs.is_empty() {
                        reg_file.copy_regs(&source_regs, &target_result_regs);
                    }

                    // Branch: exit (relative_depth + 1) blocks
                    if relative_depth <= current_label_stack_idx {
                        // target_level is the label index we want to continue in
                        // For relative_depth=0, we exit the current block and continue in parent
                        let target_level = current_label_stack_idx - relative_depth;
                        // Keep at least 1 label (function level)
                        let keep_count = target_level.max(1);
                        self.label_stack.truncate(keep_count);
                        current_label_stack_idx = self.label_stack.len() - 1;

                        // Set IP to target (already computed by fixup)
                        self.label_stack[current_label_stack_idx].ip = target_ip;
                    } else {
                        // Branch to function level - return
                        break;
                    }
                    continue;
                }
                ProcessedInstr::BrIfReg {
                    relative_depth,
                    target_ip,
                    cond_reg,
                    source_regs,
                    target_result_regs,
                } => {
                    let relative_depth = *relative_depth as usize;
                    let target_ip = *target_ip;
                    let cond_reg = *cond_reg;
                    let source_regs = source_regs.clone();
                    let target_result_regs = target_result_regs.clone();

                    // Read condition from register
                    let cond = reg_file.get_i32(cond_reg.index());

                    if cond != 0 {
                        // Take the branch
                        if !source_regs.is_empty() && !target_result_regs.is_empty() {
                            reg_file.copy_regs(&source_regs, &target_result_regs);
                        }

                        if relative_depth <= current_label_stack_idx {
                            let target_level = current_label_stack_idx - relative_depth;
                            // Keep at least 1 label (function level)
                            let keep_count = target_level.max(1);
                            self.label_stack.truncate(keep_count);
                            current_label_stack_idx = self.label_stack.len() - 1;

                            // Set IP to target (already computed by fixup)
                            self.label_stack[current_label_stack_idx].ip = target_ip;
                        } else {
                            break;
                        }
                    } else {
                        self.label_stack[current_label_stack_idx].ip = ip + 1;
                    }
                    continue;
                }
                ProcessedInstr::BrTableReg {
                    targets,
                    default_target,
                    index_reg,
                    source_regs,
                } => {
                    let targets = targets.clone();
                    let default_target = default_target.clone();
                    let index_reg = *index_reg;
                    let source_regs = source_regs.clone();

                    // Read index from register
                    let idx = reg_file.get_i32(index_reg.index()) as usize;

                    // Select target (relative_depth, target_ip, target_result_regs) tuple
                    let (relative_depth, target_ip, target_result_regs) = if idx < targets.len() {
                        targets[idx].clone()
                    } else {
                        default_target
                    };
                    let relative_depth = relative_depth as usize;

                    // Copy source to target result registers
                    if !source_regs.is_empty() && !target_result_regs.is_empty() {
                        reg_file.copy_regs(&source_regs, &target_result_regs);
                    }

                    // Branch
                    if relative_depth <= current_label_stack_idx {
                        let target_level = current_label_stack_idx - relative_depth;
                        // Keep at least 1 label (function level)
                        let keep_count = target_level.max(1);
                        self.label_stack.truncate(keep_count);
                        current_label_stack_idx = self.label_stack.len() - 1;

                        // Set IP to target (already computed by fixup)
                        self.label_stack[current_label_stack_idx].ip = target_ip;
                    } else {
                        break;
                    }
                    continue;
                }
                ProcessedInstr::NopReg => {
                    // No operation - just advance IP
                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::UnreachableReg => {
                    return Err(RuntimeError::Unreachable);
                }
            } // End of instruction match
        } // End of main loop
        Ok(Ok(None))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Label {
    pub locals_num: usize,
    pub arity: usize,
    pub is_loop: bool,
    pub stack_height: usize,
    pub return_ip: usize,
}

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
        let mut state = serializer.serialize_struct("LabelStack", 3)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("processed_instrs", self.processed_instrs.as_ref())?;
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
            processed_instrs: Vec<ProcessedInstr>,
            ip: usize,
        }

        let data = LabelStackData::deserialize(deserializer)?;
        Ok(LabelStack {
            label: data.label,
            processed_instrs: Rc::new(data.processed_instrs),
            ip: data.ip,
        })
    }
}

#[derive(Clone, Debug)]
pub enum FrameLevelInstr {
    Label(Label, Vec<Instr>),
    Br(StructureLabelIdx),
    EndLabel,
    Invoke(FuncAddr),
    Return,
}

#[derive(Clone, Debug)]
pub enum AdminInstr {
    Trap,
    Instr(Instr),
    Invoke(FuncAddr),
    Label(Label, Vec<Instr>),
    Br(StructureLabelIdx),
    Return,
    Ref(FuncAddr),
    RefExtern(ExternAddr),
}

struct I32RegContext<'a> {
    reg_file: &'a mut [i32],
    locals: &'a mut [Val],
    src1: I32RegOperand,
    src2: Option<I32RegOperand>,
}

impl<'a> I32RegContext<'a> {
    #[inline]
    fn get_operand(&self, operand: &I32RegOperand) -> Result<i32, RuntimeError> {
        match operand {
            I32RegOperand::Reg(idx) => Ok(self.reg_file[*idx as usize]),
            I32RegOperand::Const(val) => Ok(*val),
            I32RegOperand::Param(idx) => self.locals[*idx as usize].to_i32(),
        }
    }
}

fn i32_reg_local_get(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.reg_file[dst as usize] = val;
    Ok(())
}

fn i32_reg_local_set(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    // dst is the local variable index, src1 is the value source
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.locals[dst as usize] = Val::Num(Num::I32(val));
    Ok(())
}

fn i32_reg_const(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.reg_file[dst as usize] = val;
    Ok(())
}

fn i32_reg_add(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = lhs.wrapping_add(rhs);
    Ok(())
}

fn i32_reg_sub(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = lhs.wrapping_sub(rhs);
    Ok(())
}

fn i32_reg_mul(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = lhs.wrapping_mul(rhs);
    Ok(())
}

fn i32_reg_div_s(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    ctx.reg_file[dst as usize] = lhs.wrapping_div(rhs);
    Ok(())
}

fn i32_reg_div_u(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    let rhs_u = rhs as u32;
    if rhs_u == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    ctx.reg_file[dst as usize] = ((lhs as u32) / rhs_u) as i32;
    Ok(())
}

fn i32_reg_rem_s(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    ctx.reg_file[dst as usize] = lhs.wrapping_rem(rhs);
    Ok(())
}

fn i32_reg_rem_u(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    let rhs_u = rhs as u32;
    if rhs_u == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    ctx.reg_file[dst as usize] = ((lhs as u32) % rhs_u) as i32;
    Ok(())
}

fn i32_reg_and(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = lhs & rhs;
    Ok(())
}

fn i32_reg_or(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = lhs | rhs;
    Ok(())
}

fn i32_reg_xor(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = lhs ^ rhs;
    Ok(())
}

fn i32_reg_shl(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = lhs.wrapping_shl(rhs as u32);
    Ok(())
}

fn i32_reg_shr_s(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = lhs.wrapping_shr(rhs as u32);
    Ok(())
}

fn i32_reg_shr_u(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = ((lhs as u32).wrapping_shr(rhs as u32)) as i32;
    Ok(())
}

fn i32_reg_rotl(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = lhs.rotate_left(rhs as u32);
    Ok(())
}

fn i32_reg_rotr(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = lhs.rotate_right(rhs as u32);
    Ok(())
}

// Comparison handlers
fn i32_reg_eq(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = if lhs == rhs { 1 } else { 0 };
    Ok(())
}

fn i32_reg_ne(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = if lhs != rhs { 1 } else { 0 };
    Ok(())
}

fn i32_reg_lt_s(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = if lhs < rhs { 1 } else { 0 };
    Ok(())
}

fn i32_reg_lt_u(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = if (lhs as u32) < (rhs as u32) { 1 } else { 0 };
    Ok(())
}

fn i32_reg_le_s(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = if lhs <= rhs { 1 } else { 0 };
    Ok(())
}

fn i32_reg_le_u(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = if (lhs as u32) <= (rhs as u32) { 1 } else { 0 };
    Ok(())
}

fn i32_reg_gt_s(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = if lhs > rhs { 1 } else { 0 };
    Ok(())
}

fn i32_reg_gt_u(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = if (lhs as u32) > (rhs as u32) { 1 } else { 0 };
    Ok(())
}

fn i32_reg_ge_s(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = if lhs >= rhs { 1 } else { 0 };
    Ok(())
}

fn i32_reg_ge_u(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.reg_file[dst as usize] = if (lhs as u32) >= (rhs as u32) { 1 } else { 0 };
    Ok(())
}

// Unary operation handlers
fn i32_reg_clz(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.reg_file[dst as usize] = val.leading_zeros() as i32;
    Ok(())
}

fn i32_reg_ctz(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.reg_file[dst as usize] = val.trailing_zeros() as i32;
    Ok(())
}

fn i32_reg_popcnt(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.reg_file[dst as usize] = val.count_ones() as i32;
    Ok(())
}

fn i32_reg_eqz(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.reg_file[dst as usize] = if val == 0 { 1 } else { 0 };
    Ok(())
}

fn i32_reg_extend8_s(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.reg_file[dst as usize] = (val as i8) as i32;
    Ok(())
}

fn i32_reg_extend16_s(ctx: I32RegContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.reg_file[dst as usize] = (val as i16) as i32;
    Ok(())
}

// Handler function type
type I32RegHandler = fn(I32RegContext, u16) -> Result<(), RuntimeError>;

// Default error handler
fn i32_reg_invalid_handler(_ctx: I32RegContext, _dst: u16) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

// I32 Reg Handler Table
lazy_static! {
    static ref I32_REG_HANDLER_TABLE: Vec<I32RegHandler> = {
        let mut table: Vec<I32RegHandler> = vec![i32_reg_invalid_handler; 256];

        // Special handlers
        table[HANDLER_IDX_LOCAL_GET] = i32_reg_local_get;
        table[HANDLER_IDX_LOCAL_SET] = i32_reg_local_set;
        table[HANDLER_IDX_I32_CONST] = i32_reg_const;

        // Binary operations
        table[HANDLER_IDX_I32_ADD] = i32_reg_add;
        table[HANDLER_IDX_I32_SUB] = i32_reg_sub;
        table[HANDLER_IDX_I32_MUL] = i32_reg_mul;
        table[HANDLER_IDX_I32_DIV_S] = i32_reg_div_s;
        table[HANDLER_IDX_I32_DIV_U] = i32_reg_div_u;
        table[HANDLER_IDX_I32_REM_S] = i32_reg_rem_s;
        table[HANDLER_IDX_I32_REM_U] = i32_reg_rem_u;
        table[HANDLER_IDX_I32_AND] = i32_reg_and;
        table[HANDLER_IDX_I32_OR] = i32_reg_or;
        table[HANDLER_IDX_I32_XOR] = i32_reg_xor;
        table[HANDLER_IDX_I32_SHL] = i32_reg_shl;
        table[HANDLER_IDX_I32_SHR_S] = i32_reg_shr_s;
        table[HANDLER_IDX_I32_SHR_U] = i32_reg_shr_u;
        table[HANDLER_IDX_I32_ROTL] = i32_reg_rotl;
        table[HANDLER_IDX_I32_ROTR] = i32_reg_rotr;

        // Comparisons
        table[HANDLER_IDX_I32_EQ] = i32_reg_eq;
        table[HANDLER_IDX_I32_NE] = i32_reg_ne;
        table[HANDLER_IDX_I32_LT_S] = i32_reg_lt_s;
        table[HANDLER_IDX_I32_LT_U] = i32_reg_lt_u;
        table[HANDLER_IDX_I32_LE_S] = i32_reg_le_s;
        table[HANDLER_IDX_I32_LE_U] = i32_reg_le_u;
        table[HANDLER_IDX_I32_GT_S] = i32_reg_gt_s;
        table[HANDLER_IDX_I32_GT_U] = i32_reg_gt_u;
        table[HANDLER_IDX_I32_GE_S] = i32_reg_ge_s;
        table[HANDLER_IDX_I32_GE_U] = i32_reg_ge_u;

        // Unary operations
        table[HANDLER_IDX_I32_CLZ] = i32_reg_clz;
        table[HANDLER_IDX_I32_CTZ] = i32_reg_ctz;
        table[HANDLER_IDX_I32_POPCNT] = i32_reg_popcnt;
        table[HANDLER_IDX_I32_EQZ] = i32_reg_eqz;
        table[HANDLER_IDX_I32_EXTEND8_S] = i32_reg_extend8_s;
        table[HANDLER_IDX_I32_EXTEND16_S] = i32_reg_extend16_s;

        table
    };
}

struct I64RegContext<'a> {
    i64_regs: &'a mut [i64],
    i32_regs: &'a mut [i32], // For comparison operations that return i32
    locals: &'a mut [Val],
    src1: I64RegOperand,
    src2: Option<I64RegOperand>,
    dst: Reg, // Destination register (type determines which array to write to)
}

impl<'a> I64RegContext<'a> {
    #[inline]
    fn get_operand(&self, operand: &I64RegOperand) -> Result<i64, RuntimeError> {
        match operand {
            I64RegOperand::Reg(idx) => Ok(self.i64_regs[*idx as usize]),
            I64RegOperand::Const(val) => Ok(*val),
            I64RegOperand::Param(idx) => self.locals[*idx as usize].to_i64(),
        }
    }
}

fn i64_reg_local_get(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_regs[ctx.dst.index() as usize] = val;
    Ok(())
}

fn i64_reg_local_set(ctx: I64RegContext) -> Result<(), RuntimeError> {
    // dst.index() is the local variable index, src1 is the value source
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.locals[ctx.dst.index() as usize] = Val::Num(Num::I64(val));
    Ok(())
}

fn i64_reg_const(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_regs[ctx.dst.index() as usize] = val;
    Ok(())
}

fn i64_reg_add(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_regs[ctx.dst.index() as usize] = lhs.wrapping_add(rhs);
    Ok(())
}

fn i64_reg_sub(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_regs[ctx.dst.index() as usize] = lhs.wrapping_sub(rhs);
    Ok(())
}

fn i64_reg_mul(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_regs[ctx.dst.index() as usize] = lhs.wrapping_mul(rhs);
    Ok(())
}

fn i64_reg_div_s(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    if lhs == i64::MIN && rhs == -1 {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.i64_regs[ctx.dst.index() as usize] = lhs.wrapping_div(rhs);
    Ok(())
}

fn i64_reg_div_u(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    let rhs_u = rhs as u64;
    if rhs_u == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    ctx.i64_regs[ctx.dst.index() as usize] = ((lhs as u64) / rhs_u) as i64;
    Ok(())
}

fn i64_reg_rem_s(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    ctx.i64_regs[ctx.dst.index() as usize] = lhs.wrapping_rem(rhs);
    Ok(())
}

fn i64_reg_rem_u(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    let rhs_u = rhs as u64;
    if rhs_u == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    ctx.i64_regs[ctx.dst.index() as usize] = ((lhs as u64) % rhs_u) as i64;
    Ok(())
}

fn i64_reg_and(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_regs[ctx.dst.index() as usize] = lhs & rhs;
    Ok(())
}

fn i64_reg_or(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_regs[ctx.dst.index() as usize] = lhs | rhs;
    Ok(())
}

fn i64_reg_xor(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_regs[ctx.dst.index() as usize] = lhs ^ rhs;
    Ok(())
}

fn i64_reg_shl(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_regs[ctx.dst.index() as usize] = lhs.wrapping_shl((rhs & 0x3f) as u32);
    Ok(())
}

fn i64_reg_shr_s(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_regs[ctx.dst.index() as usize] = lhs.wrapping_shr((rhs & 0x3f) as u32);
    Ok(())
}

fn i64_reg_shr_u(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_regs[ctx.dst.index() as usize] =
        ((lhs as u64).wrapping_shr((rhs & 0x3f) as u32)) as i64;
    Ok(())
}

fn i64_reg_rotl(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_regs[ctx.dst.index() as usize] = (lhs as u64).rotate_left((rhs & 0x3f) as u32) as i64;
    Ok(())
}

fn i64_reg_rotr(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_regs[ctx.dst.index() as usize] = (lhs as u64).rotate_right((rhs & 0x3f) as u32) as i64;
    Ok(())
}

fn i64_reg_clz(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_regs[ctx.dst.index() as usize] = val.leading_zeros() as i64;
    Ok(())
}

fn i64_reg_ctz(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_regs[ctx.dst.index() as usize] = val.trailing_zeros() as i64;
    Ok(())
}

fn i64_reg_popcnt(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_regs[ctx.dst.index() as usize] = val.count_ones() as i64;
    Ok(())
}

fn i64_reg_extend8_s(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_regs[ctx.dst.index() as usize] = (val as i8) as i64;
    Ok(())
}

fn i64_reg_extend16_s(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_regs[ctx.dst.index() as usize] = (val as i16) as i64;
    Ok(())
}

fn i64_reg_extend32_s(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_regs[ctx.dst.index() as usize] = (val as i32) as i64;
    Ok(())
}

// Comparison operations - write to i32_regs (ctx.dst is Reg::I32)
fn i64_reg_eq(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs == rhs { 1 } else { 0 };
    Ok(())
}

fn i64_reg_ne(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs != rhs { 1 } else { 0 };
    Ok(())
}

fn i64_reg_lt_s(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs < rhs { 1 } else { 0 };
    Ok(())
}

fn i64_reg_lt_u(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if (lhs as u64) < (rhs as u64) { 1 } else { 0 };
    Ok(())
}

fn i64_reg_le_s(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs <= rhs { 1 } else { 0 };
    Ok(())
}

fn i64_reg_le_u(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if (lhs as u64) <= (rhs as u64) { 1 } else { 0 };
    Ok(())
}

fn i64_reg_gt_s(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs > rhs { 1 } else { 0 };
    Ok(())
}

fn i64_reg_gt_u(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if (lhs as u64) > (rhs as u64) { 1 } else { 0 };
    Ok(())
}

fn i64_reg_ge_s(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs >= rhs { 1 } else { 0 };
    Ok(())
}

fn i64_reg_ge_u(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if (lhs as u64) >= (rhs as u64) { 1 } else { 0 };
    Ok(())
}

fn i64_reg_eqz(ctx: I64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i32_regs[ctx.dst.index() as usize] = if val == 0 { 1 } else { 0 };
    Ok(())
}

type I64RegHandler = fn(I64RegContext) -> Result<(), RuntimeError>;

fn i64_reg_invalid_handler(_ctx: I64RegContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

lazy_static! {
    static ref I64_REG_HANDLER_TABLE: Vec<I64RegHandler> = {
        let mut table: Vec<I64RegHandler> = vec![i64_reg_invalid_handler; 256];

        // Special handlers
        table[HANDLER_IDX_LOCAL_GET] = i64_reg_local_get;
        table[HANDLER_IDX_LOCAL_SET] = i64_reg_local_set;
        table[HANDLER_IDX_I64_CONST] = i64_reg_const;

        // Binary operations
        table[HANDLER_IDX_I64_ADD] = i64_reg_add;
        table[HANDLER_IDX_I64_SUB] = i64_reg_sub;
        table[HANDLER_IDX_I64_MUL] = i64_reg_mul;
        table[HANDLER_IDX_I64_DIV_S] = i64_reg_div_s;
        table[HANDLER_IDX_I64_DIV_U] = i64_reg_div_u;
        table[HANDLER_IDX_I64_REM_S] = i64_reg_rem_s;
        table[HANDLER_IDX_I64_REM_U] = i64_reg_rem_u;
        table[HANDLER_IDX_I64_AND] = i64_reg_and;
        table[HANDLER_IDX_I64_OR] = i64_reg_or;
        table[HANDLER_IDX_I64_XOR] = i64_reg_xor;
        table[HANDLER_IDX_I64_SHL] = i64_reg_shl;
        table[HANDLER_IDX_I64_SHR_S] = i64_reg_shr_s;
        table[HANDLER_IDX_I64_SHR_U] = i64_reg_shr_u;
        table[HANDLER_IDX_I64_ROTL] = i64_reg_rotl;
        table[HANDLER_IDX_I64_ROTR] = i64_reg_rotr;

        // Unary operations
        table[HANDLER_IDX_I64_CLZ] = i64_reg_clz;
        table[HANDLER_IDX_I64_CTZ] = i64_reg_ctz;
        table[HANDLER_IDX_I64_POPCNT] = i64_reg_popcnt;
        table[HANDLER_IDX_I64_EXTEND8_S] = i64_reg_extend8_s;
        table[HANDLER_IDX_I64_EXTEND16_S] = i64_reg_extend16_s;
        table[HANDLER_IDX_I64_EXTEND32_S] = i64_reg_extend32_s;

        // Comparison operations (return i32)
        table[HANDLER_IDX_I64_EQZ] = i64_reg_eqz;
        table[HANDLER_IDX_I64_EQ] = i64_reg_eq;
        table[HANDLER_IDX_I64_NE] = i64_reg_ne;
        table[HANDLER_IDX_I64_LT_S] = i64_reg_lt_s;
        table[HANDLER_IDX_I64_LT_U] = i64_reg_lt_u;
        table[HANDLER_IDX_I64_GT_S] = i64_reg_gt_s;
        table[HANDLER_IDX_I64_GT_U] = i64_reg_gt_u;
        table[HANDLER_IDX_I64_LE_S] = i64_reg_le_s;
        table[HANDLER_IDX_I64_LE_U] = i64_reg_le_u;
        table[HANDLER_IDX_I64_GE_S] = i64_reg_ge_s;
        table[HANDLER_IDX_I64_GE_U] = i64_reg_ge_u;

        table
    };
}

// F32 register-based execution context and handlers
type F32RegHandler = fn(F32RegContext) -> Result<(), RuntimeError>;

fn f32_reg_invalid_handler(_ctx: F32RegContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::Trap)
}

struct F32RegContext<'a> {
    f32_regs: &'a mut [f32],
    i32_regs: &'a mut [i32], // For comparison operations that return i32
    locals: &'a mut [Val],
    src1: F32RegOperand,
    src2: Option<F32RegOperand>,
    dst: Reg, // Destination register (type determines which array to write to)
}

impl<'a> F32RegContext<'a> {
    #[inline]
    fn get_operand(&self, operand: &F32RegOperand) -> Result<f32, RuntimeError> {
        match operand {
            F32RegOperand::Reg(idx) => Ok(self.f32_regs[*idx as usize]),
            F32RegOperand::Const(val) => Ok(*val),
            F32RegOperand::Param(idx) => self.locals[*idx as usize].to_f32(),
        }
    }
}

fn f32_reg_local_get(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_regs[ctx.dst.index() as usize] = val;
    Ok(())
}

fn f32_reg_local_set(ctx: F32RegContext) -> Result<(), RuntimeError> {
    // dst.index() is the local variable index, src1 is the value source
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.locals[ctx.dst.index() as usize] = Val::Num(Num::F32(val));
    Ok(())
}

fn f32_reg_const(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_regs[ctx.dst.index() as usize] = val;
    Ok(())
}

fn f32_reg_add(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f32_regs[ctx.dst.index() as usize] = lhs + rhs;
    Ok(())
}

fn f32_reg_sub(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f32_regs[ctx.dst.index() as usize] = lhs - rhs;
    Ok(())
}

fn f32_reg_mul(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f32_regs[ctx.dst.index() as usize] = lhs * rhs;
    Ok(())
}

fn f32_reg_div(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f32_regs[ctx.dst.index() as usize] = lhs / rhs;
    Ok(())
}

fn f32_reg_min(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f32_regs[ctx.dst.index() as usize] = if lhs.is_nan() || rhs.is_nan() {
        f32::NAN
    } else if lhs == 0.0 && rhs == 0.0 {
        if lhs.is_sign_negative() || rhs.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else {
        lhs.min(rhs)
    };
    Ok(())
}

fn f32_reg_max(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f32_regs[ctx.dst.index() as usize] = if lhs.is_nan() || rhs.is_nan() {
        f32::NAN
    } else if lhs == 0.0 && rhs == 0.0 {
        if lhs.is_sign_positive() || rhs.is_sign_positive() {
            0.0
        } else {
            -0.0
        }
    } else {
        lhs.max(rhs)
    };
    Ok(())
}

fn f32_reg_copysign(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f32_regs[ctx.dst.index() as usize] = lhs.copysign(rhs);
    Ok(())
}

// Unary operations
fn f32_reg_abs(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_regs[ctx.dst.index() as usize] = val.abs();
    Ok(())
}

fn f32_reg_neg(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_regs[ctx.dst.index() as usize] = -val;
    Ok(())
}

fn f32_reg_ceil(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_regs[ctx.dst.index() as usize] = val.ceil();
    Ok(())
}

fn f32_reg_floor(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_regs[ctx.dst.index() as usize] = val.floor();
    Ok(())
}

fn f32_reg_trunc(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_regs[ctx.dst.index() as usize] = val.trunc();
    Ok(())
}

fn f32_reg_nearest(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_regs[ctx.dst.index() as usize] = val.round_ties_even();
    Ok(())
}

fn f32_reg_sqrt(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_regs[ctx.dst.index() as usize] = val.sqrt();
    Ok(())
}

// Comparison operations (return i32)
fn f32_reg_eq(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs == rhs { 1 } else { 0 };
    Ok(())
}

fn f32_reg_ne(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs != rhs { 1 } else { 0 };
    Ok(())
}

fn f32_reg_lt(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs < rhs { 1 } else { 0 };
    Ok(())
}

fn f32_reg_gt(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs > rhs { 1 } else { 0 };
    Ok(())
}

fn f32_reg_le(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs <= rhs { 1 } else { 0 };
    Ok(())
}

fn f32_reg_ge(ctx: F32RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs >= rhs { 1 } else { 0 };
    Ok(())
}

lazy_static! {
    static ref F32_REG_HANDLER_TABLE: Vec<F32RegHandler> = {
        let mut table: Vec<F32RegHandler> = vec![f32_reg_invalid_handler; 256];

        // Special handlers
        table[HANDLER_IDX_LOCAL_GET] = f32_reg_local_get;
        table[HANDLER_IDX_LOCAL_SET] = f32_reg_local_set;
        table[HANDLER_IDX_F32_CONST] = f32_reg_const;

        // Binary operations
        table[HANDLER_IDX_F32_ADD] = f32_reg_add;
        table[HANDLER_IDX_F32_SUB] = f32_reg_sub;
        table[HANDLER_IDX_F32_MUL] = f32_reg_mul;
        table[HANDLER_IDX_F32_DIV] = f32_reg_div;
        table[HANDLER_IDX_F32_MIN] = f32_reg_min;
        table[HANDLER_IDX_F32_MAX] = f32_reg_max;
        table[HANDLER_IDX_F32_COPYSIGN] = f32_reg_copysign;

        // Unary operations
        table[HANDLER_IDX_F32_ABS] = f32_reg_abs;
        table[HANDLER_IDX_F32_NEG] = f32_reg_neg;
        table[HANDLER_IDX_F32_CEIL] = f32_reg_ceil;
        table[HANDLER_IDX_F32_FLOOR] = f32_reg_floor;
        table[HANDLER_IDX_F32_TRUNC] = f32_reg_trunc;
        table[HANDLER_IDX_F32_NEAREST] = f32_reg_nearest;
        table[HANDLER_IDX_F32_SQRT] = f32_reg_sqrt;

        // Comparison operations (return i32)
        table[HANDLER_IDX_F32_EQ] = f32_reg_eq;
        table[HANDLER_IDX_F32_NE] = f32_reg_ne;
        table[HANDLER_IDX_F32_LT] = f32_reg_lt;
        table[HANDLER_IDX_F32_GT] = f32_reg_gt;
        table[HANDLER_IDX_F32_LE] = f32_reg_le;
        table[HANDLER_IDX_F32_GE] = f32_reg_ge;

        table
    };
}

// F64 register-based execution context and handlers
type F64RegHandler = fn(F64RegContext) -> Result<(), RuntimeError>;

fn f64_reg_invalid_handler(_ctx: F64RegContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::Trap)
}

struct F64RegContext<'a> {
    f64_regs: &'a mut [f64],
    i32_regs: &'a mut [i32], // For comparison operations that return i32
    locals: &'a mut [Val],
    src1: F64RegOperand,
    src2: Option<F64RegOperand>,
    dst: Reg,
}

impl<'a> F64RegContext<'a> {
    #[inline]
    fn get_operand(&self, operand: &F64RegOperand) -> Result<f64, RuntimeError> {
        match operand {
            F64RegOperand::Reg(idx) => Ok(self.f64_regs[*idx as usize]),
            F64RegOperand::Const(val) => Ok(*val),
            F64RegOperand::Param(idx) => self.locals[*idx as usize].to_f64(),
        }
    }
}

fn f64_reg_local_get(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_regs[ctx.dst.index() as usize] = val;
    Ok(())
}

fn f64_reg_local_set(ctx: F64RegContext) -> Result<(), RuntimeError> {
    // dst.index() is the local variable index, src1 is the value source
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.locals[ctx.dst.index() as usize] = Val::Num(Num::F64(val));
    Ok(())
}

fn f64_reg_const(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_regs[ctx.dst.index() as usize] = val;
    Ok(())
}

fn f64_reg_add(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f64_regs[ctx.dst.index() as usize] = lhs + rhs;
    Ok(())
}

fn f64_reg_sub(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f64_regs[ctx.dst.index() as usize] = lhs - rhs;
    Ok(())
}

fn f64_reg_mul(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f64_regs[ctx.dst.index() as usize] = lhs * rhs;
    Ok(())
}

fn f64_reg_div(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f64_regs[ctx.dst.index() as usize] = lhs / rhs;
    Ok(())
}

fn f64_reg_min(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f64_regs[ctx.dst.index() as usize] = if lhs.is_nan() || rhs.is_nan() {
        f64::NAN
    } else if lhs == 0.0 && rhs == 0.0 {
        if lhs.is_sign_negative() || rhs.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else {
        lhs.min(rhs)
    };
    Ok(())
}

fn f64_reg_max(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f64_regs[ctx.dst.index() as usize] = if lhs.is_nan() || rhs.is_nan() {
        f64::NAN
    } else if lhs == 0.0 && rhs == 0.0 {
        if lhs.is_sign_positive() || rhs.is_sign_positive() {
            0.0
        } else {
            -0.0
        }
    } else {
        lhs.max(rhs)
    };
    Ok(())
}

fn f64_reg_copysign(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f64_regs[ctx.dst.index() as usize] = lhs.copysign(rhs);
    Ok(())
}

// Unary operations
fn f64_reg_abs(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_regs[ctx.dst.index() as usize] = val.abs();
    Ok(())
}

fn f64_reg_neg(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_regs[ctx.dst.index() as usize] = -val;
    Ok(())
}

fn f64_reg_ceil(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_regs[ctx.dst.index() as usize] = val.ceil();
    Ok(())
}

fn f64_reg_floor(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_regs[ctx.dst.index() as usize] = val.floor();
    Ok(())
}

fn f64_reg_trunc(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_regs[ctx.dst.index() as usize] = val.trunc();
    Ok(())
}

fn f64_reg_nearest(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_regs[ctx.dst.index() as usize] = val.round_ties_even();
    Ok(())
}

fn f64_reg_sqrt(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_regs[ctx.dst.index() as usize] = val.sqrt();
    Ok(())
}

// Comparison operations (return i32)
fn f64_reg_eq(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs == rhs { 1 } else { 0 };
    Ok(())
}

fn f64_reg_ne(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs != rhs { 1 } else { 0 };
    Ok(())
}

fn f64_reg_lt(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs < rhs { 1 } else { 0 };
    Ok(())
}

fn f64_reg_gt(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs > rhs { 1 } else { 0 };
    Ok(())
}

fn f64_reg_le(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs <= rhs { 1 } else { 0 };
    Ok(())
}

fn f64_reg_ge(ctx: F64RegContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_regs[ctx.dst.index() as usize] = if lhs >= rhs { 1 } else { 0 };
    Ok(())
}

lazy_static! {
    static ref F64_REG_HANDLER_TABLE: Vec<F64RegHandler> = {
        let mut table: Vec<F64RegHandler> = vec![f64_reg_invalid_handler; 256];

        // Special handlers
        table[HANDLER_IDX_LOCAL_GET] = f64_reg_local_get;
        table[HANDLER_IDX_LOCAL_SET] = f64_reg_local_set;
        table[HANDLER_IDX_F64_CONST] = f64_reg_const;

        // Binary operations
        table[HANDLER_IDX_F64_ADD] = f64_reg_add;
        table[HANDLER_IDX_F64_SUB] = f64_reg_sub;
        table[HANDLER_IDX_F64_MUL] = f64_reg_mul;
        table[HANDLER_IDX_F64_DIV] = f64_reg_div;
        table[HANDLER_IDX_F64_MIN] = f64_reg_min;
        table[HANDLER_IDX_F64_MAX] = f64_reg_max;
        table[HANDLER_IDX_F64_COPYSIGN] = f64_reg_copysign;

        // Unary operations
        table[HANDLER_IDX_F64_ABS] = f64_reg_abs;
        table[HANDLER_IDX_F64_NEG] = f64_reg_neg;
        table[HANDLER_IDX_F64_CEIL] = f64_reg_ceil;
        table[HANDLER_IDX_F64_FLOOR] = f64_reg_floor;
        table[HANDLER_IDX_F64_TRUNC] = f64_reg_trunc;
        table[HANDLER_IDX_F64_NEAREST] = f64_reg_nearest;
        table[HANDLER_IDX_F64_SQRT] = f64_reg_sqrt;

        // Comparison operations (return i32)
        table[HANDLER_IDX_F64_EQ] = f64_reg_eq;
        table[HANDLER_IDX_F64_NE] = f64_reg_ne;
        table[HANDLER_IDX_F64_LT] = f64_reg_lt;
        table[HANDLER_IDX_F64_GT] = f64_reg_gt;
        table[HANDLER_IDX_F64_LE] = f64_reg_le;
        table[HANDLER_IDX_F64_GE] = f64_reg_ge;

        table
    };
}

// Conversion Reg handlers
struct ConversionRegContext<'a> {
    reg_file: &'a mut RegFile,
    src: Reg,
    dst: Reg,
}

type ConversionRegHandler = fn(ConversionRegContext) -> Result<(), RuntimeError>;

fn conversion_reg_invalid_handler(_ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

// i32 -> i64
fn conv_i64_extend_i32_s(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_i32(ctx.src.index());
    ctx.reg_file.set_i64(ctx.dst.index(), val as i64);
    Ok(())
}

fn conv_i64_extend_i32_u(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_i32(ctx.src.index());
    ctx.reg_file.set_i64(ctx.dst.index(), (val as u32) as i64);
    Ok(())
}

// i64 -> i32
fn conv_i32_wrap_i64(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_i64(ctx.src.index());
    ctx.reg_file.set_i32(ctx.dst.index(), val as i32);
    Ok(())
}

// f32 -> i32
fn conv_i32_trunc_f32_s(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f32(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < (i32::MIN as f32) || truncated > (i32::MAX as f32) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.reg_file.set_i32(ctx.dst.index(), truncated as i32);
    Ok(())
}

fn conv_i32_trunc_f32_u(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f32(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < 0.0 || truncated > (u32::MAX as f32) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.reg_file
        .set_i32(ctx.dst.index(), (truncated as u32) as i32);
    Ok(())
}

// f64 -> i32
fn conv_i32_trunc_f64_s(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f64(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < (i32::MIN as f64) || truncated > (i32::MAX as f64) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.reg_file.set_i32(ctx.dst.index(), truncated as i32);
    Ok(())
}

fn conv_i32_trunc_f64_u(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f64(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < 0.0 || truncated > (u32::MAX as f64) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.reg_file
        .set_i32(ctx.dst.index(), (truncated as u32) as i32);
    Ok(())
}

// f32 -> i64
fn conv_i64_trunc_f32_s(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f32(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < (i64::MIN as f32) || truncated >= (i64::MAX as f32) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.reg_file.set_i64(ctx.dst.index(), truncated as i64);
    Ok(())
}

fn conv_i64_trunc_f32_u(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f32(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < 0.0 || truncated >= (u64::MAX as f32) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.reg_file
        .set_i64(ctx.dst.index(), (truncated as u64) as i64);
    Ok(())
}

// f64 -> i64
fn conv_i64_trunc_f64_s(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f64(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < (i64::MIN as f64) || truncated >= (i64::MAX as f64) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.reg_file.set_i64(ctx.dst.index(), truncated as i64);
    Ok(())
}

fn conv_i64_trunc_f64_u(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f64(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < 0.0 || truncated >= (u64::MAX as f64) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.reg_file
        .set_i64(ctx.dst.index(), (truncated as u64) as i64);
    Ok(())
}

// Saturating truncations (i32)
fn conv_i32_trunc_sat_f32_s(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f32(ctx.src.index());
    let result = if val.is_nan() {
        0
    } else if val <= (i32::MIN as f32) {
        i32::MIN
    } else if val >= (i32::MAX as f32) {
        i32::MAX
    } else {
        val.trunc() as i32
    };
    ctx.reg_file.set_i32(ctx.dst.index(), result);
    Ok(())
}

fn conv_i32_trunc_sat_f32_u(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f32(ctx.src.index());
    let result = if val.is_nan() || val <= 0.0 {
        0
    } else if val >= (u32::MAX as f32) {
        u32::MAX
    } else {
        val.trunc() as u32
    };
    ctx.reg_file.set_i32(ctx.dst.index(), result as i32);
    Ok(())
}

fn conv_i32_trunc_sat_f64_s(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f64(ctx.src.index());
    let result = if val.is_nan() {
        0
    } else if val <= (i32::MIN as f64) {
        i32::MIN
    } else if val >= (i32::MAX as f64) {
        i32::MAX
    } else {
        val.trunc() as i32
    };
    ctx.reg_file.set_i32(ctx.dst.index(), result);
    Ok(())
}

fn conv_i32_trunc_sat_f64_u(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f64(ctx.src.index());
    let result = if val.is_nan() || val <= 0.0 {
        0
    } else if val >= (u32::MAX as f64) {
        u32::MAX
    } else {
        val.trunc() as u32
    };
    ctx.reg_file.set_i32(ctx.dst.index(), result as i32);
    Ok(())
}

// Saturating truncations (i64)
fn conv_i64_trunc_sat_f32_s(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f32(ctx.src.index());
    let result = if val.is_nan() {
        0
    } else if val <= (i64::MIN as f32) {
        i64::MIN
    } else if val >= (i64::MAX as f32) {
        i64::MAX
    } else {
        val.trunc() as i64
    };
    ctx.reg_file.set_i64(ctx.dst.index(), result);
    Ok(())
}

fn conv_i64_trunc_sat_f32_u(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f32(ctx.src.index());
    let result = if val.is_nan() || val <= 0.0 {
        0
    } else if val >= (u64::MAX as f32) {
        u64::MAX
    } else {
        val.trunc() as u64
    };
    ctx.reg_file.set_i64(ctx.dst.index(), result as i64);
    Ok(())
}

fn conv_i64_trunc_sat_f64_s(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f64(ctx.src.index());
    let result = if val.is_nan() {
        0
    } else if val <= (i64::MIN as f64) {
        i64::MIN
    } else if val >= (i64::MAX as f64) {
        i64::MAX
    } else {
        val.trunc() as i64
    };
    ctx.reg_file.set_i64(ctx.dst.index(), result);
    Ok(())
}

fn conv_i64_trunc_sat_f64_u(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f64(ctx.src.index());
    let result = if val.is_nan() || val <= 0.0 {
        0
    } else if val >= (u64::MAX as f64) {
        u64::MAX
    } else {
        val.trunc() as u64
    };
    ctx.reg_file.set_i64(ctx.dst.index(), result as i64);
    Ok(())
}

// i32 -> f32
fn conv_f32_convert_i32_s(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_i32(ctx.src.index());
    ctx.reg_file.set_f32(ctx.dst.index(), val as f32);
    Ok(())
}

fn conv_f32_convert_i32_u(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_i32(ctx.src.index());
    ctx.reg_file.set_f32(ctx.dst.index(), (val as u32) as f32);
    Ok(())
}

// i64 -> f32
fn conv_f32_convert_i64_s(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_i64(ctx.src.index());
    ctx.reg_file.set_f32(ctx.dst.index(), val as f32);
    Ok(())
}

fn conv_f32_convert_i64_u(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_i64(ctx.src.index());
    ctx.reg_file.set_f32(ctx.dst.index(), (val as u64) as f32);
    Ok(())
}

// i32 -> f64
fn conv_f64_convert_i32_s(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_i32(ctx.src.index());
    ctx.reg_file.set_f64(ctx.dst.index(), val as f64);
    Ok(())
}

fn conv_f64_convert_i32_u(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_i32(ctx.src.index());
    ctx.reg_file.set_f64(ctx.dst.index(), (val as u32) as f64);
    Ok(())
}

// i64 -> f64
fn conv_f64_convert_i64_s(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_i64(ctx.src.index());
    ctx.reg_file.set_f64(ctx.dst.index(), val as f64);
    Ok(())
}

fn conv_f64_convert_i64_u(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_i64(ctx.src.index());
    ctx.reg_file.set_f64(ctx.dst.index(), (val as u64) as f64);
    Ok(())
}

// f64 -> f32
fn conv_f32_demote_f64(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f64(ctx.src.index());
    ctx.reg_file.set_f32(ctx.dst.index(), val as f32);
    Ok(())
}

// f32 -> f64
fn conv_f64_promote_f32(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f32(ctx.src.index());
    ctx.reg_file.set_f64(ctx.dst.index(), val as f64);
    Ok(())
}

// Reinterpret operations
fn conv_i32_reinterpret_f32(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f32(ctx.src.index());
    ctx.reg_file.set_i32(ctx.dst.index(), val.to_bits() as i32);
    Ok(())
}

fn conv_f32_reinterpret_i32(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_i32(ctx.src.index());
    ctx.reg_file
        .set_f32(ctx.dst.index(), f32::from_bits(val as u32));
    Ok(())
}

fn conv_i64_reinterpret_f64(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_f64(ctx.src.index());
    ctx.reg_file.set_i64(ctx.dst.index(), val.to_bits() as i64);
    Ok(())
}

fn conv_f64_reinterpret_i64(ctx: ConversionRegContext) -> Result<(), RuntimeError> {
    let val = ctx.reg_file.get_i64(ctx.src.index());
    ctx.reg_file
        .set_f64(ctx.dst.index(), f64::from_bits(val as u64));
    Ok(())
}

lazy_static! {
    static ref CONVERSION_REG_HANDLER_TABLE: Vec<ConversionRegHandler> = {
        let mut table: Vec<ConversionRegHandler> = vec![conversion_reg_invalid_handler; 256];

        // Integer conversions
        table[HANDLER_IDX_I64_EXTEND_I32_S] = conv_i64_extend_i32_s;
        table[HANDLER_IDX_I64_EXTEND_I32_U] = conv_i64_extend_i32_u;
        table[HANDLER_IDX_I32_WRAP_I64] = conv_i32_wrap_i64;

        // Float to integer (trapping)
        table[HANDLER_IDX_I32_TRUNC_F32_S] = conv_i32_trunc_f32_s;
        table[HANDLER_IDX_I32_TRUNC_F32_U] = conv_i32_trunc_f32_u;
        table[HANDLER_IDX_I32_TRUNC_F64_S] = conv_i32_trunc_f64_s;
        table[HANDLER_IDX_I32_TRUNC_F64_U] = conv_i32_trunc_f64_u;
        table[HANDLER_IDX_I64_TRUNC_F32_S] = conv_i64_trunc_f32_s;
        table[HANDLER_IDX_I64_TRUNC_F32_U] = conv_i64_trunc_f32_u;
        table[HANDLER_IDX_I64_TRUNC_F64_S] = conv_i64_trunc_f64_s;
        table[HANDLER_IDX_I64_TRUNC_F64_U] = conv_i64_trunc_f64_u;

        // Float to integer (saturating)
        table[HANDLER_IDX_I32_TRUNC_SAT_F32_S] = conv_i32_trunc_sat_f32_s;
        table[HANDLER_IDX_I32_TRUNC_SAT_F32_U] = conv_i32_trunc_sat_f32_u;
        table[HANDLER_IDX_I32_TRUNC_SAT_F64_S] = conv_i32_trunc_sat_f64_s;
        table[HANDLER_IDX_I32_TRUNC_SAT_F64_U] = conv_i32_trunc_sat_f64_u;
        table[HANDLER_IDX_I64_TRUNC_SAT_F32_S] = conv_i64_trunc_sat_f32_s;
        table[HANDLER_IDX_I64_TRUNC_SAT_F32_U] = conv_i64_trunc_sat_f32_u;
        table[HANDLER_IDX_I64_TRUNC_SAT_F64_S] = conv_i64_trunc_sat_f64_s;
        table[HANDLER_IDX_I64_TRUNC_SAT_F64_U] = conv_i64_trunc_sat_f64_u;

        // Integer to float
        table[HANDLER_IDX_F32_CONVERT_I32_S] = conv_f32_convert_i32_s;
        table[HANDLER_IDX_F32_CONVERT_I32_U] = conv_f32_convert_i32_u;
        table[HANDLER_IDX_F32_CONVERT_I64_S] = conv_f32_convert_i64_s;
        table[HANDLER_IDX_F32_CONVERT_I64_U] = conv_f32_convert_i64_u;
        table[HANDLER_IDX_F64_CONVERT_I32_S] = conv_f64_convert_i32_s;
        table[HANDLER_IDX_F64_CONVERT_I32_U] = conv_f64_convert_i32_u;
        table[HANDLER_IDX_F64_CONVERT_I64_S] = conv_f64_convert_i64_s;
        table[HANDLER_IDX_F64_CONVERT_I64_U] = conv_f64_convert_i64_u;

        // Float conversions
        table[HANDLER_IDX_F32_DEMOTE_F64] = conv_f32_demote_f64;
        table[HANDLER_IDX_F64_PROMOTE_F32] = conv_f64_promote_f32;

        // Reinterpret
        table[HANDLER_IDX_I32_REINTERPRET_F32] = conv_i32_reinterpret_f32;
        table[HANDLER_IDX_F32_REINTERPRET_I32] = conv_f32_reinterpret_i32;
        table[HANDLER_IDX_I64_REINTERPRET_F64] = conv_i64_reinterpret_f64;
        table[HANDLER_IDX_F64_REINTERPRET_I64] = conv_f64_reinterpret_i64;

        table
    };
}

// Memory Load Reg handlers
struct MemoryLoadRegContext<'a> {
    reg_file: &'a mut RegFile,
    mem_addr: &'a MemAddr,
    addr: Reg,
    dst: Reg,
    offset: u64,
}

type MemoryLoadRegHandler = fn(MemoryLoadRegContext) -> Result<(), RuntimeError>;

fn memory_load_reg_invalid_handler(_ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

#[inline]
fn make_memarg(offset: u64) -> Memarg {
    Memarg {
        offset: offset as u32,
        align: 0,
    }
}

fn mem_load_i32(ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: i32 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.reg_file.set_i32(ctx.dst.index(), val);
    Ok(())
}

fn mem_load_i64(ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: i64 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.reg_file.set_i64(ctx.dst.index(), val);
    Ok(())
}

fn mem_load_f32(ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: f32 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.reg_file.set_f32(ctx.dst.index(), val);
    Ok(())
}

fn mem_load_f64(ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: f64 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.reg_file.set_f64(ctx.dst.index(), val);
    Ok(())
}

fn mem_load_i32_8s(ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: i8 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.reg_file.set_i32(ctx.dst.index(), val as i32);
    Ok(())
}

fn mem_load_i32_8u(ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: u8 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.reg_file.set_i32(ctx.dst.index(), val as i32);
    Ok(())
}

fn mem_load_i32_16s(ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: i16 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.reg_file.set_i32(ctx.dst.index(), val as i32);
    Ok(())
}

fn mem_load_i32_16u(ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: u16 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.reg_file.set_i32(ctx.dst.index(), val as i32);
    Ok(())
}

fn mem_load_i64_8s(ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: i8 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.reg_file.set_i64(ctx.dst.index(), val as i64);
    Ok(())
}

fn mem_load_i64_8u(ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: u8 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.reg_file.set_i64(ctx.dst.index(), val as i64);
    Ok(())
}

fn mem_load_i64_16s(ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: i16 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.reg_file.set_i64(ctx.dst.index(), val as i64);
    Ok(())
}

fn mem_load_i64_16u(ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: u16 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.reg_file.set_i64(ctx.dst.index(), val as i64);
    Ok(())
}

fn mem_load_i64_32s(ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: i32 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.reg_file.set_i64(ctx.dst.index(), val as i64);
    Ok(())
}

fn mem_load_i64_32u(ctx: MemoryLoadRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: u32 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.reg_file.set_i64(ctx.dst.index(), val as i64);
    Ok(())
}

lazy_static! {
    static ref MEMORY_LOAD_REG_HANDLER_TABLE: Vec<MemoryLoadRegHandler> = {
        let mut table: Vec<MemoryLoadRegHandler> = vec![memory_load_reg_invalid_handler; 256];

        table[HANDLER_IDX_I32_LOAD] = mem_load_i32;
        table[HANDLER_IDX_I64_LOAD] = mem_load_i64;
        table[HANDLER_IDX_F32_LOAD] = mem_load_f32;
        table[HANDLER_IDX_F64_LOAD] = mem_load_f64;
        table[HANDLER_IDX_I32_LOAD8_S] = mem_load_i32_8s;
        table[HANDLER_IDX_I32_LOAD8_U] = mem_load_i32_8u;
        table[HANDLER_IDX_I32_LOAD16_S] = mem_load_i32_16s;
        table[HANDLER_IDX_I32_LOAD16_U] = mem_load_i32_16u;
        table[HANDLER_IDX_I64_LOAD8_S] = mem_load_i64_8s;
        table[HANDLER_IDX_I64_LOAD8_U] = mem_load_i64_8u;
        table[HANDLER_IDX_I64_LOAD16_S] = mem_load_i64_16s;
        table[HANDLER_IDX_I64_LOAD16_U] = mem_load_i64_16u;
        table[HANDLER_IDX_I64_LOAD32_S] = mem_load_i64_32s;
        table[HANDLER_IDX_I64_LOAD32_U] = mem_load_i64_32u;

        table
    };
}

// Memory Store Reg handlers
struct MemoryStoreRegContext<'a> {
    reg_file: &'a RegFile,
    mem_addr: &'a MemAddr,
    addr: Reg,
    value: Reg,
    offset: u64,
}

type MemoryStoreRegHandler = fn(MemoryStoreRegContext) -> Result<(), RuntimeError>;

fn memory_store_reg_invalid_handler(_ctx: MemoryStoreRegContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

fn mem_store_i32(ctx: MemoryStoreRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let val = ctx.reg_file.get_i32(ctx.value.index());
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_i64(ctx: MemoryStoreRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let val = ctx.reg_file.get_i64(ctx.value.index());
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_f32(ctx: MemoryStoreRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let val = ctx.reg_file.get_f32(ctx.value.index());
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_f64(ctx: MemoryStoreRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let val = ctx.reg_file.get_f64(ctx.value.index());
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_i32_8(ctx: MemoryStoreRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let val = ctx.reg_file.get_i32(ctx.value.index()) as u8;
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_i32_16(ctx: MemoryStoreRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let val = ctx.reg_file.get_i32(ctx.value.index()) as u16;
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_i64_8(ctx: MemoryStoreRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let val = ctx.reg_file.get_i64(ctx.value.index()) as u8;
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_i64_16(ctx: MemoryStoreRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let val = ctx.reg_file.get_i64(ctx.value.index()) as u16;
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_i64_32(ctx: MemoryStoreRegContext) -> Result<(), RuntimeError> {
    let ptr = ctx.reg_file.get_i32(ctx.addr.index());
    let val = ctx.reg_file.get_i64(ctx.value.index()) as u32;
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

lazy_static! {
    static ref MEMORY_STORE_REG_HANDLER_TABLE: Vec<MemoryStoreRegHandler> = {
        let mut table: Vec<MemoryStoreRegHandler> = vec![memory_store_reg_invalid_handler; 256];

        table[HANDLER_IDX_I32_STORE] = mem_store_i32;
        table[HANDLER_IDX_I64_STORE] = mem_store_i64;
        table[HANDLER_IDX_F32_STORE] = mem_store_f32;
        table[HANDLER_IDX_F64_STORE] = mem_store_f64;
        table[HANDLER_IDX_I32_STORE8] = mem_store_i32_8;
        table[HANDLER_IDX_I32_STORE16] = mem_store_i32_16;
        table[HANDLER_IDX_I64_STORE8] = mem_store_i64_8;
        table[HANDLER_IDX_I64_STORE16] = mem_store_i64_16;
        table[HANDLER_IDX_I64_STORE32] = mem_store_i64_32;

        table
    };
}

// Memory Ops Reg handlers (size, grow, copy, init, fill)
struct MemoryOpsRegContext<'a> {
    reg_file: &'a mut RegFile,
    mem_addr: &'a MemAddr,
    module_inst: &'a ModuleInst,
    dst: Option<Reg>,
    args: Vec<Reg>,
    data_index: u32,
}

type MemoryOpsRegHandler = fn(MemoryOpsRegContext) -> Result<(), RuntimeError>;

fn memory_ops_reg_invalid_handler(_ctx: MemoryOpsRegContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

fn mem_ops_size(ctx: MemoryOpsRegContext) -> Result<(), RuntimeError> {
    let size = ctx.mem_addr.mem_size();
    if let Some(dst) = ctx.dst {
        ctx.reg_file.set_i32(dst.index(), size);
    }
    Ok(())
}

fn mem_ops_grow(ctx: MemoryOpsRegContext) -> Result<(), RuntimeError> {
    let delta = ctx.reg_file.get_i32(ctx.args[0].index());
    let delta_u32: u32 = delta
        .try_into()
        .map_err(|_| RuntimeError::InvalidParameterCount)?;
    let prev_size = ctx.mem_addr.mem_grow(
        delta_u32
            .try_into()
            .map_err(|_| RuntimeError::InvalidParameterCount)?,
    );
    if let Some(dst) = ctx.dst {
        ctx.reg_file.set_i32(dst.index(), prev_size);
    }
    Ok(())
}

fn mem_ops_copy(ctx: MemoryOpsRegContext) -> Result<(), RuntimeError> {
    let dest = ctx.reg_file.get_i32(ctx.args[0].index());
    let src = ctx.reg_file.get_i32(ctx.args[1].index());
    let len = ctx.reg_file.get_i32(ctx.args[2].index());
    ctx.mem_addr.memory_copy(dest, src, len)?;
    Ok(())
}

fn mem_ops_init(ctx: MemoryOpsRegContext) -> Result<(), RuntimeError> {
    let dest = ctx.reg_file.get_i32(ctx.args[0].index()) as usize;
    let offset = ctx.reg_file.get_i32(ctx.args[1].index()) as usize;
    let len = ctx.reg_file.get_i32(ctx.args[2].index()) as usize;

    if ctx.data_index as usize >= ctx.module_inst.data_addrs.len() {
        return Err(RuntimeError::InvalidDataSegmentIndex);
    }

    let data_addr = &ctx.module_inst.data_addrs[ctx.data_index as usize];
    let data_bytes = data_addr.get_data();

    if len > 0 {
        let init_data = data_bytes[offset..offset + len].to_vec();
        ctx.mem_addr.init(dest, &init_data);
    }
    Ok(())
}

fn mem_ops_fill(ctx: MemoryOpsRegContext) -> Result<(), RuntimeError> {
    let dest = ctx.reg_file.get_i32(ctx.args[0].index());
    let val = ctx.reg_file.get_i32(ctx.args[1].index()) as u8;
    let size = ctx.reg_file.get_i32(ctx.args[2].index());
    ctx.mem_addr.memory_fill(dest, val, size)?;
    Ok(())
}

lazy_static! {
    static ref MEMORY_OPS_REG_HANDLER_TABLE: Vec<MemoryOpsRegHandler> = {
        let mut table: Vec<MemoryOpsRegHandler> = vec![memory_ops_reg_invalid_handler; 256];

        table[HANDLER_IDX_MEMORY_SIZE] = mem_ops_size;
        table[HANDLER_IDX_MEMORY_GROW] = mem_ops_grow;
        table[HANDLER_IDX_MEMORY_COPY] = mem_ops_copy;
        table[HANDLER_IDX_MEMORY_INIT] = mem_ops_init;
        table[HANDLER_IDX_MEMORY_FILL] = mem_ops_fill;

        table
    };
}

// Select Reg handlers
struct SelectRegContext<'a> {
    reg_file: &'a mut RegFile,
    dst: Reg,
    val1: Reg,
    val2: Reg,
    cond: Reg,
}

type SelectRegHandler = fn(SelectRegContext) -> Result<(), RuntimeError>;

fn select_reg_invalid_handler(_ctx: SelectRegContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

fn select_i32(ctx: SelectRegContext) -> Result<(), RuntimeError> {
    let cond = ctx.reg_file.get_i32(ctx.cond.index());
    let result = if cond != 0 {
        ctx.reg_file.get_i32(ctx.val1.index())
    } else {
        ctx.reg_file.get_i32(ctx.val2.index())
    };
    ctx.reg_file.set_i32(ctx.dst.index(), result);
    Ok(())
}

fn select_i64(ctx: SelectRegContext) -> Result<(), RuntimeError> {
    let cond = ctx.reg_file.get_i32(ctx.cond.index());
    let result = if cond != 0 {
        ctx.reg_file.get_i64(ctx.val1.index())
    } else {
        ctx.reg_file.get_i64(ctx.val2.index())
    };
    ctx.reg_file.set_i64(ctx.dst.index(), result);
    Ok(())
}

fn select_f32(ctx: SelectRegContext) -> Result<(), RuntimeError> {
    let cond = ctx.reg_file.get_i32(ctx.cond.index());
    let result = if cond != 0 {
        ctx.reg_file.get_f32(ctx.val1.index())
    } else {
        ctx.reg_file.get_f32(ctx.val2.index())
    };
    ctx.reg_file.set_f32(ctx.dst.index(), result);
    Ok(())
}

fn select_f64(ctx: SelectRegContext) -> Result<(), RuntimeError> {
    let cond = ctx.reg_file.get_i32(ctx.cond.index());
    let result = if cond != 0 {
        ctx.reg_file.get_f64(ctx.val1.index())
    } else {
        ctx.reg_file.get_f64(ctx.val2.index())
    };
    ctx.reg_file.set_f64(ctx.dst.index(), result);
    Ok(())
}

pub const HANDLER_IDX_SELECT_I32: usize = 0xF0;
pub const HANDLER_IDX_SELECT_I64: usize = 0xF1;
pub const HANDLER_IDX_SELECT_F32: usize = 0xF2;
pub const HANDLER_IDX_SELECT_F64: usize = 0xF3;

lazy_static! {
    static ref SELECT_REG_HANDLER_TABLE: Vec<SelectRegHandler> = {
        let mut table: Vec<SelectRegHandler> = vec![select_reg_invalid_handler; 256];

        table[HANDLER_IDX_SELECT_I32] = select_i32;
        table[HANDLER_IDX_SELECT_I64] = select_i64;
        table[HANDLER_IDX_SELECT_F32] = select_f32;
        table[HANDLER_IDX_SELECT_F64] = select_f64;

        table
    };
}

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

// RefLocalReg handler infrastructure
struct RefLocalRegContext<'a> {
    reg_file: &'a mut RegFile,
    locals: &'a mut Vec<Val>,
    dst: u16,
    src: u16,
    local_idx: u16,
}

type RefLocalRegHandler = fn(RefLocalRegContext) -> Result<(), RuntimeError>;

fn ref_local_reg_invalid_handler(_ctx: RefLocalRegContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

// local.get for ref type: [] -> [ref]
fn ref_local_reg_get(ctx: RefLocalRegContext) -> Result<(), RuntimeError> {
    let val = ctx
        .locals
        .get(ctx.local_idx as usize)
        .ok_or(RuntimeError::LocalIndexOutOfBounds)?
        .clone();
    if let Val::Ref(r) = val {
        ctx.reg_file.set_ref(ctx.dst, r);
    }
    Ok(())
}

// local.set for ref type: [ref] -> []
fn ref_local_reg_set(ctx: RefLocalRegContext) -> Result<(), RuntimeError> {
    let ref_val = ctx.reg_file.get_ref(ctx.src);
    let idx = ctx.local_idx as usize;
    if idx < ctx.locals.len() {
        ctx.locals[idx] = Val::Ref(ref_val);
    }
    Ok(())
}

lazy_static! {
    static ref REF_LOCAL_REG_HANDLER_TABLE: Vec<RefLocalRegHandler> = {
        let mut table: Vec<RefLocalRegHandler> = vec![ref_local_reg_invalid_handler; 0x103];

        table[HANDLER_IDX_REF_LOCAL_GET_REG] = ref_local_reg_get;
        table[HANDLER_IDX_REF_LOCAL_SET_REG] = ref_local_reg_set;

        table
    };
}

// TableRefReg handler infrastructure
struct TableRefRegContext<'a> {
    reg_file: &'a mut RegFile,
    module_inst: &'a Rc<ModuleInst>,
    table_idx: u32,
    regs: [u16; 3],
    #[allow(dead_code)]
    ref_type: RefType,
}

type TableRefRegHandler = fn(TableRefRegContext) -> Result<(), RuntimeError>;

fn table_ref_reg_invalid_handler(_ctx: TableRefRegContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

// ref.null: [] -> [ref]
// dst = regs[0]
fn table_ref_reg_null(ctx: TableRefRegContext) -> Result<(), RuntimeError> {
    ctx.reg_file.set_ref(ctx.regs[0], Ref::RefNull);
    Ok(())
}

// ref.is_null: [ref] -> [i32]
// src = regs[1], dst = regs[0]
fn table_ref_reg_is_null(ctx: TableRefRegContext) -> Result<(), RuntimeError> {
    let ref_val = ctx.reg_file.get_ref(ctx.regs[1]);
    let is_null = match ref_val {
        Ref::RefNull => 1,
        _ => 0,
    };
    ctx.reg_file.set_i32(ctx.regs[0], is_null);
    Ok(())
}

// table.get: [i32] -> [ref]
// idx = regs[1], dst = regs[0]
fn table_ref_reg_get(ctx: TableRefRegContext) -> Result<(), RuntimeError> {
    let table_addr = ctx
        .module_inst
        .table_addrs
        .get(ctx.table_idx as usize)
        .ok_or(RuntimeError::TableNotFound)?;
    let index = ctx.reg_file.get_i32(ctx.regs[1]) as usize;
    let val = table_addr.get(index);
    match val {
        Val::Ref(r) => {
            ctx.reg_file.set_ref(ctx.regs[0], r);
            Ok(())
        }
        _ => Err(RuntimeError::TypeMismatch),
    }
}

// table.set: [i32, ref] -> []
// idx = regs[0], val = regs[1]
fn table_ref_reg_set(ctx: TableRefRegContext) -> Result<(), RuntimeError> {
    let table_addr = ctx
        .module_inst
        .table_addrs
        .get(ctx.table_idx as usize)
        .ok_or(RuntimeError::TableNotFound)?;
    let index = ctx.reg_file.get_i32(ctx.regs[0]) as usize;
    let ref_val = ctx.reg_file.get_ref(ctx.regs[1]);
    table_addr.set(index, Val::Ref(ref_val))
}

// table.fill: [i32, ref, i32] -> []
// i = regs[0], val = regs[1], n = regs[2]
fn table_ref_reg_fill(ctx: TableRefRegContext) -> Result<(), RuntimeError> {
    let table_addr = ctx
        .module_inst
        .table_addrs
        .get(ctx.table_idx as usize)
        .ok_or(RuntimeError::TableNotFound)?;
    let i = ctx.reg_file.get_i32(ctx.regs[0]) as usize;
    let ref_val = ctx.reg_file.get_ref(ctx.regs[1]);
    let n = ctx.reg_file.get_i32(ctx.regs[2]) as usize;
    table_addr.fill(i, Val::Ref(ref_val), n)
}

lazy_static! {
    static ref TABLE_REF_REG_HANDLER_TABLE: Vec<TableRefRegHandler> = {
        let mut table: Vec<TableRefRegHandler> = vec![table_ref_reg_invalid_handler; 0x101];

        table[HANDLER_IDX_REF_NULL_REG] = table_ref_reg_null;
        table[HANDLER_IDX_REF_IS_NULL_REG] = table_ref_reg_is_null;
        table[HANDLER_IDX_TABLE_GET_REG] = table_ref_reg_get;
        table[HANDLER_IDX_TABLE_SET_REG] = table_ref_reg_set;
        table[HANDLER_IDX_TABLE_FILL_REG] = table_ref_reg_fill;

        table
    };
}
