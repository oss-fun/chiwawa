//! Register file and register allocation for the interpreter.
//!
//! Registers are type-specialized (I32, I64, F32, F64, Ref, V128) and managed
//! per-frame with offset tracking for nested function calls.

use crate::execution::value::{Num, Ref, Val, Vec_};
use crate::structure::types::*;
use serde::{Deserialize, Serialize};

/// Type-specialized register identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Reg {
    I32(u16),
    I64(u16),
    F32(u16),
    F64(u16),
    Ref(u16),
    V128(u16),
}

impl Reg {
    fn new(vtype: &ValueType, index: usize) -> Reg {
        let index = index as u16;
        match vtype {
            ValueType::NumType(NumType::I32) => Reg::I32(index),
            ValueType::NumType(NumType::I64) => Reg::I64(index),
            ValueType::NumType(NumType::F32) => Reg::F32(index),
            ValueType::NumType(NumType::F64) => Reg::F64(index),
            ValueType::RefType(_) => Reg::Ref(index),
            ValueType::VecType(_) => Reg::V128(index),
        }
    }

    /// Get register index
    #[inline(always)]
    pub fn index(&self) -> u16 {
        match self {
            Reg::I32(i) | Reg::I64(i) | Reg::F32(i) | Reg::F64(i) | Reg::Ref(i) | Reg::V128(i) => {
                *i
            }
        }
    }

    /// Get value type information
    pub fn value_type(&self) -> ValueType {
        match self {
            Reg::I32(_) => ValueType::NumType(NumType::I32),
            Reg::I64(_) => ValueType::NumType(NumType::I64),
            Reg::F32(_) => ValueType::NumType(NumType::F32),
            Reg::F64(_) => ValueType::NumType(NumType::F64),
            Reg::Ref(_) => ValueType::RefType(RefType::FuncRef),
            Reg::V128(_) => ValueType::VecType(VecType::V128),
        }
    }
}

/// Frame register offsets for global register file
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct FrameRegOffsets {
    pub i32_offset: u32,
    pub i64_offset: u32,
    pub f32_offset: u32,
    pub f64_offset: u32,
    pub ref_offset: u32,
    pub v128_offset: u32,
}

/// Register file - holds all type-specialized registers (now global across frames)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegFile {
    pub i32_regs: Vec<i32>,
    pub i64_regs: Vec<i64>,
    pub f32_regs: Vec<f32>,
    pub f64_regs: Vec<f64>,
    pub ref_regs: Vec<Ref>,
    pub v128_regs: Vec<i128>,
    /// Frame register offset stack
    frame_offsets: Vec<FrameRegOffsets>,
    /// Cached current frame offsets (updated on save/restore)
    cached_offsets: FrameRegOffsets,
}

impl RegFile {
    pub fn new() -> Self {
        Self {
            i32_regs: Vec::with_capacity(256),
            i64_regs: Vec::with_capacity(64),
            f32_regs: Vec::with_capacity(64),
            f64_regs: Vec::with_capacity(64),
            ref_regs: Vec::with_capacity(32),
            v128_regs: Vec::with_capacity(16),
            frame_offsets: Vec::with_capacity(64),
            cached_offsets: FrameRegOffsets::default(),
        }
    }

    /// Save current offsets and advance to a new frame Only resizes register arrays if they overflow (pre-allocated capacity is preferred)
    pub fn save_offsets(&mut self, allocation: &RegAllocation) {
        let i32_new_end = self.i32_regs.len() + allocation.i32_count;
        let i64_new_end = self.i64_regs.len() + allocation.i64_count;
        let f32_new_end = self.f32_regs.len() + allocation.f32_count;
        let f64_new_end = self.f64_regs.len() + allocation.f64_count;
        let ref_new_end = self.ref_regs.len() + allocation.ref_count;
        let v128_new_end = self.v128_regs.len() + allocation.v128_count;

        let new_offsets = FrameRegOffsets {
            i32_offset: self.i32_regs.len() as u32,
            i64_offset: self.i64_regs.len() as u32,
            f32_offset: self.f32_regs.len() as u32,
            f64_offset: self.f64_regs.len() as u32,
            ref_offset: self.ref_regs.len() as u32,
            v128_offset: self.v128_regs.len() as u32,
        };
        self.frame_offsets.push(new_offsets.clone());
        self.cached_offsets = new_offsets;

        // Resize only if capacity is insufficient
        if i32_new_end > self.i32_regs.len() {
            self.i32_regs.resize(i32_new_end, 0);
        }
        if i64_new_end > self.i64_regs.len() {
            self.i64_regs.resize(i64_new_end, 0);
        }
        if f32_new_end > self.f32_regs.len() {
            self.f32_regs.resize(f32_new_end, 0.0);
        }
        if f64_new_end > self.f64_regs.len() {
            self.f64_regs.resize(f64_new_end, 0.0);
        }
        if ref_new_end > self.ref_regs.len() {
            self.ref_regs.resize(ref_new_end, Ref::RefNull);
        }
        if v128_new_end > self.v128_regs.len() {
            self.v128_regs.resize(v128_new_end, 0);
        }
    }

