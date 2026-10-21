//! Component binary parser.
//!
//! Walks wasmparser's payload stream with a stack of builders:
//! one for each component being read, and one for each core module nested inside it.
//! Nested modules go through the core section decoders.
//! Every index space is resolved as it is read, so the finished [`Component`] holds no aliases,
//! and types come out as trees that carry no indices.

use super::{
    decode_module_payload, func_type_of_rec_group, global_type_of, match_value_type, mem_type_of,
    table_type_of,
};
use crate::error::ParserError;
use crate::shared::Shared;
use crate::structure::component::*;
use crate::structure::module::{ExportDesc, ImportDesc, Module};
use crate::structure::types::FuncType as CoreFuncType;
use wasmparser::{
    CanonicalFunction, CanonicalOption, ComponentAlias, ComponentDefinedType,
    ComponentExternalKind, ComponentOuterAliasKind, ComponentTypeDeclaration, ComponentTypeRef,
    ComponentValType, Encoding, ExternalKind, InstanceTypeDeclaration, ModuleTypeDeclaration,
    OuterAliasKind, Parser, Payload, PrimitiveValType, TypeBounds, TypeRef,
};

type ParseResult<T> = Result<T, Box<dyn std::error::Error>>;

fn invalid(message: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(ParserError::InvalidComponent(message.into()))
}

fn unsupported(feature: &'static str) -> Box<dyn std::error::Error> {
    Box::new(ParserError::UnsupportedComponentFeature(feature))
}

/// Parses a component binary file.
pub fn parse_component(path: &str) -> ParseResult<Shared<Component>> {
    parse_component_bytes(&std::fs::read(path)?)
}

/// Parses a component binary.
pub fn parse_component_bytes(buf: &[u8]) -> ParseResult<Shared<Component>> {
    let mut stack: Vec<Frame> = Vec::new();
    for payload in Parser::new(0).parse_all(buf) {
        let payload = payload?;
        match payload {
            Payload::Version { encoding, .. } if stack.is_empty() => {
                if encoding != Encoding::Component {
                    return Err(Box::new(ParserError::NotAComponent));
                }
                stack.push(Frame::Component(Component::new()));
            }
            Payload::ModuleSection { .. } => stack.push(Frame::Module(ModuleFrame::new())),
            Payload::ComponentSection { .. } => stack.push(Frame::Component(Component::new())),
            Payload::End(_) => {
                let finished = stack.pop().ok_or_else(|| invalid("unbalanced end"))?;
                let Some(parent) = stack.last_mut() else {
                    let Frame::Component(component) = finished else {
                        return Err(invalid("a module cannot be the outermost definition"));
                    };
                    return Ok(Shared::new(component));
                };
                let Frame::Component(parent) = parent else {
                    return Err(invalid("definition nested inside a core module"));
                };
                match finished {
                    Frame::Module(frame) => parent.core_modules.push(CoreModule {
                        ty: Shared::new(module_type_of(&frame.module)?),
                        origin: CoreModuleOrigin::Defined(Shared::new(frame.module)),
                    }),
                    Frame::Component(component) => parent.components.push(ComponentDef {
                        ty: Shared::new(component_type_of(&component)),
                        origin: ComponentOrigin::Defined(Shared::new(component)),
                    }),
                }
            }
            payload => {
                let split = stack.len().saturating_sub(1);
                let (outer, top) = stack.split_at_mut(split);
                match top.first_mut() {
                    Some(Frame::Module(frame)) => frame.decode(payload)?,
                    Some(Frame::Component(component)) => {
                        let outer: Vec<&Component> = outer
                            .iter()
                            .filter_map(|frame| match frame {
                                Frame::Component(c) => Some(c),
                                Frame::Module(_) => None,
                            })
                            .collect();
                        decode_component_payload(payload, &outer, component)?
                    }
                    None => return Err(invalid("section before the component header")),
                }
            }
        }
    }
    Err(invalid("component ended without an end marker"))
}

enum Frame {
    Component(Component),
    Module(ModuleFrame),
}

/// A nested core module under construction.
struct ModuleFrame {
    module: Module,
    next_func: usize,
}

impl ModuleFrame {
    fn new() -> Self {
        ModuleFrame {
            module: Module::new("nested"),
            next_func: 0,
        }
    }

    fn decode(&mut self, payload: Payload<'_>) -> ParseResult<()> {
        decode_module_payload(
            payload,
            &mut self.module,
            &mut self.next_func,
            #[cfg(feature = "call_graph")]
            None,
        )
    }
}

