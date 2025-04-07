use super::{value::*};
use crate::structure::{instructions::Instr};
use crate::error::RuntimeError;
use std::arch::asm;
use crate::structure::types::{FuncIdx, GlobalIdx, LocalIdx, TableIdx, TypeIdx, LabelIdx as StructureLabelIdx};
use crate::structure::instructions::Memarg; 
use crate::structure::module::Func;
use std::collections::HashMap;
use num::{Float, NumCast};
use std::ops::{BitAnd, BitOr, BitXor, Add, Sub, Mul, Div};
use lazy_static::lazy_static;
use crate::execution::value::Val;
use crate::execution::module::{ModuleInst, GetInstanceByIdx}; 
use crate::execution::func::{FuncAddr, FuncInst};
use std::rc::{Rc, Weak}; 
use crate::structure::types::{FuncType, ValueType, NumType, VecType}; 

#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    None,
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    LocalIdx(LocalIdx),
    GlobalIdx(GlobalIdx),
    FuncIdx(FuncIdx),
    TableIdx(TableIdx),
    TypeIdx(TypeIdx),
    LabelIdx(usize),
    MemArg(Memarg),
    BrTable { targets: Vec<usize>, default: usize },
}

#[derive(Clone, Debug)]
pub struct ProcessedInstr {
    handler_index: usize,
    operand: Operand,
}

pub struct ExecutionContext<'a> {
    pub frame: &'a mut crate::execution::stackopt::Frame,
    pub value_stack: &'a mut Vec<Val>,
    pub ip: usize,
}

type HandlerFn = fn(&mut ExecutionContext, Operand) -> Result<usize, RuntimeError>;

#[derive(Clone)]
pub enum ModuleLevelInstr{
    Invoke(FuncAddr),
    Return,
}

const HANDLER_IDX_UNREACHABLE: usize = 0x00;
const HANDLER_IDX_NOP: usize = 0x01;
const HANDLER_IDX_BLOCK: usize = 0x02;
const HANDLER_IDX_LOOP: usize = 0x03;
const HANDLER_IDX_IF: usize = 0x04;
const HANDLER_IDX_ELSE: usize = 0x05;
const HANDLER_IDX_END: usize = 0x0B;
const HANDLER_IDX_BR: usize = 0x0C;
const HANDLER_IDX_BR_IF: usize = 0x0D;
const HANDLER_IDX_BR_TABLE: usize = 0x0E;
const HANDLER_IDX_RETURN: usize = 0x0F;
const HANDLER_IDX_CALL: usize = 0x10;
const HANDLER_IDX_CALL_INDIRECT: usize = 0x11;

const HANDLER_IDX_DROP: usize = 0x1A;
const HANDLER_IDX_SELECT: usize = 0x1B;
// const HANDLER_IDX_SELECT_T: usize = 0x1C;

const HANDLER_IDX_LOCAL_GET: usize = 0x20;
const HANDLER_IDX_LOCAL_SET: usize = 0x21;
const HANDLER_IDX_LOCAL_TEE: usize = 0x22;
const HANDLER_IDX_GLOBAL_GET: usize = 0x23;
const HANDLER_IDX_GLOBAL_SET: usize = 0x24;
// const HANDLER_IDX_TABLE_GET: usize = 0x25;
// const HANDLER_IDX_TABLE_SET: usize = 0x26;

const HANDLER_IDX_I32_LOAD: usize = 0x28;
const HANDLER_IDX_I64_LOAD: usize = 0x29;
const HANDLER_IDX_F32_LOAD: usize = 0x2A;
const HANDLER_IDX_F64_LOAD: usize = 0x2B;
const HANDLER_IDX_I32_LOAD8_S: usize = 0x2C;
const HANDLER_IDX_I32_LOAD8_U: usize = 0x2D;
const HANDLER_IDX_I32_LOAD16_S: usize = 0x2E;
const HANDLER_IDX_I32_LOAD16_U: usize = 0x2F;
const HANDLER_IDX_I64_LOAD8_S: usize = 0x30;
const HANDLER_IDX_I64_LOAD8_U: usize = 0x31;
const HANDLER_IDX_I64_LOAD16_S: usize = 0x32;
const HANDLER_IDX_I64_LOAD16_U: usize = 0x33;
const HANDLER_IDX_I64_LOAD32_S: usize = 0x34;
const HANDLER_IDX_I64_LOAD32_U: usize = 0x35;
const HANDLER_IDX_I32_STORE: usize = 0x36;
const HANDLER_IDX_I64_STORE: usize = 0x37;
const HANDLER_IDX_F32_STORE: usize = 0x38;
const HANDLER_IDX_F64_STORE: usize = 0x39;
const HANDLER_IDX_I32_STORE8: usize = 0x3A;
const HANDLER_IDX_I32_STORE16: usize = 0x3B;
const HANDLER_IDX_I64_STORE8: usize = 0x3C;
const HANDLER_IDX_I64_STORE16: usize = 0x3D;
const HANDLER_IDX_I64_STORE32: usize = 0x3E;
const HANDLER_IDX_MEMORY_SIZE: usize = 0x3F;
const HANDLER_IDX_MEMORY_GROW: usize = 0x40;

const HANDLER_IDX_I32_CONST: usize = 0x41;
const HANDLER_IDX_I64_CONST: usize = 0x42;
const HANDLER_IDX_F32_CONST: usize = 0x43;
const HANDLER_IDX_F64_CONST: usize = 0x44;

const HANDLER_IDX_I32_EQZ: usize = 0x45;
const HANDLER_IDX_I32_EQ: usize = 0x46;
const HANDLER_IDX_I32_NE: usize = 0x47;
const HANDLER_IDX_I32_LT_S: usize = 0x48;
const HANDLER_IDX_I32_LT_U: usize = 0x49;
const HANDLER_IDX_I32_GT_S: usize = 0x4A;
const HANDLER_IDX_I32_GT_U: usize = 0x4B;
const HANDLER_IDX_I32_LE_S: usize = 0x4C;
const HANDLER_IDX_I32_LE_U: usize = 0x4D;
const HANDLER_IDX_I32_GE_S: usize = 0x4E;
const HANDLER_IDX_I32_GE_U: usize = 0x4F;

const HANDLER_IDX_I64_EQZ: usize = 0x50;
const HANDLER_IDX_I64_EQ: usize = 0x51;
const HANDLER_IDX_I64_NE: usize = 0x52;
const HANDLER_IDX_I64_LT_S: usize = 0x53;
const HANDLER_IDX_I64_LT_U: usize = 0x54;
const HANDLER_IDX_I64_GT_S: usize = 0x55;
const HANDLER_IDX_I64_GT_U: usize = 0x56;
const HANDLER_IDX_I64_LE_S: usize = 0x57;
const HANDLER_IDX_I64_LE_U: usize = 0x58;
const HANDLER_IDX_I64_GE_S: usize = 0x59;
const HANDLER_IDX_I64_GE_U: usize = 0x5A;

const HANDLER_IDX_F32_EQ: usize = 0x5B;
const HANDLER_IDX_F32_NE: usize = 0x5C;
const HANDLER_IDX_F32_LT: usize = 0x5D;
const HANDLER_IDX_F32_GT: usize = 0x5E;
const HANDLER_IDX_F32_LE: usize = 0x5F;
const HANDLER_IDX_F32_GE: usize = 0x60;

const HANDLER_IDX_F64_EQ: usize = 0x61;
const HANDLER_IDX_F64_NE: usize = 0x62;
const HANDLER_IDX_F64_LT: usize = 0x63;
const HANDLER_IDX_F64_GT: usize = 0x64;
const HANDLER_IDX_F64_LE: usize = 0x65;
const HANDLER_IDX_F64_GE: usize = 0x66;

const HANDLER_IDX_I32_CLZ: usize = 0x67;
const HANDLER_IDX_I32_CTZ: usize = 0x68;
const HANDLER_IDX_I32_POPCNT: usize = 0x69;
const HANDLER_IDX_I32_ADD: usize = 0x6A;
const HANDLER_IDX_I32_SUB: usize = 0x6B;
const HANDLER_IDX_I32_MUL: usize = 0x6C;
const HANDLER_IDX_I32_DIV_S: usize = 0x6D;
const HANDLER_IDX_I32_DIV_U: usize = 0x6E;
const HANDLER_IDX_I32_REM_S: usize = 0x6F;
const HANDLER_IDX_I32_REM_U: usize = 0x70;
const HANDLER_IDX_I32_AND: usize = 0x71;
const HANDLER_IDX_I32_OR: usize = 0x72;
const HANDLER_IDX_I32_XOR: usize = 0x73;
const HANDLER_IDX_I32_SHL: usize = 0x74;
const HANDLER_IDX_I32_SHR_S: usize = 0x75;
const HANDLER_IDX_I32_SHR_U: usize = 0x76;
const HANDLER_IDX_I32_ROTL: usize = 0x77;
const HANDLER_IDX_I32_ROTR: usize = 0x78;

const HANDLER_IDX_I64_CLZ: usize = 0x79;
const HANDLER_IDX_I64_CTZ: usize = 0x7A;
const HANDLER_IDX_I64_POPCNT: usize = 0x7B;
const HANDLER_IDX_I64_ADD: usize = 0x7C;
const HANDLER_IDX_I64_SUB: usize = 0x7D;
const HANDLER_IDX_I64_MUL: usize = 0x7E;
const HANDLER_IDX_I64_DIV_S: usize = 0x7F;
const HANDLER_IDX_I64_DIV_U: usize = 0x80;
const HANDLER_IDX_I64_REM_S: usize = 0x81;
const HANDLER_IDX_I64_REM_U: usize = 0x82;
const HANDLER_IDX_I64_AND: usize = 0x83;
const HANDLER_IDX_I64_OR: usize = 0x84;
const HANDLER_IDX_I64_XOR: usize = 0x85;
const HANDLER_IDX_I64_SHL: usize = 0x86;
const HANDLER_IDX_I64_SHR_S: usize = 0x87;
const HANDLER_IDX_I64_SHR_U: usize = 0x88;
const HANDLER_IDX_I64_ROTL: usize = 0x89;
const HANDLER_IDX_I64_ROTR: usize = 0x8A;

const HANDLER_IDX_F32_ABS: usize = 0x8B;
const HANDLER_IDX_F32_NEG: usize = 0x8C;
const HANDLER_IDX_F32_CEIL: usize = 0x8D;
const HANDLER_IDX_F32_FLOOR: usize = 0x8E;
const HANDLER_IDX_F32_TRUNC: usize = 0x8F;
const HANDLER_IDX_F32_NEAREST: usize = 0x90;
const HANDLER_IDX_F32_SQRT: usize = 0x91;
const HANDLER_IDX_F32_ADD: usize = 0x92;
const HANDLER_IDX_F32_SUB: usize = 0x93;
const HANDLER_IDX_F32_MUL: usize = 0x94;
const HANDLER_IDX_F32_DIV: usize = 0x95;
const HANDLER_IDX_F32_MIN: usize = 0x96;
const HANDLER_IDX_F32_MAX: usize = 0x97;
const HANDLER_IDX_F32_COPYSIGN: usize = 0x98;

const HANDLER_IDX_F64_ABS: usize = 0x99;
const HANDLER_IDX_F64_NEG: usize = 0x9A;
const HANDLER_IDX_F64_CEIL: usize = 0x9B;
const HANDLER_IDX_F64_FLOOR: usize = 0x9C;
const HANDLER_IDX_F64_TRUNC: usize = 0x9D;
const HANDLER_IDX_F64_NEAREST: usize = 0x9E;
const HANDLER_IDX_F64_SQRT: usize = 0x9F;
const HANDLER_IDX_F64_ADD: usize = 0xA0;
const HANDLER_IDX_F64_SUB: usize = 0xA1;
const HANDLER_IDX_F64_MUL: usize = 0xA2;
const HANDLER_IDX_F64_DIV: usize = 0xA3;
const HANDLER_IDX_F64_MIN: usize = 0xA4;
const HANDLER_IDX_F64_MAX: usize = 0xA5;
const HANDLER_IDX_F64_COPYSIGN: usize = 0xA6;

