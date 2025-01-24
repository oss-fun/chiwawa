use anyhow::{Result};
use clap::Parser;
use maplit::hashmap;
use chiwawa::{parser,structure::module::Module,execution::module::*, execution::value::*, execution::func::*};
use std::collections::HashMap;


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

    let mut imports: ImportObjects = HashMap::new();

    let inst = ModuleInst::new(&module, imports).unwrap();
    let params: Vec<Val> = vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0))];
    let ret = inst.get_export_func("main")?.call(params);
    //println!("pi", ret?.pop().unwrap().to_i32());
    Ok(())
}