/// Decodes one payload of the component on top of the stack.
/// `outer` holds the enclosing components, outermost first, for outer aliases.
fn decode_component_payload(
    payload: Payload<'_>,
    outer: &[&Component],
    comp: &mut Component,
) -> ParseResult<()> {
    match payload {
        Payload::Version { encoding, .. } => {
            if encoding != Encoding::Component {
                return Err(invalid("nested component with a module header"));
            }
        }
        Payload::CoreTypeSection(reader) => {
            for ty in reader {
                let ty = core_type(&chain(outer, comp), ty?)?;
                comp.core_types.push(ty);
            }
        }
        Payload::ComponentTypeSection(reader) => {
            for ty in reader {
                let ty = component_type(&chain(outer, comp), &ty?)?;
                comp.types.push(ty);
            }
        }
        Payload::ComponentImportSection(reader) => {
            for import in reader {
                let import = import?;
                let desc = extern_desc(&chain(outer, comp), &import.ty)?;
                let index = comp.imports.len() as u32;
                comp.imports.push(Import {
                    name: import.name.name.to_string(),
                    desc: desc.clone(),
                });
                add_to_space(comp, desc, Origin::Import(index));
            }
        }
        Payload::ComponentExportSection(reader) => {
            for export in reader {
                let export = export?;
                let kind = sort_of(export.kind);
                let desc = match &export.ty {
                    Some(ty) => extern_desc(&chain(outer, comp), ty)?,
                    None => desc_of(comp, kind, export.index)?,
                };
                if desc.sort() != kind {
                    return Err(invalid("export ascription of another sort"));
                }
                let origin = origin_of(comp, kind, export.index)?;
                comp.exports.push(Export {
                    name: export.name.name.to_string(),
                    kind,
                    index: export.index,
                    desc: desc.clone(),
                });
                add_to_space(comp, desc, origin);
            }
        }
        Payload::ComponentCanonicalSection(reader) => {
            for func in reader {
                match func? {
                    CanonicalFunction::Lift {
                        core_func_index,
                        type_index,
                        options,
                    } => {
                        let Type::Func(ty) = type_at(comp, type_index)? else {
                            return Err(invalid("canon lift with a non-function type"));
                        };
                        check_index(comp.core_funcs.len(), core_func_index, "core function")?;
                        let options = canon_options(comp, &options)?;
                        comp.funcs.push(Func {
                            ty: Shared::clone(ty),
                            origin: FuncOrigin::Lift {
                                core_func: core_func_index,
                                options,
                            },
                        });
                    }
                    CanonicalFunction::Lower {
                        func_index,
                        options,
                    } => {
                        check_index(comp.funcs.len(), func_index, "function")?;
                        let options = canon_options(comp, &options)?;
                        comp.core_funcs.push(CoreFunc::Lower {
                            func: func_index,
                            options,
                        });
                    }
                    CanonicalFunction::ResourceNew { resource } => {
                        resource_at(comp, resource)?;
                        comp.core_funcs.push(CoreFunc::ResourceNew(resource));
                    }
                    CanonicalFunction::ResourceDrop { resource } => {
                        resource_at(comp, resource)?;
                        comp.core_funcs.push(CoreFunc::ResourceDrop(resource));
                    }
                    CanonicalFunction::ResourceRep { resource } => {
                        resource_at(comp, resource)?;
                        comp.core_funcs.push(CoreFunc::ResourceRep(resource));
                    }
                    _ => return Err(unsupported("async and threading canonical functions")),
                }
            }
        }
        Payload::InstanceSection(reader) => {
            for instance in reader {
                let instance = match instance? {
                    wasmparser::Instance::Instantiate { module_index, args } => {
                        check_index(comp.core_modules.len(), module_index, "core module")?;
                        let args = args
                            .iter()
                            .map(|arg| {
                                check_index(comp.core_instances.len(), arg.index, "core instance")?;
                                Ok((arg.name.to_string(), arg.index))
                            })
                            .collect::<ParseResult<_>>()?;
                        CoreInstance::Instantiate {
                            module: module_index,
                            args,
                        }
                    }
                    wasmparser::Instance::FromExports(exports) => CoreInstance::FromExports(
                        exports
                            .iter()
                            .map(|export| {
                                let kind = core_sort_of(export.kind)?;
                                check_core_index(comp, kind, export.index)?;
                                Ok(CoreInstanceExport {
                                    name: export.name.to_string(),
                                    kind,
                                    index: export.index,
                                })
                            })
                            .collect::<ParseResult<_>>()?,
                    ),
                };
                comp.core_instances.push(instance);
            }
        }
        Payload::ComponentInstanceSection(reader) => {
            for instance in reader {
                let instance = match instance? {
                    wasmparser::ComponentInstance::Instantiate {
                        component_index,
                        args,
                    } => {
                        let component = comp
                            .components
                            .get(component_index as usize)
                            .ok_or_else(|| invalid("unknown component index"))?;
                        let ty = Shared::new(InstanceType {
                            exports: component.ty.exports.clone(),
                        });
                        let mut resolved = Vec::with_capacity(args.len());
                        for arg in args.iter() {
                            let kind = sort_of(arg.kind);
                            desc_of(comp, kind, arg.index)?;
                            resolved.push(InstantiationArg {
                                name: arg.name.to_string(),
                                kind,
                                index: arg.index,
                            });
                        }
                        let args = resolved;
                        Instance {
                            ty,
                            origin: InstanceOrigin::Instantiate {
                                component: component_index,
                                args,
                            },
                        }
                    }
                    wasmparser::ComponentInstance::FromExports(exports) => {
                        let mut ty = InstanceType::default();
                        let mut list = Vec::new();
                        for export in exports.iter() {
                            let kind = sort_of(export.kind);
                            ty.exports.push((
                                export.name.name.to_string(),
                                desc_of(comp, kind, export.index)?,
                            ));
                            list.push(InstanceExport {
                                name: export.name.name.to_string(),
                                kind,
                                index: export.index,
                            });
                        }
                        Instance {
                            ty: Shared::new(ty),
                            origin: InstanceOrigin::FromExports(list),
                        }
                    }
                };
                comp.instances.push(instance);
            }
        }
        Payload::ComponentAliasSection(reader) => {
            for alias in reader {
                match alias? {
                    ComponentAlias::InstanceExport {
                        kind,
                        instance_index,
                        name,
                    } => {
                        let desc = instance_export(comp, instance_index, name)?;
                        if desc.sort() != sort_of(kind) {
                            return Err(invalid(format!("alias of {name} with the wrong sort")));
                        }
                        let alias = Alias {
                            instance: instance_index,
                            name: name.to_string(),
                        };
                        add_to_space(comp, desc, Origin::Alias(alias));
                    }
                    ComponentAlias::CoreInstanceExport {
                        kind,
                        instance_index,
                        name,
                    } => {
                        check_index(comp.core_instances.len(), instance_index, "core instance")?;
                        let alias = CoreAlias {
                            instance: instance_index,
                            name: name.to_string(),
                        };
                        match core_sort_of(kind)? {
                            CoreSort::Func => comp.core_funcs.push(CoreFunc::Alias(alias)),
                            CoreSort::Table => comp.core_tables.push(alias),
                            CoreSort::Memory => comp.core_memories.push(alias),
                            CoreSort::Global => comp.core_globals.push(alias),
                        }
                    }
                    ComponentAlias::Outer { kind, count, index } => {
                        let target = outer_component(outer, comp, count)?;
                        match kind {
                            ComponentOuterAliasKind::CoreModule => {
                                let entry = target
                                    .core_modules
                                    .get(index as usize)
                                    .ok_or_else(|| invalid("unknown outer core module"))?
                                    .clone();
                                comp.core_modules.push(entry);
                            }
                            ComponentOuterAliasKind::CoreType => {
                                let entry = target
                                    .core_types
                                    .get(index as usize)
                                    .ok_or_else(|| invalid("unknown outer core type"))?
                                    .clone();
                                comp.core_types.push(entry);
                            }
                            ComponentOuterAliasKind::Type => {
                                let entry = target
                                    .types
                                    .get(index as usize)
                                    .ok_or_else(|| invalid("unknown outer type"))?
                                    .clone();
                                comp.types.push(entry);
                            }
                            ComponentOuterAliasKind::Component => {
                                let entry = target
                                    .components
                                    .get(index as usize)
                                    .ok_or_else(|| invalid("unknown outer component"))?
                                    .clone();
                                comp.components.push(entry);
                            }
                        }
                    }
                }
            }
        }
        Payload::ComponentStartSection { start, .. } => {
            let func = comp
                .funcs
                .get(start.func_index as usize)
                .ok_or_else(|| invalid("unknown start function"))?;
            for arg in start.arguments.iter() {
                check_index(comp.values.len(), *arg, "value")?;
            }
            let results: Vec<Value> = match (start.results, &func.ty.result) {
                (0, None) => Vec::new(),
                (1, Some(ty)) => vec![Value {
                    ty: ty.clone(),
                    origin: ValueOrigin::StartResult,
                }],
                _ => return Err(invalid("start result count does not match the function")),
            };
            comp.values.extend(results);
            comp.start = Some(Start {
                func: start.func_index,
                args: start.arguments.to_vec(),
                results: start.results,
            });
        }
        Payload::CustomSection(_) | Payload::UnknownSection { .. } => {}
        _ => return Err(invalid("core section outside a core module")),
    }
    Ok(())
}

