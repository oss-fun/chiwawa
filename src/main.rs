use anyhow::Result;
use chiwawa::{
    execution::module::*,
    execution::runtime::Runtime,
    execution::value::*,
    execution::{migration, stack::Stacks},
    parser,
    structure::module::Module,
};
use clap::Parser;
use fancy_regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// WebAssembly file to execute
    wasm_file: String,
    #[arg(long)]
    restore: Option<String>,
    #[arg(short, long, default_value = "_start")]
    invoke: String,
    #[arg(short, long, value_delimiter = ',', num_args = 0..)]
    params: Option<Vec<String>>,
    /// Preopen directories for WASI (format: "/path/to/dir")
    #[arg(long, value_delimiter = ',', num_args = 0..)]
    preopen: Option<Vec<String>>,
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut module = Module::new("test");
    let _ = parser::parse_bytecode(&mut module, &cli.wasm_file);
    let imports: ImportObjects = HashMap::new();
    let preopen_dirs = cli.preopen.unwrap_or_default();
    let inst = ModuleInst::new(&module, imports, preopen_dirs).unwrap();

    if let Some(restore_path) = cli.restore {
        println!("Restoring from checkpoint: {}", restore_path);

        let restored_stacks: Stacks = match migration::restore(Arc::clone(&inst), &restore_path) {
            Ok(stacks) => stacks,
            Err(e) => {
                eprintln!("Failed to restore state: {:?}", e);
                return Err(anyhow::anyhow!("Restore failed: {:?}", e));
            }
        };
        println!("State restored into module instance. Stacks obtained.");

        let mut runtime = Runtime::new_restored(Arc::clone(&inst), restored_stacks);
        println!("Runtime reconstructed. Resuming execution...");

        let result = runtime.run();
        handle_result(result);
    } else {
        let func_addr = inst.get_export_func(&cli.invoke)?;
        let params = parse_params(cli.params.unwrap_or_default());

        match Runtime::new(Arc::clone(&inst), &func_addr, params) {
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
