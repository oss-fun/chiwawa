use anyhow::{Result};
use clap::{Parser};
use chiwawa::{parser,structure::module::Module,execution::module::*, execution::value::*};
use std::collections::HashMap;
use fancy_regex::Regex;
use std::sync::{Arc, OnceLock};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
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

static MODULE_INSTANCE: OnceLock<Arc<ModuleInst>> = OnceLock::new();

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    let mut module = Module::new("test");
    let _ = parser::parse_bytecode(&mut module, "pi-Leibniz.wasm");
    let imports: ImportObjects = HashMap::new();
    let inst = ModuleInst::new(&module, imports).unwrap();
    let _ = MODULE_INSTANCE.set(inst);
}

fn main() -> Result <()>{
    let cli = Cli::parse();
    let params: Vec<Val> = match cli.params{
        Some(p) => parse_params(p),
        None => vec![]
    };

    let inst = MODULE_INSTANCE.get().unwrap();
    let result = inst.get_export_func(&cli.invoke)?.call(params);

    match result {
        Ok(mut values) => {
            if let Some(val) = values.pop() {
                println!("Result: {:?}", val);
            } else {
                println!("Result: (no values returned)");
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chiwawa::{parser, structure::module::Module, execution::value::*, execution::module::*};
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
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -2);
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I32(0x3fffffff)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x40000000 as i32);
    }

    #[test]
    fn test_i32_sub() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("sub").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("sub").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("sub").unwrap().call(vec![Val::Num(Num::I32(0x3fffffff)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x40000000 as i32);
    }

    #[test]
    fn test_i32_mul() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("mul").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("mul").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("mul").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(),  1);
    }

    #[test]
    fn test_i32_div_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(),  0);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(),  1);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(),  0xc0000000u32 as i32);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(5)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(),  2);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(5)),Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(),  -2);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(-5)),Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(),  2);
    }

    #[test]
    fn test_i32_div_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x40000000);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(0x80000001u32 as i32)),Val::Num(Num::I32(1000))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x20c49b);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(5)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(5)),Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(-5)),Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(17)),Val::Num(Num::I32(7))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_i32_rem_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(5)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(-5)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_i32_rem_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]); // Duplicate?
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x80000000u32 as i32);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(0x8ff00ff0u32 as i32)),Val::Num(Num::I32(0x10001))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x8001);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(5)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(-5)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(-5)),Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -5); // Is this correct for unsigned rem?
    }

    #[test]
    fn test_i32_and() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x7fffffffu32 as i32);
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(0xf0f0ffffu32 as i32)),Val::Num(Num::I32(0xfffff0f0u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0xf0f0f0f0u32 as i32);
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(0xffffffffu32 as i32)),Val::Num(Num::I32(0xffffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0xffffffffu32 as i32);
    }

    #[test]
    fn test_i32_or() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("or").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("or").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("or").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("or").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("or").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
        let ret = inst.get_export_func("or").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x80000000u32 as i32);
        let ret = inst.get_export_func("or").unwrap().call(vec![Val::Num(Num::I32(0xf0f0ffffu32 as i32)),Val::Num(Num::I32(0xfffff0f0u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0xffffffffu32 as i32);
    }

    #[test]
    fn test_i32_xor() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x80000000u32 as i32);
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x7fffffffu32 as i32);
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(0xf0f0ffffu32 as i32)),Val::Num(Num::I32(0xfffff0f0u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x0f0f0f0fu32 as i32);
    }

    #[test]
    fn test_i32_shl() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("shl").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
        let ret = inst.get_export_func("shl").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("shl").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0xfffffffeu32 as i32);
        let ret = inst.get_export_func("shl").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("shl").unwrap().call(vec![Val::Num(Num::I32(0x40000000u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x80000000u32 as i32);
        let ret = inst.get_export_func("shl").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(31))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x80000000u32 as i32);
    }

    #[test]
    fn test_i32_shr_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x3fffffffu32 as i32);
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0xc0000000u32 as i32);
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![Val::Num(Num::I32(0x40000000u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x20000000u32 as i32);
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(31))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_i32_shr_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x7fffffffu32 as i32);
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x3fffffffu32 as i32);
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x40000000u32 as i32);
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![Val::Num(Num::I32(0x40000000u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x20000000u32 as i32);
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(31))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_rotl() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0xabcd9876u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x579b30edu32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0xfe00dc00u32 as i32)),Val::Num(Num::I32(4))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0xe00dc00fu32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),Val::Num(Num::I32(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x183a5c76u32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0x00008000u32 as i32)),Val::Num(Num::I32(37))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x00100000u32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),Val::Num(Num::I32(0xff05u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x183a5c76u32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0x769abcdfu32 as i32)),Val::Num(Num::I32(0xffffffedu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x579beed3u32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0x769abcdfu32 as i32)),Val::Num(Num::I32(0x8000000du32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x579beed3u32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(31))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x80000000u32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_rotr() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x80000000u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0xff00cc00u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x7f806600u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0x00080000u32 as i32)),Val::Num(Num::I32(4))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x00008000u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),Val::Num(Num::I32(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x1d860e97u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0x00008000u32 as i32)),Val::Num(Num::I32(37))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x00000400u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),Val::Num(Num::I32(0xff05u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x1d860e97u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0x769abcdfu32 as i32)),Val::Num(Num::I32(0xffffffedu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0xe6fbb4d5u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0x769abcdfu32 as i32)),Val::Num(Num::I32(0x8000000du32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0xe6fbb4d5u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(31))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(31))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_clz() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(0xffffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 32);
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(0x00008000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 16);
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(0xffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 24);
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 31);
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 30);
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_ctz() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("ctz").unwrap().call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ctz").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 32);
        let ret = inst.get_export_func("ctz").unwrap().call(vec![Val::Num(Num::I32(0x00008000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 15);
        let ret = inst.get_export_func("ctz").unwrap().call(vec![Val::Num(Num::I32(0x00010000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 16);
        let ret = inst.get_export_func("ctz").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 31);
        let ret = inst.get_export_func("ctz").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_popcnt() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 32);
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(0x00008000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(0x80008000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 31);
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(0xAAAAAAAAu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 16);
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(0x55555555u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 16);
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(0xDEADBEEFu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 24);
    }

    #[test]
    fn test_i32_eqz() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("eqz").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("eqz").unwrap().call(vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eqz").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eqz").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eqz").unwrap().call(vec![Val::Num(Num::I32(0xffffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_eq() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_ne() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_lt_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_lt_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_le_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_le_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_gt_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_gt_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_ge_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_ge_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_extend8_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("extend8_s").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("extend8_s").unwrap().call(vec![Val::Num(Num::I32(0x7f))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 127);
        let ret = inst.get_export_func("extend8_s").unwrap().call(vec![Val::Num(Num::I32(0x80))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -128);
        let ret = inst.get_export_func("extend8_s").unwrap().call(vec![Val::Num(Num::I32(0xff))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
        let ret = inst.get_export_func("extend8_s").unwrap().call(vec![Val::Num(Num::I32(0x01234500u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("extend8_s").unwrap().call(vec![Val::Num(Num::I32(0xfedcba80u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -0x80);
        let ret = inst.get_export_func("extend8_s").unwrap().call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_i32_extend16_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("extend16_s").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("extend16_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 32767);
        let ret = inst.get_export_func("extend16_s").unwrap().call(vec![Val::Num(Num::I32(0x8000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -32768);
        let ret = inst.get_export_func("extend16_s").unwrap().call(vec![Val::Num(Num::I32(0xffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
        let ret = inst.get_export_func("extend16_s").unwrap().call(vec![Val::Num(Num::I32(0x01230000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
        let ret = inst.get_export_func("extend16_s").unwrap().call(vec![Val::Num(Num::I32(0xfedc8000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -0x8000);
        let ret = inst.get_export_func("extend16_s").unwrap().call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_i64_add() {
        let inst = load_instance("test/i64.wasm");
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 2);
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), -2);
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 0);
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I64(0x3fffffffu64 as i64)), Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 0x40000000u64 as i64);
    }

    #[test]
    fn test_i64_extend_i32_s() {
        let inst = load_instance("test/conversions.wasm");
        let ret = inst.get_export_func("i64.extend_i32_s").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 0);
        let ret = inst.get_export_func("i64.extend_i32_s").unwrap().call(vec![Val::Num(Num::I32(10000))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 10000);
        let ret = inst.get_export_func("i64.extend_i32_s").unwrap().call(vec![Val::Num(Num::I32(-10000))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), -10000);
        let ret = inst.get_export_func("i64.extend_i32_s").unwrap().call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), -1);
        let ret = inst.get_export_func("i64.extend_i32_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffff))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 0x000000007fffffff);
        let ret = inst.get_export_func("i64.extend_i32_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 0xffffffff80000000u64 as i64);
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
        let ret = inst.get_export_func("fac").unwrap().call(vec![Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
        let ret = inst.get_export_func("fac").unwrap().call(vec![Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
        let ret = inst.get_export_func("fac").unwrap().call(vec![Val::Num(Num::I64(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 120);
    }

    // --- call_indirect tests ---

    #[test]
    fn test_call_indirect_type_i32() {
        let inst = load_instance("test/call_indirect.wasm"); // Changed to .wasm
        let ret = inst.get_export_func("type-i32").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0x132);
    }

    #[test]
    fn test_call_indirect_type_i64() {
        let inst = load_instance("test/call_indirect.wasm"); // Changed to .wasm
        let ret = inst.get_export_func("type-i64").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 0x164);
    }

    #[test]
    fn test_call_indirect_type_f32() {
        let inst = load_instance("test/call_indirect.wasm"); // Changed to .wasm
        let ret = inst.get_export_func("type-f32").unwrap().call(vec![]);
        // Using appropriate float literal for 0xf32
        assert_eq!(ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(), 3890.0_f32.to_bits());
    }

    #[test]
    fn test_call_indirect_type_f64() {
        let inst = load_instance("test/call_indirect.wasm"); // Changed to .wasm
        let ret = inst.get_export_func("type-f64").unwrap().call(vec![]);
         // Using appropriate float literal for 0xf64
        assert_eq!(ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(), 3940.0_f64.to_bits());
    }

    #[test]
    fn test_call_indirect_type_index() {
        let inst = load_instance("test/call_indirect.wasm"); // Changed to .wasm
        let ret = inst.get_export_func("type-index").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 100); // Calls id-i64(100)
    }

    #[test]
    fn test_call_indirect_type_first_i32() {
        let inst = load_instance("test/call_indirect.wasm"); // Changed to .wasm
        let ret = inst.get_export_func("type-first-i32").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 32); // Calls id-i32(32)
    }

    #[test]
    fn test_call_indirect_type_first_i64() {
        let inst = load_instance("test/call_indirect.wasm"); // Changed to .wasm
        let ret = inst.get_export_func("type-first-i64").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 64); // Calls id-i64(64)
    }

    #[test]
    fn test_call_indirect_type_first_f32() {
        let inst = load_instance("test/call_indirect.wasm"); // Changed to .wasm
        let ret = inst.get_export_func("type-first-f32").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(), 1.32_f32.to_bits()); // Calls id-f32(1.32)
    }

    #[test]
    fn test_call_indirect_type_first_f64() {
        let inst = load_instance("test/call_indirect.wasm"); // Changed to .wasm
        let ret = inst.get_export_func("type-first-f64").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(), 1.64_f64.to_bits()); // Calls id-f64(1.64)
    }

    #[test]
    fn test_call_indirect_type_second_i32() {
        let inst = load_instance("test/call_indirect.wasm"); // Changed to .wasm
        let ret = inst.get_export_func("type-second-i32").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 32); // Calls f32-i32(32.1, 32) -> 32
    }

    #[test]
    fn test_call_indirect_type_second_i64() {
        let inst = load_instance("test/call_indirect.wasm"); // Changed to .wasm
        let ret = inst.get_export_func("type-second-i64").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 64); // Calls i32-i64(32, 64) -> 64
    }

    #[test]
    fn test_call_indirect_type_second_f32() {
        let inst = load_instance("test/call_indirect.wasm"); // Changed to .wasm
        let ret = inst.get_export_func("type-second-f32").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(), 32.0_f32.to_bits()); // Calls f64-f32(64, 32) -> 32
    }

    #[test]
    fn test_call_indirect_type_second_f64() {
        let inst = load_instance("test/call_indirect.wasm"); // Changed to .wasm
        let ret = inst.get_export_func("type-second-f64").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(), 64.1_f64.to_bits()); // Calls i64-f64(64, 64.1) -> 64.1
    }

    // Test multiple return values (i64, i32)
    #[test]
    fn test_call_indirect_type_all_i32_i64() {
        let inst = load_instance("test/call_indirect.wasm"); // Changed to .wasm
        let ret = inst.get_export_func("type-all-i32-i64").unwrap().call(vec![]).unwrap();
        assert_eq!(ret.len(), 2);
        // Assuming [i64, i32] order (swap)
        assert_eq!(ret[0].to_i64().unwrap(), 2);
        assert_eq!(ret[1].to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_dispatch() {
        let inst = load_instance("test/call_indirect.wasm");
        // dispatch(5, 64) -> calls id-i64(64) at index 5 -> returns 64
        let ret = inst.get_export_func("dispatch").unwrap().call(vec![Val::Num(Num::I32(5)), Val::Num(Num::I64(64))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 64);
        // dispatch(7, 64.1) -> calls id-f64(64.1) at index 7 -> returns 64.1 (but func expects i64 result) -> Trap expected? Or type mismatch?
        // Let's test a case that should work: dispatch(5, 123) -> id-i64(123) -> 123
        let ret = inst.get_export_func("dispatch").unwrap().call(vec![Val::Num(Num::I32(5)), Val::Num(Num::I64(123))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 123);
    }

    #[test]
    fn test_call_indirect_dispatch_structural_i64() {
        let inst = load_instance("test/call_indirect.wasm");
        // dispatch-structural-i64(20) -> calls over-i64-duplicate(9) at index 20 -> returns 9
        let ret = inst.get_export_func("dispatch-structural-i64").unwrap().call(vec![Val::Num(Num::I32(20))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 9);
    }

    #[test]
    fn test_call_indirect_dispatch_structural_i32() {
        let inst = load_instance("test/call_indirect.wasm");
         // dispatch-structural-i32(19) -> calls over-i32-duplicate(9) at index 19 -> returns 9
        let ret = inst.get_export_func("dispatch-structural-i32").unwrap().call(vec![Val::Num(Num::I32(19))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_call_indirect_dispatch_structural_f32() {
        let inst = load_instance("test/call_indirect.wasm");
        // dispatch-structural-f32(21) -> calls over-f32-duplicate(9.0) at index 21 -> returns 9.0
        let ret = inst.get_export_func("dispatch-structural-f32").unwrap().call(vec![Val::Num(Num::I32(21))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(), 9.0_f32.to_bits());
    }

    #[test]
    fn test_call_indirect_dispatch_structural_f64() {
        let inst = load_instance("test/call_indirect.wasm");
         // dispatch-structural-f64(22) -> calls over-f64-duplicate(9.0) at index 22 -> returns 9.0
        let ret = inst.get_export_func("dispatch-structural-f64").unwrap().call(vec![Val::Num(Num::I32(22))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(), 9.0_f64.to_bits());
    }

    #[test]
    fn test_call_indirect_fac_i64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("fac-i64").unwrap().call(vec![Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
        let ret = inst.get_export_func("fac-i64").unwrap().call(vec![Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
        let ret = inst.get_export_func("fac-i64").unwrap().call(vec![Val::Num(Num::I64(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 120);
        let ret = inst.get_export_func("fac-i64").unwrap().call(vec![Val::Num(Num::I64(10))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 3628800);
    }

    #[test]
    fn test_call_indirect_fib_i64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("fib-i64").unwrap().call(vec![Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
        let ret = inst.get_export_func("fib-i64").unwrap().call(vec![Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 1);
        let ret = inst.get_export_func("fib-i64").unwrap().call(vec![Val::Num(Num::I64(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 2);
        let ret = inst.get_export_func("fib-i64").unwrap().call(vec![Val::Num(Num::I64(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 8);
        let ret = inst.get_export_func("fib-i64").unwrap().call(vec![Val::Num(Num::I64(10))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 89);
    }

     #[test]
    fn test_call_indirect_fac_i32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("fac-i32").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("fac-i32").unwrap().call(vec![Val::Num(Num::I32(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 120);
    }

    #[test]
    fn test_call_indirect_fac_f32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("fac-f32").unwrap().call(vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(), 1.0_f32.to_bits());
        let ret = inst.get_export_func("fac-f32").unwrap().call(vec![Val::Num(Num::F32(5.0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(), 120.0_f32.to_bits());
    }

    #[test]
    fn test_call_indirect_fac_f64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("fac-f64").unwrap().call(vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(), 1.0_f64.to_bits());
        let ret = inst.get_export_func("fac-f64").unwrap().call(vec![Val::Num(Num::F64(5.0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(), 120.0_f64.to_bits());
    }

    #[test]
    fn test_call_indirect_fib_i32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("fib-i32").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
        let ret = inst.get_export_func("fib-i32").unwrap().call(vec![Val::Num(Num::I32(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 8);
    }

     #[test]
    fn test_call_indirect_fib_f32() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("fib-f32").unwrap().call(vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(), 1.0_f32.to_bits());
        let ret = inst.get_export_func("fib-f32").unwrap().call(vec![Val::Num(Num::F32(5.0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(), 8.0_f32.to_bits());
    }

    #[test]
    fn test_call_indirect_fib_f64() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("fib-f64").unwrap().call(vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(), 1.0_f64.to_bits());
        let ret = inst.get_export_func("fib-f64").unwrap().call(vec![Val::Num(Num::F64(5.0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(), 8.0_f64.to_bits());
    }

    #[test]
    fn test_call_indirect_even_odd() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("even").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 44);
        let ret = inst.get_export_func("even").unwrap().call(vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 99);
        let ret = inst.get_export_func("even").unwrap().call(vec![Val::Num(Num::I32(100))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 44);
        let ret = inst.get_export_func("even").unwrap().call(vec![Val::Num(Num::I32(77))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 99);
        let ret = inst.get_export_func("odd").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 99);
        let ret = inst.get_export_func("odd").unwrap().call(vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 44);
        let ret = inst.get_export_func("odd").unwrap().call(vec![Val::Num(Num::I32(200))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 99);
        let ret = inst.get_export_func("odd").unwrap().call(vec![Val::Num(Num::I32(77))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 44);
    }

    #[test]
    fn test_call_indirect_as_select_first() {
        let inst = load_instance("test/call_indirect.wasm");
        let ret = inst.get_export_func("as-select-first").unwrap().call(vec![]);
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
        // select (i32.const 2) (i32.const 3) (call_indirect type$out-i32 (i32.const 0))
        // select (2) (3) (0x132) -> selects 2 because condition (0x132) is non-zero
        let ret = inst.get_export_func("as-select-last").unwrap().call(vec![]);
         // The condition is the result of call_indirect(0) -> 0x132 (non-zero)
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_call_indirect_as_if_condition() {
        let inst = load_instance("test/call_indirect.wasm");
        // if (call_indirect type$out-i32 (i32.const 0)) then 1 else 2
        // if (0x132) then 1 else 2 -> condition is non-zero, takes then branch
        let ret = inst.get_export_func("as-if-condition").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_as_br_if_first() {
        let inst = load_instance("test/call_indirect.wasm");
        // block (result i64) (br_if 0 (call_indirect type$out-i64 (i32.const 1)) (i32.const 2))
        // br_if 0 (0x164) (2) -> condition is non-zero, takes branch with value 0x164
        let ret = inst.get_export_func("as-br_if-first").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64().unwrap(), 0x164);
    }

     #[test]
    fn test_call_indirect_as_br_if_last() {
        let inst = load_instance("test/call_indirect.wasm");
        // block (result i32) (br_if 0 (i32.const 2) (call_indirect type$out-i32 (i32.const 0)))
        // br_if 0 (2) (0x132) -> condition is non-zero (0x132), takes branch with value 2
        let ret = inst.get_export_func("as-br_if-last").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_call_indirect_as_br_table_first() {
        let inst = load_instance("test/call_indirect.wasm");
        // block (result f32) (call_indirect type$out-f32 (i32.const 2)) (i32.const 2) (br_table 0 0)
        // stack: [3890.0, 2] -> br_table 0 0 with index 2 -> branches to default (0) with value 3890.0
        let ret = inst.get_export_func("as-br_table-first").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(), 3890.0_f32.to_bits());
    }

    #[test]
    fn test_call_indirect_as_br_table_last() {
        let inst = load_instance("test/call_indirect.wasm");
        // block (result i32) (i32.const 2) (call_indirect type$out-i32 (i32.const 0)) (br_table 0 0)
        // stack: [2, 0x132] -> br_table 0 0 with index 0x132 -> branches to default (0) with value 2
        let ret = inst.get_export_func("as-br_table-last").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_call_indirect_as_store_first() {
        let inst = load_instance("test/call_indirect.wasm");
        // (call_indirect type$out-i32 (i32.const 0)) (i32.const 1) (i32.store)
        // stack: [0x132, 1] -> i32.store value 1 at address 0x132
        // Just check if it executes without error
        let result = inst.get_export_func("as-store-first").unwrap().call(vec![]);
        assert!(result.is_ok());
        // TODO: Could add memory check if runtime allows reading memory easily
    }

    #[test]
    fn test_call_indirect_as_store_last() {
        let inst = load_instance("test/call_indirect.wasm");
        // (i32.const 10) (call_indirect type$out-f64 (i32.const 3)) (f64.store)
        // stack: [10, 3940.0] -> f64.store value 3940.0 at address 10
        let result = inst.get_export_func("as-store-last").unwrap().call(vec![]);
        assert!(result.is_ok());
         // TODO: Could add memory check
    }

    #[test]
    fn test_call_indirect_as_memory_grow_value() {
        let inst = load_instance("test/call_indirect.wasm");
        // memory.grow (call_indirect type$out-i32 (i32.const 0))
        // memory.grow(0x132) -> grows memory by 0x132 pages, returns previous size (1)
        let ret = inst.get_export_func("as-memory.grow-value").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1); // Assuming initial memory size is 1 page
    }

    #[test]
    fn test_call_indirect_as_return_value() {
        let inst = load_instance("test/call_indirect.wasm");
        // (call_indirect type$over-i32 (i32.const 1) (i32.const 4)) (return)
        // calls id-i32(1) -> returns 1
        let ret = inst.get_export_func("as-return-value").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_as_drop_operand() {
        let inst = load_instance("test/call_indirect.wasm");
        // (call_indirect type$over-i64 (i64.const 1) (i32.const 5)) (drop)
        // calls id-i64(1) -> returns 1, then drops it. Check for successful execution.
        let result = inst.get_export_func("as-drop-operand").unwrap().call(vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_indirect_as_br_value() {
        let inst = load_instance("test/call_indirect.wasm");
        // block (result f32) (br 0 (call_indirect type$over-f32 (f32.const 1) (i32.const 6)))
        // calls id-f32(1.0) -> returns 1.0, branches with this value
        let ret = inst.get_export_func("as-br-value").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(), 1.0_f32.to_bits());
    }

    #[test]
    fn test_call_indirect_as_local_set_value() {
        let inst = load_instance("test/call_indirect.wasm");
        // local f64; local.set 0 (call_indirect type$over-f64 (f64.const 1) (i32.const 7)); local.get 0
        // calls id-f64(1.0) -> returns 1.0, sets local 0 to 1.0, returns local 0
        let ret = inst.get_export_func("as-local.set-value").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(), 1.0_f64.to_bits());
    }

    #[test]
    fn test_call_indirect_as_local_tee_value() {
        let inst = load_instance("test/call_indirect.wasm");
        // local f64; local.tee 0 (call_indirect type$over-f64 (f64.const 1) (i32.const 7))
        // calls id-f64(1.0) -> returns 1.0, sets local 0 to 1.0, returns 1.0
        let ret = inst.get_export_func("as-local.tee-value").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(), 1.0_f64.to_bits());
    }

    #[test]
    fn test_call_indirect_as_global_set_value() {
        let inst = load_instance("test/call_indirect.wasm");
        // global.set $a (call_indirect type$over-f64 (f64.const 1.0) (i32.const 7)); global.get $a
        // calls id-f64(1.0) -> returns 1.0, sets global $a to 1.0, returns global $a
        let ret = inst.get_export_func("as-global.set-value").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f64().unwrap().to_bits(), 1.0_f64.to_bits());
    }

    #[test]
    fn test_call_indirect_as_load_operand() {
        let inst = load_instance("test/call_indirect.wasm");
        // i32.load (call_indirect type$out-i32 (i32.const 0))
        // i32.load(0x132) -> loads i32 from memory address 0x132
        // Assuming memory is zero-initialized or setup elsewhere. Expect 0.
        let ret = inst.get_export_func("as-load-operand").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0); // Expect 0 assuming zero-initialized memory
    }

    #[test]
    fn test_call_indirect_as_unary_operand() {
        let inst = load_instance("test/call_indirect.wasm");
        // f32.sqrt (call_indirect type$over-f32 (f32.const 0x0p+0) (i32.const 6))
        // calls id-f32(0.0) -> returns 0.0; f32.sqrt(0.0) -> 0.0
        let ret = inst.get_export_func("as-unary-operand").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_f32().unwrap().to_bits(), 0.0_f32.to_bits());
    }

    #[test]
    fn test_call_indirect_as_binary_left() {
        let inst = load_instance("test/call_indirect.wasm");
        // i32.add (call_indirect type$over-i32 (i32.const 1) (i32.const 4)) (i32.const 10)
        // calls id-i32(1) -> returns 1; i32.add(1, 10) -> 11
        let ret = inst.get_export_func("as-binary-left").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 11);
    }

    #[test]
    fn test_call_indirect_as_binary_right() {
        let inst = load_instance("test/call_indirect.wasm");
        // i32.sub (i32.const 10) (call_indirect type$over-i32 (i32.const 1) (i32.const 4))
        // calls id-i32(1) -> returns 1; i32.sub(10, 1) -> 9
        let ret = inst.get_export_func("as-binary-right").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_call_indirect_as_test_operand() {
        let inst = load_instance("test/call_indirect.wasm");
        // i32.eqz (call_indirect type$over-i32 (i32.const 1) (i32.const 4))
        // calls id-i32(1) -> returns 1; i32.eqz(1) -> 0
        let ret = inst.get_export_func("as-test-operand").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_call_indirect_as_compare_left() {
        let inst = load_instance("test/call_indirect.wasm");
        // i32.le_u (call_indirect type$over-i32 (i32.const 1) (i32.const 4)) (i32.const 10)
        // calls id-i32(1) -> returns 1; i32.le_u(1, 10) -> 1
        let ret = inst.get_export_func("as-compare-left").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_as_compare_right() {
        let inst = load_instance("test/call_indirect.wasm");
        // i32.ne (i32.const 10) (call_indirect type$over-i32 (i32.const 1) (i32.const 4))
        // calls id-i32(1) -> returns 1; i32.ne(10, 1) -> 1
        let ret = inst.get_export_func("as-compare-right").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_as_convert_operand() {
        let inst = load_instance("test/call_indirect.wasm");
        // i64.extend_i32_s (call_indirect type$over-i32 (i32.const 1) (i32.const 4))
        // calls id-i32(1) -> returns 1; i64.extend_i32_s(1) -> 1
        let ret = inst.get_export_func("as-convert-operand").unwrap().call(vec![]);
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
        let result = inst.get_export_func("as-select-first").unwrap().call(vec![]);
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
        let result = inst.get_export_func("as-if-condition").unwrap().call(vec![]);
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
        let result = inst.get_export_func("as-br_table-first").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_br_table_last() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-br_table-last").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop_as_call_indirect_first() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-call_indirect-first").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_call_indirect_mid() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-call_indirect-mid").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop_as_call_indirect_last() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-call_indirect-last").unwrap().call(vec![]);
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
        let result = inst.get_export_func("as-memory.grow-value").unwrap().call(vec![]);
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
        let result = inst.get_export_func("as-return-value").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_drop_operand() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-drop-operand").unwrap().call(vec![]);
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
        let result = inst.get_export_func("as-local.set-value").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_local_tee_value() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-local.tee-value").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_global_set_value() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-global.set-value").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_load_operand() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-load-operand").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_unary_operand() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-unary-operand").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_loop_as_binary_operand() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-binary-operand").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 12);
    }

    #[test]
    fn test_loop_as_test_operand() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-test-operand").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_loop_as_compare_operand() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-compare-operand").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_loop_as_binary_operands() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-binary-operands").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 12);
    }

    #[test]
    fn test_loop_as_compare_operands() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-compare-operands").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_loop_as_mixed_operands() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("as-mixed-operands").unwrap().call(vec![]);
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
        let result = inst.get_export_func("params-id-break").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_loop_effects() {
        let inst = load_instance("test/loop.wasm");
        let result = inst.get_export_func("effects").unwrap().call(vec![]);
        assert_eq!(result.unwrap().pop().unwrap().to_i32().unwrap(), 1);
    }
}