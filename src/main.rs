use anyhow::Result;
use chiwawa::{execution::{migration, stack::Stacks}, execution::module::*, execution::value::*, parser, structure::module::Module, execution::runtime::Runtime};
use clap::Parser;
use fancy_regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    restore: Option<String>,
    #[arg(short, long, default_value = "main")]
    invoke: String,
    #[arg(short, long, value_delimiter = ',', num_args = 0..)]
    params: Option<Vec<String>>,
}

fn parse_params(params: Vec<String>) -> Vec<Val> {
    let mut parsed: Vec<Val> = Vec::new();
    let re = Regex::new(r"(?<=\().*(?=\))").unwrap();
    for param in params {
        if param.contains("I32") {
            let captures = re
                .captures(&param)
                .expect("Error running regex")
                .expect("No match found")
                .get(0)
                .expect("No group");
            parsed.push(Val::Num(Num::I32(
                captures.as_str().parse::<i32>().unwrap(),
            )));
        } else if param.contains("I64") {
            let captures = re
                .captures(&param)
                .expect("Error running regex")
                .expect("No match found")
                .get(0)
                .expect("No group");
            parsed.push(Val::Num(Num::I64(
                captures.as_str().parse::<i64>().unwrap(),
            )));
        } else if param.contains("F32") {
            let captures = re
                .captures(&param)
                .expect("Error running regex")
                .expect("No match found")
                .get(0)
                .expect("No group");
            parsed.push(Val::Num(Num::F32(
                captures.as_str().parse::<f32>().unwrap(),
            )));
        } else if param.contains("F64") {
            let captures = re
                .captures(&param)
                .expect("Error running regex")
                .expect("No match found")
                .get(0)
                .expect("No group");
            parsed.push(Val::Num(Num::F64(
                captures.as_str().parse::<f64>().unwrap(),
            )));
        }
    }
    return parsed;
}

static MODULE_INSTANCE: OnceLock<Arc<ModuleInst>> = OnceLock::new();

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let mut module = Module::new("test");
    let _ = parser::parse_bytecode(&mut module, "pi-Leibniz.wasm");
    let imports: ImportObjects = HashMap::new();
    let inst = ModuleInst::new(&module, imports).unwrap();
    let _ = MODULE_INSTANCE.set(inst);
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(restore_path) = cli.restore {
        println!("Restoring from checkpoint: {}", restore_path);

        let module_inst = MODULE_INSTANCE.get().ok_or_else(|| {
            anyhow::anyhow!("Module instance not initialized by Wizer during restore")
        })?;

        let restored_stacks: Stacks = match migration::restore(Arc::clone(module_inst), &restore_path) {
            Ok(stacks) => stacks,
            Err(e) => {
                eprintln!("Failed to restore state: {:?}", e);
                return Err(anyhow::anyhow!("Restore failed: {:?}", e));
            }
        };
        println!("State restored into module instance. Stacks obtained.");

        let mut runtime = Runtime::new_restored(Arc::clone(module_inst), restored_stacks);
        println!("Runtime reconstructed. Resuming execution...");

        let result = runtime.run();
        handle_result(result);

    } else {
        let module_inst = MODULE_INSTANCE.get().ok_or_else(|| {
            anyhow::anyhow!("Module instance not initialized by Wizer")
        })?;

        let func_addr = module_inst.get_export_func(&cli.invoke)?;
        let params = parse_params(cli.params.unwrap_or_default());

        match Runtime::new(Arc::clone(module_inst), &func_addr, params) {
            Ok(mut runtime) => {
                let result = runtime.run();
                handle_result(result);
            }
            Err(e) => {
                eprintln!("Runtime initialization failed: {:?}", e);
            }
        }
    }

    Ok(())
}

