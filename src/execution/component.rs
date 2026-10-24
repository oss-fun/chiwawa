//! Component instances: instantiation, and calls that cross the canonical ABI.
//!
//! Entries refer across index spaces, so the linker follows references instead of filling one space at a time:
//! each resolver builds its entry by calling the resolvers of what the entry refers to.
//! Each entry must become one object, so every resolver goes through a memo table and a second request returns the first result.
//! The definitions with side effects are forced in the order they appear in the binary,
//! so instantiations and the start function run in that order even when nothing refers to them.
//! See `doc/component.md`.

use super::func::{FuncAddr, FuncInst};
use super::global::GlobalAddr;
use super::mem::MemAddr;
use super::module::{ImportObjects, ModuleInst};
use super::runtime::{run_start_section, Runtime, RuntimeConfig};
use super::table::TableAddr;
use super::value::{Externval, Num, Val};
use crate::error::RuntimeError;
use crate::shared::Shared;
use crate::structure::component::*;
use crate::structure::module::Module;
use crate::structure::types::{FuncType as CoreFuncType, NumType, ValueType};
use rustc_hash::FxHashMap;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

type Result<T> = std::result::Result<T, RuntimeError>;

fn error(message: impl Into<String>) -> RuntimeError {
    RuntimeError::ComponentError(message.into())
}

fn unsupported(feature: &'static str) -> RuntimeError {
    RuntimeError::UnsupportedComponentFeature(feature)
}

/// A component-level value.
#[derive(Clone, Debug, PartialEq)]
pub enum ComponentVal {
    Bool(bool),
    S8(i8),
    U8(u8),
    S16(i16),
    U16(u16),
    S32(i32),
    U32(u32),
    S64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    Char(char),
    /// The names of the set flags.
    Flags(Vec<String>),
}

/// A component-level function: a core function lifted to component values.
pub struct ComponentFunc {
    pub ty: Shared<FuncType>,
    core: FuncAddr,
    post_return: Option<FuncAddr>,
}

impl ComponentFunc {
    /// Calls the function with component values and returns its results as component values.
    pub fn call(&self, args: Vec<ComponentVal>) -> Result<Vec<ComponentVal>> {
        if args.len() != self.ty.params.len() {
            return Err(RuntimeError::InvalidParameterCount);
        }
        let params = self
            .ty
            .params
            .iter()
            .zip(&args)
            .map(|((_, ty), arg)| lower_val(ty, arg))
            .collect::<Result<Vec<_>>>()?;
        let results = call_core(&self.core, params)?;
        let lifted = match (&self.ty.result, results.as_slice()) {
            (None, []) => Vec::new(),
            (Some(ty), [val]) => vec![lift_val(ty, val)?],
            _ => return Err(error("core function returned the wrong number of results")),
        };
        if let Some(post_return) = &self.post_return {
            call_core(post_return, results)?;
        }
        Ok(lifted)
    }
}

fn call_core(func: &FuncAddr, params: Vec<Val>) -> Result<Vec<Val>> {
    match func.read_lock() {
        FuncInst::RuntimeFunc { module, .. } => {
            let inst = module
                .upgrade()
                .ok_or_else(|| error("the function's instance is gone"))?;
            Runtime::new(inst, func, params, RuntimeConfig::default())?.run()
        }
        FuncInst::HostFunc { host_code, .. } => Ok(host_code(params)?.into_iter().collect()),
        FuncInst::WasiFunc { .. } => Err(RuntimeError::UnimplementedHostFunction),
    }
}

/// `canon lower`: a core function that lifts its arguments, calls `func`, and lowers the result.
fn lower_func(func: Rc<ComponentFunc>) -> Result<FuncAddr> {
    let core_ty = flat_func_type(&func.ty)?;
    let ty = Shared::clone(&func.ty);
    Ok(FuncAddr::alloc_host(
        core_ty,
        Rc::new(move |params: Vec<Val>| {
            let args = ty
                .params
                .iter()
                .zip(&params)
                .map(|((_, ty), val)| lift_val(ty, val))
                .collect::<Result<Vec<_>>>()?;
            let results = func.call(args)?;
            match (&ty.result, results.first()) {
                (Some(ty), Some(val)) => Ok(Some(lower_val(ty, val)?)),
                _ => Ok(None),
            }
        }),
    ))
}

