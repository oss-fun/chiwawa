use crate::execution::value::{Num, Ref, Val, Vec_};
use crate::structure::types::*;
use serde::{Deserialize, Serialize};

/// Type-specialized register identifier
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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
}

impl RegFile {
    /// Create a new empty global register file
    pub fn new_global() -> Self {
        Self {
            i32_regs: Vec::with_capacity(256),
            i64_regs: Vec::with_capacity(64),
            f32_regs: Vec::with_capacity(64),
            f64_regs: Vec::with_capacity(64),
            ref_regs: Vec::with_capacity(32),
            v128_regs: Vec::with_capacity(16),
            frame_offsets: Vec::with_capacity(64),
        }
    }

    /// Create a new register file with specified capacities (legacy, for single frame)
    pub fn new(
        i32_count: usize,
        i64_count: usize,
        f32_count: usize,
        f64_count: usize,
        ref_count: usize,
        v128_count: usize,
    ) -> Self {
        let mut sf = Self {
            i32_regs: vec![0; i32_count],
            i64_regs: vec![0; i64_count],
            f32_regs: vec![0.0; f32_count],
            f64_regs: vec![0.0; f64_count],
            ref_regs: vec![Ref::RefNull; ref_count],
            v128_regs: vec![0; v128_count],
            frame_offsets: Vec::new(),
        };
        // Push initial frame offset at 0
        sf.frame_offsets.push(FrameRegOffsets::default());
        sf
    }

    /// Create a new register file from RegAllocation
    pub fn from_allocation(allocation: &RegAllocation) -> Self {
        Self::new(
            allocation.i32_count,
            allocation.i64_count,
            allocation.f32_count,
            allocation.f64_count,
            allocation.ref_count,
            allocation.v128_count,
        )
    }

    /// Save current offsets and advance to a new frame
    /// Only resizes register arrays if they overflow (pre-allocated capacity is preferred)
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
        self.frame_offsets.push(new_offsets);

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

    /// Restore offsets to previous frame (does not deallocate registers)
    pub fn restore_offsets(&mut self) {
        self.frame_offsets.pop();
    }

    /// Get current frame offsets
    #[inline(always)]
    fn current_offsets(&self) -> &FrameRegOffsets {
        self.frame_offsets.last().unwrap_or(&FrameRegOffsets {
            i32_offset: 0,
            i64_offset: 0,
            f32_offset: 0,
            f64_offset: 0,
            ref_offset: 0,
            v128_offset: 0,
        })
    }

    /// Get frame depth
    pub fn frame_depth(&self) -> usize {
        self.frame_offsets.len()
    }

    /// Get/set methods for each type (with frame offset)
    #[inline(always)]
    pub fn get_i32(&self, reg: u16) -> i32 {
        let offset = self.current_offsets().i32_offset as usize;
        self.i32_regs[offset + reg as usize]
    }

    #[inline(always)]
    pub fn set_i32(&mut self, reg: u16, val: i32) {
        let offset = self.current_offsets().i32_offset as usize;
        self.i32_regs[offset + reg as usize] = val;
    }

    #[inline(always)]
    pub fn get_i64(&self, reg: u16) -> i64 {
        let offset = self.current_offsets().i64_offset as usize;
        self.i64_regs[offset + reg as usize]
    }

    #[inline(always)]
    pub fn set_i64(&mut self, reg: u16, val: i64) {
        let offset = self.current_offsets().i64_offset as usize;
        self.i64_regs[offset + reg as usize] = val;
    }

    #[inline(always)]
    pub fn get_f32(&self, reg: u16) -> f32 {
        let offset = self.current_offsets().f32_offset as usize;
        self.f32_regs[offset + reg as usize]
    }

    #[inline(always)]
    pub fn set_f32(&mut self, reg: u16, val: f32) {
        let offset = self.current_offsets().f32_offset as usize;
        self.f32_regs[offset + reg as usize] = val;
    }

