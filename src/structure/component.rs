//! Internal representation of a parsed component.
//!
//! A component is a list of definitions over several index spaces:
//! core modules, core instances, core functions, tables, memories, globals and core types on the core side,
//! and components, instances, functions, values and types on the component side.
//! The parser resolves every alias while reading, so each index space is a plain vector
//! and an entry records only where the item comes from.
//! Types are resolved into trees that carry no indices, so a type aliased from an enclosing component is a shared pointer.

use super::module::Module;
use super::types::{FuncType as CoreFuncType, GlobalType, MemType, TableType, ValueType};
use crate::shared::Shared;

pub struct Component {
    pub core_modules: Vec<CoreModule>,
    pub core_instances: Vec<CoreInstance>,
    pub core_funcs: Vec<CoreFunc>,
    pub core_tables: Vec<CoreAlias>,
    pub core_memories: Vec<CoreAlias>,
    pub core_globals: Vec<CoreAlias>,
    pub core_types: Vec<CoreType>,
    pub components: Vec<ComponentDef>,
    pub instances: Vec<Instance>,
    pub funcs: Vec<Func>,
    pub values: Vec<Value>,
    pub types: Vec<Type>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    pub start: Option<Start>,
}

impl Component {
    pub fn new() -> Self {
        Component {
            core_modules: Vec::new(),
            core_instances: Vec::new(),
            core_funcs: Vec::new(),
            core_tables: Vec::new(),
            core_memories: Vec::new(),
            core_globals: Vec::new(),
            core_types: Vec::new(),
            components: Vec::new(),
            instances: Vec::new(),
            funcs: Vec::new(),
            values: Vec::new(),
            types: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            start: None,
        }
    }
}

impl Default for Component {
    fn default() -> Self {
        Self::new()
    }
}

/// A sort is the kind of item an index refers to; a component keeps one index space per sort.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort {
    Module,
    Func,
    Value,
    Type,
    Instance,
    Component,
}

/// A core instance exports items of these sorts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreSort {
    Func,
    Table,
    Memory,
    Global,
}

/// An alias is an export of an instance, named by instance index and export name.
#[derive(Clone, Debug)]
pub struct Alias {
    pub instance: u32,
    pub name: String,
}

/// An export of a core instance.
#[derive(Clone, Debug)]
pub struct CoreAlias {
    pub instance: u32,
    pub name: String,
}

#[derive(Clone)]
pub struct CoreModule {
    pub ty: Shared<ModuleType>,
    pub origin: CoreModuleOrigin,
}

#[derive(Clone)]
pub enum CoreModuleOrigin {
    Defined(Shared<Module>),
    Import(u32),
    Alias(Alias),
}

#[derive(Clone)]
pub enum CoreInstance {
    Instantiate {
        module: u32,
        /// Import name and the core instance that satisfies it.
        args: Vec<(String, u32)>,
    },
    FromExports(Vec<CoreInstanceExport>),
}

#[derive(Clone, Debug)]
pub struct CoreInstanceExport {
    pub name: String,
    pub kind: CoreSort,
    pub index: u32,
}

#[derive(Clone)]
pub enum CoreFunc {
    Alias(CoreAlias),
    Lower { func: u32, options: CanonOptions },
    ResourceNew(u32),
    ResourceDrop(u32),
    ResourceRep(u32),
}

#[derive(Clone, Debug)]
pub enum CoreType {
    Func(CoreFuncType),
    Module(Shared<ModuleType>),
}