const HANDLER_IDX_I32_WRAP_I64: usize = 0xA7;
const HANDLER_IDX_I32_TRUNC_F32_S: usize = 0xA8;
const HANDLER_IDX_I32_TRUNC_F32_U: usize = 0xA9;
const HANDLER_IDX_I32_TRUNC_F64_S: usize = 0xAA;
const HANDLER_IDX_I32_TRUNC_F64_U: usize = 0xAB;
const HANDLER_IDX_I64_EXTEND_I32_S: usize = 0xAC;
const HANDLER_IDX_I64_EXTEND_I32_U: usize = 0xAD;
const HANDLER_IDX_I64_TRUNC_F32_S: usize = 0xAE;
const HANDLER_IDX_I64_TRUNC_F32_U: usize = 0xAF;
const HANDLER_IDX_I64_TRUNC_F64_S: usize = 0xB0;
const HANDLER_IDX_I64_TRUNC_F64_U: usize = 0xB1;
const HANDLER_IDX_F32_CONVERT_I32_S: usize = 0xB2;
const HANDLER_IDX_F32_CONVERT_I32_U: usize = 0xB3;
const HANDLER_IDX_F32_CONVERT_I64_S: usize = 0xB4;
const HANDLER_IDX_F32_CONVERT_I64_U: usize = 0xB5;
const HANDLER_IDX_F32_DEMOTE_F64: usize = 0xB6;
const HANDLER_IDX_F64_CONVERT_I32_S: usize = 0xB7;
const HANDLER_IDX_F64_CONVERT_I32_U: usize = 0xB8;
const HANDLER_IDX_F64_CONVERT_I64_S: usize = 0xB9;
const HANDLER_IDX_F64_CONVERT_I64_U: usize = 0xBA;
const HANDLER_IDX_F64_PROMOTE_F32: usize = 0xBB;
const HANDLER_IDX_I32_REINTERPRET_F32: usize = 0xBC;
const HANDLER_IDX_I64_REINTERPRET_F64: usize = 0xBD;
const HANDLER_IDX_F32_REINTERPRET_I32: usize = 0xBE;
const HANDLER_IDX_F64_REINTERPRET_I64: usize = 0xBF;

const HANDLER_IDX_I32_EXTEND8_S: usize = 0xC0;
const HANDLER_IDX_I32_EXTEND16_S: usize = 0xC1;
const HANDLER_IDX_I64_EXTEND8_S: usize = 0xC2;
const HANDLER_IDX_I64_EXTEND16_S: usize = 0xC3;
const HANDLER_IDX_I64_EXTEND32_S: usize = 0xC4;

// TODO: Add remaining indices (Ref types, Table, Bulk Memory, SIMD, TruncSat)

const MAX_HANDLER_INDEX: usize = 0xC5;

