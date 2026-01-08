use crate::execution::value::{Num, Ref, Val, Vec_};
use crate::structure::types::*;
use serde::{Deserialize, Serialize};

/// Type-specialized slot identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Slot {
    I32(u16),
    I64(u16),
    F32(u16),
    F64(u16),
    Ref(u16),
    V128(u16),
}

impl Slot {
    /// Get slot index
    #[inline(always)]
    pub fn index(&self) -> u16 {
        match self {
            Slot::I32(i)
            | Slot::I64(i)
            | Slot::F32(i)
            | Slot::F64(i)
            | Slot::Ref(i)
            | Slot::V128(i) => *i,
        }
    }

    /// Get value type information
    pub fn value_type(&self) -> ValueType {
        match self {
            Slot::I32(_) => ValueType::NumType(NumType::I32),
            Slot::I64(_) => ValueType::NumType(NumType::I64),
            Slot::F32(_) => ValueType::NumType(NumType::F32),
            Slot::F64(_) => ValueType::NumType(NumType::F64),
            Slot::Ref(_) => ValueType::RefType(RefType::FuncRef),
            Slot::V128(_) => ValueType::VecType(VecType::V128),
        }
    }
}

/// Frame slot offsets for global slot file
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FrameSlotOffsets {
    pub i32_offset: u32,
    pub i64_offset: u32,
    pub f32_offset: u32,
    pub f64_offset: u32,
    pub ref_offset: u32,
    pub v128_offset: u32,
}

/// Slot file - holds all type-specialized slots (now global across frames)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlotFile {
    pub i32_slots: Vec<i32>,
    pub i64_slots: Vec<i64>,
    pub f32_slots: Vec<f32>,
    pub f64_slots: Vec<f64>,
    pub ref_slots: Vec<Ref>,
    pub v128_slots: Vec<i128>,
    /// Frame slot offset stack
    frame_offsets: Vec<FrameSlotOffsets>,
}

impl SlotFile {
    /// Create a new empty global slot file
    pub fn new_global() -> Self {
        Self {
            i32_slots: Vec::with_capacity(256),
            i64_slots: Vec::with_capacity(64),
            f32_slots: Vec::with_capacity(64),
            f64_slots: Vec::with_capacity(64),
            ref_slots: Vec::with_capacity(32),
            v128_slots: Vec::with_capacity(16),
            frame_offsets: Vec::with_capacity(64),
        }
    }

    /// Create a new slot file with specified capacities (legacy, for single frame)
    pub fn new(
        i32_count: usize,
        i64_count: usize,
        f32_count: usize,
        f64_count: usize,
        ref_count: usize,
        v128_count: usize,
    ) -> Self {
        let mut sf = Self {
            i32_slots: vec![0; i32_count],
            i64_slots: vec![0; i64_count],
            f32_slots: vec![0.0; f32_count],
            f64_slots: vec![0.0; f64_count],
            ref_slots: vec![Ref::RefNull; ref_count],
            v128_slots: vec![0; v128_count],
            frame_offsets: Vec::new(),
        };
        // Push initial frame offset at 0
        sf.frame_offsets.push(FrameSlotOffsets::default());
        sf
    }

    /// Create a new slot file from SlotAllocation
    pub fn from_allocation(allocation: &SlotAllocation) -> Self {
        Self::new(
            allocation.i32_count,
            allocation.i64_count,
            allocation.f32_count,
            allocation.f64_count,
            allocation.ref_count,
            allocation.v128_count,
        )
    }

    /// Push a new frame with the given allocation
    pub fn push_frame(&mut self, allocation: &SlotAllocation) {
        let new_offsets = FrameSlotOffsets {
            i32_offset: self.i32_slots.len() as u32,
            i64_offset: self.i64_slots.len() as u32,
            f32_offset: self.f32_slots.len() as u32,
            f64_offset: self.f64_slots.len() as u32,
            ref_offset: self.ref_slots.len() as u32,
            v128_offset: self.v128_slots.len() as u32,
        };
        self.frame_offsets.push(new_offsets);

        // Allocate slots for this frame
        self.i32_slots
            .resize(self.i32_slots.len() + allocation.i32_count, 0);
        self.i64_slots
            .resize(self.i64_slots.len() + allocation.i64_count, 0);
        self.f32_slots
            .resize(self.f32_slots.len() + allocation.f32_count, 0.0);
        self.f64_slots
            .resize(self.f64_slots.len() + allocation.f64_count, 0.0);
        self.ref_slots
            .resize(self.ref_slots.len() + allocation.ref_count, Ref::RefNull);
        self.v128_slots
            .resize(self.v128_slots.len() + allocation.v128_count, 0);
    }