    /// Restore offsets to previous frame, truncating register vectors to reclaim space.
    ///
    /// The popped frame's offset values were recorded as `Vec::len()` at the time `save_offsets` was called,
    /// so truncating to them restores the vectors to the state before that frame was pushed.
    /// Capacity is preserved for reuse.
    pub fn restore_offsets(&mut self) {
        if let Some(popped) = self.frame_offsets.pop() {
            self.i32_regs.truncate(popped.i32_offset as usize);
            self.i64_regs.truncate(popped.i64_offset as usize);
            self.f32_regs.truncate(popped.f32_offset as usize);
            self.f64_regs.truncate(popped.f64_offset as usize);
            self.ref_regs.truncate(popped.ref_offset as usize);
            self.v128_regs.truncate(popped.v128_offset as usize);
        }
        self.cached_offsets = self.frame_offsets.last().cloned().unwrap_or_default();
    }

    /// Copy the callee's return values into the caller's result registers,
    /// then pop the callee's frame.
    #[inline]
    pub fn pop_frame_with_results(&mut self, src_regs: &[Reg], dst_regs: &[Reg]) {
        let callee = self.cached_offsets;
        let caller = match self.frame_offsets.len().checked_sub(2) {
            Some(i) => self.frame_offsets[i],
            None => FrameRegOffsets::default(),
        };
        self.copy_between_frames(&callee, src_regs, &caller, dst_regs);
        self.restore_offsets();
    }

    /// Open the callee's frame and move the call arguments from the caller's
    /// registers into the callee's local registers.
    #[inline]
    pub fn push_frame_with_params(&mut self, allocation: &RegAllocation, param_regs: &[Reg]) {
        let caller = self.cached_offsets;
        self.save_offsets(allocation);
        let callee = self.cached_offsets;
        self.copy_between_frames(&caller, param_regs, &callee, &allocation.local_regs);
    }

    /// Copy each register of `src_regs` in the frame at `src_base` to its pair in `dst_regs` in the frame at `dst_base`.
    #[inline(always)]
    fn copy_between_frames(
        &mut self,
        src_base: &FrameRegOffsets,
        src_regs: &[Reg],
        dst_base: &FrameRegOffsets,
        dst_regs: &[Reg],
    ) {
        for (src, dst) in src_regs.iter().zip(dst_regs.iter()) {
            // The parser pairs registers of the same type.
            match (src, dst) {
                (Reg::I32(s), Reg::I32(d)) => unsafe {
                    let v = *self
                        .i32_regs
                        .get_unchecked(src_base.i32_offset as usize + *s as usize);
                    *self
                        .i32_regs
                        .get_unchecked_mut(dst_base.i32_offset as usize + *d as usize) = v;
                },
                (Reg::I64(s), Reg::I64(d)) => unsafe {
                    let v = *self
                        .i64_regs
                        .get_unchecked(src_base.i64_offset as usize + *s as usize);
                    *self
                        .i64_regs
                        .get_unchecked_mut(dst_base.i64_offset as usize + *d as usize) = v;
                },
                (Reg::F32(s), Reg::F32(d)) => unsafe {
                    let v = *self
                        .f32_regs
                        .get_unchecked(src_base.f32_offset as usize + *s as usize);
                    *self
                        .f32_regs
                        .get_unchecked_mut(dst_base.f32_offset as usize + *d as usize) = v;
                },
                (Reg::F64(s), Reg::F64(d)) => unsafe {
                    let v = *self
                        .f64_regs
                        .get_unchecked(src_base.f64_offset as usize + *s as usize);
                    *self
                        .f64_regs
                        .get_unchecked_mut(dst_base.f64_offset as usize + *d as usize) = v;
                },
                (Reg::Ref(s), Reg::Ref(d)) => {
                    let v = self.ref_regs[src_base.ref_offset as usize + *s as usize].clone();
                    self.ref_regs[dst_base.ref_offset as usize + *d as usize] = v;
                }
                (Reg::V128(s), Reg::V128(d)) => {
                    let v = self.v128_regs[src_base.v128_offset as usize + *s as usize];
                    self.v128_regs[dst_base.v128_offset as usize + *d as usize] = v;
                }
                _ => {}
            }
        }
    }