const I32: ValueType = ValueType::NumType(NumType::I32);
const I64: ValueType = ValueType::NumType(NumType::I64);
const F32: ValueType = ValueType::NumType(NumType::F32);
const F64: ValueType = ValueType::NumType(NumType::F64);

fn flat_type(ty: &ValType) -> Result<ValueType> {
    Ok(match ty {
        ValType::Bool
        | ValType::S8
        | ValType::U8
        | ValType::S16
        | ValType::U16
        | ValType::S32
        | ValType::U32
        | ValType::Char => I32,
        ValType::S64 | ValType::U64 => I64,
        ValType::F32 => F32,
        ValType::F64 => F64,
        ValType::String => return Err(unsupported("string values")),
        ValType::Defined(defined) => match &**defined {
            DefinedType::Flags(names) if names.len() <= 32 => I32,
            DefinedType::Flags(_) => return Err(unsupported("flags with more than 32 names")),
            _ => return Err(unsupported("compound values")),
        },
    })
}

/// The core type of a function whose values are all passed in registers.
fn flat_func_type(ty: &FuncType) -> Result<CoreFuncType> {
    if ty.params.len() > 16 {
        return Err(unsupported("more than 16 flat parameters"));
    }
    Ok(CoreFuncType {
        params: ty
            .params
            .iter()
            .map(|(_, ty)| flat_type(ty))
            .collect::<Result<_>>()?,
        results: ty.result.iter().map(flat_type).collect::<Result<_>>()?,
    })
}

fn canonical_nan32(x: f32) -> f32 {
    if x.is_nan() {
        f32::from_bits(0x7fc0_0000)
    } else {
        x
    }
}

fn canonical_nan64(x: f64) -> f64 {
    if x.is_nan() {
        f64::from_bits(0x7ff8_0000_0000_0000)
    } else {
        x
    }
}

fn lift_val(ty: &ValType, val: &Val) -> Result<ComponentVal> {
    Ok(match ty {
        ValType::Bool => ComponentVal::Bool(val.to_i32()? != 0),
        ValType::S8 => ComponentVal::S8(val.to_i32()? as i8),
        ValType::U8 => ComponentVal::U8(val.to_i32()? as u8),
        ValType::S16 => ComponentVal::S16(val.to_i32()? as i16),
        ValType::U16 => ComponentVal::U16(val.to_i32()? as u16),
        ValType::S32 => ComponentVal::S32(val.to_i32()?),
        ValType::U32 => ComponentVal::U32(val.to_i32()? as u32),
        ValType::S64 => ComponentVal::S64(val.to_i64()?),
        ValType::U64 => ComponentVal::U64(val.to_i64()? as u64),
        ValType::F32 => ComponentVal::F32(canonical_nan32(val.to_f32()?)),
        ValType::F64 => ComponentVal::F64(canonical_nan64(val.to_f64()?)),
        ValType::Char => char::from_u32(val.to_i32()? as u32)
            .map(ComponentVal::Char)
            .ok_or(RuntimeError::InvalidCharBitPattern)?,
        ValType::String => return Err(unsupported("string values")),
        ValType::Defined(defined) => match &**defined {
            DefinedType::Flags(names) => {
                let bits = val.to_i32()? as u32;
                ComponentVal::Flags(
                    names
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i < 32 && (bits >> i) & 1 == 1)
                        .map(|(_, name)| name.clone())
                        .collect(),
                )
            }
            _ => return Err(unsupported("compound values")),
        },
    })
}