    /// Pop the current frame
    pub fn pop_frame(&mut self) {
        if let Some(offsets) = self.frame_offsets.pop() {
            self.i32_slots.truncate(offsets.i32_offset as usize);
            self.i64_slots.truncate(offsets.i64_offset as usize);
            self.f32_slots.truncate(offsets.f32_offset as usize);
            self.f64_slots.truncate(offsets.f64_offset as usize);
            self.ref_slots.truncate(offsets.ref_offset as usize);
            self.v128_slots.truncate(offsets.v128_offset as usize);
        }
    }

    /// Get current frame offsets
    #[inline(always)]
    fn current_offsets(&self) -> &FrameSlotOffsets {
        self.frame_offsets.last().unwrap_or(&FrameSlotOffsets {
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
    pub fn get_i32(&self, slot: u16) -> i32 {
        let offset = self.current_offsets().i32_offset as usize;
        self.i32_slots[offset + slot as usize]
    }

    #[inline(always)]
    pub fn set_i32(&mut self, slot: u16, val: i32) {
        let offset = self.current_offsets().i32_offset as usize;
        self.i32_slots[offset + slot as usize] = val;
    }

    #[inline(always)]
    pub fn get_i64(&self, slot: u16) -> i64 {
        let offset = self.current_offsets().i64_offset as usize;
        self.i64_slots[offset + slot as usize]
    }

    #[inline(always)]
    pub fn set_i64(&mut self, slot: u16, val: i64) {
        let offset = self.current_offsets().i64_offset as usize;
        self.i64_slots[offset + slot as usize] = val;
    }

    #[inline(always)]
    pub fn get_f32(&self, slot: u16) -> f32 {
        let offset = self.current_offsets().f32_offset as usize;
        self.f32_slots[offset + slot as usize]
    }

    #[inline(always)]
    pub fn set_f32(&mut self, slot: u16, val: f32) {
        let offset = self.current_offsets().f32_offset as usize;
        self.f32_slots[offset + slot as usize] = val;
    }

    #[inline(always)]
    pub fn get_f64(&self, slot: u16) -> f64 {
        let offset = self.current_offsets().f64_offset as usize;
        self.f64_slots[offset + slot as usize]
    }

    #[inline(always)]
    pub fn set_f64(&mut self, slot: u16, val: f64) {
        let offset = self.current_offsets().f64_offset as usize;
        self.f64_slots[offset + slot as usize] = val;
    }

    #[inline(always)]
    pub fn get_ref(&self, slot: u16) -> Ref {
        let offset = self.current_offsets().ref_offset as usize;
        self.ref_slots[offset + slot as usize].clone()
    }

    #[inline(always)]
    pub fn set_ref(&mut self, slot: u16, val: Ref) {
        let offset = self.current_offsets().ref_offset as usize;
        self.ref_slots[offset + slot as usize] = val;
    }

    #[inline(always)]
    pub fn get_v128(&self, slot: u16) -> i128 {
        let offset = self.current_offsets().v128_offset as usize;
        self.v128_slots[offset + slot as usize]
    }

    #[inline(always)]
    pub fn set_v128(&mut self, slot: u16, val: i128) {
        let offset = self.current_offsets().v128_offset as usize;
        self.v128_slots[offset + slot as usize] = val;
    }

    /// Get i32 slots slice for current frame
    #[inline(always)]
    pub fn get_i32_slots(&mut self) -> &mut [i32] {
        let offset = self.current_offsets().i32_offset as usize;
        &mut self.i32_slots[offset..]
    }

    /// Get i64 slots slice for current frame
    #[inline(always)]
    pub fn get_i64_slots(&mut self) -> &mut [i64] {
        let offset = self.current_offsets().i64_offset as usize;
        &mut self.i64_slots[offset..]
    }

    /// Get both i32 and i64 slots for current frame (for i64 comparison operations)
    #[inline(always)]
    pub fn get_i32_and_i64_slots(&mut self) -> (&mut [i32], &mut [i64]) {
        let i32_offset = self.current_offsets().i32_offset as usize;
        let i64_offset = self.current_offsets().i64_offset as usize;
        let i32_ptr = &mut self.i32_slots[i32_offset..] as *mut [i32];
        let i64_ptr = &mut self.i64_slots[i64_offset..] as *mut [i64];
        unsafe { (&mut *i32_ptr, &mut *i64_ptr) }
    }

    #[inline(always)]
    pub fn get_i32_and_f32_slots(&mut self) -> (&mut [i32], &mut [f32]) {
        let i32_offset = self.current_offsets().i32_offset as usize;
        let f32_offset = self.current_offsets().f32_offset as usize;
        let i32_ptr = &mut self.i32_slots[i32_offset..] as *mut [i32];
        let f32_ptr = &mut self.f32_slots[f32_offset..] as *mut [f32];
        unsafe { (&mut *i32_ptr, &mut *f32_ptr) }
    }

    #[inline(always)]
    pub fn get_i32_and_f64_slots(&mut self) -> (&mut [i32], &mut [f64]) {
        let i32_offset = self.current_offsets().i32_offset as usize;
        let f64_offset = self.current_offsets().f64_offset as usize;
        let i32_ptr = &mut self.i32_slots[i32_offset..] as *mut [i32];
        let f64_ptr = &mut self.f64_slots[f64_offset..] as *mut [f64];
        unsafe { (&mut *i32_ptr, &mut *f64_ptr) }
    }

    /// Get value from slot as Val
    #[inline(always)]
    pub fn get_val(&self, slot: &Slot) -> Val {
        match slot {
            Slot::I32(idx) => Val::Num(Num::I32(self.get_i32(*idx))),
            Slot::I64(idx) => Val::Num(Num::I64(self.get_i64(*idx))),
            Slot::F32(idx) => Val::Num(Num::F32(self.get_f32(*idx))),
            Slot::F64(idx) => Val::Num(Num::F64(self.get_f64(*idx))),
            Slot::Ref(idx) => Val::Ref(self.get_ref(*idx)),
            Slot::V128(idx) => Val::Vec_(Vec_::V128(self.get_v128(*idx))),
        }
    }

    /// Set value to slot from Val
    #[inline(always)]
    pub fn set_val(&mut self, slot: &Slot, val: &Val) {
        match slot {
            Slot::I32(idx) => self.set_i32(*idx, val.to_i32().unwrap_or(0)),
            Slot::I64(idx) => self.set_i64(*idx, val.to_i64().unwrap_or(0)),
            Slot::F32(idx) => self.set_f32(*idx, val.to_f32().unwrap_or(0.0)),
            Slot::F64(idx) => self.set_f64(*idx, val.to_f64().unwrap_or(0.0)),
            Slot::Ref(idx) => {
                if let Val::Ref(r) = val {
                    self.set_ref(*idx, r.clone());
                }
            }
            Slot::V128(_) => {}
        }
    }

    /// Write values from value_stack to slots (stack_to_slots operation)
    /// Returns the number of values consumed from value_stack
    #[inline]
    pub fn write_from_stack(&mut self, stack_to_slots: &[Slot], value_stack: &[Val]) -> usize {
        let slot_count = stack_to_slots.len();
        let stack_len = value_stack.len();
        if slot_count == 0 || stack_len < slot_count {
            return 0;
        }
        for (i, slot) in stack_to_slots.iter().enumerate() {
            self.set_val(slot, &value_stack[stack_len - slot_count + i]);
        }
        slot_count
    }
}

/// Slot allocation information (number of slots needed per function)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlotAllocation {
    pub i32_count: usize,
    pub i64_count: usize,
    pub f32_count: usize,
    pub f64_count: usize,
    pub ref_count: usize,
    pub v128_count: usize,
}

/// Slot allocator - tracks stack depth to assign virtual slots
pub struct SlotAllocator {
    // Current stack depth per type
    i32_depth: usize,
    i64_depth: usize,
    f32_depth: usize,
    f64_depth: usize,
    ref_depth: usize,
    v128_depth: usize,

    // Maximum depth reached (used to determine slot file size)
    max_i32_depth: usize,
    max_i64_depth: usize,
    max_f32_depth: usize,
    max_f64_depth: usize,
    max_ref_depth: usize,
    max_v128_depth: usize,

    // Stack order tracking for legacy sync
    stack_order: Vec<Slot>,
}

impl SlotAllocator {
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
            stack_order: Vec::new(),
        };

        // Reserve slots for local variables
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

    /// Push a value onto the stack (allocate a new slot)
    pub fn push(&mut self, vtype: ValueType) -> Slot {
        let slot = match vtype {
            ValueType::NumType(NumType::I32) => {
                let slot = Slot::I32(self.i32_depth as u16);
                self.i32_depth += 1;
                self.max_i32_depth = self.max_i32_depth.max(self.i32_depth);
                slot
            }
            ValueType::NumType(NumType::I64) => {
                let slot = Slot::I64(self.i64_depth as u16);
                self.i64_depth += 1;
                self.max_i64_depth = self.max_i64_depth.max(self.i64_depth);
                slot
            }
            ValueType::NumType(NumType::F32) => {
                let slot = Slot::F32(self.f32_depth as u16);
                self.f32_depth += 1;
                self.max_f32_depth = self.max_f32_depth.max(self.f32_depth);
                slot
            }
            ValueType::NumType(NumType::F64) => {
                let slot = Slot::F64(self.f64_depth as u16);
                self.f64_depth += 1;
                self.max_f64_depth = self.max_f64_depth.max(self.f64_depth);
                slot
            }
            ValueType::RefType(_) => {
                let slot = Slot::Ref(self.ref_depth as u16);
                self.ref_depth += 1;
                self.max_ref_depth = self.max_ref_depth.max(self.ref_depth);
                slot
            }
            ValueType::VecType(_) => {
                let slot = Slot::V128(self.v128_depth as u16);
                self.v128_depth += 1;
                self.max_v128_depth = self.max_v128_depth.max(self.v128_depth);
                slot
            }
        };
        // Track stack order for legacy sync
        self.stack_order.push(slot.clone());
        slot
    }

    /// Pop a value from the stack (decrease depth and return the slot)
    pub fn pop(&mut self, vtype: &ValueType) -> Slot {
        // Remove from stack order tracking
        self.stack_order.pop();
        match vtype {
            ValueType::NumType(NumType::I32) => {
                self.i32_depth = self.i32_depth.saturating_sub(1);
                Slot::I32(self.i32_depth as u16)
            }
            ValueType::NumType(NumType::I64) => {
                self.i64_depth = self.i64_depth.saturating_sub(1);
                Slot::I64(self.i64_depth as u16)
            }
            ValueType::NumType(NumType::F32) => {
                self.f32_depth = self.f32_depth.saturating_sub(1);
                Slot::F32(self.f32_depth as u16)
            }
            ValueType::NumType(NumType::F64) => {
                self.f64_depth = self.f64_depth.saturating_sub(1);
                Slot::F64(self.f64_depth as u16)
            }
            ValueType::RefType(_) => {
                self.ref_depth = self.ref_depth.saturating_sub(1);
                Slot::Ref(self.ref_depth as u16)
            }
            ValueType::VecType(_) => {
                self.v128_depth = self.v128_depth.saturating_sub(1);
                Slot::V128(self.v128_depth as u16)
            }
        }
    }

    /// Pop a value of any type from the stack (uses stack_order to determine type)
    /// Returns the popped slot, or None if stack is empty
    pub fn pop_any_type(&mut self) -> Option<Slot> {
        if let Some(slot) = self.stack_order.pop() {
            match &slot {
                Slot::I32(_) => {
                    self.i32_depth = self.i32_depth.saturating_sub(1);
                }
                Slot::I64(_) => {
                    self.i64_depth = self.i64_depth.saturating_sub(1);
                }
                Slot::F32(_) => {
                    self.f32_depth = self.f32_depth.saturating_sub(1);
                }
                Slot::F64(_) => {
                    self.f64_depth = self.f64_depth.saturating_sub(1);
                }
                Slot::Ref(_) => {
                    self.ref_depth = self.ref_depth.saturating_sub(1);
                }
                Slot::V128(_) => {
                    self.v128_depth = self.v128_depth.saturating_sub(1);
                }
            }
            Some(slot)
        } else {
            None
        }
    }

    /// Peek at the current stack top (without popping)
    pub fn peek(&self, vtype: &ValueType) -> Option<Slot> {
        match vtype {
            ValueType::NumType(NumType::I32) if self.i32_depth > 0 => {
                Some(Slot::I32((self.i32_depth - 1) as u16))
            }
            ValueType::NumType(NumType::I64) if self.i64_depth > 0 => {
                Some(Slot::I64((self.i64_depth - 1) as u16))
            }
            ValueType::NumType(NumType::F32) if self.f32_depth > 0 => {
                Some(Slot::F32((self.f32_depth - 1) as u16))
            }
            ValueType::NumType(NumType::F64) if self.f64_depth > 0 => {
                Some(Slot::F64((self.f64_depth - 1) as u16))
            }
            ValueType::RefType(_) if self.ref_depth > 0 => {
                Some(Slot::Ref((self.ref_depth - 1) as u16))
            }
            ValueType::VecType(_) if self.v128_depth > 0 => {
                Some(Slot::V128((self.v128_depth - 1) as u16))
            }
            _ => None,
        }
    }

    /// Get all active slots on the stack (for syncing to legacy value_stack)
    /// Returns slots in correct stack order (bottom to top)
    pub fn get_active_slots(&self) -> Vec<Slot> {
        self.stack_order.clone()
    }

    /// Clear the current stack depths (slots are consumed by legacy instruction)
    pub fn clear_stack(&mut self) {
        self.i32_depth = 0;
        self.i64_depth = 0;
        self.f32_depth = 0;
        self.f64_depth = 0;
        self.ref_depth = 0;
        self.v128_depth = 0;
        self.stack_order.clear();
    }

    /// Save current stack state for block entry
    pub fn save_state(&self) -> SlotAllocatorState {
        SlotAllocatorState {
            i32_depth: self.i32_depth,
            i64_depth: self.i64_depth,
            f32_depth: self.f32_depth,
            f64_depth: self.f64_depth,
            ref_depth: self.ref_depth,
            v128_depth: self.v128_depth,
            stack_order_len: self.stack_order.len(),
        }
    }

    /// Restore stack state for block exit (keeps max depths intact)
    pub fn restore_state(&mut self, state: &SlotAllocatorState) {
        self.i32_depth = state.i32_depth;
        self.i64_depth = state.i64_depth;
        self.f32_depth = state.f32_depth;
        self.f64_depth = state.f64_depth;
        self.ref_depth = state.ref_depth;
        self.v128_depth = state.v128_depth;
        self.stack_order.truncate(state.stack_order_len);
    }

    /// Finalize and return allocation information
    pub fn finalize(self) -> SlotAllocation {
        SlotAllocation {
            i32_count: self.max_i32_depth,
            i64_count: self.max_i64_depth,
            f32_count: self.max_f32_depth,
            f64_count: self.max_f64_depth,
            ref_count: self.max_ref_depth,
            v128_count: self.max_v128_depth,
        }
    }
}

/// Saved state of SlotAllocator at block entry
#[derive(Clone, Debug)]
pub struct SlotAllocatorState {
    pub i32_depth: usize,
    pub i64_depth: usize,
    pub f32_depth: usize,
    pub f64_depth: usize,
    pub ref_depth: usize,
    pub v128_depth: usize,
    pub stack_order_len: usize,
}
