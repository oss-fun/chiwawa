//! Runs the threads proposal's `atomic.wast` from the official testsuite.
//!
//! `tests/proposals/fetch.sh` downloads the file; without it this test fails.
//! `assert_invalid` and `assert_malformed` are counted and skipped, since
//! chiwawa has no validator.
//!
//! One `#[test]` covers the whole script, so a passing run reports "1 passed".
//! `cargo test -- --show-output` prints how many directives that was; a failure
//! lists each one with its line in the upstream file.

use chiwawa::{
    execution::module::*,
    execution::runtime::{Runtime, RuntimeConfig},
    execution::value::*,
    parser,
    structure::module::Module,
};
use rustc_hash::FxHashMap;
use std::rc::Rc;
use wast::core::{WastArgCore, WastRetCore};
use wast::{QuoteWat, WastArg, WastDirective, WastExecute, WastRet};

const WAST: &str = "tests/proposals/threads/atomic.wast";

/// What one directive did, for the tally printed at the end.
#[derive(Default)]
struct Tally {
    passed: usize,
    skipped: usize,
    failures: Vec<String>,
}

/// Where a script's module is staged. `parse_bytecode` reads from a path, and
/// a `.wast` carries its modules inline, so each one goes through a file.
const STAGED: &str = "tests/proposals/.module.wasm";

fn instantiate(wat: &mut QuoteWat) -> Result<Rc<ModuleInst>, String> {
    let bytes = wat.encode().map_err(|e| format!("encode: {e}"))?;
    std::fs::write(STAGED, &bytes).map_err(|e| format!("stage: {e}"))?;
    let mut module = Module::new("spec");
    let parsed = parser::parse_bytecode(&mut module, STAGED).map_err(|e| format!("parse: {e:?}"));
    let _ = std::fs::remove_file(STAGED);
    parsed?;
    let imports: ImportObjects = FxHashMap::default();
    ModuleInst::new(&module, imports, Vec::new()).map_err(|e| format!("instantiate: {e:?}"))
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

/// Compares one result against what the directive expects. `atomic.wast` only
/// ever expects plain integers, so anything else is reported rather than
/// silently accepted.
fn matches(expected: &WastRet, actual: &Val) -> Result<bool, String> {
    let WastRet::Core(core) = expected else {
        return Err("non-core result".into());
    };
    Ok(match (core, actual) {
        (WastRetCore::I32(e), Val::Num(Num::I32(a))) => e == a,
        (WastRetCore::I64(e), Val::Num(Num::I64(a))) => e == a,
        (other, _) => return Err(format!("unsupported result {other:?}")),
    })
}

fn invoke(
    inst: &Rc<ModuleInst>,
    name: &str,
    args: &[WastArg],
) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
    let params = args
        .iter()
        .map(|a| arg_to_val(a).expect("argument the runner understands"))
        .collect();
    let func = inst.get_export_func(name)?;
    Runtime::new(Rc::clone(inst), &func, params, RuntimeConfig::default())?.run()
}

#[test]
fn atomic_wast() {
    let source = std::fs::read_to_string(WAST).unwrap_or_else(|e| {
        panic!("cannot read {WAST}: {e}\nrun tests/proposals/fetch.sh to download the testsuite");
    });

    let buf = wast::parser::ParseBuffer::new(&source).expect("lex the script");
    let wast: wast::Wast = wast::parser::parse(&buf).expect("parse the script");

    let mut tally = Tally::default();
    let mut inst: Option<Rc<ModuleInst>> = None;

    for directive in wast.directives {
        // Line numbers make a failure locatable in the upstream file.
        let line = directive.span().linecol_in(&source).0 + 1;
        match directive {
            WastDirective::Wat(mut wat) => match instantiate(&mut wat) {
                Ok(new) => {
                    inst = Some(new);
                    tally.passed += 1;
                }
                Err(e) => {
                    inst = None;
                    tally.failures.push(format!("{line}: module: {e}"));
                }
            },

            WastDirective::Invoke(i) => {
                let Some(inst) = inst.as_ref() else { continue };
                match invoke(inst, i.name, &i.args) {
                    Ok(_) => tally.passed += 1,
                    Err(e) => tally
                        .failures
                        .push(format!("{line}: invoke {}: {e:?}", i.name)),
                }
            }

            WastDirective::AssertReturn { exec, results, .. } => {
                let WastExecute::Invoke(i) = exec else {
                    tally.skipped += 1;
                    continue;
                };
                let Some(inst) = inst.as_ref() else { continue };
                match invoke(inst, i.name, &i.args) {
                    Ok(actual) => {
                        let ok = actual.len() == results.len()
                            && results
                                .iter()
                                .zip(&actual)
                                .all(|(e, a)| matches(e, a).unwrap_or(false));
                        if ok {
                            tally.passed += 1;
                        } else {
                            tally.failures.push(format!(
                                "{line}: {}: expected {results:?}, got {actual:?}",
                                i.name
                            ));
                        }
                    }
                    Err(e) => tally
                        .failures
                        .push(format!("{line}: {}: trapped: {e:?}", i.name)),
                }
            }

            WastDirective::AssertTrap { exec, message, .. } => {
                let WastExecute::Invoke(i) = exec else {
                    tally.skipped += 1;
                    continue;
                };
                let Some(inst) = inst.as_ref() else { continue };
                match invoke(inst, i.name, &i.args) {
                    Err(_) => tally.passed += 1,
                    Ok(v) => tally.failures.push(format!(
                        "{line}: {}: expected trap \"{message}\", got {v:?}",
                        i.name
                    )),
                }
            }

            // chiwawa has no validator, so the module-rejection directives
            // cannot be checked.
            WastDirective::AssertInvalid { .. }
            | WastDirective::AssertMalformed { .. }
            | WastDirective::AssertUnlinkable { .. } => tally.skipped += 1,

            _ => tally.skipped += 1,
        }
    }

    println!(
        "atomic.wast: {} passed, {} skipped, {} failed",
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
