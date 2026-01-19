//! WebAssembly type definitions.
//!
//! This module defines types used throughout Chiwawa to represent WebAssembly's
//! type system, including value types, function types, and index types.
//!
//! ## Type Hierarchy
//!
//! ```text
//! ValueType
//! ├── NumType (i32, i64, f32, f64)
//! ├── VecType (v128)
//! └── RefType (funcref, externref)
//! ```

use serde::{Deserialize, Serialize};

/// WebAssembly value type.
///
/// Represents the types that can appear on the operand stack and in locals/globals.
#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub enum ValueType {
    NumType(NumType),
    VecType(VecType),
    RefType(RefType),
}

/// Numeric types (i32, i64, f32, f64).
#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

/// SIMD vector type (v128).
#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub enum VecType {
    V128,
}

/// Reference types (funcref, externref).
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RefType {
    FuncRef,
    ExternalRef,
}

/// Function type signature.
///
/// Defines the parameter and result types of a function.
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

/// Trait for index types that can be converted to `usize`.
pub trait GetIdx
where
    Self: Into<u32>,
{
    fn to_usize(self) -> usize {
        self.into() as usize
    }
}

/// Index into the type section.
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TypeIdx(pub u32);
impl Into<u32> for TypeIdx {
    fn into(self) -> u32 {
        self.0
    }
}
impl GetIdx for TypeIdx {}

/// Index into the table section.
#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TableIdx(pub u32);
impl Into<u32> for TableIdx {
    fn into(self) -> u32 {
        self.0
    }
}
impl GetIdx for TableIdx {}

/// Index into the memory section.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct MemIdx(pub u32);
impl Into<u32> for MemIdx {
    fn into(self) -> u32 {
        self.0
    }
}
impl GetIdx for MemIdx {}

/// Index into the function section.
#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FuncIdx(pub u32);
impl Into<u32> for FuncIdx {
    fn into(self) -> u32 {
        self.0
    }
}
impl GetIdx for FuncIdx {}

/// Index into the global section.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct GlobalIdx(pub u32);
impl Into<u32> for GlobalIdx {
    fn into(self) -> u32 {
        self.0
    }
}
impl GetIdx for GlobalIdx {}

/// Index into local variables within a function.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct LocalIdx(pub u32);

/// SIMD lane index (0-15 for v128).
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct LaneIdx(pub u8);

/// Index into the data section.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct DataIdx(pub u32);

/// Label index for branch instructions.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct LabelIdx(pub u32);
impl Into<u32> for LabelIdx {
    fn into(self) -> u32 {
        self.0
    }
}
impl GetIdx for LabelIdx {}

/// Index into the element section.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct ElemIdx(pub u32);

/// Block type for structured control flow.
///
/// Can be either a type index (for multi-value blocks) or an optional value type
/// (for single-result or void blocks).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BlockType(pub Option<TypeIdx>, pub Option<ValueType>);

/// A single byte value.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Byte(pub u8);

/// UTF-8 encoded name string.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Name(pub String);

/// Table type specifying limits and element reference type.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct TableType(pub Limits, pub RefType);

/// Size limits for tables and memories.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Limits {
    /// Minimum size (in pages for memory, elements for tables).
    pub min: u32,
    /// Optional maximum size.
    pub max: Option<u32>,
}

/// Memory type specifying size limits.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct MemType(pub Limits);

/// Global type specifying mutability and value type.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct GlobalType(pub Mut, pub ValueType);

/// Mutability indicator for globals.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Mut {
    /// Immutable global.
    Const,
    /// Mutable global.
    Var,
}