/// Where an item added to an index space came from.
enum Origin {
    Import(u32),
    Alias(Alias),
    /// The same item already at another index, re-added by an export.
    Existing(ExistingOrigin),
}

enum ExistingOrigin {
    Module(CoreModuleOrigin),
    Func(FuncOrigin),
    Value(ValueOrigin),
    Instance(InstanceOrigin),
    Component(ComponentOrigin),
    Type,
}

/// Adds an item of `desc`'s sort to its index space.
fn add_to_space(comp: &mut Component, desc: ExternDesc, origin: Origin) {
    match desc {
        ExternDesc::Module(ty) => {
            let origin = match origin {
                Origin::Import(i) => CoreModuleOrigin::Import(i),
                Origin::Alias(a) => CoreModuleOrigin::Alias(a),
                Origin::Existing(ExistingOrigin::Module(o)) => o,
                Origin::Existing(_) => unreachable!("origin of another sort"),
            };
            comp.core_modules.push(CoreModule { ty, origin });
        }
        ExternDesc::Func(ty) => {
            let origin = match origin {
                Origin::Import(i) => FuncOrigin::Import(i),
                Origin::Alias(a) => FuncOrigin::Alias(a),
                Origin::Existing(ExistingOrigin::Func(o)) => o,
                Origin::Existing(_) => unreachable!("origin of another sort"),
            };
            comp.funcs.push(Func { ty, origin });
        }
        ExternDesc::Value(ty) => {
            let origin = match origin {
                Origin::Import(i) => ValueOrigin::Import(i),
                Origin::Alias(a) => ValueOrigin::Alias(a),
                Origin::Existing(ExistingOrigin::Value(o)) => o,
                Origin::Existing(_) => unreachable!("origin of another sort"),
            };
            comp.values.push(Value { ty, origin });
        }
        ExternDesc::Type(bound) => comp.types.push(bound.to_type()),
        ExternDesc::Instance(ty) => {
            let origin = match origin {
                Origin::Import(i) => InstanceOrigin::Import(i),
                Origin::Alias(a) => InstanceOrigin::Alias(a),
                Origin::Existing(ExistingOrigin::Instance(o)) => o,
                Origin::Existing(_) => unreachable!("origin of another sort"),
            };
            comp.instances.push(Instance { ty, origin });
        }
        ExternDesc::Component(ty) => {
            let origin = match origin {
                Origin::Import(i) => ComponentOrigin::Import(i),
                Origin::Alias(a) => ComponentOrigin::Alias(a),
                Origin::Existing(ExistingOrigin::Component(o)) => o,
                Origin::Existing(_) => unreachable!("origin of another sort"),
            };
            comp.components.push(ComponentDef { ty, origin });
        }
    }
}