fn preprocess_instructions(original_instrs: &[Instr]) -> Result<Vec<ProcessedInstr>, RuntimeError> {
    let mut processed = Vec::with_capacity(original_instrs.len());

    // Stores fixup info: (pc_to_patch, relative_depth, is_if_false_jump, is_else_jump)
    // Note: relative_depth is usize::MAX for fixups that are completed.
    let mut fixups: Vec<(usize, usize, bool, bool)> = Vec::new();

    // Phase 1: Map instructions, identify labels, store fixup info
    for instr in original_instrs.iter() {
        let current_processed_pc = processed.len();
        let mut handler_index = HANDLER_IDX_NOP;
        let mut operand = Operand::None;

        match instr {
            Instr::Unreachable => handler_index = HANDLER_IDX_UNREACHABLE,
            Instr::Nop => handler_index = HANDLER_IDX_NOP,

            Instr::Block(_, _) => {
                handler_index = HANDLER_IDX_BLOCK;
            }
            Instr::Loop(_, _) => {
                handler_index = HANDLER_IDX_LOOP;
                operand = Operand::LabelIdx(current_processed_pc);
            }
            Instr::If(_, _, _) => {
                handler_index = HANDLER_IDX_IF;
                fixups.push((current_processed_pc, 0, true, false));
                operand = Operand::LabelIdx(usize::MAX);
            }
            Instr::ElseMarker => {
                handler_index = HANDLER_IDX_ELSE;
                fixups.push((current_processed_pc, 0, false, true));
                operand = Operand::LabelIdx(usize::MAX);
            }
            Instr::EndMarker => {
                handler_index = HANDLER_IDX_END;
            }
            Instr::Br(label_idx) => {
                handler_index = HANDLER_IDX_BR;
                fixups.push((current_processed_pc, label_idx.0 as usize, false, false));
                operand = Operand::LabelIdx(usize::MAX);
            }
            Instr::BrIf(label_idx) => {
                handler_index = HANDLER_IDX_BR_IF;
                fixups.push((current_processed_pc, label_idx.0 as usize, false, false));
                operand = Operand::LabelIdx(usize::MAX); 
            }
            Instr::BrTable(indices, default) => {
                handler_index = HANDLER_IDX_BR_TABLE;
                for label_idx in indices.iter() {
                    fixups.push((current_processed_pc, label_idx.0 as usize, false, false));
                }
                fixups.push((current_processed_pc, default.0 as usize, false, false));
                operand = Operand::None;
            }
            Instr::Return => handler_index = HANDLER_IDX_RETURN,
            Instr::Call(func_idx) => {
                handler_index = HANDLER_IDX_CALL;
                operand = Operand::FuncIdx(func_idx.clone());
            }
            Instr::CallIndirect(table_idx, type_idx) => {
                handler_index = HANDLER_IDX_CALL_INDIRECT;
                operand = Operand::TypeIdx(type_idx.clone());
            }
            Instr::Drop => handler_index = HANDLER_IDX_DROP,
            Instr::Select(_) => {
                handler_index = HANDLER_IDX_SELECT;
                operand = Operand::None;
            }
            Instr::LocalGet(idx) => { handler_index = HANDLER_IDX_LOCAL_GET; operand = Operand::LocalIdx(idx.clone()); }
            Instr::LocalSet(idx) => { handler_index = HANDLER_IDX_LOCAL_SET; operand = Operand::LocalIdx(idx.clone()); }
            Instr::LocalTee(idx) => { handler_index = HANDLER_IDX_LOCAL_TEE; operand = Operand::LocalIdx(idx.clone()); }
            Instr::GlobalGet(idx) => { handler_index = HANDLER_IDX_GLOBAL_GET; operand = Operand::GlobalIdx(idx.clone()); }
            Instr::GlobalSet(idx) => { handler_index = HANDLER_IDX_GLOBAL_SET; operand = Operand::GlobalIdx(idx.clone()); }
            Instr::I32Load(arg) => { handler_index = HANDLER_IDX_I32_LOAD; operand = Operand::MemArg(arg.clone()); }
            Instr::I64Load(arg) => { handler_index = HANDLER_IDX_I64_LOAD; operand = Operand::MemArg(arg.clone()); }
            Instr::F32Load(arg) => { handler_index = HANDLER_IDX_F32_LOAD; operand = Operand::MemArg(arg.clone()); }
            Instr::F64Load(arg) => { handler_index = HANDLER_IDX_F64_LOAD; operand = Operand::MemArg(arg.clone()); }
            Instr::I32Load8S(arg) => { handler_index = HANDLER_IDX_I32_LOAD8_S; operand = Operand::MemArg(arg.clone()); }
            Instr::I32Load8U(arg) => { handler_index = HANDLER_IDX_I32_LOAD8_U; operand = Operand::MemArg(arg.clone()); }
            Instr::I32Load16S(arg) => { handler_index = HANDLER_IDX_I32_LOAD16_S; operand = Operand::MemArg(arg.clone()); }
            Instr::I32Load16U(arg) => { handler_index = HANDLER_IDX_I32_LOAD16_U; operand = Operand::MemArg(arg.clone()); }
            Instr::I64Load8S(arg) => { handler_index = HANDLER_IDX_I64_LOAD8_S; operand = Operand::MemArg(arg.clone()); }
            Instr::I64Load8U(arg) => { handler_index = HANDLER_IDX_I64_LOAD8_U; operand = Operand::MemArg(arg.clone()); }
            Instr::I64Load16S(arg) => { handler_index = HANDLER_IDX_I64_LOAD16_S; operand = Operand::MemArg(arg.clone()); }
            Instr::I64Load16U(arg) => { handler_index = HANDLER_IDX_I64_LOAD16_U; operand = Operand::MemArg(arg.clone()); }
            Instr::I64Load32S(arg) => { handler_index = HANDLER_IDX_I64_LOAD32_S; operand = Operand::MemArg(arg.clone()); }
            Instr::I64Load32U(arg) => { handler_index = HANDLER_IDX_I64_LOAD32_U; operand = Operand::MemArg(arg.clone()); }
            Instr::I32Store(arg) => { handler_index = HANDLER_IDX_I32_STORE; operand = Operand::MemArg(arg.clone()); }
            Instr::I64Store(arg) => { handler_index = HANDLER_IDX_I64_STORE; operand = Operand::MemArg(arg.clone()); }
            Instr::F32Store(arg) => { handler_index = HANDLER_IDX_F32_STORE; operand = Operand::MemArg(arg.clone()); }
            Instr::F64Store(arg) => { handler_index = HANDLER_IDX_F64_STORE; operand = Operand::MemArg(arg.clone()); }
            Instr::I32Store8(arg) => { handler_index = HANDLER_IDX_I32_STORE8; operand = Operand::MemArg(arg.clone()); }
            Instr::I32Store16(arg) => { handler_index = HANDLER_IDX_I32_STORE16; operand = Operand::MemArg(arg.clone()); }
            Instr::I64Store8(arg) => { handler_index = HANDLER_IDX_I64_STORE8; operand = Operand::MemArg(arg.clone()); }
            Instr::I64Store16(arg) => { handler_index = HANDLER_IDX_I64_STORE16; operand = Operand::MemArg(arg.clone()); }
            Instr::I64Store32(arg) => { handler_index = HANDLER_IDX_I64_STORE32; operand = Operand::MemArg(arg.clone()); }
            Instr::MemorySize => handler_index = HANDLER_IDX_MEMORY_SIZE,
            Instr::MemoryGrow => handler_index = HANDLER_IDX_MEMORY_GROW,
            Instr::I32Const(v) => { handler_index = HANDLER_IDX_I32_CONST; operand = Operand::I32(*v); }
            Instr::I64Const(v) => { handler_index = HANDLER_IDX_I64_CONST; operand = Operand::I64(*v); }
            Instr::F32Const(v) => { handler_index = HANDLER_IDX_F32_CONST; operand = Operand::F32(*v); }
            Instr::F64Const(v) => { handler_index = HANDLER_IDX_F64_CONST; operand = Operand::F64(*v); }
            Instr::I32Eqz => handler_index = HANDLER_IDX_I32_EQZ,
            Instr::I32Eq => handler_index = HANDLER_IDX_I32_EQ,
            Instr::I32Ne => handler_index = HANDLER_IDX_I32_NE,
            Instr::I32LtS => handler_index = HANDLER_IDX_I32_LT_S,
            Instr::I32LtU => handler_index = HANDLER_IDX_I32_LT_U,
            Instr::I32GtS => handler_index = HANDLER_IDX_I32_GT_S,
            Instr::I32GtU => handler_index = HANDLER_IDX_I32_GT_U,
            Instr::I32LeS => handler_index = HANDLER_IDX_I32_LE_S,
            Instr::I32LeU => handler_index = HANDLER_IDX_I32_LE_U,
            Instr::I32GeS => handler_index = HANDLER_IDX_I32_GE_S,
            Instr::I32GeU => handler_index = HANDLER_IDX_I32_GE_U,
            Instr::I64Eqz => handler_index = HANDLER_IDX_I64_EQZ,
            Instr::I64Eq => handler_index = HANDLER_IDX_I64_EQ,
            Instr::I64Ne => handler_index = HANDLER_IDX_I64_NE,
            Instr::I64LtS => handler_index = HANDLER_IDX_I64_LT_S,
            Instr::I64LtU => handler_index = HANDLER_IDX_I64_LT_U,
            Instr::I64GtS => handler_index = HANDLER_IDX_I64_GT_S,
            Instr::I64GtU => handler_index = HANDLER_IDX_I64_GT_U,
            Instr::I64LeS => handler_index = HANDLER_IDX_I64_LE_S,
            Instr::I64LeU => handler_index = HANDLER_IDX_I64_LE_U,
            Instr::I64GeS => handler_index = HANDLER_IDX_I64_GE_S,
            Instr::I64GeU => handler_index = HANDLER_IDX_I64_GE_U,
            Instr::F32Eq => handler_index = HANDLER_IDX_F32_EQ,
            Instr::F32Ne => handler_index = HANDLER_IDX_F32_NE,
            Instr::F32Lt => handler_index = HANDLER_IDX_F32_LT,
            Instr::F32Gt => handler_index = HANDLER_IDX_F32_GT,
            Instr::F32Le => handler_index = HANDLER_IDX_F32_LE,
            Instr::F32Ge => handler_index = HANDLER_IDX_F32_GE,
            Instr::F64Eq => handler_index = HANDLER_IDX_F64_EQ,
            Instr::F64Ne => handler_index = HANDLER_IDX_F64_NE,
            Instr::F64Lt => handler_index = HANDLER_IDX_F64_LT,
            Instr::F64Gt => handler_index = HANDLER_IDX_F64_GT,
            Instr::F64Le => handler_index = HANDLER_IDX_F64_LE,
            Instr::F64Ge => handler_index = HANDLER_IDX_F64_GE,
            Instr::I32Clz => handler_index = HANDLER_IDX_I32_CLZ,
            Instr::I32Ctz => handler_index = HANDLER_IDX_I32_CTZ,
            Instr::I32Popcnt => handler_index = HANDLER_IDX_I32_POPCNT,
            Instr::I32Add => handler_index = HANDLER_IDX_I32_ADD,
            Instr::I32Sub => handler_index = HANDLER_IDX_I32_SUB,
            Instr::I32Mul => handler_index = HANDLER_IDX_I32_MUL,
            Instr::I32DivS => handler_index = HANDLER_IDX_I32_DIV_S,
            Instr::I32DivU => handler_index = HANDLER_IDX_I32_DIV_U,
            Instr::I32RemS => handler_index = HANDLER_IDX_I32_REM_S,
            Instr::I32RemU => handler_index = HANDLER_IDX_I32_REM_U,
            Instr::I32And => handler_index = HANDLER_IDX_I32_AND,
            Instr::I32Or => handler_index = HANDLER_IDX_I32_OR,
            Instr::I32Xor => handler_index = HANDLER_IDX_I32_XOR,
            Instr::I32Shl => handler_index = HANDLER_IDX_I32_SHL,
            Instr::I32ShrS => handler_index = HANDLER_IDX_I32_SHR_S,
            Instr::I32ShrU => handler_index = HANDLER_IDX_I32_SHR_U,
            Instr::I32Rotl => handler_index = HANDLER_IDX_I32_ROTL,
            Instr::I32Rotr => handler_index = HANDLER_IDX_I32_ROTR,
            Instr::I64Clz => handler_index = HANDLER_IDX_I64_CLZ,
            Instr::I64Ctz => handler_index = HANDLER_IDX_I64_CTZ,
            Instr::I64Popcnt => handler_index = HANDLER_IDX_I64_POPCNT,
            Instr::I64Add => handler_index = HANDLER_IDX_I64_ADD,
            Instr::I64Sub => handler_index = HANDLER_IDX_I64_SUB,
            Instr::I64Mul => handler_index = HANDLER_IDX_I64_MUL,
            Instr::I64DivS => handler_index = HANDLER_IDX_I64_DIV_S,
            Instr::I64DivU => handler_index = HANDLER_IDX_I64_DIV_U,
            Instr::I64RemS => handler_index = HANDLER_IDX_I64_REM_S,
            Instr::I64RemU => handler_index = HANDLER_IDX_I64_REM_U,
            Instr::I64And => handler_index = HANDLER_IDX_I64_AND,
            Instr::I64Or => handler_index = HANDLER_IDX_I64_OR,
            Instr::I64Xor => handler_index = HANDLER_IDX_I64_XOR,
            Instr::I64Shl => handler_index = HANDLER_IDX_I64_SHL,
            Instr::I64ShrS => handler_index = HANDLER_IDX_I64_SHR_S,
            Instr::I64ShrU => handler_index = HANDLER_IDX_I64_SHR_U,
            Instr::I64Rotl => handler_index = HANDLER_IDX_I64_ROTL,
            Instr::I64Rotr => handler_index = HANDLER_IDX_I64_ROTR,
            Instr::F32Abs => handler_index = HANDLER_IDX_F32_ABS,
            Instr::F32Neg => handler_index = HANDLER_IDX_F32_NEG,
            Instr::F32Ceil => handler_index = HANDLER_IDX_F32_CEIL,
            Instr::F32Floor => handler_index = HANDLER_IDX_F32_FLOOR,
            Instr::F32Trunc => handler_index = HANDLER_IDX_F32_TRUNC,
            Instr::F32Nearest => handler_index = HANDLER_IDX_F32_NEAREST,
            Instr::F32Sqrt => handler_index = HANDLER_IDX_F32_SQRT,
            Instr::F32Add => handler_index = HANDLER_IDX_F32_ADD,
            Instr::F32Sub => handler_index = HANDLER_IDX_F32_SUB,
            Instr::F32Mul => handler_index = HANDLER_IDX_F32_MUL,
            Instr::F32Div => handler_index = HANDLER_IDX_F32_DIV,
            Instr::F32Min => handler_index = HANDLER_IDX_F32_MIN,
            Instr::F32Max => handler_index = HANDLER_IDX_F32_MAX,
            Instr::F32Copysign => handler_index = HANDLER_IDX_F32_COPYSIGN,
            Instr::F64Abs => handler_index = HANDLER_IDX_F64_ABS,
            Instr::F64Neg => handler_index = HANDLER_IDX_F64_NEG,
            Instr::F64Ceil => handler_index = HANDLER_IDX_F64_CEIL,
            Instr::F64Floor => handler_index = HANDLER_IDX_F64_FLOOR,
            Instr::F64Trunc => handler_index = HANDLER_IDX_F64_TRUNC,
            Instr::F64Nearest => handler_index = HANDLER_IDX_F64_NEAREST,
            Instr::F64Sqrt => handler_index = HANDLER_IDX_F64_SQRT,
            Instr::F64Add => handler_index = HANDLER_IDX_F64_ADD,
            Instr::F64Sub => handler_index = HANDLER_IDX_F64_SUB,
            Instr::F64Mul => handler_index = HANDLER_IDX_F64_MUL,
            Instr::F64Div => handler_index = HANDLER_IDX_F64_DIV,
            Instr::F64Min => handler_index = HANDLER_IDX_F64_MIN,
            Instr::F64Max => handler_index = HANDLER_IDX_F64_MAX,
            Instr::F64Copysign => handler_index = HANDLER_IDX_F64_COPYSIGN,
            Instr::I32WrapI64 => handler_index = HANDLER_IDX_I32_WRAP_I64,
            Instr::I32TruncF32S => handler_index = HANDLER_IDX_I32_TRUNC_F32_S,
            Instr::I32TruncF32U => handler_index = HANDLER_IDX_I32_TRUNC_F32_U,
            Instr::I32TruncF64S => handler_index = HANDLER_IDX_I32_TRUNC_F64_S,
            Instr::I32TruncF64U => handler_index = HANDLER_IDX_I32_TRUNC_F64_U,
            Instr::I64ExtendI32S => handler_index = HANDLER_IDX_I64_EXTEND_I32_S,
            Instr::I64ExtendI32U => handler_index = HANDLER_IDX_I64_EXTEND_I32_U,
            Instr::I64TruncF32S => handler_index = HANDLER_IDX_I64_TRUNC_F32_S,
            Instr::I64TruncF32U => handler_index = HANDLER_IDX_I64_TRUNC_F32_U,
            Instr::I64TruncF64S => handler_index = HANDLER_IDX_I64_TRUNC_F64_S,
            Instr::I64TruncF64U => handler_index = HANDLER_IDX_I64_TRUNC_F64_U,
            Instr::F32ConvertI32S => handler_index = HANDLER_IDX_F32_CONVERT_I32_S,
            Instr::F32ConvertI32U => handler_index = HANDLER_IDX_F32_CONVERT_I32_U,
            Instr::F32ConvertI64S => handler_index = HANDLER_IDX_F32_CONVERT_I64_S,
            Instr::F32ConvertI64U => handler_index = HANDLER_IDX_F32_CONVERT_I64_U,
            Instr::F32DemoteF64 => handler_index = HANDLER_IDX_F32_DEMOTE_F64,
            Instr::F64ConvertI32S => handler_index = HANDLER_IDX_F64_CONVERT_I32_S,
            Instr::F64ConvertI32U => handler_index = HANDLER_IDX_F64_CONVERT_I32_U,
            Instr::F64ConvertI64S => handler_index = HANDLER_IDX_F64_CONVERT_I64_S,
            Instr::F64ConvertI64U => handler_index = HANDLER_IDX_F64_CONVERT_I64_U,
            Instr::F64PromoteF32 => handler_index = HANDLER_IDX_F64_PROMOTE_F32,
            Instr::I32ReinterpretF32 => handler_index = HANDLER_IDX_I32_REINTERPRET_F32,
            Instr::I64ReinterpretF64 => handler_index = HANDLER_IDX_I64_REINTERPRET_F64,
            Instr::F32ReinterpretI32 => handler_index = HANDLER_IDX_F32_REINTERPRET_I32,
            Instr::F64ReinterpretI64 => handler_index = HANDLER_IDX_F64_REINTERPRET_I64,
            Instr::I32Extend8S => handler_index = HANDLER_IDX_I32_EXTEND8_S,
            Instr::I32Extend16S => handler_index = HANDLER_IDX_I32_EXTEND16_S,
            Instr::I64Extend8S => handler_index = HANDLER_IDX_I64_EXTEND8_S,
            Instr::I64Extend16S => handler_index = HANDLER_IDX_I64_EXTEND16_S,
            Instr::I64Extend32S => handler_index = HANDLER_IDX_I64_EXTEND32_S,
            _ => {
                println!("Warning: Unhandled instruction in preprocessing pass 1: {:?}", instr);
                handler_index = HANDLER_IDX_NOP;
            }
        }
        processed.push(ProcessedInstr { handler_index, operand });
    }

    // Phase 2 (Map Building): Build maps for End and Else targets
    let mut block_end_map: HashMap<usize, usize> = HashMap::new(); // Map block_start_pc -> end_marker_pc + 1
    let mut if_else_map: HashMap<usize, usize> = HashMap::new(); // Map if_start_pc -> else_marker_pc + 1
    let mut control_stack_for_map_building: Vec<(usize, bool)> = Vec::new(); // (processed_pc_start, is_loop)

    for (pc, instr) in processed.iter().enumerate() {
         match instr.handler_index {
            HANDLER_IDX_BLOCK | HANDLER_IDX_LOOP | HANDLER_IDX_IF => {
                control_stack_for_map_building.push((pc, instr.handler_index == HANDLER_IDX_LOOP));
            }
            HANDLER_IDX_ELSE => {
                // Find the corresponding If's start_pc on the stack
                if let Some((if_start_pc, false)) = control_stack_for_map_building.last() {
                    if_else_map.insert(*if_start_pc, pc + 1); // Map If start to Else+1
                } else {
                     return Err(RuntimeError::InvalidWasm("ElseMarker without matching If"));
                }
            }
            HANDLER_IDX_END => {
                if let Some((start_pc, _)) = control_stack_for_map_building.pop() {
                    block_end_map.insert(start_pc, pc + 1); // Map Block/Loop/If start to End+1
                } else {
                     // Allow EndMarker at the end of the function body
                     if pc == processed.len() - 1 && control_stack_for_map_building.is_empty() {
                         // This is the final EndMarker for the function, ignore it for block mapping
                     } else {
                         return Err(RuntimeError::InvalidWasm("Unmatched EndMarker"));
                     }
                }
            }
            _ => {}
        }
    }
    if !control_stack_for_map_building.is_empty() {
        return Err(RuntimeError::InvalidWasm("Unclosed control block at end of function"));
    }

    // Phase 3 (Fixup Br, BrIf, If, Else): Resolve branch targets using maps
    let mut current_control_stack_pass3: Vec<(usize, bool)> = Vec::new(); // Separate stack for this pass
    for fixup_index in 0..fixups.len() {
        // Use a mutable borrow to potentially mark fixups as done (if needed later)
        let (fixup_pc, relative_depth, is_if_false_jump, is_else_jump) = fixups[fixup_index];

        // Skip if already processed (e.g., by BrTable logic in Pass 4) or if it's a BrTable fixup
        let is_br_table_fixup = processed.get(fixup_pc).map_or(false, |instr| instr.handler_index == HANDLER_IDX_BR_TABLE);
        if relative_depth == usize::MAX || is_br_table_fixup {
             continue;
        }


        // Rebuild the control stack state *up to the point of the fixup instruction*
        current_control_stack_pass3.clear();
        for (pc, instr) in processed.iter().enumerate().take(fixup_pc + 1) { // Include the instruction needing fixup
            match instr.handler_index {
                HANDLER_IDX_BLOCK | HANDLER_IDX_LOOP | HANDLER_IDX_IF => {
                    current_control_stack_pass3.push((pc, instr.handler_index == HANDLER_IDX_LOOP));
                }
                HANDLER_IDX_END => {
                    // Only pop if the stack is not empty (handles final function EndMarker)
                    if !current_control_stack_pass3.is_empty() {
                        current_control_stack_pass3.pop();
                    }
                }
                _ => {}
            }
        }


        // Find the target block's start_pc based on relative_depth
        if current_control_stack_pass3.len() <= relative_depth {
            println!("Warning: Invalid relative depth {} for branch at pc {}", relative_depth, fixup_pc);
            // Mark fixup as done/error
            fixups[fixup_index].1 = usize::MAX;
            continue; // Skip this fixup
        }
        // Target block is 'relative_depth' levels up the stack
        let (target_start_pc, is_loop) = current_control_stack_pass3[current_control_stack_pass3.len() - 1 - relative_depth];

        let target_ip = if is_loop {
            target_start_pc // Loop branches target the Loop instruction itself
        } else {
            // Block/If branches target *after* the corresponding EndMarker
            *block_end_map.get(&target_start_pc)
                .ok_or_else(|| {
                    println!("Error: Could not find EndMarker for block starting at {}", target_start_pc);
                    RuntimeError::InvalidWasm("Missing EndMarker for branch target")
                })?
        };

        // Patch the operand in the 'processed' vector
        if let Some(instr_to_patch) = processed.get_mut(fixup_pc) {
            if is_if_false_jump { // This is an If instruction needing its false jump target
                // Target is ElseMarker+1 if it exists, otherwise EndMarker+1
                let else_target = *if_else_map.get(&target_start_pc).unwrap_or(&target_ip);
                instr_to_patch.operand = Operand::LabelIdx(else_target);
            } else if is_else_jump { // This is an ElseMarker instruction needing its jump-to-end target
                // Target is the corresponding EndMarker + 1
                instr_to_patch.operand = Operand::LabelIdx(target_ip);
            } else { // This is a Br or BrIf instruction
                // Target is Loop start or EndMarker + 1
                instr_to_patch.operand = Operand::LabelIdx(target_ip);
            }
        } else {
             println!("Warning: Could not find instruction to patch at pc {}", fixup_pc);
        }

        // Mark fixup as done
        fixups[fixup_index].1 = usize::MAX;
    }

    // Phase 4 (Fixup BrTable): Resolve BrTable targets
    let mut current_control_stack_pass4: Vec<(usize, bool)> = Vec::new(); // Separate stack for this pass
    for (pc, instr) in processed.iter_mut().enumerate() {
        // Maintain control stack state for Pass 4
        match instr.handler_index {
            HANDLER_IDX_BLOCK | HANDLER_IDX_LOOP | HANDLER_IDX_IF => {
                current_control_stack_pass4.push((pc, instr.handler_index == HANDLER_IDX_LOOP));
            }
            HANDLER_IDX_END => {
                 if !current_control_stack_pass4.is_empty() {
                    current_control_stack_pass4.pop();
                 }
            }
            _ => {}
        }

        // Process BrTable instructions
        if instr.handler_index == HANDLER_IDX_BR_TABLE && instr.operand == Operand::None {
            let mut resolved_targets = Vec::new();
            let mut resolved_default = usize::MAX;

            // Find fixup indices associated *only* with this BrTable pc
            let mut fixup_indices_for_this_br_table = fixups.iter().enumerate()
                .filter(|(_, (fp, depth, _, _))| *fp == pc && *depth != usize::MAX) // Filter by pc and not already processed
                .map(|(idx, _)| idx) // Get the original index in the fixups vector
                .collect::<Vec<_>>();

            // The last fixup entry corresponds to the default target
            if let Some(default_fixup_idx) = fixup_indices_for_this_br_table.pop() {
                 let (_, relative_depth, _, _) = fixups[default_fixup_idx]; // Use original index
                 if current_control_stack_pass4.len() <= relative_depth {
                     println!("Warning: Invalid relative depth {} for BrTable default at pc {}", relative_depth, pc);
                     resolved_default = usize::MAX; // Mark as error
                 } else {
                     let (target_start_pc, is_loop) = current_control_stack_pass4[current_control_stack_pass4.len() - 1 - relative_depth];
                     resolved_default = if is_loop { target_start_pc } else { *block_end_map.get(&target_start_pc).unwrap_or(&usize::MAX) };
                 }
                 fixups[default_fixup_idx].1 = usize::MAX; // Mark as done using original index
            } else {
                 println!("Warning: Could not find default target fixup for BrTable at pc {}", pc);
            }

            // Resolve remaining targets (in the order they appeared in the original BrTable instruction)
            for fixup_idx in fixup_indices_for_this_br_table { // Iterate through original indices
                 let (_, relative_depth, _, _) = fixups[fixup_idx]; // Use original index
                 let target_ip = if current_control_stack_pass4.len() <= relative_depth {
                     println!("Warning: Invalid relative depth {} for BrTable target at pc {}", relative_depth, pc);
                     usize::MAX // Mark as error
                 } else {
                     let (target_start_pc, is_loop) = current_control_stack_pass4[current_control_stack_pass4.len() - 1 - relative_depth];
                     if is_loop { target_start_pc } else { *block_end_map.get(&target_start_pc).unwrap_or(&usize::MAX) }
                 };
                 resolved_targets.push(target_ip);
                 fixups[fixup_idx].1 = usize::MAX; // Mark as done using original index
            }

            instr.operand = Operand::BrTable { targets: resolved_targets, default: resolved_default };
        }
    }


    // Verify all fixups were applied (optional, for debugging)
    if let Some((pc, depth, _, _)) = fixups.iter().find(|(_, d, _, _)| *d != usize::MAX) {
         println!("Warning: Unresolved fixup remaining at pc {} with depth {}", pc, depth);
         // return Err(RuntimeError::InvalidWasm("Unresolved branch target"));
    }


    Ok(processed)
}

