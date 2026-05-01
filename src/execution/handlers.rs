//! v2 dispatcher handlers — single source of truth for both `dispatch_loop`
//! and `dispatch_tco` modes.
//!
//! Each handler reads operands from the active `ProcessedInstr`, performs
//! its operation via `ops::*`, writes the result, advances `state.pc`, and
//! invokes the `advance!` macro to continue dispatch. The macro expansion
//! depends on `cfg(feature = "tco")`:
//!
//! - **non-tco**: returns `Outcome::Continue` so the outer loop fetches the
//!   next instruction.
//! - **tco**: tail-calls the next handler via `state.handlers[state.pc]`,
//!   which LLVM with `+tail-call` emits as `return_call_indirect`.
//!
//! Trap conditions tail-call the `h_trap` sentinel, function-end uses
//! `h_halt`, runtime yields use `h_yield`. All sentinels just return
//! their respective `Outcome`, terminating the chain cleanly.

#![allow(unused_unsafe)]

use crate::error::RuntimeError;
use crate::execution::ir::{Handler, Outcome};
use crate::execution::module::GetInstanceByIdx;
use crate::execution::operand;
use crate::execution::ops;
use crate::execution::regs::Reg;
use crate::execution::state::VmState;
use crate::execution::value::Val;
use crate::execution::vm::{self, Label, LabelStack, ModuleLevelInstr, ProcessedInstr, RegOrLocal};
use arrayvec::ArrayVec;

// ============================================================================
// advance! macro — the difference between tco and non-tco mode
// ============================================================================

#[cfg(feature = "tco")]
macro_rules! advance {
    ($state:expr) => {{
        let h = unsafe { *$state.handlers.add($state.pc) };
        h($state)
    }};
}

#[cfg(not(feature = "tco"))]
macro_rules! advance {
    ($state:expr) => {{
        let _ = $state;
        Outcome::Continue
    }};
}

// ============================================================================
// Sentinel handlers — chain terminators
// ============================================================================

#[inline(never)]
pub fn h_trap(_state: &mut VmState) -> Outcome {
    Outcome::Trap
}

#[inline(never)]
pub fn h_halt(_state: &mut VmState) -> Outcome {
    Outcome::Halt
}

#[inline(never)]
pub fn h_yield(_state: &mut VmState) -> Outcome {
    Outcome::Yield
}

/// Default handler for unknown handler_index — returns Trap with InvalidHandlerIndex.
pub fn h_invalid(state: &mut VmState) -> Outcome {
    state.trap = Some(RuntimeError::InvalidHandlerIndex);
    h_trap(state)
}

// ============================================================================
// I32 arithmetic / comparison / unary handlers
// ============================================================================

