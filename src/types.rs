#[derive(PartialEq, Debug)]
pub enum ValueType {
    NumType(NumType),
    VecType(VecType),
    RefType(RefType),
}

#[derive(PartialEq, Debug)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(PartialEq, Debug)]
pub enum VecType {
    V128,
}

#[derive(PartialEq, Debug)]
pub enum RefType{
    FuncRef,
    ExternalRef,
}
#[derive(Debug)]
pub struct FuncType{
    pub params: Vec<ValueType>,
    pub results: Vec<ValueType>,
}

#[derive(PartialEq, Debug)]
pub struct TypeIdx(pub u32); 
#[derive(PartialEq, Debug)]
pub struct TableIdx(pub u32); 
pub struct MemIdx(pub u32); 
#[derive(Debug, PartialEq)]
pub struct FuncIdx(pub u32); 
#[derive(Debug,PartialEq)]
pub struct GlobalIdx(pub u32); 
#[derive(Debug,PartialEq)]
pub struct LocalIdx(pub u32); 
#[derive(Debug,PartialEq)]
pub struct LaneIdx(pub u8); 
#[derive(Debug,PartialEq)]
pub struct DataIdx(pub u32); 
#[derive(Debug,PartialEq)]
pub struct LabelIdx(pub u32); 
#[derive(Debug,PartialEq)]
pub struct ElemIdx(pub u32);

#[derive(Debug,PartialEq)]
pub struct BlockType(pub Option<TypeIdx>, pub Option<ValueType>);

pub struct Byte(pub u8);
pub struct Name(pub String);

#[derive(PartialEq)]
pub struct TableType (pub Limits, pub RefType);

#[derive(PartialEq)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(PartialEq)]
pub struct MemType (pub Limits);

#[derive(PartialEq)]
pub struct GlobalType (pub Mut , pub ValueType);

#[derive(PartialEq)]
pub enum Mut {
    Const,
    Var,
}