pub struct Stacks {
    pub activation_frame_stack: Vec<FrameStack>,
}

impl Stacks {
    /// Creates a new stack setup for invoking a function.
    pub fn new(funcaddr: &FuncAddr, params: Vec<Val>) -> Result<Stacks, RuntimeError> {
        let func_inst_ref = funcaddr.borrow();
        match &*func_inst_ref {
            FuncInst::RuntimeFunc { type_, module, code } => {
                // Validate params count
                if params.len() != type_.params.len() {
                    return Err(RuntimeError::InvalidParameterCount);
                }

                // Preprocess the function body for DTC
                let processed_instrs = preprocess_instructions(&code.body.0)?;

                // Initialize locals (params + defaults)
                let mut locals = params;
                for v in code.locals.iter() {
                    for _ in 0..(v.0) {
                        locals.push(match v.1 {
                            ValueType::NumType(NumType::I32) => Val::Num(Num::I32(0)),
                            ValueType::NumType(NumType::I64) => Val::Num(Num::I64(0)),
                            ValueType::NumType(NumType::F32) => Val::Num(Num::F32(0.0)),
                            ValueType::NumType(NumType::F64) => Val::Num(Num::F64(0.0)),
                            ValueType::VecType(VecType::V128) => Val::Vec_(Vec_::V128(0)),
                            ValueType::RefType(_) => todo!("RefType local initialization"),
                        });
                    }
                }

                let initial_frame = FrameStack {
                    frame: Frame {
                        locals,
                        module: module.clone(),
                        n: type_.results.len(),
                    },
                    label_stack: vec![LabelStack {
                        label: Label { // Label might be simplified later for DTC
                            locals_num: type_.results.len(),
                        },
                        processed_instrs,
                        value_stack: vec![],
                        ip: 0, // Initialize IP to 0
                    }],
                    void: type_.results.is_empty(),
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
        }
    }

    /*
    This Function Only Handle Instruction Spanning FrameStack.
    i.e., Invoke Wasm Function, Return Function and Call Host-function.
    */
    pub fn exec_instr(&mut self) -> Result<Vec<Val>, RuntimeError> {
        // This outer loop manages frame transitions (call/return)
        while !self.activation_frame_stack.is_empty() {
            let frame_stack_idx = self.activation_frame_stack.len() - 1;
            // Store the FuncAddr if a call happens within run_dtc_loop
            let mut called_func_addr: Option<FuncAddr> = None; // Needed for Invoke

            let module_level_instr_result = {
                let current_frame_stack = &mut self.activation_frame_stack[frame_stack_idx];
                // Pass a way to signal back the called function index if needed
                current_frame_stack.run_dtc_loop(&mut called_func_addr)? // called_func_addr is now set inside run_dtc_loop via context
            };

            match module_level_instr_result {
                Ok(Some(instr)) => { // Frame transition requested (Return/Call)
                    // Need mutable borrow for potential param passing in Invoke
                    let current_frame_stack = self.activation_frame_stack.last_mut().unwrap(); // Should not panic if loop condition holds
                    // Ensure label_stack is not empty before accessing last_mut
                    if current_frame_stack.label_stack.is_empty() {
                        // This should ideally not happen if run_dtc_loop returned Ok(Some(...))
                        return Err(RuntimeError::StackError("Label stack empty during frame transition"));
                    }
                    let mut cur_label_stack = current_frame_stack.label_stack.last_mut().unwrap();

                    match instr {
                        ModuleLevelInstr::Invoke(func_addr) => {
                            match &*func_addr.borrow(){
                                FuncInst::RuntimeFunc{type_,module,code} => {
                                    // Prepare locals for the new frame
                                    let params_len = type_.params.len();
                                    // Take params from the current value stack
                                    if cur_label_stack.value_stack.len() < params_len {
                                        return Err(RuntimeError::ValueStackUnderflow);
                                    }
                                    let params = cur_label_stack.value_stack.split_off(cur_label_stack.value_stack.len() - params_len); // No mut needed

                                    // Initialize locals (params + defaults)
                                    let mut locals = params;
                                    for v in code.locals.iter(){
                                        for _ in 0..(v.0){
                                            locals.push(
                                                match v.1{
                                                    ValueType::NumType(NumType::I32) => Val::Num(Num::I32(0)),
                                                    ValueType::NumType(NumType::I64) => Val::Num(Num::I64(0)),
                                                    ValueType::NumType(NumType::F32) => Val::Num(Num::F32(0.0)),
                                                    ValueType::NumType(NumType::F64) => Val::Num(Num::F64(0.0)),
                                                    ValueType::VecType(VecType::V128) => Val::Vec_(Vec_::V128(0)),
                                                    ValueType::RefType(_) => todo!("RefType local init in Invoke"),
                                                }
                                            );
                                        }
                                    };

                                    // Preprocess the function body (or get from cache later)
                                    let processed_code = preprocess_instructions(&code.body.0)?;

                                    // Create the new frame stack
                                    let frame = FrameStack{
                                        frame: Frame{
                                            locals,
                                            module: module.clone(),
                                            n: type_.results.len()
                                        },
                                        label_stack: vec![
                                            LabelStack{
                                                label: Label{
                                                    locals_num: type_.results.len(),
                                                },
                                                processed_instrs: processed_code,
                                                value_stack: vec![],
                                                ip: 0, // Initialize IP for new frame
                                            }
                                        ],
                                        void: type_.results.is_empty(),
                                    };
                                    self.activation_frame_stack.push(frame);
                                    // Continue to the next iteration to run the new frame's loop

                                },
                                FuncInst::HostFunc{type_, host_code} => {
                                    // Handle host function call
                                    let params_len = type_.params.len();
                                    if cur_label_stack.value_stack.len() < params_len {
                                        return Err(RuntimeError::ValueStackUnderflow);
                                    }
                                    let params = cur_label_stack.value_stack.split_off(cur_label_stack.value_stack.len() - params_len);
                                    match host_code(params) {
                                        Ok(Some(result_val)) => {
                                            // Push result onto the *caller's* stack (which is current_frame_stack)
                                            cur_label_stack.value_stack.push(result_val);
                                            // Host call finished, advance IP in the *caller* frame
                                            cur_label_stack.ip += 1;
                                        }
                                        Ok(None) => {
                                            // No return value, still advance IP in the caller frame
                                            cur_label_stack.ip += 1;
                                        }
                                        Err(e) => return Err(e), // Propagate host error
                                    }
                                    // Host call doesn't push a new Wasm frame, continue in current frame loop?
                                    // The loop continues naturally after IP is advanced.
                                },
                            }
                        },
                        ModuleLevelInstr::Return =>{ // Triggered by handle_return result
                            // Pop the current frame
                            let finished_frame = self.activation_frame_stack.pop().unwrap(); // Should not panic if loop condition holds

                            // Check if this was the last frame
                            if self.activation_frame_stack.is_empty() {
                                // Return the final result from the finished frame's value stack
                                return Ok(finished_frame.label_stack.last().map_or(vec![], |ls| ls.value_stack.clone()));
                            }

                            // Get potential return value(s) from the finished frame's label stack
                            let finished_label_stack = finished_frame.label_stack.last().ok_or(RuntimeError::StackError("Finished frame has no label stack"))?;
                            let return_values = finished_label_stack.value_stack.clone(); // Clone return values

                            if finished_frame.frame.n != return_values.len() {
                                // This might indicate an issue, but spec allows dropping excess values?
                                // Let's proceed but maybe add a warning later.
                                // return Err(RuntimeError::InvalidReturnValueCount);
                            }

                            // Push return value(s) onto caller's stack
                            if let Some(caller_frame) = self.activation_frame_stack.last_mut() {
                                 if let Some(caller_label_stack) = caller_frame.label_stack.last_mut() {
                                     caller_label_stack.value_stack.extend(return_values);
                                     // Caller's IP was already advanced by run_dtc_loop before returning the Return signal
                                 } else {
                                     return Err(RuntimeError::StackError("Caller label stack empty during return"));
                                 }
                            }
                            // Continue execution in the caller frame in the next iteration
                        }
                    }
                },
                Ok(None) => { // Frame completed normally (implicit return)
                     let finished_frame = self.activation_frame_stack.pop().unwrap(); // Should not panic

                     // Check if this was the last frame
                     if self.activation_frame_stack.is_empty() {
                         // Return the final result from the finished frame's value stack
                         return Ok(finished_frame.label_stack.last().map_or(vec![], |ls| ls.value_stack.clone()));
                     }

                     // Handle potential return value(s) based on finished_frame.frame.n
                     let finished_label_stack = finished_frame.label_stack.last().ok_or(RuntimeError::StackError("Finished frame has no label stack on implicit return"))?;
                     let return_values = finished_label_stack.value_stack.clone();

                     if finished_frame.frame.n != return_values.len() {
                         // return Err(RuntimeError::InvalidReturnValueCount);
                     }

                     if let Some(caller_frame) = self.activation_frame_stack.last_mut() {
                          if let Some(caller_label_stack) = caller_frame.label_stack.last_mut() {
                             caller_label_stack.value_stack.extend(return_values);
                             // Caller's IP was already advanced by run_dtc_loop before reaching the end
                         } else {
                             return Err(RuntimeError::StackError("Caller label stack empty during implicit return"));
                         }
                     }
                     // Continue execution in the caller frame
                },
                Err(e) => { // Propagate runtime errors
                    return Err(e);
                }
            }
        }
        // Should only be reached if the initial stack was empty or after the last frame is popped.
    Ok(vec![])
    }
}

// Frame represents the runtime information for a function activation.
// Note: Using wasmparser::Frame directly in ExecutionContext now.
// This Frame struct might be needed for FrameStack still.
pub struct Frame{
    pub locals: Vec<Val>,
    pub module: Weak<ModuleInst>,
    pub n: usize, // Number of expected return values
}

pub struct FrameStack {
    pub frame: Frame,
    pub label_stack: Vec<LabelStack>, // Keep for block/loop/if context, maybe simplify later
    pub void: bool,
}

impl FrameStack{
/// Runs the Direct Threaded Code loop for this frame.
    /// Returns Ok(Some(ModuleLevelInstr)) if a frame transition (Call/Return) is needed,
    /// Ok(None) if the frame execution completes normally (reaches end),
    /// or Err on runtime error.
    fn run_dtc_loop(&mut self, called_func_addr_out: &mut Option<FuncAddr>) -> Result<Result<Option<ModuleLevelInstr>, RuntimeError>, RuntimeError> { // Outer Result for panic safety
        // Get mutable access to the current label stack
        let current_label_stack = self.label_stack.last_mut().ok_or(RuntimeError::StackError("Label stack empty"))?;
        let processed_code = &current_label_stack.processed_instrs; // Immutable borrow

        if processed_code.is_empty() {
             return Ok(Ok(None));
        }

        // Use the ip stored in the LabelStack
        let mut ip = current_label_stack.ip;

        loop {
            if ip >= processed_code.len() {
                // Reached end of code block naturally
                // Update the stored IP before breaking/returning
                current_label_stack.ip = ip;
                break;
            }

            // Clone instruction to avoid borrowing issues when context is passed mutably
            let instruction = processed_code[ip].clone(); // Clone from immutable borrow

            // Need mutable access to value_stack within the LabelStack
            // We already have `current_label_stack` as mutable, so we can borrow its fields mutably.
            let handler_fn = HANDLER_TABLE.get(instruction.handler_index)
                                        .ok_or(RuntimeError::InvalidHandlerIndex)?;

            // Pass mutable references to frame and value_stack
            let mut context = ExecutionContext {
                frame: &mut self.frame,
                value_stack: &mut current_label_stack.value_stack, // Mutable borrow here
                ip, // Pass current ip
            };

            // --- Execute Handler ---
            let result = handler_fn(&mut context, instruction.operand.clone()); // Clone operand
            // --- End Execute Handler ---

            match result {
                Ok(next_ip) => {
                    if next_ip == usize::MAX { // Sentinel for Return
                        // Update IP *before* returning signal
                        current_label_stack.ip = ip + 1; // Point to instruction *after* return
                        return Ok(Ok(Some(ModuleLevelInstr::Return)));
                    } else if next_ip == usize::MAX - 1 { // Sentinel for Call
                         // Update IP *before* returning signal
                         current_label_stack.ip = ip + 1; // Point to instruction *after* call
                         // Extract FuncIdx from the *original* instruction operand
                         if let Operand::FuncIdx(func_idx) = &instruction.operand { // Use cloned instruction
                             let instance = self.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
                             let func_addr = instance.func_addrs.get_by_idx(func_idx.clone()).clone();
                             *called_func_addr_out = Some(func_addr.clone()); // Store the called address
                             return Ok(Ok(Some(ModuleLevelInstr::Invoke(func_addr))));
                         } else {
                             return Ok(Err(RuntimeError::ExecutionFailed("Invalid operand found for call signal")));
                         }
                    }
                    // TODO: Handle CallIndirect similarly
                    ip = next_ip; // Continue execution at the address returned by handler (e.g., branch target)
                }
                Err(e) => {
                    // Update IP before propagating error
                    current_label_stack.ip = ip;
                    return Ok(Err(e));
                } // Wrap runtime error in Ok for panic safety layer
            }
        }

        // If loop finishes normally (reaches end of code), it's like an implicit return.
        // The outer exec_instr loop will handle popping the frame.
        // IP is already updated before breaking.
        Ok(Ok(None))
    }
}

#[derive(Clone)]
pub struct Label {
    // pub continue_: Vec<Instr>, // Not needed for DTC execution loop
    pub locals_num: usize, // Still needed for Br value transfer?
}

pub struct LabelStack {
    pub label: Label,
    pub processed_instrs: Vec<ProcessedInstr>, // DTC instruction format
    pub value_stack: Vec<Val>,
    pub ip: usize, // Instruction pointer for this label/frame level
}


// These enums might be simplified or removed later if FrameStack::run_dtc_loop handles everything
#[derive(Clone)]
pub enum FrameLevelInstr{
    Label(Label, Vec<Instr>),
    Br(StructureLabelIdx), // Use the imported alias
    EndLabel,
    Invoke(FuncAddr),
    Return
}

// #[derive(Clone)] // Moved ModuleLevelInstr definition higher up
// pub enum ModuleLevelInstr{
//     Invoke(FuncAddr),
//     Return,
// }

// This enum might become obsolete or change significantly with DTC
#[derive(Clone)]
pub enum AdminInstr {
    Trap,
    Instr(Instr),
    Invoke(FuncAddr),
    Label(Label, Vec<Instr>),
    Br(StructureLabelIdx), // Use the imported alias
    Return,
    Ref(FuncAddr),
    RefExtern(ExternAddr),
}


// --- Instruction Handlers ---

// Helper macro for binary operations
macro_rules! binop {
    // Version for when result type might differ
    ($ctx:ident, $operand_type:ident, $op_trait:ident, $op_method:ident, $result_type:ident) => {
        {
            // Ensure rhs is popped first according to Wasm spec (val2 then val1)
            let rhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let (lhs, rhs) = match (lhs_val, rhs_val) {
                (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l, r),
                _ => return Err(RuntimeError::TypeMismatch),
            };
            // Use the trait method
            $ctx.value_stack.push(Val::Num(Num::$result_type($op_trait::$op_method(lhs, rhs))));
            Ok($ctx.ip + 1)
        }
    };
     // Version when result type is the same as operand type
     ($ctx:ident, $operand_type:ident, $op_trait:ident, $op_method:ident) => {
        binop!($ctx, $operand_type, $op_trait, $op_method, $operand_type)
     };
}

// Helper macro for binary operations with wrapping
macro_rules! binop_wrapping {
    ($ctx:ident, $operand_type:ident, $op_method:ident, $result_type:ident) => {
        {
            let rhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            // E0599 Fix: Match Val::Num and then extract the primitive value
             let (lhs, rhs) = match (lhs_val, rhs_val) {
                (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l, r),
                _ => return Err(RuntimeError::TypeMismatch), // Or appropriate error
            };
            $ctx.value_stack.push(Val::Num(Num::$result_type(lhs.$op_method(rhs))));
            Ok($ctx.ip + 1)
        }
    };
     ($ctx:ident, $operand_type:ident, $op_method:ident) => { // Result type same as operand type
        binop_wrapping!($ctx, $operand_type, $op_method, $operand_type)
     };
}

// Removed unused macro: unop
// macro_rules! unop { ... }

// Helper macro for comparison operations
macro_rules! cmpop {
    ($ctx:ident, $operand_type:ident, $op:tt) => {
        {
            let rhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let (lhs, rhs) = match (lhs_val, rhs_val) {
                 (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l, r),
                 _ => return Err(RuntimeError::TypeMismatch),
            };
            $ctx.value_stack.push(Val::Num(Num::I32((lhs $op rhs) as i32)));
            Ok($ctx.ip + 1)
        }
    };
     ($ctx:ident, $operand_type:ident, $op:tt, $cast_type:ty) => { // For comparing unsigned
        {
            let rhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let (lhs, rhs) = match (lhs_val, rhs_val) {
                 (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l as $cast_type, r as $cast_type), // Cast here
                 _ => return Err(RuntimeError::TypeMismatch),
            };
            $ctx.value_stack.push(Val::Num(Num::I32((lhs $op rhs) as i32)));
            Ok($ctx.ip + 1)
        }
    };
}

fn handle_unreachable(_ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    Err(RuntimeError::Unreachable)
}

fn handle_nop(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    Ok(ctx.ip + 1)
}

fn handle_block(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    // Block itself does nothing at runtime in this model, control flow handled by Br/End/If
    Ok(ctx.ip + 1)
}

fn handle_loop(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    // Loop itself does nothing at runtime, Br targets the start (stored in operand)
    Ok(ctx.ip + 1)
}

fn handle_if(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    // Pops condition, jumps to else_ip or end_ip based on condition
    let cond = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    if let Operand::LabelIdx(target_ip) = operand { // If operand is patched LabelIdx
         if target_ip == usize::MAX { return Err(RuntimeError::ExecutionFailed("Branch fixup not done for If")); }
        if cond == 0 {
            // Jump to else or end
            Ok(target_ip)
        } else {
            // Continue to the 'then' block
            Ok(ctx.ip + 1)
        }
    } else {
         // This case should ideally not happen after fixup
         Err(RuntimeError::InvalidOperand)
    }
}

fn handle_else(_ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> { // Renamed ctx
    // Unconditional jump to the end of the If block
     if let Operand::LabelIdx(target_ip) = operand {
        if target_ip == usize::MAX { return Err(RuntimeError::ExecutionFailed("Branch fixup not done for Else")); }
        Ok(target_ip)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}


fn handle_end(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    // End simply proceeds to the next instruction after the block/if/loop
    Ok(ctx.ip + 1)
}

fn handle_br(_ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> { // Renamed ctx
    // Jumps to the target IP stored in the operand (must be resolved during preprocessing)
    if let Operand::LabelIdx(target_ip) = operand {
        if target_ip == usize::MAX { return Err(RuntimeError::ExecutionFailed("Branch fixup not done for Br")); }
        // TODO: Handle value transfer for branches (polymorphic stack)
        Ok(target_ip)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_br_if(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    let cond = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    if cond != 0 {
        // Condition is true, perform the branch
        if let Operand::LabelIdx(target_ip) = operand {
             if target_ip == usize::MAX { return Err(RuntimeError::ExecutionFailed("Branch fixup not done for BrIf")); }
             // TODO: Handle value transfer
            Ok(target_ip)
        } else {
            Err(RuntimeError::InvalidOperand)
        }
    } else {
        // Condition is false, continue to the next instruction
        Ok(ctx.ip + 1)
    }
}

// Signals function call to the outer loop
fn handle_call(_ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::FuncIdx(_) = operand {
         // Signal call using sentinel value. The actual FuncAddr resolution
         // happens in the run_dtc_loop based on the operand.
         Ok(usize::MAX - 1) // Use sentinel value for Call
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}


fn handle_i32_const(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::I32(val) = operand {
        ctx.value_stack.push(Val::Num(Num::I32(val)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_const(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::I64(val) = operand {
        ctx.value_stack.push(Val::Num(Num::I64(val)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f32_const(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::F32(val) = operand {
        ctx.value_stack.push(Val::Num(Num::F32(val)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f64_const(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::F64(val) = operand {
        ctx.value_stack.push(Val::Num(Num::F64(val)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

// --- Comparison Handlers ---
fn handle_i32_eqz(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    ctx.value_stack.push(Val::Num(Num::I32((val == 0) as i32)));
    Ok(ctx.ip + 1)
}
fn handle_i32_eq(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I32, ==) }
fn handle_i32_ne(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I32, !=) }
fn handle_i32_lt_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I32, <) }
fn handle_i32_lt_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I32, <, u32) }
fn handle_i32_gt_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I32, >) }
fn handle_i32_gt_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I32, >, u32) }
fn handle_i32_le_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I32, <=) }
fn handle_i32_le_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I32, <=, u32) }
fn handle_i32_ge_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I32, >=) }
fn handle_i32_ge_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I32, >=, u32) }

fn handle_i64_eqz(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    ctx.value_stack.push(Val::Num(Num::I32((val == 0) as i32)));
    Ok(ctx.ip + 1)
}
fn handle_i64_eq(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I64, ==) }
fn handle_i64_ne(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I64, !=) }
fn handle_i64_lt_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I64, <) }
fn handle_i64_lt_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I64, <, u64) }
fn handle_i64_gt_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I64, >) }
fn handle_i64_gt_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I64, >, u64) }
fn handle_i64_le_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I64, <=) }
fn handle_i64_le_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I64, <=, u64) }
fn handle_i64_ge_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I64, >=) }
fn handle_i64_ge_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, I64, >=, u64) }