macro_rules! i32_binop {
    ($name:ident, $op:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::I32Reg {
                    dst, src1, src2, ..
                } = instr
                else {
                    std::hint::unreachable_unchecked()
                };
                let a = operand::read_i32(state, src1);
                let b = operand::read_i32(state, src2.as_ref().unwrap_unchecked());
                operand::write_i32(state, dst, $op(a, b));
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

macro_rules! i32_unop {
    ($name:ident, $op:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::I32Reg { dst, src1, .. } = instr else {
                    std::hint::unreachable_unchecked()
                };
                let a = operand::read_i32(state, src1);
                operand::write_i32(state, dst, $op(a));
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

// local.get / local.set / i32.const all reduce to "read src1, write dst" (identity copy).
i32_unop!(h_i32_local_get, |a: i32| a);
i32_unop!(h_i32_local_set, |a: i32| a);
i32_unop!(h_i32_const, |a: i32| a);

// Binary
i32_binop!(h_i32_add, ops::i32_add);
i32_binop!(h_i32_sub, ops::i32_sub);
i32_binop!(h_i32_mul, ops::i32_mul);
i32_binop!(h_i32_and, ops::i32_and);
i32_binop!(h_i32_or, ops::i32_or);
i32_binop!(h_i32_xor, ops::i32_xor);
i32_binop!(h_i32_shl, ops::i32_shl);
i32_binop!(h_i32_shr_s, ops::i32_shr_s);
i32_binop!(h_i32_shr_u, ops::i32_shr_u);
i32_binop!(h_i32_rotl, ops::i32_rotl);
i32_binop!(h_i32_rotr, ops::i32_rotr);

// Comparisons (already return i32 from ops::*)
i32_binop!(h_i32_eq, ops::i32_eq);
i32_binop!(h_i32_ne, ops::i32_ne);
i32_binop!(h_i32_lt_s, ops::i32_lt_s);
i32_binop!(h_i32_lt_u, ops::i32_lt_u);
i32_binop!(h_i32_le_s, ops::i32_le_s);
i32_binop!(h_i32_le_u, ops::i32_le_u);
i32_binop!(h_i32_gt_s, ops::i32_gt_s);
i32_binop!(h_i32_gt_u, ops::i32_gt_u);
i32_binop!(h_i32_ge_s, ops::i32_ge_s);
i32_binop!(h_i32_ge_u, ops::i32_ge_u);

// Unary
i32_unop!(h_i32_clz, ops::i32_clz);
i32_unop!(h_i32_ctz, ops::i32_ctz);
i32_unop!(h_i32_popcnt, ops::i32_popcnt);
i32_unop!(h_i32_eqz, ops::i32_eqz);
i32_unop!(h_i32_extend8_s, ops::i32_extend8_s);
i32_unop!(h_i32_extend16_s, ops::i32_extend16_s);

// Division / remainder need trap handling. Tail-call h_trap on error so
// the success path is preserved as a tail call (LLVM still emits
// return_call_indirect for it).
pub fn h_i32_div_s(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::I32Reg {
            dst, src1, src2, ..
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let a = operand::read_i32(state, src1);
        let b = operand::read_i32(state, src2.as_ref().unwrap_unchecked());
        if b == 0 {
            state.trap = Some(RuntimeError::ZeroDivideError);
            return h_trap(state);
        }
        operand::write_i32(state, dst, a.wrapping_div(b));
        state.pc += 1;
        advance!(state)
    }
}

pub fn h_i32_div_u(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::I32Reg {
            dst, src1, src2, ..
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let a = operand::read_i32(state, src1);
        let b = operand::read_i32(state, src2.as_ref().unwrap_unchecked()) as u32;
        if b == 0 {
            state.trap = Some(RuntimeError::ZeroDivideError);
            return h_trap(state);
        }
        operand::write_i32(state, dst, ((a as u32) / b) as i32);
        state.pc += 1;
        advance!(state)
    }
}

pub fn h_i32_rem_s(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::I32Reg {
            dst, src1, src2, ..
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let a = operand::read_i32(state, src1);
        let b = operand::read_i32(state, src2.as_ref().unwrap_unchecked());
        if b == 0 {
            state.trap = Some(RuntimeError::ZeroDivideError);
            return h_trap(state);
        }
        operand::write_i32(state, dst, a.wrapping_rem(b));
        state.pc += 1;
        advance!(state)
    }
}

pub fn h_i32_rem_u(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::I32Reg {
            dst, src1, src2, ..
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let a = operand::read_i32(state, src1);
        let b = operand::read_i32(state, src2.as_ref().unwrap_unchecked()) as u32;
        if b == 0 {
            state.trap = Some(RuntimeError::ZeroDivideError);
            return h_trap(state);
        }
        operand::write_i32(state, dst, ((a as u32) % b) as i32);
        state.pc += 1;
        advance!(state)
    }
}

// ============================================================================
// I64 arithmetic / comparison / unary handlers
// ============================================================================

macro_rules! i64_binop {
    ($name:ident, $op:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::I64Reg {
                    dst, src1, src2, ..
                } = instr
                else {
                    std::hint::unreachable_unchecked()
                };
                let a = operand::read_i64(state, src1);
                let b = operand::read_i64(state, src2.as_ref().unwrap_unchecked());
                operand::write_i64(state, dst, $op(a, b));
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

/// I64 comparison: i64 inputs, i32 result (written via i32_regs route)
macro_rules! i64_cmp {
    ($name:ident, $op:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::I64Reg {
                    dst, src1, src2, ..
                } = instr
                else {
                    std::hint::unreachable_unchecked()
                };
                let a = operand::read_i64(state, src1);
                let b = operand::read_i64(state, src2.as_ref().unwrap_unchecked());
                operand::write_i64dst_i32(state, dst, $op(a, b));
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

macro_rules! i64_unop {
    ($name:ident, $op:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::I64Reg { dst, src1, .. } = instr else {
                    std::hint::unreachable_unchecked()
                };
                let a = operand::read_i64(state, src1);
                operand::write_i64(state, dst, $op(a));
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

i64_unop!(h_i64_local_get, |a: i64| a);
i64_unop!(h_i64_local_set, |a: i64| a);
i64_unop!(h_i64_const, |a: i64| a);

// Binary
i64_binop!(h_i64_add, ops::i64_add);
i64_binop!(h_i64_sub, ops::i64_sub);
i64_binop!(h_i64_mul, ops::i64_mul);
i64_binop!(h_i64_and, ops::i64_and);
i64_binop!(h_i64_or, ops::i64_or);
i64_binop!(h_i64_xor, ops::i64_xor);
i64_binop!(h_i64_shl, ops::i64_shl);
i64_binop!(h_i64_shr_s, ops::i64_shr_s);
i64_binop!(h_i64_shr_u, ops::i64_shr_u);
i64_binop!(h_i64_rotl, ops::i64_rotl);
i64_binop!(h_i64_rotr, ops::i64_rotr);

// Comparison (i32 result)
i64_cmp!(h_i64_eq, ops::i64_eq);
i64_cmp!(h_i64_ne, ops::i64_ne);
i64_cmp!(h_i64_lt_s, ops::i64_lt_s);
i64_cmp!(h_i64_lt_u, ops::i64_lt_u);
i64_cmp!(h_i64_le_s, ops::i64_le_s);
i64_cmp!(h_i64_le_u, ops::i64_le_u);
i64_cmp!(h_i64_gt_s, ops::i64_gt_s);
i64_cmp!(h_i64_gt_u, ops::i64_gt_u);
i64_cmp!(h_i64_ge_s, ops::i64_ge_s);
i64_cmp!(h_i64_ge_u, ops::i64_ge_u);

// Unary
i64_unop!(h_i64_clz, ops::i64_clz);
i64_unop!(h_i64_ctz, ops::i64_ctz);
i64_unop!(h_i64_popcnt, ops::i64_popcnt);
i64_unop!(h_i64_extend8_s, ops::i64_extend8_s);
i64_unop!(h_i64_extend16_s, ops::i64_extend16_s);
i64_unop!(h_i64_extend32_s, ops::i64_extend32_s);

// i64.eqz: i64 input, i32 result (custom path because of mismatched types)
pub fn h_i64_eqz(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::I64Reg { dst, src1, .. } = instr else {
            std::hint::unreachable_unchecked()
        };
        let a = operand::read_i64(state, src1);
        operand::write_i64dst_i32(state, dst, ops::i64_eqz(a));
        state.pc += 1;
        advance!(state)
    }
}

// I64 division / remainder with trap (overflow check on div_s)
pub fn h_i64_div_s(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::I64Reg {
            dst, src1, src2, ..
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let a = operand::read_i64(state, src1);
        let b = operand::read_i64(state, src2.as_ref().unwrap_unchecked());
        if b == 0 {
            state.trap = Some(RuntimeError::ZeroDivideError);
            return h_trap(state);
        }
        if a == i64::MIN && b == -1 {
            state.trap = Some(RuntimeError::IntegerOverflow);
            return h_trap(state);
        }
        operand::write_i64(state, dst, a.wrapping_div(b));
        state.pc += 1;
        advance!(state)
    }
}

pub fn h_i64_div_u(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::I64Reg {
            dst, src1, src2, ..
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let a = operand::read_i64(state, src1);
        let b = operand::read_i64(state, src2.as_ref().unwrap_unchecked()) as u64;
        if b == 0 {
            state.trap = Some(RuntimeError::ZeroDivideError);
            return h_trap(state);
        }
        operand::write_i64(state, dst, ((a as u64) / b) as i64);
        state.pc += 1;
        advance!(state)
    }
}

pub fn h_i64_rem_s(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::I64Reg {
            dst, src1, src2, ..
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let a = operand::read_i64(state, src1);
        let b = operand::read_i64(state, src2.as_ref().unwrap_unchecked());
        if b == 0 {
            state.trap = Some(RuntimeError::ZeroDivideError);
            return h_trap(state);
        }
        operand::write_i64(state, dst, a.wrapping_rem(b));
        state.pc += 1;
        advance!(state)
    }
}

pub fn h_i64_rem_u(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::I64Reg {
            dst, src1, src2, ..
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let a = operand::read_i64(state, src1);
        let b = operand::read_i64(state, src2.as_ref().unwrap_unchecked()) as u64;
        if b == 0 {
            state.trap = Some(RuntimeError::ZeroDivideError);
            return h_trap(state);
        }
        operand::write_i64(state, dst, ((a as u64) % b) as i64);
        state.pc += 1;
        advance!(state)
    }
}

// ============================================================================
// F32 handlers
// ============================================================================

macro_rules! f32_binop {
    ($name:ident, $op:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::F32Reg {
                    dst, src1, src2, ..
                } = instr
                else {
                    std::hint::unreachable_unchecked()
                };
                let a = operand::read_f32(state, src1);
                let b = operand::read_f32(state, src2.as_ref().unwrap_unchecked());
                operand::write_f32(state, dst, $op(a, b));
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

macro_rules! f32_cmp {
    ($name:ident, $op:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::F32Reg {
                    dst, src1, src2, ..
                } = instr
                else {
                    std::hint::unreachable_unchecked()
                };
                let a = operand::read_f32(state, src1);
                let b = operand::read_f32(state, src2.as_ref().unwrap_unchecked());
                operand::write_f32dst_i32(state, dst, $op(a, b));
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

macro_rules! f32_unop {
    ($name:ident, $op:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::F32Reg { dst, src1, .. } = instr else {
                    std::hint::unreachable_unchecked()
                };
                let a = operand::read_f32(state, src1);
                operand::write_f32(state, dst, $op(a));
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

f32_unop!(h_f32_local_get, |a: f32| a);
f32_unop!(h_f32_local_set, |a: f32| a);
f32_unop!(h_f32_const, |a: f32| a);

// Binary
f32_binop!(h_f32_add, ops::f32_add);
f32_binop!(h_f32_sub, ops::f32_sub);
f32_binop!(h_f32_mul, ops::f32_mul);
f32_binop!(h_f32_div, ops::f32_div);
f32_binop!(h_f32_min, ops::f32_min);
f32_binop!(h_f32_max, ops::f32_max);
f32_binop!(h_f32_copysign, ops::f32_copysign);

// Unary
f32_unop!(h_f32_abs, ops::f32_abs);
f32_unop!(h_f32_neg, ops::f32_neg);
f32_unop!(h_f32_ceil, ops::f32_ceil);
f32_unop!(h_f32_floor, ops::f32_floor);
f32_unop!(h_f32_trunc, ops::f32_trunc);
f32_unop!(h_f32_nearest, ops::f32_nearest);
f32_unop!(h_f32_sqrt, ops::f32_sqrt);

// Comparison (i32 result)
f32_cmp!(h_f32_eq, ops::f32_eq);
f32_cmp!(h_f32_ne, ops::f32_ne);
f32_cmp!(h_f32_lt, ops::f32_lt);
f32_cmp!(h_f32_gt, ops::f32_gt);
f32_cmp!(h_f32_le, ops::f32_le);
f32_cmp!(h_f32_ge, ops::f32_ge);

// ============================================================================
// F64 handlers
// ============================================================================

macro_rules! f64_binop {
    ($name:ident, $op:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::F64Reg {
                    dst, src1, src2, ..
                } = instr
                else {
                    std::hint::unreachable_unchecked()
                };
                let a = operand::read_f64(state, src1);
                let b = operand::read_f64(state, src2.as_ref().unwrap_unchecked());
                operand::write_f64(state, dst, $op(a, b));
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

macro_rules! f64_cmp {
    ($name:ident, $op:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::F64Reg {
                    dst, src1, src2, ..
                } = instr
                else {
                    std::hint::unreachable_unchecked()
                };
                let a = operand::read_f64(state, src1);
                let b = operand::read_f64(state, src2.as_ref().unwrap_unchecked());
                operand::write_f64dst_i32(state, dst, $op(a, b));
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

macro_rules! f64_unop {
    ($name:ident, $op:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::F64Reg { dst, src1, .. } = instr else {
                    std::hint::unreachable_unchecked()
                };
                let a = operand::read_f64(state, src1);
                operand::write_f64(state, dst, $op(a));
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

f64_unop!(h_f64_local_get, |a: f64| a);
f64_unop!(h_f64_local_set, |a: f64| a);
f64_unop!(h_f64_const, |a: f64| a);

// Binary
f64_binop!(h_f64_add, ops::f64_add);
f64_binop!(h_f64_sub, ops::f64_sub);
f64_binop!(h_f64_mul, ops::f64_mul);
f64_binop!(h_f64_div, ops::f64_div);
f64_binop!(h_f64_min, ops::f64_min);
f64_binop!(h_f64_max, ops::f64_max);
f64_binop!(h_f64_copysign, ops::f64_copysign);

// Unary
f64_unop!(h_f64_abs, ops::f64_abs);
f64_unop!(h_f64_neg, ops::f64_neg);
f64_unop!(h_f64_ceil, ops::f64_ceil);
f64_unop!(h_f64_floor, ops::f64_floor);
f64_unop!(h_f64_trunc, ops::f64_trunc);
f64_unop!(h_f64_nearest, ops::f64_nearest);
f64_unop!(h_f64_sqrt, ops::f64_sqrt);

// Comparison (i32 result)
f64_cmp!(h_f64_eq, ops::f64_eq);
f64_cmp!(h_f64_ne, ops::f64_ne);
f64_cmp!(h_f64_lt, ops::f64_lt);
f64_cmp!(h_f64_gt, ops::f64_gt);
f64_cmp!(h_f64_le, ops::f64_le);
f64_cmp!(h_f64_ge, ops::f64_ge);

// ============================================================================
// ConversionReg handlers
// ============================================================================

/// Macro for non-trapping conversions (extend, reinterpret, sat trunc, int↔float).
macro_rules! conv {
    ($name:ident, $read:ident, $write:ident, $body:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::ConversionReg { src, dst, .. } = instr else {
                    std::hint::unreachable_unchecked()
                };
                let v = operand::$read(state, src);
                operand::$write(state, dst, $body(v));
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

// i32 ↔ i64
conv!(
    h_conv_i64_extend_i32_s,
    read_reg_i32,
    write_dst_i64,
    |v: i32| v as i64
);
conv!(
    h_conv_i64_extend_i32_u,
    read_reg_i32,
    write_dst_i64,
    |v: i32| (v as u32) as i64
);
conv!(
    h_conv_i32_wrap_i64,
    read_reg_i64,
    write_dst_i32,
    |v: i64| v as i32
);

// Saturating float→int (no traps)
conv!(
    h_conv_i32_trunc_sat_f32_s,
    read_reg_f32,
    write_dst_i32,
    |v: f32| {
        if v.is_nan() {
            0
        } else if v <= (i32::MIN as f32) {
            i32::MIN
        } else if v >= (i32::MAX as f32) {
            i32::MAX
        } else {
            v.trunc() as i32
        }
    }
);
conv!(
    h_conv_i32_trunc_sat_f32_u,
    read_reg_f32,
    write_dst_i32,
    |v: f32| {
        if v.is_nan() || v <= 0.0 {
            0i32
        } else if v >= (u32::MAX as f32) {
            u32::MAX as i32
        } else {
            (v.trunc() as u32) as i32
        }
    }
);
conv!(
    h_conv_i32_trunc_sat_f64_s,
    read_reg_f64,
    write_dst_i32,
    |v: f64| {
        if v.is_nan() {
            0
        } else if v <= (i32::MIN as f64) {
            i32::MIN
        } else if v >= (i32::MAX as f64) {
            i32::MAX
        } else {
            v.trunc() as i32
        }
    }
);
conv!(
    h_conv_i32_trunc_sat_f64_u,
    read_reg_f64,
    write_dst_i32,
    |v: f64| {
        if v.is_nan() || v <= 0.0 {
            0i32
        } else if v >= (u32::MAX as f64) {
            u32::MAX as i32
        } else {
            (v.trunc() as u32) as i32
        }
    }
);
conv!(
    h_conv_i64_trunc_sat_f32_s,
    read_reg_f32,
    write_dst_i64,
    |v: f32| {
        if v.is_nan() {
            0
        } else if v <= (i64::MIN as f32) {
            i64::MIN
        } else if v >= (i64::MAX as f32) {
            i64::MAX
        } else {
            v.trunc() as i64
        }
    }
);
conv!(
    h_conv_i64_trunc_sat_f32_u,
    read_reg_f32,
    write_dst_i64,
    |v: f32| {
        if v.is_nan() || v <= 0.0 {
            0i64
        } else if v >= (u64::MAX as f32) {
            u64::MAX as i64
        } else {
            (v.trunc() as u64) as i64
        }
    }
);
conv!(
    h_conv_i64_trunc_sat_f64_s,
    read_reg_f64,
    write_dst_i64,
    |v: f64| {
        if v.is_nan() {
            0
        } else if v <= (i64::MIN as f64) {
            i64::MIN
        } else if v >= (i64::MAX as f64) {
            i64::MAX
        } else {
            v.trunc() as i64
        }
    }
);
conv!(
    h_conv_i64_trunc_sat_f64_u,
    read_reg_f64,
    write_dst_i64,
    |v: f64| {
        if v.is_nan() || v <= 0.0 {
            0i64
        } else if v >= (u64::MAX as f64) {
            u64::MAX as i64
        } else {
            (v.trunc() as u64) as i64
        }
    }
);

// Int → float
conv!(
    h_conv_f32_convert_i32_s,
    read_reg_i32,
    write_dst_f32,
    |v: i32| v as f32
);
conv!(
    h_conv_f32_convert_i32_u,
    read_reg_i32,
    write_dst_f32,
    |v: i32| (v as u32) as f32
);
conv!(
    h_conv_f32_convert_i64_s,
    read_reg_i64,
    write_dst_f32,
    |v: i64| v as f32
);
conv!(
    h_conv_f32_convert_i64_u,
    read_reg_i64,
    write_dst_f32,
    |v: i64| (v as u64) as f32
);
conv!(
    h_conv_f64_convert_i32_s,
    read_reg_i32,
    write_dst_f64,
    |v: i32| v as f64
);
conv!(
    h_conv_f64_convert_i32_u,
    read_reg_i32,
    write_dst_f64,
    |v: i32| (v as u32) as f64
);
conv!(
    h_conv_f64_convert_i64_s,
    read_reg_i64,
    write_dst_f64,
    |v: i64| v as f64
);
conv!(
    h_conv_f64_convert_i64_u,
    read_reg_i64,
    write_dst_f64,
    |v: i64| (v as u64) as f64
);

// Float ↔ float
conv!(
    h_conv_f32_demote_f64,
    read_reg_f64,
    write_dst_f32,
    |v: f64| v as f32
);
conv!(
    h_conv_f64_promote_f32,
    read_reg_f32,
    write_dst_f64,
    |v: f32| v as f64
);

// Reinterpret (bitwise)
conv!(
    h_conv_i32_reinterpret_f32,
    read_reg_f32,
    write_dst_i32,
    |v: f32| v.to_bits() as i32
);
conv!(
    h_conv_f32_reinterpret_i32,
    read_reg_i32,
    write_dst_f32,
    |v: i32| f32::from_bits(v as u32)
);
conv!(
    h_conv_i64_reinterpret_f64,
    read_reg_f64,
    write_dst_i64,
    |v: f64| v.to_bits() as i64
);
conv!(
    h_conv_f64_reinterpret_i64,
    read_reg_i64,
    write_dst_f64,
    |v: i64| f64::from_bits(v as u64)
);

// Trapping float→int converters — branch + trap sentinel tail-call.
macro_rules! conv_trap {
    ($name:ident, $read:ident, $write:ident, $ty:ty, $min:expr, $max:expr, $cast:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::ConversionReg { src, dst, .. } = instr else {
                    std::hint::unreachable_unchecked()
                };
                let v = operand::$read(state, src);
                if v.is_nan() {
                    state.trap = Some(RuntimeError::InvalidConversionToInt);
                    return h_trap(state);
                }
                let t = v.trunc();
                if t < $min || t > $max {
                    state.trap = Some(RuntimeError::IntegerOverflow);
                    return h_trap(state);
                }
                operand::$write(state, dst, $cast(t));
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

conv_trap!(
    h_conv_i32_trunc_f32_s,
    read_reg_f32,
    write_dst_i32,
    f32,
    i32::MIN as f32,
    i32::MAX as f32,
    |t: f32| t as i32
);
conv_trap!(
    h_conv_i32_trunc_f32_u,
    read_reg_f32,
    write_dst_i32,
    f32,
    0.0_f32,
    u32::MAX as f32,
    |t: f32| (t as u32) as i32
);
conv_trap!(
    h_conv_i32_trunc_f64_s,
    read_reg_f64,
    write_dst_i32,
    f64,
    i32::MIN as f64,
    i32::MAX as f64,
    |t: f64| t as i32
);
conv_trap!(
    h_conv_i32_trunc_f64_u,
    read_reg_f64,
    write_dst_i32,
    f64,
    0.0_f64,
    u32::MAX as f64,
    |t: f64| (t as u32) as i32
);

// i64 trunc has different bound check (>= for max), so write explicit functions
pub fn h_conv_i64_trunc_f32_s(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::ConversionReg { src, dst, .. } = instr else {
            std::hint::unreachable_unchecked()
        };
        let v = operand::read_reg_f32(state, src);
        if v.is_nan() {
            state.trap = Some(RuntimeError::InvalidConversionToInt);
            return h_trap(state);
        }
        let t = v.trunc();
        if t < (i64::MIN as f32) || t >= (i64::MAX as f32) {
            state.trap = Some(RuntimeError::IntegerOverflow);
            return h_trap(state);
        }
        operand::write_dst_i64(state, dst, t as i64);
        state.pc += 1;
        advance!(state)
    }
}
pub fn h_conv_i64_trunc_f32_u(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::ConversionReg { src, dst, .. } = instr else {
            std::hint::unreachable_unchecked()
        };
        let v = operand::read_reg_f32(state, src);
        if v.is_nan() {
            state.trap = Some(RuntimeError::InvalidConversionToInt);
            return h_trap(state);
        }
        let t = v.trunc();
        if t < 0.0 || t >= (u64::MAX as f32) {
            state.trap = Some(RuntimeError::IntegerOverflow);
            return h_trap(state);
        }
        operand::write_dst_i64(state, dst, (t as u64) as i64);
        state.pc += 1;
        advance!(state)
    }
}
pub fn h_conv_i64_trunc_f64_s(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::ConversionReg { src, dst, .. } = instr else {
            std::hint::unreachable_unchecked()
        };
        let v = operand::read_reg_f64(state, src);
        if v.is_nan() {
            state.trap = Some(RuntimeError::InvalidConversionToInt);
            return h_trap(state);
        }
        let t = v.trunc();
        if t < (i64::MIN as f64) || t >= (i64::MAX as f64) {
            state.trap = Some(RuntimeError::IntegerOverflow);
            return h_trap(state);
        }
        operand::write_dst_i64(state, dst, t as i64);
        state.pc += 1;
        advance!(state)
    }
}
pub fn h_conv_i64_trunc_f64_u(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::ConversionReg { src, dst, .. } = instr else {
            std::hint::unreachable_unchecked()
        };
        let v = operand::read_reg_f64(state, src);
        if v.is_nan() {
            state.trap = Some(RuntimeError::InvalidConversionToInt);
            return h_trap(state);
        }
        let t = v.trunc();
        if t < 0.0 || t >= (u64::MAX as f64) {
            state.trap = Some(RuntimeError::IntegerOverflow);
            return h_trap(state);
        }
        operand::write_dst_i64(state, dst, (t as u64) as i64);
        state.pc += 1;
        advance!(state)
    }
}

// ============================================================================
// MemoryLoadReg handlers
// ============================================================================

/// Macro for memory load — N-byte read from `mem_ptr + addr + offset`,
/// extended/converted, written to RegOrLocal dst.
macro_rules! mem_load {
    ($name:ident, $ty:ty, $cast_to:ty, $write:ident, $convert:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::MemoryLoadReg {
                    addr, dst, offset, ..
                } = instr
                else {
                    std::hint::unreachable_unchecked()
                };
                let p = operand::read_i32(state, addr);
                let raw_ptr = state.mem_ptr.add((p as usize) + (*offset as usize)) as *const $ty;
                let v: $ty = std::ptr::read_unaligned(raw_ptr);
                let result: $cast_to = $convert(v);
                operand::$write(state, dst, result);
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

mem_load!(h_mem_load_i32, i32, i32, write_dst_i32, |v: i32| v);
mem_load!(h_mem_load_i64, i64, i64, write_dst_i64, |v: i64| v);
mem_load!(h_mem_load_f32, f32, f32, write_dst_f32, |v: f32| v);
mem_load!(h_mem_load_f64, f64, f64, write_dst_f64, |v: f64| v);
mem_load!(h_mem_load_i32_8s, i8, i32, write_dst_i32, |v: i8| v as i32);
mem_load!(h_mem_load_i32_8u, u8, i32, write_dst_i32, |v: u8| v as i32);
mem_load!(h_mem_load_i32_16s, i16, i32, write_dst_i32, |v: i16| v
    as i32);
mem_load!(h_mem_load_i32_16u, u16, i32, write_dst_i32, |v: u16| v
    as i32);
mem_load!(h_mem_load_i64_8s, i8, i64, write_dst_i64, |v: i8| v as i64);
mem_load!(h_mem_load_i64_8u, u8, i64, write_dst_i64, |v: u8| v as i64);
mem_load!(h_mem_load_i64_16s, i16, i64, write_dst_i64, |v: i16| v
    as i64);
mem_load!(h_mem_load_i64_16u, u16, i64, write_dst_i64, |v: u16| v
    as i64);
mem_load!(h_mem_load_i64_32s, i32, i64, write_dst_i64, |v: i32| v
    as i64);
mem_load!(h_mem_load_i64_32u, u32, i64, write_dst_i64, |v: u32| v
    as i64);

// ============================================================================
// MemoryStoreReg handlers
// ============================================================================

macro_rules! mem_store {
    ($name:ident, $read:ident, $store_ty:ty, $cast:expr) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::MemoryStoreReg {
                    addr,
                    value,
                    offset,
                    ..
                } = instr
                else {
                    std::hint::unreachable_unchecked()
                };
                let p = operand::read_i32(state, addr);
                let v = operand::$read(state, value);
                let raw_ptr =
                    state.mem_ptr.add((p as usize) + (*offset as usize)) as *mut $store_ty;
                std::ptr::write_unaligned(raw_ptr, $cast(v));
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

mem_store!(h_mem_store_i32, read_reg_i32, i32, |v: i32| v);
mem_store!(h_mem_store_i64, read_reg_i64, i64, |v: i64| v);
mem_store!(h_mem_store_f32, read_reg_f32, f32, |v: f32| v);
mem_store!(h_mem_store_f64, read_reg_f64, f64, |v: f64| v);
mem_store!(h_mem_store_i32_8, read_reg_i32, u8, |v: i32| v as u8);
mem_store!(h_mem_store_i32_16, read_reg_i32, u16, |v: i32| v as u16);
mem_store!(h_mem_store_i64_8, read_reg_i64, u8, |v: i64| v as u8);
mem_store!(h_mem_store_i64_16, read_reg_i64, u16, |v: i64| v as u16);
mem_store!(h_mem_store_i64_32, read_reg_i64, u32, |v: i64| v as u32);

// ============================================================================
// SelectReg handlers
// ============================================================================

macro_rules! select {
    ($name:ident, $get:ident, $set:ident) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::SelectReg {
                    dst,
                    val1,
                    val2,
                    cond,
                    ..
                } = instr
                else {
                    std::hint::unreachable_unchecked()
                };
                let regs = &mut *state.reg_file;
                let c = regs.get_i32(cond.index());
                let r = if c != 0 {
                    regs.$get(val1.index())
                } else {
                    regs.$get(val2.index())
                };
                regs.$set(dst.index(), r);
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

select!(h_select_i32, get_i32, set_i32);
select!(h_select_i64, get_i64, set_i64);
select!(h_select_f32, get_f32, set_f32);
select!(h_select_f64, get_f64, set_f64);

// ============================================================================
// Nop / Unreachable
// ============================================================================

pub fn h_nop(state: &mut VmState) -> Outcome {
    state.pc += 1;
    advance!(state)
}

pub fn h_unreachable(state: &mut VmState) -> Outcome {
    state.trap = Some(RuntimeError::Unreachable);
    h_trap(state)
}

// ============================================================================
// Control flow: Br / BrIf / BrTable / Block / Loop / If / End / Jump
// ============================================================================
//
// Semantics ported 1:1 from the legacy `run_dtc_loop` match arms in
// vm.rs:1511-1753 (see Phase 1 exploration findings). Key invariants:
//
// - All label stacks within a frame share the same `processed_instrs` Rc, so
//   `state.instrs` / `state.instrs_len` are invariant across push/pop.
// - `state.pc` is the source of truth for the active label's ip during
//   dispatch. `label_stack[*].ip` is stale and is only refreshed at
//   yield-to-runtime time (writeback handled by the dispatcher driver).
// - Br/BrIf/BrTable that escape the current function (relative_depth >
//   current_label_idx) return `Outcome::Halt` (matches legacy `break`).

pub fn h_br(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::BrReg {
            relative_depth,
            target_ip,
            source_regs,
            target_result_regs,
            ..
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let depth = *relative_depth as usize;
        let target_ip = *target_ip;
        if !source_regs.is_empty() && !target_result_regs.is_empty() {
            (*state.reg_file).copy_regs(source_regs, target_result_regs);
        }
        // For valid Wasm `relative_depth <= current_label_idx` is guaranteed by
        // the parser/validator. saturating_sub keeps the function safe without
        // the early-return path that blocks tail-call optimization.
        let target_level = state.current_label_idx.saturating_sub(depth);
        let keep_count = target_level.max(1);
        (*state.label_stack).truncate(keep_count);
        state.current_label_idx = (*state.label_stack).len() - 1;
        state.pc = target_ip;
        advance!(state)
    }
}

pub fn h_br_if(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::BrIfReg {
            relative_depth,
            target_ip,
            cond_reg,
            source_regs,
            target_result_regs,
            ..
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let depth = *relative_depth as usize;
        let target_ip = *target_ip;
        let cond_reg = *cond_reg;
        let cond = (*state.reg_file).get_i32(cond_reg.index());
        // Fall-through branch: condition false → just advance pc and tail-call.
        if cond == 0 {
            state.pc += 1;
            return advance!(state);
        }
        // Taken: copy result regs via slice (no ArrayVec) then truncate label stack.
        if !source_regs.is_empty() && !target_result_regs.is_empty() {
            (*state.reg_file).copy_regs(source_regs, target_result_regs);
        }
        let target_level = state.current_label_idx.saturating_sub(depth);
        let keep_count = target_level.max(1);
        (*state.label_stack).truncate(keep_count);
        state.current_label_idx = (*state.label_stack).len() - 1;
        state.pc = target_ip;
        advance!(state)
    }
}

pub fn h_br_table(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::BrTableReg {
            targets,
            default_target,
            index_reg,
            source_regs,
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let index_reg = *index_reg;
        let idx = (*state.reg_file).get_i32(index_reg.index()) as usize;

        let (depth, target_ip, target_result_regs_slice): (usize, usize, &[Reg]) =
            if idx < targets.len() {
                let (d, ip, rs) = &targets[idx];
                (*d as usize, *ip, &rs[..])
            } else {
                let (d, ip, rs) = default_target;
                (*d as usize, *ip, &rs[..])
            };

        if !source_regs.is_empty() && !target_result_regs_slice.is_empty() {
            (*state.reg_file).copy_regs(source_regs, target_result_regs_slice);
        }
        let target_level = state.current_label_idx.saturating_sub(depth);
        let keep_count = target_level.max(1);
        (*state.label_stack).truncate(keep_count);
        state.current_label_idx = (*state.label_stack).len() - 1;
        state.pc = target_ip;
        advance!(state)
    }
}

pub fn h_block(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::BlockReg {
            arity,
            param_count,
            is_loop,
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let arity = *arity;
        let param_count = *param_count;
        let is_loop = *is_loop;
        let next_ip = state.pc + 1;
        let pi_rc = (*state.label_stack)[state.current_label_idx]
            .processed_instrs
            .clone();
        let new_label = Label {
            locals_num: param_count,
            arity,
            is_loop,
            stack_height: 0,
            return_ip: next_ip,
        };
        (*state.label_stack).push(LabelStack {
            label: new_label,
            processed_instrs: pi_rc,
            ip: next_ip,
        });
        state.current_label_idx = (*state.label_stack).len() - 1;
        state.pc = next_ip;
        advance!(state)
    }
}

pub fn h_if(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::IfReg {
            arity,
            cond_reg,
            else_target_ip,
            has_else,
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let arity = *arity;
        let cond_reg = *cond_reg;
        let else_target_ip = *else_target_ip;
        let has_else = *has_else;

        let cond = (*state.reg_file).get_i32(cond_reg.index());
        // Only clone pi_rc inside branches that need it, so the no-else path
        // has zero `Rc` destructors at the tail call.
        if cond != 0 {
            let pi_rc = (*state.label_stack)[state.current_label_idx]
                .processed_instrs
                .clone();
            (*state.label_stack).push(LabelStack {
                label: Label {
                    locals_num: 0,
                    arity,
                    is_loop: false,
                    stack_height: 0,
                    return_ip: else_target_ip,
                },
                processed_instrs: pi_rc,
                ip: state.pc + 1,
            });
            state.current_label_idx = (*state.label_stack).len() - 1;
            state.pc += 1;
        } else if has_else {
            let pi_rc = (*state.label_stack)[state.current_label_idx]
                .processed_instrs
                .clone();
            (*state.label_stack).push(LabelStack {
                label: Label {
                    locals_num: 0,
                    arity,
                    is_loop: false,
                    stack_height: 0,
                    return_ip: else_target_ip,
                },
                processed_instrs: pi_rc,
                ip: else_target_ip,
            });
            state.current_label_idx = (*state.label_stack).len() - 1;
            state.pc = else_target_ip;
        } else {
            state.pc = else_target_ip;
        }
        advance!(state)
    }
}

pub fn h_end(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::EndReg {
            source_regs,
            target_result_regs,
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };

        // Decide whether this end is a function-level halt or a nested block pop.
        // To preserve tail-call optimization, we have a SINGLE call site at the
        // bottom (`advance!`). The halt path directs `state.pc` to the
        // `h_halt` sentinel (parser appends it at index `instrs_len`).
        let mut halt = false;
        if (*state.label_stack).len() <= 1 {
            halt = true;
        } else {
            (*state.reg_file).copy_regs(source_regs, target_result_regs);
            (*state.label_stack).pop();
            state.current_label_idx = (*state.label_stack).len() - 1;
            let next_ip = state.pc + 1;
            if next_ip >= state.instrs_len && state.current_label_idx == 0 {
                halt = true;
            } else {
                state.pc = next_ip;
            }
        }
        if halt {
            // Write source_regs directly to return_result_regs (no temp).
            let dst = &mut *state.return_result_regs;
            dst.clear();
            for r in source_regs.iter() {
                dst.push(*r);
            }
            // Dispatch to h_halt sentinel at handlers[instrs_len].
            state.pc = state.instrs_len;
        }
        advance!(state)
    }
}

pub fn h_jump(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::JumpReg { target_ip } = instr else {
            std::hint::unreachable_unchecked()
        };
        let target_ip = *target_ip;
        if (*state.label_stack).len() > 1 {
            (*state.label_stack).pop();
            state.current_label_idx = (*state.label_stack).len() - 1;
        }
        state.pc = target_ip;
        advance!(state)
    }
}

// ============================================================================
// Call / Return / CallIndirect / CallWasi (yield to runtime)
// ============================================================================
//
// These handlers prepare a `ModuleLevelInstr` in `state.yielded` and return
// `Outcome::Yield`. The dispatcher driver (in runtime.rs) handles frame
// transitions. State.pc is advanced to the post-call position so resume
// continues correctly.

pub fn h_call(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::CallReg {
            func_idx,
            param_regs,
            result_regs,
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let func_idx = *func_idx;
        let module_inst = &*state.module;
        let func_addr = match module_inst.func_addrs.get(func_idx.0 as usize) {
            Some(fa) => fa.clone(),
            None => {
                state.trap = Some(RuntimeError::ExportFuncNotFound);
                return h_trap(state);
            }
        };
        let regs = &*state.reg_file;
        let params: Vec<Val> = param_regs.iter().map(|r| regs.get_val(r)).collect();
        let result_regs_vec: ArrayVec<Reg, 8> = result_regs.iter().copied().collect();
        state.pc += 1;
        state.yielded = Some(ModuleLevelInstr::InvokeReg {
            func_addr,
            params,
            result_regs: result_regs_vec,
        });
        Outcome::Yield
    }
}

pub fn h_call_indirect(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::CallIndirectReg {
            type_idx,
            table_idx,
            index_reg,
            param_regs,
            result_regs,
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let type_idx = *type_idx;
        let table_idx = *table_idx;
        let index_reg = *index_reg;
        let module_inst = &*state.module;
        let i = (*state.reg_file).get_i32(index_reg.index());
        let table_addr = match module_inst.table_addrs.get(table_idx.0 as usize) {
            Some(t) => t,
            None => {
                state.trap = Some(RuntimeError::TableNotFound);
                return h_trap(state);
            }
        };
        let func_addr = match table_addr.get_func_addr(i as usize) {
            Some(fa) => fa,
            None => {
                state.trap = Some(RuntimeError::UninitializedElement);
                return h_trap(state);
            }
        };
        let actual_type = func_addr.func_type();
        let expected_type = &module_inst.types[type_idx.0 as usize];
        if *actual_type != *expected_type {
            state.trap = Some(RuntimeError::IndirectCallTypeMismatch);
            return h_trap(state);
        }
        let regs = &*state.reg_file;
        let params: Vec<Val> = param_regs.iter().map(|r| regs.get_val(r)).collect();
        let result_regs_vec: ArrayVec<Reg, 8> = result_regs.iter().copied().collect();
        state.pc += 1;
        state.yielded = Some(ModuleLevelInstr::InvokeReg {
            func_addr,
            params,
            result_regs: result_regs_vec,
        });
        Outcome::Yield
    }
}

pub fn h_call_wasi(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::CallWasiReg {
            wasi_func_type,
            param_regs,
            result_reg,
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let wasi_func_type = *wasi_func_type;
        let result_reg = *result_reg;
        let regs = &*state.reg_file;
        let params: Vec<Val> = param_regs.iter().map(|r| regs.get_val(r)).collect();
        state.pc += 1;
        state.yielded = Some(ModuleLevelInstr::InvokeWasiReg {
            wasi_func_type,
            params,
            result_reg,
        });
        Outcome::Yield
    }
}

pub fn h_return(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::ReturnReg { result_regs } = instr else {
            std::hint::unreachable_unchecked()
        };
        let rrr: ArrayVec<Reg, 8> = result_regs.iter().copied().collect();
        *state.return_result_regs = rrr;
        state.yielded = Some(ModuleLevelInstr::Return);
        Outcome::Yield
    }
}

// ============================================================================
// GlobalGetReg / GlobalSetReg
// ============================================================================

macro_rules! global_get {
    ($name:ident, $to:ident, $write:ident, $variant:ident) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::GlobalGetReg {
                    dst, global_index, ..
                } = instr
                else {
                    std::hint::unreachable_unchecked()
                };
                let module_inst = &*state.module;
                let global_addr = module_inst
                    .global_addrs
                    .get_by_idx(crate::structure::types::GlobalIdx(*global_index))
                    .clone();
                let val = global_addr.get();
                let v = val.$to().unwrap_or(Default::default());
                operand::$write(state, dst, v);
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

global_get!(h_global_get_i32, to_i32, write_dst_i32, I32);
global_get!(h_global_get_i64, to_i64, write_dst_i64, I64);
global_get!(h_global_get_f32, to_f32, write_dst_f32, F32);
global_get!(h_global_get_f64, to_f64, write_dst_f64, F64);

macro_rules! global_set {
    ($name:ident, $get:ident, $variant:ident) => {
        pub fn $name(state: &mut VmState) -> Outcome {
            unsafe {
                let instr = &*state.instrs.add(state.pc);
                let ProcessedInstr::GlobalSetReg {
                    src, global_index, ..
                } = instr
                else {
                    std::hint::unreachable_unchecked()
                };
                let v = match src {
                    RegOrLocal::Reg(idx) => (*state.reg_file).$get(*idx),
                    RegOrLocal::Local(idx) => match &*state.locals.add(*idx as usize) {
                        Val::Num(crate::execution::value::Num::$variant(v)) => *v,
                        _ => Default::default(),
                    },
                };
                let module_inst = &*state.module;
                let global_addr = module_inst
                    .global_addrs
                    .get_by_idx(crate::structure::types::GlobalIdx(*global_index))
                    .clone();
                if let Err(e) = global_addr.set(Val::Num(crate::execution::value::Num::$variant(v)))
                {
                    state.trap = Some(e);
                    return h_trap(state);
                }
                state.pc += 1;
                advance!(state)
            }
        }
    };
}

global_set!(h_global_set_i32, get_i32, I32);
global_set!(h_global_set_i64, get_i64, I64);
global_set!(h_global_set_f32, get_f32, F32);
global_set!(h_global_set_f64, get_f64, F64);

// ============================================================================
// DataDrop
// ============================================================================

pub fn h_data_drop(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::DataDropReg { data_index } = instr else {
            std::hint::unreachable_unchecked()
        };
        let module_inst = &*state.module;
        if (*data_index as usize) < module_inst.data_addrs.len() {
            module_inst.data_addrs[*data_index as usize].drop_data();
        }
        state.pc += 1;
        advance!(state)
    }
}

// ============================================================================
// RefLocalReg
// ============================================================================

pub fn h_ref_local_get(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::RefLocalReg { dst, local_idx, .. } = instr else {
            std::hint::unreachable_unchecked()
        };
        let dst = *dst;
        let local_idx = *local_idx as usize;
        if local_idx >= state.locals_len {
            state.trap = Some(RuntimeError::LocalIndexOutOfBounds);
            return h_trap(state);
        }
        let val = (&*state.locals.add(local_idx)).clone();
        if let Val::Ref(r) = val {
            (*state.reg_file).set_ref(dst, r);
        }
        state.pc += 1;
        advance!(state)
    }
}

pub fn h_ref_local_set(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::RefLocalReg { src, local_idx, .. } = instr else {
            std::hint::unreachable_unchecked()
        };
        let src = *src;
        let local_idx = *local_idx as usize;
        let ref_val = (*state.reg_file).get_ref(src);
        if local_idx < state.locals_len {
            *state.locals.add(local_idx) = Val::Ref(ref_val);
        }
        state.pc += 1;
        advance!(state)
    }
}

// ============================================================================
// TableRefReg (ref.null / ref.is_null / table.get / table.set / table.fill)
// ============================================================================

pub fn h_ref_null(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::TableRefReg { regs, .. } = instr else {
            std::hint::unreachable_unchecked()
        };
        (*state.reg_file).set_ref(regs[0], crate::execution::value::Ref::RefNull);
        state.pc += 1;
        advance!(state)
    }
}

pub fn h_ref_is_null(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::TableRefReg { regs, .. } = instr else {
            std::hint::unreachable_unchecked()
        };
        let ref_val = (*state.reg_file).get_ref(regs[1]);
        let is_null = matches!(ref_val, crate::execution::value::Ref::RefNull) as i32;
        (*state.reg_file).set_i32(regs[0], is_null);
        state.pc += 1;
        advance!(state)
    }
}

pub fn h_table_get(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::TableRefReg {
            table_idx, regs, ..
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let module_inst = &*state.module;
        let table_addr = match module_inst.table_addrs.get(*table_idx as usize) {
            Some(t) => t,
            None => {
                state.trap = Some(RuntimeError::TableNotFound);
                return h_trap(state);
            }
        };
        let index = (*state.reg_file).get_i32(regs[1]) as usize;
        let val = table_addr.get(index);
        match val {
            Val::Ref(r) => {
                (*state.reg_file).set_ref(regs[0], r);
                state.pc += 1;
                advance!(state)
            }
            _ => {
                state.trap = Some(RuntimeError::TypeMismatch);
                h_trap(state)
            }
        }
    }
}

pub fn h_table_set(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::TableRefReg {
            table_idx, regs, ..
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let module_inst = &*state.module;
        let table_addr = match module_inst.table_addrs.get(*table_idx as usize) {
            Some(t) => t,
            None => {
                state.trap = Some(RuntimeError::TableNotFound);
                return h_trap(state);
            }
        };
        let index = (*state.reg_file).get_i32(regs[0]) as usize;
        let ref_val = (*state.reg_file).get_ref(regs[1]);
        if let Err(e) = table_addr.set(index, Val::Ref(ref_val)) {
            state.trap = Some(e);
            return h_trap(state);
        }
        state.pc += 1;
        advance!(state)
    }
}

pub fn h_table_fill(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::TableRefReg {
            table_idx, regs, ..
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let module_inst = &*state.module;
        let table_addr = match module_inst.table_addrs.get(*table_idx as usize) {
            Some(t) => t,
            None => {
                state.trap = Some(RuntimeError::TableNotFound);
                return h_trap(state);
            }
        };
        let i = (*state.reg_file).get_i32(regs[0]) as usize;
        let ref_val = (*state.reg_file).get_ref(regs[1]);
        let n = (*state.reg_file).get_i32(regs[2]) as usize;
        if let Err(e) = table_addr.fill(i, Val::Ref(ref_val), n) {
            state.trap = Some(e);
            return h_trap(state);
        }
        state.pc += 1;
        advance!(state)
    }
}

// ============================================================================
// MemoryOpsReg (memory.size / grow / copy / init / fill)
// ============================================================================

pub fn h_mem_size(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::MemoryOpsReg { dst, .. } = instr else {
            std::hint::unreachable_unchecked()
        };
        let module_inst = &*state.module;
        let mem_addr = match module_inst.mem_addrs.first() {
            Some(m) => m,
            None => {
                state.trap = Some(RuntimeError::MemoryNotFound);
                return h_trap(state);
            }
        };
        let size = mem_addr.mem_size();
        if let Some(d) = dst {
            (*state.reg_file).set_i32(d.index(), size);
        }
        state.pc += 1;
        advance!(state)
    }
}

pub fn h_mem_grow(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::MemoryOpsReg { dst, args, .. } = instr else {
            std::hint::unreachable_unchecked()
        };
        let module_inst = &*state.module;
        let mem_addr = match module_inst.mem_addrs.first() {
            Some(m) => m,
            None => {
                state.trap = Some(RuntimeError::MemoryNotFound);
                return h_trap(state);
            }
        };
        let delta = (*state.reg_file).get_i32(args[0].index());
        let delta_u32: u32 = match delta.try_into() {
            Ok(v) => v,
            Err(_) => {
                state.trap = Some(RuntimeError::InvalidParameterCount);
                return h_trap(state);
            }
        };
        let prev_size = mem_addr.mem_grow(delta_u32 as i32);
        if let Some(d) = dst {
            (*state.reg_file).set_i32(d.index(), prev_size);
        }
        // Refresh cached memory pointer (Vec may have reallocated)
        state.mem_ptr = mem_addr.data_ptr();
        state.pc += 1;
        advance!(state)
    }
}

pub fn h_mem_copy(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::MemoryOpsReg { args, .. } = instr else {
            std::hint::unreachable_unchecked()
        };
        let module_inst = &*state.module;
        let mem_addr = match module_inst.mem_addrs.first() {
            Some(m) => m,
            None => {
                state.trap = Some(RuntimeError::MemoryNotFound);
                return h_trap(state);
            }
        };
        let regs = &*state.reg_file;
        let dest = regs.get_i32(args[0].index());
        let src = regs.get_i32(args[1].index());
        let len = regs.get_i32(args[2].index());
        mem_addr.memory_copy(dest, src, len);
        state.pc += 1;
        advance!(state)
    }
}

pub fn h_mem_init(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::MemoryOpsReg {
            args, data_index, ..
        } = instr
        else {
            std::hint::unreachable_unchecked()
        };
        let module_inst = &*state.module;
        let mem_addr = match module_inst.mem_addrs.first() {
            Some(m) => m,
            None => {
                state.trap = Some(RuntimeError::MemoryNotFound);
                return h_trap(state);
            }
        };
        let regs = &*state.reg_file;
        let dest = regs.get_i32(args[0].index()) as usize;
        let offset = regs.get_i32(args[1].index()) as usize;
        let len = regs.get_i32(args[2].index()) as usize;
        if (*data_index as usize) >= module_inst.data_addrs.len() {
            state.trap = Some(RuntimeError::InvalidDataSegmentIndex);
            return h_trap(state);
        }
        let data_addr = &module_inst.data_addrs[*data_index as usize];
        let data_bytes = data_addr.get_data();
        if len > 0 {
            mem_addr.init(dest, &data_bytes[offset..offset + len]);
        }
        state.pc += 1;
        advance!(state)
    }
}

pub fn h_mem_fill(state: &mut VmState) -> Outcome {
    unsafe {
        let instr = &*state.instrs.add(state.pc);
        let ProcessedInstr::MemoryOpsReg { args, .. } = instr else {
            std::hint::unreachable_unchecked()
        };
        let module_inst = &*state.module;
        let mem_addr = match module_inst.mem_addrs.first() {
            Some(m) => m,
            None => {
                state.trap = Some(RuntimeError::MemoryNotFound);
                return h_trap(state);
            }
        };
        let regs = &*state.reg_file;
        let dest = regs.get_i32(args[0].index());
        let val = regs.get_i32(args[1].index()) as u8;
        let size = regs.get_i32(args[2].index());
        mem_addr.memory_fill(dest, val, size);
        state.pc += 1;
        advance!(state)
    }
}

// ============================================================================
// select_handler — map ProcessedInstr → Handler
// ============================================================================

pub fn select_handler(instr: &ProcessedInstr) -> Handler {
    match instr {
        ProcessedInstr::I32Reg { handler_index, .. } => match *handler_index {
            vm::HANDLER_IDX_LOCAL_GET => h_i32_local_get,
            vm::HANDLER_IDX_LOCAL_SET => h_i32_local_set,
            vm::HANDLER_IDX_I32_CONST => h_i32_const,
            vm::HANDLER_IDX_I32_ADD => h_i32_add,
            vm::HANDLER_IDX_I32_SUB => h_i32_sub,
            vm::HANDLER_IDX_I32_MUL => h_i32_mul,
            vm::HANDLER_IDX_I32_DIV_S => h_i32_div_s,
            vm::HANDLER_IDX_I32_DIV_U => h_i32_div_u,
            vm::HANDLER_IDX_I32_REM_S => h_i32_rem_s,
            vm::HANDLER_IDX_I32_REM_U => h_i32_rem_u,
            vm::HANDLER_IDX_I32_AND => h_i32_and,
            vm::HANDLER_IDX_I32_OR => h_i32_or,
            vm::HANDLER_IDX_I32_XOR => h_i32_xor,
            vm::HANDLER_IDX_I32_SHL => h_i32_shl,
            vm::HANDLER_IDX_I32_SHR_S => h_i32_shr_s,
            vm::HANDLER_IDX_I32_SHR_U => h_i32_shr_u,
            vm::HANDLER_IDX_I32_ROTL => h_i32_rotl,
            vm::HANDLER_IDX_I32_ROTR => h_i32_rotr,
            vm::HANDLER_IDX_I32_EQ => h_i32_eq,
            vm::HANDLER_IDX_I32_NE => h_i32_ne,
            vm::HANDLER_IDX_I32_LT_S => h_i32_lt_s,
            vm::HANDLER_IDX_I32_LT_U => h_i32_lt_u,
            vm::HANDLER_IDX_I32_LE_S => h_i32_le_s,
            vm::HANDLER_IDX_I32_LE_U => h_i32_le_u,
            vm::HANDLER_IDX_I32_GT_S => h_i32_gt_s,
            vm::HANDLER_IDX_I32_GT_U => h_i32_gt_u,
            vm::HANDLER_IDX_I32_GE_S => h_i32_ge_s,
            vm::HANDLER_IDX_I32_GE_U => h_i32_ge_u,
            vm::HANDLER_IDX_I32_CLZ => h_i32_clz,
            vm::HANDLER_IDX_I32_CTZ => h_i32_ctz,
            vm::HANDLER_IDX_I32_POPCNT => h_i32_popcnt,
            vm::HANDLER_IDX_I32_EQZ => h_i32_eqz,
            vm::HANDLER_IDX_I32_EXTEND8_S => h_i32_extend8_s,
            vm::HANDLER_IDX_I32_EXTEND16_S => h_i32_extend16_s,
            _ => h_invalid,
        },
        ProcessedInstr::I64Reg { handler_index, .. } => match *handler_index {
            vm::HANDLER_IDX_LOCAL_GET => h_i64_local_get,
            vm::HANDLER_IDX_LOCAL_SET => h_i64_local_set,
            vm::HANDLER_IDX_I64_CONST => h_i64_const,
            vm::HANDLER_IDX_I64_ADD => h_i64_add,
            vm::HANDLER_IDX_I64_SUB => h_i64_sub,
            vm::HANDLER_IDX_I64_MUL => h_i64_mul,
            vm::HANDLER_IDX_I64_DIV_S => h_i64_div_s,
            vm::HANDLER_IDX_I64_DIV_U => h_i64_div_u,
            vm::HANDLER_IDX_I64_REM_S => h_i64_rem_s,
            vm::HANDLER_IDX_I64_REM_U => h_i64_rem_u,
            vm::HANDLER_IDX_I64_AND => h_i64_and,
            vm::HANDLER_IDX_I64_OR => h_i64_or,
            vm::HANDLER_IDX_I64_XOR => h_i64_xor,
            vm::HANDLER_IDX_I64_SHL => h_i64_shl,
            vm::HANDLER_IDX_I64_SHR_S => h_i64_shr_s,
            vm::HANDLER_IDX_I64_SHR_U => h_i64_shr_u,
            vm::HANDLER_IDX_I64_ROTL => h_i64_rotl,
            vm::HANDLER_IDX_I64_ROTR => h_i64_rotr,
            vm::HANDLER_IDX_I64_EQ => h_i64_eq,
            vm::HANDLER_IDX_I64_NE => h_i64_ne,
            vm::HANDLER_IDX_I64_LT_S => h_i64_lt_s,
            vm::HANDLER_IDX_I64_LT_U => h_i64_lt_u,
            vm::HANDLER_IDX_I64_LE_S => h_i64_le_s,
            vm::HANDLER_IDX_I64_LE_U => h_i64_le_u,
            vm::HANDLER_IDX_I64_GT_S => h_i64_gt_s,
            vm::HANDLER_IDX_I64_GT_U => h_i64_gt_u,
            vm::HANDLER_IDX_I64_GE_S => h_i64_ge_s,
            vm::HANDLER_IDX_I64_GE_U => h_i64_ge_u,
            vm::HANDLER_IDX_I64_CLZ => h_i64_clz,
            vm::HANDLER_IDX_I64_CTZ => h_i64_ctz,
            vm::HANDLER_IDX_I64_POPCNT => h_i64_popcnt,
            vm::HANDLER_IDX_I64_EQZ => h_i64_eqz,
            vm::HANDLER_IDX_I64_EXTEND8_S => h_i64_extend8_s,
            vm::HANDLER_IDX_I64_EXTEND16_S => h_i64_extend16_s,
            vm::HANDLER_IDX_I64_EXTEND32_S => h_i64_extend32_s,
            _ => h_invalid,
        },
        ProcessedInstr::F32Reg { handler_index, .. } => match *handler_index {
            vm::HANDLER_IDX_LOCAL_GET => h_f32_local_get,
            vm::HANDLER_IDX_LOCAL_SET => h_f32_local_set,
            vm::HANDLER_IDX_F32_CONST => h_f32_const,
            vm::HANDLER_IDX_F32_ADD => h_f32_add,
            vm::HANDLER_IDX_F32_SUB => h_f32_sub,
            vm::HANDLER_IDX_F32_MUL => h_f32_mul,
            vm::HANDLER_IDX_F32_DIV => h_f32_div,
            vm::HANDLER_IDX_F32_MIN => h_f32_min,
            vm::HANDLER_IDX_F32_MAX => h_f32_max,
            vm::HANDLER_IDX_F32_COPYSIGN => h_f32_copysign,
            vm::HANDLER_IDX_F32_ABS => h_f32_abs,
            vm::HANDLER_IDX_F32_NEG => h_f32_neg,
            vm::HANDLER_IDX_F32_CEIL => h_f32_ceil,
            vm::HANDLER_IDX_F32_FLOOR => h_f32_floor,
            vm::HANDLER_IDX_F32_TRUNC => h_f32_trunc,
            vm::HANDLER_IDX_F32_NEAREST => h_f32_nearest,
            vm::HANDLER_IDX_F32_SQRT => h_f32_sqrt,
            vm::HANDLER_IDX_F32_EQ => h_f32_eq,
            vm::HANDLER_IDX_F32_NE => h_f32_ne,
            vm::HANDLER_IDX_F32_LT => h_f32_lt,
            vm::HANDLER_IDX_F32_GT => h_f32_gt,
            vm::HANDLER_IDX_F32_LE => h_f32_le,
            vm::HANDLER_IDX_F32_GE => h_f32_ge,
            _ => h_invalid,
        },
        ProcessedInstr::F64Reg { handler_index, .. } => match *handler_index {
            vm::HANDLER_IDX_LOCAL_GET => h_f64_local_get,
            vm::HANDLER_IDX_LOCAL_SET => h_f64_local_set,
            vm::HANDLER_IDX_F64_CONST => h_f64_const,
            vm::HANDLER_IDX_F64_ADD => h_f64_add,
            vm::HANDLER_IDX_F64_SUB => h_f64_sub,
            vm::HANDLER_IDX_F64_MUL => h_f64_mul,
            vm::HANDLER_IDX_F64_DIV => h_f64_div,
            vm::HANDLER_IDX_F64_MIN => h_f64_min,
            vm::HANDLER_IDX_F64_MAX => h_f64_max,
            vm::HANDLER_IDX_F64_COPYSIGN => h_f64_copysign,
            vm::HANDLER_IDX_F64_ABS => h_f64_abs,
            vm::HANDLER_IDX_F64_NEG => h_f64_neg,
            vm::HANDLER_IDX_F64_CEIL => h_f64_ceil,
            vm::HANDLER_IDX_F64_FLOOR => h_f64_floor,
            vm::HANDLER_IDX_F64_TRUNC => h_f64_trunc,
            vm::HANDLER_IDX_F64_NEAREST => h_f64_nearest,
            vm::HANDLER_IDX_F64_SQRT => h_f64_sqrt,
            vm::HANDLER_IDX_F64_EQ => h_f64_eq,
            vm::HANDLER_IDX_F64_NE => h_f64_ne,
            vm::HANDLER_IDX_F64_LT => h_f64_lt,
            vm::HANDLER_IDX_F64_GT => h_f64_gt,
            vm::HANDLER_IDX_F64_LE => h_f64_le,
            vm::HANDLER_IDX_F64_GE => h_f64_ge,
            _ => h_invalid,
        },
        ProcessedInstr::ConversionReg { handler_index, .. } => match *handler_index {
            vm::HANDLER_IDX_I64_EXTEND_I32_S => h_conv_i64_extend_i32_s,
            vm::HANDLER_IDX_I64_EXTEND_I32_U => h_conv_i64_extend_i32_u,
            vm::HANDLER_IDX_I32_WRAP_I64 => h_conv_i32_wrap_i64,
            vm::HANDLER_IDX_I32_TRUNC_F32_S => h_conv_i32_trunc_f32_s,
            vm::HANDLER_IDX_I32_TRUNC_F32_U => h_conv_i32_trunc_f32_u,
            vm::HANDLER_IDX_I32_TRUNC_F64_S => h_conv_i32_trunc_f64_s,
            vm::HANDLER_IDX_I32_TRUNC_F64_U => h_conv_i32_trunc_f64_u,
            vm::HANDLER_IDX_I64_TRUNC_F32_S => h_conv_i64_trunc_f32_s,
            vm::HANDLER_IDX_I64_TRUNC_F32_U => h_conv_i64_trunc_f32_u,
            vm::HANDLER_IDX_I64_TRUNC_F64_S => h_conv_i64_trunc_f64_s,
            vm::HANDLER_IDX_I64_TRUNC_F64_U => h_conv_i64_trunc_f64_u,
            vm::HANDLER_IDX_I32_TRUNC_SAT_F32_S => h_conv_i32_trunc_sat_f32_s,
            vm::HANDLER_IDX_I32_TRUNC_SAT_F32_U => h_conv_i32_trunc_sat_f32_u,
            vm::HANDLER_IDX_I32_TRUNC_SAT_F64_S => h_conv_i32_trunc_sat_f64_s,
            vm::HANDLER_IDX_I32_TRUNC_SAT_F64_U => h_conv_i32_trunc_sat_f64_u,
            vm::HANDLER_IDX_I64_TRUNC_SAT_F32_S => h_conv_i64_trunc_sat_f32_s,
            vm::HANDLER_IDX_I64_TRUNC_SAT_F32_U => h_conv_i64_trunc_sat_f32_u,
            vm::HANDLER_IDX_I64_TRUNC_SAT_F64_S => h_conv_i64_trunc_sat_f64_s,
            vm::HANDLER_IDX_I64_TRUNC_SAT_F64_U => h_conv_i64_trunc_sat_f64_u,
            vm::HANDLER_IDX_F32_CONVERT_I32_S => h_conv_f32_convert_i32_s,
            vm::HANDLER_IDX_F32_CONVERT_I32_U => h_conv_f32_convert_i32_u,
            vm::HANDLER_IDX_F32_CONVERT_I64_S => h_conv_f32_convert_i64_s,
            vm::HANDLER_IDX_F32_CONVERT_I64_U => h_conv_f32_convert_i64_u,
            vm::HANDLER_IDX_F64_CONVERT_I32_S => h_conv_f64_convert_i32_s,
            vm::HANDLER_IDX_F64_CONVERT_I32_U => h_conv_f64_convert_i32_u,
            vm::HANDLER_IDX_F64_CONVERT_I64_S => h_conv_f64_convert_i64_s,
            vm::HANDLER_IDX_F64_CONVERT_I64_U => h_conv_f64_convert_i64_u,
            vm::HANDLER_IDX_F32_DEMOTE_F64 => h_conv_f32_demote_f64,
            vm::HANDLER_IDX_F64_PROMOTE_F32 => h_conv_f64_promote_f32,
            vm::HANDLER_IDX_I32_REINTERPRET_F32 => h_conv_i32_reinterpret_f32,
            vm::HANDLER_IDX_F32_REINTERPRET_I32 => h_conv_f32_reinterpret_i32,
            vm::HANDLER_IDX_I64_REINTERPRET_F64 => h_conv_i64_reinterpret_f64,
            vm::HANDLER_IDX_F64_REINTERPRET_I64 => h_conv_f64_reinterpret_i64,
            _ => h_invalid,
        },
        ProcessedInstr::MemoryLoadReg { handler_index, .. } => match *handler_index {
            vm::HANDLER_IDX_I32_LOAD => h_mem_load_i32,
            vm::HANDLER_IDX_I64_LOAD => h_mem_load_i64,
            vm::HANDLER_IDX_F32_LOAD => h_mem_load_f32,
            vm::HANDLER_IDX_F64_LOAD => h_mem_load_f64,
            vm::HANDLER_IDX_I32_LOAD8_S => h_mem_load_i32_8s,
            vm::HANDLER_IDX_I32_LOAD8_U => h_mem_load_i32_8u,
            vm::HANDLER_IDX_I32_LOAD16_S => h_mem_load_i32_16s,
            vm::HANDLER_IDX_I32_LOAD16_U => h_mem_load_i32_16u,
            vm::HANDLER_IDX_I64_LOAD8_S => h_mem_load_i64_8s,
            vm::HANDLER_IDX_I64_LOAD8_U => h_mem_load_i64_8u,
            vm::HANDLER_IDX_I64_LOAD16_S => h_mem_load_i64_16s,
            vm::HANDLER_IDX_I64_LOAD16_U => h_mem_load_i64_16u,
            vm::HANDLER_IDX_I64_LOAD32_S => h_mem_load_i64_32s,
            vm::HANDLER_IDX_I64_LOAD32_U => h_mem_load_i64_32u,
            _ => h_invalid,
        },
        ProcessedInstr::MemoryStoreReg { handler_index, .. } => match *handler_index {
            vm::HANDLER_IDX_I32_STORE => h_mem_store_i32,
            vm::HANDLER_IDX_I64_STORE => h_mem_store_i64,
            vm::HANDLER_IDX_F32_STORE => h_mem_store_f32,
            vm::HANDLER_IDX_F64_STORE => h_mem_store_f64,
            vm::HANDLER_IDX_I32_STORE8 => h_mem_store_i32_8,
            vm::HANDLER_IDX_I32_STORE16 => h_mem_store_i32_16,
            vm::HANDLER_IDX_I64_STORE8 => h_mem_store_i64_8,
            vm::HANDLER_IDX_I64_STORE16 => h_mem_store_i64_16,
            vm::HANDLER_IDX_I64_STORE32 => h_mem_store_i64_32,
            _ => h_invalid,
        },
        ProcessedInstr::MemoryOpsReg { handler_index, .. } => match *handler_index {
            vm::HANDLER_IDX_MEMORY_SIZE => h_mem_size,
            vm::HANDLER_IDX_MEMORY_GROW => h_mem_grow,
            vm::HANDLER_IDX_MEMORY_COPY => h_mem_copy,
            vm::HANDLER_IDX_MEMORY_INIT => h_mem_init,
            vm::HANDLER_IDX_MEMORY_FILL => h_mem_fill,
            _ => h_invalid,
        },
        ProcessedInstr::SelectReg { handler_index, .. } => match *handler_index {
            vm::HANDLER_IDX_SELECT_I32 => h_select_i32,
            vm::HANDLER_IDX_SELECT_I64 => h_select_i64,
            vm::HANDLER_IDX_SELECT_F32 => h_select_f32,
            vm::HANDLER_IDX_SELECT_F64 => h_select_f64,
            _ => h_invalid,
        },
        ProcessedInstr::GlobalGetReg { handler_index, .. } => match *handler_index {
            vm::HANDLER_IDX_GLOBAL_GET_I32 => h_global_get_i32,
            vm::HANDLER_IDX_GLOBAL_GET_I64 => h_global_get_i64,
            vm::HANDLER_IDX_GLOBAL_GET_F32 => h_global_get_f32,
            vm::HANDLER_IDX_GLOBAL_GET_F64 => h_global_get_f64,
            _ => h_invalid,
        },
        ProcessedInstr::GlobalSetReg { handler_index, .. } => match *handler_index {
            vm::HANDLER_IDX_GLOBAL_SET_I32 => h_global_set_i32,
            vm::HANDLER_IDX_GLOBAL_SET_I64 => h_global_set_i64,
            vm::HANDLER_IDX_GLOBAL_SET_F32 => h_global_set_f32,
            vm::HANDLER_IDX_GLOBAL_SET_F64 => h_global_set_f64,
            _ => h_invalid,
        },
        ProcessedInstr::RefLocalReg { handler_index, .. } => match *handler_index {
            vm::HANDLER_IDX_REF_LOCAL_GET_REG => h_ref_local_get,
            vm::HANDLER_IDX_REF_LOCAL_SET_REG => h_ref_local_set,
            _ => h_invalid,
        },
        ProcessedInstr::TableRefReg { handler_index, .. } => match *handler_index {
            vm::HANDLER_IDX_REF_NULL_REG => h_ref_null,
            vm::HANDLER_IDX_REF_IS_NULL_REG => h_ref_is_null,
            vm::HANDLER_IDX_TABLE_GET_REG => h_table_get,
            vm::HANDLER_IDX_TABLE_SET_REG => h_table_set,
            vm::HANDLER_IDX_TABLE_FILL_REG => h_table_fill,
            _ => h_invalid,
        },
        ProcessedInstr::DataDropReg { .. } => h_data_drop,
        ProcessedInstr::CallReg { .. } => h_call,
        ProcessedInstr::CallIndirectReg { .. } => h_call_indirect,
        ProcessedInstr::CallWasiReg { .. } => h_call_wasi,
        ProcessedInstr::ReturnReg { .. } => h_return,
        ProcessedInstr::JumpReg { .. } => h_jump,
        ProcessedInstr::BlockReg { .. } => h_block,
        ProcessedInstr::IfReg { .. } => h_if,
        ProcessedInstr::EndReg { .. } => h_end,
        ProcessedInstr::BrReg { .. } => h_br,
        ProcessedInstr::BrIfReg { .. } => h_br_if,
        ProcessedInstr::BrTableReg { .. } => h_br_table,
        ProcessedInstr::NopReg => h_nop,
        ProcessedInstr::UnreachableReg => h_unreachable,
    }
}

/// Build a parallel handler array from a slice of processed instructions.
pub fn build_handlers(instrs: &[ProcessedInstr]) -> Vec<Handler> {
    instrs.iter().map(select_handler).collect()
}
