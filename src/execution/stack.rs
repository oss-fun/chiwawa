use super::value::*;
use crate::error::RuntimeError;
use crate::execution::{
    func::*,
    mem::MemAddr,
    memoization::{GlobalAccessTracker, LocalAccessTracker},
    migration,
    module::*,
    slots::{Slot, SlotFile},
};
use crate::structure::types::LabelIdx as StructureLabelIdx;
use crate::structure::{instructions::*, types::*};
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Sub};
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

// Helper function for storing results to target
fn store_result(
    ctx: &mut ExecutionContext,
    result_val: Val,
    store_target: &Option<StoreTarget>,
) -> Result<(), RuntimeError> {
    if let Some(target) = store_target {
        match target {
            StoreTarget::Local(idx) => {
                let index = *idx as usize;
                if index >= ctx.frame.locals.len() {
                    return Err(RuntimeError::LocalIndexOutOfBounds);
                }
                ctx.frame.locals[index] = result_val;
                ctx.frame.local_versions[index] += 1;
            }
            StoreTarget::Global(idx) => {
                let module_inst = ctx
                    .frame
                    .module
                    .upgrade()
                    .ok_or(RuntimeError::ModuleInstanceGone)?;
                let global_addr = module_inst.global_addrs.get_by_idx(GlobalIdx(*idx)).clone();
                global_addr.set(result_val)?;
            }
        }
    } else {
        ctx.value_stack.push(result_val);
    }
    Ok(())
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
    },
    MemArg(Memarg),
    BrTable {
        targets: Vec<Operand>,
        default: Box<Operand>,
    },
    CallIndirect {
        type_idx: TypeIdx,
        table_idx: TableIdx,
    },
    Block {
        arity: usize,
        param_count: usize,
        is_loop: bool,
        start_ip: usize,
        end_ip: usize,
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
    /// Legacy stack-based instruction format (backward compatible)
    Legacy {
        handler_index: usize,
        operand: Operand,
        /// Slots to copy from slot_file to value_stack before executing this instruction
        slots_to_stack: Vec<crate::execution::slots::Slot>,
        /// Slots to write back from value_stack to slot_file after executing this instruction
        stack_to_slots: Vec<crate::execution::slots::Slot>,
    },
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
}

impl ProcessedInstr {
    /// Get handler_index (for backward compatibility)
    #[inline(always)]
    pub fn handler_index(&self) -> usize {
        match self {
            ProcessedInstr::Legacy { handler_index, .. } => *handler_index,
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
            ProcessedInstr::RefLocalSlot { handler_index, .. } => *handler_index,
            ProcessedInstr::TableRefSlot { handler_index, .. } => *handler_index,
        }
    }

    /// Get operand reference (for backward compatibility)
    #[inline(always)]
    pub fn operand(&self) -> &Operand {
        match self {
            ProcessedInstr::Legacy { operand, .. } => operand,
            _ => panic!("operand() called on slot-based instruction"),
        }
    }

    /// Get mutable operand reference (for backward compatibility)
    #[inline(always)]
    pub fn operand_mut(&mut self) -> &mut Operand {
        match self {
            ProcessedInstr::Legacy { operand, .. } => operand,
            _ => panic!("operand_mut() called on slot-based instruction"),
        }
    }

    /// Set handler_index (for backward compatibility)
    #[inline(always)]
    pub fn set_handler_index(&mut self, new_handler_index: usize) {
        match self {
            ProcessedInstr::Legacy { handler_index, .. } => *handler_index = new_handler_index,
            _ => {} // No-op for slot-based instructions
        }
    }
}

pub struct ExecutionContext<'a> {
    pub frame: &'a mut crate::execution::stack::Frame,
    pub value_stack: &'a mut Vec<Val>,
    pub ip: usize,
    pub block_has_mutable_op: bool,
    pub accessed_globals: &'a mut Option<GlobalAccessTracker>,
    pub accessed_locals: &'a mut Option<LocalAccessTracker>,
}

#[derive(Clone, Debug)]
enum HandlerResult {
    Continue(usize),
    Return,
    Invoke(FuncAddr),
    Branch {
        target_ip: usize,
        values_to_push: Vec<Val>,
        branch_depth: usize,
    },
    PushLabelStack {
        label: Label,
        next_ip: usize,
        start_ip: usize,
        end_ip: usize,
    },
    PopLabelStack {
        next_ip: usize,
    },
}

type HandlerFn = fn(&mut ExecutionContext, &Operand) -> Result<HandlerResult, RuntimeError>;