    #[inline(always)]
    pub fn get_f64(&self, reg: u16) -> f64 {
        let offset = self.current_offsets().f64_offset as usize;
        self.f64_regs[offset + reg as usize]
    }

    #[inline(always)]
    pub fn set_f64(&mut self, reg: u16, val: f64) {
        let offset = self.current_offsets().f64_offset as usize;
        self.f64_regs[offset + reg as usize] = val;
    }

    #[inline(always)]
    pub fn get_ref(&self, reg: u16) -> Ref {
        let offset = self.current_offsets().ref_offset as usize;
        self.ref_regs[offset + reg as usize].clone()
    }

    #[inline(always)]
    pub fn set_ref(&mut self, reg: u16, val: Ref) {
        let offset = self.current_offsets().ref_offset as usize;
        self.ref_regs[offset + reg as usize] = val;
    }

    #[inline(always)]
    pub fn get_v128(&self, reg: u16) -> i128 {
        let offset = self.current_offsets().v128_offset as usize;
        self.v128_regs[offset + reg as usize]
    }

    #[inline(always)]
    pub fn set_v128(&mut self, reg: u16, val: i128) {
        let offset = self.current_offsets().v128_offset as usize;
        self.v128_regs[offset + reg as usize] = val;
    }

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

    #[inline]
    pub fn copy_regs(&mut self, src_regs: &[Reg], dst_regs: &[Reg]) {
        for (src, dst) in src_regs.iter().zip(dst_regs.iter()) {
            self.copy_reg(src, dst);
        }
    }

    /// Get i32 registers slice for current frame
    #[inline(always)]
    pub fn get_i32_regs(&mut self) -> &mut [i32] {
        let offset = self.current_offsets().i32_offset as usize;
        &mut self.i32_regs[offset..]
    }

    /// Get i64 registers slice for current frame
    #[inline(always)]
    pub fn get_i64_regs(&mut self) -> &mut [i64] {
        let offset = self.current_offsets().i64_offset as usize;
        &mut self.i64_regs[offset..]
    }

    /// Get both i32 and i64 registers for current frame (for i64 comparison operations)
    #[inline(always)]
    pub fn get_i32_and_i64_regs(&mut self) -> (&mut [i32], &mut [i64]) {
        let i32_offset = self.current_offsets().i32_offset as usize;
        let i64_offset = self.current_offsets().i64_offset as usize;
        let i32_ptr = &mut self.i32_regs[i32_offset..] as *mut [i32];
        let i64_ptr = &mut self.i64_regs[i64_offset..] as *mut [i64];
        unsafe { (&mut *i32_ptr, &mut *i64_ptr) }
    }

    #[inline(always)]
    pub fn get_i32_and_f32_regs(&mut self) -> (&mut [i32], &mut [f32]) {
        let i32_offset = self.current_offsets().i32_offset as usize;
        let f32_offset = self.current_offsets().f32_offset as usize;
        let i32_ptr = &mut self.i32_regs[i32_offset..] as *mut [i32];
        let f32_ptr = &mut self.f32_regs[f32_offset..] as *mut [f32];
        unsafe { (&mut *i32_ptr, &mut *f32_ptr) }
    }

    #[inline(always)]
    pub fn get_i32_and_f64_regs(&mut self) -> (&mut [i32], &mut [f64]) {
        let i32_offset = self.current_offsets().i32_offset as usize;
        let f64_offset = self.current_offsets().f64_offset as usize;
        let i32_ptr = &mut self.i32_regs[i32_offset..] as *mut [i32];
        let f64_ptr = &mut self.f64_regs[f64_offset..] as *mut [f64];
        unsafe { (&mut *i32_ptr, &mut *f64_ptr) }
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
            Reg::V128(_) => {}
        }
    }

    /// Write values from value_stack to registers (stack_to_regs operation)
    /// Returns the number of values consumed from value_stack
    #[inline]
    pub fn write_from_stack(&mut self, stack_to_regs: &[Reg], value_stack: &[Val]) -> usize {
        let reg_count = stack_to_regs.len();
        let stack_len = value_stack.len();
        if reg_count == 0 || stack_len < reg_count {
            return 0;
        }
        for (i, reg) in stack_to_regs.iter().enumerate() {
            self.set_val(reg, &value_stack[stack_len - reg_count + i]);
        }
        reg_count
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
}

