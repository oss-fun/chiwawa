//! Pure (ABI-independent) Wasm numeric operations.
//!
//! Each function corresponds to one Wasm instruction's pure semantics
//! (no state access, no traps unless the operation itself traps).
//! Used by handlers in both `dispatch_loop` and `dispatch_tco` modes.

// ============================================================================
// I32 binary
// ============================================================================

#[inline(always)]
pub fn i32_add(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}
#[inline(always)]
pub fn i32_sub(a: i32, b: i32) -> i32 {
    a.wrapping_sub(b)
}
#[inline(always)]
pub fn i32_mul(a: i32, b: i32) -> i32 {
    a.wrapping_mul(b)
}
#[inline(always)]
pub fn i32_and(a: i32, b: i32) -> i32 {
    a & b
}
#[inline(always)]
pub fn i32_or(a: i32, b: i32) -> i32 {
    a | b
}
#[inline(always)]
pub fn i32_xor(a: i32, b: i32) -> i32 {
    a ^ b
}
#[inline(always)]
pub fn i32_shl(a: i32, b: i32) -> i32 {
    a.wrapping_shl(b as u32)
}
#[inline(always)]
pub fn i32_shr_s(a: i32, b: i32) -> i32 {
    a.wrapping_shr(b as u32)
}
#[inline(always)]
pub fn i32_shr_u(a: i32, b: i32) -> i32 {
    ((a as u32).wrapping_shr(b as u32)) as i32
}
#[inline(always)]
pub fn i32_rotl(a: i32, b: i32) -> i32 {
    a.rotate_left(b as u32)
}
#[inline(always)]
pub fn i32_rotr(a: i32, b: i32) -> i32 {
    a.rotate_right(b as u32)
}

// I32 comparisons (return 0 or 1)
#[inline(always)]
pub fn i32_eq(a: i32, b: i32) -> i32 {
    (a == b) as i32
}
#[inline(always)]
pub fn i32_ne(a: i32, b: i32) -> i32 {
    (a != b) as i32
}
#[inline(always)]
pub fn i32_lt_s(a: i32, b: i32) -> i32 {
    (a < b) as i32
}
#[inline(always)]
pub fn i32_lt_u(a: i32, b: i32) -> i32 {
    ((a as u32) < (b as u32)) as i32
}
#[inline(always)]
pub fn i32_le_s(a: i32, b: i32) -> i32 {
    (a <= b) as i32
}
#[inline(always)]
pub fn i32_le_u(a: i32, b: i32) -> i32 {
    ((a as u32) <= (b as u32)) as i32
}
#[inline(always)]
pub fn i32_gt_s(a: i32, b: i32) -> i32 {
    (a > b) as i32
}
#[inline(always)]
pub fn i32_gt_u(a: i32, b: i32) -> i32 {
    ((a as u32) > (b as u32)) as i32
}
#[inline(always)]
pub fn i32_ge_s(a: i32, b: i32) -> i32 {
    (a >= b) as i32
}
#[inline(always)]
pub fn i32_ge_u(a: i32, b: i32) -> i32 {
    ((a as u32) >= (b as u32)) as i32
}

// I32 unary
#[inline(always)]
pub fn i32_clz(a: i32) -> i32 {
    a.leading_zeros() as i32
}
#[inline(always)]
pub fn i32_ctz(a: i32) -> i32 {
    a.trailing_zeros() as i32
}
#[inline(always)]
pub fn i32_popcnt(a: i32) -> i32 {
    a.count_ones() as i32
}
#[inline(always)]
pub fn i32_eqz(a: i32) -> i32 {
    (a == 0) as i32
}
#[inline(always)]
pub fn i32_extend8_s(a: i32) -> i32 {
    (a as i8) as i32
}
#[inline(always)]
pub fn i32_extend16_s(a: i32) -> i32 {
    (a as i16) as i32
}

// ============================================================================
// I64 binary
// ============================================================================