fn lower_val(ty: &ValType, val: &ComponentVal) -> Result<Val> {
    let i32_val = |v: i32| Val::Num(Num::I32(v));
    Ok(match (ty, val) {
        (ValType::Bool, ComponentVal::Bool(v)) => i32_val(*v as i32),
        (ValType::S8, ComponentVal::S8(v)) => i32_val(*v as i32),
        (ValType::U8, ComponentVal::U8(v)) => i32_val(*v as i32),
        (ValType::S16, ComponentVal::S16(v)) => i32_val(*v as i32),
        (ValType::U16, ComponentVal::U16(v)) => i32_val(*v as i32),
        (ValType::S32, ComponentVal::S32(v)) => i32_val(*v),
        (ValType::U32, ComponentVal::U32(v)) => i32_val(*v as i32),
        (ValType::S64, ComponentVal::S64(v)) => Val::Num(Num::I64(*v)),
        (ValType::U64, ComponentVal::U64(v)) => Val::Num(Num::I64(*v as i64)),
        (ValType::F32, ComponentVal::F32(v)) => Val::Num(Num::F32(canonical_nan32(*v))),
        (ValType::F64, ComponentVal::F64(v)) => Val::Num(Num::F64(canonical_nan64(*v))),
        (ValType::Char, ComponentVal::Char(c)) => i32_val(*c as i32),
        (ValType::Defined(defined), ComponentVal::Flags(set)) => match &**defined {
            DefinedType::Flags(names) => {
                let mut bits = 0u32;
                for name in set {
                    let i = names
                        .iter()
                        .position(|n| n == name)
                        .ok_or_else(|| error(format!("unknown flag {name}")))?;
                    bits |= 1 << i;
                }
                i32_val(bits as i32)
            }
            _ => return Err(unsupported("compound values")),
        },
        _ => return Err(RuntimeError::TypeMismatch),
    })
}

/// A component-level extern
pub enum ExternVal {
    Module(Shared<Module>),
    Func(Rc<ComponentFunc>),
    Value(ComponentVal),
    Type,
    Instance(Rc<InstanceInst>),
    Component(Rc<ComponentClosure>),
}

/// A component definition with the outer definitions it captured, ready to instantiate.
pub struct ComponentClosure {
    definition: Shared<Component>,
    /// One scope per enclosing component, outermost first.
    outer: Vec<Rc<Scope>>,
}

/// What a nested definition's subtree takes from one enclosing component through outer aliases.
#[derive(Default)]
struct Scope {
    core_modules: FxHashMap<u32, Shared<Module>>,
    components: FxHashMap<u32, Rc<ComponentClosure>>,
}

impl ExternVal {
    fn sort(&self) -> Sort {
        match self {
            ExternVal::Module(_) => Sort::Module,
            ExternVal::Func(_) => Sort::Func,
            ExternVal::Value(_) => Sort::Value,
            ExternVal::Type => Sort::Type,
            ExternVal::Instance(_) => Sort::Instance,
            ExternVal::Component(_) => Sort::Component,
        }
    }
}

/// A component-level instance
pub struct InstanceInst {
    exports: Vec<(String, ExternVal)>,
    /// The instantiated component, which keeps its core instances alive.
    _inner: Option<Rc<ComponentInst>>,
}

impl InstanceInst {
    pub fn export(&self, name: &str) -> Option<&ExternVal> {
        self.exports
            .iter()
            .find(|(export, _)| export == name)
            .map(|(_, val)| val)
    }
}

/// An instantiated component.
pub struct ComponentInst {
    pub exports: Vec<(String, ExternVal)>,
    /// Kept alive by the instance, since a function address holds only a weak reference to its module.
    _core_instances: Vec<Rc<ModuleInst>>,
    _instances: Vec<Rc<InstanceInst>>,
}

impl ComponentInst {
    pub fn new(
        component: &Component,
        imports: Vec<(String, ExternVal)>,
    ) -> Result<Rc<ComponentInst>> {
        Linker::new(component, imports, Vec::new()).instantiate()
    }

    pub fn export(&self, name: &str) -> Option<&ExternVal> {
        self.exports
            .iter()
            .find(|(export, _)| export == name)
            .map(|(_, val)| val)
    }

