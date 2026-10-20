//! Runs the core spec's `linking.wast`:
//! modules importing functions, tables, memories and globals from each other,
//! and the link-time and instantiation failures the spec defines.
//!
//! `tests/proposals/fetch.sh` downloads the file;
//! without it this test fails.
//!
//! The script's modules are instantiated in order.
//! `register` exposes one to the modules after it,
//! and `invoke` and `get` address a module by its `$id` or default to the last one instantiated.
//! The `spectest` host module the core scripts import is provided as a Wasm module.
//! `assert_invalid` and `assert_malformed` are counted and skipped,
//! since chiwawa has no validator.
//!
//! One `#[test]` covers the whole script,
//! so a passing run reports "1 passed".
//! `cargo test -- --show-output` prints how many directives that was;
//! a failure lists each one with its line in the upstream file.

use chiwawa::error::RuntimeError;
use chiwawa::{
    execution::module::*,
    execution::runtime::{run_start_section, Runtime, RuntimeConfig},
    execution::value::*,
    parser,
    structure::module::Module,
};
use rustc_hash::FxHashMap;
use std::rc::Rc;
use wast::core::{NanPattern, WastArgCore, WastRetCore};
use wast::token::Id;
use wast::{WastArg, WastDirective, WastExecute, WastRet};

/// The host module the spec scripts import, matching `interpreter/host/spectest.ml` in the spec repository.
/// The print functions do nothing here; no script asserts on their output.
const SPECTEST: &str = r#"(module
  (func (export "print"))
  (func (export "print_i32") (param i32))
  (func (export "print_i64") (param i64))
  (func (export "print_f32") (param f32))
  (func (export "print_f64") (param f64))
  (func (export "print_i32_f32") (param i32 f32))
  (func (export "print_f64_f64") (param f64 f64))
  (global (export "global_i32") i32 (i32.const 666))
  (global (export "global_i64") i64 (i64.const 666))
  (global (export "global_f32") f32 (f32.const 666.6))
  (global (export "global_f64") f64 (f64.const 666.6))
  (table (export "table") 10 20 funcref)
  (memory (export "memory") 1 2)
)"#;

/// What one directive did, for the tally printed at the end.
#[derive(Default)]
struct Tally {
    passed: usize,
    skipped: usize,
    failures: Vec<String>,
}

/// Why a module did not come up, since the directives expect a specific stage to fail.
enum Failure {
    Parse(String),
    Link,
    Trap(RuntimeError),
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::Parse(e) => write!(f, "parse: {e}"),
            Failure::Link => write!(f, "link error"),
            Failure::Trap(e) => write!(f, "trap: {e:?}"),
        }
    }
}

struct Script {
    /// Where a module is staged: `parse_bytecode` reads from a path, and a `.wast` carries its modules inline.
    staged: String,
    /// Exports of the registered modules, by the name `register` gave them.
    registry: ImportObjects,
    named: FxHashMap<String, Rc<ModuleInst>>,
    current: Option<Rc<ModuleInst>>,
    /// Every instance made, since a function imported from one keeps only a weak reference to it.
    all: Vec<Rc<ModuleInst>>,
    tally: Tally,
}

impl Script {
    fn instantiate(&mut self, bytes: &[u8]) -> Result<Rc<ModuleInst>, Failure> {
        std::fs::write(&self.staged, bytes).map_err(|e| Failure::Parse(format!("stage: {e}")))?;
        let mut module = Module::new("spec");
        let parsed = parser::parse_bytecode(&mut module, &self.staged)
            .map_err(|e| Failure::Parse(format!("{e:?}")));
        let _ = std::fs::remove_file(&self.staged);
        parsed?;
        let inst =
            ModuleInst::new(&module, self.registry.clone(), Vec::new()).map_err(|e| match e {
                RuntimeError::LinkError => Failure::Link,
                other => Failure::Trap(other),
            })?;
        self.all.push(Rc::clone(&inst));
        run_start_section(&inst).map_err(Failure::Trap)?;
        Ok(inst)
    }

