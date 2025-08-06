use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub enum ValueType {
    NumType(NumType),
    VecType(VecType),
    RefType(RefType),
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub enum VecType {
    V128,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub enum RefType {
    FuncRef,
    ExternalRef,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FuncType {
    pub params: Vec<ValueType>,
    pub results: Vec<ValueType>,
}

impl FuncType {
    pub fn type_match(&self, other: &FuncType) -> bool {
        self == other
    }

    pub fn params(&self) -> &Vec<ValueType> {
        &self.params
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

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct TypeIdx(pub u32);
impl Into<u32> for TypeIdx {
    fn into(self) -> u32 {
        self.0
    }
}
impl GetIdx for TypeIdx {}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct TableIdx(pub u32);
impl Into<u32> for TableIdx {
    fn into(self) -> u32 {
        self.0
    }
}
impl GetIdx for TableIdx {}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct MemIdx(pub u32);
impl Into<u32> for MemIdx {
    fn into(self) -> u32 {
        self.0
    }
}
impl GetIdx for MemIdx {}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct FuncIdx(pub u32);
impl Into<u32> for FuncIdx {
    fn into(self) -> u32 {
        self.0
    }
}
impl GetIdx for FuncIdx {}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct GlobalIdx(pub u32);
impl Into<u32> for GlobalIdx {
    fn into(self) -> u32 {
        self.0
    }
}
impl GetIdx for GlobalIdx {}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct LocalIdx(pub u32);
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct LaneIdx(pub u8);
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct DataIdx(pub u32);
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct LabelIdx(pub u32);
impl Into<u32> for LabelIdx {
    fn into(self) -> u32 {
        self.0
    }
}
impl GetIdx for LabelIdx {}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct ElemIdx(pub u32);

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BlockType(pub Option<TypeIdx>, pub Option<ValueType>);

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Byte(pub u8);
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Name(pub String);

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct TableType(pub Limits, pub RefType);

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct MemType(pub Limits);

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct GlobalType(pub Mut, pub ValueType);

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Mut {
    Const,
    Var,
}
