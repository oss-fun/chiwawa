use anyhow::{Result};
use clap::Parser;
use PPWasm::{parser,structure::module::Module,execution::module::*};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,
}


fn main() -> Result <()>{
    let args = Args::parse();
    let mut module = Module::new("test");
    let _ = parser::parse_bytecode(&mut module, &args.path);
    let inst = ModuleInst::new(&module);
    Ok(())
}
