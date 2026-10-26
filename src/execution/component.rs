//! Component instances: instantiation, and calls that cross the canonical ABI.
//!
//! Entries refer across index spaces, so the linker follows references instead of filling one space at a time:
//! each resolver builds its entry by calling the resolvers of what the entry refers to.
//! Each entry must become one object, so every resolver goes through a memo table and a second request returns the first result.
//! The definitions with side effects are forced in the order they appear in the binary,
//! so instantiations and the start function run in that order even when nothing refers to them.
//! See `doc/component.md`.

use super::canon::{
    call_core, check_encoding, flatten_functype, lift_values, lower_values, CanonContext, Context,
    LeaveGuard, Reader, MAX_FLAT_PARAMS, MAX_FLAT_RESULTS,
};
use super::func::FuncAddr;
use super::global::GlobalAddr;
use super::mem::MemAddr;
use super::module::{ImportObjects, ModuleInst};
use super::runtime::run_start_section;
use super::table::TableAddr;
use super::value::{Externval, Val};
use crate::error::RuntimeError;
use crate::shared::Shared;
use crate::structure::component::*;
use crate::structure::module::Module;
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

pub use super::canon::ComponentVal;

/// A component-level function: a core function lifted to component values.
pub struct ComponentFunc {
    pub ty: Shared<FuncType>,
    core: FuncAddr,
    post_return: Option<FuncAddr>,
    cx: CanonContext,
}

impl ComponentFunc {
    /// Calls the function with component values and returns its results as component values.
    pub fn call(&self, args: Vec<ComponentVal>) -> Result<Vec<ComponentVal>> {
        let param_types: Vec<ValType> = self.ty.params.iter().map(|(_, t)| t.clone()).collect();
        let result_types: Vec<ValType> = self.ty.result.iter().cloned().collect();
        let params = lower_values(&self.cx, &args, &param_types, MAX_FLAT_PARAMS, None)?;
        let results = call_core(&self.core, params)?;
        let lifted = lift_values(
            &self.cx,
            &mut Reader::new(&results),
            &result_types,
            MAX_FLAT_RESULTS,
        )?;
        if let Some(post_return) = &self.post_return {
            let _guard = LeaveGuard::new(&self.cx.may_leave);
            call_core(post_return, results)?;
        }
        Ok(lifted)
    }
}

/// `canon lower`: a core function that lifts its arguments, calls `func`, and lowers the result.
/// `cx` holds the options of the lower, in the importing instance.
fn lower_func(func: Rc<ComponentFunc>, cx: CanonContext) -> Result<FuncAddr> {
    let core_ty = flatten_functype(&func.ty, Context::Lower)?;
    let ty = Shared::clone(&func.ty);
    Ok(FuncAddr::alloc_host(
        core_ty,
        Rc::new(move |params: Vec<Val>| {
            if !cx.may_leave.get() {
                return Err(error("cannot leave component instance"));
            }
            let param_types: Vec<ValType> = ty.params.iter().map(|(_, t)| t.clone()).collect();
            let result_types: Vec<ValType> = ty.result.iter().cloned().collect();
            let mut reader = Reader::new(&params);
            let args = lift_values(&cx, &mut reader, &param_types, MAX_FLAT_PARAMS)?;
            let results = func.call(args)?;
            let lowered = lower_values(
                &cx,
                &results,
                &result_types,
                MAX_FLAT_RESULTS,
                Some(&mut reader),
            )?;
            Ok(lowered.into_iter().next())
        }),
    ))
}

/// A component-level extern
#[derive(Clone)]
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
    may_leave: Rc<Cell<bool>>,
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
            may_leave: Rc::new(Cell::new(true)),
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

    /// The resolved options of a lift or lower defined in this component.
    fn context(&self, options: &CanonOptions) -> Result<CanonContext> {
        Ok(CanonContext {
            memory: options.memory.map(|i| self.core_memory(i)).transpose()?,
            realloc: options.realloc.map(|i| self.core_func(i)).transpose()?,
            encoding: options.string_encoding,
            may_leave: Rc::clone(&self.may_leave),
        })
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
                CoreFunc::Lower { func, options } => {
                    let cx = self.context(options)?;
                    lower_func(self.func(*func)?, cx)?
                }
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
                    let cx = self.context(options)?;
                    check_encoding(&cx, &ty)?;
                    if !core
                        .func_type()
                        .type_match(&flatten_functype(&ty, Context::Lift)?)
                    {
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
                        cx,
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
