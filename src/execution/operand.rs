//! Operand read/write helpers for v2 dispatcher handlers.
//!
//! These functions bridge raw `VmState` pointers to the typed operand
//! enums (`I32RegOperand`, `I64RegOperand`, `F32RegOperand`, `F64RegOperand`,
//! `RegOrLocal`, `Reg`) defined in `ir.rs` alongside `ProcessedInstr`.
//!
//! All functions are `#[inline(always)]` so handlers compile to tight code
//! without function-call overhead per operand access.

use crate::execution::ir::{
    F32RegOperand, F64RegOperand, I32RegOperand, I64RegOperand, RegOrLocal,
};
use crate::execution::regs::Reg;
use crate::execution::state::VmState;
use crate::execution::value::{Num, Val};

// ============================================================================
// I32 typed operand (I32RegOperand: Reg | Const | Param)
// ============================================================================

#[inline(always)]
pub unsafe fn read_i32(state: &VmState, op: &I32RegOperand) -> i32 {
    match op {
        I32RegOperand::Reg(idx) => (*state.reg_file).get_i32(*idx),
        I32RegOperand::Const(v) => *v,
        I32RegOperand::Param(idx) => match &*state.locals.add(*idx as usize) {
            Val::Num(Num::I32(v)) => *v,
            _ => 0,
        },
    }
}

#[inline(always)]
pub unsafe fn write_i32(state: &mut VmState, dst: &I32RegOperand, val: i32) {
    match dst {
        I32RegOperand::Reg(idx) => (*state.reg_file).set_i32(*idx, val),
        I32RegOperand::Param(idx) => {
            *state.locals.add(*idx as usize) = Val::Num(Num::I32(val));
        }
        I32RegOperand::Const(_) => std::hint::unreachable_unchecked(),
    }
}

// ============================================================================
// I64 typed operand
// ============================================================================

#[inline(always)]
pub unsafe fn read_i64(state: &VmState, op: &I64RegOperand) -> i64 {
    match op {
        I64RegOperand::Reg(idx) => (*state.reg_file).get_i64(*idx),
        I64RegOperand::Const(v) => *v,
        I64RegOperand::Param(idx) => match &*state.locals.add(*idx as usize) {
            Val::Num(Num::I64(v)) => *v,
            _ => 0,
        },
    }
}

#[inline(always)]
pub unsafe fn write_i64(state: &mut VmState, dst: &I64RegOperand, val: i64) {
    match dst {
        I64RegOperand::Reg(idx) => (*state.reg_file).set_i64(*idx, val),
        I64RegOperand::Param(idx) => {
            *state.locals.add(*idx as usize) = Val::Num(Num::I64(val));
        }
        I64RegOperand::Const(_) => std::hint::unreachable_unchecked(),
    }
}

/// I64 comparison handlers produce i32 result; dst encoded as I64RegOperand
/// where Reg(idx) is reinterpreted as an index into i32_regs (matches
/// legacy `I64RegContext::set_dst_i32`).
#[inline(always)]
pub unsafe fn write_i64dst_i32(state: &mut VmState, dst: &I64RegOperand, val: i32) {
    match dst {
        I64RegOperand::Reg(idx) => (*state.reg_file).set_i32(*idx, val),
        I64RegOperand::Param(idx) => {
            *state.locals.add(*idx as usize) = Val::Num(Num::I32(val));
        }
        I64RegOperand::Const(_) => std::hint::unreachable_unchecked(),
    }
}

// ============================================================================
// F32 typed operand
// ============================================================================

#[inline(always)]
pub unsafe fn read_f32(state: &VmState, op: &F32RegOperand) -> f32 {
    match op {
        F32RegOperand::Reg(idx) => (*state.reg_file).get_f32(*idx),
        F32RegOperand::Const(v) => *v,
        F32RegOperand::Param(idx) => match &*state.locals.add(*idx as usize) {
            Val::Num(Num::F32(v)) => *v,
            _ => 0.0,
        },
    }
}

#[inline(always)]
pub unsafe fn write_f32(state: &mut VmState, dst: &F32RegOperand, val: f32) {
    match dst {
        F32RegOperand::Reg(idx) => (*state.reg_file).set_f32(*idx, val),
        F32RegOperand::Param(idx) => {
            *state.locals.add(*idx as usize) = Val::Num(Num::F32(val));
        }
        F32RegOperand::Const(_) => std::hint::unreachable_unchecked(),
    }
}