    fn register(&mut self, name: &str, inst: &Rc<ModuleInst>) {
        let exports = inst
            .exports
            .iter()
            .map(|export| (export.name.clone(), export.value.clone()))
            .collect();
        self.registry.insert(name.to_string(), exports);
    }

    /// The module a directive addresses: by `$id`, or the last one instantiated.
    fn module(&self, id: Option<Id>) -> Result<Rc<ModuleInst>, String> {
        match id {
            Some(id) => self
                .named
                .get(id.name())
                .cloned()
                .ok_or_else(|| format!("unknown module ${}", id.name())),
            None => self.current.clone().ok_or_else(|| "no module".to_string()),
        }
    }

    fn invoke(&self, id: Option<Id>, name: &str, args: &[WastArg]) -> Result<Vec<Val>, String> {
        let inst = self.module(id)?;
        let params = args.iter().map(arg_to_val).collect::<Result<_, _>>()?;
        let func = inst.get_export_func(name).map_err(|e| format!("{e:?}"))?;
        Runtime::new(Rc::clone(&inst), &func, params, RuntimeConfig::default())
            .and_then(|mut runtime| runtime.run())
            .map_err(|e| format!("trapped: {e:?}"))
    }

    fn get(&self, id: Option<Id>, name: &str) -> Result<Val, String> {
        let inst = self.module(id)?;
        inst.exports
            .iter()
            .find(|export| export.name == name)
            .and_then(|export| export.value.clone().as_global())
            .map(|global| global.get())
            .ok_or_else(|| format!("no global export {name}"))
    }
}

fn arg_to_val(arg: &WastArg) -> Result<Val, String> {
    let WastArg::Core(core) = arg else {
        return Err("non-core argument".into());
    };
    Ok(match core {
        WastArgCore::I32(v) => Val::Num(Num::I32(*v)),
        WastArgCore::I64(v) => Val::Num(Num::I64(*v)),
        WastArgCore::F32(v) => Val::Num(Num::F32(f32::from_bits(v.bits))),
        WastArgCore::F64(v) => Val::Num(Num::F64(f64::from_bits(v.bits))),
        other => return Err(format!("unsupported argument {other:?}")),
    })
}

/// Compares one result against what the directive expects.
/// Floats compare by bit pattern;
/// anything else the runner does not understand is reported rather than silently accepted.
fn matches(expected: &WastRet, actual: &Val) -> Result<bool, String> {
    let WastRet::Core(core) = expected else {
        return Err("non-core result".into());
    };
    Ok(match (core, actual) {
        (WastRetCore::I32(e), Val::Num(Num::I32(a))) => e == a,
        (WastRetCore::I64(e), Val::Num(Num::I64(a))) => e == a,
        (WastRetCore::F32(NanPattern::Value(e)), Val::Num(Num::F32(a))) => e.bits == a.to_bits(),
        (WastRetCore::F64(NanPattern::Value(e)), Val::Num(Num::F64(a))) => e.bits == a.to_bits(),
        (other, _) => return Err(format!("unsupported result {other:?}")),
    })
}

fn results_match(expected: &[WastRet], actual: &[Val]) -> bool {
    expected.len() == actual.len()
        && expected
            .iter()
            .zip(actual)
            .all(|(e, a)| matches(e, a).unwrap_or(false))
}

