use anyhow::{Result};
use clap::Parser;
use PPWasm::{parser, module::*};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,
}


fn main() -> Result <()>{
    let args = Args::parse();
    let module = Module::new("test");
    let _ = parser::parse_bytecode(module, &args.path);
    Ok(())
}
