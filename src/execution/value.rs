use crate::structure::types::ValueType;
use std::{rc::Rc, cell::RefCell};
use super::{mem::MemAddr, global::GlobalAddr};

pub enum Val {
    Num(Num),
    Vec_(Vec_),
    Ref(Ref),
}

pub enum Num {
    I32(i32),
    I64(i64),
    F32(u32),
    F64(u64),
}

pub enum Vec_ {
    V128(i128),   
}

pub enum Ref {
    RefNull,
    RefExtern(ExternAddr),
}

pub struct ExternAddr(Rc<RefCell<Externval>>);
pub enum Externval {
    Func(MemAddr),
    Global(GlobalAddr),
}