/// The type of the item at `index` in the index space of `kind`.
fn desc_of(comp: &Component, kind: Sort, index: u32) -> ParseResult<ExternDesc> {
    let i = index as usize;
    Ok(match kind {
        Sort::Module => ExternDesc::Module(Shared::clone(
            &comp
                .core_modules
                .get(i)
                .ok_or_else(|| invalid("unknown core module index"))?
                .ty,
        )),
        Sort::Func => ExternDesc::Func(Shared::clone(
            &comp
                .funcs
                .get(i)
                .ok_or_else(|| invalid("unknown function index"))?
                .ty,
        )),
        Sort::Value => ExternDesc::Value(
            comp.values
                .get(i)
                .ok_or_else(|| invalid("unknown value index"))?
                .ty
                .clone(),
        ),
        Sort::Type => ExternDesc::Type(TypeBound::Eq(type_at(comp, index)?.clone())),
        Sort::Instance => ExternDesc::Instance(Shared::clone(
            &comp
                .instances
                .get(i)
                .ok_or_else(|| invalid("unknown instance index"))?
                .ty,
        )),
        Sort::Component => ExternDesc::Component(Shared::clone(
            &comp
                .components
                .get(i)
                .ok_or_else(|| invalid("unknown component index"))?
                .ty,
        )),
    })
}

/// The origin of the item at `index` in the index space of `kind`, for an export that re-adds it.
fn origin_of(comp: &Component, kind: Sort, index: u32) -> ParseResult<Origin> {
    let i = index as usize;
    Ok(Origin::Existing(match kind {
        Sort::Module => ExistingOrigin::Module(
            comp.core_modules
                .get(i)
                .ok_or_else(|| invalid("unknown core module index"))?
                .origin
                .clone(),
        ),
        Sort::Func => ExistingOrigin::Func(
            comp.funcs
                .get(i)
                .ok_or_else(|| invalid("unknown function index"))?
                .origin
                .clone(),
        ),
        Sort::Value => ExistingOrigin::Value(
            comp.values
                .get(i)
                .ok_or_else(|| invalid("unknown value index"))?
                .origin
                .clone(),
        ),
        Sort::Type => ExistingOrigin::Type,
        Sort::Instance => ExistingOrigin::Instance(
            comp.instances
                .get(i)
                .ok_or_else(|| invalid("unknown instance index"))?
                .origin
                .clone(),
        ),
        Sort::Component => ExistingOrigin::Component(
            comp.components
                .get(i)
                .ok_or_else(|| invalid("unknown component index"))?
                .origin
                .clone(),
        ),
    }))
}

fn instance_export(comp: &Component, instance: u32, name: &str) -> ParseResult<ExternDesc> {
    let ty = &comp
        .instances
        .get(instance as usize)
        .ok_or_else(|| invalid("unknown instance index"))?
        .ty;
    export_of(ty, name)
}

fn export_of(ty: &InstanceType, name: &str) -> ParseResult<ExternDesc> {
    ty.exports
        .iter()
        .find(|(export, _)| export == name)
        .map(|(_, desc)| desc.clone())
        .ok_or_else(|| invalid(format!("instance has no export {name}")))
}

fn type_at(comp: &Component, index: u32) -> ParseResult<&Type> {
    comp.types
        .get(index as usize)
        .ok_or_else(|| invalid("unknown type index"))
}

fn resource_at(comp: &Component, index: u32) -> ParseResult<Shared<Resource>> {
    match type_at(comp, index)? {
        Type::Resource(resource) => Ok(Shared::clone(resource)),
        _ => Err(invalid("type index is not a resource")),
    }
}

fn check_index(len: usize, index: u32, what: &str) -> ParseResult<()> {
    if (index as usize) < len {
        Ok(())
    } else {
        Err(invalid(format!("unknown {what} index {index}")))
    }
}

fn check_core_index(comp: &Component, kind: CoreSort, index: u32) -> ParseResult<()> {
    let (len, what) = match kind {
        CoreSort::Func => (comp.core_funcs.len(), "core function"),
        CoreSort::Table => (comp.core_tables.len(), "core table"),
        CoreSort::Memory => (comp.core_memories.len(), "core memory"),
        CoreSort::Global => (comp.core_globals.len(), "core global"),
    };
    check_index(len, index, what)
}

/// The component `count` levels out from the current one.
fn outer_component<'a>(
    outer: &[&'a Component],
    comp: &'a Component,
    count: u32,
) -> ParseResult<&'a Component> {
    match count {
        0 => Ok(comp),
        n => outer
            .len()
            .checked_sub(n as usize)
            .and_then(|i| outer.get(i).copied())
            .ok_or_else(|| invalid("outer alias past the outermost component")),
    }
}

fn sort_of(kind: ComponentExternalKind) -> Sort {
    match kind {
        ComponentExternalKind::Module => Sort::Module,
        ComponentExternalKind::Func => Sort::Func,
        ComponentExternalKind::Value => Sort::Value,
        ComponentExternalKind::Type => Sort::Type,
        ComponentExternalKind::Instance => Sort::Instance,
        ComponentExternalKind::Component => Sort::Component,
    }
}

