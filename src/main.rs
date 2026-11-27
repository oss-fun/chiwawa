use anyhow::Result;
#[cfg(feature = "trace")]
use chiwawa::instrument::trace::TraceConfig;
use chiwawa::{
    execution::canon::ComponentVal,
    execution::component::ComponentInst,
    execution::hook::{HookPoint, Hooks},
    execution::mem::MemAddr,
    execution::migration,
    execution::module::*,
    execution::runtime::{run_start_section, Runtime, RuntimeConfig},
    execution::text,
    execution::value::*,
    parser,
    structure::component::Component,
    structure::module::{ImportDesc, Module},
    wasi::descriptors,
};
#[cfg(feature = "threads")]
use chiwawa::{shared::Shared, wasi::threads::ThreadContext};
use clap::Parser;
use rustc_hash::FxHashMap;
use std::rc::Rc;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// WebAssembly file to execute
    wasm_file: String,
    #[arg(long)]
    restore: Option<String>,
    /// Call the guest export of a hook point (needs --restore)
    #[arg(long, value_enum, requires = "restore")]
    hook: Vec<HookPoint>,
    /// Exported function to call
    #[arg(short, long, default_value = "_start")]
    invoke: String,
    /// Parameters of the function: (i32.const 5) (i64.const 6) for a core module, (s32.const 5) (str.const "a") for a component
    #[arg(short, long, num_args = 0..)]
    params: Option<Vec<String>>,
    /// Additional arguments to pass to WASM application (argv[1], argv[2], ...)
    /// Example: --app-args "--database test.db --iterations 1000"
    #[arg(long, allow_hyphen_values = true)]
    app_args: Option<String>,
    /// Enable statistics output
    #[arg(long = "stats", default_value = "false")]
    enable_stats: bool,
    /// Enable checkpoint/restore
    #[arg(long = "cr", default_value = "false")]
    enable_checkpoint: bool,
    /// Enable wasi-threads (requires a host runtime with thread support)
    #[arg(long = "threads", default_value = "false")]
    enable_threads: bool,
    /// Enable trace output
    #[arg(long = "trace", default_value = "false")]
    enable_trace: bool,
    /// Trace events to monitor (all,store,load,call,branch)
    #[arg(long = "trace-events", value_delimiter = ',', num_args = 0..)]
    trace_events: Option<Vec<String>>,
    /// Resources to trace (pc,regs,memory,globals)
    #[arg(long = "trace-resource", value_delimiter = ',', num_args = 0..)]
    trace_resource: Option<Vec<String>>,
    /// Trace output destination (defaults to stderr)
    #[arg(long = "trace-output")]
    trace_output: Option<String>,
    /// Call graph output file in DOT format
    #[arg(long = "call-graph-output")]
    call_graph_output: Option<String>,
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

