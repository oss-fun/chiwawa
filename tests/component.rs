//! Runs the Component Model reference tests from the component-model repository's `test/` directory.
//!
//! `tests/proposals/fetch.sh` downloads the files; without them these tests fail.
//!
//! Every `component` directive must parse, and one without imports is instantiated.
//! `invoke`, `assert_return` and `assert_trap` then call its exports with component values.
//! A component that uses a feature chiwawa leaves out, such as async, resources or strings,
//! is skipped and listed, together with the directives that address it.
//! Every other directive is counted and skipped,
//! `assert_malformed` and `assert_invalid` among them, since chiwawa has no validator.
//!
//! One `#[test]` covers a whole script, so a passing run reports "1 passed".
//! `cargo test -- --show-output` prints how many directives that was;
//! a failure lists each one with its line in the upstream file.

use chiwawa::error::{ParserError, RuntimeError};
use chiwawa::execution::component::{ComponentInst, ComponentVal};
use chiwawa::parser::{self, Parsed};
use chiwawa::shared::Shared;
use chiwawa::structure::component::Component;
use rustc_hash::FxHashMap;
use std::rc::Rc;
use wast::component::WastVal;
use wast::token::Id;
use wast::{QuoteWat, WastArg, WastDirective, WastExecute, WastRet};

/// What one directive did, for the tally printed at the end.
#[derive(Default)]
struct Tally {
    passed: usize,
    skipped: usize,
    /// Components and calls skipped for using a feature chiwawa leaves out, with the feature.
    unsupported: Vec<String>,
    failures: Vec<String>,
}

enum Failure {
    Unsupported(String),
    Other(String),
}

impl From<RuntimeError> for Failure {
    fn from(e: RuntimeError) -> Self {
        match e {
            RuntimeError::UnsupportedComponentFeature(feature)
            | RuntimeError::UnsupportedFeature(feature) => {
                Failure::Unsupported(feature.to_string())
            }
            other => Failure::Other(format!("{other:?}")),
        }
    }
}

/// What a directive's binary turned out to be.
enum Loaded {
    Module,
    Component(Shared<Component>),
}

fn parse(bytes: &[u8]) -> Result<Loaded, Failure> {
    match parser::parse_bytes(bytes) {
        Ok(Parsed::Module(..)) => Ok(Loaded::Module),
        Ok(Parsed::Component(component)) => Ok(Loaded::Component(component)),
        Err(e) => Err(match e.downcast_ref::<ParserError>() {
            Some(ParserError::UnsupportedComponentFeature(feature)) => {
                Failure::Unsupported(feature.to_string())
            }
            Some(ParserError::UnsupportedInstruction(instr)) => {
                Failure::Unsupported(format!("instruction {instr}"))
            }
            _ => Failure::Other(format!("{e}")),
        }),
    }
}

/// The instance the next `invoke` addresses when it names none.
enum Current {
    None,
    /// The last component was skipped, so the directives addressing it are skipped too.
    Skipped,
    Instance(Rc<ComponentInst>),
}

struct Script {
    /// `component definition` components, by `$id`; `None` for one skipped as unsupported.
    definitions: FxHashMap<String, Option<Shared<Component>>>,
    named: FxHashMap<String, Rc<ComponentInst>>,
    current: Current,
    tally: Tally,
}

impl Script {
    fn fail(&mut self, line: usize, what: String) {
        self.tally.failures.push(format!("{line}: {what}"));
    }

    fn skip(&mut self, line: usize, feature: String) {
        self.tally.skipped += 1;
        self.tally.unsupported.push(format!("{line}: {feature}"));
    }

