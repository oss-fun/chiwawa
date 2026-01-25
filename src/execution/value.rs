//! Runtime value types (Num, Vec, Ref) and external values.

use super::{func::FuncAddr, global::GlobalAddr, mem::MemAddr, table::TableAddr};
use crate::error::RuntimeError;
use crate::structure::module::WasiFuncType;
use crate::structure::types::{NumType, RefType, ValueType, VecType};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

/// WebAssembly runtime value.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
pub enum Val {
    Num(Num),
    Vec_(Vec_),
    Ref(Ref),
}

impl Val {
    /// Extracts i32 value or returns TypeMismatch error.
    pub fn to_i32(&self) -> Result<i32, RuntimeError> {
        match self {
            Val::Num(Num::I32(v)) => Ok(*v),
            _ => Err(RuntimeError::TypeMismatch),
        }
    }
    /// Extracts i64 value or returns TypeMismatch error.
    pub fn to_i64(&self) -> Result<i64, RuntimeError> {
        match self {
            Val::Num(Num::I64(v)) => Ok(*v),
            _ => Err(RuntimeError::TypeMismatch),
        }
    }
    /// Extracts f32 value or returns TypeMismatch error.
    pub fn to_f32(&self) -> Result<f32, RuntimeError> {
        match self {
            Val::Num(Num::F32(v)) => Ok(*v),
            _ => Err(RuntimeError::TypeMismatch),
        }
    }
    /// Extracts f64 value or returns TypeMismatch error.
    pub fn to_f64(&self) -> Result<f64, RuntimeError> {
        match self {
            Val::Num(Num::F64(v)) => Ok(*v),
            _ => Err(RuntimeError::TypeMismatch),
        }
    }
    /// Returns the WebAssembly type of this value.
    pub fn val_type(&self) -> ValueType {
        match self {
            Val::Num(Num::I32(_)) => ValueType::NumType(NumType::I32),
            Val::Num(Num::I64(_)) => ValueType::NumType(NumType::I64),
            Val::Num(Num::F32(_)) => ValueType::NumType(NumType::F32),
            Val::Num(Num::F64(_)) => ValueType::NumType(NumType::F64),
            Val::Vec_(Vec_::V128(_)) => ValueType::VecType(VecType::V128),
            Val::Ref(Ref::RefNull) => ValueType::RefType(RefType::FuncRef),
            Val::Ref(Ref::FuncAddr(_)) => ValueType::RefType(RefType::FuncRef),
            Val::Ref(Ref::RefExtern(_)) => ValueType::RefType(RefType::ExternalRef),
        }
    }
}

/// Numeric value variants (i32, i64, f32, f64).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Num {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Hash for Num {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Num::I32(v) => {
                0.hash(state);
                v.hash(state);
            }
            Num::I64(v) => {
                1.hash(state);
                v.hash(state);
            }
            Num::F32(v) => {
                2.hash(state);
                v.to_bits().hash(state);
            }
            Num::F64(v) => {
                3.hash(state);
                v.to_bits().hash(state);
            }
        }
    }
}

/// SIMD vector value (128-bit).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Hash)]
pub enum Vec_ {
    V128(i128),
}

/// Reference value variants (null, function, external).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Ref {
    /// Null reference.
    RefNull,
    /// Function reference.
    #[serde(skip)]
    FuncAddr(FuncAddr),
    /// External reference.
    #[serde(skip)]
    RefExtern(ExternAddr),
}

use std::hash::{Hash, Hasher};

impl Hash for Ref {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Ref::RefNull => 0.hash(state),
            Ref::FuncAddr(addr) => {
                1.hash(state);
                // Hash the pointer address for FuncAddr
                (addr as *const _ as usize).hash(state);
            }
            Ref::RefExtern(addr) => {
                2.hash(state);
                // Hash the Rc pointer address for ExternAddr
                Rc::as_ptr(&addr.0).hash(state);
            }
        }
    }
}

impl PartialEq for Ref {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Ref::RefNull, Ref::RefNull) => true,
            (Ref::FuncAddr(a), Ref::FuncAddr(b)) => {
                // Compare FuncAddr by pointer equality
                std::ptr::eq(a as *const _, b as *const _)
            }
            (Ref::RefExtern(a), Ref::RefExtern(b)) => Rc::ptr_eq(&a.0, &b.0),
            _ => false,
        }
    }
}

/// External address wrapping an external value.
#[derive(Clone, Debug)]
pub struct ExternAddr(Rc<RefCell<Externval>>);

impl ExternAddr {
    /// Creates a new external address from an external value.
    pub fn new(externval: Externval) -> Self {
        ExternAddr(Rc::new(RefCell::new(externval)))
    }
}

/// WASI function address containing the function type.
#[derive(Clone, Debug)]
pub struct WasiFuncAddr {
    pub func_type: WasiFuncType,
}

impl WasiFuncAddr {
    /// Creates a new WASI function address from a function type.
    pub fn new(func_type: WasiFuncType) -> Self {
        Self { func_type }
    }
}

/// External value variants for imports/exports.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Externval {
    /// Function external value.
    #[serde(skip)]
    Func(FuncAddr),
    /// Table external value.
    #[serde(skip)]
    Table(TableAddr),
    /// Memory external value.
    #[serde(skip)]
    Mem(MemAddr),
    /// Global external value.
    #[serde(skip)]
    Global(GlobalAddr),
    /// WASI function external value.
    #[serde(skip)]
    WasiFunc(WasiFuncAddr),
}

impl Externval {
    /// Extracts function address if this is a Func variant.
    pub fn as_func(self) -> Option<FuncAddr> {
        if let Externval::Func(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// Extracts WASI function address if this is a WasiFunc variant.
    pub fn as_wasi_func(self) -> Option<WasiFuncAddr> {
        if let Externval::WasiFunc(x) = self {
            Some(x)
        } else {
            None
        }
    }
}
