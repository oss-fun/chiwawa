use anyhow::{Result};
use clap::{Parser};
use chiwawa::{parser,structure::module::Module,execution::module::*, execution::value::*};
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
    use std::rc::Rc; // Import Rc for the return type

    // Helper function to load module and get instance
    fn load_instance(wasm_path: &str) -> Rc<ModuleInst> { // Changed return type
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path);
        let imports: ImportObjects = HashMap::new();
        ModuleInst::new(&module, imports).unwrap() // This already returns Rc<ModuleInst>
    }

    // --- i32 tests ---
    #[test]
    fn test_i32_add() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 2);
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -2);
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I32(0x3fffffff)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x40000000 as i32);
    }

    #[test]
    fn test_i32_sub() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("sub").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("sub").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("sub").unwrap().call(vec![Val::Num(Num::I32(0x3fffffff)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x40000000 as i32);
    }

    #[test]
    fn test_i32_mul() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("mul").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("mul").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("mul").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(),  1);
    }

    #[test]
    fn test_i32_div_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(),  0);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(),  1);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(),  0xc0000000u32 as i32);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(5)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(),  2);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(5)),Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(),  -2);
        let ret = inst.get_export_func("div_s").unwrap().call(vec![Val::Num(Num::I32(-5)),Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(),  2);
    }

    #[test]
    fn test_i32_div_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x40000000);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(0x80000001u32 as i32)),Val::Num(Num::I32(1000))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x20c49b);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(5)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 2);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(5)),Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(-5)),Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("div_u").unwrap().call(vec![Val::Num(Num::I32(17)),Val::Num(Num::I32(7))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 2);
    }

    #[test]
    fn test_i32_rem_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(5)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("rem_s").unwrap().call(vec![Val::Num(Num::I32(-5)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -1);
    }

    #[test]
    fn test_i32_rem_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]); // Duplicate?
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x80000000u32 as i32);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(0x8ff00ff0u32 as i32)),Val::Num(Num::I32(0x10001))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x8001);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(5)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(-5)),Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("rem_u").unwrap().call(vec![Val::Num(Num::I32(-5)),Val::Num(Num::I32(-2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -5); // Is this correct for unsigned rem?
    }

    #[test]
    fn test_i32_and() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x7fffffffu32 as i32);
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(0xf0f0ffffu32 as i32)),Val::Num(Num::I32(0xfffff0f0u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0xf0f0f0f0u32 as i32);
        let ret = inst.get_export_func("and").unwrap().call(vec![Val::Num(Num::I32(0xffffffffu32 as i32)),Val::Num(Num::I32(0xffffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0xffffffffu32 as i32);
    }

    #[test]
    fn test_i32_or() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("or").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("or").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("or").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("or").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("or").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -1);
        let ret = inst.get_export_func("or").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x80000000u32 as i32);
        let ret = inst.get_export_func("or").unwrap().call(vec![Val::Num(Num::I32(0xf0f0ffffu32 as i32)),Val::Num(Num::I32(0xfffff0f0u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0xffffffffu32 as i32);
    }

    #[test]
    fn test_i32_xor() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -1);
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x80000000u32 as i32);
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x7fffffffu32 as i32);
        let ret = inst.get_export_func("xor").unwrap().call(vec![Val::Num(Num::I32(0xf0f0ffffu32 as i32)),Val::Num(Num::I32(0xfffff0f0u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x0f0f0f0fu32 as i32);
    }

    #[test]
    fn test_i32_shl() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("shl").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 2);
        let ret = inst.get_export_func("shl").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("shl").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0xfffffffeu32 as i32);
        let ret = inst.get_export_func("shl").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("shl").unwrap().call(vec![Val::Num(Num::I32(0x40000000u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x80000000u32 as i32);
        let ret = inst.get_export_func("shl").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(31))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x80000000u32 as i32);
    }

    #[test]
    fn test_i32_shr_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -1);
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x3fffffffu32 as i32);
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0xc0000000u32 as i32);
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![Val::Num(Num::I32(0x40000000u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x20000000u32 as i32);
        let ret = inst.get_export_func("shr_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(31))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -1);
    }

    #[test]
    fn test_i32_shr_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x7fffffffu32 as i32);
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x3fffffffu32 as i32);
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x40000000u32 as i32);
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![Val::Num(Num::I32(0x40000000u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x20000000u32 as i32);
        let ret = inst.get_export_func("shr_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(31))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
    }

    #[test]
    fn test_i32_rotl() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 2);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -1);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0xabcd9876u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x579b30edu32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0xfe00dc00u32 as i32)),Val::Num(Num::I32(4))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0xe00dc00fu32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),Val::Num(Num::I32(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x183a5c76u32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0x00008000u32 as i32)),Val::Num(Num::I32(37))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x00100000u32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),Val::Num(Num::I32(0xff05u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x183a5c76u32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0x769abcdfu32 as i32)),Val::Num(Num::I32(0xffffffedu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x579beed3u32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0x769abcdfu32 as i32)),Val::Num(Num::I32(0x8000000du32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x579beed3u32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(31))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x80000000u32 as i32);
        let ret = inst.get_export_func("rotl").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
    }

    #[test]
    fn test_i32_rotr() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x80000000u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -1);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0xff00cc00u32 as i32)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x7f806600u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0x00080000u32 as i32)),Val::Num(Num::I32(4))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x00008000u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),Val::Num(Num::I32(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x1d860e97u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0x00008000u32 as i32)),Val::Num(Num::I32(37))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x00000400u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),Val::Num(Num::I32(0xff05u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x1d860e97u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0x769abcdfu32 as i32)),Val::Num(Num::I32(0xffffffedu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0xe6fbb4d5u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0x769abcdfu32 as i32)),Val::Num(Num::I32(0x8000000du32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0xe6fbb4d5u32 as i32);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(31))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 2);
        let ret = inst.get_export_func("rotr").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(31))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
    }

    #[test]
    fn test_i32_clz() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(0xffffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 32);
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(0x00008000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 16);
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(0xffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 24);
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 31);
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 30);
        let ret = inst.get_export_func("clz").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
    }

    #[test]
    fn test_i32_ctz() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("ctz").unwrap().call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("ctz").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 32);
        let ret = inst.get_export_func("ctz").unwrap().call(vec![Val::Num(Num::I32(0x00008000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 15);
        let ret = inst.get_export_func("ctz").unwrap().call(vec![Val::Num(Num::I32(0x00010000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 16);
        let ret = inst.get_export_func("ctz").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 31);
        let ret = inst.get_export_func("ctz").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
    }

    #[test]
    fn test_i32_popcnt() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 32);
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(0x00008000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(0x80008000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 2);
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 31);
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(0xAAAAAAAAu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 16);
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(0x55555555u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 16);
        let ret = inst.get_export_func("popcnt").unwrap().call(vec![Val::Num(Num::I32(0xDEADBEEFu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 24);
    }

    #[test]
    fn test_i32_eqz() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("eqz").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("eqz").unwrap().call(vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("eqz").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("eqz").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("eqz").unwrap().call(vec![Val::Num(Num::I32(0xffffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
    }

    #[test]
    fn test_i32_eq() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("eq").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
    }

    #[test]
    fn test_i32_ne() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)),Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(1)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(-1)),Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ne").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)),Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
    }

    #[test]
    fn test_i32_lt_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("lt_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
    }

    #[test]
    fn test_i32_lt_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("lt_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
    }

    #[test]
    fn test_i32_le_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
    }

    #[test]
    fn test_i32_le_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("le_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
    }

    #[test]
    fn test_i32_gt_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
    }

    #[test]
    fn test_i32_gt_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("gt_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
    }

    #[test]
    fn test_i32_ge_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("ge_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
    }

    #[test]
    fn test_i32_ge_u() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32)), Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 1);
        let ret = inst.get_export_func("ge_u").unwrap().call(vec![Val::Num(Num::I32(0x7fffffffu32 as i32)), Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
    }

    #[test]
    fn test_i32_extend8_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("extend8_s").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("extend8_s").unwrap().call(vec![Val::Num(Num::I32(0x7f))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 127);
        let ret = inst.get_export_func("extend8_s").unwrap().call(vec![Val::Num(Num::I32(0x80))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -128);
        let ret = inst.get_export_func("extend8_s").unwrap().call(vec![Val::Num(Num::I32(0xff))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -1);
        let ret = inst.get_export_func("extend8_s").unwrap().call(vec![Val::Num(Num::I32(0x01234500u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("extend8_s").unwrap().call(vec![Val::Num(Num::I32(0xfedcba80u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -0x80);
        let ret = inst.get_export_func("extend8_s").unwrap().call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -1);
    }

    #[test]
    fn test_i32_extend16_s() {
        let inst = load_instance("test/i32.wasm");
        let ret = inst.get_export_func("extend16_s").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("extend16_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 32767);
        let ret = inst.get_export_func("extend16_s").unwrap().call(vec![Val::Num(Num::I32(0x8000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -32768);
        let ret = inst.get_export_func("extend16_s").unwrap().call(vec![Val::Num(Num::I32(0xffffu32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -1);
        let ret = inst.get_export_func("extend16_s").unwrap().call(vec![Val::Num(Num::I32(0x01230000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0);
        let ret = inst.get_export_func("extend16_s").unwrap().call(vec![Val::Num(Num::I32(0xfedc8000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -0x8000);
        let ret = inst.get_export_func("extend16_s").unwrap().call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), -1);
    }

    #[test]
    fn test_i64_add() {
        let inst = load_instance("test/i64.wasm");
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64(), 2);
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64(), 1);
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64(), -2);
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64(), 0);
        let ret = inst.get_export_func("add").unwrap().call(vec![Val::Num(Num::I64(0x3fffffffu64 as i64)), Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64(), 0x40000000u64 as i64);
    }

    #[test]
    fn test_i64_extend_i32_s() {
        let inst = load_instance("test/conversions.wasm");
        let ret = inst.get_export_func("i64.extend_i32_s").unwrap().call(vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64(), 0);
        let ret = inst.get_export_func("i64.extend_i32_s").unwrap().call(vec![Val::Num(Num::I32(10000))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64(), 10000);
        let ret = inst.get_export_func("i64.extend_i32_s").unwrap().call(vec![Val::Num(Num::I32(-10000))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64(), -10000);
        let ret = inst.get_export_func("i64.extend_i32_s").unwrap().call(vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64(), -1);
        let ret = inst.get_export_func("i64.extend_i32_s").unwrap().call(vec![Val::Num(Num::I32(0x7fffffff))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64(), 0x000000007fffffff);
        let ret = inst.get_export_func("i64.extend_i32_s").unwrap().call(vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64(), 0xffffffff80000000u64 as i64);
    }

    #[test]
    fn test_call_type_i32() {
        let inst = load_instance("test/call.wasm");
        let ret = inst.get_export_func("type-i32").unwrap().call(vec![]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i32(), 0x132);
    }

    #[test]
    fn test_call_fac() {
        let inst = load_instance("test/call.wasm");
        let ret = inst.get_export_func("fac").unwrap().call(vec![Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64(), 1);
        let ret = inst.get_export_func("fac").unwrap().call(vec![Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64(), 1);
        let ret = inst.get_export_func("fac").unwrap().call(vec![Val::Num(Num::I64(5))]);
        assert_eq!(ret.unwrap().pop().unwrap().to_i64(), 120);
    }

}
