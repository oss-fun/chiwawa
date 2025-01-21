#[derive(PartialEq, Debug, Clone)]
pub enum ValueType {
    NumType(NumType),
    VecType(VecType),
    RefType(RefType),
}

#[derive(PartialEq, Debug, Clone)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(PartialEq, Debug, Clone)]
pub enum VecType {
    V128,
}

#[derive(PartialEq, Debug, Clone)]
pub enum RefType{
    FuncRef,
    ExternalRef,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FuncType{
    pub params: Vec<ValueType>,
    pub results: Vec<ValueType>,
}

impl FuncType{
    pub fn type_match(&self, other: &FuncType) -> bool {
        self == other
    }
}

pub trait GetIdx
where
Self: Into<u32>,
{
    fn to_usize(self) -> usize {
        self.into() as usize
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct TypeIdx(pub u32); 
impl Into<u32> for TypeIdx{
    fn into(self) -> u32{
        self.0
    }
}
impl GetIdx for TypeIdx{}

#[derive(PartialEq, Debug, Clone)]
pub struct TableIdx(pub u32);
impl Into<u32> for TableIdx{
    fn into(self) -> u32{
        self.0
    }
}
impl GetIdx for TableIdx{}

#[derive(PartialEq, Debug, Clone)]
pub struct MemIdx(pub u32); 
impl Into<u32> for MemIdx{
    fn into(self) -> u32{
        self.0
    }
}
impl GetIdx for MemIdx{}

#[derive(PartialEq, Debug, Clone)]
pub struct FuncIdx(pub u32); 
impl Into<u32> for FuncIdx{
    fn into(self) -> u32{
        self.0
    }
}
impl GetIdx for FuncIdx{}

#[derive(PartialEq, Debug, Clone)]
pub struct GlobalIdx(pub u32); 
impl Into<u32> for GlobalIdx{
    fn into(self) -> u32{
        self.0
    }
}
impl GetIdx for GlobalIdx{}

#[derive(PartialEq, Debug, Clone)]
pub struct LocalIdx(pub u32); 
#[derive(PartialEq, Debug, Clone)]
pub struct LaneIdx(pub u8); 
#[derive(PartialEq, Debug, Clone)]
pub struct DataIdx(pub u32); 
#[derive(PartialEq, Debug, Clone)]
pub struct LabelIdx(pub u32); 
impl Into<u32> for LabelIdx{
    fn into(self) -> u32{
        self.0
    }
}
impl GetIdx for LabelIdx{}

#[derive(PartialEq, Debug, Clone)]
pub struct ElemIdx(pub u32);

#[derive(Debug,PartialEq, Clone)]
pub struct BlockType(pub Option<TypeIdx>, pub Option<ValueType>);

pub struct Byte(pub u8);
pub struct Name(pub String);

#[derive(PartialEq, Clone)]
pub struct TableType (pub Limits, pub RefType);

#[derive(PartialEq, Clone)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(PartialEq, Clone)]
pub struct MemType (pub Limits);

#[derive(PartialEq, Clone)]
pub struct GlobalType (pub Mut , pub ValueType);

#[derive(PartialEq, Clone)]
pub enum Mut {
    Const,
    Var,
}