fn handle_f32_eq(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, F32, ==) }
fn handle_f32_ne(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, F32, !=) }
fn handle_f32_lt(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, F32, <) }
fn handle_f32_gt(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, F32, >) }
fn handle_f32_le(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, F32, <=) }
fn handle_f32_ge(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, F32, >=) }

fn handle_f64_eq(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, F64, ==) }
fn handle_f64_ne(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, F64, !=) }
fn handle_f64_lt(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, F64, <) }
fn handle_f64_gt(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, F64, >) }
fn handle_f64_le(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, F64, <=) }
fn handle_f64_ge(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { cmpop!(ctx, F64, >=) }

// --- Arithmetic Handlers ---
fn handle_i32_clz(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let result: i32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "i32.clz", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.leading_zeros() as i32; }
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(ctx.ip + 1)
}
fn handle_i32_ctz(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let result: i32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "i32.ctz", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.trailing_zeros() as i32; }
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(ctx.ip + 1)
}
fn handle_i32_popcnt(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let result: i32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "i32.popcnt", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.count_ones() as i32; }
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(ctx.ip + 1)
}
fn handle_i32_add(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop_wrapping!(ctx, I32, wrapping_add) }
fn handle_i32_sub(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop_wrapping!(ctx, I32, wrapping_sub) }
fn handle_i32_mul(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop_wrapping!(ctx, I32, wrapping_mul) }
fn handle_i32_div_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let result: i32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i32.div_s", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.checked_div(rhs).ok_or(RuntimeError::ZeroDivideError)?; } // TODO: Check overflow MIN / -1
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(ctx.ip + 1)
}
fn handle_i32_div_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32() as u32;
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32() as u32;
    let result: u32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i32.div_u", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.checked_div(rhs).ok_or(RuntimeError::ZeroDivideError)?; }
    ctx.value_stack.push(Val::Num(Num::I32(result as i32)));
    Ok(ctx.ip + 1)
}
fn handle_i32_rem_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let result: i32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i32.rem_s", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    {
        result = lhs.overflowing_rem(rhs).0;
    }
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(ctx.ip + 1)
}
fn handle_i32_rem_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32() as u32;
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32() as u32;
    let result: u32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i32.rem_u", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.checked_rem(rhs).ok_or(RuntimeError::ZeroDivideError)?; }
    ctx.value_stack.push(Val::Num(Num::I32(result as i32)));
    Ok(ctx.ip + 1)
}
fn handle_i32_and(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop!(ctx, I32, BitAnd, bitand) }
fn handle_i32_or(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop!(ctx, I32, BitOr, bitor) }
fn handle_i32_xor(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop!(ctx, I32, BitXor, bitxor) }
fn handle_i32_shl(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let result: i32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i32.shl", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.wrapping_shl(rhs as u32); } // Use wrapping_shl for Rust equivalent
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(ctx.ip + 1)
}
fn handle_i32_shr_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let result: i32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i32.shr_s", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.wrapping_shr(rhs as u32); } // Use wrapping_shr
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(ctx.ip + 1)
}
fn handle_i32_shr_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32() as u32; // Shift amount is also treated as u32 by Wasm spec
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32() as u32;
    let result: u32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i32.shr_u", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.wrapping_shr(rhs); } // Use wrapping_shr
    ctx.value_stack.push(Val::Num(Num::I32(result as i32)));
    Ok(ctx.ip + 1)
}
fn handle_i32_rotl(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let result: i32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i32.rotl", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.rotate_left(rhs as u32); }
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(ctx.ip + 1)
}
fn handle_i32_rotr(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let result: i32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i32.rotr", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.rotate_right(rhs as u32); }
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(ctx.ip + 1)
}