/// Register allocator - tracks stack depth to assign virtual registers
pub struct RegAllocator {
    // Current stack depth per type
    i32_depth: usize,
    i64_depth: usize,
    f32_depth: usize,
    f64_depth: usize,
    ref_depth: usize,
    v128_depth: usize,

    // Maximum depth reached (used to determine register file size)
    max_i32_depth: usize,
    max_i64_depth: usize,
    max_f32_depth: usize,
    max_f64_depth: usize,
    max_ref_depth: usize,
    max_v128_depth: usize,

    // Type stack to track push order.
    // Since depths are tracked per-type, we cannot determine which type is on top
    // without this. Required for drop and untyped select instructions.
    type_stack: Vec<ValueType>,
}

impl RegAllocator {
    /// Create a new allocator
    /// local_types: List of function local variable types
    pub fn new(local_types: &[(u32, ValueType)]) -> Self {
        let mut allocator = Self {
            i32_depth: 0,
            i64_depth: 0,
            f32_depth: 0,
            f64_depth: 0,
            ref_depth: 0,
            v128_depth: 0,
            max_i32_depth: 0,
            max_i64_depth: 0,
            max_f32_depth: 0,
            max_f64_depth: 0,
            max_ref_depth: 0,
            max_v128_depth: 0,
            type_stack: Vec::new(),
        };

        // Reserve registers for local variables
        for (count, vtype) in local_types {
            for _ in 0..*count {
                match vtype {
                    ValueType::NumType(NumType::I32) => allocator.i32_depth += 1,
                    ValueType::NumType(NumType::I64) => allocator.i64_depth += 1,
                    ValueType::NumType(NumType::F32) => allocator.f32_depth += 1,
                    ValueType::NumType(NumType::F64) => allocator.f64_depth += 1,
                    ValueType::RefType(_) => allocator.ref_depth += 1,
                    ValueType::VecType(_) => allocator.v128_depth += 1,
                }
            }
        }

        // Initialize max depths
        allocator.max_i32_depth = allocator.i32_depth;
        allocator.max_i64_depth = allocator.i64_depth;
        allocator.max_f32_depth = allocator.f32_depth;
        allocator.max_f64_depth = allocator.f64_depth;
        allocator.max_ref_depth = allocator.ref_depth;
        allocator.max_v128_depth = allocator.v128_depth;

        allocator
    }

    /// Push a value onto the stack (allocate a new register)
    pub fn push(&mut self, vtype: ValueType) -> Reg {
        self.type_stack.push(vtype.clone());
        match vtype {
            ValueType::NumType(NumType::I32) => {
                let reg = Reg::I32(self.i32_depth as u16);
                self.i32_depth += 1;
                self.max_i32_depth = self.max_i32_depth.max(self.i32_depth);
                reg
            }
            ValueType::NumType(NumType::I64) => {
                let reg = Reg::I64(self.i64_depth as u16);
                self.i64_depth += 1;
                self.max_i64_depth = self.max_i64_depth.max(self.i64_depth);
                reg
            }
            ValueType::NumType(NumType::F32) => {
                let reg = Reg::F32(self.f32_depth as u16);
                self.f32_depth += 1;
                self.max_f32_depth = self.max_f32_depth.max(self.f32_depth);
                reg
            }
            ValueType::NumType(NumType::F64) => {
                let reg = Reg::F64(self.f64_depth as u16);
                self.f64_depth += 1;
                self.max_f64_depth = self.max_f64_depth.max(self.f64_depth);
                reg
            }
            ValueType::RefType(_) => {
                let reg = Reg::Ref(self.ref_depth as u16);
                self.ref_depth += 1;
                self.max_ref_depth = self.max_ref_depth.max(self.ref_depth);
                reg
            }
            ValueType::VecType(_) => {
                let reg = Reg::V128(self.v128_depth as u16);
                self.v128_depth += 1;
                self.max_v128_depth = self.max_v128_depth.max(self.v128_depth);
                reg
            }
        }
    }