#[inline(always)]
pub unsafe fn write_f32dst_i32(state: &mut VmState, dst: &F32RegOperand, val: i32) {
    match dst {
        F32RegOperand::Reg(idx) => (*state.reg_file).set_i32(*idx, val),
        F32RegOperand::Param(idx) => {
            *state.locals.add(*idx as usize) = Val::Num(Num::I32(val));
        }
        F32RegOperand::Const(_) => std::hint::unreachable_unchecked(),
    }
}

// ============================================================================
// F64 typed operand
// ============================================================================

#[inline(always)]
pub unsafe fn read_f64(state: &VmState, op: &F64RegOperand) -> f64 {
    match op {
        F64RegOperand::Reg(idx) => (*state.reg_file).get_f64(*idx),
        F64RegOperand::Const(v) => *v,
        F64RegOperand::Param(idx) => match &*state.locals.add(*idx as usize) {
            Val::Num(Num::F64(v)) => *v,
            _ => 0.0,
        },
    }
}

#[inline(always)]
pub unsafe fn write_f64(state: &mut VmState, dst: &F64RegOperand, val: f64) {
    match dst {
        F64RegOperand::Reg(idx) => (*state.reg_file).set_f64(*idx, val),
        F64RegOperand::Param(idx) => {
            *state.locals.add(*idx as usize) = Val::Num(Num::F64(val));
        }
        F64RegOperand::Const(_) => std::hint::unreachable_unchecked(),
    }
}

#[inline(always)]
pub unsafe fn write_f64dst_i32(state: &mut VmState, dst: &F64RegOperand, val: i32) {
    match dst {
        F64RegOperand::Reg(idx) => (*state.reg_file).set_i32(*idx, val),
        F64RegOperand::Param(idx) => {
            *state.locals.add(*idx as usize) = Val::Num(Num::I32(val));
        }
        F64RegOperand::Const(_) => std::hint::unreachable_unchecked(),
    }
}

// ============================================================================
// Reg (typed register: I32/I64/F32/F64/Ref/V128) — used by ConversionReg src
// ============================================================================

#[inline(always)]
pub unsafe fn read_reg_i32(state: &VmState, reg: &Reg) -> i32 {
    (*state.reg_file).get_i32(reg.index())
}
#[inline(always)]
pub unsafe fn read_reg_i64(state: &VmState, reg: &Reg) -> i64 {
    (*state.reg_file).get_i64(reg.index())
}
#[inline(always)]
pub unsafe fn read_reg_f32(state: &VmState, reg: &Reg) -> f32 {
    (*state.reg_file).get_f32(reg.index())
}
#[inline(always)]
pub unsafe fn read_reg_f64(state: &VmState, reg: &Reg) -> f64 {
    (*state.reg_file).get_f64(reg.index())
}

// ============================================================================
// RegOrLocal dst (used by ConversionReg, MemoryLoadReg, GlobalGetReg)
// ============================================================================

#[inline(always)]
pub unsafe fn write_dst_i32(state: &mut VmState, dst: &RegOrLocal, val: i32) {
    match dst {
        RegOrLocal::Reg(idx) => (*state.reg_file).set_i32(*idx, val),
        RegOrLocal::Local(idx) => *state.locals.add(*idx as usize) = Val::Num(Num::I32(val)),
    }
}
#[inline(always)]
pub unsafe fn write_dst_i64(state: &mut VmState, dst: &RegOrLocal, val: i64) {
    match dst {
        RegOrLocal::Reg(idx) => (*state.reg_file).set_i64(*idx, val),
        RegOrLocal::Local(idx) => *state.locals.add(*idx as usize) = Val::Num(Num::I64(val)),
    }
}
#[inline(always)]
pub unsafe fn write_dst_f32(state: &mut VmState, dst: &RegOrLocal, val: f32) {
    match dst {
        RegOrLocal::Reg(idx) => (*state.reg_file).set_f32(*idx, val),
        RegOrLocal::Local(idx) => *state.locals.add(*idx as usize) = Val::Num(Num::F32(val)),
    }
}
#[inline(always)]
pub unsafe fn write_dst_f64(state: &mut VmState, dst: &RegOrLocal, val: f64) {
    match dst {
        RegOrLocal::Reg(idx) => (*state.reg_file).set_f64(*idx, val),
        RegOrLocal::Local(idx) => *state.locals.add(*idx as usize) = Val::Num(Num::F64(val)),
    }
}