fn handle_i64_clz(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let result: i64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "i64.clz", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.leading_zeros() as i64; }
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(ctx.ip + 1)
}
fn handle_i64_ctz(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let result: i64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "i64.ctz", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.trailing_zeros() as i64; }
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(ctx.ip + 1)
}
fn handle_i64_popcnt(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let result: i64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "i64.popcnt", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.count_ones() as i64; }
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(ctx.ip + 1)
}
fn handle_i64_add(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop_wrapping!(ctx, I64, wrapping_add) }
fn handle_i64_sub(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop_wrapping!(ctx, I64, wrapping_sub) }
fn handle_i64_mul(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop_wrapping!(ctx, I64, wrapping_mul) }
fn handle_i64_div_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let result: i64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i64.div_s", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.checked_div(rhs).ok_or(RuntimeError::ZeroDivideError)?; } // TODO: Check overflow MIN / -1
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(ctx.ip + 1)
}
fn handle_i64_div_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64() as u64;
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64() as u64;
    let result: u64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i64.div_u", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.checked_div(rhs).ok_or(RuntimeError::ZeroDivideError)?; }
    ctx.value_stack.push(Val::Num(Num::I64(result as i64)));
    Ok(ctx.ip + 1)
}
fn handle_i64_rem_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let result: i64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i64.rem_s", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.checked_rem(rhs).ok_or(RuntimeError::ZeroDivideError)?; }
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(ctx.ip + 1)
}
fn handle_i64_rem_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64() as u64;
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64() as u64;
    let result: u64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i64.rem_u", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.checked_rem(rhs).ok_or(RuntimeError::ZeroDivideError)?; }
    ctx.value_stack.push(Val::Num(Num::I64(result as i64)));
    Ok(ctx.ip + 1)
}
fn handle_i64_and(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop!(ctx, I64, BitAnd, bitand) }
fn handle_i64_or(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop!(ctx, I64, BitOr, bitor) }
fn handle_i64_xor(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop!(ctx, I64, BitXor, bitxor) }
fn handle_i64_shl(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64(); // Wasm uses i64 for shift amount
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let result: i64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i64.shl", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.wrapping_shl(rhs as u32); } // Shift amount is modulo 64, cast to u32 is safe
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(ctx.ip + 1)
}
fn handle_i64_shr_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let result: i64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i64.shr_s", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.wrapping_shr(rhs as u32); }
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(ctx.ip + 1)
}
fn handle_i64_shr_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64() as u64; // Wasm uses i64 for shift amount, but spec treats it modulo 64
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64() as u64; // Treat lhs as unsigned
    let result: u64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i64.shr_u", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.wrapping_shr(rhs as u32); }
    ctx.value_stack.push(Val::Num(Num::I64(result as i64)));
    Ok(ctx.ip + 1)
}
fn handle_i64_rotl(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let result: i64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i64.rotl", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.rotate_left(rhs as u32); }
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(ctx.ip + 1)
}
fn handle_i64_rotr(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let result: i64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "i64.rotr", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.rotate_right(rhs as u32); }
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(ctx.ip + 1)
}