fn host_imports(module: &Module) -> ImportObjects {
    let mut imports: ImportObjects = FxHashMap::default();
    for import in &module.imports {
        if let ImportDesc::Mem(mem_type) = &import.desc {
            imports.entry(import.module.0.clone()).or_default().insert(
                import.name.0.clone(),
                Externval::Mem(MemAddr::new(mem_type)),
            );
        }
    }
    imports
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    #[cfg(not(feature = "threads"))]
    if cli.enable_threads {
        eprintln!(
            "Warning: --threads flag is ignored because the 'threads' feature is not enabled."
        );
        eprintln!(
            "         Rebuild with: cargo build --target wasm32-wasip1-threads --features threads"
        );
    }

    #[cfg(not(feature = "stats"))]
    if cli.enable_stats {
        eprintln!("Warning: --stats flag is ignored because the 'stats' feature is not enabled.");
        eprintln!("         Rebuild with: cargo build --features stats");
    }

    #[cfg(all(feature = "stats", feature = "tco"))]
    if cli.enable_stats {
        eprintln!("Warning: --stats is ignored because the 'tco' feature is enabled.");
        eprintln!("         Statistics are only collected by the non-tco dispatcher.");
    }

    #[cfg(not(feature = "trace"))]
    if cli.enable_trace {
        eprintln!("Warning: --trace flag is ignored because the 'trace' feature is not enabled.");
        eprintln!("         Rebuild with: cargo build --features trace");
    }

    #[cfg(all(feature = "trace", feature = "tco"))]
    if cli.enable_trace {
        eprintln!("Warning: --trace is ignored because the 'tco' feature is enabled.");
        eprintln!("         Tracing is only performed by the non-tco dispatcher.");
    }

    #[cfg(not(feature = "call_graph"))]
    if cli.call_graph_output.is_some() {
        eprintln!("Warning: --call-graph-output is ignored because the 'call_graph' feature is not enabled.");
        eprintln!("         Rebuild with: cargo build --features call_graph");
    }

    let (module, _parse_out) = match parser::parse_file(&cli.wasm_file) {
        Ok(parser::Parsed::Module(module, out)) => (module, out),
        Ok(parser::Parsed::Component(component)) => return run_component(cli, &component),
        Err(e) => return Err(anyhow::anyhow!("failed to parse {}: {e}", cli.wasm_file)),
    };

    #[cfg(feature = "threads")]
    let module = Shared::new(module);

    #[cfg(feature = "call_graph")]
    if let Some(path) = &cli.call_graph_output {
        if let Err(e) = _parse_out.call_graph.report(&module, path) {
            eprintln!("Warning: failed to write call graph: {}", e);
        }
    }

    let mut wasm_argv = vec![cli.wasm_file.clone()];
    if let Some(args_string) = cli.app_args {
        let additional_args = parse_args_string(&args_string);
        wasm_argv.extend(additional_args);
    }

    #[cfg(feature = "threads")]
    let thread_ctx = if cli.enable_threads {
        let ctx = ThreadContext::new(Shared::clone(&module), wasm_argv.clone());
        if ctx.is_none() {
            eprintln!("Warning: --threads is ignored because the module has no shared memory for threads to share.");
        }
        ctx
    } else {
        None
    };

    #[cfg(feature = "threads")]
    let inst = match thread_ctx.as_ref() {
        Some(ctx) => ctx.instantiate().unwrap(),
        None => ModuleInst::new(&module, host_imports(&module), wasm_argv).unwrap(),
    };
    #[cfg(not(feature = "threads"))]
    let inst = ModuleInst::new(&module, host_imports(&module), wasm_argv).unwrap();

    let hooks = Hooks::new(&cli.hook, &inst)?;

    #[cfg(feature = "trace")]
    let trace_config = if cli.enable_trace {
        Some(TraceConfig::new(
            cli.trace_events,
            cli.trace_resource,
            cli.trace_output,
        ))
    } else {
        None
    };

    let runtime_config = RuntimeConfig {
        enable_checkpoint: cli.enable_checkpoint,
        #[cfg(feature = "stats")]
        enable_stats: cli.enable_stats,
        #[cfg(feature = "trace")]
        trace_config,
        #[cfg(feature = "threads")]
        thread_ctx: thread_ctx.clone(),
    };

    migration::thread_started();

    if let Some(restore_path) = cli.restore {
        println!("Restoring from checkpoint: {}", restore_path);

        let state = match migration::load_checkpoint(&restore_path) {
            Ok(state) => state,
            Err(e) => {
                eprintln!("Failed to restore state: {:?}", e);
                return Err(anyhow::anyhow!("Restore failed: {:?}", e));
            }
        };
        migration::restore_memory(&inst, &state)?;
        descriptors::restore(&state.descriptors);

        let mut threads = state.threads;
        let Some(main_index) = threads.iter().position(|thread| thread.is_main) else {
            return Err(anyhow::anyhow!(
                "Restore failed: checkpoint has no main thread"
            ));
        };
        let main_thread = threads.remove(main_index);
        let stacks = migration::restore_thread(&inst, &main_thread.data)?;
        hooks.run(HookPoint::AfterRestore)?;

        #[cfg(feature = "threads")]
        if let Some(ctx) = thread_ctx.as_ref() {
            ctx.set_next_tid(state.next_tid);
            for thread in threads {
                ctx.resume(thread)?;
            }
        } else if !threads.is_empty() {
            eprintln!("Warning: --threads is off, so the other saved threads are dropped.");
        }
        #[cfg(not(feature = "threads"))]
        if !threads.is_empty() {
            eprintln!("Warning: this build has no thread support, so the other saved threads are dropped.");
        }

        println!("Runtime reconstructed. Resuming execution...");

        let mut runtime = Runtime::new_restored(Rc::clone(&inst), stacks, runtime_config);
        let result = runtime.run();
        handle_result(result);
    } else {
        run_start_section(&inst)?;

        let func_addr = inst.get_export_func(&cli.invoke)?;
        let params = text::parse_sequence::<Val>(&cli.params.unwrap_or_default().join(" "))?;

        match Runtime::new(Rc::clone(&inst), &func_addr, params, runtime_config) {
            Ok(mut runtime) => {
                let result = runtime.run();
                handle_result(result);
            }
            Err(e) => {
                eprintln!("Runtime initialization failed: {:?}", e);
            }
        }
    }

    migration::thread_finished();
    Ok(())
}

fn run_component(cli: Cli, component: &Component) -> Result<()> {
    if cli.enable_checkpoint || cli.restore.is_some() {
        anyhow::bail!("--cr and --restore are not available for a component");
    }
    if cli.enable_threads {
        anyhow::bail!("--threads is not available for a component");
    }
    if let Some(import) = component.imports.first() {
        anyhow::bail!(
            "{} imports \"{}\"; chiwawa cannot supply imports to a component",
            cli.wasm_file,
            import.name
        );
    }
    let inst = ComponentInst::new(component, Vec::new())
        .map_err(|e| anyhow::anyhow!("instantiation failed: {e:?}"))?;
    let func = inst.get_export_func(&cli.invoke).map_err(|_| {
        anyhow::anyhow!("{} has no exported function {}", cli.wasm_file, cli.invoke)
    })?;
    let params = text::parse_sequence::<ComponentVal>(&cli.params.unwrap_or_default().join(" "))?;
    match func.call(params) {
        Ok(values) => {
            for val in values {
                println!("Result: {val}");
            }
        }
        Err(e) => eprintln!("Execution Error: {:?}", e),
    }
    Ok(())
}

fn handle_result(result: Result<Vec<Val>, chiwawa::error::RuntimeError>) {
    match result {
        Ok(mut values) => {
            if let Some(val) = values.pop() {
                println!("Result: {val}");
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
