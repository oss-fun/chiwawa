use anyhow::{Result};
use clap::{Args, Parser, Subcommand};
use maplit::hashmap;
use chiwawa::{parser,structure::module::Module,execution::module::*, execution::value::*, execution::func::*};
use std::collections::HashMap;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    path: String,
    #[arg(short, long, default_value = "main")]
    invoke: String,
}


fn main() -> Result <()>{
    let args = Cli::parse();
    let mut module = Module::new("test");
    let _ = parser::parse_bytecode(&mut module, &args.path);        
    let mut imports: ImportObjects = HashMap::new();
    let inst = ModuleInst::new(&module, imports).unwrap();
    let params: Vec<Val> = vec![];
    let ret = inst.get_export_func(&args.invoke)?.call(params);
    println!("pi{}", ret?.pop().unwrap().to_f64());
    Ok(())
}