#[inline(always)]
pub fn i64_add(a: i64, b: i64) -> i64 {
    a.wrapping_add(b)
}
#[inline(always)]
pub fn i64_sub(a: i64, b: i64) -> i64 {
    a.wrapping_sub(b)
}
#[inline(always)]
pub fn i64_mul(a: i64, b: i64) -> i64 {
    a.wrapping_mul(b)
}
#[inline(always)]
pub fn i64_and(a: i64, b: i64) -> i64 {
    a & b
}
#[inline(always)]
pub fn i64_or(a: i64, b: i64) -> i64 {
    a | b
}
#[inline(always)]
pub fn i64_xor(a: i64, b: i64) -> i64 {
    a ^ b
}
#[inline(always)]
pub fn i64_shl(a: i64, b: i64) -> i64 {
    a.wrapping_shl(b as u32)
}
#[inline(always)]
pub fn i64_shr_s(a: i64, b: i64) -> i64 {
    a.wrapping_shr(b as u32)
}
#[inline(always)]
pub fn i64_shr_u(a: i64, b: i64) -> i64 {
    ((a as u64).wrapping_shr(b as u32)) as i64
}
#[inline(always)]
pub fn i64_rotl(a: i64, b: i64) -> i64 {
    a.rotate_left(b as u32)
}
#[inline(always)]
pub fn i64_rotr(a: i64, b: i64) -> i64 {
    a.rotate_right(b as u32)
}

// I64 comparisons (return 0 or 1, i32-typed result)
#[inline(always)]
pub fn i64_eq(a: i64, b: i64) -> i32 {
    (a == b) as i32
}
#[inline(always)]
pub fn i64_ne(a: i64, b: i64) -> i32 {
    (a != b) as i32
}
#[inline(always)]
pub fn i64_lt_s(a: i64, b: i64) -> i32 {
    (a < b) as i32
}
#[inline(always)]
pub fn i64_lt_u(a: i64, b: i64) -> i32 {
    ((a as u64) < (b as u64)) as i32
}
#[inline(always)]
pub fn i64_le_s(a: i64, b: i64) -> i32 {
    (a <= b) as i32
}
#[inline(always)]
pub fn i64_le_u(a: i64, b: i64) -> i32 {
    ((a as u64) <= (b as u64)) as i32
}
#[inline(always)]
pub fn i64_gt_s(a: i64, b: i64) -> i32 {
    (a > b) as i32
}
#[inline(always)]
pub fn i64_gt_u(a: i64, b: i64) -> i32 {
    ((a as u64) > (b as u64)) as i32
}
#[inline(always)]
pub fn i64_ge_s(a: i64, b: i64) -> i32 {
    (a >= b) as i32
}
#[inline(always)]
pub fn i64_ge_u(a: i64, b: i64) -> i32 {
    ((a as u64) >= (b as u64)) as i32
}

// I64 unary
#[inline(always)]
pub fn i64_clz(a: i64) -> i64 {
    a.leading_zeros() as i64
}
#[inline(always)]
pub fn i64_ctz(a: i64) -> i64 {
    a.trailing_zeros() as i64
}
#[inline(always)]
pub fn i64_popcnt(a: i64) -> i64 {
    a.count_ones() as i64
}
#[inline(always)]
pub fn i64_eqz(a: i64) -> i32 {
    (a == 0) as i32
}
#[inline(always)]
pub fn i64_extend8_s(a: i64) -> i64 {
    (a as i8) as i64
}
#[inline(always)]
pub fn i64_extend16_s(a: i64) -> i64 {
    (a as i16) as i64
}
#[inline(always)]
pub fn i64_extend32_s(a: i64) -> i64 {
    (a as i32) as i64
}

// ============================================================================
// F32 binary
// ============================================================================

#[inline(always)]
pub fn f32_add(a: f32, b: f32) -> f32 {
    a + b
}
#[inline(always)]
pub fn f32_sub(a: f32, b: f32) -> f32 {
    a - b
}
#[inline(always)]
pub fn f32_mul(a: f32, b: f32) -> f32 {
    a * b
}
#[inline(always)]
pub fn f32_div(a: f32, b: f32) -> f32 {
    a / b
}
#[inline(always)]
pub fn f32_copysign(a: f32, b: f32) -> f32 {
    a.copysign(b)
}

/// Wasm-spec-compliant f32.min: NaN propagates, -0.0/+0.0 preserves negative.
#[inline(always)]
pub fn f32_min(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        f32::NAN
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_negative() || b.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else {
        a.min(b)
    }
}