    /// Reserve a register without tracking on type stack
    pub fn reserve(&mut self, vtype: ValueType) -> Reg {
        match vtype {
            ValueType::NumType(NumType::I32) => {
                let reg = Reg::I32(self.i32_depth as u16);
                self.i32_depth += 1;
                self.max_i32_depth = self.max_i32_depth.max(self.i32_depth);
                reg
            }
            ValueType::NumType(NumType::I64) => {
                let reg = Reg::I64(self.i64_depth as u16);
                self.i64_depth += 1;
                self.max_i64_depth = self.max_i64_depth.max(self.i64_depth);
                reg
            }
            ValueType::NumType(NumType::F32) => {
                let reg = Reg::F32(self.f32_depth as u16);
                self.f32_depth += 1;
                self.max_f32_depth = self.max_f32_depth.max(self.f32_depth);
                reg
            }
            ValueType::NumType(NumType::F64) => {
                let reg = Reg::F64(self.f64_depth as u16);
                self.f64_depth += 1;
                self.max_f64_depth = self.max_f64_depth.max(self.f64_depth);
                reg
            }
            ValueType::RefType(_) => {
                let reg = Reg::Ref(self.ref_depth as u16);
                self.ref_depth += 1;
                self.max_ref_depth = self.max_ref_depth.max(self.ref_depth);
                reg
            }
            ValueType::VecType(_) => {
                let reg = Reg::V128(self.v128_depth as u16);
                self.v128_depth += 1;
                self.max_v128_depth = self.max_v128_depth.max(self.v128_depth);
                reg
            }
        }
    }

    pub fn free(&mut self, vtype: &ValueType) {
        match vtype {
            ValueType::NumType(NumType::I32) => {
                self.i32_depth = self.i32_depth.saturating_sub(1);
            }
            ValueType::NumType(NumType::I64) => {
                self.i64_depth = self.i64_depth.saturating_sub(1);
            }
            ValueType::NumType(NumType::F32) => {
                self.f32_depth = self.f32_depth.saturating_sub(1);
            }
            ValueType::NumType(NumType::F64) => {
                self.f64_depth = self.f64_depth.saturating_sub(1);
            }
            ValueType::RefType(_) => {
                self.ref_depth = self.ref_depth.saturating_sub(1);
            }
            ValueType::VecType(_) => {
                self.v128_depth = self.v128_depth.saturating_sub(1);
            }
        }
    }

    /// Pop a value from the stack (decrease depth and return the register)
    pub fn pop(&mut self, vtype: &ValueType) -> Reg {
        self.type_stack.pop();
        match vtype {
            ValueType::NumType(NumType::I32) => {
                self.i32_depth = self.i32_depth.saturating_sub(1);
                Reg::I32(self.i32_depth as u16)
            }
            ValueType::NumType(NumType::I64) => {
                self.i64_depth = self.i64_depth.saturating_sub(1);
                Reg::I64(self.i64_depth as u16)
            }
            ValueType::NumType(NumType::F32) => {
                self.f32_depth = self.f32_depth.saturating_sub(1);
                Reg::F32(self.f32_depth as u16)
            }
            ValueType::NumType(NumType::F64) => {
                self.f64_depth = self.f64_depth.saturating_sub(1);
                Reg::F64(self.f64_depth as u16)
            }
            ValueType::RefType(_) => {
                self.ref_depth = self.ref_depth.saturating_sub(1);
                Reg::Ref(self.ref_depth as u16)
            }
            ValueType::VecType(_) => {
                self.v128_depth = self.v128_depth.saturating_sub(1);
                Reg::V128(self.v128_depth as u16)
            }
        }
    }