fn core_sort_of(kind: ExternalKind) -> ParseResult<CoreSort> {
    Ok(match kind {
        ExternalKind::Func => CoreSort::Func,
        ExternalKind::Table => CoreSort::Table,
        ExternalKind::Memory => CoreSort::Memory,
        ExternalKind::Global => CoreSort::Global,
        ExternalKind::Tag => return Err(unsupported("exception tags")),
        ExternalKind::FuncExact => return Err(unsupported("exact function types")),
    })
}

fn canon_options(comp: &Component, options: &[CanonicalOption]) -> ParseResult<CanonOptions> {
    let mut out = CanonOptions::default();
    for option in options {
        match option {
            CanonicalOption::UTF8 => out.string_encoding = StringEncoding::Utf8,
            CanonicalOption::UTF16 => out.string_encoding = StringEncoding::Utf16,
            CanonicalOption::CompactUTF16 => out.string_encoding = StringEncoding::Latin1Utf16,
            CanonicalOption::Memory(i) => {
                check_core_index(comp, CoreSort::Memory, *i)?;
                out.memory = Some(*i);
            }
            CanonicalOption::Realloc(i) => {
                check_core_index(comp, CoreSort::Func, *i)?;
                out.realloc = Some(*i);
            }
            CanonicalOption::PostReturn(i) => {
                check_core_index(comp, CoreSort::Func, *i)?;
                out.post_return = Some(*i);
            }
            CanonicalOption::Async | CanonicalOption::Callback(_) => {
                return Err(unsupported("async canonical options"))
            }
            CanonicalOption::CoreType(_) => return Err(unsupported("core-type canonical option")),
            CanonicalOption::Gc => return Err(unsupported("gc canonical option")),
        }
    }
    Ok(out)
}

// ---- Types ----

/// A scope whose types an alias can refer to: a component, or the declarations of a type.
trait TypeScope {
    fn type_at(&self, index: u32) -> Option<&Type>;
    fn core_type_at(&self, index: u32) -> Option<&CoreType>;
    fn instance_type_at(&self, index: u32) -> Option<&Shared<InstanceType>>;
}

impl TypeScope for Component {
    fn type_at(&self, index: u32) -> Option<&Type> {
        self.types.get(index as usize)
    }
    fn core_type_at(&self, index: u32) -> Option<&CoreType> {
        self.core_types.get(index as usize)
    }
    fn instance_type_at(&self, index: u32) -> Option<&Shared<InstanceType>> {
        self.instances.get(index as usize).map(|i| &i.ty)
    }
}

/// The index spaces of an instance or component type's declarations.
#[derive(Default)]
struct LocalScope {
    types: Vec<Type>,
    core_types: Vec<CoreType>,
    instances: Vec<Shared<InstanceType>>,
}

impl TypeScope for LocalScope {
    fn type_at(&self, index: u32) -> Option<&Type> {
        self.types.get(index as usize)
    }
    fn core_type_at(&self, index: u32) -> Option<&CoreType> {
        self.core_types.get(index as usize)
    }
    fn instance_type_at(&self, index: u32) -> Option<&Shared<InstanceType>> {
        self.instances.get(index as usize)
    }
}

/// Scopes from the outermost component to the innermost scope, which is the last entry.
type Chain<'a> = Vec<&'a dyn TypeScope>;

fn chain<'a>(outer: &[&'a Component], comp: &'a Component) -> Chain<'a> {
    let mut chain: Chain<'a> = outer.iter().map(|c| *c as &dyn TypeScope).collect();
    chain.push(comp);
    chain
}

fn with_local<'a>(chain: &Chain<'a>, local: &'a LocalScope) -> Chain<'a> {
    let mut inner = chain.clone();
    inner.push(local);
    inner
}

fn innermost<'a>(chain: &Chain<'a>) -> &'a dyn TypeScope {
    *chain.last().expect("a chain has at least one scope")
}

fn scope_at<'a>(chain: &Chain<'a>, count: u32) -> ParseResult<&'a dyn TypeScope> {
    chain
        .len()
        .checked_sub(1 + count as usize)
        .map(|i| chain[i])
        .ok_or_else(|| invalid("outer alias past the outermost scope"))
}

fn scope_type(scope: &dyn TypeScope, index: u32) -> ParseResult<Type> {
    scope
        .type_at(index)
        .cloned()
        .ok_or_else(|| invalid("unknown type index"))
}

fn val_type(chain: &Chain<'_>, ty: &ComponentValType) -> ParseResult<ValType> {
    match ty {
        ComponentValType::Primitive(p) => primitive(*p),
        ComponentValType::Type(index) => match scope_type(innermost(chain), *index)? {
            Type::Val(v) => Ok(v),
            _ => Err(invalid("type index is not a value type")),
        },
    }
}

fn primitive(p: PrimitiveValType) -> ParseResult<ValType> {
    Ok(match p {
        PrimitiveValType::Bool => ValType::Bool,
        PrimitiveValType::S8 => ValType::S8,
        PrimitiveValType::U8 => ValType::U8,
        PrimitiveValType::S16 => ValType::S16,
        PrimitiveValType::U16 => ValType::U16,
        PrimitiveValType::S32 => ValType::S32,
        PrimitiveValType::U32 => ValType::U32,
        PrimitiveValType::S64 => ValType::S64,
        PrimitiveValType::U64 => ValType::U64,
        PrimitiveValType::F32 => ValType::F32,
        PrimitiveValType::F64 => ValType::F64,
        PrimitiveValType::Char => ValType::Char,
        PrimitiveValType::String => ValType::String,
        PrimitiveValType::ErrorContext => return Err(unsupported("error-context")),
    })
}