#[derive(Clone)]
pub enum ModuleLevelInstr {
    Invoke(FuncAddr),
    Return,
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
// pub const HANDLER_IDX_DATA_DROP: usize = 0xE8;      // Unsupported (0xFC 0x0D for data)

// SIMD Instructions (0xFD prefix) - not implemented
// Vector instructions would start here (0xFD00 - 0xFDFF range)

// Thread Instructions (0xFE prefix) - not implemented
// Atomic operations would use 0xFE prefix

pub const MAX_HANDLER_INDEX: usize = 0x168;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stacks {
    pub activation_frame_stack: Vec<FrameStack>,
}

impl Stacks {
    pub fn new(funcaddr: &FuncAddr, params: Vec<Val>) -> Result<Stacks, RuntimeError> {
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

                // Initialize SlotFile if slot allocation is present
                let slot_file = code.slot_allocation.as_ref().map(|alloc| {
                    use crate::execution::slots::SlotFile;
                    SlotFile::new(
                        alloc.i32_count,
                        alloc.i64_count,
                        alloc.f32_count,
                        alloc.f64_count,
                        alloc.ref_count,
                        alloc.v128_count,
                    )
                });

                let initial_frame = FrameStack {
                    frame: Frame {
                        local_versions: vec![0; locals.len()],
                        locals,
                        module: module.clone(),
                        n: type_.results.len(),
                        slot_file,
                        result_slot: code.result_slot,
                    },
                    label_stack: vec![LabelStack {
                        label: Label {
                            locals_num: type_.results.len(),
                            arity: type_.results.len(),
                            is_loop: false,
                            stack_height: 0, // Function level starts with empty stack
                            return_ip: 0,    // No return needed for function level
                            start_ip: 0,     // Function level starts at 0
                            end_ip: code.body.len(), // Function level ends at body length
                            input_stack: vec![], // Function level has empty input
                            is_immutable: None, // Will be determined during execution
                        },
                        processed_instrs: code.body.clone(),
                        value_stack: vec![],
                        ip: 0,
                        block_has_mutable_ops: false,
                    }],
                    void: type_.results.is_empty(),
                    instruction_count: 0,
                    global_value_stack: vec![],
                    current_block_accessed_globals: Some(GlobalAccessTracker::new()),
                    current_block_accessed_locals: Some(LocalAccessTracker::new()),
                    enable_checkpoint: false,
                };

                Ok(Stacks {
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub locals: Vec<Val>,
    pub local_versions: Vec<u64>,
    #[serde(skip)]
    pub module: Weak<ModuleInst>,
    pub n: usize,
    pub slot_file: Option<crate::execution::slots::SlotFile>, // Used in slot mode
    pub result_slot: Option<crate::execution::slots::Slot>, // Slot for return value (slot mode only)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameStack {
    pub frame: Frame,
    pub label_stack: Vec<LabelStack>,
    pub void: bool,
    pub instruction_count: u64,
    // Global value stack shared across all label stacks
    pub global_value_stack: Vec<Val>,
    #[serde(skip)]
    pub current_block_accessed_globals: Option<GlobalAccessTracker>, // Track accessed globals when enabled
    #[serde(skip)]
    pub current_block_accessed_locals: Option<LocalAccessTracker>, // Track accessed locals when enabled
    #[serde(skip)]
    pub enable_checkpoint: bool,
}

impl FrameStack {
    /// Push a new label stack for a block or loop
    pub fn push_label_stack(&mut self, label: Label, instructions: Vec<ProcessedInstr>) {
        let new_label_stack = LabelStack {
            label,
            processed_instrs: Rc::new(instructions),
            value_stack: vec![],
            ip: 0,
            block_has_mutable_ops: false,
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

    pub fn current_stack_height(&self) -> usize {
        if let Some(current_label) = self.label_stack.last() {
            current_label.value_stack.len()
        } else {
            0
        }
    }

    pub fn apply_call_stack_to_slots(&mut self) {
        let Some(current_label) = self.label_stack.last() else {
            return;
        };
        let call_ip = current_label.ip.saturating_sub(1);
        let Some(ProcessedInstr::Legacy { stack_to_slots, .. }) =
            current_label.processed_instrs.get(call_ip)
        else {
            return;
        };
        if stack_to_slots.is_empty() {
            return;
        }
        let Some(ref mut slot_file) = self.frame.slot_file else {
            return;
        };
        slot_file.write_from_stack(stack_to_slots, &self.global_value_stack);
    }

    /// DTC execution loop with block memoization callbacks
    ///
    /// Uses callback functions to access Runtime's cache from Stack without borrowing conflicts.
    /// - `get_block_cache`: Get block result from cache (returns Some if hit)
    /// - `store_block_cache`: Store block execution result in cache
    pub fn run_dtc_loop<F, G>(
        &mut self,
        _called_func_addr_out: &mut Option<FuncAddr>,
        mut get_block_cache: F,
        mut store_block_cache: G,
        mut execution_stats: Option<&mut super::stats::ExecutionStats>,
        mut tracer: Option<&mut super::trace::Tracer>,
    ) -> Result<Result<Option<ModuleLevelInstr>, RuntimeError>, RuntimeError>
    where
        F: FnMut(usize, usize, &[Val], &[Val], &[u64]) -> Option<Vec<Val>>, // Cache lookup callback with locals and versions
        G: FnMut(usize, usize, &[Val], &[Val], Vec<Val>, GlobalAccessTracker, LocalAccessTracker), // Cache store callback with locals, globals and accessed locals
    {
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
                    let slot_file = self
                        .frame
                        .slot_file
                        .as_mut()
                        .ok_or(RuntimeError::InvalidWasm("SlotFile not initialized"))?;

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
                    let slot_file = self
                        .frame
                        .slot_file
                        .as_mut()
                        .ok_or(RuntimeError::InvalidWasm("SlotFile not initialized"))?;

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
                    let slot_file = self
                        .frame
                        .slot_file
                        .as_mut()
                        .ok_or(RuntimeError::InvalidWasm("SlotFile not initialized"))?;

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
                    let slot_file = self
                        .frame
                        .slot_file
                        .as_mut()
                        .ok_or(RuntimeError::InvalidWasm("SlotFile not initialized"))?;

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
                    let slot_file = self
                        .frame
                        .slot_file
                        .as_mut()
                        .ok_or(RuntimeError::InvalidWasm("SlotFile not initialized"))?;

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
                    let slot_file = self
                        .frame
                        .slot_file
                        .as_mut()
                        .ok_or(RuntimeError::InvalidWasm("SlotFile not initialized"))?;

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
                    let slot_file = self
                        .frame
                        .slot_file
                        .as_ref()
                        .ok_or(RuntimeError::InvalidWasm("SlotFile not initialized"))?;

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
                    let slot_file = self
                        .frame
                        .slot_file
                        .as_mut()
                        .ok_or(RuntimeError::InvalidWasm("SlotFile not initialized"))?;

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
                    let slot_file = self
                        .frame
                        .slot_file
                        .as_mut()
                        .ok_or(RuntimeError::InvalidWasm("SlotFile not initialized"))?;

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

                    let slot_file = self
                        .frame
                        .slot_file
                        .as_mut()
                        .ok_or(RuntimeError::InvalidWasm("SlotFile not initialized"))?;

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
                    let slot_file = self
                        .frame
                        .slot_file
                        .as_ref()
                        .ok_or(RuntimeError::InvalidWasm("SlotFile not initialized"))?;

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
                ProcessedInstr::RefLocalSlot {
                    handler_index,
                    dst,
                    src,
                    local_idx,
                } => {
                    let slot_file = self
                        .frame
                        .slot_file
                        .as_mut()
                        .ok_or(RuntimeError::InvalidWasm("SlotFile not initialized"))?;

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
                    let slot_file = self
                        .frame
                        .slot_file
                        .as_mut()
                        .ok_or(RuntimeError::InvalidWasm("SlotFile not initialized"))?;

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
                ProcessedInstr::Legacy {
                    handler_index,
                    operand,
                    slots_to_stack,
                    stack_to_slots,
                } => {
                    // Copy slots to value_stack before executing legacy instruction
                    if !slots_to_stack.is_empty() {
                        if let Some(ref slot_file) = self.frame.slot_file {
                            // Clear value_stack and sync fresh values from slots
                            self.global_value_stack.clear();
                            for slot in slots_to_stack.iter() {
                                let val = slot_file.get_val(slot);
                                self.global_value_stack.push(val);
                            }
                        }
                    }
                    let stack_to_slots = stack_to_slots.clone();

                    // Execute legacy stack-based instruction
                    let (handler_index, operand) = (handler_index, operand);

                    // Record instruction execution for statistics
                    if let Some(ref mut stats) = execution_stats {
                        stats.record_instruction(*handler_index);
                    }

                    let handler_fn = HANDLER_TABLE
                        .get(*handler_index)
                        .ok_or(RuntimeError::InvalidHandlerIndex)?;

                    if let Some(ref mut tracer) = tracer {
                        let module_inst = self
                            .frame
                            .module
                            .upgrade()
                            .ok_or(RuntimeError::ModuleInstanceGone)?;

                        tracer.trace_instruction(
                            ip,
                            *handler_index,
                            &self.global_value_stack,
                            &self.frame.locals,
                            &module_inst.global_addrs,
                        );
                    }

                    let mut context = ExecutionContext {
                        frame: &mut self.frame,
                        value_stack: &mut self.global_value_stack,
                        ip,
                        block_has_mutable_op: false,
                        accessed_globals: &mut self.current_block_accessed_globals,
                        accessed_locals: &mut self.current_block_accessed_locals,
                    };

                    let result = handler_fn(&mut context, operand);
                    let has_mutable_op = context.block_has_mutable_op;

                    // Track if this block executed a mutable operation
                    if has_mutable_op {
                        current_label_stack.block_has_mutable_ops = true;
                    }

                    match result {
                        Err(e) => {
                            eprintln!(
                                "Error at IP {}, handler_index: {}: {:?}",
                                ip, handler_index, e
                            );
                            return Ok(Err(e));
                        }
                        Ok(handler_result) => {
                            match handler_result {
                                HandlerResult::Continue(next_ip) => {
                                    // Write back results to slot_file if specified
                                    // (value_stack is left as-is for legacy instructions that may follow)
                                    if let Some(ref mut slot_file) = self.frame.slot_file {
                                        slot_file.write_from_stack(
                                            &stack_to_slots,
                                            &self.global_value_stack,
                                        );
                                    }
                                    self.label_stack[current_label_stack_idx].ip = next_ip;
                                }
                                HandlerResult::Return => {
                                    return Ok(Ok(Some(ModuleLevelInstr::Return)));
                                }
                                HandlerResult::Invoke(func_addr) => {
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

                                    self.label_stack[current_label_stack_idx].ip = ip + 1;
                                    return Ok(Ok(Some(ModuleLevelInstr::Invoke(func_addr))));
                                }
                                HandlerResult::Branch {
                                    target_ip,
                                    values_to_push,
                                    branch_depth,
                                } => {
                                    // Use the branch depth directly from the Branch result
                                    // Calculate target label stack index from current position and branch depth
                                    // Branch depth 0 = current block, 1 = parent block, etc.
                                    if branch_depth <= current_label_stack_idx {
                                        let target_depth = current_label_stack_idx - branch_depth;
                                        let target_level = target_depth + 1;

                                        // Mark all blocks being exited as having executed a branch (early exit)
                                        for i in target_level..self.label_stack.len() {
                                            self.label_stack[i].block_has_mutable_ops = true;
                                        }

                                        let current_label_stack =
                                            &self.label_stack[current_label_stack_idx];
                                        let current_stack_height =
                                            current_label_stack.label.stack_height;

                                        // For branch depth 0, we need to exit the current block
                                        if branch_depth == 0 {
                                            self.label_stack.pop();
                                            if self.label_stack.len() > 0 {
                                                current_label_stack_idx =
                                                    self.label_stack.len() - 1;
                                            } else {
                                                return Err(RuntimeError::StackError(
                                                    "Label stack underflow during branch",
                                                ));
                                            }
                                        } else {
                                            // For deeper branches, truncate to the target level
                                            self.label_stack.truncate(target_level);
                                            if self.label_stack.len() > 0 {
                                                current_label_stack_idx =
                                                    self.label_stack.len() - 1;
                                            } else {
                                                return Err(RuntimeError::StackError(
                                                    "Label stack underflow during branch",
                                                ));
                                            }
                                        }

                                        let target_label_stack =
                                            &mut self.label_stack[current_label_stack_idx];

                                        let stack_height = if branch_depth == 0 {
                                            current_stack_height
                                        } else {
                                            target_label_stack.label.stack_height
                                        };

                                        let current_len = self.global_value_stack.len();
                                        let actual_height = stack_height.min(current_len);
                                        self.global_value_stack
                                            .splice(actual_height.., values_to_push);

                                        // Apply the End instruction's stack_to_slots if in slot mode
                                        // The End instruction is at target_ip - 1 (since target_ip = End_IP + 1)
                                        if let (Some(ref mut slot_file), Some(end_instr)) = (
                                            self.frame.slot_file.as_mut(),
                                            target_ip.checked_sub(1).and_then(|ip| {
                                                self.label_stack[current_label_stack_idx]
                                                    .processed_instrs
                                                    .get(ip)
                                            }),
                                        ) {
                                            if let ProcessedInstr::Legacy {
                                                stack_to_slots, ..
                                            } = end_instr
                                            {
                                                let consumed = slot_file.write_from_stack(
                                                    stack_to_slots,
                                                    &self.global_value_stack,
                                                );
                                                self.global_value_stack.truncate(
                                                    self.global_value_stack.len() - consumed,
                                                );
                                            }
                                        }

                                        let target_label_stack =
                                            &mut self.label_stack[current_label_stack_idx];
                                        target_label_stack.ip = target_ip;

                                        // Reset tracking for new block after branch
                                        if self.current_block_accessed_globals.is_some() {
                                            self.current_block_accessed_globals =
                                                Some(GlobalAccessTracker::new());
                                        }
                                        if self.current_block_accessed_locals.is_some() {
                                            self.current_block_accessed_locals =
                                                Some(LocalAccessTracker::new());
                                        }
                                    } else {
                                        return Err(RuntimeError::InvalidBranchTarget);
                                    }
                                    continue;
                                }
                                HandlerResult::PushLabelStack {
                                    label,
                                    next_ip,
                                    start_ip,
                                    end_ip,
                                } => {
                                    let mut cache_hit = false;
                                    let mut cached_result_values = Vec::new();

                                    if current_label_stack_idx > 0
                                        && label.is_immutable != Some(false)
                                    {
                                        if let Some(cached_result) = get_block_cache(
                                            start_ip,
                                            end_ip,
                                            &label.input_stack,
                                            &self.frame.locals,
                                            &self.frame.local_versions,
                                        ) {
                                            cached_result_values = cached_result;
                                            cache_hit = true;
                                        }
                                    }

                                    // Apply cache result if hit
                                    if cache_hit {
                                        let block_start_height = label.stack_height;
                                        let current_len = self.global_value_stack.len();
                                        let actual_height = block_start_height.min(current_len);
                                        self.global_value_stack
                                            .splice(actual_height.., cached_result_values);
                                    }

                                    // Create new label stack
                                    let current_instrs =
                                        &self.label_stack[current_label_stack_idx].processed_instrs;
                                    // Inherit parent's mutable status if this is a nested structure
                                    let parent_has_mutable_ops = self.label_stack
                                        [current_label_stack_idx]
                                        .block_has_mutable_ops;
                                    let new_label_stack = LabelStack {
                                        label,
                                        processed_instrs: current_instrs.clone(),
                                        value_stack: vec![],
                                        ip: if cache_hit { end_ip } else { next_ip },
                                        block_has_mutable_ops: parent_has_mutable_ops,
                                    };

                                    self.label_stack.push(new_label_stack);
                                    current_label_stack_idx = self.label_stack.len() - 1;

                                    // Reset tracking for new block
                                    if self.current_block_accessed_globals.is_some() {
                                        self.current_block_accessed_globals =
                                            Some(GlobalAccessTracker::new());
                                    }
                                    if self.current_block_accessed_locals.is_some() {
                                        self.current_block_accessed_locals =
                                            Some(LocalAccessTracker::new());
                                    }
                                }
                                HandlerResult::PopLabelStack { next_ip } => {
                                    // Pop the current label stack when ending a block/loop
                                    if self.label_stack.len() > 1 {
                                        // Determine if the block was immutable based on whether any mutable operations were executed
                                        let block_was_mutable = self.label_stack
                                            [current_label_stack_idx]
                                            .block_has_mutable_ops;

                                        // Update the label's immutability status
                                        self.label_stack[current_label_stack_idx]
                                            .label
                                            .is_immutable = Some(!block_was_mutable);

                                        // Extract values before mutating
                                        let (stack_height, arity, input_stack, start_ip, end_ip) = {
                                            let current_label =
                                                &self.label_stack[current_label_stack_idx].label;
                                            (
                                                current_label.stack_height,
                                                current_label.arity,
                                                current_label.input_stack.clone(),
                                                current_label.start_ip,
                                                current_label.end_ip,
                                            )
                                        };

                                        // Extract the result values from global stack
                                        // For internal blocks, always use value_stack (slot_file.result_slot is for function return)
                                        let result_values = if arity > 0 {
                                            if self.global_value_stack.len() >= arity {
                                                let start_idx =
                                                    self.global_value_stack.len() - arity;
                                                self.global_value_stack[start_idx..].to_vec()
                                            } else {
                                                // Not enough values on stack for the required arity
                                                if self.global_value_stack.len() > stack_height {
                                                    self.global_value_stack[stack_height..].to_vec()
                                                } else {
                                                    Vec::new()
                                                }
                                            }
                                        } else {
                                            Vec::new()
                                        };

                                        // Restore the global stack to the entry state
                                        if arity == 0 && result_values.is_empty() {
                                            // For void blocks (arity=0) with no results, preserve the current stack state
                                        } else {
                                            // Normal case: restore to entry state and add result values
                                            let correct_stack_height = if arity > 0
                                                && self.global_value_stack.len() >= arity
                                            {
                                                self.global_value_stack.len() - arity
                                            } else {
                                                stack_height
                                            };

                                            let current_len = self.global_value_stack.len();
                                            let actual_height =
                                                correct_stack_height.min(current_len);
                                            self.global_value_stack
                                                .splice(actual_height.., result_values);
                                        }

                                        // Only cache nested blocks, not function level (index 0)
                                        if current_label_stack_idx > 0 {
                                            // Update immutability status if not yet determined
                                            let block_is_mutable = self.label_stack
                                                [current_label_stack_idx]
                                                .block_has_mutable_ops;
                                            let block_start_height = stack_height;

                                            if self.label_stack[current_label_stack_idx]
                                                .label
                                                .is_immutable
                                                .is_none()
                                            {
                                                self.label_stack[current_label_stack_idx]
                                                    .label
                                                    .is_immutable = Some(!block_is_mutable);
                                            }

                                            // Check if block is immutable before caching
                                            let is_immutable = self.label_stack
                                                [current_label_stack_idx]
                                                .label
                                                .is_immutable;
                                            if is_immutable == Some(true) {
                                                // Save block results from block start height
                                                // For blocks (not functions), we always save from block_start_height
                                                let final_stack_state = if block_start_height
                                                    <= self.global_value_stack.len()
                                                {
                                                    self.global_value_stack[block_start_height..]
                                                        .to_vec()
                                                } else {
                                                    Vec::new()
                                                };
                                                if get_block_cache(
                                                    start_ip,
                                                    end_ip,
                                                    &input_stack,
                                                    &self.frame.locals,
                                                    &self.frame.local_versions,
                                                )
                                                .is_none()
                                                {
                                                    // Get tracked globals before storing
                                                    let tracked_globals = self
                                                        .current_block_accessed_globals
                                                        .take()
                                                        .unwrap_or_else(GlobalAccessTracker::new);
                                                    // Get tracked locals before storing
                                                    let tracked_locals = self
                                                        .current_block_accessed_locals
                                                        .take()
                                                        .unwrap_or_else(LocalAccessTracker::new);
                                                    store_block_cache(
                                                        start_ip,
                                                        end_ip,
                                                        &input_stack,
                                                        &self.frame.locals,
                                                        final_stack_state,
                                                        tracked_globals,
                                                        tracked_locals,
                                                    );
                                                }
                                            }
                                        }

                                        self.label_stack.pop();
                                        current_label_stack_idx = self.label_stack.len() - 1;
                                        self.label_stack[current_label_stack_idx].ip = next_ip;

                                        // Write back results to slot_file if specified (for End instructions)
                                        if !stack_to_slots.is_empty() {
                                            if let Some(ref mut slot_file) = self.frame.slot_file {
                                                let stack_len = self.global_value_stack.len();
                                                let slot_count = stack_to_slots.len();
                                                // Peek values from value_stack and write to slots
                                                for (i, slot) in stack_to_slots.iter().enumerate() {
                                                    let val = &self.global_value_stack
                                                        [stack_len - slot_count + i];
                                                    match slot {
                                                        crate::execution::slots::Slot::I32(idx) => {
                                                            slot_file.set_i32(
                                                                *idx,
                                                                val.to_i32().unwrap_or(0),
                                                            );
                                                        }
                                                        crate::execution::slots::Slot::I64(idx) => {
                                                            slot_file.set_i64(
                                                                *idx,
                                                                val.to_i64().unwrap_or(0),
                                                            );
                                                        }
                                                        crate::execution::slots::Slot::F32(idx) => {
                                                            slot_file.set_f32(
                                                                *idx,
                                                                val.to_f32().unwrap_or(0.0),
                                                            );
                                                        }
                                                        crate::execution::slots::Slot::F64(idx) => {
                                                            slot_file.set_f64(
                                                                *idx,
                                                                val.to_f64().unwrap_or(0.0),
                                                            );
                                                        }
                                                        crate::execution::slots::Slot::Ref(idx) => {
                                                            if let Val::Ref(r) = val {
                                                                slot_file.set_ref(*idx, r.clone());
                                                            }
                                                        }
                                                        crate::execution::slots::Slot::V128(_) => {}
                                                    }
                                                }
                                                // Remove values from value_stack (slots_to_stack will handle pushing when needed)
                                                self.global_value_stack
                                                    .truncate(stack_len - slot_count);
                                            }
                                        }
                                    } else {
                                        let current_label =
                                            &self.label_stack[current_label_stack_idx].label;
                                        let function_arity = current_label.arity;

                                        // Extract the function result values from value_stack or slot_file
                                        if function_arity > 0 {
                                            if self.global_value_stack.len() >= function_arity {
                                                // Use value_stack (may contain result from br jump)
                                                let start_idx =
                                                    self.global_value_stack.len() - function_arity;
                                                let result_values =
                                                    self.global_value_stack[start_idx..].to_vec();
                                                self.global_value_stack.clear();
                                                self.global_value_stack.extend(result_values);
                                            } else if let (
                                                Some(ref slot_file),
                                                Some(ref result_slot),
                                            ) =
                                                (&self.frame.slot_file, &self.frame.result_slot)
                                            {
                                                // Slot mode fallback: read result from specified slot
                                                let result_val = slot_file.get_val(result_slot);
                                                self.global_value_stack.clear();
                                                self.global_value_stack.push(result_val);
                                            } else {
                                                return Err(RuntimeError::ValueStackUnderflow);
                                            }
                                        } else {
                                            // If arity is 0, keep the stack as is
                                        };
                                        break;
                                    }
                                }
                            }
                        }
                    }
                } // End of Legacy match arm
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
    pub start_ip: usize,
    pub end_ip: usize,
    pub input_stack: Vec<Val>, // Cache input stack state at block start
    pub is_immutable: Option<bool>, // None = not determined yet, Some(true/false) = determined
}

#[derive(Clone, Debug)]
pub struct LabelStack {
    pub label: Label,
    pub processed_instrs: Rc<Vec<ProcessedInstr>>,
    pub value_stack: Vec<Val>,
    pub ip: usize,
    pub block_has_mutable_ops: bool, // Track if this block executed mutable operations
}

impl Serialize for LabelStack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("LabelStack", 4)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("processed_instrs", self.processed_instrs.as_ref())?;
        state.serialize_field("value_stack", &self.value_stack)?;
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
            value_stack: Vec<Val>,
            ip: usize,
        }

        let data = LabelStackData::deserialize(deserializer)?;
        Ok(LabelStack {
            label: data.label,
            processed_instrs: Rc::new(data.processed_instrs),
            value_stack: data.value_stack,
            ip: data.ip,
            block_has_mutable_ops: false, // Default to false when deserializing
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

macro_rules! binop {
    ($ctx:ident, $operand:ident, $operand_type:ident, $op_trait:ident, $op_method:ident, $result_type:ident) => {{
        match $operand {
            Operand::Optimized(OptimizedOperand::Double {
                first,
                second,
                memarg: None,
                store_target,
            }) => {
                // Get values from sources
                let lhs_val = get_value_from_source($ctx, first)?;
                let rhs_val = get_value_from_source($ctx, second)?;

                // Perform operation
                let result = match (lhs_val, rhs_val) {
                    (Val::Num(Num::$operand_type(a)), Val::Num(Num::$operand_type(b))) => {
                        Val::Num(Num::$result_type($op_trait::$op_method(a, b)))
                    }
                    _ => return Err(RuntimeError::TypeMismatch),
                };

                // Use store_result helper function
                store_result($ctx, result, store_target)?;
                Ok(HandlerResult::Continue($ctx.ip + 1))
            }
            _ => {
                // Normal stack-based operation
                if $ctx.value_stack.len() < 2 {
                    return Err(RuntimeError::ValueStackUnderflow);
                }
                let rhs_val = $ctx
                    .value_stack
                    .pop()
                    .ok_or(RuntimeError::ValueStackUnderflow)?;
                let lhs_val = $ctx
                    .value_stack
                    .pop()
                    .ok_or(RuntimeError::ValueStackUnderflow)?;
                let (lhs, rhs) = match (lhs_val, rhs_val) {
                    (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l, r),
                    _ => return Err(RuntimeError::TypeMismatch),
                };
                $ctx.value_stack
                    .push(Val::Num(Num::$result_type($op_trait::$op_method(lhs, rhs))));
                Ok(HandlerResult::Continue($ctx.ip + 1))
            }
        }
    }};
    ($ctx:ident, $operand:ident, $operand_type:ident, $op_trait:ident, $op_method:ident) => {
        binop!(
            $ctx,
            $operand,
            $operand_type,
            $op_trait,
            $op_method,
            $operand_type
        )
    };
}

macro_rules! binop_wrapping {
    ($ctx:ident, $operand:ident, $operand_type:ident, $op_method:ident, $result_type:ident) => {{
        match $operand {
            Operand::Optimized(OptimizedOperand::Double {
                first,
                second,
                memarg: None,
                store_target,
            }) => {
                // Get values from sources
                let lhs_val = get_value_from_source($ctx, first)?;
                let rhs_val = get_value_from_source($ctx, second)?;

                // Perform operation
                let result = match (lhs_val, rhs_val) {
                    (Val::Num(Num::$operand_type(a)), Val::Num(Num::$operand_type(b))) => {
                        Val::Num(Num::$result_type(a.$op_method(b)))
                    }
                    _ => return Err(RuntimeError::TypeMismatch),
                };

                // Use store_result helper function
                store_result($ctx, result, store_target)?;
                Ok(HandlerResult::Continue($ctx.ip + 1))
            }
            _ => {
                // Normal stack-based operation
                if $ctx.value_stack.len() < 2 {
                    return Err(RuntimeError::ValueStackUnderflow);
                }
                let rhs_val = $ctx
                    .value_stack
                    .pop()
                    .ok_or(RuntimeError::ValueStackUnderflow)?;
                let lhs_val = $ctx
                    .value_stack
                    .pop()
                    .ok_or(RuntimeError::ValueStackUnderflow)?;
                let (lhs, rhs) = match (lhs_val, rhs_val) {
                    (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l, r),
                    _ => return Err(RuntimeError::TypeMismatch),
                };
                let result_val = Val::Num(Num::$result_type(lhs.$op_method(rhs)));
                $ctx.value_stack.push(result_val);
                Ok(HandlerResult::Continue($ctx.ip + 1))
            }
        }
    }};
    ($ctx:ident, $operand:ident, $operand_type:ident, $op_method:ident) => {
        binop_wrapping!($ctx, $operand, $operand_type, $op_method, $operand_type)
    };
}

// Unary operation macro
macro_rules! unaryop {
    ($ctx:ident, $operand:ident, $type:ident, $val_method:ident, $op:expr) => {{
        match $operand {
            Operand::Optimized(OptimizedOperand::Single {
                value,
                memarg: None,
                store_target,
            }) => {
                // Get value from source
                let val = get_value_from_source($ctx, value)?;
                let x = val.$val_method()?;
                let result_val = Val::Num(Num::$type($op(x)));

                // Store to target if specified, otherwise push to stack
                store_result($ctx, result_val, store_target)?;
                Ok(HandlerResult::Continue($ctx.ip + 1))
            }
            _ => {
                // Normal stack-based operation
                let val = $ctx
                    .value_stack
                    .pop()
                    .ok_or(RuntimeError::ValueStackUnderflow)?;
                let x = val.$val_method()?;
                let result = $op(x);
                $ctx.value_stack.push(Val::Num(Num::$type(result)));
                Ok(HandlerResult::Continue($ctx.ip + 1))
            }
        }
    }};
}

macro_rules! shiftop {
    ($ctx:ident, $operand:ident, $type:ident, $val_method:ident, $shift_method:ident) => {{
        match $operand {
            Operand::Optimized(OptimizedOperand::Double {
                first,
                second,
                memarg: None,
                store_target,
            }) => {
                // Get values from sources
                let lhs_val = get_value_from_source($ctx, first)?;
                let rhs_val = get_value_from_source($ctx, second)?;

                let lhs = lhs_val.$val_method()?;
                let rhs = rhs_val.$val_method()?;
                let result_val = Val::Num(Num::$type(lhs.$shift_method(rhs as u32)));

                // Store to target if specified, otherwise push to stack
                store_result($ctx, result_val, store_target)?;
                Ok(HandlerResult::Continue($ctx.ip + 1))
            }
            _ => {
                // Normal stack-based operation
                let rhs_val = $ctx
                    .value_stack
                    .pop()
                    .ok_or(RuntimeError::ValueStackUnderflow)?;
                let lhs_val = $ctx
                    .value_stack
                    .pop()
                    .ok_or(RuntimeError::ValueStackUnderflow)?;
                let rhs = rhs_val.$val_method()?;
                let lhs = lhs_val.$val_method()?;
                let result = lhs.$shift_method(rhs as u32);
                $ctx.value_stack.push(Val::Num(Num::$type(result)));
                Ok(HandlerResult::Continue($ctx.ip + 1))
            }
        }
    }};
}

macro_rules! shiftop_unsigned {
    ($ctx:ident, $operand:ident, $type:ident, $val_method:ident, $unsigned_type:ident, $signed_type:ident) => {{
        match $operand {
            Operand::Optimized(OptimizedOperand::Double {
                first,
                second,
                memarg: None,
                store_target,
            }) => {
                // Get values from sources
                let lhs_val = get_value_from_source($ctx, first)?;
                let rhs_val = get_value_from_source($ctx, second)?;

                let lhs = lhs_val.$val_method()? as $unsigned_type;
                let rhs = rhs_val.$val_method()? as u32;
                let result = lhs.wrapping_shr(rhs);
                let result_val = Val::Num(Num::$type(result as $signed_type));

                // Store to target if specified, otherwise push to stack
                store_result($ctx, result_val, store_target)?;
                Ok(HandlerResult::Continue($ctx.ip + 1))
            }
            _ => {
                // Normal stack-based operation
                let rhs_val = $ctx
                    .value_stack
                    .pop()
                    .ok_or(RuntimeError::ValueStackUnderflow)?;
                let lhs_val = $ctx
                    .value_stack
                    .pop()
                    .ok_or(RuntimeError::ValueStackUnderflow)?;
                let rhs = rhs_val.$val_method()? as u32;
                let lhs = lhs_val.$val_method()? as $unsigned_type;
                let result = lhs.wrapping_shr(rhs);
                $ctx.value_stack
                    .push(Val::Num(Num::$type(result as $signed_type)));
                Ok(HandlerResult::Continue($ctx.ip + 1))
            }
        }
    }};
}

// Direct comparison macro (for signed comparisons and equality)
macro_rules! cmpop {
    ($ctx:ident, $operand:ident, $operand_type:ident, $op:tt) => {{
        match $operand {
            Operand::Optimized(OptimizedOperand::Double { first, second, memarg: None, store_target }) => {
                // Get values from sources
                let lhs_val = get_value_from_source($ctx, first)?;
                let rhs_val = get_value_from_source($ctx, second)?;

                let (lhs, rhs) = match (lhs_val, rhs_val) {
                    (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l, r),
                    _ => return Err(RuntimeError::TypeMismatch),
                };
                let result = Val::Num(Num::I32((lhs $op rhs) as i32));

                // Use store_result helper function
                store_result($ctx, result, store_target)?;
                Ok(HandlerResult::Continue($ctx.ip + 1))
            }
            _ => {
                // Normal stack-based operation
                let rhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
                let lhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
                let (lhs, rhs) = match (lhs_val, rhs_val) {
                     (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l, r),
                     _ => return Err(RuntimeError::TypeMismatch),
                };
                $ctx.value_stack.push(Val::Num(Num::I32((lhs $op rhs) as i32)));
                Ok(HandlerResult::Continue($ctx.ip + 1))
            }
        }
    }};
}

// Comparison macro with casting (for unsigned comparisons)
macro_rules! cmpop_cast {
    ($ctx:ident, $operand:ident, $operand_type:ident, $op:tt, $cast_type:ty) => {{
        match $operand {
            Operand::Optimized(OptimizedOperand::Double { first, second, memarg: None, store_target }) => {
                // Get values from sources
                let lhs_val = get_value_from_source($ctx, first)?;
                let rhs_val = get_value_from_source($ctx, second)?;

                let (lhs, rhs) = match (lhs_val, rhs_val) {
                    (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l as $cast_type, r as $cast_type),
                    _ => return Err(RuntimeError::TypeMismatch),
                };
                let result = Val::Num(Num::I32((lhs $op rhs) as i32));

                // Use store_result helper function
                store_result($ctx, result, store_target)?;
                Ok(HandlerResult::Continue($ctx.ip + 1))
            }
            _ => {
                // Normal stack-based operation
                let rhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
                let lhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
                let (lhs, rhs) = match (lhs_val, rhs_val) {
                     (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l as $cast_type, r as $cast_type),
                     _ => return Err(RuntimeError::TypeMismatch),
                };
                $ctx.value_stack.push(Val::Num(Num::I32((lhs $op rhs) as i32)));
                Ok(HandlerResult::Continue($ctx.ip + 1))
            }
        }
    }};
}

fn handle_unreachable(
    _ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    Err(RuntimeError::Unreachable)
}

fn handle_nop(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_block(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::Block {
        arity,
        param_count,
        is_loop,
        start_ip,
        end_ip,
    } = operand
    {
        // Create a new label for this block
        let current_stack_height = ctx.value_stack.len();
        let label = Label {
            locals_num: *param_count,
            arity: *arity,
            is_loop: *is_loop,
            stack_height: current_stack_height,
            return_ip: *end_ip + 1, // IP to return to after this block (after the end instruction)
            start_ip: *start_ip,
            end_ip: *end_ip,
            input_stack: ctx.value_stack[..current_stack_height].to_vec(),
            is_immutable: None, // Will be determined during execution
        };

        Ok(HandlerResult::PushLabelStack {
            label,
            next_ip: ctx.ip + 1,
            start_ip: *start_ip,
            end_ip: *end_ip,
        })
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_loop(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::Block {
        arity,
        param_count,
        is_loop,
        start_ip,
        end_ip,
    } = operand
    {
        let current_stack_height = ctx.value_stack.len();
        let label = Label {
            locals_num: *param_count,
            arity: *arity,
            is_loop: *is_loop,
            stack_height: current_stack_height,
            return_ip: *end_ip + 1, // IP to return to after this loop (after the end instruction)
            start_ip: *start_ip,
            end_ip: *end_ip,
            input_stack: ctx.value_stack[..current_stack_height].to_vec(),
            is_immutable: None, // Will be determined during execution
        };

        // Signal to the main execution loop to create a new label stack
        Ok(HandlerResult::PushLabelStack {
            label,
            next_ip: ctx.ip + 1,
            start_ip: *start_ip,
            end_ip: *end_ip,
        })
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_if(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let cond_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let cond = cond_val.to_i32()?;

    if let &Operand::LabelIdx {
        target_ip,
        arity,
        original_wasm_depth: _,
        is_loop: _,
    } = operand
    {
        if target_ip == usize::MAX {
            return Err(RuntimeError::ExecutionFailed(
                "Branch fixup not done for If",
            ));
        }

        let current_stack_height = ctx.value_stack.len();
        let label = Label {
            locals_num: 0, // If blocks don't have parameters
            arity,
            is_loop: false,
            stack_height: current_stack_height,
            return_ip: target_ip, // IP after the if block (either else or end)
            start_ip: ctx.ip,     // Current IP as start
            end_ip: target_ip,    // Target IP as end
            input_stack: ctx.value_stack[..current_stack_height].to_vec(),
            is_immutable: None, // Will be determined during execution
        };

        // WebAssembly if: 0 = false (else/skip), non-zero = true (then)
        let next_ip = if cond != 0 { ctx.ip + 1 } else { target_ip };

        Ok(HandlerResult::PushLabelStack {
            label,
            next_ip,
            start_ip: ctx.ip + 1,
            end_ip: target_ip,
        })
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_else(
    _ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::LabelIdx {
        target_ip,
        arity: _,
        original_wasm_depth: _,
        is_loop: _,
    } = operand
    {
        if target_ip == usize::MAX {
            return Err(RuntimeError::ExecutionFailed(
                "Branch fixup not done for Else",
            ));
        }
        Ok(HandlerResult::Continue(target_ip))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_end(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    Ok(HandlerResult::PopLabelStack {
        next_ip: ctx.ip + 1,
    })
}

fn handle_br(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::LabelIdx {
        target_ip,
        arity,
        original_wasm_depth,
        is_loop: _,
    } = operand
    {
        if *target_ip == usize::MAX {
            return Err(RuntimeError::ExecutionFailed(
                "Branch fixup not done for Br",
            ));
        }

        let values_to_push = ctx.pop_n_values(*arity)?;

        Ok(HandlerResult::Branch {
            target_ip: *target_ip,
            values_to_push,
            branch_depth: *original_wasm_depth,
        })
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_br_if(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let cond_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let cond = cond_val.to_i32()?;

    if cond != 0 {
        if let Operand::LabelIdx {
            target_ip,
            arity,
            original_wasm_depth,
            is_loop: _,
        } = operand
        {
            if *target_ip == usize::MAX {
                return Err(RuntimeError::ExecutionFailed(
                    "Branch fixup not done for BrIf",
                ));
            }

            let values_to_push = ctx.pop_n_values(*arity)?;

            Ok(HandlerResult::Branch {
                target_ip: *target_ip,
                values_to_push,
                branch_depth: *original_wasm_depth,
            })
        } else {
            Err(RuntimeError::InvalidOperand)
        }
    } else {
        Ok(HandlerResult::Continue(ctx.ip + 1))
    }
}

fn handle_call(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::FuncIdx(func_idx) = operand {
        let instance = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        let func_addr = instance.func_addrs[func_idx.0 as usize].clone();
        ctx.block_has_mutable_op = true;
        Ok(HandlerResult::Invoke(func_addr))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_const(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::I32(val) => {
            ctx.value_stack.push(Val::Num(Num::I32(*val)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => Err(RuntimeError::InvalidOperand),
    }
}

fn handle_i64_const(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::I64(val) = operand {
        ctx.value_stack.push(Val::Num(Num::I64(val)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f32_const(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::F32(val) = operand {
        ctx.value_stack.push(Val::Num(Num::F32(val)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f64_const(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::F64(val) = operand {
        ctx.value_stack.push(Val::Num(Num::F64(val)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

// --- Comparison Handlers (Operand not used, signature change only) ---
fn handle_i32_eqz(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val = get_value_from_source(ctx, value)?;
            let x = val.to_i32()?;
            let result = (x == 0) as i32;
            let result_val = Val::Num(Num::I32(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val_opt = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let val = val_opt.to_i32()?;
            ctx.value_stack.push(Val::Num(Num::I32((val == 0) as i32)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}
fn handle_i32_eq(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, I32, ==)
}
fn handle_i32_ne(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, I32, !=)
}
fn handle_i32_lt_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, I32, <)
}
fn handle_i32_lt_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop_cast!(ctx, operand, I32, <, u32)
}
fn handle_i32_gt_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, I32, >)
}
fn handle_i32_gt_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop_cast!(ctx, operand, I32, >, u32)
}
fn handle_i32_le_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, I32, <=)
}
fn handle_i32_le_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop_cast!(ctx, operand, I32, <=, u32)
}
fn handle_i32_ge_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, I32, >=)
}
fn handle_i32_ge_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop_cast!(ctx, operand, I32, >=, u32)
}

fn handle_i64_eqz(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val = get_value_from_source(ctx, value)?;
            let x = val.to_i64()?;
            let result = (x == 0) as i32;
            let result_val = Val::Num(Num::I32(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val_opt = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let val = val_opt.to_i64()?;
            ctx.value_stack.push(Val::Num(Num::I32((val == 0) as i32)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}
fn handle_i64_eq(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, I64, ==)
}
fn handle_i64_ne(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, I64, !=)
}
fn handle_i64_lt_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, I64, <)
}
fn handle_i64_lt_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop_cast!(ctx, operand, I64, <, u64)
}
fn handle_i64_gt_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, I64, >)
}
fn handle_i64_gt_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop_cast!(ctx, operand, I64, >, u64)
}
fn handle_i64_le_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, I64, <=)
}
fn handle_i64_le_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop_cast!(ctx, operand, I64, <=, u64)
}
fn handle_i64_ge_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, I64, >=)
}
fn handle_i64_ge_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop_cast!(ctx, operand, I64, >=, u64)
}

fn handle_f32_eq(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, F32, ==)
}
fn handle_f32_ne(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, F32, !=)
}
fn handle_f32_lt(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, F32, <)
}
fn handle_f32_gt(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, F32, >)
}
fn handle_f32_le(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, F32, <=)
}
fn handle_f32_ge(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, F32, >=)
}

fn handle_f64_eq(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, F64, ==)
}
fn handle_f64_ne(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, F64, !=)
}
fn handle_f64_lt(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, F64, <)
}
fn handle_f64_gt(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, F64, >)
}
fn handle_f64_le(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, F64, <=)
}
fn handle_f64_ge(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, operand, F64, >=)
}

// Helper macro to resolve memory address from operand
macro_rules! resolve_mem_addr {
    ($ctx:ident, $operand:ident) => {{
        match $operand {
            Operand::MemArg(arg) => {
                let ptr_val = $ctx
                    .value_stack
                    .pop()
                    .ok_or(RuntimeError::ValueStackUnderflow)?;
                let ptr = ptr_val.to_i32()?;
                (arg, ptr)
            }
            Operand::Optimized(OptimizedOperand::Single {
                value,
                memarg: Some(m),
                store_target: _,
            }) => {
                let ptr = match value {
                    Some(src) => {
                        let val = get_value_from_source($ctx, &Some(src.clone()))?;
                        val.to_i32()?
                    }
                    None => {
                        let ptr_val = $ctx
                            .value_stack
                            .pop()
                            .ok_or(RuntimeError::ValueStackUnderflow)?;
                        ptr_val.to_i32()?
                    }
                };
                (m, ptr)
            }
            _ => {
                return Err(RuntimeError::InvalidOperand);
            }
        }
    }};
}

// Helper macro to resolve memory address and value for store operations
macro_rules! resolve_store_args {
    ($ctx:ident, $operand:ident) => {{
        match $operand {
            Operand::MemArg(arg) => {
                let val = $ctx
                    .value_stack
                    .pop()
                    .ok_or(RuntimeError::ValueStackUnderflow)?;
                let ptr_val = $ctx
                    .value_stack
                    .pop()
                    .ok_or(RuntimeError::ValueStackUnderflow)?;
                let ptr = ptr_val.to_i32()?;
                (arg, val, ptr)
            }
            Operand::Optimized(OptimizedOperand::Double {
                first: addr_source,
                second: value_source,
                memarg: Some(m),
                store_target: _,
            }) => {
                let val = match value_source {
                    Some(src) => get_value_from_source($ctx, &Some(src.clone()))?,
                    None => $ctx
                        .value_stack
                        .pop()
                        .ok_or(RuntimeError::ValueStackUnderflow)?,
                };
                let ptr = match addr_source {
                    Some(src) => {
                        let ptr_val = get_value_from_source($ctx, &Some(src.clone()))?;
                        ptr_val.to_i32()?
                    }
                    None => {
                        let ptr_val = $ctx
                            .value_stack
                            .pop()
                            .ok_or(RuntimeError::ValueStackUnderflow)?;
                        ptr_val.to_i32()?
                    }
                };
                (m, val, ptr)
            }
            _ => return Err(RuntimeError::InvalidOperand),
        }
    }};
}

// Memory load macro for types that don't need conversion (i32, i64, f32, f64)
macro_rules! load_with_source {
    ($ctx:ident, $operand:ident, $type:ty, $val_type:ident) => {{
        let (arg, ptr) = resolve_mem_addr!($ctx, $operand);

        let module_inst = $ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val = mem_addr.load::<$type>(arg, ptr)?;
        $ctx.value_stack.push(Val::Num(Num::$val_type(val)));
        Ok(HandlerResult::Continue($ctx.ip + 1))
    }};
}

// Memory load macro for types that need conversion (i8/u8/i16/u16 -> i32, i8/u8/i16/u16/i32/u32 -> i64)
macro_rules! load_with_source_convert {
    ($ctx:ident, $operand:ident, $load_type:ty, $result_type:ident) => {{
        let (arg, ptr) = resolve_mem_addr!($ctx, $operand);

        let module_inst = $ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val = mem_addr.load::<$load_type>(arg, ptr)?;
        // Convert to the appropriate result type
        let result = match stringify!($result_type) {
            "I32" => Val::Num(Num::I32(val as i32)),
            "I64" => Val::Num(Num::I64(val as i64)),
            _ => unreachable!(),
        };
        $ctx.value_stack.push(result);
        Ok(HandlerResult::Continue($ctx.ip + 1))
    }};
}

// Memory store macro for types that don't need truncation (i32, i64, f32, f64)
macro_rules! store_with_source {
    ($ctx:ident, $operand:ident, $type:ty, $to_method:ident) => {{
        let (arg, val, ptr) = resolve_store_args!($ctx, $operand);
        let module_inst = $ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_typed = val.$to_method()?;
        mem_addr.store::<$type>(arg, ptr, val_typed)?;
        Ok(HandlerResult::Continue($ctx.ip + 1))
    }};
}

// Memory store macro for types that need truncation (i8/i16/i32 from larger values)
macro_rules! store_with_source_truncate {
    ($ctx:ident, $operand:ident, $store_type:ty, $to_method:ident) => {{
        let (arg, val, ptr) = resolve_store_args!($ctx, $operand);
        let module_inst = $ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_truncated = val.$to_method()? as $store_type;
        mem_addr.store::<$store_type>(arg, ptr, val_truncated)?;
        Ok(HandlerResult::Continue($ctx.ip + 1))
    }};
}

// --- Arithmetic Handlers ---
// Helper function to get value from various sources
fn get_value_from_source(
    ctx: &mut ExecutionContext,
    source: &Option<ValueSource>,
) -> Result<Val, RuntimeError> {
    match source {
        Some(ValueSource::Const(val)) => Ok(val.to_val()),
        Some(ValueSource::Local(idx)) => ctx
            .frame
            .locals
            .get(*idx as usize)
            .cloned()
            .ok_or(RuntimeError::LocalIndexOutOfBounds),
        Some(ValueSource::Global(idx)) => {
            let module_inst = ctx
                .frame
                .module
                .upgrade()
                .ok_or(RuntimeError::ModuleInstanceGone)?;
            if *idx as usize >= module_inst.global_addrs.len() {
                return Err(RuntimeError::GlobalIndexOutOfBounds);
            }
            let global_addr = &module_inst.global_addrs[*idx as usize];
            Ok(global_addr.get())
        }
        Some(ValueSource::Stack) => ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow),
        None => Err(RuntimeError::InvalidOperand),
    }
}

fn handle_i32_clz(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I32, to_i32, |x: i32| x.leading_zeros() as i32)
}
fn handle_i32_ctz(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I32, to_i32, |x: i32| x.trailing_zeros()
        as i32)
}
fn handle_i32_popcnt(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I32, to_i32, |x: i32| x.count_ones() as i32)
}
fn handle_i32_add(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop_wrapping!(ctx, operand, I32, wrapping_add)
}
fn handle_i32_sub(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop_wrapping!(ctx, operand, I32, wrapping_sub)
}
fn handle_i32_mul(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop_wrapping!(ctx, operand, I32, wrapping_mul)
}
fn handle_i32_div_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    // Get operand values based on source
    let (lhs_val, rhs_val, store_target) = match operand {
        Operand::Optimized(OptimizedOperand::Double {
            first,
            second,
            memarg: None,
            store_target,
        }) => {
            let rhs = get_value_from_source(ctx, second)?;
            let lhs = get_value_from_source(ctx, first)?;
            (lhs, rhs, store_target)
        }
        _ => {
            // Fallback to stack-based operation
            let rhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            (lhs, rhs, &None)
        }
    };

    let rhs = rhs_val.to_i32()?;
    let lhs = lhs_val.to_i32()?;

    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }

    if lhs == i32::MIN && rhs == -1 {
        return Err(RuntimeError::IntegerOverflow);
    }

    let result = lhs / rhs;
    let result_val = Val::Num(Num::I32(result));

    // Use store_result helper function
    store_result(ctx, result_val, store_target)?;
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_div_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    // Get operand values based on source
    let (lhs_val, rhs_val, store_target) = match operand {
        Operand::Optimized(OptimizedOperand::Double {
            first,
            second,
            memarg: None,
            store_target,
        }) => {
            let rhs = get_value_from_source(ctx, second)?;
            let lhs = get_value_from_source(ctx, first)?;
            (lhs, rhs, store_target)
        }
        _ => {
            // Fallback to stack-based operation
            let rhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            (lhs, rhs, &None)
        }
    };

    let rhs = rhs_val.to_i32()? as u32;
    let lhs = lhs_val.to_i32()? as u32;

    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }

    let result = (lhs / rhs) as i32;
    let result_val = Val::Num(Num::I32(result));

    // Store to target if specified, otherwise push to stack
    store_result(ctx, result_val, store_target)?;
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_rem_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    // Get operand values based on source
    let (lhs_val, rhs_val, store_target) = match operand {
        Operand::Optimized(OptimizedOperand::Double {
            first,
            second,
            memarg: None,
            store_target,
        }) => {
            let rhs = get_value_from_source(ctx, second)?;
            let lhs = get_value_from_source(ctx, first)?;
            (lhs, rhs, store_target)
        }
        _ => {
            // Fallback to stack-based operation
            let rhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            (lhs, rhs, &None)
        }
    };

    let rhs = rhs_val.to_i32()?;
    let lhs = lhs_val.to_i32()?;

    let result = if lhs == i32::MIN && rhs == -1 {
        0 // WebAssembly仕様: i32::MIN % -1 = 0
    } else {
        lhs % rhs
    };

    let result_val = Val::Num(Num::I32(result));

    // Store to target if specified, otherwise push to stack
    store_result(ctx, result_val, store_target)?;
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_rem_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    // Get operand values based on source
    let (lhs_val, rhs_val, store_target) = match operand {
        Operand::Optimized(OptimizedOperand::Double {
            first,
            second,
            memarg: None,
            store_target,
        }) => {
            let rhs = get_value_from_source(ctx, second)?;
            let lhs = get_value_from_source(ctx, first)?;
            (lhs, rhs, store_target)
        }
        _ => {
            // Fallback to stack-based operation
            let rhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            (lhs, rhs, &None)
        }
    };

    let rhs = rhs_val.to_i32()? as u32;
    let lhs = lhs_val.to_i32()? as u32;

    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }

    let result = (lhs % rhs) as i32;
    let result_val = Val::Num(Num::I32(result));

    // Store to target if specified, otherwise push to stack
    store_result(ctx, result_val, store_target)?;
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_and(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, operand, I32, BitAnd, bitand)
}
fn handle_i32_or(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, operand, I32, BitOr, bitor)
}
fn handle_i32_xor(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, operand, I32, BitXor, bitxor)
}
fn handle_i32_shl(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    shiftop!(ctx, operand, I32, to_i32, wrapping_shl)
}
fn handle_i32_shr_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    shiftop!(ctx, operand, I32, to_i32, wrapping_shr)
}
fn handle_i32_shr_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    shiftop_unsigned!(ctx, operand, I32, to_i32, u32, i32)
}
fn handle_i32_rotl(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    shiftop!(ctx, operand, I32, to_i32, rotate_left)
}
fn handle_i32_rotr(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    shiftop!(ctx, operand, I32, to_i32, rotate_right)
}

