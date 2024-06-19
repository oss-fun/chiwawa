pub enum ValueType {
    NumType(NumType),
    VecType(VecType),
    RefType(RefType),
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

pub struct TypeIdx(pub u32); 
pub struct TableIdx(u32); 
pub struct MemIdx(u32); 
pub struct FuncIdx(u32); 
pub struct GlobalIdx(u32); 


pub struct Byte(u8);
pub struct Name(pub String);

pub struct TableType (pub Limits, pub RefType);

pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

pub struct MemType (pub Limits);

pub struct GlobalType (pub Mut , pub ValueType);

pub enum Mut {
    Const,
    Var,
}