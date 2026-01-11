use super::value::*;
use crate::error::RuntimeError;
use crate::execution::{
    func::*,
    mem::MemAddr,
    migration,
    module::*,
    slots::{Slot, SlotFile},
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
        source_slots: Vec<Slot>,
        target_result_slots: Vec<Slot>,
        condition_slot: Option<Slot>, // For br_if: slot containing condition value
    },
    MemArg(Memarg),
    BrTable {
        targets: Vec<Operand>,
        default: Box<Operand>,
        index_slot: Option<Slot>, // Slot containing table index
    },
    CallIndirect {
        type_idx: TypeIdx,
        table_idx: TableIdx,
        index_slot: Option<Slot>, // Slot containing table element index
    },
    Block {
        arity: usize,
        param_count: usize,
        is_loop: bool,
    },

    // Unified optimized operand
    Optimized(OptimizedOperand),
}

/// Slot-based operand for I32 operations
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum I32SlotOperand {
    Slot(u16),  // Read from slot
    Const(i32), // Constant value
    Param(u16), // Read from parameter/local
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum I64SlotOperand {
    Slot(u16),
    Const(i64),
    Param(u16),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum F32SlotOperand {
    Slot(u16),
    Const(f32),
    Param(u16),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum F64SlotOperand {
    Slot(u16),
    Const(f64),
    Param(u16),
}

/// I32 operations for slot-based execution
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
    /// Slot-based I32 instruction
    I32Slot {
        handler_index: usize,
        dst: u16,                     // Destination slot index
        src1: I32SlotOperand,         // First operand
        src2: Option<I32SlotOperand>, // Second operand (None for unary ops)
    },
    /// Slot-based I64 instruction
    I64Slot {
        handler_index: usize,
        dst: Slot, // Destination slot (I64 for arithmetic, I32 for comparisons)
        src1: I64SlotOperand,
        src2: Option<I64SlotOperand>,
    },

    F32Slot {
        handler_index: usize,
        dst: Slot,
        src1: F32SlotOperand,
        src2: Option<F32SlotOperand>,
    },

    F64Slot {
        handler_index: usize,
        dst: Slot,
        src1: F64SlotOperand,
        src2: Option<F64SlotOperand>,
    },
    ConversionSlot {
        handler_index: usize,
        dst: Slot, // Destination slot (type determined by output type)
        src: Slot, // Source slot (type determined by input type)
    },
    MemoryLoadSlot {
        handler_index: usize,
        dst: Slot,   // Destination slot for loaded value
        addr: Slot,  // Address slot (always I32)
        offset: u64, // Memory offset
    },
    MemoryStoreSlot {
        handler_index: usize,
        addr: Slot,  // Address slot (always I32)
        value: Slot, // Value slot to store
        offset: u64, // Memory offset
    },

    MemoryOpsSlot {
        handler_index: usize,
        dst: Option<Slot>, // Destination slot (for size/grow results)
        args: Vec<Slot>,   // Argument slots (varies by operation)
        data_index: u32,   // Data segment index (for memory.init)
    },

    SelectSlot {
        handler_index: usize,
        dst: Slot,  // Destination slot
        val1: Slot, // First value (selected when cond != 0)
        val2: Slot, // Second value (selected when cond == 0)
        cond: Slot, // Condition slot (always I32)
    },

    GlobalGetSlot {
        handler_index: usize,
        dst: Slot,         // Destination slot
        global_index: u32, // Global variable index
    },

    GlobalSetSlot {
        handler_index: usize,
        src: Slot,         // Source slot
        global_index: u32, // Global variable index
    },

    DataDropSlot {
        data_index: u32, // Data segment index to drop
    },

    /// Slot-based ref local operations
    RefLocalSlot {
        handler_index: usize,
        dst: u16,       // Destination slot index (for get) or unused (for set)
        src: u16,       // Source slot index (for set) or unused (for get)
        local_idx: u16, // Local variable index
    },

    TableRefSlot {
        handler_index: usize,
        table_idx: u32,
        slots: [u16; 3],   // Operand slots (usage depends on handler)
        ref_type: RefType, // Used for RefNull
    },
    CallWasiSlot {
        wasi_func_type: WasiFuncType,
        param_slots: Vec<Slot>,    // Parameter slots
        result_slot: Option<Slot>, // Result slot (most WASI functions return i32)
    },
    CallIndirectSlot {
        type_idx: TypeIdx,
        table_idx: TableIdx,
        index_slot: Slot,        // Table index slot
        param_slots: Vec<Slot>,  // Parameter slots
        result_slots: Vec<Slot>, // Result slots
    },
    CallSlot {
        func_idx: FuncIdx,
        param_slots: Vec<Slot>,  // Parameter slots
        result_slots: Vec<Slot>, // Result slots
    },
    ReturnSlot {
        result_slots: Vec<Slot>, // Result slots to return
    },
    /// Unconditional jump (for Else)
    JumpSlot { target_ip: usize },
    /// Block/Loop control structure
    BlockSlot {
        arity: usize,
        param_count: usize,
        is_loop: bool,
    },
    /// If control structure
    IfSlot {
        arity: usize,
        condition_slot: Slot,
        else_target_ip: usize, // Jump target if condition is false (else or end)
        has_else: bool,        // True if this if has an else branch
    },
    /// End of block/loop/if
    EndSlot {
        source_slots: Vec<Slot>,
        target_result_slots: Vec<Slot>,
    },
    /// Unconditional branch
    BrSlot {
        relative_depth: u32,
        target_ip: usize, // Target instruction pointer (set by fixup)
        source_slots: Vec<Slot>,
        target_result_slots: Vec<Slot>,
    },
    /// Conditional branch
    BrIfSlot {
        relative_depth: u32,
        target_ip: usize, // Target instruction pointer (set by fixup)
        condition_slot: Slot,
        source_slots: Vec<Slot>,
        target_result_slots: Vec<Slot>,
    },
    /// Branch table
    BrTableSlot {
        targets: Vec<(u32, usize, Vec<Slot>)>, // (relative_depth, target_ip, target_result_slots) for each target
        default_target: (u32, usize, Vec<Slot>), // (relative_depth, target_ip, target_result_slots) for default
        index_slot: Slot,                        // Index slot
        source_slots: Vec<Slot>,                 // Source slots (same for all targets)
    },
    /// No operation
    NopSlot,
    /// Unreachable instruction
    UnreachableSlot,
}

impl ProcessedInstr {
    /// Get handler_index
    #[inline(always)]
    pub fn handler_index(&self) -> usize {
        match self {
            ProcessedInstr::I32Slot { handler_index, .. } => *handler_index,
            ProcessedInstr::I64Slot { handler_index, .. } => *handler_index,
            ProcessedInstr::F32Slot { handler_index, .. } => *handler_index,
            ProcessedInstr::F64Slot { handler_index, .. } => *handler_index,
            ProcessedInstr::ConversionSlot { handler_index, .. } => *handler_index,
            ProcessedInstr::MemoryLoadSlot { handler_index, .. } => *handler_index,
            ProcessedInstr::MemoryStoreSlot { handler_index, .. } => *handler_index,
            ProcessedInstr::MemoryOpsSlot { handler_index, .. } => *handler_index,
            ProcessedInstr::SelectSlot { handler_index, .. } => *handler_index,
            ProcessedInstr::GlobalGetSlot { handler_index, .. } => *handler_index,
            ProcessedInstr::GlobalSetSlot { handler_index, .. } => *handler_index,
            ProcessedInstr::DataDropSlot { .. } => HANDLER_IDX_DATA_DROP,
            ProcessedInstr::RefLocalSlot { handler_index, .. } => *handler_index,
            ProcessedInstr::TableRefSlot { handler_index, .. } => *handler_index,
            ProcessedInstr::CallWasiSlot { .. } => HANDLER_IDX_CALL_WASI_SLOT,
            ProcessedInstr::CallIndirectSlot { .. } => HANDLER_IDX_CALL_INDIRECT,
            ProcessedInstr::CallSlot { .. } => HANDLER_IDX_CALL,
            ProcessedInstr::ReturnSlot { .. } => HANDLER_IDX_RETURN,
            ProcessedInstr::JumpSlot { .. } => HANDLER_IDX_ELSE,
            ProcessedInstr::BlockSlot { is_loop: false, .. } => HANDLER_IDX_BLOCK,
            ProcessedInstr::BlockSlot { is_loop: true, .. } => HANDLER_IDX_LOOP,
            ProcessedInstr::IfSlot { .. } => HANDLER_IDX_IF,
            ProcessedInstr::EndSlot { .. } => HANDLER_IDX_END,
            ProcessedInstr::BrSlot { .. } => HANDLER_IDX_BR,
            ProcessedInstr::BrIfSlot { .. } => HANDLER_IDX_BR_IF,
            ProcessedInstr::BrTableSlot { .. } => HANDLER_IDX_BR_TABLE,
            ProcessedInstr::NopSlot => HANDLER_IDX_NOP,
            ProcessedInstr::UnreachableSlot => HANDLER_IDX_UNREACHABLE,
        }
    }
}

#[derive(Clone)]
pub enum ModuleLevelInstr {
    Return,
    InvokeWasiSlot {
        wasi_func_type: WasiFuncType,
        params: Vec<Val>,          // Parameters read from slots
        result_slot: Option<Slot>, // Slot to write result to
    },
    InvokeSlot {
        func_addr: FuncAddr,
        params: Vec<Val>,        // Parameters read from slots
        result_slots: Vec<Slot>, // Slots to write results to
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
    /// Global slot file for all frames
    pub slot_file: SlotFile,
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

                // Initialize global SlotFile
                let mut slot_file = SlotFile::new_global();

                // Push initial frame allocation if present
                if let Some(alloc) = code.slot_allocation.as_ref() {
                    slot_file.save_offsets(alloc);
                }

                let initial_frame = FrameStack {
                    frame: Frame {
                        locals,
                        module: module.clone(),
                        n: type_.results.len(),
                        result_slot: code.result_slot,
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
                    result_slots: vec![],
                    return_result_slots: vec![],
                };

                Ok(VMState {
                    slot_file,
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

    /// Get mutable references to both slot_file and activation_frame_stack
    /// This allows split borrowing of VMState fields
    pub fn get_slot_file_and_frames(&mut self) -> (&mut SlotFile, &mut Vec<FrameStack>) {
        (&mut self.slot_file, &mut self.activation_frame_stack)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub locals: Vec<Val>,
    #[serde(skip)]
    pub module: Weak<ModuleInst>,
    pub n: usize,
    pub result_slot: Option<crate::execution::slots::Slot>, // Slot for return value (slot mode only)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameStack {
    pub frame: Frame,
    pub label_stack: Vec<LabelStack>,
    pub void: bool,
    pub instruction_count: u64,
    #[serde(skip)]
    pub enable_checkpoint: bool,
    /// Slots where caller expects results to be written
    #[serde(skip)]
    pub result_slots: Vec<Slot>,
    /// Slots containing this function's return values (set on return)
    #[serde(skip)]
    pub return_result_slots: Vec<Slot>,
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
        slot_file: &mut SlotFile,
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
                ProcessedInstr::I32Slot {
                    handler_index,
                    dst,
                    src1,
                    src2,
                } => {
                    // Create context for handler
                    let ctx = I32SlotContext {
                        slot_file: slot_file.get_i32_slots(),
                        locals: &mut self.frame.locals,
                        src1: src1.clone(),
                        src2: src2.clone(),
                    };

                    let handler = I32_SLOT_HANDLER_TABLE[*handler_index];
                    handler(ctx, *dst)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::I64Slot {
                    handler_index,
                    dst,
                    src1,
                    src2,
                } => {
                    // Get both i32 and i64 slots for comparison operations
                    let (i32_slots, i64_slots) = slot_file.get_i32_and_i64_slots();
                    let ctx = I64SlotContext {
                        i64_slots,
                        i32_slots,
                        locals: &mut self.frame.locals,
                        src1: src1.clone(),
                        src2: src2.clone(),
                        dst: dst.clone(),
                    };

                    let handler = I64_SLOT_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::F32Slot {
                    handler_index,
                    dst,
                    src1,
                    src2,
                } => {
                    // Get both i32 and f32 slots for comparison operations
                    let (i32_slots, f32_slots) = slot_file.get_i32_and_f32_slots();
                    let ctx = F32SlotContext {
                        f32_slots,
                        i32_slots,
                        locals: &mut self.frame.locals,
                        src1: src1.clone(),
                        src2: src2.clone(),
                        dst: dst.clone(),
                    };

                    let handler = F32_SLOT_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::F64Slot {
                    handler_index,
                    dst,
                    src1,
                    src2,
                } => {
                    // Get both i32 and f64 slots for comparison operations
                    let (i32_slots, f64_slots) = slot_file.get_i32_and_f64_slots();
                    let ctx = F64SlotContext {
                        f64_slots,
                        i32_slots,
                        locals: &mut self.frame.locals,
                        src1: src1.clone(),
                        src2: src2.clone(),
                        dst: dst.clone(),
                    };

                    let handler = F64_SLOT_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::ConversionSlot {
                    handler_index,
                    dst,
                    src,
                } => {
                    let ctx = ConversionSlotContext {
                        slot_file,
                        src: src.clone(),
                        dst: dst.clone(),
                    };

                    let handler = CONVERSION_SLOT_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::MemoryLoadSlot {
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

                    let ctx = MemoryLoadSlotContext {
                        slot_file,
                        mem_addr,
                        addr: addr.clone(),
                        dst: dst.clone(),
                        offset: *offset,
                    };

                    let handler = MEMORY_LOAD_SLOT_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::MemoryStoreSlot {
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

                    let ctx = MemoryStoreSlotContext {
                        slot_file,
                        mem_addr,
                        addr: addr.clone(),
                        value: value.clone(),
                        offset: *offset,
                    };

                    let handler = MEMORY_STORE_SLOT_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::MemoryOpsSlot {
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

                    let ctx = MemoryOpsSlotContext {
                        slot_file,
                        mem_addr,
                        module_inst: &module_inst,
                        dst: dst.clone(),
                        args: args.clone(),
                        data_index: *data_index,
                    };

                    let handler = MEMORY_OPS_SLOT_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::SelectSlot {
                    handler_index,
                    dst,
                    val1,
                    val2,
                    cond,
                } => {
                    let ctx = SelectSlotContext {
                        slot_file,
                        dst: dst.clone(),
                        val1: val1.clone(),
                        val2: val2.clone(),
                        cond: cond.clone(),
                    };

                    let handler = SELECT_SLOT_HANDLER_TABLE[*handler_index];
                    handler(ctx)?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::GlobalGetSlot {
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
                            slot_file.set_i32(dst.index(), val.to_i32().unwrap());
                        }
                        HANDLER_IDX_GLOBAL_GET_I64 => {
                            slot_file.set_i64(dst.index(), val.to_i64().unwrap());
                        }
                        HANDLER_IDX_GLOBAL_GET_F32 => {
                            slot_file.set_f32(dst.index(), val.to_f32().unwrap());
                        }
                        HANDLER_IDX_GLOBAL_GET_F64 => {
                            slot_file.set_f64(dst.index(), val.to_f64().unwrap());
                        }
                        _ => return Err(RuntimeError::InvalidHandlerIndex),
                    }

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::GlobalSetSlot {
                    handler_index,
                    src,
                    global_index,
                } => {
                    let val = match *handler_index {
                        HANDLER_IDX_GLOBAL_SET_I32 => {
                            Val::Num(Num::I32(slot_file.get_i32(src.index())))
                        }
                        HANDLER_IDX_GLOBAL_SET_I64 => {
                            Val::Num(Num::I64(slot_file.get_i64(src.index())))
                        }
                        HANDLER_IDX_GLOBAL_SET_F32 => {
                            Val::Num(Num::F32(slot_file.get_f32(src.index())))
                        }
                        HANDLER_IDX_GLOBAL_SET_F64 => {
                            Val::Num(Num::F64(slot_file.get_f64(src.index())))
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
                ProcessedInstr::DataDropSlot { data_index } => {
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
                ProcessedInstr::RefLocalSlot {
                    handler_index,
                    dst,
                    src,
                    local_idx,
                } => {
                    let handler_fn = REF_LOCAL_SLOT_HANDLER_TABLE
                        .get(*handler_index)
                        .ok_or(RuntimeError::InvalidHandlerIndex)?;

                    handler_fn(RefLocalSlotContext {
                        slot_file,
                        locals: &mut self.frame.locals,
                        dst: *dst,
                        src: *src,
                        local_idx: *local_idx,
                    })?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::TableRefSlot {
                    handler_index,
                    table_idx,
                    slots,
                    ref_type,
                } => {
                    let handler_fn = TABLE_REF_SLOT_HANDLER_TABLE
                        .get(*handler_index)
                        .ok_or(RuntimeError::InvalidHandlerIndex)?;

                    let module_inst = self
                        .frame
                        .module
                        .upgrade()
                        .ok_or(RuntimeError::ModuleInstanceGone)?;

                    handler_fn(TableRefSlotContext {
                        slot_file,
                        module_inst: &module_inst,
                        table_idx: *table_idx,
                        slots: *slots,
                        ref_type: *ref_type,
                    })?;

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::CallWasiSlot {
                    wasi_func_type,
                    param_slots,
                    result_slot,
                } => {
                    let wasi_func_type_copy = *wasi_func_type;
                    let param_slots_copy = param_slots.clone();
                    let result_slot_copy = *result_slot;

                    // Read parameters from slots
                    let params: Vec<Val> = param_slots_copy
                        .iter()
                        .map(|slot| slot_file.get_val(slot))
                        .collect();

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    return Ok(Ok(Some(ModuleLevelInstr::InvokeWasiSlot {
                        wasi_func_type: wasi_func_type_copy,
                        params,
                        result_slot: result_slot_copy,
                    })));
                }
                ProcessedInstr::CallIndirectSlot {
                    type_idx,
                    table_idx,
                    index_slot,
                    param_slots,
                    result_slots,
                } => {
                    let type_idx = *type_idx;
                    let table_idx = *table_idx;
                    let index_slot = *index_slot;
                    let param_slots = param_slots.clone();
                    let result_slots = result_slots.clone();

                    // Read table index from slot
                    let i = slot_file.get_i32(index_slot.index());

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

                        let params: Vec<Val> = param_slots
                            .iter()
                            .map(|slot| slot_file.get_val(slot))
                            .collect();

                        self.label_stack[current_label_stack_idx].ip = ip + 1;
                        return Ok(Ok(Some(ModuleLevelInstr::InvokeSlot {
                            func_addr,
                            params,
                            result_slots,
                        })));
                    } else {
                        return Err(RuntimeError::UninitializedElement);
                    }
                }
                ProcessedInstr::CallSlot {
                    func_idx,
                    param_slots,
                    result_slots,
                } => {
                    let func_idx = *func_idx;
                    let param_slots = param_slots.clone();
                    let result_slots = result_slots.clone();

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

                    // Read parameters from slots
                    let params: Vec<Val> = param_slots
                        .iter()
                        .map(|slot| slot_file.get_val(slot))
                        .collect();

                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    return Ok(Ok(Some(ModuleLevelInstr::InvokeSlot {
                        func_addr,
                        params,
                        result_slots,
                    })));
                }
                ProcessedInstr::ReturnSlot { result_slots } => {
                    // Store result slots for caller to read
                    self.return_result_slots = result_slots.clone();

                    return Ok(Ok(Some(ModuleLevelInstr::Return)));
                }
                ProcessedInstr::JumpSlot { target_ip } => {
                    let target_ip = *target_ip;
                    if self.label_stack.len() > 1 {
                        self.label_stack.pop();
                        current_label_stack_idx = self.label_stack.len() - 1;
                    }
                    self.label_stack[current_label_stack_idx].ip = target_ip;
                    continue;
                }
                ProcessedInstr::BlockSlot {
                    arity,
                    param_count,
                    is_loop,
                } => {
                    // Push a new label for Block/Loop
                    let label = Label {
                        locals_num: *param_count,
                        arity: *arity,
                        is_loop: *is_loop,
                        stack_height: 0, // Not used in slot mode
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
                ProcessedInstr::IfSlot {
                    arity,
                    condition_slot,
                    else_target_ip,
                    has_else,
                } => {
                    let arity = *arity;
                    let condition_slot = *condition_slot;
                    let else_target_ip = *else_target_ip;
                    let has_else = *has_else;

                    // Read condition from slot
                    let cond = slot_file.get_i32(condition_slot.index());

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
                            stack_height: 0, // Not used in slot mode
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
                ProcessedInstr::EndSlot {
                    source_slots,
                    target_result_slots,
                } => {
                    let source_slots = source_slots.clone();
                    let target_result_slots = target_result_slots.clone();

                    // Pop label stack
                    if self.label_stack.len() > 1 {
                        // Block/Loop/If level end: copy results to target slots
                        slot_file.copy_slots(&source_slots, &target_result_slots);
                        self.label_stack.pop();
                        current_label_stack_idx = self.label_stack.len() - 1;
                        // Set parent's ip to continue after the End instruction
                        self.label_stack[current_label_stack_idx].ip = ip + 1;

                        // If next ip is beyond code length and we're at function level, store result slots and break
                        let parent_processed_code =
                            &self.label_stack[current_label_stack_idx].processed_instrs;
                        if ip + 1 >= parent_processed_code.len() && current_label_stack_idx == 0 {
                            self.return_result_slots = source_slots.clone();
                            break;
                        }
                    } else {
                        // Function level end: store result slots for return
                        self.return_result_slots = source_slots.clone();
                        break;
                    }
                    continue;
                }
                ProcessedInstr::BrSlot {
                    relative_depth,
                    target_ip,
                    source_slots,
                    target_result_slots,
                } => {
                    let relative_depth = *relative_depth as usize;
                    let target_ip = *target_ip;
                    let source_slots = source_slots.clone();
                    let target_result_slots = target_result_slots.clone();

                    if !source_slots.is_empty() && !target_result_slots.is_empty() {
                        slot_file.copy_slots(&source_slots, &target_result_slots);
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
                ProcessedInstr::BrIfSlot {
                    relative_depth,
                    target_ip,
                    condition_slot,
                    source_slots,
                    target_result_slots,
                } => {
                    let relative_depth = *relative_depth as usize;
                    let target_ip = *target_ip;
                    let condition_slot = *condition_slot;
                    let source_slots = source_slots.clone();
                    let target_result_slots = target_result_slots.clone();

                    // Read condition from slot
                    let cond = slot_file.get_i32(condition_slot.index());

                    if cond != 0 {
                        // Take the branch
                        if !source_slots.is_empty() && !target_result_slots.is_empty() {
                            slot_file.copy_slots(&source_slots, &target_result_slots);
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
                ProcessedInstr::BrTableSlot {
                    targets,
                    default_target,
                    index_slot,
                    source_slots,
                } => {
                    let targets = targets.clone();
                    let default_target = default_target.clone();
                    let index_slot = *index_slot;
                    let source_slots = source_slots.clone();

                    // Read index from slot
                    let idx = slot_file.get_i32(index_slot.index()) as usize;

                    // Select target (relative_depth, target_ip, target_result_slots) tuple
                    let (relative_depth, target_ip, target_result_slots) = if idx < targets.len() {
                        targets[idx].clone()
                    } else {
                        default_target
                    };
                    let relative_depth = relative_depth as usize;

                    // Copy source to target result slots
                    if !source_slots.is_empty() && !target_result_slots.is_empty() {
                        slot_file.copy_slots(&source_slots, &target_result_slots);
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
                ProcessedInstr::NopSlot => {
                    // No operation - just advance IP
                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                    continue;
                }
                ProcessedInstr::UnreachableSlot => {
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

struct I32SlotContext<'a> {
    slot_file: &'a mut [i32],
    locals: &'a mut [Val],
    src1: I32SlotOperand,
    src2: Option<I32SlotOperand>,
}

impl<'a> I32SlotContext<'a> {
    #[inline]
    fn get_operand(&self, operand: &I32SlotOperand) -> Result<i32, RuntimeError> {
        match operand {
            I32SlotOperand::Slot(idx) => Ok(self.slot_file[*idx as usize]),
            I32SlotOperand::Const(val) => Ok(*val),
            I32SlotOperand::Param(idx) => self.locals[*idx as usize].to_i32(),
        }
    }
}

fn i32_slot_local_get(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.slot_file[dst as usize] = val;
    Ok(())
}

fn i32_slot_local_set(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    // dst is the local variable index, src1 is the value source
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.locals[dst as usize] = Val::Num(Num::I32(val));
    Ok(())
}

fn i32_slot_const(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.slot_file[dst as usize] = val;
    Ok(())
}

fn i32_slot_add(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = lhs.wrapping_add(rhs);
    Ok(())
}

fn i32_slot_sub(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = lhs.wrapping_sub(rhs);
    Ok(())
}

fn i32_slot_mul(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = lhs.wrapping_mul(rhs);
    Ok(())
}

fn i32_slot_div_s(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    ctx.slot_file[dst as usize] = lhs.wrapping_div(rhs);
    Ok(())
}

fn i32_slot_div_u(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    let rhs_u = rhs as u32;
    if rhs_u == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    ctx.slot_file[dst as usize] = ((lhs as u32) / rhs_u) as i32;
    Ok(())
}

fn i32_slot_rem_s(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    ctx.slot_file[dst as usize] = lhs.wrapping_rem(rhs);
    Ok(())
}

fn i32_slot_rem_u(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    let rhs_u = rhs as u32;
    if rhs_u == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    ctx.slot_file[dst as usize] = ((lhs as u32) % rhs_u) as i32;
    Ok(())
}

fn i32_slot_and(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = lhs & rhs;
    Ok(())
}

fn i32_slot_or(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = lhs | rhs;
    Ok(())
}

fn i32_slot_xor(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = lhs ^ rhs;
    Ok(())
}

fn i32_slot_shl(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = lhs.wrapping_shl(rhs as u32);
    Ok(())
}

fn i32_slot_shr_s(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = lhs.wrapping_shr(rhs as u32);
    Ok(())
}

fn i32_slot_shr_u(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = ((lhs as u32).wrapping_shr(rhs as u32)) as i32;
    Ok(())
}

fn i32_slot_rotl(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = lhs.rotate_left(rhs as u32);
    Ok(())
}

fn i32_slot_rotr(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = lhs.rotate_right(rhs as u32);
    Ok(())
}

// Comparison handlers
fn i32_slot_eq(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = if lhs == rhs { 1 } else { 0 };
    Ok(())
}

fn i32_slot_ne(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = if lhs != rhs { 1 } else { 0 };
    Ok(())
}

fn i32_slot_lt_s(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = if lhs < rhs { 1 } else { 0 };
    Ok(())
}

fn i32_slot_lt_u(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = if (lhs as u32) < (rhs as u32) { 1 } else { 0 };
    Ok(())
}

fn i32_slot_le_s(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = if lhs <= rhs { 1 } else { 0 };
    Ok(())
}

fn i32_slot_le_u(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = if (lhs as u32) <= (rhs as u32) { 1 } else { 0 };
    Ok(())
}

fn i32_slot_gt_s(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = if lhs > rhs { 1 } else { 0 };
    Ok(())
}

fn i32_slot_gt_u(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = if (lhs as u32) > (rhs as u32) { 1 } else { 0 };
    Ok(())
}

fn i32_slot_ge_s(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = if lhs >= rhs { 1 } else { 0 };
    Ok(())
}

fn i32_slot_ge_u(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.slot_file[dst as usize] = if (lhs as u32) >= (rhs as u32) { 1 } else { 0 };
    Ok(())
}

// Unary operation handlers
fn i32_slot_clz(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.slot_file[dst as usize] = val.leading_zeros() as i32;
    Ok(())
}

fn i32_slot_ctz(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.slot_file[dst as usize] = val.trailing_zeros() as i32;
    Ok(())
}

fn i32_slot_popcnt(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.slot_file[dst as usize] = val.count_ones() as i32;
    Ok(())
}

fn i32_slot_eqz(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.slot_file[dst as usize] = if val == 0 { 1 } else { 0 };
    Ok(())
}

fn i32_slot_extend8_s(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.slot_file[dst as usize] = (val as i8) as i32;
    Ok(())
}

fn i32_slot_extend16_s(ctx: I32SlotContext, dst: u16) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.slot_file[dst as usize] = (val as i16) as i32;
    Ok(())
}

// Handler function type
type I32SlotHandler = fn(I32SlotContext, u16) -> Result<(), RuntimeError>;

// Default error handler
fn i32_slot_invalid_handler(_ctx: I32SlotContext, _dst: u16) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

// I32 Slot Handler Table
lazy_static! {
    static ref I32_SLOT_HANDLER_TABLE: Vec<I32SlotHandler> = {
        let mut table: Vec<I32SlotHandler> = vec![i32_slot_invalid_handler; 256];

        // Special handlers
        table[HANDLER_IDX_LOCAL_GET] = i32_slot_local_get;
        table[HANDLER_IDX_LOCAL_SET] = i32_slot_local_set;
        table[HANDLER_IDX_I32_CONST] = i32_slot_const;

        // Binary operations
        table[HANDLER_IDX_I32_ADD] = i32_slot_add;
        table[HANDLER_IDX_I32_SUB] = i32_slot_sub;
        table[HANDLER_IDX_I32_MUL] = i32_slot_mul;
        table[HANDLER_IDX_I32_DIV_S] = i32_slot_div_s;
        table[HANDLER_IDX_I32_DIV_U] = i32_slot_div_u;
        table[HANDLER_IDX_I32_REM_S] = i32_slot_rem_s;
        table[HANDLER_IDX_I32_REM_U] = i32_slot_rem_u;
        table[HANDLER_IDX_I32_AND] = i32_slot_and;
        table[HANDLER_IDX_I32_OR] = i32_slot_or;
        table[HANDLER_IDX_I32_XOR] = i32_slot_xor;
        table[HANDLER_IDX_I32_SHL] = i32_slot_shl;
        table[HANDLER_IDX_I32_SHR_S] = i32_slot_shr_s;
        table[HANDLER_IDX_I32_SHR_U] = i32_slot_shr_u;
        table[HANDLER_IDX_I32_ROTL] = i32_slot_rotl;
        table[HANDLER_IDX_I32_ROTR] = i32_slot_rotr;

        // Comparisons
        table[HANDLER_IDX_I32_EQ] = i32_slot_eq;
        table[HANDLER_IDX_I32_NE] = i32_slot_ne;
        table[HANDLER_IDX_I32_LT_S] = i32_slot_lt_s;
        table[HANDLER_IDX_I32_LT_U] = i32_slot_lt_u;
        table[HANDLER_IDX_I32_LE_S] = i32_slot_le_s;
        table[HANDLER_IDX_I32_LE_U] = i32_slot_le_u;
        table[HANDLER_IDX_I32_GT_S] = i32_slot_gt_s;
        table[HANDLER_IDX_I32_GT_U] = i32_slot_gt_u;
        table[HANDLER_IDX_I32_GE_S] = i32_slot_ge_s;
        table[HANDLER_IDX_I32_GE_U] = i32_slot_ge_u;

        // Unary operations
        table[HANDLER_IDX_I32_CLZ] = i32_slot_clz;
        table[HANDLER_IDX_I32_CTZ] = i32_slot_ctz;
        table[HANDLER_IDX_I32_POPCNT] = i32_slot_popcnt;
        table[HANDLER_IDX_I32_EQZ] = i32_slot_eqz;
        table[HANDLER_IDX_I32_EXTEND8_S] = i32_slot_extend8_s;
        table[HANDLER_IDX_I32_EXTEND16_S] = i32_slot_extend16_s;

        table
    };
}

struct I64SlotContext<'a> {
    i64_slots: &'a mut [i64],
    i32_slots: &'a mut [i32], // For comparison operations that return i32
    locals: &'a mut [Val],
    src1: I64SlotOperand,
    src2: Option<I64SlotOperand>,
    dst: Slot, // Destination slot (type determines which array to write to)
}

impl<'a> I64SlotContext<'a> {
    #[inline]
    fn get_operand(&self, operand: &I64SlotOperand) -> Result<i64, RuntimeError> {
        match operand {
            I64SlotOperand::Slot(idx) => Ok(self.i64_slots[*idx as usize]),
            I64SlotOperand::Const(val) => Ok(*val),
            I64SlotOperand::Param(idx) => self.locals[*idx as usize].to_i64(),
        }
    }
}

fn i64_slot_local_get(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_slots[ctx.dst.index() as usize] = val;
    Ok(())
}

fn i64_slot_local_set(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    // dst.index() is the local variable index, src1 is the value source
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.locals[ctx.dst.index() as usize] = Val::Num(Num::I64(val));
    Ok(())
}

fn i64_slot_const(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_slots[ctx.dst.index() as usize] = val;
    Ok(())
}

fn i64_slot_add(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_slots[ctx.dst.index() as usize] = lhs.wrapping_add(rhs);
    Ok(())
}

fn i64_slot_sub(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_slots[ctx.dst.index() as usize] = lhs.wrapping_sub(rhs);
    Ok(())
}

fn i64_slot_mul(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_slots[ctx.dst.index() as usize] = lhs.wrapping_mul(rhs);
    Ok(())
}

fn i64_slot_div_s(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    if lhs == i64::MIN && rhs == -1 {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.i64_slots[ctx.dst.index() as usize] = lhs.wrapping_div(rhs);
    Ok(())
}

fn i64_slot_div_u(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    let rhs_u = rhs as u64;
    if rhs_u == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    ctx.i64_slots[ctx.dst.index() as usize] = ((lhs as u64) / rhs_u) as i64;
    Ok(())
}

fn i64_slot_rem_s(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    ctx.i64_slots[ctx.dst.index() as usize] = lhs.wrapping_rem(rhs);
    Ok(())
}

fn i64_slot_rem_u(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    let rhs_u = rhs as u64;
    if rhs_u == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    ctx.i64_slots[ctx.dst.index() as usize] = ((lhs as u64) % rhs_u) as i64;
    Ok(())
}

fn i64_slot_and(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_slots[ctx.dst.index() as usize] = lhs & rhs;
    Ok(())
}

fn i64_slot_or(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_slots[ctx.dst.index() as usize] = lhs | rhs;
    Ok(())
}

fn i64_slot_xor(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_slots[ctx.dst.index() as usize] = lhs ^ rhs;
    Ok(())
}

fn i64_slot_shl(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_slots[ctx.dst.index() as usize] = lhs.wrapping_shl((rhs & 0x3f) as u32);
    Ok(())
}

fn i64_slot_shr_s(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_slots[ctx.dst.index() as usize] = lhs.wrapping_shr((rhs & 0x3f) as u32);
    Ok(())
}

fn i64_slot_shr_u(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_slots[ctx.dst.index() as usize] =
        ((lhs as u64).wrapping_shr((rhs & 0x3f) as u32)) as i64;
    Ok(())
}

fn i64_slot_rotl(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_slots[ctx.dst.index() as usize] = (lhs as u64).rotate_left((rhs & 0x3f) as u32) as i64;
    Ok(())
}

fn i64_slot_rotr(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i64_slots[ctx.dst.index() as usize] = (lhs as u64).rotate_right((rhs & 0x3f) as u32) as i64;
    Ok(())
}

fn i64_slot_clz(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_slots[ctx.dst.index() as usize] = val.leading_zeros() as i64;
    Ok(())
}

fn i64_slot_ctz(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_slots[ctx.dst.index() as usize] = val.trailing_zeros() as i64;
    Ok(())
}

fn i64_slot_popcnt(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_slots[ctx.dst.index() as usize] = val.count_ones() as i64;
    Ok(())
}

fn i64_slot_extend8_s(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_slots[ctx.dst.index() as usize] = (val as i8) as i64;
    Ok(())
}

fn i64_slot_extend16_s(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_slots[ctx.dst.index() as usize] = (val as i16) as i64;
    Ok(())
}

fn i64_slot_extend32_s(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i64_slots[ctx.dst.index() as usize] = (val as i32) as i64;
    Ok(())
}

// Comparison operations - write to i32_slots (ctx.dst is Slot::I32)
fn i64_slot_eq(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs == rhs { 1 } else { 0 };
    Ok(())
}

fn i64_slot_ne(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs != rhs { 1 } else { 0 };
    Ok(())
}

fn i64_slot_lt_s(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs < rhs { 1 } else { 0 };
    Ok(())
}

fn i64_slot_lt_u(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if (lhs as u64) < (rhs as u64) { 1 } else { 0 };
    Ok(())
}

fn i64_slot_le_s(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs <= rhs { 1 } else { 0 };
    Ok(())
}

fn i64_slot_le_u(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if (lhs as u64) <= (rhs as u64) { 1 } else { 0 };
    Ok(())
}

fn i64_slot_gt_s(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs > rhs { 1 } else { 0 };
    Ok(())
}

fn i64_slot_gt_u(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if (lhs as u64) > (rhs as u64) { 1 } else { 0 };
    Ok(())
}

fn i64_slot_ge_s(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs >= rhs { 1 } else { 0 };
    Ok(())
}

fn i64_slot_ge_u(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if (lhs as u64) >= (rhs as u64) { 1 } else { 0 };
    Ok(())
}

fn i64_slot_eqz(ctx: I64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.i32_slots[ctx.dst.index() as usize] = if val == 0 { 1 } else { 0 };
    Ok(())
}

type I64SlotHandler = fn(I64SlotContext) -> Result<(), RuntimeError>;

fn i64_slot_invalid_handler(_ctx: I64SlotContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

lazy_static! {
    static ref I64_SLOT_HANDLER_TABLE: Vec<I64SlotHandler> = {
        let mut table: Vec<I64SlotHandler> = vec![i64_slot_invalid_handler; 256];

        // Special handlers
        table[HANDLER_IDX_LOCAL_GET] = i64_slot_local_get;
        table[HANDLER_IDX_LOCAL_SET] = i64_slot_local_set;
        table[HANDLER_IDX_I64_CONST] = i64_slot_const;

        // Binary operations
        table[HANDLER_IDX_I64_ADD] = i64_slot_add;
        table[HANDLER_IDX_I64_SUB] = i64_slot_sub;
        table[HANDLER_IDX_I64_MUL] = i64_slot_mul;
        table[HANDLER_IDX_I64_DIV_S] = i64_slot_div_s;
        table[HANDLER_IDX_I64_DIV_U] = i64_slot_div_u;
        table[HANDLER_IDX_I64_REM_S] = i64_slot_rem_s;
        table[HANDLER_IDX_I64_REM_U] = i64_slot_rem_u;
        table[HANDLER_IDX_I64_AND] = i64_slot_and;
        table[HANDLER_IDX_I64_OR] = i64_slot_or;
        table[HANDLER_IDX_I64_XOR] = i64_slot_xor;
        table[HANDLER_IDX_I64_SHL] = i64_slot_shl;
        table[HANDLER_IDX_I64_SHR_S] = i64_slot_shr_s;
        table[HANDLER_IDX_I64_SHR_U] = i64_slot_shr_u;
        table[HANDLER_IDX_I64_ROTL] = i64_slot_rotl;
        table[HANDLER_IDX_I64_ROTR] = i64_slot_rotr;

        // Unary operations
        table[HANDLER_IDX_I64_CLZ] = i64_slot_clz;
        table[HANDLER_IDX_I64_CTZ] = i64_slot_ctz;
        table[HANDLER_IDX_I64_POPCNT] = i64_slot_popcnt;
        table[HANDLER_IDX_I64_EXTEND8_S] = i64_slot_extend8_s;
        table[HANDLER_IDX_I64_EXTEND16_S] = i64_slot_extend16_s;
        table[HANDLER_IDX_I64_EXTEND32_S] = i64_slot_extend32_s;

        // Comparison operations (return i32)
        table[HANDLER_IDX_I64_EQZ] = i64_slot_eqz;
        table[HANDLER_IDX_I64_EQ] = i64_slot_eq;
        table[HANDLER_IDX_I64_NE] = i64_slot_ne;
        table[HANDLER_IDX_I64_LT_S] = i64_slot_lt_s;
        table[HANDLER_IDX_I64_LT_U] = i64_slot_lt_u;
        table[HANDLER_IDX_I64_GT_S] = i64_slot_gt_s;
        table[HANDLER_IDX_I64_GT_U] = i64_slot_gt_u;
        table[HANDLER_IDX_I64_LE_S] = i64_slot_le_s;
        table[HANDLER_IDX_I64_LE_U] = i64_slot_le_u;
        table[HANDLER_IDX_I64_GE_S] = i64_slot_ge_s;
        table[HANDLER_IDX_I64_GE_U] = i64_slot_ge_u;

        table
    };
}

// F32 slot-based execution context and handlers
type F32SlotHandler = fn(F32SlotContext) -> Result<(), RuntimeError>;

fn f32_slot_invalid_handler(_ctx: F32SlotContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::Trap)
}

struct F32SlotContext<'a> {
    f32_slots: &'a mut [f32],
    i32_slots: &'a mut [i32], // For comparison operations that return i32
    locals: &'a mut [Val],
    src1: F32SlotOperand,
    src2: Option<F32SlotOperand>,
    dst: Slot, // Destination slot (type determines which array to write to)
}

impl<'a> F32SlotContext<'a> {
    #[inline]
    fn get_operand(&self, operand: &F32SlotOperand) -> Result<f32, RuntimeError> {
        match operand {
            F32SlotOperand::Slot(idx) => Ok(self.f32_slots[*idx as usize]),
            F32SlotOperand::Const(val) => Ok(*val),
            F32SlotOperand::Param(idx) => self.locals[*idx as usize].to_f32(),
        }
    }
}

fn f32_slot_local_get(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_slots[ctx.dst.index() as usize] = val;
    Ok(())
}

fn f32_slot_local_set(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    // dst.index() is the local variable index, src1 is the value source
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.locals[ctx.dst.index() as usize] = Val::Num(Num::F32(val));
    Ok(())
}

fn f32_slot_const(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_slots[ctx.dst.index() as usize] = val;
    Ok(())
}

fn f32_slot_add(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f32_slots[ctx.dst.index() as usize] = lhs + rhs;
    Ok(())
}

fn f32_slot_sub(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f32_slots[ctx.dst.index() as usize] = lhs - rhs;
    Ok(())
}

fn f32_slot_mul(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f32_slots[ctx.dst.index() as usize] = lhs * rhs;
    Ok(())
}

fn f32_slot_div(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f32_slots[ctx.dst.index() as usize] = lhs / rhs;
    Ok(())
}

fn f32_slot_min(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f32_slots[ctx.dst.index() as usize] = if lhs.is_nan() || rhs.is_nan() {
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

fn f32_slot_max(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f32_slots[ctx.dst.index() as usize] = if lhs.is_nan() || rhs.is_nan() {
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

fn f32_slot_copysign(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f32_slots[ctx.dst.index() as usize] = lhs.copysign(rhs);
    Ok(())
}

// Unary operations
fn f32_slot_abs(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_slots[ctx.dst.index() as usize] = val.abs();
    Ok(())
}

fn f32_slot_neg(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_slots[ctx.dst.index() as usize] = -val;
    Ok(())
}

fn f32_slot_ceil(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_slots[ctx.dst.index() as usize] = val.ceil();
    Ok(())
}

fn f32_slot_floor(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_slots[ctx.dst.index() as usize] = val.floor();
    Ok(())
}

fn f32_slot_trunc(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_slots[ctx.dst.index() as usize] = val.trunc();
    Ok(())
}

fn f32_slot_nearest(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_slots[ctx.dst.index() as usize] = val.round_ties_even();
    Ok(())
}

fn f32_slot_sqrt(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f32_slots[ctx.dst.index() as usize] = val.sqrt();
    Ok(())
}

// Comparison operations (return i32)
fn f32_slot_eq(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs == rhs { 1 } else { 0 };
    Ok(())
}

fn f32_slot_ne(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs != rhs { 1 } else { 0 };
    Ok(())
}

fn f32_slot_lt(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs < rhs { 1 } else { 0 };
    Ok(())
}

fn f32_slot_gt(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs > rhs { 1 } else { 0 };
    Ok(())
}

fn f32_slot_le(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs <= rhs { 1 } else { 0 };
    Ok(())
}

fn f32_slot_ge(ctx: F32SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs >= rhs { 1 } else { 0 };
    Ok(())
}

lazy_static! {
    static ref F32_SLOT_HANDLER_TABLE: Vec<F32SlotHandler> = {
        let mut table: Vec<F32SlotHandler> = vec![f32_slot_invalid_handler; 256];

        // Special handlers
        table[HANDLER_IDX_LOCAL_GET] = f32_slot_local_get;
        table[HANDLER_IDX_LOCAL_SET] = f32_slot_local_set;
        table[HANDLER_IDX_F32_CONST] = f32_slot_const;

        // Binary operations
        table[HANDLER_IDX_F32_ADD] = f32_slot_add;
        table[HANDLER_IDX_F32_SUB] = f32_slot_sub;
        table[HANDLER_IDX_F32_MUL] = f32_slot_mul;
        table[HANDLER_IDX_F32_DIV] = f32_slot_div;
        table[HANDLER_IDX_F32_MIN] = f32_slot_min;
        table[HANDLER_IDX_F32_MAX] = f32_slot_max;
        table[HANDLER_IDX_F32_COPYSIGN] = f32_slot_copysign;

        // Unary operations
        table[HANDLER_IDX_F32_ABS] = f32_slot_abs;
        table[HANDLER_IDX_F32_NEG] = f32_slot_neg;
        table[HANDLER_IDX_F32_CEIL] = f32_slot_ceil;
        table[HANDLER_IDX_F32_FLOOR] = f32_slot_floor;
        table[HANDLER_IDX_F32_TRUNC] = f32_slot_trunc;
        table[HANDLER_IDX_F32_NEAREST] = f32_slot_nearest;
        table[HANDLER_IDX_F32_SQRT] = f32_slot_sqrt;

        // Comparison operations (return i32)
        table[HANDLER_IDX_F32_EQ] = f32_slot_eq;
        table[HANDLER_IDX_F32_NE] = f32_slot_ne;
        table[HANDLER_IDX_F32_LT] = f32_slot_lt;
        table[HANDLER_IDX_F32_GT] = f32_slot_gt;
        table[HANDLER_IDX_F32_LE] = f32_slot_le;
        table[HANDLER_IDX_F32_GE] = f32_slot_ge;

        table
    };
}

// F64 slot-based execution context and handlers
type F64SlotHandler = fn(F64SlotContext) -> Result<(), RuntimeError>;

fn f64_slot_invalid_handler(_ctx: F64SlotContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::Trap)
}

struct F64SlotContext<'a> {
    f64_slots: &'a mut [f64],
    i32_slots: &'a mut [i32], // For comparison operations that return i32
    locals: &'a mut [Val],
    src1: F64SlotOperand,
    src2: Option<F64SlotOperand>,
    dst: Slot,
}

impl<'a> F64SlotContext<'a> {
    #[inline]
    fn get_operand(&self, operand: &F64SlotOperand) -> Result<f64, RuntimeError> {
        match operand {
            F64SlotOperand::Slot(idx) => Ok(self.f64_slots[*idx as usize]),
            F64SlotOperand::Const(val) => Ok(*val),
            F64SlotOperand::Param(idx) => self.locals[*idx as usize].to_f64(),
        }
    }
}

fn f64_slot_local_get(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_slots[ctx.dst.index() as usize] = val;
    Ok(())
}

fn f64_slot_local_set(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    // dst.index() is the local variable index, src1 is the value source
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.locals[ctx.dst.index() as usize] = Val::Num(Num::F64(val));
    Ok(())
}

fn f64_slot_const(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_slots[ctx.dst.index() as usize] = val;
    Ok(())
}

fn f64_slot_add(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f64_slots[ctx.dst.index() as usize] = lhs + rhs;
    Ok(())
}

fn f64_slot_sub(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f64_slots[ctx.dst.index() as usize] = lhs - rhs;
    Ok(())
}

fn f64_slot_mul(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f64_slots[ctx.dst.index() as usize] = lhs * rhs;
    Ok(())
}

fn f64_slot_div(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f64_slots[ctx.dst.index() as usize] = lhs / rhs;
    Ok(())
}

fn f64_slot_min(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f64_slots[ctx.dst.index() as usize] = if lhs.is_nan() || rhs.is_nan() {
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

fn f64_slot_max(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f64_slots[ctx.dst.index() as usize] = if lhs.is_nan() || rhs.is_nan() {
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

fn f64_slot_copysign(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.f64_slots[ctx.dst.index() as usize] = lhs.copysign(rhs);
    Ok(())
}

// Unary operations
fn f64_slot_abs(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_slots[ctx.dst.index() as usize] = val.abs();
    Ok(())
}

fn f64_slot_neg(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_slots[ctx.dst.index() as usize] = -val;
    Ok(())
}

fn f64_slot_ceil(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_slots[ctx.dst.index() as usize] = val.ceil();
    Ok(())
}

fn f64_slot_floor(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_slots[ctx.dst.index() as usize] = val.floor();
    Ok(())
}

fn f64_slot_trunc(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_slots[ctx.dst.index() as usize] = val.trunc();
    Ok(())
}

fn f64_slot_nearest(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_slots[ctx.dst.index() as usize] = val.round_ties_even();
    Ok(())
}

fn f64_slot_sqrt(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let val = ctx.get_operand(&ctx.src1)?;
    ctx.f64_slots[ctx.dst.index() as usize] = val.sqrt();
    Ok(())
}

// Comparison operations (return i32)
fn f64_slot_eq(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs == rhs { 1 } else { 0 };
    Ok(())
}

fn f64_slot_ne(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs != rhs { 1 } else { 0 };
    Ok(())
}

fn f64_slot_lt(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs < rhs { 1 } else { 0 };
    Ok(())
}

fn f64_slot_gt(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs > rhs { 1 } else { 0 };
    Ok(())
}

fn f64_slot_le(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs <= rhs { 1 } else { 0 };
    Ok(())
}

fn f64_slot_ge(ctx: F64SlotContext) -> Result<(), RuntimeError> {
    let lhs = ctx.get_operand(&ctx.src1)?;
    let rhs = ctx.get_operand(&ctx.src2.as_ref().unwrap())?;
    ctx.i32_slots[ctx.dst.index() as usize] = if lhs >= rhs { 1 } else { 0 };
    Ok(())
}

lazy_static! {
    static ref F64_SLOT_HANDLER_TABLE: Vec<F64SlotHandler> = {
        let mut table: Vec<F64SlotHandler> = vec![f64_slot_invalid_handler; 256];

        // Special handlers
        table[HANDLER_IDX_LOCAL_GET] = f64_slot_local_get;
        table[HANDLER_IDX_LOCAL_SET] = f64_slot_local_set;
        table[HANDLER_IDX_F64_CONST] = f64_slot_const;

        // Binary operations
        table[HANDLER_IDX_F64_ADD] = f64_slot_add;
        table[HANDLER_IDX_F64_SUB] = f64_slot_sub;
        table[HANDLER_IDX_F64_MUL] = f64_slot_mul;
        table[HANDLER_IDX_F64_DIV] = f64_slot_div;
        table[HANDLER_IDX_F64_MIN] = f64_slot_min;
        table[HANDLER_IDX_F64_MAX] = f64_slot_max;
        table[HANDLER_IDX_F64_COPYSIGN] = f64_slot_copysign;

        // Unary operations
        table[HANDLER_IDX_F64_ABS] = f64_slot_abs;
        table[HANDLER_IDX_F64_NEG] = f64_slot_neg;
        table[HANDLER_IDX_F64_CEIL] = f64_slot_ceil;
        table[HANDLER_IDX_F64_FLOOR] = f64_slot_floor;
        table[HANDLER_IDX_F64_TRUNC] = f64_slot_trunc;
        table[HANDLER_IDX_F64_NEAREST] = f64_slot_nearest;
        table[HANDLER_IDX_F64_SQRT] = f64_slot_sqrt;

        // Comparison operations (return i32)
        table[HANDLER_IDX_F64_EQ] = f64_slot_eq;
        table[HANDLER_IDX_F64_NE] = f64_slot_ne;
        table[HANDLER_IDX_F64_LT] = f64_slot_lt;
        table[HANDLER_IDX_F64_GT] = f64_slot_gt;
        table[HANDLER_IDX_F64_LE] = f64_slot_le;
        table[HANDLER_IDX_F64_GE] = f64_slot_ge;

        table
    };
}

// Conversion Slot handlers
struct ConversionSlotContext<'a> {
    slot_file: &'a mut SlotFile,
    src: Slot,
    dst: Slot,
}

type ConversionSlotHandler = fn(ConversionSlotContext) -> Result<(), RuntimeError>;

fn conversion_slot_invalid_handler(_ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

// i32 -> i64
fn conv_i64_extend_i32_s(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_i32(ctx.src.index());
    ctx.slot_file.set_i64(ctx.dst.index(), val as i64);
    Ok(())
}

fn conv_i64_extend_i32_u(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_i32(ctx.src.index());
    ctx.slot_file.set_i64(ctx.dst.index(), (val as u32) as i64);
    Ok(())
}

// i64 -> i32
fn conv_i32_wrap_i64(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_i64(ctx.src.index());
    ctx.slot_file.set_i32(ctx.dst.index(), val as i32);
    Ok(())
}

// f32 -> i32
fn conv_i32_trunc_f32_s(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f32(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < (i32::MIN as f32) || truncated > (i32::MAX as f32) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.slot_file.set_i32(ctx.dst.index(), truncated as i32);
    Ok(())
}

fn conv_i32_trunc_f32_u(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f32(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < 0.0 || truncated > (u32::MAX as f32) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.slot_file
        .set_i32(ctx.dst.index(), (truncated as u32) as i32);
    Ok(())
}

// f64 -> i32
fn conv_i32_trunc_f64_s(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f64(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < (i32::MIN as f64) || truncated > (i32::MAX as f64) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.slot_file.set_i32(ctx.dst.index(), truncated as i32);
    Ok(())
}

fn conv_i32_trunc_f64_u(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f64(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < 0.0 || truncated > (u32::MAX as f64) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.slot_file
        .set_i32(ctx.dst.index(), (truncated as u32) as i32);
    Ok(())
}

// f32 -> i64
fn conv_i64_trunc_f32_s(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f32(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < (i64::MIN as f32) || truncated >= (i64::MAX as f32) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.slot_file.set_i64(ctx.dst.index(), truncated as i64);
    Ok(())
}

fn conv_i64_trunc_f32_u(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f32(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < 0.0 || truncated >= (u64::MAX as f32) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.slot_file
        .set_i64(ctx.dst.index(), (truncated as u64) as i64);
    Ok(())
}

// f64 -> i64
fn conv_i64_trunc_f64_s(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f64(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < (i64::MIN as f64) || truncated >= (i64::MAX as f64) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.slot_file.set_i64(ctx.dst.index(), truncated as i64);
    Ok(())
}

fn conv_i64_trunc_f64_u(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f64(ctx.src.index());
    if val.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val.trunc();
    if truncated < 0.0 || truncated >= (u64::MAX as f64) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.slot_file
        .set_i64(ctx.dst.index(), (truncated as u64) as i64);
    Ok(())
}

// Saturating truncations (i32)
fn conv_i32_trunc_sat_f32_s(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f32(ctx.src.index());
    let result = if val.is_nan() {
        0
    } else if val <= (i32::MIN as f32) {
        i32::MIN
    } else if val >= (i32::MAX as f32) {
        i32::MAX
    } else {
        val.trunc() as i32
    };
    ctx.slot_file.set_i32(ctx.dst.index(), result);
    Ok(())
}

fn conv_i32_trunc_sat_f32_u(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f32(ctx.src.index());
    let result = if val.is_nan() || val <= 0.0 {
        0
    } else if val >= (u32::MAX as f32) {
        u32::MAX
    } else {
        val.trunc() as u32
    };
    ctx.slot_file.set_i32(ctx.dst.index(), result as i32);
    Ok(())
}

fn conv_i32_trunc_sat_f64_s(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f64(ctx.src.index());
    let result = if val.is_nan() {
        0
    } else if val <= (i32::MIN as f64) {
        i32::MIN
    } else if val >= (i32::MAX as f64) {
        i32::MAX
    } else {
        val.trunc() as i32
    };
    ctx.slot_file.set_i32(ctx.dst.index(), result);
    Ok(())
}

fn conv_i32_trunc_sat_f64_u(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f64(ctx.src.index());
    let result = if val.is_nan() || val <= 0.0 {
        0
    } else if val >= (u32::MAX as f64) {
        u32::MAX
    } else {
        val.trunc() as u32
    };
    ctx.slot_file.set_i32(ctx.dst.index(), result as i32);
    Ok(())
}

// Saturating truncations (i64)
fn conv_i64_trunc_sat_f32_s(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f32(ctx.src.index());
    let result = if val.is_nan() {
        0
    } else if val <= (i64::MIN as f32) {
        i64::MIN
    } else if val >= (i64::MAX as f32) {
        i64::MAX
    } else {
        val.trunc() as i64
    };
    ctx.slot_file.set_i64(ctx.dst.index(), result);
    Ok(())
}

fn conv_i64_trunc_sat_f32_u(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f32(ctx.src.index());
    let result = if val.is_nan() || val <= 0.0 {
        0
    } else if val >= (u64::MAX as f32) {
        u64::MAX
    } else {
        val.trunc() as u64
    };
    ctx.slot_file.set_i64(ctx.dst.index(), result as i64);
    Ok(())
}

fn conv_i64_trunc_sat_f64_s(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f64(ctx.src.index());
    let result = if val.is_nan() {
        0
    } else if val <= (i64::MIN as f64) {
        i64::MIN
    } else if val >= (i64::MAX as f64) {
        i64::MAX
    } else {
        val.trunc() as i64
    };
    ctx.slot_file.set_i64(ctx.dst.index(), result);
    Ok(())
}

fn conv_i64_trunc_sat_f64_u(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f64(ctx.src.index());
    let result = if val.is_nan() || val <= 0.0 {
        0
    } else if val >= (u64::MAX as f64) {
        u64::MAX
    } else {
        val.trunc() as u64
    };
    ctx.slot_file.set_i64(ctx.dst.index(), result as i64);
    Ok(())
}

// i32 -> f32
fn conv_f32_convert_i32_s(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_i32(ctx.src.index());
    ctx.slot_file.set_f32(ctx.dst.index(), val as f32);
    Ok(())
}

fn conv_f32_convert_i32_u(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_i32(ctx.src.index());
    ctx.slot_file.set_f32(ctx.dst.index(), (val as u32) as f32);
    Ok(())
}

// i64 -> f32
fn conv_f32_convert_i64_s(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_i64(ctx.src.index());
    ctx.slot_file.set_f32(ctx.dst.index(), val as f32);
    Ok(())
}

fn conv_f32_convert_i64_u(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_i64(ctx.src.index());
    ctx.slot_file.set_f32(ctx.dst.index(), (val as u64) as f32);
    Ok(())
}

// i32 -> f64
fn conv_f64_convert_i32_s(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_i32(ctx.src.index());
    ctx.slot_file.set_f64(ctx.dst.index(), val as f64);
    Ok(())
}

fn conv_f64_convert_i32_u(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_i32(ctx.src.index());
    ctx.slot_file.set_f64(ctx.dst.index(), (val as u32) as f64);
    Ok(())
}

// i64 -> f64
fn conv_f64_convert_i64_s(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_i64(ctx.src.index());
    ctx.slot_file.set_f64(ctx.dst.index(), val as f64);
    Ok(())
}

fn conv_f64_convert_i64_u(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_i64(ctx.src.index());
    ctx.slot_file.set_f64(ctx.dst.index(), (val as u64) as f64);
    Ok(())
}

// f64 -> f32
fn conv_f32_demote_f64(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f64(ctx.src.index());
    ctx.slot_file.set_f32(ctx.dst.index(), val as f32);
    Ok(())
}

// f32 -> f64
fn conv_f64_promote_f32(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f32(ctx.src.index());
    ctx.slot_file.set_f64(ctx.dst.index(), val as f64);
    Ok(())
}

// Reinterpret operations
fn conv_i32_reinterpret_f32(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f32(ctx.src.index());
    ctx.slot_file.set_i32(ctx.dst.index(), val.to_bits() as i32);
    Ok(())
}

fn conv_f32_reinterpret_i32(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_i32(ctx.src.index());
    ctx.slot_file
        .set_f32(ctx.dst.index(), f32::from_bits(val as u32));
    Ok(())
}

fn conv_i64_reinterpret_f64(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_f64(ctx.src.index());
    ctx.slot_file.set_i64(ctx.dst.index(), val.to_bits() as i64);
    Ok(())
}

fn conv_f64_reinterpret_i64(ctx: ConversionSlotContext) -> Result<(), RuntimeError> {
    let val = ctx.slot_file.get_i64(ctx.src.index());
    ctx.slot_file
        .set_f64(ctx.dst.index(), f64::from_bits(val as u64));
    Ok(())
}

lazy_static! {
    static ref CONVERSION_SLOT_HANDLER_TABLE: Vec<ConversionSlotHandler> = {
        let mut table: Vec<ConversionSlotHandler> = vec![conversion_slot_invalid_handler; 256];

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

// Memory Load Slot handlers
struct MemoryLoadSlotContext<'a> {
    slot_file: &'a mut SlotFile,
    mem_addr: &'a MemAddr,
    addr: Slot,
    dst: Slot,
    offset: u64,
}

type MemoryLoadSlotHandler = fn(MemoryLoadSlotContext) -> Result<(), RuntimeError>;

fn memory_load_slot_invalid_handler(_ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

#[inline]
fn make_memarg(offset: u64) -> Memarg {
    Memarg {
        offset: offset as u32,
        align: 0,
    }
}

fn mem_load_i32(ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: i32 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.slot_file.set_i32(ctx.dst.index(), val);
    Ok(())
}

fn mem_load_i64(ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: i64 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.slot_file.set_i64(ctx.dst.index(), val);
    Ok(())
}

fn mem_load_f32(ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: f32 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.slot_file.set_f32(ctx.dst.index(), val);
    Ok(())
}

fn mem_load_f64(ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: f64 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.slot_file.set_f64(ctx.dst.index(), val);
    Ok(())
}

fn mem_load_i32_8s(ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: i8 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.slot_file.set_i32(ctx.dst.index(), val as i32);
    Ok(())
}

fn mem_load_i32_8u(ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: u8 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.slot_file.set_i32(ctx.dst.index(), val as i32);
    Ok(())
}

fn mem_load_i32_16s(ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: i16 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.slot_file.set_i32(ctx.dst.index(), val as i32);
    Ok(())
}

fn mem_load_i32_16u(ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: u16 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.slot_file.set_i32(ctx.dst.index(), val as i32);
    Ok(())
}

fn mem_load_i64_8s(ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: i8 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.slot_file.set_i64(ctx.dst.index(), val as i64);
    Ok(())
}

fn mem_load_i64_8u(ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: u8 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.slot_file.set_i64(ctx.dst.index(), val as i64);
    Ok(())
}

fn mem_load_i64_16s(ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: i16 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.slot_file.set_i64(ctx.dst.index(), val as i64);
    Ok(())
}

fn mem_load_i64_16u(ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: u16 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.slot_file.set_i64(ctx.dst.index(), val as i64);
    Ok(())
}

fn mem_load_i64_32s(ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: i32 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.slot_file.set_i64(ctx.dst.index(), val as i64);
    Ok(())
}

fn mem_load_i64_32u(ctx: MemoryLoadSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let memarg = make_memarg(ctx.offset);
    let val: u32 = ctx.mem_addr.load(&memarg, ptr)?;
    ctx.slot_file.set_i64(ctx.dst.index(), val as i64);
    Ok(())
}

lazy_static! {
    static ref MEMORY_LOAD_SLOT_HANDLER_TABLE: Vec<MemoryLoadSlotHandler> = {
        let mut table: Vec<MemoryLoadSlotHandler> = vec![memory_load_slot_invalid_handler; 256];

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

// Memory Store Slot handlers
struct MemoryStoreSlotContext<'a> {
    slot_file: &'a SlotFile,
    mem_addr: &'a MemAddr,
    addr: Slot,
    value: Slot,
    offset: u64,
}

type MemoryStoreSlotHandler = fn(MemoryStoreSlotContext) -> Result<(), RuntimeError>;

fn memory_store_slot_invalid_handler(_ctx: MemoryStoreSlotContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

fn mem_store_i32(ctx: MemoryStoreSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let val = ctx.slot_file.get_i32(ctx.value.index());
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_i64(ctx: MemoryStoreSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let val = ctx.slot_file.get_i64(ctx.value.index());
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_f32(ctx: MemoryStoreSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let val = ctx.slot_file.get_f32(ctx.value.index());
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_f64(ctx: MemoryStoreSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let val = ctx.slot_file.get_f64(ctx.value.index());
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_i32_8(ctx: MemoryStoreSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let val = ctx.slot_file.get_i32(ctx.value.index()) as u8;
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_i32_16(ctx: MemoryStoreSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let val = ctx.slot_file.get_i32(ctx.value.index()) as u16;
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_i64_8(ctx: MemoryStoreSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let val = ctx.slot_file.get_i64(ctx.value.index()) as u8;
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_i64_16(ctx: MemoryStoreSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let val = ctx.slot_file.get_i64(ctx.value.index()) as u16;
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

fn mem_store_i64_32(ctx: MemoryStoreSlotContext) -> Result<(), RuntimeError> {
    let ptr = ctx.slot_file.get_i32(ctx.addr.index());
    let val = ctx.slot_file.get_i64(ctx.value.index()) as u32;
    let memarg = make_memarg(ctx.offset);
    ctx.mem_addr.store(&memarg, ptr, val)?;
    Ok(())
}

lazy_static! {
    static ref MEMORY_STORE_SLOT_HANDLER_TABLE: Vec<MemoryStoreSlotHandler> = {
        let mut table: Vec<MemoryStoreSlotHandler> = vec![memory_store_slot_invalid_handler; 256];

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

// Memory Ops Slot handlers (size, grow, copy, init, fill)
struct MemoryOpsSlotContext<'a> {
    slot_file: &'a mut SlotFile,
    mem_addr: &'a MemAddr,
    module_inst: &'a ModuleInst,
    dst: Option<Slot>,
    args: Vec<Slot>,
    data_index: u32,
}

type MemoryOpsSlotHandler = fn(MemoryOpsSlotContext) -> Result<(), RuntimeError>;

fn memory_ops_slot_invalid_handler(_ctx: MemoryOpsSlotContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

fn mem_ops_size(ctx: MemoryOpsSlotContext) -> Result<(), RuntimeError> {
    let size = ctx.mem_addr.mem_size();
    if let Some(dst) = ctx.dst {
        ctx.slot_file.set_i32(dst.index(), size);
    }
    Ok(())
}

fn mem_ops_grow(ctx: MemoryOpsSlotContext) -> Result<(), RuntimeError> {
    let delta = ctx.slot_file.get_i32(ctx.args[0].index());
    let delta_u32: u32 = delta
        .try_into()
        .map_err(|_| RuntimeError::InvalidParameterCount)?;
    let prev_size = ctx.mem_addr.mem_grow(
        delta_u32
            .try_into()
            .map_err(|_| RuntimeError::InvalidParameterCount)?,
    );
    if let Some(dst) = ctx.dst {
        ctx.slot_file.set_i32(dst.index(), prev_size);
    }
    Ok(())
}

fn mem_ops_copy(ctx: MemoryOpsSlotContext) -> Result<(), RuntimeError> {
    let dest = ctx.slot_file.get_i32(ctx.args[0].index());
    let src = ctx.slot_file.get_i32(ctx.args[1].index());
    let len = ctx.slot_file.get_i32(ctx.args[2].index());
    ctx.mem_addr.memory_copy(dest, src, len)?;
    Ok(())
}

fn mem_ops_init(ctx: MemoryOpsSlotContext) -> Result<(), RuntimeError> {
    let dest = ctx.slot_file.get_i32(ctx.args[0].index()) as usize;
    let offset = ctx.slot_file.get_i32(ctx.args[1].index()) as usize;
    let len = ctx.slot_file.get_i32(ctx.args[2].index()) as usize;

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

fn mem_ops_fill(ctx: MemoryOpsSlotContext) -> Result<(), RuntimeError> {
    let dest = ctx.slot_file.get_i32(ctx.args[0].index());
    let val = ctx.slot_file.get_i32(ctx.args[1].index()) as u8;
    let size = ctx.slot_file.get_i32(ctx.args[2].index());
    ctx.mem_addr.memory_fill(dest, val, size)?;
    Ok(())
}

lazy_static! {
    static ref MEMORY_OPS_SLOT_HANDLER_TABLE: Vec<MemoryOpsSlotHandler> = {
        let mut table: Vec<MemoryOpsSlotHandler> = vec![memory_ops_slot_invalid_handler; 256];

        table[HANDLER_IDX_MEMORY_SIZE] = mem_ops_size;
        table[HANDLER_IDX_MEMORY_GROW] = mem_ops_grow;
        table[HANDLER_IDX_MEMORY_COPY] = mem_ops_copy;
        table[HANDLER_IDX_MEMORY_INIT] = mem_ops_init;
        table[HANDLER_IDX_MEMORY_FILL] = mem_ops_fill;

        table
    };
}

// Select Slot handlers
struct SelectSlotContext<'a> {
    slot_file: &'a mut SlotFile,
    dst: Slot,
    val1: Slot,
    val2: Slot,
    cond: Slot,
}

type SelectSlotHandler = fn(SelectSlotContext) -> Result<(), RuntimeError>;

fn select_slot_invalid_handler(_ctx: SelectSlotContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

fn select_i32(ctx: SelectSlotContext) -> Result<(), RuntimeError> {
    let cond = ctx.slot_file.get_i32(ctx.cond.index());
    let result = if cond != 0 {
        ctx.slot_file.get_i32(ctx.val1.index())
    } else {
        ctx.slot_file.get_i32(ctx.val2.index())
    };
    ctx.slot_file.set_i32(ctx.dst.index(), result);
    Ok(())
}

fn select_i64(ctx: SelectSlotContext) -> Result<(), RuntimeError> {
    let cond = ctx.slot_file.get_i32(ctx.cond.index());
    let result = if cond != 0 {
        ctx.slot_file.get_i64(ctx.val1.index())
    } else {
        ctx.slot_file.get_i64(ctx.val2.index())
    };
    ctx.slot_file.set_i64(ctx.dst.index(), result);
    Ok(())
}

fn select_f32(ctx: SelectSlotContext) -> Result<(), RuntimeError> {
    let cond = ctx.slot_file.get_i32(ctx.cond.index());
    let result = if cond != 0 {
        ctx.slot_file.get_f32(ctx.val1.index())
    } else {
        ctx.slot_file.get_f32(ctx.val2.index())
    };
    ctx.slot_file.set_f32(ctx.dst.index(), result);
    Ok(())
}

fn select_f64(ctx: SelectSlotContext) -> Result<(), RuntimeError> {
    let cond = ctx.slot_file.get_i32(ctx.cond.index());
    let result = if cond != 0 {
        ctx.slot_file.get_f64(ctx.val1.index())
    } else {
        ctx.slot_file.get_f64(ctx.val2.index())
    };
    ctx.slot_file.set_f64(ctx.dst.index(), result);
    Ok(())
}

pub const HANDLER_IDX_SELECT_I32: usize = 0xF0;
pub const HANDLER_IDX_SELECT_I64: usize = 0xF1;
pub const HANDLER_IDX_SELECT_F32: usize = 0xF2;
pub const HANDLER_IDX_SELECT_F64: usize = 0xF3;

lazy_static! {
    static ref SELECT_SLOT_HANDLER_TABLE: Vec<SelectSlotHandler> = {
        let mut table: Vec<SelectSlotHandler> = vec![select_slot_invalid_handler; 256];

        table[HANDLER_IDX_SELECT_I32] = select_i32;
        table[HANDLER_IDX_SELECT_I64] = select_i64;
        table[HANDLER_IDX_SELECT_F32] = select_f32;
        table[HANDLER_IDX_SELECT_F64] = select_f64;

        table
    };
}

// GlobalGetSlot handler constants
pub const HANDLER_IDX_GLOBAL_GET_I32: usize = 0xF4;
pub const HANDLER_IDX_GLOBAL_GET_I64: usize = 0xF5;
pub const HANDLER_IDX_GLOBAL_GET_F32: usize = 0xF6;
pub const HANDLER_IDX_GLOBAL_GET_F64: usize = 0xF7;

// GlobalSetSlot handler constants
pub const HANDLER_IDX_GLOBAL_SET_I32: usize = 0xF8;
pub const HANDLER_IDX_GLOBAL_SET_I64: usize = 0xF9;
pub const HANDLER_IDX_GLOBAL_SET_F32: usize = 0xFA;
pub const HANDLER_IDX_GLOBAL_SET_F64: usize = 0xFB;

// TableRefSlot handler constants
pub const HANDLER_IDX_REF_NULL_SLOT: usize = 0xFC;
pub const HANDLER_IDX_REF_IS_NULL_SLOT: usize = 0xFD;
pub const HANDLER_IDX_TABLE_GET_SLOT: usize = 0xFE;
pub const HANDLER_IDX_TABLE_SET_SLOT: usize = 0xFF;
pub const HANDLER_IDX_TABLE_FILL_SLOT: usize = 0x100;

// RefLocalSlot handler constants
pub const HANDLER_IDX_REF_LOCAL_GET_SLOT: usize = 0x101;
pub const HANDLER_IDX_REF_LOCAL_SET_SLOT: usize = 0x102;

// CallWasiSlot handler constant
pub const HANDLER_IDX_CALL_WASI_SLOT: usize = 0x103;

// RefLocalSlot handler infrastructure
struct RefLocalSlotContext<'a> {
    slot_file: &'a mut SlotFile,
    locals: &'a mut Vec<Val>,
    dst: u16,
    src: u16,
    local_idx: u16,
}

type RefLocalSlotHandler = fn(RefLocalSlotContext) -> Result<(), RuntimeError>;

fn ref_local_slot_invalid_handler(_ctx: RefLocalSlotContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

// local.get for ref type: [] -> [ref]
fn ref_local_slot_get(ctx: RefLocalSlotContext) -> Result<(), RuntimeError> {
    let val = ctx
        .locals
        .get(ctx.local_idx as usize)
        .ok_or(RuntimeError::LocalIndexOutOfBounds)?
        .clone();
    if let Val::Ref(r) = val {
        ctx.slot_file.set_ref(ctx.dst, r);
    }
    Ok(())
}

// local.set for ref type: [ref] -> []
fn ref_local_slot_set(ctx: RefLocalSlotContext) -> Result<(), RuntimeError> {
    let ref_val = ctx.slot_file.get_ref(ctx.src);
    let idx = ctx.local_idx as usize;
    if idx < ctx.locals.len() {
        ctx.locals[idx] = Val::Ref(ref_val);
    }
    Ok(())
}

lazy_static! {
    static ref REF_LOCAL_SLOT_HANDLER_TABLE: Vec<RefLocalSlotHandler> = {
        let mut table: Vec<RefLocalSlotHandler> = vec![ref_local_slot_invalid_handler; 0x103];

        table[HANDLER_IDX_REF_LOCAL_GET_SLOT] = ref_local_slot_get;
        table[HANDLER_IDX_REF_LOCAL_SET_SLOT] = ref_local_slot_set;

        table
    };
}

// TableRefSlot handler infrastructure
struct TableRefSlotContext<'a> {
    slot_file: &'a mut SlotFile,
    module_inst: &'a Rc<ModuleInst>,
    table_idx: u32,
    slots: [u16; 3],
    #[allow(dead_code)]
    ref_type: RefType,
}

type TableRefSlotHandler = fn(TableRefSlotContext) -> Result<(), RuntimeError>;

fn table_ref_slot_invalid_handler(_ctx: TableRefSlotContext) -> Result<(), RuntimeError> {
    Err(RuntimeError::InvalidHandlerIndex)
}

// ref.null: [] -> [ref]
// dst = slots[0]
fn table_ref_slot_null(ctx: TableRefSlotContext) -> Result<(), RuntimeError> {
    ctx.slot_file.set_ref(ctx.slots[0], Ref::RefNull);
    Ok(())
}

// ref.is_null: [ref] -> [i32]
// src = slots[1], dst = slots[0]
fn table_ref_slot_is_null(ctx: TableRefSlotContext) -> Result<(), RuntimeError> {
    let ref_val = ctx.slot_file.get_ref(ctx.slots[1]);
    let is_null = match ref_val {
        Ref::RefNull => 1,
        _ => 0,
    };
    ctx.slot_file.set_i32(ctx.slots[0], is_null);
    Ok(())
}

// table.get: [i32] -> [ref]
// idx = slots[1], dst = slots[0]
fn table_ref_slot_get(ctx: TableRefSlotContext) -> Result<(), RuntimeError> {
    let table_addr = ctx
        .module_inst
        .table_addrs
        .get(ctx.table_idx as usize)
        .ok_or(RuntimeError::TableNotFound)?;
    let index = ctx.slot_file.get_i32(ctx.slots[1]) as usize;
    let val = table_addr.get(index);
    match val {
        Val::Ref(r) => {
            ctx.slot_file.set_ref(ctx.slots[0], r);
            Ok(())
        }
        _ => Err(RuntimeError::TypeMismatch),
    }
}

// table.set: [i32, ref] -> []
// idx = slots[0], val = slots[1]
fn table_ref_slot_set(ctx: TableRefSlotContext) -> Result<(), RuntimeError> {
    let table_addr = ctx
        .module_inst
        .table_addrs
        .get(ctx.table_idx as usize)
        .ok_or(RuntimeError::TableNotFound)?;
    let index = ctx.slot_file.get_i32(ctx.slots[0]) as usize;
    let ref_val = ctx.slot_file.get_ref(ctx.slots[1]);
    table_addr.set(index, Val::Ref(ref_val))
}

// table.fill: [i32, ref, i32] -> []
// i = slots[0], val = slots[1], n = slots[2]
fn table_ref_slot_fill(ctx: TableRefSlotContext) -> Result<(), RuntimeError> {
    let table_addr = ctx
        .module_inst
        .table_addrs
        .get(ctx.table_idx as usize)
        .ok_or(RuntimeError::TableNotFound)?;
    let i = ctx.slot_file.get_i32(ctx.slots[0]) as usize;
    let ref_val = ctx.slot_file.get_ref(ctx.slots[1]);
    let n = ctx.slot_file.get_i32(ctx.slots[2]) as usize;
    table_addr.fill(i, Val::Ref(ref_val), n)
}

lazy_static! {
    static ref TABLE_REF_SLOT_HANDLER_TABLE: Vec<TableRefSlotHandler> = {
        let mut table: Vec<TableRefSlotHandler> = vec![table_ref_slot_invalid_handler; 0x101];

        table[HANDLER_IDX_REF_NULL_SLOT] = table_ref_slot_null;
        table[HANDLER_IDX_REF_IS_NULL_SLOT] = table_ref_slot_is_null;
        table[HANDLER_IDX_TABLE_GET_SLOT] = table_ref_slot_get;
        table[HANDLER_IDX_TABLE_SET_SLOT] = table_ref_slot_set;
        table[HANDLER_IDX_TABLE_FILL_SLOT] = table_ref_slot_fill;

        table
    };
}