fn handle_i64_clz(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I64, to_i64, |x: i64| x.leading_zeros() as i64)
}
fn handle_i64_ctz(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I64, to_i64, |x: i64| x.trailing_zeros()
        as i64)
}
fn handle_i64_popcnt(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I64, to_i64, |x: i64| x.count_ones() as i64)
}
fn handle_i64_add(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop_wrapping!(ctx, operand, I64, wrapping_add)
}
fn handle_i64_sub(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop_wrapping!(ctx, operand, I64, wrapping_sub)
}
fn handle_i64_mul(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop_wrapping!(ctx, operand, I64, wrapping_mul)
}
fn handle_i64_div_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    // Get operand values based on source
    let (lhs_val, rhs_val, store_target) = match operand {
        Operand::Optimized(OptimizedOperand::Double {
            first,
            second,
            memarg: None,
            store_target,
        }) => {
            let rhs = get_value_from_source(ctx, second)?;
            let lhs = get_value_from_source(ctx, first)?;
            (lhs, rhs, store_target)
        }
        _ => {
            // Fallback to stack-based operation
            let rhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            (lhs, rhs, &None)
        }
    };

    let rhs = rhs_val.to_i64()?;
    let lhs = lhs_val.to_i64()?;

    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }
    if lhs == i64::MIN && rhs == -1 {
        return Err(RuntimeError::IntegerOverflow);
    }

    let result = lhs / rhs;
    let result_val = Val::Num(Num::I64(result));

    // Store to target if specified, otherwise push to stack
    store_result(ctx, result_val, store_target)?;
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_div_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    // Get operand values based on source
    let (lhs_val, rhs_val, store_target) = match operand {
        Operand::Optimized(OptimizedOperand::Double {
            first,
            second,
            memarg: None,
            store_target,
        }) => {
            let rhs = get_value_from_source(ctx, second)?;
            let lhs = get_value_from_source(ctx, first)?;
            (lhs, rhs, store_target)
        }
        _ => {
            // Fallback to stack-based operation
            let rhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            (lhs, rhs, &None)
        }
    };

    let rhs = rhs_val.to_i64()? as u64;
    let lhs = lhs_val.to_i64()? as u64;

    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }

    let result = (lhs / rhs) as i64;
    let result_val = Val::Num(Num::I64(result));

    // Store to target if specified, otherwise push to stack
    store_result(ctx, result_val, store_target)?;
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_rem_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    // Get operand values based on source
    let (lhs_val, rhs_val, store_target) = match operand {
        Operand::Optimized(OptimizedOperand::Double {
            first,
            second,
            memarg: None,
            store_target,
        }) => {
            let rhs = get_value_from_source(ctx, second)?;
            let lhs = get_value_from_source(ctx, first)?;
            (lhs, rhs, store_target)
        }
        _ => {
            // Fallback to stack-based operation
            let rhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            (lhs, rhs, &None)
        }
    };

    let rhs = rhs_val.to_i64()?;
    let lhs = lhs_val.to_i64()?;

    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }

    let result = if lhs == i64::MIN && rhs == -1 {
        0 // WebAssembly spec: i64::MIN % -1 = 0
    } else {
        lhs % rhs
    };

    let result_val = Val::Num(Num::I64(result));

    // Store to target if specified, otherwise push to stack
    store_result(ctx, result_val, store_target)?;
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_rem_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    // Get operand values based on source
    let (lhs_val, rhs_val, store_target) = match operand {
        Operand::Optimized(OptimizedOperand::Double {
            first,
            second,
            memarg: None,
            store_target,
        }) => {
            let rhs = get_value_from_source(ctx, second)?;
            let lhs = get_value_from_source(ctx, first)?;
            (lhs, rhs, store_target)
        }
        _ => {
            // Fallback to stack-based operation
            let rhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            (lhs, rhs, &None)
        }
    };

    let rhs = rhs_val.to_i64()? as u64;
    let lhs = lhs_val.to_i64()? as u64;

    if rhs == 0 {
        return Err(RuntimeError::ZeroDivideError);
    }

    let result = (lhs % rhs) as i64;
    let result_val = Val::Num(Num::I64(result));

    // Store to target if specified, otherwise push to stack
    store_result(ctx, result_val, store_target)?;
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_and(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, operand, I64, BitAnd, bitand)
}
fn handle_i64_or(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, operand, I64, BitOr, bitor)
}
fn handle_i64_xor(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, operand, I64, BitXor, bitxor)
}
fn handle_i64_shl(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    shiftop!(ctx, operand, I64, to_i64, wrapping_shl)
}
fn handle_i64_shr_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    shiftop!(ctx, operand, I64, to_i64, wrapping_shr)
}
fn handle_i64_shr_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    shiftop_unsigned!(ctx, operand, I64, to_i64, u64, i64)
}
fn handle_i64_rotl(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    shiftop!(ctx, operand, I64, to_i64, rotate_left)
}
fn handle_i64_rotr(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    shiftop!(ctx, operand, I64, to_i64, rotate_right)
}

