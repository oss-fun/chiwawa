use anyhow::{Result};
use clap::Parser;
use maplit::hashmap;
use chiwawa::{parser,structure::module::Module,execution::module::*, execution::value::*};

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
    let inst = ModuleInst::new(&module, hashmap!{}).unwrap();
    let params: Vec<Val> = vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))];
    let mut ret = inst.get_export_func("add")?.call(params);
    println!("1+1 = {}",ret.pop().unwrap().to_i32());
    Ok(())
}
