pub enum ValueType {
    NumType,
    VecType,
    RefType,
}

pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

pub enum VecType {
    V128,
}

pub enum RefType{
    FuncRef,
    ExternalRef,
}

pub struct FuncType{
    param: Vec<ValueType>,
    results: Vec<ValueType>,
}
pub struct TypeIdx(u32); 