fn handle_f32_abs(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F32, to_f32, |x: f32| x.abs())
}
fn handle_f32_neg(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F32, to_f32, |x: f32| -x)
}
fn handle_f32_ceil(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F32, to_f32, |x: f32| x.ceil())
}
fn handle_f32_floor(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F32, to_f32, |x: f32| x.floor())
}
fn handle_f32_trunc(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F32, to_f32, |x: f32| x.trunc())
}
fn handle_f32_nearest(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val = get_value_from_source(ctx, value)?;
            let x = val.to_f32()?;

            let result = x.round_ties_even();
            let result_val = Val::Num(Num::F32(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let x = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?
                .to_f32()?;
            let result = x.round_ties_even();

            ctx.value_stack.push(Val::Num(Num::F32(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}
fn handle_f32_sqrt(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F32, to_f32, |x: f32| x.sqrt())
}
fn handle_f32_add(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, operand, F32, Add, add)
}
fn handle_f32_sub(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, operand, F32, Sub, sub)
}
fn handle_f32_mul(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, operand, F32, Mul, mul)
}
fn handle_f32_div(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, operand, F32, Div, div)
}
fn handle_f32_min(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Double {
            first,
            second,
            memarg: None,
            store_target,
        }) => {
            // Get values from sources
            let lhs_val = get_value_from_source(ctx, first)?;
            let rhs_val = get_value_from_source(ctx, second)?;

            let lhs = lhs_val.to_f32()?;
            let rhs = rhs_val.to_f32()?;

            let result = if lhs.is_nan() || rhs.is_nan() {
                f32::NAN
            } else if lhs == 0.0 && rhs == 0.0 && (lhs.is_sign_negative() || rhs.is_sign_negative())
            {
                -0.0 // min(±0, ±0) where at least one is negative
            } else {
                lhs.min(rhs)
            };

            let result_val = Val::Num(Num::F32(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            // Fallback to traditional stack-based operation
            let rhs_val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs_val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let rhs = rhs_val.to_f32()?;
            let lhs = lhs_val.to_f32()?;

            let result = if lhs.is_nan() || rhs.is_nan() {
                f32::NAN
            } else if lhs == 0.0 && rhs == 0.0 && (lhs.is_sign_negative() || rhs.is_sign_negative())
            {
                -0.0 // min(±0, ±0) where at least one is negative
            } else {
                lhs.min(rhs)
            };

            ctx.value_stack.push(Val::Num(Num::F32(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}
fn handle_f32_max(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Double {
            first,
            second,
            memarg: None,
            store_target,
        }) => {
            // Get values from sources
            let lhs_val = get_value_from_source(ctx, first)?;
            let rhs_val = get_value_from_source(ctx, second)?;

            let lhs = lhs_val.to_f32()?;
            let rhs = rhs_val.to_f32()?;

            let result = if lhs.is_nan() || rhs.is_nan() {
                f32::NAN
            } else if lhs == 0.0 && rhs == 0.0 && (lhs.is_sign_positive() || rhs.is_sign_positive())
            {
                0.0 // max(±0, ±0) where at least one is positive
            } else {
                lhs.max(rhs)
            };

            let result_val = Val::Num(Num::F32(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            // Fallback to traditional stack-based operation
            let rhs_val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs_val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let rhs = rhs_val.to_f32()?;
            let lhs = lhs_val.to_f32()?;

            let result = if lhs.is_nan() || rhs.is_nan() {
                f32::NAN
            } else if lhs == 0.0 && rhs == 0.0 && (lhs.is_sign_positive() || rhs.is_sign_positive())
            {
                0.0 // max(±0, ±0) where at least one is positive
            } else {
                lhs.max(rhs)
            };

            ctx.value_stack.push(Val::Num(Num::F32(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}
fn handle_f32_copysign(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop_wrapping!(ctx, operand, F32, copysign)
}

fn handle_f64_abs(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F64, to_f64, |x: f64| x.abs())
}
fn handle_f64_neg(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F64, to_f64, |x: f64| -x)
}
fn handle_f64_ceil(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F64, to_f64, |x: f64| x.ceil())
}
fn handle_f64_floor(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F64, to_f64, |x: f64| x.floor())
}
fn handle_f64_trunc(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F64, to_f64, |x: f64| x.trunc())
}
fn handle_f64_nearest(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val = get_value_from_source(ctx, value)?;
            let x = val.to_f64()?;

            let result = x.round();
            let result_val = Val::Num(Num::F64(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let x_val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let x = x_val.to_f64()?;

            let result = x.round();

            ctx.value_stack.push(Val::Num(Num::F64(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i32_extend8_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I32, to_i32, |x: i32| (x as i8) as i32)
}

fn handle_i32_extend16_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I32, to_i32, |x: i32| (x as i16) as i32)
}

fn handle_i64_extend8_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I64, to_i64, |x: i64| (x as i8) as i64)
}

fn handle_i64_extend16_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I64, to_i64, |x: i64| (x as i16) as i64)
}

fn handle_i64_extend32_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I64, to_i32, |x: i32| x as i64)
}

fn handle_i32_wrap_i64(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I32, to_i64, |x: i64| x as i32)
}