fn resource_in(chain: &Chain<'_>, index: u32) -> ParseResult<Shared<Resource>> {
    match scope_type(innermost(chain), index)? {
        Type::Resource(resource) => Ok(resource),
        _ => Err(invalid("type index is not a resource")),
    }
}

fn defined_type(chain: &Chain<'_>, ty: &ComponentDefinedType) -> ParseResult<Type> {
    let defined = match ty {
        ComponentDefinedType::Primitive(p) => return primitive(*p).map(Type::Val),
        ComponentDefinedType::Record(fields) => DefinedType::Record(
            fields
                .iter()
                .map(|(name, ty)| Ok((name.to_string(), val_type(chain, ty)?)))
                .collect::<ParseResult<_>>()?,
        ),
        ComponentDefinedType::Variant(cases) => DefinedType::Variant(
            cases
                .iter()
                .map(|case| {
                    let ty = case.ty.as_ref().map(|ty| val_type(chain, ty)).transpose()?;
                    Ok((case.name.to_string(), ty))
                })
                .collect::<ParseResult<_>>()?,
        ),
        ComponentDefinedType::List(ty) => DefinedType::List(val_type(chain, ty)?),
        ComponentDefinedType::Tuple(types) => DefinedType::Tuple(
            types
                .iter()
                .map(|ty| val_type(chain, ty))
                .collect::<ParseResult<_>>()?,
        ),
        ComponentDefinedType::Flags(names) => {
            DefinedType::Flags(names.iter().map(|n| n.to_string()).collect())
        }
        ComponentDefinedType::Enum(names) => {
            DefinedType::Enum(names.iter().map(|n| n.to_string()).collect())
        }
        ComponentDefinedType::Option(ty) => DefinedType::Option(val_type(chain, ty)?),
        ComponentDefinedType::Result { ok, err } => DefinedType::Result {
            ok: ok.as_ref().map(|ty| val_type(chain, ty)).transpose()?,
            err: err.as_ref().map(|ty| val_type(chain, ty)).transpose()?,
        },
        ComponentDefinedType::Own(index) => DefinedType::Own(resource_in(chain, *index)?),
        ComponentDefinedType::Borrow(index) => DefinedType::Borrow(resource_in(chain, *index)?),
        ComponentDefinedType::Map(..) => return Err(unsupported("map types")),
        ComponentDefinedType::FixedLengthList(..) => return Err(unsupported("fixed-length lists")),
        ComponentDefinedType::Future(_) | ComponentDefinedType::Stream(_) => {
            return Err(unsupported("async types"))
        }
    };
    Ok(Type::Val(ValType::Defined(Shared::new(defined))))
}

fn component_type(chain: &Chain<'_>, ty: &wasmparser::ComponentType<'_>) -> ParseResult<Type> {
    Ok(match ty {
        wasmparser::ComponentType::Defined(defined) => defined_type(chain, defined)?,
        wasmparser::ComponentType::Func(func) => Type::Func(func_type(chain, func)?),
        wasmparser::ComponentType::Component(decls) => {
            Type::Component(component_type_decls(chain, decls)?)
        }
        wasmparser::ComponentType::Instance(decls) => {
            Type::Instance(instance_type_decls(chain, decls)?)
        }
        wasmparser::ComponentType::Resource { rep, dtor } => {
            Type::Resource(Shared::new(Resource {
                rep: Some(match_value_type(*rep)),
                dtor: *dtor,
            }))
        }
    })
}

fn func_type(
    chain: &Chain<'_>,
    func: &wasmparser::ComponentFuncType<'_>,
) -> ParseResult<Shared<FuncType>> {
    if func.async_ {
        return Err(unsupported("async functions"));
    }
    Ok(Shared::new(FuncType {
        params: func
            .params
            .iter()
            .map(|(name, ty)| Ok((name.to_string(), val_type(chain, ty)?)))
            .collect::<ParseResult<_>>()?,
        result: func
            .result
            .as_ref()
            .map(|ty| val_type(chain, ty))
            .transpose()?,
    }))
}

fn extern_desc(chain: &Chain<'_>, ty: &ComponentTypeRef) -> ParseResult<ExternDesc> {
    let scope = innermost(chain);
    Ok(match ty {
        ComponentTypeRef::Module(index) => match scope.core_type_at(*index) {
            Some(CoreType::Module(ty)) => ExternDesc::Module(Shared::clone(ty)),
            _ => return Err(invalid("core type index is not a module type")),
        },
        ComponentTypeRef::Func(index) => match scope_type(scope, *index)? {
            Type::Func(ty) => ExternDesc::Func(ty),
            _ => return Err(invalid("type index is not a function type")),
        },
        ComponentTypeRef::Value(ty) => ExternDesc::Value(val_type(chain, ty)?),
        ComponentTypeRef::Type(TypeBounds::Eq(index)) => {
            ExternDesc::Type(TypeBound::Eq(scope_type(scope, *index)?))
        }
        ComponentTypeRef::Type(TypeBounds::SubResource) => {
            ExternDesc::Type(TypeBound::SubResource(Shared::new(Resource {
                rep: None,
                dtor: None,
            })))
        }
        ComponentTypeRef::Instance(index) => match scope_type(scope, *index)? {
            Type::Instance(ty) => ExternDesc::Instance(ty),
            _ => return Err(invalid("type index is not an instance type")),
        },
        ComponentTypeRef::Component(index) => match scope_type(scope, *index)? {
            Type::Component(ty) => ExternDesc::Component(ty),
            _ => return Err(invalid("type index is not a component type")),
        },
    })
}