    /// Pop the top value from the stack (using type_stack to determine the type)
    pub fn pop_any(&mut self) -> Option<Reg> {
        let vtype = self.type_stack.last()?.clone();
        Some(self.pop(&vtype))
    }

    /// Peek the type at the top of the stack
    pub fn peek_type(&self) -> Option<&ValueType> {
        self.type_stack.last()
    }

    /// Peek at the current stack top (without popping)
    pub fn peek(&self, vtype: &ValueType) -> Option<Reg> {
        match vtype {
            ValueType::NumType(NumType::I32) if self.i32_depth > 0 => {
                Some(Reg::I32((self.i32_depth - 1) as u16))
            }
            ValueType::NumType(NumType::I64) if self.i64_depth > 0 => {
                Some(Reg::I64((self.i64_depth - 1) as u16))
            }
            ValueType::NumType(NumType::F32) if self.f32_depth > 0 => {
                Some(Reg::F32((self.f32_depth - 1) as u16))
            }
            ValueType::NumType(NumType::F64) if self.f64_depth > 0 => {
                Some(Reg::F64((self.f64_depth - 1) as u16))
            }
            ValueType::RefType(_) if self.ref_depth > 0 => {
                Some(Reg::Ref((self.ref_depth - 1) as u16))
            }
            ValueType::VecType(_) if self.v128_depth > 0 => {
                Some(Reg::V128((self.v128_depth - 1) as u16))
            }
            _ => None,
        }
    }