    /// Loads a `component` or `component definition` directive.
    fn load(&mut self, wat: &mut QuoteWat, definition: bool, line: usize) {
        let id = wat.name().map(|id| id.name().to_string());
        let loaded = wat
            .encode()
            .map_err(|e| Failure::Other(format!("encode: {e}")))
            .and_then(|bytes| parse(&bytes));
        let component = match loaded {
            Ok(Loaded::Component(component)) => component,
            Ok(Loaded::Module) => {
                self.current = Current::None;
                self.tally.passed += 1;
                return;
            }
            Err(Failure::Unsupported(feature)) => {
                if let Some(id) = id {
                    self.definitions.insert(id, None);
                }
                self.current = Current::Skipped;
                self.skip(line, feature);
                return;
            }
            Err(Failure::Other(e)) => {
                self.current = Current::None;
                self.fail(line, e);
                return;
            }
        };
        if definition {
            if let Some(id) = id {
                self.definitions.insert(id, Some(component));
            }
            self.tally.passed += 1;
            return;
        }
        // Nothing can supply a top-level import, so such a component is only parsed.
        if !component.imports.is_empty() {
            self.current = Current::None;
            self.tally.passed += 1;
            return;
        }
        self.instantiate_directive(&component, id.as_deref(), line);
    }

    /// Instantiates a component for a directive and tallies the outcome.
    fn instantiate_directive(&mut self, component: &Component, id: Option<&str>, line: usize) {
        match self.instantiate(component, id) {
            Ok(()) => self.tally.passed += 1,
            Err(Failure::Unsupported(feature)) => {
                self.current = Current::Skipped;
                self.skip(line, feature);
            }
            Err(Failure::Other(e)) => {
                self.current = Current::None;
                self.fail(line, e);
            }
        }
    }

    /// Instantiates a component and makes it current.
    fn instantiate(&mut self, component: &Component, id: Option<&str>) -> Result<(), Failure> {
        let inst = ComponentInst::new(component, Vec::new())?;
        if let Some(id) = id {
            self.named.insert(id.to_string(), Rc::clone(&inst));
        }
        self.current = Current::Instance(inst);
        Ok(())
    }

    fn invoke(
        &self,
        id: Option<Id>,
        name: &str,
        args: &[WastArg],
    ) -> Result<Vec<ComponentVal>, Failure> {
        let inst = match id {
            Some(id) => self
                .named
                .get(id.name())
                .ok_or_else(|| Failure::Other(format!("unknown instance ${}", id.name())))?,
            None => match &self.current {
                Current::Instance(inst) => inst,
                Current::Skipped => return Err(Failure::Unsupported("skipped component".into())),
                Current::None => return Err(Failure::Other("no instance".into())),
            },
        };
        let func = inst.get_export_func(name)?;
        let args = args
            .iter()
            .map(arg_to_val)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Failure::Other)?;
        Ok(func.call(args)?)
    }
}

fn arg_to_val(arg: &WastArg) -> Result<ComponentVal, String> {
    match arg {
        WastArg::Component(val) => wast_to_val(val),
        _ => Err("core argument to a component function".into()),
    }
}

fn wast_to_val(val: &WastVal) -> Result<ComponentVal, String> {
    let boxed = |v: &Option<Box<WastVal>>| -> Result<Option<Box<ComponentVal>>, String> {
        v.as_deref()
            .map(wast_to_val)
            .transpose()
            .map(|v| v.map(Box::new))
    };
    Ok(match val {
        WastVal::Bool(v) => ComponentVal::Bool(*v),
        WastVal::S8(v) => ComponentVal::S8(*v),
        WastVal::U8(v) => ComponentVal::U8(*v),
        WastVal::S16(v) => ComponentVal::S16(*v),
        WastVal::U16(v) => ComponentVal::U16(*v),
        WastVal::S32(v) => ComponentVal::S32(*v),
        WastVal::U32(v) => ComponentVal::U32(*v),
        WastVal::S64(v) => ComponentVal::S64(*v),
        WastVal::U64(v) => ComponentVal::U64(*v),
        WastVal::F32(v) => ComponentVal::F32(f32::from_bits(v.bits)),
        WastVal::F64(v) => ComponentVal::F64(f64::from_bits(v.bits)),
        WastVal::Char(c) => ComponentVal::Char(*c),
        WastVal::String(s) => ComponentVal::String(s.to_string()),
        WastVal::List(items) => {
            ComponentVal::List(items.iter().map(wast_to_val).collect::<Result<_, _>>()?)
        }
        WastVal::Record(fields) => ComponentVal::Record(
            fields
                .iter()
                .map(|(name, v)| Ok((name.to_string(), wast_to_val(v)?)))
                .collect::<Result<_, String>>()?,
        ),
        WastVal::Tuple(items) => {
            ComponentVal::Tuple(items.iter().map(wast_to_val).collect::<Result<_, _>>()?)
        }
        WastVal::Variant(name, payload) => ComponentVal::Variant(name.to_string(), boxed(payload)?),
        WastVal::Enum(name) => ComponentVal::Enum(name.to_string()),
        WastVal::Option(v) => ComponentVal::Option(boxed(v)?),
        WastVal::Result(r) => ComponentVal::Result(match r {
            Ok(v) => Ok(boxed(v)?),
            Err(v) => Err(boxed(v)?),
        }),
        WastVal::Flags(names) => {
            ComponentVal::Flags(names.iter().map(|name| name.to_string()).collect())
        }
    })
}