/// Handles the declarations instance and component types share.
fn declare(
    chain: &Chain<'_>,
    local: &mut LocalScope,
    decl: CommonDecl<'_, '_>,
    exports: &mut Vec<(String, ExternDesc)>,
) -> ParseResult<()> {
    match decl {
        CommonDecl::CoreType(ty) => {
            let ty = core_type(&with_local(chain, local), ty)?;
            local.core_types.push(ty);
        }
        CommonDecl::Type(ty) => {
            let ty = component_type(&with_local(chain, local), ty)?;
            local.types.push(ty);
        }
        CommonDecl::Alias(alias) => match alias {
            ComponentAlias::Outer { kind, count, index } => {
                let inner = with_local(chain, local);
                let scope = scope_at(&inner, *count)?;
                match kind {
                    ComponentOuterAliasKind::Type => {
                        let ty = scope_type(scope, *index)?;
                        local.types.push(ty);
                    }
                    ComponentOuterAliasKind::CoreType => {
                        let ty = scope
                            .core_type_at(*index)
                            .cloned()
                            .ok_or_else(|| invalid("unknown outer core type"))?;
                        local.core_types.push(ty);
                    }
                    // Modules and components have no index space a type declaration can refer to.
                    ComponentOuterAliasKind::CoreModule | ComponentOuterAliasKind::Component => {}
                }
            }
            ComponentAlias::InstanceExport {
                kind,
                instance_index,
                name,
            } => {
                let instance = local
                    .instance_type_at(*instance_index)
                    .ok_or_else(|| invalid("unknown instance index"))?;
                let desc = export_of(instance, name)?;
                match (kind, desc) {
                    (ComponentExternalKind::Type, ExternDesc::Type(bound)) => {
                        local.types.push(bound.to_type())
                    }
                    (ComponentExternalKind::Instance, ExternDesc::Instance(ty)) => {
                        local.instances.push(ty)
                    }
                    (ComponentExternalKind::Type | ComponentExternalKind::Instance, _) => {
                        return Err(invalid(format!("alias of {name} with the wrong sort")))
                    }
                    // Other sorts have no index space inside a type.
                    _ => {}
                }
            }
            ComponentAlias::CoreInstanceExport { .. } => {
                return Err(invalid("core instance alias inside a type"))
            }
        },
        CommonDecl::Export { name, ty } => {
            let desc = extern_desc(&with_local(chain, local), ty)?;
            match &desc {
                ExternDesc::Type(bound) => local.types.push(bound.to_type()),
                ExternDesc::Instance(ty) => local.instances.push(Shared::clone(ty)),
                _ => {}
            }
            exports.push((name.to_string(), desc));
        }
    }
    Ok(())
}