fn handle_f32_abs(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let result: f32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "f32.abs", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.abs(); }
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(ctx.ip + 1)
}
fn handle_f32_neg(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let result: f32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "f32.neg", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = -x; }
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(ctx.ip + 1)
}
fn handle_f32_ceil(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let result: f32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "f32.ceil", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.ceil(); }
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(ctx.ip + 1)
}
fn handle_f32_floor(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let result: f32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "f32.floor", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.floor(); }
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(ctx.ip + 1)
}
fn handle_f32_trunc(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let result: f32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "f32.trunc", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.trunc(); }
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(ctx.ip + 1)
}
fn handle_f32_nearest(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let result: f32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "f32.nearest", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { // Rust's round ties half to even, Wasm ties half away from zero
      let y = x.fract();
      result = if y == 0.5 { x.floor() + 1.0 } else if y == -0.5 { x.ceil() - 1.0 } else { x.round() };
    }
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(ctx.ip + 1)
}
fn handle_f32_sqrt(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let result: f32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "f32.sqrt", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.sqrt(); }
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(ctx.ip + 1)
}
fn handle_f32_add(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop!(ctx, F32, Add, add) }
fn handle_f32_sub(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop!(ctx, F32, Sub, sub) }
fn handle_f32_mul(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop!(ctx, F32, Mul, mul) }
fn handle_f32_div(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop!(ctx, F32, Div, div) }
fn handle_f32_min(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let result: f32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "f32.min", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.min(rhs); } // Wasm min/max behavior matches Rust for non-NaNs
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(ctx.ip + 1)
}
fn handle_f32_max(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let result: f32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "f32.max", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.max(rhs); }
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(ctx.ip + 1)
}
fn handle_f32_copysign(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let result: f32;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "local.get {1}", "f32.copysign", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = lhs.copysign(rhs); }
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(ctx.ip + 1)
}

fn handle_f64_abs(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let result: f64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "f64.abs", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.abs(); }
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(ctx.ip + 1)
}
fn handle_f64_neg(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let result: f64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "f64.neg", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = -x; }
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(ctx.ip + 1)
}
fn handle_f64_ceil(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let result: f64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "f64.ceil", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.ceil(); }
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(ctx.ip + 1)
}
fn handle_f64_floor(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let result: f64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "f64.floor", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.floor(); }
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(ctx.ip + 1)
}
fn handle_f64_trunc(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let result: f64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "f64.trunc", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { result = x.trunc(); }
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(ctx.ip + 1)
}
fn handle_f64_nearest(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let result: f64;
    #[cfg(target_arch = "wasm32")]
    unsafe { asm!("local.get {0}", "f64.nearest", "local.set {1}", in(local) x, out(local) result); }
    #[cfg(not(target_arch = "wasm32"))]
    { // Rust's round ties half to even, Wasm ties half away from zero
      let y = x.fract();
      result = if y == 0.5 { x.floor() + 1.0 } else if y == -0.5 { x.ceil() - 1.0 } else { x.round() };
    }
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(ctx.ip + 1)
}

// --- Extend Handlers ---
fn handle_i32_extend8_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let extended = (val as i8) as i32; // Cast to i8 performs sign extension
    ctx.value_stack.push(Val::Num(Num::I32(extended)));
    Ok(ctx.ip + 1)
}

fn handle_i32_extend16_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let extended = (val as i16) as i32; // Cast to i16 performs sign extension
    ctx.value_stack.push(Val::Num(Num::I32(extended)));
    Ok(ctx.ip + 1)
}

fn handle_i64_extend8_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let extended = (val as i8) as i64; // Cast to i8 performs sign extension
    ctx.value_stack.push(Val::Num(Num::I64(extended)));
    Ok(ctx.ip + 1)
}

fn handle_i64_extend16_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let extended = (val as i16) as i64; // Cast to i16 performs sign extension
    ctx.value_stack.push(Val::Num(Num::I64(extended)));
    Ok(ctx.ip + 1)
}

fn handle_i64_extend32_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32(); // Changed to to_i32()
    let extended = val as i64; // Cast i32 to i64 performs sign extension
    ctx.value_stack.push(Val::Num(Num::I64(extended)));
    Ok(ctx.ip + 1)
}
// --- End Extend Handlers ---

// --- Conversion Handlers (Simple) ---
fn handle_i32_wrap_i64(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let result = val as i32; // Wrapping conversion
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(ctx.ip + 1)
}

fn handle_i64_extend_i32_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let extended = (val as u32) as i64; // Cast to u32 first, then to i64 for zero-extension
    ctx.value_stack.push(Val::Num(Num::I64(extended)));
    Ok(ctx.ip + 1)
}

fn handle_f64_promote_f32(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let promoted = val as f64;
    ctx.value_stack.push(Val::Num(Num::F64(promoted)));
    Ok(ctx.ip + 1)
}

fn handle_f32_demote_f64(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let demoted = val as f32;
    ctx.value_stack.push(Val::Num(Num::F32(demoted)));
    Ok(ctx.ip + 1)
}
// --- End Conversion Handlers (Simple) ---

// --- Truncation Handlers (Float to Int) ---
fn handle_i32_trunc_f32_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    if val.is_nan() { return Err(RuntimeError::InvalidConversionToInt); }
    let truncated = val.trunc();
    if !(truncated >= (i32::MIN as f32) && truncated < ((i32::MAX as u32 + 1) as f32)) { // Check range carefully
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.value_stack.push(Val::Num(Num::I32(truncated as i32)));
    Ok(ctx.ip + 1)
}

fn handle_i32_trunc_f32_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    if val.is_nan() { return Err(RuntimeError::InvalidConversionToInt); }
    let truncated = val.trunc();
    // Check range: 0 <= truncated < 2^32
    if !(truncated >= 0.0 && truncated < 4294967296.0) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.value_stack.push(Val::Num(Num::I32(truncated as u32 as i32)));
    Ok(ctx.ip + 1)
}

fn handle_i32_trunc_f64_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    if val.is_nan() { return Err(RuntimeError::InvalidConversionToInt); }
    let truncated = val.trunc();
    // Check range: i32::MIN <= truncated < 2^31
    if !(truncated >= (i32::MIN as f64) && truncated < 2147483648.0) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.value_stack.push(Val::Num(Num::I32(truncated as i32)));
    Ok(ctx.ip + 1)
}

fn handle_i32_trunc_f64_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    if val.is_nan() { return Err(RuntimeError::InvalidConversionToInt); }
    let truncated = val.trunc();
    // Check range: 0 <= truncated < 2^32
    if !(truncated >= 0.0 && truncated < 4294967296.0) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.value_stack.push(Val::Num(Num::I32(truncated as u32 as i32)));
    Ok(ctx.ip + 1)
}

fn handle_i64_trunc_f32_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    if val.is_nan() { return Err(RuntimeError::InvalidConversionToInt); }
    let truncated = val.trunc();
    // Check range: i64::MIN <= truncated < 2^63
    if !(truncated >= (i64::MIN as f32) && truncated < (i64::MAX as f32)) { // Approximate check for f32 range
         if truncated.is_finite() && truncated.is_sign_positive() { // Handle large positive values near i64::MAX
             if truncated >= 9223372036854775808.0 { return Err(RuntimeError::IntegerOverflow); }
         } else if truncated.is_finite() && truncated.is_sign_negative() { // Handle large negative values near i64::MIN
             if truncated < -9223372036854775808.0 { return Err(RuntimeError::IntegerOverflow); }
         } else { // Inf or other cases
             return Err(RuntimeError::IntegerOverflow);
         }
    }
    ctx.value_stack.push(Val::Num(Num::I64(truncated as i64)));
    Ok(ctx.ip + 1)
}

fn handle_i64_trunc_f32_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    if val.is_nan() { return Err(RuntimeError::InvalidConversionToInt); }
    let truncated = val.trunc();
    // Check range: 0 <= truncated < 2^64
    if !(truncated >= 0.0 && truncated < 18446744073709551616.0) { // 2.0f32.powi(64)
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.value_stack.push(Val::Num(Num::I64(truncated as u64 as i64)));
    Ok(ctx.ip + 1)
}

fn handle_i64_trunc_f64_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    if val.is_nan() { return Err(RuntimeError::InvalidConversionToInt); }
    let truncated = val.trunc();
    // Check range: i64::MIN <= truncated < 2^63
    if !(truncated >= (i64::MIN as f64) && truncated < 9223372036854775808.0) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.value_stack.push(Val::Num(Num::I64(truncated as i64)));
    Ok(ctx.ip + 1)
}

fn handle_i64_trunc_f64_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    if val.is_nan() { return Err(RuntimeError::InvalidConversionToInt); }
    let truncated = val.trunc();
    // Check range: 0 <= truncated < 2^64
    if !(truncated >= 0.0 && truncated < 18446744073709551616.0) { // 2.0f64.powi(64)
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.value_stack.push(Val::Num(Num::I64(truncated as u64 as i64)));
    Ok(ctx.ip + 1)
}
// --- End Truncation Handlers ---


fn handle_unimplemented(_ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    println!("Error: Unimplemented instruction reached with operand: {:?}", operand);
    Err(RuntimeError::UnimplementedInstruction) // Return error for unimplemented
}

