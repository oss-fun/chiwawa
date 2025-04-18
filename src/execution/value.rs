use crate::structure::types::{ValueType, NumType, VecType};
use std::sync::{Arc, RwLock};
use super::{mem::MemAddr, global::GlobalAddr, func::FuncAddr, table::TableAddr};
use crate::error::RuntimeError;

#[derive(Clone, Debug)]
pub enum Val {
    Num(Num),
    Vec_(Vec_),
    Ref(Ref),
}

impl Val {
    pub fn to_i32(&self) -> Result<i32, RuntimeError> {
        match self {
            Val::Num(Num::I32(v)) => Ok(*v),
            _ => Err(RuntimeError::TypeMismatch),
        }
    }
    pub fn to_i64(&self) -> Result<i64, RuntimeError> {
        match self {
            Val::Num(Num::I64(v)) => Ok(*v),
            _ => Err(RuntimeError::TypeMismatch),
        }
    }
    pub fn to_f32(&self) -> Result<f32, RuntimeError> {
        match self {
            Val::Num(Num::F32(v)) => Ok(*v),
            _ => Err(RuntimeError::TypeMismatch),
        }
    }
    pub fn to_f64(&self) -> Result<f64, RuntimeError> {
        match self {
            Val::Num(Num::F64(v)) => Ok(*v),
            _ => Err(RuntimeError::TypeMismatch),
        }
    }
    pub fn val_type(&self) -> ValueType {
        match self {
            Val::Num(Num::I32(_)) => ValueType::NumType(NumType::I32),
            Val::Num(Num::I64(_))  => ValueType::NumType(NumType::I64),
            Val::Num(Num::F32(_))  => ValueType::NumType(NumType::F32),
            Val::Num(Num::F64(_))  => ValueType::NumType(NumType::F64),
            Val::Vec_(Vec_::V128(_))  => ValueType::VecType(VecType::V128),
            Val::Ref(_) => todo!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Num {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

#[derive(Clone, Debug)]
pub enum Vec_ {
    V128(i128),
}

#[derive(Clone, Debug)]
pub enum Ref {
    RefNull,
    FuncAddr(FuncAddr),
    RefExtern(ExternAddr),
}

#[derive(Clone, Debug)]
pub struct ExternAddr(Arc<RwLock<Externval>>);

#[derive(Clone, Debug)]
pub enum Externval {
    Func(FuncAddr),
    Table(TableAddr),
    Mem(MemAddr),
    Global(GlobalAddr),
}

impl Externval {
    pub fn as_func(self) -> Option<FuncAddr> {
        if let Externval::Func(x) = self {
            Some(x)
        } else {
            None
        }
    }
}