/// Compares one result against what the directive expects.
fn matches(expected: &WastRet, actual: &ComponentVal) -> Result<bool, String> {
    let WastRet::Component(val) = expected else {
        return Err("core result from a component function".into());
    };
    Ok(same(&wast_to_val(val)?, actual))
}

/// Value equality with floats by bit pattern, flags as sets and records by field name.
fn same(e: &ComponentVal, a: &ComponentVal) -> bool {
    type Boxed = Option<Box<ComponentVal>>;
    let same_boxed = |e: &Boxed, a: &Boxed| match (e, a) {
        (Some(e), Some(a)) => same(e, a),
        (None, None) => true,
        _ => false,
    };
    match (e, a) {
        (ComponentVal::F32(e), ComponentVal::F32(a)) => e.to_bits() == a.to_bits(),
        (ComponentVal::F64(e), ComponentVal::F64(a)) => e.to_bits() == a.to_bits(),
        (ComponentVal::Flags(e), ComponentVal::Flags(a)) => {
            let (mut e, mut a) = (e.clone(), a.clone());
            e.sort();
            a.sort();
            e == a
        }
        (ComponentVal::List(e), ComponentVal::List(a))
        | (ComponentVal::Tuple(e), ComponentVal::Tuple(a)) => {
            e.len() == a.len() && e.iter().zip(a).all(|(e, a)| same(e, a))
        }
        (ComponentVal::Record(e), ComponentVal::Record(a)) => {
            e.len() == a.len()
                && e.iter()
                    .all(|(name, ev)| a.iter().any(|(an, av)| an == name && same(ev, av)))
        }
        (ComponentVal::Variant(en, ep), ComponentVal::Variant(an, ap)) => {
            en == an && same_boxed(ep, ap)
        }
        (ComponentVal::Option(e), ComponentVal::Option(a)) => same_boxed(e, a),
        (ComponentVal::Result(Ok(e)), ComponentVal::Result(Ok(a)))
        | (ComponentVal::Result(Err(e)), ComponentVal::Result(Err(a))) => same_boxed(e, a),
        (e, a) => e == a,
    }
}

fn results_match(expected: &[WastRet], actual: &[ComponentVal]) -> bool {
    expected.len() == actual.len()
        && expected
            .iter()
            .zip(actual)
            .all(|(e, a)| matches(e, a).unwrap_or(false))
}

