use crate::structure::types::ValueType;
use std::{rc::Rc, cell::RefCell};
use super::{mem::MemAddr, global::GlobalAddr, func::FuncAddr, table::TableAddr};

#[derive(Clone)]
pub enum Val {
    Num(Num),
    Vec_(Vec_),
    Ref(Ref),
}

impl Val {
    pub fn to_i32(&self) ->i32{
        if let Val::Num(Num::I32(num)) = self {
            *num
        } else {
            panic!();
        }
    }
}

#[derive(Clone)]
pub enum Num {
    I32(i32),
    I64(i64),
    F32(u32),
    F64(u64),
}

#[derive(Clone)]
pub enum Vec_ {
    V128(i128),   
}

#[derive(Clone)]
pub enum Ref {
    RefNull,
    FuncAddr(FuncAddr),
    RefExtern(ExternAddr),
}

#[derive(Clone)]
pub struct ExternAddr(Rc<RefCell<Externval>>);

#[derive(Clone)]
pub enum Externval {
    Func(FuncAddr),
    Table(TableAddr),
    Mem(MemAddr),
    Global(GlobalAddr),
}