    /// Get current frame offsets (returns cached value for performance)
    #[inline(always)]
    fn current_offsets(&self) -> &FrameRegOffsets {
        &self.cached_offsets
    }

    #[inline]
    pub fn frame_bases(&mut self) -> (*mut i32, *mut i64, *mut f32, *mut f64) {
        let o = self.cached_offsets;
        unsafe {
            (
                self.i32_regs.as_mut_ptr().add(o.i32_offset as usize),
                self.i64_regs.as_mut_ptr().add(o.i64_offset as usize),
                self.f32_regs.as_mut_ptr().add(o.f32_offset as usize),
                self.f64_regs.as_mut_ptr().add(o.f64_offset as usize),
            )
        }
    }

    /// Get/set methods for each type (with frame offset)
    #[inline(always)]
    pub fn get_i32(&self, reg: u16) -> i32 {
        let idx = self.current_offsets().i32_offset as usize + reg as usize;
        unsafe { *self.i32_regs.get_unchecked(idx) }
    }

    #[inline(always)]
    pub fn set_i32(&mut self, reg: u16, val: i32) {
        let idx = self.current_offsets().i32_offset as usize + reg as usize;
        unsafe {
            *self.i32_regs.get_unchecked_mut(idx) = val;
        }
    }

    #[inline(always)]
    pub fn get_i64(&self, reg: u16) -> i64 {
        let idx = self.current_offsets().i64_offset as usize + reg as usize;
        unsafe { *self.i64_regs.get_unchecked(idx) }
    }

    #[inline(always)]
    pub fn set_i64(&mut self, reg: u16, val: i64) {
        let idx = self.current_offsets().i64_offset as usize + reg as usize;
        unsafe {
            *self.i64_regs.get_unchecked_mut(idx) = val;
        }
    }

    #[inline(always)]
    pub fn get_f32(&self, reg: u16) -> f32 {
        let idx = self.current_offsets().f32_offset as usize + reg as usize;
        unsafe { *self.f32_regs.get_unchecked(idx) }
    }

    #[inline(always)]
    pub fn set_f32(&mut self, reg: u16, val: f32) {
        let idx = self.current_offsets().f32_offset as usize + reg as usize;
        unsafe {
            *self.f32_regs.get_unchecked_mut(idx) = val;
        }
    }

    #[inline(always)]
    pub fn get_f64(&self, reg: u16) -> f64 {
        let idx = self.current_offsets().f64_offset as usize + reg as usize;
        unsafe { *self.f64_regs.get_unchecked(idx) }
    }

    #[inline(always)]
    pub fn set_f64(&mut self, reg: u16, val: f64) {
        let idx = self.current_offsets().f64_offset as usize + reg as usize;
        unsafe {
            *self.f64_regs.get_unchecked_mut(idx) = val;
        }
    }

    #[inline(always)]
    pub fn get_ref(&self, reg: u16) -> Ref {
        let idx = self.current_offsets().ref_offset as usize + reg as usize;
        unsafe { self.ref_regs.get_unchecked(idx).clone() }
    }

    #[inline(always)]
    pub fn set_ref(&mut self, reg: u16, val: Ref) {
        let idx = self.current_offsets().ref_offset as usize + reg as usize;
        unsafe {
            *self.ref_regs.get_unchecked_mut(idx) = val;
        }
    }

    #[inline(always)]
    pub fn get_v128(&self, reg: u16) -> i128 {
        let idx = self.current_offsets().v128_offset as usize + reg as usize;
        unsafe { *self.v128_regs.get_unchecked(idx) }
    }

    #[inline(always)]
    pub fn set_v128(&mut self, reg: u16, val: i128) {
        let idx = self.current_offsets().v128_offset as usize + reg as usize;
        unsafe {
            *self.v128_regs.get_unchecked_mut(idx) = val;
        }
    }