fn handle_result(result: Result<Vec<Val>, chiwawa::error::RuntimeError>) {
     match result {
        Ok(mut values) => {
            if let Some(val) = values.pop() {
                println!("Result: {:?}", val);
            } else {
                println!("Result: (no values returned)");
            }
        }
        Err(chiwawa::error::RuntimeError::CheckpointRequested) => {
            println!("Execution stopped for checkpoint.");
        }
        Err(e) => {
            eprintln!("Execution Error: {:?}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chiwawa::{execution::module::*, execution::value::*, parser, structure::module::Module};
    use std::collections::HashMap;
    use std::sync::Arc;
    // Helper function to load module and get instance
    fn load_instance(wasm_path: &str) -> Arc<ModuleInst> {
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path);
        let imports: ImportObjects = HashMap::new();
        ModuleInst::new(&module, imports).unwrap()
    }

    // --- i32 tests ---
    #[test]
    fn test_i32_add() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("add")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
        let ret = inst
            .get_export_func("add")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -2);
        let ret = inst
            .get_export_func("add")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x3fffffff)), Val::Num(Num::I32(1))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x40000000 as i32
        );
    }

    #[test]
    fn test_i32_sub() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("sub")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("sub")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("sub")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x3fffffff)), Val::Num(Num::I32(-1))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x40000000 as i32
        );
    }

    #[test]
    fn test_i32_mul() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("mul")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("mul")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("mul")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_div_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("div_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("div_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("div_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("div_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(2)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0xc0000000u32 as i32
        );
        let ret = inst
            .get_export_func("div_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(5)), Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
        let ret = inst
            .get_export_func("div_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(5)), Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -2);
        let ret = inst
            .get_export_func("div_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-5)), Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_i32_div_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("div_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("div_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("div_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(2)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x40000000);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000001u32 as i32)),
            Val::Num(Num::I32(1000)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x20c49b);
        let ret = inst
            .get_export_func("div_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(5)), Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
        let ret = inst
            .get_export_func("div_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(5)), Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("div_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-5)), Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("div_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(17)), Val::Num(Num::I32(7))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_i32_rem_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(-1)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("rem_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("rem_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("rem_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("rem_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(-1)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("rem_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(5)), Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("rem_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-5)), Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_i32_rem_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("rem_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("rem_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("rem_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]); // Duplicate?
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("rem_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(-1)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(2)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![
            Val::Num(Num::I32(0x8ff00ff0u32 as i32)),
            Val::Num(Num::I32(0x10001)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x8001);
        let ret = inst
            .get_export_func("rem_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(5)), Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("rem_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-5)), Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("rem_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-5)), Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -5);
    }

    #[test]
    fn test_i32_and() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("and")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("and")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("and")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("and")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("and").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("and").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(-1)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x7fffffffu32 as i32
        );
        let ret = inst.get_export_func("and").unwrap().call(vec![
            Val::Num(Num::I32(0xf0f0ffffu32 as i32)),
            Val::Num(Num::I32(0xfffff0f0u32 as i32)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0xf0f0f0f0u32 as i32
        );
        let ret = inst.get_export_func("and").unwrap().call(vec![
            Val::Num(Num::I32(0xffffffffu32 as i32)),
            Val::Num(Num::I32(0xffffffffu32 as i32)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0xffffffffu32 as i32
        );
    }

    #[test]
    fn test_i32_or() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("or")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("or")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("or")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("or")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("or").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
        let ret = inst.get_export_func("or").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
        let ret = inst.get_export_func("or").unwrap().call(vec![
            Val::Num(Num::I32(0xf0f0ffffu32 as i32)),
            Val::Num(Num::I32(0xfffff0f0u32 as i32)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0xffffffffu32 as i32
        );
    }

    #[test]
    fn test_i32_xor() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("xor")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("xor")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("xor")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("xor")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("xor").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
        let ret = inst.get_export_func("xor").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
        let ret = inst.get_export_func("xor").unwrap().call(vec![
            Val::Num(Num::I32(-1)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x7fffffffu32 as i32
        );
        let ret = inst.get_export_func("xor").unwrap().call(vec![
            Val::Num(Num::I32(0xf0f0ffffu32 as i32)),
            Val::Num(Num::I32(0xfffff0f0u32 as i32)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x0f0f0f0fu32 as i32
        );
    }

    #[test]
    fn test_i32_shl() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("shl")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
        let ret = inst
            .get_export_func("shl")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("shl").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(1)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0xfffffffeu32 as i32
        );
        let ret = inst.get_export_func("shl").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(1)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("shl").unwrap().call(vec![
            Val::Num(Num::I32(0x40000000u32 as i32)),
            Val::Num(Num::I32(1)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
        let ret = inst
            .get_export_func("shl")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(31))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
    }

    #[test]
    fn test_i32_shr_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("shr_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("shr_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("shr_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(1)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x3fffffffu32 as i32
        );
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(1)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0xc0000000u32 as i32
        );
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![
            Val::Num(Num::I32(0x40000000u32 as i32)),
            Val::Num(Num::I32(1)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x20000000u32 as i32
        );
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(31)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_i32_shr_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("shr_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("shr_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("shr_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x7fffffffu32 as i32
        );
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(1)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x3fffffffu32 as i32
        );
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(1)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x40000000u32 as i32
        );
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![
            Val::Num(Num::I32(0x40000000u32 as i32)),
            Val::Num(Num::I32(1)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x20000000u32 as i32
        );
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(31)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_rotl() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("rotl")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
        let ret = inst
            .get_export_func("rotl")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("rotl")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
        let ret = inst
            .get_export_func("rotl")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![
            Val::Num(Num::I32(0xabcd9876u32 as i32)),
            Val::Num(Num::I32(1)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x579b30edu32 as i32
        );
        let ret = inst.get_export_func("rotl").unwrap().call(vec![
            Val::Num(Num::I32(0xfe00dc00u32 as i32)),
            Val::Num(Num::I32(4)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0xe00dc00fu32 as i32
        );
        let ret = inst.get_export_func("rotl").unwrap().call(vec![
            Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),
            Val::Num(Num::I32(5)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x183a5c76u32 as i32
        );
        let ret = inst.get_export_func("rotl").unwrap().call(vec![
            Val::Num(Num::I32(0x00008000u32 as i32)),
            Val::Num(Num::I32(37)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x00100000u32 as i32
        );
        let ret = inst.get_export_func("rotl").unwrap().call(vec![
            Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),
            Val::Num(Num::I32(0xff05u32 as i32)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x183a5c76u32 as i32
        );
        let ret = inst.get_export_func("rotl").unwrap().call(vec![
            Val::Num(Num::I32(0x769abcdfu32 as i32)),
            Val::Num(Num::I32(0xffffffedu32 as i32)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x579beed3u32 as i32
        );
        let ret = inst.get_export_func("rotl").unwrap().call(vec![
            Val::Num(Num::I32(0x769abcdfu32 as i32)),
            Val::Num(Num::I32(0x8000000du32 as i32)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x579beed3u32 as i32
        );
        let ret = inst
            .get_export_func("rotl")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(31))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
        let ret = inst.get_export_func("rotl").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(1)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_rotr() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("rotr")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
        let ret = inst
            .get_export_func("rotr")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("rotr")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
        let ret = inst
            .get_export_func("rotr")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![
            Val::Num(Num::I32(0xff00cc00u32 as i32)),
            Val::Num(Num::I32(1)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x7f806600u32 as i32
        );
        let ret = inst.get_export_func("rotr").unwrap().call(vec![
            Val::Num(Num::I32(0x00080000u32 as i32)),
            Val::Num(Num::I32(4)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x00008000u32 as i32
        );
        let ret = inst.get_export_func("rotr").unwrap().call(vec![
            Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),
            Val::Num(Num::I32(5)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x1d860e97u32 as i32
        );
        let ret = inst.get_export_func("rotr").unwrap().call(vec![
            Val::Num(Num::I32(0x00008000u32 as i32)),
            Val::Num(Num::I32(37)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x00000400u32 as i32
        );
        let ret = inst.get_export_func("rotr").unwrap().call(vec![
            Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),
            Val::Num(Num::I32(0xff05u32 as i32)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0x1d860e97u32 as i32
        );
        let ret = inst.get_export_func("rotr").unwrap().call(vec![
            Val::Num(Num::I32(0x769abcdfu32 as i32)),
            Val::Num(Num::I32(0xffffffedu32 as i32)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0xe6fbb4d5u32 as i32
        );
        let ret = inst.get_export_func("rotr").unwrap().call(vec![
            Val::Num(Num::I32(0x769abcdfu32 as i32)),
            Val::Num(Num::I32(0x8000000du32 as i32)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i32().unwrap(),
            0xe6fbb4d5u32 as i32
        );
        let ret = inst
            .get_export_func("rotr")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(31))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(31)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_clz() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("clz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0xffffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("clz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 32);
        let ret = inst
            .get_export_func("clz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x00008000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 16);
        let ret = inst
            .get_export_func("clz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0xffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 24);
        let ret = inst
            .get_export_func("clz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("clz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 31);
        let ret = inst
            .get_export_func("clz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 30);
        let ret = inst
            .get_export_func("clz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_ctz() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("ctz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("ctz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 32);
        let ret = inst
            .get_export_func("ctz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x00008000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 15);
        let ret = inst
            .get_export_func("ctz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x00010000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 16);
        let ret = inst
            .get_export_func("ctz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 31);
        let ret = inst
            .get_export_func("ctz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_popcnt() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("popcnt")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 32);
        let ret = inst
            .get_export_func("popcnt")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("popcnt")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x00008000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("popcnt")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x80008000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
        let ret = inst
            .get_export_func("popcnt")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 31);
        let ret = inst
            .get_export_func("popcnt")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0xAAAAAAAAu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 16);
        let ret = inst
            .get_export_func("popcnt")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x55555555u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 16);
        let ret = inst
            .get_export_func("popcnt")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0xDEADBEEFu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 24);
    }

    #[test]
    fn test_i32_eqz() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("eqz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("eqz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("eqz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("eqz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("eqz")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0xffffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_eq() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("eq")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("eq")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("eq")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("eq").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("eq")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("eq")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("eq")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![
            Val::Num(Num::I32(0)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(-1)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![
            Val::Num(Num::I32(-1)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_ne() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("ne")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("ne")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("ne")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ne").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("ne")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("ne")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("ne")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![
            Val::Num(Num::I32(0)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(-1)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![
            Val::Num(Num::I32(-1)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_lt_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("lt_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("lt_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("lt_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("lt_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("lt_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("lt_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![
            Val::Num(Num::I32(0)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(-1)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![
            Val::Num(Num::I32(-1)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_lt_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("lt_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("lt_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("lt_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("lt_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("lt_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("lt_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![
            Val::Num(Num::I32(0)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(-1)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![
            Val::Num(Num::I32(-1)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_le_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("le_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("le_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("le_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("le_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("le_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("le_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![
            Val::Num(Num::I32(0)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(-1)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![
            Val::Num(Num::I32(-1)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_le_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("le_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("le_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("le_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("le_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("le_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("le_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![
            Val::Num(Num::I32(0)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(-1)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![
            Val::Num(Num::I32(-1)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_gt_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("gt_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("gt_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("gt_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("gt_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("gt_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("gt_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![
            Val::Num(Num::I32(0)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(-1)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![
            Val::Num(Num::I32(-1)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_gt_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("gt_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("gt_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("gt_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("gt_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("gt_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("gt_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![
            Val::Num(Num::I32(0)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(-1)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![
            Val::Num(Num::I32(-1)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_ge_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("ge_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("ge_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("ge_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("ge_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("ge_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("ge_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![
            Val::Num(Num::I32(0)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(-1)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![
            Val::Num(Num::I32(-1)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_ge_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("ge_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("ge_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("ge_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("ge_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("ge_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("ge_u")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![
            Val::Num(Num::I32(0)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(-1)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![
            Val::Num(Num::I32(-1)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![
            Val::Num(Num::I32(0x80000000u32 as i32)),
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![
            Val::Num(Num::I32(0x7fffffffu32 as i32)),
            Val::Num(Num::I32(0x80000000u32 as i32)),
        ]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_extend8_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("extend8_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("extend8_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x7f))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 127);
        let ret = inst
            .get_export_func("extend8_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x80))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -128);
        let ret = inst
            .get_export_func("extend8_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0xff))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
        let ret = inst
            .get_export_func("extend8_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x01234500u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("extend8_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0xfedcba80u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -0x80);
        let ret = inst
            .get_export_func("extend8_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_i32_extend16_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst
            .get_export_func("extend16_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("extend16_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x7fffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 32767);
        let ret = inst
            .get_export_func("extend16_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x8000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -32768);
        let ret = inst
            .get_export_func("extend16_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0xffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
        let ret = inst
            .get_export_func("extend16_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x01230000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst
            .get_export_func("extend16_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0xfedc8000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -0x8000);
        let ret = inst
            .get_export_func("extend16_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_i64_add() {
        let inst = load_instance("test/i64.wasm");
        let ret = inst
            .get_export_func("add")
            .unwrap()
            .call(vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 2);
        let ret = inst
            .get_export_func("add")
            .unwrap()
            .call(vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
        let ret = inst
            .get_export_func("add")
            .unwrap()
            .call(vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), -2);
        let ret = inst
            .get_export_func("add")
            .unwrap()
            .call(vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 0);
        let ret = inst.get_export_func("add").unwrap().call(vec![
            Val::Num(Num::I64(0x3fffffffu64 as i64)),
            Val::Num(Num::I64(1)),
        ]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i64().unwrap(),
            0x40000000u64 as i64
        );
    }

    #[test]
    fn test_i64_extend_i32_s() {
        let inst = load_instance("test/conversions.wasm");
        let ret = inst
            .get_export_func("i64.extend_i32_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 0);
        let ret = inst
            .get_export_func("i64.extend_i32_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(10000))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 10000);
        let ret = inst
            .get_export_func("i64.extend_i32_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-10000))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), -10000);
        let ret = inst
            .get_export_func("i64.extend_i32_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), -1);
        let ret = inst
            .get_export_func("i64.extend_i32_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x7fffffff))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i64().unwrap(),
            0x000000007fffffff
        );
        let ret = inst
            .get_export_func("i64.extend_i32_s")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_i64().unwrap(),
            0xffffffff80000000u64 as i64
        );
    }

    #[test]
    fn test_call_type_i32() {
        let inst = load_instance("test/call.wasm");
        let ret = inst.get_export_func("type-i32").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x132);
    }

    #[test]
    fn test_call_fac() {
        let inst = load_instance("test/call.wasm");
        let ret = inst
            .get_export_func("fac")
            .unwrap()
            .call(vec![Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
        let ret = inst
            .get_export_func("fac")
            .unwrap()
            .call(vec![Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
        let ret = inst
            .get_export_func("fac")
            .unwrap()
            .call(vec![Val::Num(Num::I64(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 120);
    }

    // --- call_indirect tests ---

    #[test]
    fn test_call_indirect_type_i32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("type-i32").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x132);
    }

    #[test]
    fn test_call_indirect_type_i64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("type-i64").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 0x164);
    }

    #[test]
    fn test_call_indirect_type_f32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("type-f32").unwrap().call(vec![]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(),
            3890.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_type_f64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("type-f64").unwrap().call(vec![]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(),
            3940.0_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_type_index() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("type-index").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 100);
    }

    #[test]
    fn test_call_indirect_type_first_i32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("type-first-i32").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 32);
    }

    #[test]
    fn test_call_indirect_type_first_i64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("type-first-i64").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 64);
    }

    #[test]
    fn test_call_indirect_type_first_f32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("type-first-f32").unwrap().call(vec![]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(),
            1.32_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_type_first_f64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("type-first-f64").unwrap().call(vec![]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(),
            1.64_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_type_second_i32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("type-second-i32")
            .unwrap()
            .call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 32);
    }

    #[test]
    fn test_call_indirect_type_second_i64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("type-second-i64")
            .unwrap()
            .call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 64);
    }

    #[test]
    fn test_call_indirect_type_second_f32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("type-second-f32")
            .unwrap()
            .call(vec![]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(),
            32.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_type_second_f64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("type-second-f64")
            .unwrap()
            .call(vec![]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(),
            64.1_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_type_all_i32_i64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("type-all-i32-i64")
            .unwrap()
            .call(vec![])
            .unwrap();
        assert_eq!(ret.len(), 2);
        assert_eq!(ret[0].to_i64().unwrap(), 2);
        assert_eq!(ret[1].to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_dispatch() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("dispatch")
            .unwrap()
            .call(vec![Val::Num(Num::I32(5)), Val::Num(Num::I64(64))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 64);
        let ret = inst
            .get_export_func("dispatch")
            .unwrap()
            .call(vec![Val::Num(Num::I32(5)), Val::Num(Num::I64(123))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 123);
    }

    #[test]
    fn test_call_indirect_dispatch_structural_i64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("dispatch-structural-i64")
            .unwrap()
            .call(vec![Val::Num(Num::I32(20))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 9);
    }

    #[test]
    fn test_call_indirect_dispatch_structural_i32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("dispatch-structural-i32")
            .unwrap()
            .call(vec![Val::Num(Num::I32(19))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_call_indirect_dispatch_structural_f32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("dispatch-structural-f32")
            .unwrap()
            .call(vec![Val::Num(Num::I32(21))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(),
            9.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_dispatch_structural_f64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("dispatch-structural-f64")
            .unwrap()
            .call(vec![Val::Num(Num::I32(22))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(),
            9.0_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_fac_i64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("fac-i64")
            .unwrap()
            .call(vec![Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
        let ret = inst
            .get_export_func("fac-i64")
            .unwrap()
            .call(vec![Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
        let ret = inst
            .get_export_func("fac-i64")
            .unwrap()
            .call(vec![Val::Num(Num::I64(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 120);
        let ret = inst
            .get_export_func("fac-i64")
            .unwrap()
            .call(vec![Val::Num(Num::I64(10))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 3628800);
    }

    #[test]
    fn test_call_indirect_fib_i64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("fib-i64")
            .unwrap()
            .call(vec![Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
        let ret = inst
            .get_export_func("fib-i64")
            .unwrap()
            .call(vec![Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
        let ret = inst
            .get_export_func("fib-i64")
            .unwrap()
            .call(vec![Val::Num(Num::I64(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 2);
        let ret = inst
            .get_export_func("fib-i64")
            .unwrap()
            .call(vec![Val::Num(Num::I64(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 8);
        let ret = inst
            .get_export_func("fib-i64")
            .unwrap()
            .call(vec![Val::Num(Num::I64(10))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 89);
    }

    #[test]
    fn test_call_indirect_fac_i32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("fac-i32")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("fac-i32")
            .unwrap()
            .call(vec![Val::Num(Num::I32(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 120);
    }

    #[test]
    fn test_call_indirect_fac_f32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("fac-f32")
            .unwrap()
            .call(vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(),
            1.0_f32.to_bits()
        );
        let ret = inst
            .get_export_func("fac-f32")
            .unwrap()
            .call(vec![Val::Num(Num::F32(5.0))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(),
            120.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_fac_f64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("fac-f64")
            .unwrap()
            .call(vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(),
            1.0_f64.to_bits()
        );
        let ret = inst
            .get_export_func("fac-f64")
            .unwrap()
            .call(vec![Val::Num(Num::F64(5.0))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(),
            120.0_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_fib_i32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("fib-i32")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst
            .get_export_func("fib-i32")
            .unwrap()
            .call(vec![Val::Num(Num::I32(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 8);
    }

    #[test]
    fn test_call_indirect_fib_f32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("fib-f32")
            .unwrap()
            .call(vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(),
            1.0_f32.to_bits()
        );
        let ret = inst
            .get_export_func("fib-f32")
            .unwrap()
            .call(vec![Val::Num(Num::F32(5.0))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(),
            8.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_fib_f64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("fib-f64")
            .unwrap()
            .call(vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(),
            1.0_f64.to_bits()
        );
        let ret = inst
            .get_export_func("fib-f64")
            .unwrap()
            .call(vec![Val::Num(Num::F64(5.0))]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(),
            8.0_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_even_odd() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("even")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 44);
        let ret = inst
            .get_export_func("even")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 99);
        let ret = inst
            .get_export_func("even")
            .unwrap()
            .call(vec![Val::Num(Num::I32(100))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 44);
        let ret = inst
            .get_export_func("even")
            .unwrap()
            .call(vec![Val::Num(Num::I32(77))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 99);
        let ret = inst
            .get_export_func("odd")
            .unwrap()
            .call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 99);
        let ret = inst
            .get_export_func("odd")
            .unwrap()
            .call(vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 44);
        let ret = inst
            .get_export_func("odd")
            .unwrap()
            .call(vec![Val::Num(Num::I32(200))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 99);
        let ret = inst
            .get_export_func("odd")
            .unwrap()
            .call(vec![Val::Num(Num::I32(77))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 44);
    }

    #[test]
    fn test_call_indirect_as_select_first() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-select-first")
            .unwrap()
            .call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x132);
    }

    #[test]
    fn test_call_indirect_as_select_mid() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("as-select-mid").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_call_indirect_as_select_last() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("as-select-last").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_call_indirect_as_if_condition() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-if-condition")
            .unwrap()
            .call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_as_br_if_first() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("as-br_if-first").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 0x164);
    }

    #[test]
    fn test_call_indirect_as_br_if_last() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("as-br_if-last").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_call_indirect_as_br_table_first() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-br_table-first")
            .unwrap()
            .call(vec![]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(),
            3890.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_as_br_table_last() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-br_table-last")
            .unwrap()
            .call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_call_indirect_as_store_first() {
        let inst = load_instance("test/call_indirect.wasm");
        let result = inst.get_export_func("as-store-first").unwrap().call(vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_indirect_as_store_last() {
        let inst = load_instance("test/call_indirect.wasm");
        let result = inst.get_export_func("as-store-last").unwrap().call(vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_indirect_as_memory_grow_value() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-memory.grow-value")
            .unwrap()
            .call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_as_return_value() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-return-value")
            .unwrap()
            .call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_as_drop_operand() {
        let inst = load_instance("test/call_indirect.wasm");
        let result = inst
            .get_export_func("as-drop-operand")
            .unwrap()
            .call(vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_indirect_as_br_value() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("as-br-value").unwrap().call(vec![]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(),
            1.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_as_local_set_value() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-local.set-value")
            .unwrap()
            .call(vec![]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(),
            1.0_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_as_local_tee_value() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-local.tee-value")
            .unwrap()
            .call(vec![]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(),
            1.0_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_as_global_set_value() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-global.set-value")
            .unwrap()
            .call(vec![]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(),
            1.0_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_as_load_operand() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-load-operand")
            .unwrap()
            .call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_call_indirect_as_unary_operand() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-unary-operand")
            .unwrap()
            .call(vec![]);
        assert_eq!(
            ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(),
            0.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_as_binary_left() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("as-binary-left").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 11);
    }

    #[test]
    fn test_call_indirect_as_binary_right() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-binary-right")
            .unwrap()
            .call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_call_indirect_as_test_operand() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-test-operand")
            .unwrap()
            .call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_call_indirect_as_compare_left() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-compare-left")
            .unwrap()
            .call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_as_compare_right() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-compare-right")
            .unwrap()
            .call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_as_convert_operand() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst
            .get_export_func("as-convert-operand")
            .unwrap()
            .call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
    }

    #[test]
    fn test_loop_empty() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("empty").unwrap().call(vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_loop_singular() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("singular").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 7);
    }

    #[test]
    fn test_loop_multi() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("multi").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 8);
    }

    #[test]
    fn test_loop_nested() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("nested").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_loop_deep() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("deep").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 150);
    }

    #[test]
    fn test_loop_as_select_first() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-select-first")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_select_mid() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-select-mid").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop_as_select_last() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-select-last").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop_as_if_condition() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-if-condition")
            .unwrap()
            .call(vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_loop_as_if_then() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-if-then").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_if_else() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-if-else").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop_as_br_if_first() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-br_if-first").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_br_if_last() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-br_if-last").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop_as_br_table_first() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-br_table-first")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_br_table_last() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-br_table-last")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop_as_call_indirect_first() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-call_indirect-first")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_call_indirect_mid() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-call_indirect-mid")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop_as_call_indirect_last() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-call_indirect-last")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_store_first() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-store-first").unwrap().call(vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_loop_as_store_last() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-store-last").unwrap().call(vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_loop_as_memory_grow_value() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-memory.grow-value")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_call_value() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-call-value").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_return_value() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-return-value")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_drop_operand() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-drop-operand")
            .unwrap()
            .call(vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_loop_as_br_value() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-br-value").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_local_set_value() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-local.set-value")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_local_tee_value() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-local.tee-value")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_global_set_value() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-global.set-value")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_load_operand() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-store-first").unwrap().call(vec![]);
        let result = inst.get_export_func("as-store-last").unwrap().call(vec![]);
        let result = inst
            .get_export_func("as-load-operand")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_unary_operand() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-unary-operand")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_loop_as_binary_operand() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-binary-operand")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 12);
    }

    #[test]
    fn test_loop_as_test_operand() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-test-operand")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_loop_as_compare_operand() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-compare-operand")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_loop_as_binary_operands() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-binary-operands")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 12);
    }

    #[test]
    fn test_loop_as_compare_operands() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-compare-operands")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_loop_as_mixed_operands() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("as-mixed-operands")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 27);
    }

    #[test]
    fn test_loop_break_bare() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("break-bare").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 19);
    }

    #[test]
    fn test_loop_break_value() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("break-value").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 18);
    }

    #[test]
    fn test_loop_break_repeated() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("break-repeated").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 18);
    }

    #[test]
    fn test_loop_break_inner() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("break-inner").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 0x1f);
    }

    #[test]
    fn test_loop_param() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("param").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_loop_params() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("params").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_loop_params_id() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("params-id").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_loop_param_break() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("param-break").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 13);
    }

    #[test]
    fn test_loop_params_break() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("params-break").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 12);
    }

    #[test]
    fn test_loop_params_id_break() {
        let inst = load_instance("test/loop.wasm");
        let result = inst
            .get_export_func("params-id-break")
            .unwrap()
            .call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_loop_effects() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("effects").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }
}