fn handle_br_table(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    let i = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    if let Operand::BrTable { targets, default } = operand {
        // Get the target index, using default if index is out of bounds
        let target_ip = if let Some(target) = targets.get(i as usize) {
            *target
        } else {
            default
        };

        // Check if the target IP was resolved during preprocessing
        if target_ip == usize::MAX {
            return Err(RuntimeError::ExecutionFailed("Branch fixup not done for BrTable target"));
        }
        // TODO: Handle value transfer for branches if needed based on label arity
        Ok(target_ip)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_call_indirect(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::TypeIdx(expected_type_idx) = operand {
        let table_idx = 0; // Assuming table index 0 for now
        let i = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32(); // Function index from stack

        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        let table_addr = module_inst.table_addrs.get(table_idx).ok_or(RuntimeError::TableNotFound)?; // Get table address

        // Use the get method which returns Option<FuncAddr>
        let func_ref_option = table_addr.get(i as usize); // Corrected: Use get() and usize

        if let Some(func_addr) = func_ref_option { // Check the Option
             // Check function type
             let actual_type = func_addr.func_type();
             let expected_type = module_inst.types.get_by_idx(expected_type_idx);

            if actual_type != *expected_type {
                 return Err(RuntimeError::IndirectCallTypeMismatch);
            }
            // Signal call using sentinel value.
             Ok(usize::MAX - 1) // Sentinel for Call
        } else {
            // Handle the case where the table element is None or index is out of bounds
            // TODO: Distinguish between UninitializedElement and TableOutOfBounds if necessary
            Err(RuntimeError::UninitializedElement)
        }
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_select(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let cond = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let val2 = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val1 = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;

    // Type check (optional but good practice, though validation should catch this)
    // if val1.value_type() != val2.value_type() {
    //     return Err(RuntimeError::TypeMismatch);
    // }

    if cond != 0 {
        ctx.value_stack.push(val1);
    } else {
        ctx.value_stack.push(val2);
    }
    Ok(ctx.ip + 1)
}

// Signals function return to the outer loop
fn handle_return(_ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    Ok(usize::MAX) // Sentinel for Return
}

fn handle_drop(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let _ = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    Ok(ctx.ip + 1)
}

fn handle_local_get(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::LocalIdx(idx) = operand {
        let index = idx.0 as usize;
        if index >= ctx.frame.locals.len() {
            return Err(RuntimeError::LocalIndexOutOfBounds);
        }
        let val = ctx.frame.locals[index].clone();
        ctx.value_stack.push(val);
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_local_set(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::LocalIdx(idx) = operand {
        let index = idx.0 as usize;
        if index >= ctx.frame.locals.len() {
            return Err(RuntimeError::LocalIndexOutOfBounds);
        }
        let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        // TODO: Check type compatibility if needed? Wasm validation should handle this.
        ctx.frame.locals[index] = val;
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_local_tee(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::LocalIdx(idx) = operand {
        let index = idx.0 as usize;
        if index >= ctx.frame.locals.len() {
            return Err(RuntimeError::LocalIndexOutOfBounds);
        }
        let val = ctx.value_stack.last().ok_or(RuntimeError::ValueStackUnderflow)?.clone();
        // TODO: Check type compatibility if needed? Wasm validation should handle this.
        ctx.frame.locals[index] = val;
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_global_get(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::GlobalIdx(idx) = operand {
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        // Assuming GetInstanceByIdx trait is implemented for the Vec<GlobalAddr> or similar structure
        let global_addr = module_inst.global_addrs.get_by_idx(idx).clone(); // Clone Rc<RefCell<GlobalInst>>
        ctx.value_stack.push(global_addr.get()); // get() likely returns Val
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_global_set(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::GlobalIdx(idx) = operand {
        let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        let global_addr = module_inst.global_addrs.get_by_idx(idx).clone();
        global_addr.set(val)?; // set() likely returns Result<(), RuntimeError> for mutability checks
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound); // Or appropriate error like NoMemoryInstance
        }
        let mem_addr = &module_inst.mem_addrs[0]; // Assuming memidx 0
        // Assuming mem_addr is Rc<RefCell<MemInst>> and MemInst has load/store methods
        // Also assuming MemAddr has a method like `load` similar to stack.rs
        let val = mem_addr.load::<i32>(&arg, ptr)?;
        ctx.value_stack.push(Val::Num(Num::I32(val)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32(); // Address is i32
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0]; // Assuming memidx 0
        // Load i64 value from memory
        let val = mem_addr.load::<i64>(&arg, ptr)?;
        ctx.value_stack.push(Val::Num(Num::I64(val)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f32_load(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32(); // Address is i32
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0]; // Assuming memidx 0
        // Load f32 value from memory
        let val = mem_addr.load::<f32>(&arg, ptr)?;
        ctx.value_stack.push(Val::Num(Num::F32(val)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f64_load(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32(); // Address is i32
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0]; // Assuming memidx 0
        // Load f64 value from memory
        let val = mem_addr.load::<f64>(&arg, ptr)?;
        ctx.value_stack.push(Val::Num(Num::F64(val)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load8_s(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i8 = mem_addr.load::<i8>(&arg, ptr)?;
        let val_i32 = val_i8 as i32; // Sign extension
        ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load8_u(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u8 = mem_addr.load::<u8>(&arg, ptr)?;
        let val_i32 = val_u8 as i32; // Zero extension
        ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load16_s(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i16 = mem_addr.load::<i16>(&arg, ptr)?;
        let val_i32 = val_i16 as i32; // Sign extension
        ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load16_u(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u16 = mem_addr.load::<u16>(&arg, ptr)?;
        let val_i32 = val_u16 as i32; // Zero extension
        ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load8_s(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i8 = mem_addr.load::<i8>(&arg, ptr)?;
        let val_i64 = val_i8 as i64; // Sign extension
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load8_u(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u8 = mem_addr.load::<u8>(&arg, ptr)?;
        let val_i64 = val_u8 as i64; // Zero extension
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load16_s(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i16 = mem_addr.load::<i16>(&arg, ptr)?;
        let val_i64 = val_i16 as i64; // Sign extension
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load16_u(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u16 = mem_addr.load::<u16>(&arg, ptr)?;
        let val_i64 = val_u16 as i64; // Zero extension
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load32_s(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i32 = mem_addr.load::<i32>(&arg, ptr)?;
        let val_i64 = val_i32 as i64; // Sign extension
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load32_u(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u32 = mem_addr.load::<u32>(&arg, ptr)?;
        let val_i64 = val_u32 as i64; // Zero extension
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_store(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        // Wasm spec pops value first, then address
        let data = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound); // Or appropriate error like NoMemoryInstance
        }
        let mem_addr = &module_inst.mem_addrs[0]; // Assuming memidx 0
        // Assuming mem_addr is Rc<RefCell<MemInst>> and MemInst has load/store methods
        // Also assuming MemAddr has a method like `store` similar to stack.rs
        mem_addr.store::<i32>(&arg, ptr, data)?;
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_store(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let data = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i64>(&arg, ptr, data)?;
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f32_store(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let data = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<f32>(&arg, ptr, data)?;
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f64_store(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let data = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<f64>(&arg, ptr, data)?;
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_store8(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let data = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        // Store lower 8 bits
        mem_addr.store::<i8>(&arg, ptr, data as i8)?;
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_store16(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let data = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        // Store lower 16 bits
        mem_addr.store::<i16>(&arg, ptr, data as i16)?;
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_store8(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let data = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        // Store lower 8 bits
        mem_addr.store::<i8>(&arg, ptr, data as i8)?;
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_store16(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let data = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        // Store lower 16 bits
        mem_addr.store::<i16>(&arg, ptr, data as i16)?;
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_store32(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let data = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
        let ptr = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        // Store lower 32 bits
        mem_addr.store::<i32>(&arg, ptr, data as i32)?;
        Ok(ctx.ip + 1)
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

// Note: F64 handlers can be implemented without ctx.frame access for now
fn handle_f64_sqrt(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let x = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let mut result: f64;
    // Using standard Rust sqrt for simplicity, could use asm! later if needed
    result = x.sqrt();
    // unsafe { asm!("local.get {0}", "f64.sqrt", "local.set {1}", in(local) x, out(local) result); }
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(ctx.ip + 1)
}

fn handle_f64_add(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop!(ctx, F64, Add, add) }
fn handle_f64_sub(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop!(ctx, F64, Sub, sub) }
fn handle_f64_mul(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop!(ctx, F64, Mul, mul) }
fn handle_f64_div(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> { binop!(ctx, F64, Div, div) }

fn handle_f64_min(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let mut result: f64;
    // Using standard Rust min
    result = lhs.min(rhs);
    // unsafe { asm!("local.get {0}", "local.get {1}", "f64.min", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(ctx.ip + 1)
}

fn handle_f64_max(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let mut result: f64;
    // Using standard Rust max
    result = lhs.max(rhs);
    // unsafe { asm!("local.get {0}", "local.get {1}", "f64.max", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(ctx.ip + 1)
}

fn handle_f64_copysign(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let rhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let lhs = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let mut result: f64;
    // Using standard Rust copysign
    result = lhs.copysign(rhs);
    // unsafe { asm!("local.get {0}", "local.get {1}", "f64.copysign", "local.set {2}", in(local) lhs, in(local) rhs, out(local) result); }
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(ctx.ip + 1)
}

fn handle_memory_size(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
    if module_inst.mem_addrs.is_empty() {
        return Err(RuntimeError::MemoryNotFound);
    }
    let mem_addr = &module_inst.mem_addrs[0]; // Assuming memidx 0
    // Assuming MemAddr has a size() method returning current size in pages
    let size = mem_addr.mem_size();
    ctx.value_stack.push(Val::Num(Num::I32(size as i32)));
    Ok(ctx.ip + 1)
}

fn handle_memory_grow(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let delta = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
    if module_inst.mem_addrs.is_empty() {
        return Err(RuntimeError::MemoryNotFound);
    }
    let mem_addr = &module_inst.mem_addrs[0]; // Assuming memidx 0
    // Assuming MemAddr has a grow() method returning previous size or -1 on failure
    let prev_size = mem_addr.mem_grow((delta as u32).try_into().unwrap());
    ctx.value_stack.push(Val::Num(Num::I32(prev_size as i32)));
    Ok(ctx.ip + 1)
}

fn handle_f32_convert_i32_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let result = val as f32; // i32 to f32 conversion
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(ctx.ip + 1)
}

fn handle_f32_convert_i32_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let result = (val as u32) as f32; // i32 -> u32 -> f32 conversion
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(ctx.ip + 1)
}

fn handle_f32_convert_i64_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let result = val as f32; // i64 to f32 conversion
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(ctx.ip + 1)
}

fn handle_f32_convert_i64_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let result = (val as u64) as f32; // i64 -> u64 -> f32 conversion
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(ctx.ip + 1)
}

fn handle_f64_convert_i32_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let result = val as f64; // i32 to f64 conversion
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(ctx.ip + 1)
}

fn handle_f64_convert_i32_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let result = (val as u32) as f64; // i32 -> u32 -> f64 conversion
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(ctx.ip + 1)
}

fn handle_f64_convert_i64_s(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let result = val as f64; // i64 to f64 conversion
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(ctx.ip + 1)
}

fn handle_f64_convert_i64_u(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let result = (val as u64) as f64; // i64 -> u64 -> f64 conversion
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(ctx.ip + 1)
}

fn handle_i32_reinterpret_f32(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val_f32 = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f32();
    let val_i32 = unsafe { std::mem::transmute::<f32, i32>(val_f32) };
    ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
    Ok(ctx.ip + 1)
}

fn handle_i64_reinterpret_f64(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val_f64 = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_f64();
    let val_i64 = unsafe { std::mem::transmute::<f64, i64>(val_f64) };
    ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
    Ok(ctx.ip + 1)
}

fn handle_f32_reinterpret_i32(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val_i32 = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i32();
    let val_f32 = unsafe { std::mem::transmute::<i32, f32>(val_i32) };
    ctx.value_stack.push(Val::Num(Num::F32(val_f32)));
    Ok(ctx.ip + 1)
}

fn handle_f64_reinterpret_i64(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
    let val_i64 = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?.to_i64();
    let val_f64 = unsafe { std::mem::transmute::<i64, f64>(val_i64) };
    ctx.value_stack.push(Val::Num(Num::F64(val_f64)));
    Ok(ctx.ip + 1)
}

// --- Handler Table ---
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
        // TODO: Add remaining handlers as they are implemented (TruncSat, etc.)
        table
    };
}