/// Wasm-spec-compliant f32.max.
#[inline(always)]
pub fn f32_max(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        f32::NAN
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_positive() || b.is_sign_positive() {
            0.0
        } else {
            -0.0
        }
    } else {
        a.max(b)
    }
}

// F32 unary
#[inline(always)]
pub fn f32_abs(a: f32) -> f32 {
    a.abs()
}
#[inline(always)]
pub fn f32_neg(a: f32) -> f32 {
    -a
}
#[inline(always)]
pub fn f32_ceil(a: f32) -> f32 {
    a.ceil()
}
#[inline(always)]
pub fn f32_floor(a: f32) -> f32 {
    a.floor()
}
#[inline(always)]
pub fn f32_trunc(a: f32) -> f32 {
    a.trunc()
}
#[inline(always)]
pub fn f32_nearest(a: f32) -> f32 {
    a.round_ties_even()
}
#[inline(always)]
pub fn f32_sqrt(a: f32) -> f32 {
    a.sqrt()
}

// F32 comparisons (return i32 0 or 1)
#[inline(always)]
pub fn f32_eq(a: f32, b: f32) -> i32 {
    (a == b) as i32
}
#[inline(always)]
pub fn f32_ne(a: f32, b: f32) -> i32 {
    (a != b) as i32
}
#[inline(always)]
pub fn f32_lt(a: f32, b: f32) -> i32 {
    (a < b) as i32
}
#[inline(always)]
pub fn f32_gt(a: f32, b: f32) -> i32 {
    (a > b) as i32
}
#[inline(always)]
pub fn f32_le(a: f32, b: f32) -> i32 {
    (a <= b) as i32
}
#[inline(always)]
pub fn f32_ge(a: f32, b: f32) -> i32 {
    (a >= b) as i32
}

// ============================================================================
// F64 binary
// ============================================================================

#[inline(always)]
pub fn f64_add(a: f64, b: f64) -> f64 {
    a + b
}
#[inline(always)]
pub fn f64_sub(a: f64, b: f64) -> f64 {
    a - b
}
#[inline(always)]
pub fn f64_mul(a: f64, b: f64) -> f64 {
    a * b
}
#[inline(always)]
pub fn f64_div(a: f64, b: f64) -> f64 {
    a / b
}
#[inline(always)]
pub fn f64_copysign(a: f64, b: f64) -> f64 {
    a.copysign(b)
}

#[inline(always)]
pub fn f64_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_negative() || b.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else {
        a.min(b)
    }
}

#[inline(always)]
pub fn f64_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_positive() || b.is_sign_positive() {
            0.0
        } else {
            -0.0
        }
    } else {
        a.max(b)
    }
}

// F64 unary
#[inline(always)]
pub fn f64_abs(a: f64) -> f64 {
    a.abs()
}
#[inline(always)]
pub fn f64_neg(a: f64) -> f64 {
    -a
}
#[inline(always)]
pub fn f64_ceil(a: f64) -> f64 {
    a.ceil()
}
#[inline(always)]
pub fn f64_floor(a: f64) -> f64 {
    a.floor()
}
#[inline(always)]
pub fn f64_trunc(a: f64) -> f64 {
    a.trunc()
}
#[inline(always)]
pub fn f64_nearest(a: f64) -> f64 {
    a.round_ties_even()
}
#[inline(always)]
pub fn f64_sqrt(a: f64) -> f64 {
    a.sqrt()
}

// F64 comparisons
#[inline(always)]
pub fn f64_eq(a: f64, b: f64) -> i32 {
    (a == b) as i32
}
#[inline(always)]
pub fn f64_ne(a: f64, b: f64) -> i32 {
    (a != b) as i32
}
#[inline(always)]
pub fn f64_lt(a: f64, b: f64) -> i32 {
    (a < b) as i32
}
#[inline(always)]
pub fn f64_gt(a: f64, b: f64) -> i32 {
    (a > b) as i32
}
#[inline(always)]
pub fn f64_le(a: f64, b: f64) -> i32 {
    (a <= b) as i32
}
#[inline(always)]
pub fn f64_ge(a: f64, b: f64) -> i32 {
    (a >= b) as i32
}