fn handle_i64_extend_i32_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I64, to_i32, |x: i32| (x as u32) as i64)
}

fn handle_f64_promote_f32(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F64, to_f32, |x: f32| x as f64)
}

fn handle_f32_demote_f64(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F32, to_f64, |x: f64| x as f32)
}

fn handle_i32_trunc_f32_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val = get_value_from_source(ctx, value)?;
            let x = val.to_f32()?;

            let result = x.trunc() as i32;
            let result_val = Val::Num(Num::I32(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val_opt = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let val = val_opt.to_f32()?;

            let result = val.trunc() as i32;

            ctx.value_stack.push(Val::Num(Num::I32(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i32_trunc_f32_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val = get_value_from_source(ctx, value)?;
            let x = val.to_f32()?;

            let result = x.trunc() as u32 as i32;
            let result_val = Val::Num(Num::I32(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val_opt = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let val = val_opt.to_f32()?;

            let result = val.trunc() as u32 as i32;

            ctx.value_stack.push(Val::Num(Num::I32(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i32_trunc_f64_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val = get_value_from_source(ctx, value)?;
            let x = val.to_f64()?;

            let result = x.trunc() as i32;
            let result_val = Val::Num(Num::I32(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?
                .to_f64()?;

            let result = val.trunc() as i32;

            ctx.value_stack.push(Val::Num(Num::I32(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i32_trunc_f64_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val = get_value_from_source(ctx, value)?;
            let val_f64 = val.to_f64()?;
            if val_f64.is_nan() {
                return Err(RuntimeError::InvalidConversionToInt);
            }
            let truncated = val_f64.trunc();
            if !(truncated >= 0.0 && truncated < u32::MAX as f64 + 1.0) {
                return Err(RuntimeError::IntegerOverflow);
            }
            let result_val = Val::Num(Num::I32(truncated as u32 as i32));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val_opt = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let val_f64 = val_opt.to_f64()?;
            if val_f64.is_nan() {
                return Err(RuntimeError::InvalidConversionToInt);
            }
            let truncated = val_f64.trunc();
            if !(truncated >= 0.0 && truncated < u32::MAX as f64 + 1.0) {
                return Err(RuntimeError::IntegerOverflow);
            }
            ctx.value_stack
                .push(Val::Num(Num::I32(truncated as u32 as i32)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i64_trunc_f32_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val = get_value_from_source(ctx, value)?;
            let x = val.to_f32()?;

            let result = x.trunc() as i64;
            let result_val = Val::Num(Num::I64(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?
                .to_f32()?;

            let result = val.trunc() as i64;

            ctx.value_stack.push(Val::Num(Num::I64(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i64_trunc_f32_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val_obj = get_value_from_source(ctx, value)?;
            let val = val_obj.to_f32()?;

            // Check for overflow conditions according to WebAssembly spec
            if val.is_nan() || val.is_infinite() || val < 0.0 || val >= (u64::MAX as f32) {
                return Err(RuntimeError::IntegerOverflow);
            }

            let result = val.trunc() as u64 as i64;
            let result_val = Val::Num(Num::I64(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?
                .to_f32()?;

            // Check for overflow conditions according to WebAssembly spec
            if val.is_nan() || val.is_infinite() || val < 0.0 || val >= (u64::MAX as f32) {
                return Err(RuntimeError::IntegerOverflow);
            }

            let result = val.trunc() as u64 as i64;

            ctx.value_stack.push(Val::Num(Num::I64(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i64_trunc_f64_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val_obj = get_value_from_source(ctx, value)?;
            let val = val_obj.to_f64()?;

            let result = val.trunc() as i64;
            let result_val = Val::Num(Num::I64(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?
                .to_f64()?;

            let result = val.trunc() as i64;

            ctx.value_stack.push(Val::Num(Num::I64(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i64_trunc_f64_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val_obj = get_value_from_source(ctx, value)?;
            let val = val_obj.to_f64()?;

            let result = val.trunc() as u64 as i64;
            let result_val = Val::Num(Num::I64(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?
                .to_f64()?;

            let result = val.trunc() as u64 as i64;

            ctx.value_stack.push(Val::Num(Num::I64(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_unimplemented(
    _ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    Err(RuntimeError::UnimplementedInstruction)
}

fn handle_br_table(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    // First pop the index
    let i_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let i = i_val.to_i32()?;

    if let Operand::BrTable { targets, default } = operand {
        let chosen_operand = if let Some(target_operand) = targets.get(i as usize) {
            target_operand
        } else {
            default
        };

        if let Operand::LabelIdx {
            target_ip,
            arity,
            original_wasm_depth,
            is_loop: _,
        } = chosen_operand
        {
            if *target_ip == usize::MAX {
                return Err(RuntimeError::ExecutionFailed(
                    "Branch fixup not done for BrTable target",
                ));
            }

            // Then pop the values needed for the branch target
            let values_to_push = if *arity > 0 {
                ctx.pop_n_values(*arity)?
            } else {
                Vec::new()
            };

            Ok(HandlerResult::Branch {
                target_ip: *target_ip,
                values_to_push,
                branch_depth: *original_wasm_depth,
            })
        } else {
            Err(RuntimeError::InvalidOperand)
        }
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_call_indirect(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::CallIndirect {
        type_idx: expected_type_idx,
        table_idx,
    } = operand
    {
        let i_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let i = i_val.to_i32()?;
        let module_inst = ctx
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
            let expected_type = &module_inst.types[expected_type_idx.0 as usize];

            if actual_type != *expected_type {
                return Err(RuntimeError::IndirectCallTypeMismatch);
            }
            ctx.block_has_mutable_op = true;
            Ok(HandlerResult::Invoke(func_addr.clone()))
        } else {
            // TODO: Distinguish between UninitializedElement and TableOutOfBounds if necessary
            Err(RuntimeError::UninitializedElement)
        }
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_select(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let cond_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let cond = cond_val.to_i32()?;
    let val2 = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val1 = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;

    if cond != 0 {
        ctx.value_stack.push(val1);
    } else {
        ctx.value_stack.push(val2);
    }
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_return(
    _ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    Ok(HandlerResult::Return)
}

fn handle_drop(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let _ = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_local_get(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::LocalIdx(LocalIdx(index_val)) = operand {
        let index = index_val as usize;
        if index >= ctx.frame.locals.len() {
            return Err(RuntimeError::LocalIndexOutOfBounds);
        }

        // Track local access with version
        if let Some(ref mut tracker) = ctx.accessed_locals {
            let version = ctx.frame.local_versions[index];
            tracker.track_access(index as u32, version);
        }

        let val = ctx.frame.locals[index].clone();
        ctx.value_stack.push(val);
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_local_set(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::LocalIdx(LocalIdx(index_val)) = operand {
        let index = index_val as usize;
        if index >= ctx.frame.locals.len() {
            return Err(RuntimeError::LocalIndexOutOfBounds);
        }
        let val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        ctx.frame.locals[index] = val;
        ctx.frame.local_versions[index] += 1;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_local_tee(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::LocalIdx(LocalIdx(index_val)) = operand {
        let index = index_val as usize;
        if index >= ctx.frame.locals.len() {
            return Err(RuntimeError::LocalIndexOutOfBounds);
        }
        let val = ctx
            .value_stack
            .last()
            .ok_or(RuntimeError::ValueStackUnderflow)?
            .clone();

        // Track local access with version before modification
        if let Some(ref mut tracker) = ctx.accessed_locals {
            let version = ctx.frame.local_versions[index];
            tracker.track_access(index as u32, version);
        }

        ctx.frame.locals[index] = val;
        ctx.frame.local_versions[index] += 1;
        ctx.block_has_mutable_op = true;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_global_get(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::GlobalIdx(GlobalIdx(index_val)) = operand {
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        let global_addr = module_inst
            .global_addrs
            .get_by_idx(GlobalIdx(index_val))
            .clone();

        // Track global access
        if let Some(ref mut tracker) = ctx.accessed_globals {
            tracker.track_access(index_val);
        }

        ctx.value_stack.push(global_addr.get());
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_global_set(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::GlobalIdx(GlobalIdx(index_val)) = operand {
        let val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        let global_addr = module_inst
            .global_addrs
            .get_by_idx(GlobalIdx(index_val))
            .clone();
        global_addr.set(val)?;
        // Track global write
        if let Some(ref mut tracker) = ctx.accessed_globals {
            tracker.track_access(index_val);
        }
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    load_with_source!(ctx, operand, i32, I32)
}

fn handle_i64_load(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    load_with_source!(ctx, operand, i64, I64)
}

fn handle_f32_load(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val = mem_addr.load::<f32>(&arg, ptr)?;
        ctx.value_stack.push(Val::Num(Num::F32(val)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f64_load(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    load_with_source!(ctx, operand, f64, F64)
}

fn handle_i32_load8_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    load_with_source_convert!(ctx, operand, i8, I32)
}

fn handle_i32_load8_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    load_with_source_convert!(ctx, operand, u8, I32)
}

fn handle_i32_load16_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    load_with_source_convert!(ctx, operand, i16, I32)
}

fn handle_i32_load16_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    load_with_source_convert!(ctx, operand, u16, I32)
}

fn handle_i64_load8_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    load_with_source_convert!(ctx, operand, i8, I64)
}

fn handle_i64_load8_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    load_with_source_convert!(ctx, operand, u8, I64)
}

fn handle_i64_load16_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    load_with_source_convert!(ctx, operand, i16, I64)
}

fn handle_i64_load16_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    load_with_source_convert!(ctx, operand, u16, I64)
}

fn handle_i64_load32_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    load_with_source_convert!(ctx, operand, i32, I64)
}

fn handle_i64_load32_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    load_with_source_convert!(ctx, operand, u32, I64)
}

fn handle_i32_store(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    store_with_source!(ctx, operand, i32, to_i32)
}

fn handle_i64_store(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    store_with_source!(ctx, operand, i64, to_i64)
}

fn handle_f32_store(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    store_with_source!(ctx, operand, f32, to_f32)
}

fn handle_f64_store(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    store_with_source!(ctx, operand, f64, to_f64)
}

fn handle_i32_store8(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    store_with_source_truncate!(ctx, operand, i8, to_i32)
}

fn handle_i32_store16(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    store_with_source_truncate!(ctx, operand, i16, to_i32)
}

fn handle_i64_store8(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    store_with_source_truncate!(ctx, operand, i8, to_i64)
}

fn handle_i64_store16(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    store_with_source_truncate!(ctx, operand, i16, to_i64)
}

fn handle_i64_store32(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    store_with_source_truncate!(ctx, operand, i32, to_i64)
}

fn handle_f64_sqrt(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F64, to_f64, |x: f64| x.sqrt())
}

fn handle_f64_add(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, operand, F64, Add, add)
}
fn handle_f64_sub(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, operand, F64, Sub, sub)
}
fn handle_f64_mul(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, operand, F64, Mul, mul)
}
fn handle_f64_div(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, operand, F64, Div, div)
}

fn handle_f64_min(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Double {
            first,
            second,
            memarg: None,
            store_target,
        }) => {
            // Get values from sources
            let lhs_val = get_value_from_source(ctx, first)?;
            let rhs_val = get_value_from_source(ctx, second)?;

            let lhs = lhs_val.to_f64()?;
            let rhs = rhs_val.to_f64()?;

            let result = if lhs.is_nan() || rhs.is_nan() {
                f64::NAN
            } else if lhs == 0.0 && rhs == 0.0 && (lhs.is_sign_negative() || rhs.is_sign_negative())
            {
                -0.0 // min(±0, ±0) where at least one is negative
            } else {
                lhs.min(rhs)
            };

            let result_val = Val::Num(Num::F64(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            // Fallback to traditional stack-based operation
            let rhs_val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs_val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let rhs = rhs_val.to_f64()?;
            let lhs = lhs_val.to_f64()?;

            let result = if lhs.is_nan() || rhs.is_nan() {
                f64::NAN
            } else if lhs == 0.0 && rhs == 0.0 && (lhs.is_sign_negative() || rhs.is_sign_negative())
            {
                -0.0 // min(±0, ±0) where at least one is negative
            } else {
                lhs.min(rhs)
            };

            ctx.value_stack.push(Val::Num(Num::F64(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_f64_max(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Double {
            first,
            second,
            memarg: None,
            store_target,
        }) => {
            // Get values from sources
            let lhs_val = get_value_from_source(ctx, first)?;
            let rhs_val = get_value_from_source(ctx, second)?;

            let lhs = lhs_val.to_f64()?;
            let rhs = rhs_val.to_f64()?;

            let result = if lhs.is_nan() || rhs.is_nan() {
                f64::NAN
            } else if lhs == 0.0 && rhs == 0.0 && (lhs.is_sign_positive() || rhs.is_sign_positive())
            {
                0.0 // max(±0, ±0) where at least one is positive
            } else {
                lhs.max(rhs)
            };

            let result_val = Val::Num(Num::F64(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            // Fallback to traditional stack-based operation
            let rhs_val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs_val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?;
            let rhs = rhs_val.to_f64()?;
            let lhs = lhs_val.to_f64()?;

            let result = if lhs.is_nan() || rhs.is_nan() {
                f64::NAN
            } else if lhs == 0.0 && rhs == 0.0 && (lhs.is_sign_positive() || rhs.is_sign_positive())
            {
                0.0 // max(±0, ±0) where at least one is positive
            } else {
                lhs.max(rhs)
            };

            ctx.value_stack.push(Val::Num(Num::F64(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_f64_copysign(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop_wrapping!(ctx, operand, F64, copysign)
}

fn handle_memory_size(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let module_inst = ctx
        .frame
        .module
        .upgrade()
        .ok_or(RuntimeError::ModuleInstanceGone)?;
    if module_inst.mem_addrs.is_empty() {
        return Err(RuntimeError::MemoryNotFound);
    }
    let mem_addr = &module_inst.mem_addrs[0];
    let size = mem_addr.mem_size();
    ctx.value_stack.push(Val::Num(Num::I32(size as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_memory_grow(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let delta_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let delta = delta_val.to_i32()?;
    let module_inst = ctx
        .frame
        .module
        .upgrade()
        .ok_or(RuntimeError::ModuleInstanceGone)?;
    if module_inst.mem_addrs.is_empty() {
        return Err(RuntimeError::MemoryNotFound);
    }
    let mem_addr = &module_inst.mem_addrs[0];
    let delta_u32: u32 = delta
        .try_into()
        .map_err(|_| RuntimeError::InvalidParameterCount)?;
    let prev_size = mem_addr.mem_grow(
        delta_u32
            .try_into()
            .map_err(|_| RuntimeError::InvalidParameterCount)?,
    );
    ctx.value_stack.push(Val::Num(Num::I32(prev_size as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_memory_copy(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let len_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let src_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let dest_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;

    let len = len_val.to_i32()?;
    let src = src_val.to_i32()?;
    let dest = dest_val.to_i32()?;

    let module_inst = ctx
        .frame
        .module
        .upgrade()
        .ok_or(RuntimeError::ModuleInstanceGone)?;

    if module_inst.mem_addrs.is_empty() {
        return Err(RuntimeError::MemoryNotFound);
    }

    let mem_addr = &module_inst.mem_addrs[0];
    mem_addr.memory_copy(dest, src, len)?;

    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_memory_init(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let len_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let offset_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let dest_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;

    let len = len_val.to_i32()? as usize;
    let offset = offset_val.to_i32()? as usize;
    let dest = dest_val.to_i32()? as usize;

    let data_index = match operand {
        Operand::I32(idx) => *idx as u32,
        _ => return Err(RuntimeError::InvalidOperand),
    };

    let module_inst = ctx
        .frame
        .module
        .upgrade()
        .ok_or(RuntimeError::ModuleInstanceGone)?;

    if module_inst.mem_addrs.is_empty() {
        return Err(RuntimeError::MemoryNotFound);
    }

    // Get data segment
    if data_index as usize >= module_inst.data_addrs.len() {
        return Err(RuntimeError::InvalidDataSegmentIndex);
    }

    let data_addr = &module_inst.data_addrs[data_index as usize];
    let data_bytes = data_addr.get_data();

    let mem_addr = &module_inst.mem_addrs[0];

    if len > 0 {
        let init_data = data_bytes[offset..offset + len].to_vec();
        mem_addr.init(dest, &init_data);
    }

    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_memory_fill(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let size_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let dest_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;

    let size = size_val.to_i32()? as usize;
    let val = val_val.to_i32()? as u8;
    let dest = dest_val.to_i32()? as usize;

    let module_inst = ctx
        .frame
        .module
        .upgrade()
        .ok_or(RuntimeError::ModuleInstanceGone)?;

    if module_inst.mem_addrs.is_empty() {
        return Err(RuntimeError::MemoryNotFound);
    }

    let mem_addr = &module_inst.mem_addrs[0];

    // Fill memory with the specified value using the dedicated memory_fill method
    mem_addr.memory_fill(dest as i32, val, size as i32)?;

    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_ref_null(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    ctx.value_stack.push(Val::Ref(Ref::RefNull));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_ref_is_null(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let ref_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;

    let is_null = match ref_val {
        Val::Ref(Ref::RefNull) => 1,
        Val::Ref(_) => 0,
        _ => return Err(RuntimeError::TypeMismatch),
    };

    ctx.value_stack.push(Val::Num(Num::I32(is_null)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_table_get(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::TableIdx(TableIdx(table_idx)) = operand {
        let index_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let index = index_val.to_i32()? as usize;

        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;

        let table_addr = module_inst
            .table_addrs
            .get(table_idx as usize)
            .ok_or(RuntimeError::InvalidTableIndex)?;

        let ref_val = table_addr.get(index);
        ctx.value_stack.push(ref_val);
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_table_set(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::TableIdx(TableIdx(table_idx)) = operand {
        let ref_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let index_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let index = index_val.to_i32()? as usize;

        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;

        let table_addr = module_inst
            .table_addrs
            .get(table_idx as usize)
            .ok_or(RuntimeError::InvalidTableIndex)?;

        table_addr.set(index, ref_val)?;
        ctx.block_has_mutable_op = true;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_table_fill(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::TableIdx(TableIdx(table_idx)) = operand {
        // Pop arguments from stack: size, value, index (in reverse order)
        let size_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let size = size_val.to_i32()? as usize;

        let fill_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;

        let index_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let index = index_val.to_i32()? as usize;

        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;

        let table_addr = module_inst
            .table_addrs
            .get(table_idx as usize)
            .ok_or(RuntimeError::InvalidTableIndex)?;

        // Fill the table with the specified value for the given range
        for i in 0..size {
            table_addr.set(index + i, fill_val.clone())?;
        }

        ctx.block_has_mutable_op = true;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f32_convert_i32_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F32, to_i32, |x: i32| x as f32)
}

fn handle_f32_convert_i32_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F32, to_i32, |x: i32| (x as u32) as f32)
}

fn handle_f32_convert_i64_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F32, to_i64, |x: i64| x as f32)
}

fn handle_f32_convert_i64_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F32, to_i64, |x: i64| (x as u64) as f32)
}

fn handle_f64_convert_i32_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F64, to_i32, |x: i32| x as f64)
}

fn handle_f64_convert_i32_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F64, to_i32, |x: i32| (x as u32) as f64)
}

fn handle_f64_convert_i64_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F64, to_i64, |x: i64| x as f64)
}

fn handle_f64_convert_i64_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F64, to_i64, |x: i64| (x as u64) as f64)
}

fn handle_i32_reinterpret_f32(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I32, to_f32, |x: f32| unsafe {
        std::mem::transmute::<f32, i32>(x)
    })
}

fn handle_i64_reinterpret_f64(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, I64, to_f64, |x: f64| unsafe {
        std::mem::transmute::<f64, i64>(x)
    })
}

fn handle_f32_reinterpret_i32(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F32, to_i32, |x: i32| unsafe {
        std::mem::transmute::<i32, f32>(x)
    })
}

fn handle_f64_reinterpret_i64(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    unaryop!(ctx, operand, F64, to_i64, |x: i64| unsafe {
        std::mem::transmute::<i64, f64>(x)
    })
}

fn handle_i32_trunc_sat_f32_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val_obj = get_value_from_source(ctx, value)?;
            let val = val_obj.to_f32()?;

            let result = if val.is_nan() {
                0
            } else if val >= i32::MAX as f32 + 1.0 {
                i32::MAX
            } else if val <= i32::MIN as f32 - 1.0 {
                i32::MIN
            } else {
                val as i32
            };

            let result_val = Val::Num(Num::I32(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?
                .to_f32()?;

            let result = if val.is_nan() {
                0
            } else if val >= i32::MAX as f32 + 1.0 {
                i32::MAX
            } else if val <= i32::MIN as f32 - 1.0 {
                i32::MIN
            } else {
                val as i32
            };

            ctx.value_stack.push(Val::Num(Num::I32(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i32_trunc_sat_f32_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val_obj = get_value_from_source(ctx, value)?;
            let val = val_obj.to_f32()?;

            let result = if val.is_nan() || val <= -1.0 {
                0
            } else if val >= u32::MAX as f32 + 1.0 {
                u32::MAX as i32
            } else {
                val as u32 as i32
            };

            let result_val = Val::Num(Num::I32(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?
                .to_f32()?;

            let result = if val.is_nan() || val <= -1.0 {
                0
            } else if val >= u32::MAX as f32 + 1.0 {
                u32::MAX as i32
            } else {
                val as u32 as i32
            };

            ctx.value_stack.push(Val::Num(Num::I32(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i32_trunc_sat_f64_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val_obj = get_value_from_source(ctx, value)?;
            let val = val_obj.to_f64()?;

            let result = if val.is_nan() {
                0
            } else if val >= i32::MAX as f64 + 1.0 {
                i32::MAX
            } else if val <= i32::MIN as f64 - 1.0 {
                i32::MIN
            } else {
                val as i32
            };

            let result_val = Val::Num(Num::I32(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?
                .to_f64()?;

            let result = if val.is_nan() {
                0
            } else if val >= i32::MAX as f64 + 1.0 {
                i32::MAX
            } else if val <= i32::MIN as f64 - 1.0 {
                i32::MIN
            } else {
                val as i32
            };

            ctx.value_stack.push(Val::Num(Num::I32(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i32_trunc_sat_f64_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val_obj = get_value_from_source(ctx, value)?;
            let val = val_obj.to_f64()?;

            let result = if val.is_nan() || val <= -1.0 {
                0
            } else if val >= u32::MAX as f64 + 1.0 {
                u32::MAX as i32
            } else {
                val as u32 as i32
            };

            let result_val = Val::Num(Num::I32(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?
                .to_f64()?;

            let result = if val.is_nan() || val <= -1.0 {
                0
            } else if val >= u32::MAX as f64 + 1.0 {
                u32::MAX as i32
            } else {
                val as u32 as i32
            };

            ctx.value_stack.push(Val::Num(Num::I32(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i64_trunc_sat_f32_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val_obj = get_value_from_source(ctx, value)?;
            let val = val_obj.to_f32()?;

            let result = if val.is_nan() {
                0
            } else if val >= i64::MAX as f32 + 1.0 {
                i64::MAX
            } else if val <= i64::MIN as f32 - 1.0 {
                i64::MIN
            } else {
                val as i64
            };

            let result_val = Val::Num(Num::I64(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?
                .to_f32()?;

            let result = if val.is_nan() {
                0
            } else if val >= i64::MAX as f32 + 1.0 {
                i64::MAX
            } else if val <= i64::MIN as f32 - 1.0 {
                i64::MIN
            } else {
                val as i64
            };

            ctx.value_stack.push(Val::Num(Num::I64(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i64_trunc_sat_f32_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val_obj = get_value_from_source(ctx, value)?;
            let val = val_obj.to_f32()?;

            let result = if val.is_nan() || val <= -1.0 {
                0
            } else if val >= u64::MAX as f32 + 1.0 {
                u64::MAX as i64
            } else {
                val as u64 as i64
            };

            let result_val = Val::Num(Num::I64(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?
                .to_f32()?;

            let result = if val.is_nan() || val <= -1.0 {
                0
            } else if val >= u64::MAX as f32 + 1.0 {
                u64::MAX as i64
            } else {
                val as u64 as i64
            };

            ctx.value_stack.push(Val::Num(Num::I64(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i64_trunc_sat_f64_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val_obj = get_value_from_source(ctx, value)?;
            let val = val_obj.to_f64()?;

            let result = if val.is_nan() {
                0
            } else if val >= i64::MAX as f64 + 1.0 {
                i64::MAX
            } else if val <= i64::MIN as f64 - 1.0 {
                i64::MIN
            } else {
                val as i64
            };

            let result_val = Val::Num(Num::I64(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?
                .to_f64()?;

            let result = if val.is_nan() {
                0
            } else if val >= i64::MAX as f64 + 1.0 {
                i64::MAX
            } else if val <= i64::MIN as f64 - 1.0 {
                i64::MIN
            } else {
                val as i64
            };

            ctx.value_stack.push(Val::Num(Num::I64(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

fn handle_i64_trunc_sat_f64_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::Optimized(OptimizedOperand::Single {
            value,
            memarg: None,
            store_target,
        }) => {
            let val_obj = get_value_from_source(ctx, value)?;
            let val = val_obj.to_f64()?;

            let result = if val.is_nan() || val <= -1.0 {
                0
            } else if val >= u64::MAX as f64 + 1.0 {
                u64::MAX as i64
            } else {
                val as u64 as i64
            };

            let result_val = Val::Num(Num::I64(result));

            // Store to target if specified, otherwise push to stack
            store_result(ctx, result_val, store_target)?;
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => {
            let val = ctx
                .value_stack
                .pop()
                .ok_or(RuntimeError::ValueStackUnderflow)?
                .to_f64()?;

            let result = if val.is_nan() || val <= -1.0 {
                0
            } else if val >= u64::MAX as f64 + 1.0 {
                u64::MAX as i64
            } else {
                val as u64 as i64
            };

            ctx.value_stack.push(Val::Num(Num::I64(result)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    }
}

lazy_static! {
    static ref HANDLER_TABLE: Vec<HandlerFn> = {
        let mut table: Vec<HandlerFn> = vec![handle_unimplemented; MAX_HANDLER_INDEX];
        table[HANDLER_IDX_UNREACHABLE] = handle_unreachable;
        table[HANDLER_IDX_NOP] = handle_nop;
        table[HANDLER_IDX_BLOCK] = handle_block;
        table[HANDLER_IDX_LOOP] = handle_loop;
        table[HANDLER_IDX_IF] = handle_if;
        table[HANDLER_IDX_ELSE] = handle_else;
        table[HANDLER_IDX_END] = handle_end;
        table[HANDLER_IDX_BR] = handle_br;
        table[HANDLER_IDX_BR_IF] = handle_br_if;
        table[HANDLER_IDX_BR_TABLE] = handle_br_table;
        table[HANDLER_IDX_RETURN] = handle_return;
        table[HANDLER_IDX_CALL] = handle_call;
        table[HANDLER_IDX_CALL_INDIRECT] = handle_call_indirect;
        table[HANDLER_IDX_DROP] = handle_drop;
        table[HANDLER_IDX_SELECT] = handle_select;
        table[HANDLER_IDX_LOCAL_GET] = handle_local_get;
        table[HANDLER_IDX_LOCAL_SET] = handle_local_set;
        table[HANDLER_IDX_LOCAL_TEE] = handle_local_tee;
        table[HANDLER_IDX_GLOBAL_GET] = handle_global_get;
        table[HANDLER_IDX_GLOBAL_SET] = handle_global_set;
        table[HANDLER_IDX_I32_LOAD] = handle_i32_load;
        table[HANDLER_IDX_I64_LOAD] = handle_i64_load;
        table[HANDLER_IDX_F32_LOAD] = handle_f32_load;
        table[HANDLER_IDX_F64_LOAD] = handle_f64_load;
        table[HANDLER_IDX_I32_LOAD8_S] = handle_i32_load8_s;
        table[HANDLER_IDX_I32_LOAD8_U] = handle_i32_load8_u;
        table[HANDLER_IDX_I32_LOAD16_S] = handle_i32_load16_s;
        table[HANDLER_IDX_I32_LOAD16_U] = handle_i32_load16_u;
        table[HANDLER_IDX_I64_LOAD8_S] = handle_i64_load8_s;
        table[HANDLER_IDX_I64_LOAD8_U] = handle_i64_load8_u;
        table[HANDLER_IDX_I64_LOAD16_S] = handle_i64_load16_s;
        table[HANDLER_IDX_I64_LOAD16_U] = handle_i64_load16_u;
        table[HANDLER_IDX_I64_LOAD32_S] = handle_i64_load32_s;
        table[HANDLER_IDX_I64_LOAD32_U] = handle_i64_load32_u;
        table[HANDLER_IDX_I32_STORE] = handle_i32_store;
        table[HANDLER_IDX_I64_STORE] = handle_i64_store;
        table[HANDLER_IDX_F32_STORE] = handle_f32_store;
        table[HANDLER_IDX_F64_STORE] = handle_f64_store;
        table[HANDLER_IDX_I32_STORE8] = handle_i32_store8;
        table[HANDLER_IDX_I32_STORE16] = handle_i32_store16;
        table[HANDLER_IDX_I64_STORE8] = handle_i64_store8;
        table[HANDLER_IDX_I64_STORE16] = handle_i64_store16;
        table[HANDLER_IDX_I64_STORE32] = handle_i64_store32;
        table[HANDLER_IDX_MEMORY_SIZE] = handle_memory_size;
        table[HANDLER_IDX_MEMORY_GROW] = handle_memory_grow;
        table[HANDLER_IDX_MEMORY_COPY] = handle_memory_copy;
        table[HANDLER_IDX_MEMORY_INIT] = handle_memory_init;
        table[HANDLER_IDX_MEMORY_FILL] = handle_memory_fill;
        table[HANDLER_IDX_REF_NULL] = handle_ref_null;
        table[HANDLER_IDX_REF_IS_NULL] = handle_ref_is_null;
        table[HANDLER_IDX_TABLE_GET] = handle_table_get;
        table[HANDLER_IDX_TABLE_SET] = handle_table_set;
        table[HANDLER_IDX_TABLE_FILL] = handle_table_fill;
        table[HANDLER_IDX_I32_CONST] = handle_i32_const;
        table[HANDLER_IDX_I64_CONST] = handle_i64_const;
        table[HANDLER_IDX_F32_CONST] = handle_f32_const;
        table[HANDLER_IDX_F64_CONST] = handle_f64_const;
        table[HANDLER_IDX_I32_EQZ] = handle_i32_eqz;
        table[HANDLER_IDX_I32_EQ] = handle_i32_eq;
        table[HANDLER_IDX_I32_NE] = handle_i32_ne;
        table[HANDLER_IDX_I32_LT_S] = handle_i32_lt_s;
        table[HANDLER_IDX_I32_LT_U] = handle_i32_lt_u;
        table[HANDLER_IDX_I32_GT_S] = handle_i32_gt_s;
        table[HANDLER_IDX_I32_GT_U] = handle_i32_gt_u;
        table[HANDLER_IDX_I32_LE_S] = handle_i32_le_s;
        table[HANDLER_IDX_I32_LE_U] = handle_i32_le_u;
        table[HANDLER_IDX_I32_GE_S] = handle_i32_ge_s;
        table[HANDLER_IDX_I32_GE_U] = handle_i32_ge_u;
        table[HANDLER_IDX_I64_EQZ] = handle_i64_eqz;
        table[HANDLER_IDX_I64_EQ] = handle_i64_eq;
        table[HANDLER_IDX_I64_NE] = handle_i64_ne;
        table[HANDLER_IDX_I64_LT_S] = handle_i64_lt_s;
        table[HANDLER_IDX_I64_LT_U] = handle_i64_lt_u;
        table[HANDLER_IDX_I64_GT_S] = handle_i64_gt_s;
        table[HANDLER_IDX_I64_GT_U] = handle_i64_gt_u;
        table[HANDLER_IDX_I64_LE_S] = handle_i64_le_s;
        table[HANDLER_IDX_I64_LE_U] = handle_i64_le_u;
        table[HANDLER_IDX_I64_GE_S] = handle_i64_ge_s;
        table[HANDLER_IDX_I64_GE_U] = handle_i64_ge_u;
        table[HANDLER_IDX_F32_EQ] = handle_f32_eq;
        table[HANDLER_IDX_F32_NE] = handle_f32_ne;
        table[HANDLER_IDX_F32_LT] = handle_f32_lt;
        table[HANDLER_IDX_F32_GT] = handle_f32_gt;
        table[HANDLER_IDX_F32_LE] = handle_f32_le;
        table[HANDLER_IDX_F32_GE] = handle_f32_ge;
        table[HANDLER_IDX_F64_EQ] = handle_f64_eq;
        table[HANDLER_IDX_F64_NE] = handle_f64_ne;
        table[HANDLER_IDX_F64_LT] = handle_f64_lt;
        table[HANDLER_IDX_F64_GT] = handle_f64_gt;
        table[HANDLER_IDX_F64_LE] = handle_f64_le;
        table[HANDLER_IDX_F64_GE] = handle_f64_ge;
        table[HANDLER_IDX_I32_CLZ] = handle_i32_clz;
        table[HANDLER_IDX_I32_CTZ] = handle_i32_ctz;
        table[HANDLER_IDX_I32_POPCNT] = handle_i32_popcnt;
        table[HANDLER_IDX_I32_ADD] = handle_i32_add;
        table[HANDLER_IDX_I32_SUB] = handle_i32_sub;
        table[HANDLER_IDX_I32_MUL] = handle_i32_mul;
        table[HANDLER_IDX_I32_DIV_S] = handle_i32_div_s;
        table[HANDLER_IDX_I32_DIV_U] = handle_i32_div_u;
        table[HANDLER_IDX_I32_REM_S] = handle_i32_rem_s;
        table[HANDLER_IDX_I32_REM_U] = handle_i32_rem_u;
        table[HANDLER_IDX_I32_AND] = handle_i32_and;
        table[HANDLER_IDX_I32_OR] = handle_i32_or;
        table[HANDLER_IDX_I32_XOR] = handle_i32_xor;
        table[HANDLER_IDX_I32_SHL] = handle_i32_shl;
        table[HANDLER_IDX_I32_SHR_S] = handle_i32_shr_s;
        table[HANDLER_IDX_I32_SHR_U] = handle_i32_shr_u;
        table[HANDLER_IDX_I32_ROTL] = handle_i32_rotl;
        table[HANDLER_IDX_I32_ROTR] = handle_i32_rotr;
        table[HANDLER_IDX_I64_CLZ] = handle_i64_clz;
        table[HANDLER_IDX_I64_CTZ] = handle_i64_ctz;
        table[HANDLER_IDX_I64_POPCNT] = handle_i64_popcnt;
        table[HANDLER_IDX_I64_ADD] = handle_i64_add;
        table[HANDLER_IDX_I64_SUB] = handle_i64_sub;
        table[HANDLER_IDX_I64_MUL] = handle_i64_mul;
        table[HANDLER_IDX_I64_DIV_S] = handle_i64_div_s;
        table[HANDLER_IDX_I64_DIV_U] = handle_i64_div_u;
        table[HANDLER_IDX_I64_REM_S] = handle_i64_rem_s;
        table[HANDLER_IDX_I64_REM_U] = handle_i64_rem_u;
        table[HANDLER_IDX_I64_AND] = handle_i64_and;
        table[HANDLER_IDX_I64_OR] = handle_i64_or;
        table[HANDLER_IDX_I64_XOR] = handle_i64_xor;
        table[HANDLER_IDX_I64_SHL] = handle_i64_shl;
        table[HANDLER_IDX_I64_SHR_S] = handle_i64_shr_s;
        table[HANDLER_IDX_I64_SHR_U] = handle_i64_shr_u;
        table[HANDLER_IDX_I64_ROTL] = handle_i64_rotl;
        table[HANDLER_IDX_I64_ROTR] = handle_i64_rotr;
        table[HANDLER_IDX_F32_ABS] = handle_f32_abs;
        table[HANDLER_IDX_F32_NEG] = handle_f32_neg;
        table[HANDLER_IDX_F32_CEIL] = handle_f32_ceil;
        table[HANDLER_IDX_F32_FLOOR] = handle_f32_floor;
        table[HANDLER_IDX_F32_TRUNC] = handle_f32_trunc;
        table[HANDLER_IDX_F32_NEAREST] = handle_f32_nearest;
        table[HANDLER_IDX_F32_SQRT] = handle_f32_sqrt;
        table[HANDLER_IDX_F32_ADD] = handle_f32_add;
        table[HANDLER_IDX_F32_SUB] = handle_f32_sub;
        table[HANDLER_IDX_F32_MUL] = handle_f32_mul;
        table[HANDLER_IDX_F32_DIV] = handle_f32_div;
        table[HANDLER_IDX_F32_MIN] = handle_f32_min;
        table[HANDLER_IDX_F32_MAX] = handle_f32_max;
        table[HANDLER_IDX_F32_COPYSIGN] = handle_f32_copysign;
        table[HANDLER_IDX_F64_ABS] = handle_f64_abs;
        table[HANDLER_IDX_F64_NEG] = handle_f64_neg;
        table[HANDLER_IDX_F64_CEIL] = handle_f64_ceil;
        table[HANDLER_IDX_F64_FLOOR] = handle_f64_floor;
        table[HANDLER_IDX_F64_TRUNC] = handle_f64_trunc;
        table[HANDLER_IDX_F64_NEAREST] = handle_f64_nearest;
        table[HANDLER_IDX_F64_SQRT] = handle_f64_sqrt;
        table[HANDLER_IDX_F64_ADD] = handle_f64_add;
        table[HANDLER_IDX_F64_SUB] = handle_f64_sub;
        table[HANDLER_IDX_F64_MUL] = handle_f64_mul;
        table[HANDLER_IDX_F64_DIV] = handle_f64_div;
        table[HANDLER_IDX_F64_MIN] = handle_f64_min;
        table[HANDLER_IDX_F64_MAX] = handle_f64_max;
        table[HANDLER_IDX_F64_COPYSIGN] = handle_f64_copysign;
        table[HANDLER_IDX_F32_CONVERT_I32_S] = handle_f32_convert_i32_s;
        table[HANDLER_IDX_F32_CONVERT_I32_U] = handle_f32_convert_i32_u;
        table[HANDLER_IDX_F32_CONVERT_I64_S] = handle_f32_convert_i64_s;
        table[HANDLER_IDX_F32_CONVERT_I64_U] = handle_f32_convert_i64_u;
        table[HANDLER_IDX_F64_CONVERT_I32_S] = handle_f64_convert_i32_s;
        table[HANDLER_IDX_F64_CONVERT_I32_U] = handle_f64_convert_i32_u;
        table[HANDLER_IDX_F64_CONVERT_I64_S] = handle_f64_convert_i64_s;
        table[HANDLER_IDX_F64_CONVERT_I64_U] = handle_f64_convert_i64_u;
        table[HANDLER_IDX_I32_REINTERPRET_F32] = handle_i32_reinterpret_f32;
        table[HANDLER_IDX_I64_REINTERPRET_F64] = handle_i64_reinterpret_f64;
        table[HANDLER_IDX_F32_REINTERPRET_I32] = handle_f32_reinterpret_i32;
        table[HANDLER_IDX_F64_REINTERPRET_I64] = handle_f64_reinterpret_i64;
        table[HANDLER_IDX_I32_WRAP_I64] = handle_i32_wrap_i64;
        table[HANDLER_IDX_I32_TRUNC_F32_S] = handle_i32_trunc_f32_s;
        table[HANDLER_IDX_I32_TRUNC_F32_U] = handle_i32_trunc_f32_u;
        table[HANDLER_IDX_I32_TRUNC_F64_S] = handle_i32_trunc_f64_s;
        table[HANDLER_IDX_I32_TRUNC_F64_U] = handle_i32_trunc_f64_u;
        table[HANDLER_IDX_I64_EXTEND_I32_S] = handle_i64_extend32_s;
        table[HANDLER_IDX_I64_EXTEND_I32_U] = handle_i64_extend_i32_u;
        table[HANDLER_IDX_I64_TRUNC_F32_S] = handle_i64_trunc_f32_s;
        table[HANDLER_IDX_I64_TRUNC_F32_U] = handle_i64_trunc_f32_u;
        table[HANDLER_IDX_I64_TRUNC_F64_S] = handle_i64_trunc_f64_s;
        table[HANDLER_IDX_I64_TRUNC_F64_U] = handle_i64_trunc_f64_u;
        table[HANDLER_IDX_F32_DEMOTE_F64] = handle_f32_demote_f64;
        table[HANDLER_IDX_F64_PROMOTE_F32] = handle_f64_promote_f32;
        table[HANDLER_IDX_I32_EXTEND8_S] = handle_i32_extend8_s;
        table[HANDLER_IDX_I32_EXTEND16_S] = handle_i32_extend16_s;
        table[HANDLER_IDX_I64_EXTEND8_S] = handle_i64_extend8_s;
        table[HANDLER_IDX_I64_EXTEND16_S] = handle_i64_extend16_s;
        table[HANDLER_IDX_I64_EXTEND32_S] = handle_i64_extend32_s;
        table[HANDLER_IDX_I32_TRUNC_SAT_F32_S] = handle_i32_trunc_sat_f32_s;
        table[HANDLER_IDX_I32_TRUNC_SAT_F32_U] = handle_i32_trunc_sat_f32_u;
        table[HANDLER_IDX_I32_TRUNC_SAT_F64_S] = handle_i32_trunc_sat_f64_s;
        table[HANDLER_IDX_I32_TRUNC_SAT_F64_U] = handle_i32_trunc_sat_f64_u;
        table[HANDLER_IDX_I64_TRUNC_SAT_F32_S] = handle_i64_trunc_sat_f32_s;
        table[HANDLER_IDX_I64_TRUNC_SAT_F32_U] = handle_i64_trunc_sat_f32_u;
        table[HANDLER_IDX_I64_TRUNC_SAT_F64_S] = handle_i64_trunc_sat_f64_s;
        table[HANDLER_IDX_I64_TRUNC_SAT_F64_U] = handle_i64_trunc_sat_f64_u;
        table
    };
}

impl<'a> ExecutionContext<'a> {
    fn pop_n_values(&mut self, n: usize) -> Result<Vec<Val>, RuntimeError> {
        let len = self.value_stack.len();
        if len < n {
            return Err(RuntimeError::ValueStackUnderflow);
        }

        // Use drain for efficient removal and collection
        let start = len - n;
        let mut values: Vec<Val> = self.value_stack.drain(start..).collect();
        values.reverse();
        Ok(values)
    }
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
