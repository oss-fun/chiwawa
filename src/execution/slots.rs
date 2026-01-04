use crate::execution::value::Ref;
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

/// Slot file - holds all type-specialized slots
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlotFile {
    pub i32_slots: Vec<i32>,
    pub i64_slots: Vec<i64>,
    pub f32_slots: Vec<f32>,
    pub f64_slots: Vec<f64>,
    pub ref_slots: Vec<Ref>,
    pub v128_slots: Vec<i128>,
}

impl SlotFile {
    /// Create a new slot file with specified capacities
    pub fn new(
        i32_count: usize,
        i64_count: usize,
        f32_count: usize,
        f64_count: usize,
        ref_count: usize,
        v128_count: usize,
    ) -> Self {
        Self {
            i32_slots: vec![0; i32_count],
            i64_slots: vec![0; i64_count],
            f32_slots: vec![0.0; f32_count],
            f64_slots: vec![0.0; f64_count],
            ref_slots: vec![Ref::RefNull; ref_count],
            v128_slots: vec![0; v128_count],
        }
    }

    /// Get/set methods for each type (inlined for performance)
    #[inline(always)]
    pub fn get_i32(&self, slot: u16) -> i32 {
        self.i32_slots[slot as usize]
    }

    #[inline(always)]
    pub fn set_i32(&mut self, slot: u16, val: i32) {
        self.i32_slots[slot as usize] = val;
    }

    #[inline(always)]
    pub fn get_i64(&self, slot: u16) -> i64 {
        self.i64_slots[slot as usize]
    }

    #[inline(always)]
    pub fn set_i64(&mut self, slot: u16, val: i64) {
        self.i64_slots[slot as usize] = val;
    }

    #[inline(always)]
    pub fn get_f32(&self, slot: u16) -> f32 {
        self.f32_slots[slot as usize]
    }

    #[inline(always)]
    pub fn set_f32(&mut self, slot: u16, val: f32) {
        self.f32_slots[slot as usize] = val;
    }

    #[inline(always)]
    pub fn get_f64(&self, slot: u16) -> f64 {
        self.f64_slots[slot as usize]
    }

    #[inline(always)]
    pub fn set_f64(&mut self, slot: u16, val: f64) {
        self.f64_slots[slot as usize] = val;
    }

    #[inline(always)]
    pub fn get_ref(&self, slot: u16) -> Ref {
        self.ref_slots[slot as usize].clone()
    }

    #[inline(always)]
    pub fn set_ref(&mut self, slot: u16, val: Ref) {
        self.ref_slots[slot as usize] = val;
    }

    #[inline(always)]
    pub fn get_v128(&self, slot: u16) -> i128 {
        self.v128_slots[slot as usize]
    }

    #[inline(always)]
    pub fn set_v128(&mut self, slot: u16, val: i128) {
        self.v128_slots[slot as usize] = val;
    }

    #[inline(always)]
    pub fn get_i32_slots(&mut self) -> &mut [i32] {
        &mut self.i32_slots
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
        match vtype {
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
        }
    }

    /// Pop a value from the stack (decrease depth and return the slot)
    pub fn pop(&mut self, vtype: ValueType) -> Slot {
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

    /// Peek at the current stack top (without popping)
    pub fn peek(&self, vtype: ValueType) -> Option<Slot> {
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