    /// Peek registers for given types from the top of the stack
    /// Returns registers in the same order as the input types
    pub fn peek_regs_for_types(&self, types: &[ValueType]) -> Vec<Reg> {
        if types.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(types.len());

        // Count how many of each type in the input, then calculate starting indices
        let (mut i32_idx, mut i64_idx, mut f32_idx, mut f64_idx, mut ref_idx, mut v128_idx) = {
            let (mut i32_c, mut i64_c, mut f32_c, mut f64_c, mut ref_c, mut v128_c) =
                (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
            for vtype in types.iter() {
                match vtype {
                    ValueType::NumType(NumType::I32) => i32_c += 1,
                    ValueType::NumType(NumType::I64) => i64_c += 1,
                    ValueType::NumType(NumType::F32) => f32_c += 1,
                    ValueType::NumType(NumType::F64) => f64_c += 1,
                    ValueType::RefType(_) => ref_c += 1,
                    ValueType::VecType(_) => v128_c += 1,
                }
            }
            (
                self.i32_depth.saturating_sub(i32_c),
                self.i64_depth.saturating_sub(i64_c),
                self.f32_depth.saturating_sub(f32_c),
                self.f64_depth.saturating_sub(f64_c),
                self.ref_depth.saturating_sub(ref_c),
                self.v128_depth.saturating_sub(v128_c),
            )
        };

        // Iterate and assign registers
        for vtype in types.iter() {
            let reg = match vtype {
                ValueType::NumType(NumType::I32) => {
                    let s = Reg::I32(i32_idx as u16);
                    i32_idx += 1;
                    s
                }
                ValueType::NumType(NumType::I64) => {
                    let s = Reg::I64(i64_idx as u16);
                    i64_idx += 1;
                    s
                }
                ValueType::NumType(NumType::F32) => {
                    let s = Reg::F32(f32_idx as u16);
                    f32_idx += 1;
                    s
                }
                ValueType::NumType(NumType::F64) => {
                    let s = Reg::F64(f64_idx as u16);
                    f64_idx += 1;
                    s
                }
                ValueType::RefType(_) => {
                    let s = Reg::Ref(ref_idx as u16);
                    ref_idx += 1;
                    s
                }
                ValueType::VecType(_) => {
                    let s = Reg::V128(v128_idx as u16);
                    v128_idx += 1;
                    s
                }
            };
            result.push(reg);
        }

        result
    }

    /// Clear the current stack depths
    pub fn clear_stack(&mut self) {
        self.i32_depth = 0;
        self.i64_depth = 0;
        self.f32_depth = 0;
        self.f64_depth = 0;
        self.ref_depth = 0;
        self.v128_depth = 0;
    }

    /// Save current stack state for block entry
    pub fn save_state(&self) -> RegAllocatorState {
        RegAllocatorState {
            i32_depth: self.i32_depth,
            i64_depth: self.i64_depth,
            f32_depth: self.f32_depth,
            f64_depth: self.f64_depth,
            ref_depth: self.ref_depth,
            v128_depth: self.v128_depth,
            type_stack_len: self.type_stack.len(),
        }
    }

    /// Restore stack state for block exit (keeps max depths intact)
    pub fn restore_state(&mut self, state: &RegAllocatorState) {
        self.i32_depth = state.i32_depth;
        self.i64_depth = state.i64_depth;
        self.f32_depth = state.f32_depth;
        self.f64_depth = state.f64_depth;
        self.ref_depth = state.ref_depth;
        self.v128_depth = state.v128_depth;
        self.type_stack.truncate(state.type_stack_len);
    }

    /// Finalize and return allocation information
    pub fn finalize(self) -> RegAllocation {
        RegAllocation {
            i32_count: self.max_i32_depth,
            i64_count: self.max_i64_depth,
            f32_count: self.max_f32_depth,
            f64_count: self.max_f64_depth,
            ref_count: self.max_ref_depth,
            v128_count: self.max_v128_depth,
        }
    }
}

/// Saved state of RegAllocator at block entry
#[derive(Clone, Debug)]
pub struct RegAllocatorState {
    pub i32_depth: usize,
    pub i64_depth: usize,
    pub f32_depth: usize,
    pub f64_depth: usize,
    pub ref_depth: usize,
    pub v128_depth: usize,
    pub type_stack_len: usize,
}

impl RegAllocatorState {
    /// Get register at entry_depth for given type
    /// Used to determine result_regs at block entry
    pub fn reg_for_type(&self, vtype: &ValueType) -> Reg {
        match vtype {
            ValueType::NumType(NumType::I32) => Reg::I32(self.i32_depth as u16),
            ValueType::NumType(NumType::I64) => Reg::I64(self.i64_depth as u16),
            ValueType::NumType(NumType::F32) => Reg::F32(self.f32_depth as u16),
            ValueType::NumType(NumType::F64) => Reg::F64(self.f64_depth as u16),
            ValueType::RefType(_) => Reg::Ref(self.ref_depth as u16),
            ValueType::VecType(_) => Reg::V128(self.v128_depth as u16),
        }
    }

    /// Increment depth for a type and return the register at that position
    pub fn next_reg_for_type(&mut self, vtype: &ValueType) -> Reg {
        match vtype {
            ValueType::NumType(NumType::I32) => {
                let reg = Reg::I32(self.i32_depth as u16);
                self.i32_depth += 1;
                reg
            }
            ValueType::NumType(NumType::I64) => {
                let reg = Reg::I64(self.i64_depth as u16);
                self.i64_depth += 1;
                reg
            }
            ValueType::NumType(NumType::F32) => {
                let reg = Reg::F32(self.f32_depth as u16);
                self.f32_depth += 1;
                reg
            }
            ValueType::NumType(NumType::F64) => {
                let reg = Reg::F64(self.f64_depth as u16);
                self.f64_depth += 1;
                reg
            }
            ValueType::RefType(_) => {
                let reg = Reg::Ref(self.ref_depth as u16);
                self.ref_depth += 1;
                reg
            }
            ValueType::VecType(_) => {
                let reg = Reg::V128(self.v128_depth as u16);
                self.v128_depth += 1;
                reg
            }
        }
    }
}