    pub fn get_export_func(&self, name: &str) -> Result<Rc<ComponentFunc>> {
        match self.export(name) {
            Some(ExternVal::Func(func)) => Ok(Rc::clone(func)),
            _ => Err(RuntimeError::ExportFuncNotFound),
        }
    }
}

#[derive(Clone)]
enum CoreInstanceInst {
    Module(Rc<ModuleInst>),
    Exports(FxHashMap<String, Externval>),
}

impl CoreInstanceInst {
    fn export(&self, name: &str) -> Option<Externval> {
        match self {
            CoreInstanceInst::Module(inst) => inst
                .exports
                .iter()
                .find(|export| export.name == name)
                .map(|export| export.value.clone()),
            CoreInstanceInst::Exports(exports) => exports.get(name).cloned(),
        }
    }

    fn exports(&self) -> FxHashMap<String, Externval> {
        match self {
            CoreInstanceInst::Module(inst) => inst
                .exports
                .iter()
                .map(|export| (export.name.clone(), export.value.clone()))
                .collect(),
            CoreInstanceInst::Exports(exports) => exports.clone(),
        }
    }
}

fn wrong_sort(what: &str, expected: Sort, got: Sort) -> RuntimeError {
    error(format!("{what} is a {got:?}, not a {expected:?}"))
}

/// Resolves a component's index spaces into runtime values.
struct Linker<'a> {
    component: &'a Component,
    imports: Vec<(String, ExternVal)>,
    outer: Vec<Rc<Scope>>,
    core_modules: RefCell<Vec<Option<Shared<Module>>>>,
    core_instances: RefCell<Vec<Option<CoreInstanceInst>>>,
    core_funcs: RefCell<Vec<Option<FuncAddr>>>,
    core_tables: RefCell<Vec<Option<TableAddr>>>,
    core_memories: RefCell<Vec<Option<MemAddr>>>,
    core_globals: RefCell<Vec<Option<GlobalAddr>>>,
    components: RefCell<Vec<Option<Rc<ComponentClosure>>>>,
    instances: RefCell<Vec<Option<Rc<InstanceInst>>>>,
    funcs: RefCell<Vec<Option<Rc<ComponentFunc>>>>,
    values: RefCell<Vec<Option<ComponentVal>>>,
    started: Cell<bool>,
    module_insts: RefCell<Vec<Rc<ModuleInst>>>,
}

fn table<T: Clone>(len: usize) -> RefCell<Vec<Option<T>>> {
    RefCell::new(vec![None; len])
}

/// Returns the memoized entry at `index` of `table`, or computes and stores it.
fn memo<T: Clone>(
    table: &RefCell<Vec<Option<T>>>,
    index: u32,
    compute: impl FnOnce() -> Result<T>,
) -> Result<T> {
    if let Some(val) = &table.borrow()[index as usize] {
        return Ok(val.clone());
    }
    let val = compute()?;
    table.borrow_mut()[index as usize] = Some(val.clone());
    Ok(val)
}

impl<'a> Linker<'a> {
    fn new(
        component: &'a Component,
        imports: Vec<(String, ExternVal)>,
        outer: Vec<Rc<Scope>>,
    ) -> Self {
        Linker {
            component,
            imports,
            outer,
            core_modules: table(component.core_modules.len()),
            core_instances: table(component.core_instances.len()),
            core_funcs: table(component.core_funcs.len()),
            core_tables: table(component.core_tables.len()),
            core_memories: table(component.core_memories.len()),
            core_globals: table(component.core_globals.len()),
            components: table(component.components.len()),
            instances: table(component.instances.len()),
            funcs: table(component.funcs.len()),
            values: table(component.values.len()),
            started: Cell::new(false),
            module_insts: RefCell::new(Vec::new()),
        }
    }