fn run_wast(path: &str) {
    let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("cannot read {path}: {e}\nrun tests/proposals/fetch.sh to download the testsuite");
    });

    let buf = wast::parser::ParseBuffer::new(&source).expect("lex the script");
    let wast: wast::Wast = wast::parser::parse(&buf).expect("parse the script");

    let mut script = Script {
        definitions: FxHashMap::default(),
        named: FxHashMap::default(),
        current: Current::None,
        tally: Tally::default(),
    };
    for directive in wast.directives {
        // Line numbers make a failure locatable in the upstream file.
        let line = directive.span().linecol_in(&source).0 + 1;
        match directive {
            WastDirective::Module(mut wat) => script.load(&mut wat, false, line),
            WastDirective::ModuleDefinition(mut wat) => script.load(&mut wat, true, line),

            WastDirective::ModuleInstance {
                instance, module, ..
            } => {
                let definition = module.and_then(|id| script.definitions.get(id.name()).cloned());
                match definition {
                    Some(Some(definition)) => {
                        let id = instance.map(|id| id.name().to_string());
                        script.instantiate_directive(&definition, id.as_deref(), line);
                    }
                    Some(None) => {
                        script.current = Current::Skipped;
                        script.skip(line, "skipped definition".into());
                    }
                    None => script.fail(line, "component instance of an unknown definition".into()),
                }
            }

            WastDirective::Invoke(i) => match script.invoke(i.module, i.name, &i.args) {
                Ok(_) => script.tally.passed += 1,
                Err(Failure::Unsupported(feature)) => script.skip(line, feature),
                Err(Failure::Other(e)) => script.fail(line, format!("invoke {}: {e}", i.name)),
            },

            WastDirective::AssertReturn {
                exec: WastExecute::Invoke(i),
                results,
                ..
            } => match script.invoke(i.module, i.name, &i.args) {
                Ok(actual) if results_match(&results, &actual) => script.tally.passed += 1,
                Ok(actual) => script.fail(
                    line,
                    format!("{}: expected {results:?}, got {actual:?}", i.name),
                ),
                Err(Failure::Unsupported(feature)) => script.skip(line, feature),
                Err(Failure::Other(e)) => script.fail(line, format!("{}: {e}", i.name)),
            },

            WastDirective::AssertTrap {
                exec: WastExecute::Invoke(i),
                message,
                ..
            } => match script.invoke(i.module, i.name, &i.args) {
                Err(Failure::Other(_)) => script.tally.passed += 1,
                Err(Failure::Unsupported(feature)) => script.skip(line, feature),
                Ok(v) => script.fail(
                    line,
                    format!("{}: expected trap \"{message}\", got {v:?}", i.name),
                ),
            },

            _ => script.tally.skipped += 1,
        }
    }

    let tally = &script.tally;
    println!(
        "{path}: {} passed, {} skipped, {} failed",
        tally.passed,
        tally.skipped,
        tally.failures.len()
    );
    for skipped in &tally.unsupported {
        println!("  skipped, unsupported: {skipped}");
    }
    for failure in &tally.failures {
        println!("  {failure}");
    }
    assert!(
        tally.failures.is_empty(),
        "{} directives failed",
        tally.failures.len()
    );
}

macro_rules! wast_tests {
    ($($name:ident => $file:literal,)*) => {
        $(
            #[test]
            fn $name() {
                run_wast(concat!("tests/proposals/component/", $file));
            }
        )*
    };
}

wast_tests! {
    binary => "binary/binary.wast",
    linking_unit => "linking/unit.wast",
    linking_link_time_virtualization => "linking/link-time-virtualization.wast",
    linking_shared_everything_dynamic_linking => "linking/shared-everything-dynamic-linking.wast",
    values_alignment => "values/alignment.wast",
    values_concat => "values/concat.wast",
    values_numerics => "values/numerics.wast",
    values_post_return => "values/post-return.wast",
    values_realloc => "values/realloc.wast",
    values_strings => "values/strings.wast",
    values_transcode => "values/transcode.wast",
    values_variants => "values/variants.wast",
    resources_borrows => "resources/borrows.wast",
    resources_handle_table => "resources/handle-table.wast",
    resources_multiple_resources => "resources/multiple-resources.wast",
    validation_abi => "validation/abi.wast",
    validation_annotated_names => "validation/annotated-names.wast",
    validation_attributes => "validation/attributes.wast",
    validation_core_modules => "validation/core-modules.wast",
    validation_defined_types => "validation/defined-types.wast",
    validation_extern_names => "validation/extern-names.wast",
    validation_external_visibility => "validation/external-visibility.wast",
    validation_indicies => "validation/indicies.wast",
    validation_instantiation => "validation/instantiation.wast",
    validation_kebab => "validation/kebab.wast",
    validation_max_value_size => "validation/max-value-size.wast",
    validation_outer_alias => "validation/outer-alias.wast",
    validation_resources => "validation/resources.wast",
}