enum CommonDecl<'a, 'b> {
    CoreType(wasmparser::CoreType<'a>),
    Type(&'b wasmparser::ComponentType<'a>),
    Alias(&'b ComponentAlias<'a>),
    Export {
        name: &'a str,
        ty: &'b ComponentTypeRef,
    },
}

fn instance_type_decls(
    chain: &Chain<'_>,
    decls: &[InstanceTypeDeclaration<'_>],
) -> ParseResult<Shared<InstanceType>> {
    let mut local = LocalScope::default();
    let mut exports = Vec::new();
    for decl in decls {
        let decl = match decl {
            InstanceTypeDeclaration::CoreType(ty) => CommonDecl::CoreType(ty.clone()),
            InstanceTypeDeclaration::Type(ty) => CommonDecl::Type(ty),
            InstanceTypeDeclaration::Alias(alias) => CommonDecl::Alias(alias),
            InstanceTypeDeclaration::Export { name, ty } => CommonDecl::Export {
                name: name.name,
                ty,
            },
        };
        declare(chain, &mut local, decl, &mut exports)?;
    }
    Ok(Shared::new(InstanceType { exports }))
}

fn component_type_decls(
    chain: &Chain<'_>,
    decls: &[ComponentTypeDeclaration<'_>],
) -> ParseResult<Shared<ComponentType>> {
    let mut local = LocalScope::default();
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    for decl in decls {
        let decl = match decl {
            ComponentTypeDeclaration::CoreType(ty) => CommonDecl::CoreType(ty.clone()),
            ComponentTypeDeclaration::Type(ty) => CommonDecl::Type(ty),
            ComponentTypeDeclaration::Alias(alias) => CommonDecl::Alias(alias),
            ComponentTypeDeclaration::Export { name, ty } => CommonDecl::Export {
                name: name.name,
                ty,
            },
            ComponentTypeDeclaration::Import(import) => {
                declare(
                    chain,
                    &mut local,
                    CommonDecl::Export {
                        name: import.name.name,
                        ty: &import.ty,
                    },
                    &mut imports,
                )?;
                continue;
            }
        };
        declare(chain, &mut local, decl, &mut exports)?;
    }
    Ok(Shared::new(ComponentType { imports, exports }))
}

fn core_type(chain: &Chain<'_>, ty: wasmparser::CoreType<'_>) -> ParseResult<CoreType> {
    Ok(match ty {
        wasmparser::CoreType::Rec(group) => CoreType::Func(func_type_of_rec_group(group)?),
        wasmparser::CoreType::Module(decls) => CoreType::Module(module_type_decls(chain, &decls)?),
    })
}

fn module_type_decls(
    chain: &Chain<'_>,
    decls: &[ModuleTypeDeclaration<'_>],
) -> ParseResult<Shared<ModuleType>> {
    let mut func_types: Vec<CoreFuncType> = Vec::new();
    let mut module_ty = ModuleType::default();
    let core_extern = |func_types: &[CoreFuncType], ty: &TypeRef| -> ParseResult<CoreExternType> {
        Ok(match ty {
            TypeRef::Func(index) => CoreExternType::Func(
                func_types
                    .get(*index as usize)
                    .cloned()
                    .ok_or_else(|| invalid("unknown core type index in module type"))?,
            ),
            TypeRef::Table(t) => CoreExternType::Table(table_type_of(t)),
            TypeRef::Memory(m) => CoreExternType::Memory(mem_type_of(m)),
            TypeRef::Global(g) => CoreExternType::Global(global_type_of(g)),
            TypeRef::Tag(_) => return Err(unsupported("exception tags")),
            TypeRef::FuncExact(_) => return Err(unsupported("exact function types")),
        })
    };
    for decl in decls {
        match decl {
            ModuleTypeDeclaration::Type(group) => {
                func_types.push(func_type_of_rec_group(group.clone())?)
            }
            ModuleTypeDeclaration::Export { name, ty } => module_ty
                .exports
                .push((name.to_string(), core_extern(&func_types, ty)?)),
            ModuleTypeDeclaration::Import(import) => module_ty.imports.push((
                import.module.to_string(),
                import.name.to_string(),
                core_extern(&func_types, &import.ty)?,
            )),
            ModuleTypeDeclaration::OuterAlias {
                kind: OuterAliasKind::Type,
                count,
                index,
            } => {
                // Count 0 is the module type's own types; one level out is the enclosing scope.
                let func = if *count == 0 {
                    func_types.get(*index as usize).cloned()
                } else {
                    match scope_at(chain, count - 1)?.core_type_at(*index) {
                        Some(CoreType::Func(func)) => Some(func.clone()),
                        _ => None,
                    }
                };
                func_types.push(func.ok_or_else(|| {
                    invalid("outer alias in a module type is not a function type")
                })?);
            }
        }
    }
    Ok(Shared::new(module_ty))
}

/// The type of a parsed core module: its imports and exports with their types.
fn module_type_of(module: &Module) -> ParseResult<ModuleType> {
    let mut func_types: Vec<CoreFuncType> = Vec::new();
    let mut tables = Vec::new();
    let mut memories = Vec::new();
    let mut globals = Vec::new();
    let mut ty = ModuleType::default();
    let core_type = |index: u32| -> ParseResult<CoreFuncType> {
        module
            .types
            .get(index as usize)
            .cloned()
            .ok_or_else(|| invalid("unknown type index in a nested module"))
    };
    for import in &module.imports {
        let desc = match &import.desc {
            ImportDesc::Func(index) => {
                let func = core_type(index.0)?;
                func_types.push(func.clone());
                CoreExternType::Func(func)
            }
            ImportDesc::WasiFunc(wasi) => {
                let func = wasi.expected_func_type();
                func_types.push(func.clone());
                CoreExternType::Func(func)
            }
            ImportDesc::Table(t) => {
                tables.push(*t);
                CoreExternType::Table(*t)
            }
            ImportDesc::Mem(m) => {
                memories.push(*m);
                CoreExternType::Memory(*m)
            }
            ImportDesc::Global(g) => {
                globals.push(*g);
                CoreExternType::Global(*g)
            }
        };
        ty.imports
            .push((import.module.0.clone(), import.name.0.clone(), desc));
    }
    for func in &module.funcs {
        func_types.push(core_type(func.type_.0)?);
    }
    tables.extend(module.tables.iter().map(|t| t.type_));
    memories.extend(module.mems.iter().map(|m| m.type_));
    globals.extend(module.globals.iter().map(|g| g.type_));
    fn exported<T: Clone>(items: &[T], index: u32, what: &str) -> ParseResult<T> {
        items
            .get(index as usize)
            .cloned()
            .ok_or_else(|| invalid(format!("nested module exports an unknown {what}")))
    }
    for export in &module.exports {
        let desc = match &export.desc {
            ExportDesc::Func(i) => CoreExternType::Func(exported(&func_types, i.0, "function")?),
            ExportDesc::Table(i) => CoreExternType::Table(exported(&tables, i.0, "table")?),
            ExportDesc::Mem(i) => CoreExternType::Memory(exported(&memories, i.0, "memory")?),
            ExportDesc::Global(i) => CoreExternType::Global(exported(&globals, i.0, "global")?),
        };
        ty.exports.push((export.name.0.clone(), desc));
    }
    Ok(ty)
}

/// The type of a parsed component: its imports and exports.
fn component_type_of(component: &Component) -> ComponentType {
    ComponentType {
        imports: component
            .imports
            .iter()
            .map(|i| (i.name.clone(), i.desc.clone()))
            .collect(),
        exports: component
            .exports
            .iter()
            .map(|e| (e.name.clone(), e.desc.clone()))
            .collect(),
    }
}
