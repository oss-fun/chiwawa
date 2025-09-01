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
use rustc_hash::FxHashMap;
use std::rc::Rc;

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
    /// Additional arguments to pass to WASM application (argv[1], argv[2], ...)
    /// Example: --app-args "--database test.db --iterations 1000"
    #[arg(long, allow_hyphen_values = true)]
    app_args: Option<String>,
    /// Enable superinstructions optimizations (const + local.set)
    #[arg(long, default_value = "false")]
    enable_superinstructions: bool,
    /// Enable memoization optimizations for pure instructions
    #[arg(long, default_value = "false")]
    enable_memoization: bool,
    /// Enable statistics output
    #[arg(long, default_value = "false")]
    enable_stats: bool,
    /// Enable checkpoint/restore
    #[arg(long = "cr", default_value = "false")]
    enable_checkpoint: bool,
}

fn parse_args_string(args: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;
    let mut chars = args.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            ' ' if !in_quotes => {
                if !current_arg.is_empty() {
                    result.push(current_arg.clone());
                    current_arg.clear();
                }
            }
            _ => {
                current_arg.push(ch);
            }
        }
    }

    if !current_arg.is_empty() {
        result.push(current_arg);
    }

    result
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
    let _ = parser::parse_bytecode(&mut module, &cli.wasm_file, cli.enable_superinstructions);
    let imports: ImportObjects = FxHashMap::default();

    let mut wasm_argv = vec![cli.wasm_file.clone()];
    if let Some(args_string) = cli.app_args {
        let additional_args = parse_args_string(&args_string);
        wasm_argv.extend(additional_args);
    }

    let inst = ModuleInst::new(&module, imports, wasm_argv).unwrap();

    if let Some(restore_path) = cli.restore {
        println!("Restoring from checkpoint: {}", restore_path);

        let restored_stacks: Stacks = match migration::restore(Rc::clone(&inst), &restore_path) {
            Ok(stacks) => stacks,
            Err(e) => {
                eprintln!("Failed to restore state: {:?}", e);
                return Err(anyhow::anyhow!("Restore failed: {:?}", e));
            }
        };
        println!("State restored into module instance. Stacks obtained.");

        let mut runtime = Runtime::new_restored(
            Rc::clone(&inst),
            restored_stacks,
            cli.enable_memoization,
            cli.enable_stats,
            cli.enable_checkpoint,
        );
        println!("Runtime reconstructed. Resuming execution...");

        let result = runtime.run();
        handle_result(result);
    } else {
        let func_addr = inst.get_export_func(&cli.invoke)?;
        let params = parse_params(cli.params.unwrap_or_default());

        match Runtime::new(
            Rc::clone(&inst),
            &func_addr,
            params,
            cli.enable_memoization,
            cli.enable_stats,
            cli.enable_checkpoint,
        ) {
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