/// The options of a `canon lift` or `canon lower`.
#[derive(Clone, Debug, Default)]
pub struct CanonOptions {
    pub string_encoding: StringEncoding,
    pub memory: Option<u32>,
    pub realloc: Option<u32>,
    pub post_return: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StringEncoding {
    #[default]
    Utf8,
    Utf16,
    Latin1Utf16,
}

#[derive(Clone)]
pub struct ComponentDef {
    pub ty: Shared<ComponentType>,
    pub origin: ComponentOrigin,
}

#[derive(Clone)]
pub enum ComponentOrigin {
    Defined(Shared<Component>),
    Import(u32),
    Alias(Alias),
}

#[derive(Clone)]
pub struct Instance {
    pub ty: Shared<InstanceType>,
    pub origin: InstanceOrigin,
}

#[derive(Clone)]
pub enum InstanceOrigin {
    Instantiate {
        component: u32,
        args: Vec<InstantiationArg>,
    },
    FromExports(Vec<InstanceExport>),
    Import(u32),
    Alias(Alias),
}

#[derive(Clone, Debug)]
pub struct InstantiationArg {
    pub name: String,
    pub kind: Sort,
    pub index: u32,
}

#[derive(Clone, Debug)]
pub struct InstanceExport {
    pub name: String,
    pub kind: Sort,
    pub index: u32,
}

#[derive(Clone)]
pub struct Func {
    pub ty: Shared<FuncType>,
    pub origin: FuncOrigin,
}

#[derive(Clone)]
pub enum FuncOrigin {
    Lift {
        core_func: u32,
        options: CanonOptions,
    },
    Import(u32),
    Alias(Alias),
}

#[derive(Clone)]
pub struct Value {
    pub ty: ValType,
    pub origin: ValueOrigin,
}

#[derive(Clone)]
pub enum ValueOrigin {
    Import(u32),
    Alias(Alias),
    StartResult,
}

pub struct Import {
    pub name: String,
    pub desc: ExternDesc,
}

pub struct Export {
    pub name: String,
    pub kind: Sort,
    pub index: u32,
    pub desc: ExternDesc,
}

pub struct Start {
    pub func: u32,
    pub args: Vec<u32>,
    pub results: u32,
}

// ---- Types ----

/// An entry in a component's type index space.
#[derive(Clone, Debug)]
pub enum Type {
    Val(ValType),
    Resource(Shared<Resource>),
    Func(Shared<FuncType>),
    Instance(Shared<InstanceType>),
    Component(Shared<ComponentType>),
}

#[derive(Clone, Debug)]
pub enum ValType {
    Bool,
    S8,
    U8,
    S16,
    U16,
    S32,
    U32,
    S64,
    U64,
    F32,
    F64,
    Char,
    String,
    Defined(Shared<DefinedType>),
}

#[derive(Clone, Debug)]
pub enum DefinedType {
    Record(Vec<(String, ValType)>),
    Variant(Vec<(String, Option<ValType>)>),
    List(ValType),
    Tuple(Vec<ValType>),
    Flags(Vec<String>),
    Enum(Vec<String>),
    Option(ValType),
    Result {
        ok: Option<ValType>,
        err: Option<ValType>,
    },
    Own(Shared<Resource>),
    Borrow(Shared<Resource>),
}

/// A resource type. Each definition is its own type, so identity is the pointer.
/// An imported or exported-as-abstract resource has no representation.
#[derive(Debug)]
pub struct Resource {
    pub rep: Option<ValueType>,
    /// Core function index of the destructor in the defining component.
    pub dtor: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct FuncType {
    pub params: Vec<(String, ValType)>,
    pub result: Option<ValType>,
}

#[derive(Clone, Debug, Default)]
pub struct InstanceType {
    pub exports: Vec<(String, ExternDesc)>,
}

#[derive(Clone, Debug, Default)]
pub struct ComponentType {
    pub imports: Vec<(String, ExternDesc)>,
    pub exports: Vec<(String, ExternDesc)>,
}

#[derive(Clone, Debug, Default)]
pub struct ModuleType {
    pub imports: Vec<(String, String, CoreExternType)>,
    pub exports: Vec<(String, CoreExternType)>,
}

#[derive(Clone, Debug)]
pub enum CoreExternType {
    Func(CoreFuncType),
    Table(TableType),
    Memory(MemType),
    Global(GlobalType),
}

/// The type of an import or export.
#[derive(Clone, Debug)]
pub enum ExternDesc {
    Module(Shared<ModuleType>),
    Func(Shared<FuncType>),
    Value(ValType),
    Type(TypeBound),
    Instance(Shared<InstanceType>),
    Component(Shared<ComponentType>),
}

#[derive(Clone, Debug)]
pub enum TypeBound {
    Eq(Type),
    SubResource(Shared<Resource>),
}

impl ExternDesc {
    pub fn sort(&self) -> Sort {
        match self {
            ExternDesc::Module(_) => Sort::Module,
            ExternDesc::Func(_) => Sort::Func,
            ExternDesc::Value(_) => Sort::Value,
            ExternDesc::Type(_) => Sort::Type,
            ExternDesc::Instance(_) => Sort::Instance,
            ExternDesc::Component(_) => Sort::Component,
        }
    }
}

impl TypeBound {
    /// The type an import or export with this bound introduces.
    pub fn to_type(&self) -> Type {
        match self {
            TypeBound::Eq(ty) => ty.clone(),
            TypeBound::SubResource(resource) => Type::Resource(Shared::clone(resource)),
        }
    }
}