    fn instantiate(self) -> Result<Rc<ComponentInst>> {
        let component = self.component;
        for definition in &component.definitions {
            match *definition {
                Definition::CoreInstance(i) => {
                    self.core_instance(i)?;
                }
                Definition::Instance(i) => {
                    self.instance(i)?;
                }
                Definition::Start => self.run_start()?,
            }
        }
        let exports = component
            .exports
            .iter()
            .map(|export| {
                Ok((
                    export.name.clone(),
                    self.extern_val(export.kind, export.index)?,
                ))
            })
            .collect::<Result<_>>()?;
        Ok(Rc::new(ComponentInst {
            exports,
            _core_instances: self.module_insts.into_inner(),
            _instances: self.instances.into_inner().into_iter().flatten().collect(),
        }))
    }

    /// The captured scope `count` levels out.
    fn outer(&self, count: u32) -> Result<&Scope> {
        self.outer
            .len()
            .checked_sub(count as usize)
            .and_then(|i| self.outer.get(i))
            .map(|scope| &**scope)
            .ok_or_else(|| error("outer alias past the outermost component"))
    }

    /// Captures what `definition`'s subtree takes from this component through outer aliases.
    fn capture(&self, definition: &Component, depth: u32, scope: &mut Scope) -> Result<()> {
        for entry in &definition.core_modules {
            if let CoreModuleOrigin::Outer { count, index } = entry.origin {
                if count == depth {
                    scope.core_modules.insert(index, self.core_module(index)?);
                }
            }
        }
        for entry in &definition.components {
            match &entry.origin {
                ComponentOrigin::Defined(inner) => self.capture(inner, depth + 1, scope)?,
                ComponentOrigin::Outer { count, index } if *count == depth => {
                    scope.components.insert(*index, self.component(*index)?);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn extern_val(&self, kind: Sort, index: u32) -> Result<ExternVal> {
        Ok(match kind {
            Sort::Module => ExternVal::Module(self.core_module(index)?),
            Sort::Func => ExternVal::Func(self.func(index)?),
            Sort::Value => ExternVal::Value(self.value(index)?),
            Sort::Type => ExternVal::Type,
            Sort::Instance => ExternVal::Instance(self.instance(index)?),
            Sort::Component => ExternVal::Component(self.component(index)?),
        })
    }

    fn import(&self, index: u32) -> Result<ExternVal> {
        let name = &self.component.imports[index as usize].name;
        self.imports
            .iter()
            .find(|(import, _)| import == name)
            .map(|(_, val)| val.clone())
            .ok_or_else(|| error(format!("missing import {name}")))
    }

    fn alias(&self, alias: &Alias) -> Result<ExternVal> {
        self.instance(alias.instance)?
            .export(&alias.name)
            .cloned()
            .ok_or_else(|| error(format!("instance has no export {}", alias.name)))
    }

    fn core_alias(&self, alias: &CoreAlias) -> Result<Externval> {
        self.core_instance(alias.instance)?
            .export(&alias.name)
            .ok_or_else(|| error(format!("core instance has no export {}", alias.name)))
    }

    fn core_module(&self, index: u32) -> Result<Shared<Module>> {
        memo(&self.core_modules, index, || {
            let val = match &self.component.core_modules[index as usize].origin {
                CoreModuleOrigin::Defined(module) => return Ok(Shared::clone(module)),
                CoreModuleOrigin::Import(j) => self.import(*j)?,
                CoreModuleOrigin::Alias(alias) => self.alias(alias)?,
                CoreModuleOrigin::Outer { count, index } => {
                    return self
                        .outer(*count)?
                        .core_modules
                        .get(index)
                        .cloned()
                        .ok_or_else(|| error("outer core module was not captured"))
                }
            };
            match val {
                ExternVal::Module(module) => Ok(module),
                other => Err(wrong_sort("core module", Sort::Module, other.sort())),
            }
        })
    }

    fn core_instance(&self, index: u32) -> Result<CoreInstanceInst> {
        memo(&self.core_instances, index, || {
            Ok(match &self.component.core_instances[index as usize] {
                CoreInstance::Instantiate { module, args } => {
                    let module = self.core_module(*module)?;
                    let mut objects = ImportObjects::default();
                    for (name, instance) in args {
                        objects.insert(name.clone(), self.core_instance(*instance)?.exports());
                    }
                    let inst = ModuleInst::new(&module, objects, Vec::new())?;
                    run_start_section(&inst)?;
                    self.module_insts.borrow_mut().push(Rc::clone(&inst));
                    CoreInstanceInst::Module(inst)
                }
                CoreInstance::FromExports(exports) => {
                    let mut map = FxHashMap::default();
                    for export in exports {
                        let val = match export.kind {
                            CoreSort::Func => Externval::Func(self.core_func(export.index)?),
                            CoreSort::Table => Externval::Table(self.core_table(export.index)?),
                            CoreSort::Memory => Externval::Mem(self.core_memory(export.index)?),
                            CoreSort::Global => Externval::Global(self.core_global(export.index)?),
                        };
                        map.insert(export.name.clone(), val);
                    }
                    CoreInstanceInst::Exports(map)
                }
            })
        })
    }

    fn core_func(&self, index: u32) -> Result<FuncAddr> {
        memo(&self.core_funcs, index, || {
            Ok(match &self.component.core_funcs[index as usize] {
                CoreFunc::Alias(alias) => self
                    .core_alias(alias)?
                    .as_func()
                    .ok_or_else(|| error("core alias is not a function"))?,
                CoreFunc::Lower { func, .. } => lower_func(self.func(*func)?)?,
                CoreFunc::ResourceNew(_) | CoreFunc::ResourceDrop(_) | CoreFunc::ResourceRep(_) => {
                    return Err(unsupported("resources"))
                }
            })
        })
    }

    fn core_table(&self, index: u32) -> Result<TableAddr> {
        memo(&self.core_tables, index, || {
            self.core_alias(&self.component.core_tables[index as usize])?
                .as_table()
                .ok_or_else(|| error("core alias is not a table"))
        })
    }

    fn core_memory(&self, index: u32) -> Result<MemAddr> {
        memo(&self.core_memories, index, || {
            self.core_alias(&self.component.core_memories[index as usize])?
                .as_mem()
                .ok_or_else(|| error("core alias is not a memory"))
        })
    }

    fn core_global(&self, index: u32) -> Result<GlobalAddr> {
        memo(&self.core_globals, index, || {
            self.core_alias(&self.component.core_globals[index as usize])?
                .as_global()
                .ok_or_else(|| error("core alias is not a global"))
        })
    }

    fn component(&self, index: u32) -> Result<Rc<ComponentClosure>> {
        memo(&self.components, index, || {
            let val = match &self.component.components[index as usize].origin {
                ComponentOrigin::Defined(definition) => {
                    let mut scope = Scope::default();
                    self.capture(definition, 1, &mut scope)?;
                    let mut outer = self.outer.clone();
                    outer.push(Rc::new(scope));
                    return Ok(Rc::new(ComponentClosure {
                        definition: Shared::clone(definition),
                        outer,
                    }));
                }
                ComponentOrigin::Import(j) => self.import(*j)?,
                ComponentOrigin::Alias(alias) => self.alias(alias)?,
                ComponentOrigin::Outer { count, index } => {
                    return self
                        .outer(*count)?
                        .components
                        .get(index)
                        .cloned()
                        .ok_or_else(|| error("outer component was not captured"))
                }
            };
            match val {
                ExternVal::Component(component) => Ok(component),
                other => Err(wrong_sort("component", Sort::Component, other.sort())),
            }
        })
    }

    fn instance(&self, index: u32) -> Result<Rc<InstanceInst>> {
        memo(&self.instances, index, || {
            Ok(match &self.component.instances[index as usize].origin {
                InstanceOrigin::Instantiate { component, args } => {
                    let closure = self.component(*component)?;
                    let mut imports = Vec::with_capacity(args.len());
                    for arg in args {
                        imports.push((arg.name.clone(), self.extern_val(arg.kind, arg.index)?));
                    }
                    let inner = Linker::new(&closure.definition, imports, closure.outer.clone())
                        .instantiate()?;
                    Rc::new(InstanceInst {
                        exports: inner.exports.clone(),
                        _inner: Some(inner),
                    })
                }
                InstanceOrigin::FromExports(exports) => {
                    let mut list = Vec::with_capacity(exports.len());
                    for export in exports {
                        list.push((
                            export.name.clone(),
                            self.extern_val(export.kind, export.index)?,
                        ));
                    }
                    Rc::new(InstanceInst {
                        exports: list,
                        _inner: None,
                    })
                }
                InstanceOrigin::Import(j) => match self.import(*j)? {
                    ExternVal::Instance(inst) => inst,
                    other => return Err(wrong_sort("import", Sort::Instance, other.sort())),
                },
                InstanceOrigin::Alias(alias) => match self.alias(alias)? {
                    ExternVal::Instance(inst) => inst,
                    other => return Err(wrong_sort("alias", Sort::Instance, other.sort())),
                },
            })
        })
    }

    fn func(&self, index: u32) -> Result<Rc<ComponentFunc>> {
        memo(&self.funcs, index, || {
            let entry = &self.component.funcs[index as usize];
            Ok(match &entry.origin {
                FuncOrigin::Lift { core_func, options } => {
                    let core = self.core_func(*core_func)?;
                    let ty = Shared::clone(&entry.ty);
                    if !core.func_type().type_match(&flat_func_type(&ty)?) {
                        return Err(error("lifted core function has the wrong type"));
                    }
                    let post_return = options
                        .post_return
                        .map(|index| self.core_func(index))
                        .transpose()?;
                    Rc::new(ComponentFunc {
                        ty,
                        core,
                        post_return,
                    })
                }
                FuncOrigin::Import(j) => match self.import(*j)? {
                    ExternVal::Func(func) => func,
                    other => return Err(wrong_sort("import", Sort::Func, other.sort())),
                },
                FuncOrigin::Alias(alias) => match self.alias(alias)? {
                    ExternVal::Func(func) => func,
                    other => return Err(wrong_sort("alias", Sort::Func, other.sort())),
                },
            })
        })
    }

    fn value(&self, index: u32) -> Result<ComponentVal> {
        memo(&self.values, index, || {
            Ok(match &self.component.values[index as usize].origin {
                ValueOrigin::Import(j) => match self.import(*j)? {
                    ExternVal::Value(val) => val,
                    other => return Err(wrong_sort("import", Sort::Value, other.sort())),
                },
                ValueOrigin::Alias(alias) => match self.alias(alias)? {
                    ExternVal::Value(val) => val,
                    other => return Err(wrong_sort("alias", Sort::Value, other.sort())),
                },
                ValueOrigin::StartResult => {
                    self.run_start()?;
                    return self.values.borrow()[index as usize]
                        .clone()
                        .ok_or_else(|| error("the start function did not produce the value"));
                }
            })
        })
    }

    /// Runs the start function once, filling the values it produces.
    fn run_start(&self) -> Result<()> {
        if self.started.replace(true) {
            return Ok(());
        }
        let Some(start) = &self.component.start else {
            return Ok(());
        };
        let func = self.func(start.func)?;
        let args = start
            .args
            .iter()
            .map(|arg| self.value(*arg))
            .collect::<Result<Vec<_>>>()?;
        let results = func.call(args)?;
        let slots: Vec<usize> = self
            .component
            .values
            .iter()
            .enumerate()
            .filter(|(_, value)| matches!(value.origin, ValueOrigin::StartResult))
            .map(|(i, _)| i)
            .collect();
        if slots.len() != results.len() {
            return Err(error(
                "the start function returned the wrong number of results",
            ));
        }
        let mut values = self.values.borrow_mut();
        for (slot, result) in slots.into_iter().zip(results) {
            values[slot] = Some(result);
        }
        Ok(())
    }
}