    /// Copy value from source register to destination register.
    #[inline(always)]
    pub fn copy_reg(&mut self, src: &Reg, dst: &Reg) {
        match (src, dst) {
            (Reg::I32(src_idx), Reg::I32(dst_idx)) => {
                let val = self.get_i32(*src_idx);
                self.set_i32(*dst_idx, val);
            }
            (Reg::I64(src_idx), Reg::I64(dst_idx)) => {
                let val = self.get_i64(*src_idx);
                self.set_i64(*dst_idx, val);
            }
            (Reg::F32(src_idx), Reg::F32(dst_idx)) => {
                let val = self.get_f32(*src_idx);
                self.set_f32(*dst_idx, val);
            }
            (Reg::F64(src_idx), Reg::F64(dst_idx)) => {
                let val = self.get_f64(*src_idx);
                self.set_f64(*dst_idx, val);
            }
            (Reg::Ref(src_idx), Reg::Ref(dst_idx)) => {
                let val = self.get_ref(*src_idx);
                self.set_ref(*dst_idx, val);
            }
            (Reg::V128(src_idx), Reg::V128(dst_idx)) => {
                let val = self.get_v128(*src_idx);
                self.set_v128(*dst_idx, val);
            }
            _ => {}
        }
    }

    /// Get value from register as Val
    #[inline(always)]
    pub fn get_val(&self, reg: &Reg) -> Val {
        match reg {
            Reg::I32(idx) => Val::Num(Num::I32(self.get_i32(*idx))),
            Reg::I64(idx) => Val::Num(Num::I64(self.get_i64(*idx))),
            Reg::F32(idx) => Val::Num(Num::F32(self.get_f32(*idx))),
            Reg::F64(idx) => Val::Num(Num::F64(self.get_f64(*idx))),
            Reg::Ref(idx) => Val::Ref(self.get_ref(*idx)),
            Reg::V128(idx) => Val::Vec_(Vec_::V128(self.get_v128(*idx))),
        }
    }

    /// Set value to register from Val
    #[inline(always)]
    pub fn set_val(&mut self, reg: &Reg, val: &Val) {
        match reg {
            Reg::I32(idx) => self.set_i32(*idx, val.to_i32().unwrap_or(0)),
            Reg::I64(idx) => self.set_i64(*idx, val.to_i64().unwrap_or(0)),
            Reg::F32(idx) => self.set_f32(*idx, val.to_f32().unwrap_or(0.0)),
            Reg::F64(idx) => self.set_f64(*idx, val.to_f64().unwrap_or(0.0)),
            Reg::Ref(idx) => {
                if let Val::Ref(r) = val {
                    self.set_ref(*idx, r.clone());
                }
            }
            Reg::V128(idx) => {
                if let Val::Vec_(Vec_::V128(v)) = val {
                    self.set_v128(*idx, *v);
                }
            }
        }
    }

    /// Write function parameters into their local registers for the current frame.
    /// `local_regs[i]` is the register slot of wasm local `i`;
    /// the first `params.len()` locals are the function parameters.
    #[inline]
    pub fn write_params(&mut self, params: &[Val], local_regs: &[Reg]) {
        for (val, reg) in params.iter().zip(local_regs.iter()) {
            self.set_val(reg, val);
        }
    }
}

/// Register allocation information (number of registers needed per function)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegAllocation {
    pub i32_count: usize,
    pub i64_count: usize,
    pub f32_count: usize,
    pub f64_count: usize,
    pub ref_count: usize,
    pub v128_count: usize,
    /// Register slot for each wasm local index (params first, then declared
    /// locals). Used to scatter call params into local registers at frame entry.
    pub local_regs: Vec<Reg>,
}

/// The register types, in `Reg` variant order: i32, i64, f32, f64, ref, v128.
const REG_TYPES: usize = 6;

fn reg_type(vtype: &ValueType) -> usize {
    match vtype {
        ValueType::NumType(NumType::I32) => 0,
        ValueType::NumType(NumType::I64) => 1,
        ValueType::NumType(NumType::F32) => 2,
        ValueType::NumType(NumType::F64) => 3,
        ValueType::RefType(_) => 4,
        ValueType::VecType(_) => 5,
    }
}

/// Register allocator - tracks stack depth to assign virtual registers
pub struct RegAllocator {
    /// Operand-stack depth of each register type.
    depth: [usize; REG_TYPES],
    /// Deepest the stack has been for each register type; sizes the register file.
    max_depth: [usize; REG_TYPES],

    // Type stack to track push order.
    // Since depths are tracked per-type, we cannot determine which type is on top
    // without this. Required for drop and untyped select instructions.
    type_stack: Vec<ValueType>,

