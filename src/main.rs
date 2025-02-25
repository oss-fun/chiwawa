use anyhow::{Result};
use clap::{Args, Parser, Subcommand};
use maplit::hashmap;
use chiwawa::{parser,structure::module::Module,execution::module::*, execution::value::*, execution::func::*};
use std::collections::HashMap;
use fancy_regex::Regex;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    path: String,
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
            let captures = re.captures(&param).expect("Error running regex").expect("No match found").get(0).expect("No group");
            parsed.push(Val::Num(Num::I32(captures.as_str().parse::<i32>().unwrap())));
        } else if param.contains("I64") {
            let captures = re.captures(&param).expect("Error running regex").expect("No match found").get(0).expect("No group");
            parsed.push(Val::Num(Num::I64(captures.as_str().parse::<i64>().unwrap())));
        } else if param.contains("F32") {
            let captures = re.captures(&param).expect("Error running regex").expect("No match found").get(0).expect("No group");
            parsed.push(Val::Num(Num::F32(captures.as_str().parse::<f32>().unwrap())));
        } else if param.contains("F64") {
            let captures = re.captures(&param).expect("Error running regex").expect("No match found").get(0).expect("No group");
            parsed.push(Val::Num(Num::F64(captures.as_str().parse::<f64>().unwrap())));
        }
    }
    return parsed
}
fn main() -> Result <()>{
    let cli = Cli::parse();
    let params: Vec<Val> = match cli.params{
        Some(p) => parse_params(p),
        None => vec![]
    };
    let mut module = Module::new("test");
    let _ = parser::parse_bytecode(&mut module, &cli.path);        
    let mut imports: ImportObjects = HashMap::new();
    let inst = ModuleInst::new(&module, imports).unwrap();
    let ret = inst.get_export_func(&cli.invoke)?.call(params);
    println!("pi{}", ret?.pop().unwrap().to_f64());
    Ok(())
}
