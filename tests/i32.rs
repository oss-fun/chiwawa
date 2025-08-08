use chiwawa::{
    execution::module::*, execution::runtime::Runtime, execution::value::*, parser,
    structure::module::Module,
};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to load module and get instance
    fn load_instance(wasm_path: &str) -> Arc<ModuleInst> {
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path, true);
        let imports: ImportObjects = HashMap::new();
        ModuleInst::new(&module, imports, Vec::new(), Vec::new()).unwrap()
    }

    // Helper function to call a function using Runtime
    fn call_function(
        inst: &Arc<ModuleInst>,
        func_name: &str,
        params: Vec<Val>,
    ) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func(func_name)?;
        let mut runtime = Runtime::new(Arc::clone(inst), &func_addr, params)?;
        runtime.set_memoization_enabled(true);
        runtime.run()
    }

    // --- i32 tests ---
    #[test]
    fn test_i32_add() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2);
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::I32(0x3fffffff)), Val::Num(Num::I32(1))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x40000000 as i32
        );
    }

    #[test]
    fn test_i32_sub() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "sub",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "sub",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "sub",
            vec![Val::Num(Num::I32(0x3fffffff)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x40000000 as i32
        );
    }

    #[test]
    fn test_i32_mul() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_div_s() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "div_s",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "div_s",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "div_s",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "div_s",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "div_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(2)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xc0000000u32 as i32
        );
        let ret = call_function(
            &inst,
            "div_s",
            vec![Val::Num(Num::I32(5)), Val::Num(Num::I32(2))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(
            &inst,
            "div_s",
            vec![Val::Num(Num::I32(5)), Val::Num(Num::I32(-2))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2);
        let ret = call_function(
            &inst,
            "div_s",
            vec![Val::Num(Num::I32(-5)), Val::Num(Num::I32(-2))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_i32_div_u() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "div_u",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "div_u",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "div_u",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "div_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(2)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x40000000);
        let ret = call_function(
            &inst,
            "div_u",
            vec![
                Val::Num(Num::I32(0x80000001u32 as i32)),
                Val::Num(Num::I32(1000)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x20c49b);
        let ret = call_function(
            &inst,
            "div_u",
            vec![Val::Num(Num::I32(5)), Val::Num(Num::I32(2))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(
            &inst,
            "div_u",
            vec![Val::Num(Num::I32(5)), Val::Num(Num::I32(-2))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "div_u",
            vec![Val::Num(Num::I32(-5)), Val::Num(Num::I32(-2))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "div_u",
            vec![Val::Num(Num::I32(17)), Val::Num(Num::I32(7))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_i32_rem_s() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "rem_s",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "rem_s",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "rem_s",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "rem_s",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "rem_s",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "rem_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "rem_s",
            vec![Val::Num(Num::I32(5)), Val::Num(Num::I32(2))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "rem_s",
            vec![Val::Num(Num::I32(-5)), Val::Num(Num::I32(2))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_i32_rem_u() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "rem_u",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "rem_u",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "rem_u",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        ); // Duplicate?
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "rem_u",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "rem_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
        let ret = call_function(
            &inst,
            "rem_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(2)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "rem_u",
            vec![
                Val::Num(Num::I32(0x8ff00ff0u32 as i32)),
                Val::Num(Num::I32(0x10001)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x8001);
        let ret = call_function(
            &inst,
            "rem_u",
            vec![Val::Num(Num::I32(5)), Val::Num(Num::I32(2))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "rem_u",
            vec![Val::Num(Num::I32(-5)), Val::Num(Num::I32(2))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "rem_u",
            vec![Val::Num(Num::I32(-5)), Val::Num(Num::I32(-2))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -5);
    }

    #[test]
    fn test_i32_and() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "and",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "and",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "and",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "and",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "and",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "and",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x7fffffffu32 as i32
        );
        let ret = call_function(
            &inst,
            "and",
            vec![
                Val::Num(Num::I32(0xf0f0ffffu32 as i32)),
                Val::Num(Num::I32(0xfffff0f0u32 as i32)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xf0f0f0f0u32 as i32
        );
        let ret = call_function(
            &inst,
            "and",
            vec![
                Val::Num(Num::I32(0xffffffffu32 as i32)),
                Val::Num(Num::I32(0xffffffffu32 as i32)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffffffffu32 as i32
        );
    }

    #[test]
    fn test_i32_or() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "or",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "or",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "or",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "or",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "or",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(
            &inst,
            "or",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
        let ret = call_function(
            &inst,
            "or",
            vec![
                Val::Num(Num::I32(0xf0f0ffffu32 as i32)),
                Val::Num(Num::I32(0xfffff0f0u32 as i32)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffffffffu32 as i32
        );
    }

    #[test]
    fn test_i32_xor() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "xor",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "xor",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "xor",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "xor",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "xor",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(
            &inst,
            "xor",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
        let ret = call_function(
            &inst,
            "xor",
            vec![
                Val::Num(Num::I32(-1)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x7fffffffu32 as i32
        );
        let ret = call_function(
            &inst,
            "xor",
            vec![
                Val::Num(Num::I32(0xf0f0ffffu32 as i32)),
                Val::Num(Num::I32(0xfffff0f0u32 as i32)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x0f0f0f0fu32 as i32
        );
    }

    #[test]
    fn test_i32_shl() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "shl",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(
            &inst,
            "shl",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "shl",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xfffffffeu32 as i32
        );
        let ret = call_function(
            &inst,
            "shl",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "shl",
            vec![
                Val::Num(Num::I32(0x40000000u32 as i32)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
        let ret = call_function(
            &inst,
            "shl",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(31))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
    }

    #[test]
    fn test_i32_shr_s() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "shr_s",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "shr_s",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "shr_s",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(
            &inst,
            "shr_s",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x3fffffffu32 as i32
        );
        let ret = call_function(
            &inst,
            "shr_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xc0000000u32 as i32
        );
        let ret = call_function(
            &inst,
            "shr_s",
            vec![
                Val::Num(Num::I32(0x40000000u32 as i32)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x20000000u32 as i32
        );
        let ret = call_function(
            &inst,
            "shr_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(31)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_i32_shr_u() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "shr_u",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "shr_u",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "shr_u",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x7fffffffu32 as i32
        );
        let ret = call_function(
            &inst,
            "shr_u",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x3fffffffu32 as i32
        );
        let ret = call_function(
            &inst,
            "shr_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x40000000u32 as i32
        );
        let ret = call_function(
            &inst,
            "shr_u",
            vec![
                Val::Num(Num::I32(0x40000000u32 as i32)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x20000000u32 as i32
        );
        let ret = call_function(
            &inst,
            "shr_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(31)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_rotl() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "rotl",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(
            &inst,
            "rotl",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "rotl",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(
            &inst,
            "rotl",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "rotl",
            vec![
                Val::Num(Num::I32(0xabcd9876u32 as i32)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x579b30edu32 as i32
        );
        let ret = call_function(
            &inst,
            "rotl",
            vec![
                Val::Num(Num::I32(0xfe00dc00u32 as i32)),
                Val::Num(Num::I32(4)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xe00dc00fu32 as i32
        );
        let ret = call_function(
            &inst,
            "rotl",
            vec![
                Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),
                Val::Num(Num::I32(5)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x183a5c76u32 as i32
        );
        let ret = call_function(
            &inst,
            "rotl",
            vec![
                Val::Num(Num::I32(0x00008000u32 as i32)),
                Val::Num(Num::I32(37)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x00100000u32 as i32
        );
        let ret = call_function(
            &inst,
            "rotl",
            vec![
                Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),
                Val::Num(Num::I32(0xff05u32 as i32)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x183a5c76u32 as i32
        );
        let ret = call_function(
            &inst,
            "rotl",
            vec![
                Val::Num(Num::I32(0x769abcdfu32 as i32)),
                Val::Num(Num::I32(0xffffffedu32 as i32)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x579beed3u32 as i32
        );
        let ret = call_function(
            &inst,
            "rotl",
            vec![
                Val::Num(Num::I32(0x769abcdfu32 as i32)),
                Val::Num(Num::I32(0x8000000du32 as i32)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x579beed3u32 as i32
        );
        let ret = call_function(
            &inst,
            "rotl",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(31))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
        let ret = call_function(
            &inst,
            "rotl",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_rotr() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "rotr",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
        let ret = call_function(
            &inst,
            "rotr",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "rotr",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(
            &inst,
            "rotr",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "rotr",
            vec![
                Val::Num(Num::I32(0xff00cc00u32 as i32)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x7f806600u32 as i32
        );
        let ret = call_function(
            &inst,
            "rotr",
            vec![
                Val::Num(Num::I32(0x00080000u32 as i32)),
                Val::Num(Num::I32(4)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x00008000u32 as i32
        );
        let ret = call_function(
            &inst,
            "rotr",
            vec![
                Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),
                Val::Num(Num::I32(5)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x1d860e97u32 as i32
        );
        let ret = call_function(
            &inst,
            "rotr",
            vec![
                Val::Num(Num::I32(0x00008000u32 as i32)),
                Val::Num(Num::I32(37)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x00000400u32 as i32
        );
        let ret = call_function(
            &inst,
            "rotr",
            vec![
                Val::Num(Num::I32(0xb0c1d2e3u32 as i32)),
                Val::Num(Num::I32(0xff05u32 as i32)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x1d860e97u32 as i32
        );
        let ret = call_function(
            &inst,
            "rotr",
            vec![
                Val::Num(Num::I32(0x769abcdfu32 as i32)),
                Val::Num(Num::I32(0xffffffedu32 as i32)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xe6fbb4d5u32 as i32
        );
        let ret = call_function(
            &inst,
            "rotr",
            vec![
                Val::Num(Num::I32(0x769abcdfu32 as i32)),
                Val::Num(Num::I32(0x8000000du32 as i32)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xe6fbb4d5u32 as i32
        );
        let ret = call_function(
            &inst,
            "rotr",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(31))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(
            &inst,
            "rotr",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(31)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_clz() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(&inst, "clz", vec![Val::Num(Num::I32(0xffffffffu32 as i32))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "clz", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 32);
        let ret = call_function(&inst, "clz", vec![Val::Num(Num::I32(0x00008000u32 as i32))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 16);
        let ret = call_function(&inst, "clz", vec![Val::Num(Num::I32(0xffu32 as i32))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 24);
        let ret = call_function(&inst, "clz", vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "clz", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 31);
        let ret = call_function(&inst, "clz", vec![Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 30);
        let ret = call_function(&inst, "clz", vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_ctz() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(&inst, "ctz", vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "ctz", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 32);
        let ret = call_function(&inst, "ctz", vec![Val::Num(Num::I32(0x00008000u32 as i32))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 15);
        let ret = call_function(&inst, "ctz", vec![Val::Num(Num::I32(0x00010000u32 as i32))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 16);
        let ret = call_function(&inst, "ctz", vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 31);
        let ret = call_function(&inst, "ctz", vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_popcnt() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(&inst, "popcnt", vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 32);
        let ret = call_function(&inst, "popcnt", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "popcnt",
            vec![Val::Num(Num::I32(0x00008000u32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "popcnt",
            vec![Val::Num(Num::I32(0x80008000u32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(
            &inst,
            "popcnt",
            vec![Val::Num(Num::I32(0x7fffffffu32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 31);
        let ret = call_function(
            &inst,
            "popcnt",
            vec![Val::Num(Num::I32(0xAAAAAAAAu32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 16);
        let ret = call_function(
            &inst,
            "popcnt",
            vec![Val::Num(Num::I32(0x55555555u32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 16);
        let ret = call_function(
            &inst,
            "popcnt",
            vec![Val::Num(Num::I32(0xDEADBEEFu32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 24);
    }

    #[test]
    fn test_i32_eqz() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(&inst, "eqz", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "eqz", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "eqz", vec![Val::Num(Num::I32(0x80000000u32 as i32))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "eqz", vec![Val::Num(Num::I32(0x7fffffffu32 as i32))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "eqz", vec![Val::Num(Num::I32(0xffffffffu32 as i32))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_eq() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "eq",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "eq",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "eq",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "eq",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "eq",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "eq",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "eq",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "eq",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "eq",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "eq",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "eq",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "eq",
            vec![
                Val::Num(Num::I32(-1)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "eq",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "eq",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_ne() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "ne",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "ne",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "ne",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ne",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "ne",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "ne",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "ne",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ne",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ne",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ne",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ne",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ne",
            vec![
                Val::Num(Num::I32(-1)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ne",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_lt_s() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "lt_s",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_s",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_s",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "lt_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_s",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_s",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_s",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_s",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "lt_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "lt_s",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "lt_s",
            vec![
                Val::Num(Num::I32(-1)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "lt_s",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_lt_u() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "lt_u",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_u",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_u",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_u",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_u",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_u",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_u",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "lt_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_u",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "lt_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "lt_u",
            vec![
                Val::Num(Num::I32(-1)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "lt_u",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_le_s() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "le_s",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_s",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_s",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_s",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_s",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_s",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "le_s",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_s",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "le_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_s",
            vec![
                Val::Num(Num::I32(-1)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "le_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_s",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_le_u() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "le_u",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_u",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_u",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "le_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_u",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_u",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_u",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "le_u",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "le_u",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "le_u",
            vec![
                Val::Num(Num::I32(-1)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "le_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "le_u",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_gt_s() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "gt_s",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_s",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_s",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_s",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_s",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_s",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "gt_s",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_s",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "gt_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_s",
            vec![
                Val::Num(Num::I32(-1)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "gt_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_s",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_gt_u() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "gt_u",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_u",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_u",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "gt_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_u",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_u",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_u",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "gt_u",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "gt_u",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "gt_u",
            vec![
                Val::Num(Num::I32(-1)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "gt_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "gt_u",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_ge_s() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "ge_s",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_s",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_s",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "ge_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_s",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_s",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_s",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_s",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "ge_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "ge_s",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "ge_s",
            vec![
                Val::Num(Num::I32(-1)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_s",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "ge_s",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_i32_ge_u() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(
            &inst,
            "ge_u",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_u",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_u",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_u",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_u",
            vec![Val::Num(Num::I32(-1)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_u",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_u",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "ge_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_u",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "ge_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "ge_u",
            vec![
                Val::Num(Num::I32(-1)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_u",
            vec![
                Val::Num(Num::I32(0x80000000u32 as i32)),
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "ge_u",
            vec![
                Val::Num(Num::I32(0x7fffffffu32 as i32)),
                Val::Num(Num::I32(0x80000000u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_i32_extend8_s() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(&inst, "extend8_s", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "extend8_s", vec![Val::Num(Num::I32(0x7f))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 127);
        let ret = call_function(&inst, "extend8_s", vec![Val::Num(Num::I32(0x80))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -128);
        let ret = call_function(&inst, "extend8_s", vec![Val::Num(Num::I32(0xff))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(
            &inst,
            "extend8_s",
            vec![Val::Num(Num::I32(0x01234500u32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "extend8_s",
            vec![Val::Num(Num::I32(0xfedcba80u32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -0x80);
        let ret = call_function(&inst, "extend8_s", vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_i32_extend16_s() {
        let inst = load_instance("tests/wasm/i32.wasm");
        let ret = call_function(&inst, "extend16_s", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "extend16_s",
            vec![Val::Num(Num::I32(0x7fffu32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 32767);
        let ret = call_function(
            &inst,
            "extend16_s",
            vec![Val::Num(Num::I32(0x8000u32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -32768);
        let ret = call_function(
            &inst,
            "extend16_s",
            vec![Val::Num(Num::I32(0xffffu32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(
            &inst,
            "extend16_s",
            vec![Val::Num(Num::I32(0x01230000u32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "extend16_s",
            vec![Val::Num(Num::I32(0xfedc8000u32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -0x8000);
        let ret = call_function(&inst, "extend16_s", vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
    }
}