    // Register slot assigned to each wasm local index (params first, then
    // declared locals), recorded while reserving local slots in `new`.
    local_regs: Vec<Reg>,
}

impl RegAllocator {
    /// Create a new allocator local_types: List of function local variable types
    pub fn new(local_types: &[(u32, ValueType)]) -> Self {
        let mut allocator = Self {
            depth: [0; REG_TYPES],
            max_depth: [0; REG_TYPES],
            type_stack: Vec::with_capacity(64),
            local_regs: Vec::with_capacity(local_types.len()),
        };

        // Reserve registers for local variables, recording the slot of each wasm local index so locals can be addressed directly as registers.
        for (count, vtype) in local_types {
            for _ in 0..*count {
                let reg = allocator.alloc(vtype);
                allocator.local_regs.push(reg);
            }
        }
        allocator
    }

    /// Takes the next register specialized for `vtype`.
    fn alloc(&mut self, vtype: &ValueType) -> Reg {
        let ty = reg_type(vtype);
        let reg = Reg::new(vtype, self.depth[ty]);
        self.depth[ty] += 1;
        self.max_depth[ty] = self.max_depth[ty].max(self.depth[ty]);
        reg
    }

    /// Push a value onto the stack (allocate a new register)
    pub fn push(&mut self, vtype: ValueType) -> Reg {
        self.type_stack.push(vtype);
        self.alloc(&vtype)
    }

    /// Pop a value from the stack (decrease depth and return the register)
    pub fn pop(&mut self, vtype: &ValueType) -> Reg {
        self.type_stack.pop();
        let ty = reg_type(vtype);
        self.depth[ty] = self.depth[ty].saturating_sub(1);
        Reg::new(vtype, self.depth[ty])
    }

    /// Register slots for each wasm local index (params first, then locals).
    pub fn local_regs(&self) -> &[Reg] {
        &self.local_regs
    }

    /// Pop the top value from the stack (using type_stack to determine the type)
    pub fn pop_any(&mut self) -> Option<Reg> {
        let vtype = *self.type_stack.last()?;
        Some(self.pop(&vtype))
    }

    /// Peek the type at the top of the stack
    pub fn peek_type(&self) -> Option<&ValueType> {
        self.type_stack.last()
    }

    /// Peek at the current stack top (without popping)
    pub fn peek(&self, vtype: &ValueType) -> Option<Reg> {
        let depth = self.depth[reg_type(vtype)];
        (depth > 0).then(|| Reg::new(vtype, depth - 1))
    }

    /// The registers holding the top values of `types`, in the same order:
    /// each type's values are its last `count` registers, in push order.
    pub fn peek_regs_for_types(&self, types: &[ValueType]) -> Vec<Reg> {
        let mut count = [0usize; REG_TYPES];
        for vtype in types {
            count[reg_type(vtype)] += 1;
        }
        let mut next: [usize; REG_TYPES] =
            std::array::from_fn(|b| self.depth[b].saturating_sub(count[b]));
        types
            .iter()
            .map(|vtype| {
                let ty = reg_type(vtype);
                let reg = Reg::new(vtype, next[ty]);
                next[ty] += 1;
                reg
            })
            .collect()
    }

    /// Save current stack state for block entry
    pub fn save_state(&self) -> RegAllocatorState {
        RegAllocatorState {
            depth: self.depth,
            type_stack_len: self.type_stack.len(),
        }
    }

    /// Restore stack state for block exit (keeps max depths intact)
    pub fn restore_state(&mut self, state: &RegAllocatorState) {
        self.depth = state.depth;
        self.type_stack.truncate(state.type_stack_len);
    }

    /// Finalize and return allocation information
    pub fn finalize(self) -> RegAllocation {
        RegAllocation {
            i32_count: self.max_depth[0],
            i64_count: self.max_depth[1],
            f32_count: self.max_depth[2],
            f64_count: self.max_depth[3],
            ref_count: self.max_depth[4],
            v128_count: self.max_depth[5],
            local_regs: self.local_regs,
        }
    }
}

/// Saved state of RegAllocator at block entry
#[derive(Clone, Debug)]
pub struct RegAllocatorState {
    depth: [usize; REG_TYPES],
    type_stack_len: usize,
}

impl RegAllocatorState {
    /// Increment depth for a type and return the register at that position
    pub fn next_reg_for_type(&mut self, vtype: &ValueType) -> Reg {
        let ty = reg_type(vtype);
        let reg = Reg::new(vtype, self.depth[ty]);
        self.depth[ty] += 1;
        reg
    }
}
