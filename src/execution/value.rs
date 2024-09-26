use crate::structure::types::ValueType;
use super::instance::{MemAddr, GlobalAddr};

pub enum Val {
    Num(Num),
    Vec(Vec),
    Ref(Ref),
}

enum Num {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

enum Vec {
    V128(i128),   
}

enum Ref {

}

pub enum Externval {
    Func(MemAddr),
    Global(GlobalAddr),
}