/// Runs the script at `path` and panics with every failed directive.
fn run_wast(path: &str) {
    let source = std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("cannot read {path}: {e}\nrun tests/proposals/fetch.sh to download the testsuite");
    });

    let buf = wast::parser::ParseBuffer::new(&source).expect("lex the script");
    let wast: wast::Wast = wast::parser::parse(&buf).expect("parse the script");

    let mut script = Script {
        staged: format!("{path}.wasm"),
        registry: ImportObjects::default(),
        named: FxHashMap::default(),
        current: None,
        all: Vec::new(),
        tally: Tally::default(),
    };
    let spectest = script
        .instantiate(&wat::parse_str(SPECTEST).expect("spectest module"))
        .unwrap_or_else(|e| panic!("spectest: {e}"));
    script.register("spectest", &spectest);

    for directive in wast.directives {
        // Line numbers make a failure locatable in the upstream file.
        let line = directive.span().linecol_in(&source).0 + 1;
        let fail = |script: &mut Script, what: String| {
            script.tally.failures.push(format!("{line}: {what}"))
        };
        match directive {
            WastDirective::Module(mut wat) => {
                let id = wat.name().map(|id| id.name().to_string());
                let bytes = match wat.encode() {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        fail(&mut script, format!("module: encode: {e}"));
                        continue;
                    }
                };
                match script.instantiate(&bytes) {
                    Ok(inst) => {
                        if let Some(id) = id {
                            script.named.insert(id, Rc::clone(&inst));
                        }
                        script.current = Some(inst);
                        script.tally.passed += 1;
                    }
                    Err(e) => {
                        script.current = None;
                        fail(&mut script, format!("module: {e}"));
                    }
                }
            }

            WastDirective::Register { name, module, .. } => match script.module(module) {
                Ok(inst) => {
                    script.register(name, &inst);
                    script.tally.passed += 1;
                }
                Err(e) => fail(&mut script, format!("register {name}: {e}")),
            },

            WastDirective::Invoke(i) => match script.invoke(i.module, i.name, &i.args) {
                Ok(_) => script.tally.passed += 1,
                Err(e) => fail(&mut script, format!("invoke {}: {e}", i.name)),
            },

            WastDirective::AssertReturn { exec, results, .. } => {
                let (name, actual) = match exec {
                    WastExecute::Invoke(i) => (i.name, script.invoke(i.module, i.name, &i.args)),
                    WastExecute::Get { module, global, .. } => {
                        (global, script.get(module, global).map(|v| vec![v]))
                    }
                    WastExecute::Wat(_) => {
                        script.tally.skipped += 1;
                        continue;
                    }
                };
                match actual {
                    Ok(actual) if results_match(&results, &actual) => script.tally.passed += 1,
                    Ok(actual) => fail(
                        &mut script,
                        format!("{name}: expected {results:?}, got {actual:?}"),
                    ),
                    Err(e) => fail(&mut script, format!("{name}: {e}")),
                }
            }

            WastDirective::AssertTrap { exec, message, .. } => match exec {
                WastExecute::Invoke(i) => match script.invoke(i.module, i.name, &i.args) {
                    Err(_) => script.tally.passed += 1,
                    Ok(v) => fail(
                        &mut script,
                        format!("{}: expected trap \"{message}\", got {v:?}", i.name),
                    ),
                },
                WastExecute::Wat(mut wat) => {
                    match wat.encode().map(|bytes| script.instantiate(&bytes)) {
                        Ok(Err(Failure::Trap(_))) => script.tally.passed += 1,
                        Ok(Err(e)) => {
                            fail(&mut script, format!("expected trap \"{message}\", got {e}"))
                        }
                        Ok(Ok(_)) => fail(
                            &mut script,
                            format!("expected trap \"{message}\", got a module"),
                        ),
                        Err(e) => fail(&mut script, format!("encode: {e}")),
                    }
                }
                WastExecute::Get { .. } => script.tally.skipped += 1,
            },

            WastDirective::AssertUnlinkable {
                mut module,
                message,
                ..
            } => match module.encode().map(|bytes| script.instantiate(&bytes)) {
                Ok(Err(Failure::Link)) => script.tally.passed += 1,
                Ok(Err(e)) => fail(
                    &mut script,
                    format!("expected link error \"{message}\", got {e}"),
                ),
                Ok(Ok(_)) => fail(
                    &mut script,
                    format!("expected link error \"{message}\", got a module"),
                ),
                Err(e) => fail(&mut script, format!("encode: {e}")),
            },

            // chiwawa has no validator, so the module-rejection directives cannot be checked.
            WastDirective::AssertInvalid { .. } | WastDirective::AssertMalformed { .. } => {
                script.tally.skipped += 1
            }

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
    for failure in &tally.failures {
        println!("  {failure}");
    }
    assert!(
        tally.failures.is_empty(),
        "{} directives failed",
        tally.failures.len()
    );
}

#[test]
fn linking_wast() {
    run_wast("tests/proposals/core/linking.wast");
}
