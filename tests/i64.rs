use chiwawa::{
    execution::module::*, execution::runtime::Runtime, execution::value::*, parser,
    structure::module::Module,
};
use rustc_hash::FxHashMap;
use std::rc::Rc;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to load module and get instance
    fn load_instance(wasm_path: &str) -> Rc<ModuleInst> {
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path, true, "slot");
        let imports: ImportObjects = FxHashMap::default();
        ModuleInst::new(&module, imports, Vec::new()).unwrap()
    }

    // Helper function to call a function using Runtime
    fn call_function(
        inst: &Rc<ModuleInst>,
        func_name: &str,
        params: Vec<Val>,
    ) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func(func_name)?;
        let mut runtime = Runtime::new(
            Rc::clone(inst),
            &func_addr,
            params,
            true,
            false,
            false,
            None,
        )?;
        runtime.run()
    }

    #[test]
    fn test_i64_add() {
        let inst = load_instance("tests/wasm/i64.wasm");

        // Basic addition tests
        assert_eq!(
            call_function(
                &inst,
                "add",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            2
        );
        assert_eq!(
            call_function(
                &inst,
                "add",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "add",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -2
        );
        assert_eq!(
            call_function(
                &inst,
                "add",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "add",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "add",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(-1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x7fffffffffffffff
        );
        assert_eq!(
            call_function(
                &inst,
                "add",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "add",
                vec![Val::Num(Num::I64(0x3fffffff)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x40000000
        );
    }

    #[test]
    fn test_i64_sub() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "sub",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "sub",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "sub",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "sub",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(-1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "sub",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x7fffffffffffffff
        );
        assert_eq!(
            call_function(
                &inst,
                "sub",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "sub",
                vec![Val::Num(Num::I64(0x3fffffff)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x40000000
        );
    }

    #[test]
    fn test_i64_mul() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "mul",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "mul",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "mul",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "mul",
                vec![
                    Val::Num(Num::I64(0x1000000000000000u64 as i64)),
                    Val::Num(Num::I64(4096))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "mul",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "mul",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(-1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "mul",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(-1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8000000000000001u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "mul",
                vec![
                    Val::Num(Num::I64(0x0123456789abcdefu64 as i64)),
                    Val::Num(Num::I64(0xfedcba9876543210u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x2236d88fe5618cf0u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "mul",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
    }

    #[test]
    fn test_i64_div_s() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(2))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0xc000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000001u64 as i64)),
                    Val::Num(Num::I64(1000))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0xffdf3b645a1cac09u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![Val::Num(Num::I64(5)), Val::Num(Num::I64(2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            2
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![Val::Num(Num::I64(-5)), Val::Num(Num::I64(2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -2
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![Val::Num(Num::I64(5)), Val::Num(Num::I64(-2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -2
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![Val::Num(Num::I64(-5)), Val::Num(Num::I64(-2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            2
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![Val::Num(Num::I64(7)), Val::Num(Num::I64(3))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            2
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![Val::Num(Num::I64(-7)), Val::Num(Num::I64(3))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -2
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![Val::Num(Num::I64(7)), Val::Num(Num::I64(-3))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -2
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![Val::Num(Num::I64(-7)), Val::Num(Num::I64(-3))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            2
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![Val::Num(Num::I64(11)), Val::Num(Num::I64(5))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            2
        );
        assert_eq!(
            call_function(
                &inst,
                "div_s",
                vec![Val::Num(Num::I64(17)), Val::Num(Num::I64(7))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            2
        );
    }

    #[test]
    fn test_i64_div_u() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "div_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "div_u",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "div_u",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "div_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(-1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "div_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(2))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x4000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "div_u",
                vec![
                    Val::Num(Num::I64(0x8ff00ff00ff00ff0u64 as i64)),
                    Val::Num(Num::I64(0x100000001u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8ff00fef
        );
        assert_eq!(
            call_function(
                &inst,
                "div_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000001u64 as i64)),
                    Val::Num(Num::I64(1000))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x20c49ba5e353f7
        );
        assert_eq!(
            call_function(
                &inst,
                "div_u",
                vec![Val::Num(Num::I64(5)), Val::Num(Num::I64(2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            2
        );
        assert_eq!(
            call_function(
                &inst,
                "div_u",
                vec![Val::Num(Num::I64(-5)), Val::Num(Num::I64(2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x7ffffffffffffffd
        );
        assert_eq!(
            call_function(
                &inst,
                "div_u",
                vec![Val::Num(Num::I64(5)), Val::Num(Num::I64(-2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "div_u",
                vec![Val::Num(Num::I64(-5)), Val::Num(Num::I64(-2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "div_u",
                vec![Val::Num(Num::I64(7)), Val::Num(Num::I64(3))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            2
        );
        assert_eq!(
            call_function(
                &inst,
                "div_u",
                vec![Val::Num(Num::I64(11)), Val::Num(Num::I64(5))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            2
        );
        assert_eq!(
            call_function(
                &inst,
                "div_u",
                vec![Val::Num(Num::I64(17)), Val::Num(Num::I64(7))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            2
        );
    }

    #[test]
    fn test_i64_rem_s() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(-1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(2))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000001u64 as i64)),
                    Val::Num(Num::I64(1000))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -807
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![Val::Num(Num::I64(5)), Val::Num(Num::I64(2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![Val::Num(Num::I64(-5)), Val::Num(Num::I64(2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![Val::Num(Num::I64(5)), Val::Num(Num::I64(-2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![Val::Num(Num::I64(-5)), Val::Num(Num::I64(-2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![Val::Num(Num::I64(7)), Val::Num(Num::I64(3))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![Val::Num(Num::I64(-7)), Val::Num(Num::I64(3))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![Val::Num(Num::I64(7)), Val::Num(Num::I64(-3))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![Val::Num(Num::I64(-7)), Val::Num(Num::I64(-3))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![Val::Num(Num::I64(11)), Val::Num(Num::I64(5))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_s",
                vec![Val::Num(Num::I64(17)), Val::Num(Num::I64(7))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            3
        );
    }

    #[test]
    fn test_i64_rem_u() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "rem_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_u",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_u",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(-1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(2))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_u",
                vec![
                    Val::Num(Num::I64(0x8ff00ff00ff00ff0u64 as i64)),
                    Val::Num(Num::I64(0x100000001u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x80000001u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000001u64 as i64)),
                    Val::Num(Num::I64(1000))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            809
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_u",
                vec![Val::Num(Num::I64(5)), Val::Num(Num::I64(2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_u",
                vec![Val::Num(Num::I64(-5)), Val::Num(Num::I64(2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_u",
                vec![Val::Num(Num::I64(5)), Val::Num(Num::I64(-2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            5
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_u",
                vec![Val::Num(Num::I64(-5)), Val::Num(Num::I64(-2))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -5
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_u",
                vec![Val::Num(Num::I64(7)), Val::Num(Num::I64(3))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_u",
                vec![Val::Num(Num::I64(11)), Val::Num(Num::I64(5))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "rem_u",
                vec![Val::Num(Num::I64(17)), Val::Num(Num::I64(7))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            3
        );
    }

    #[test]
    fn test_i64_and() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "and",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "and",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "and",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "and",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "and",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "and",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(-1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x7fffffffffffffff
        );
        assert_eq!(
            call_function(
                &inst,
                "and",
                vec![
                    Val::Num(Num::I64(0xf0f0ffff)),
                    Val::Num(Num::I64(0xfffff0f0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0xf0f0f0f0
        );
        assert_eq!(
            call_function(
                &inst,
                "and",
                vec![
                    Val::Num(Num::I64(0xffffffffffffff00u64 as i64)),
                    Val::Num(Num::I64(0xff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "and",
                vec![
                    Val::Num(Num::I64(0xffffffffffffffffu64 as i64)),
                    Val::Num(Num::I64(0x55))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x55
        );
    }

    #[test]
    fn test_i64_or() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "or",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "or",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "or",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "or",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "or",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "or",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "or",
                vec![
                    Val::Num(Num::I64(0xf0f0ffff)),
                    Val::Num(Num::I64(0xfffff0f0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0xffffffff
        );
        assert_eq!(
            call_function(
                &inst,
                "or",
                vec![
                    Val::Num(Num::I64(0xffffffffffffff00u64 as i64)),
                    Val::Num(Num::I64(0xff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "or",
                vec![
                    Val::Num(Num::I64(0xaaaaaaaaaaaaaaaau64 as i64)),
                    Val::Num(Num::I64(0x5555555555555555u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
    }

    #[test]
    fn test_i64_xor() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "xor",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "xor",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "xor",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "xor",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "xor",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "xor",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "xor",
                vec![
                    Val::Num(Num::I64(-1)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x7fffffffffffffff
        );
        assert_eq!(
            call_function(
                &inst,
                "xor",
                vec![
                    Val::Num(Num::I64(-1)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "xor",
                vec![
                    Val::Num(Num::I64(0xf0f0ffff)),
                    Val::Num(Num::I64(0xfffff0f0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x0f0f0f0f
        );
        assert_eq!(
            call_function(
                &inst,
                "xor",
                vec![
                    Val::Num(Num::I64(0xffffffffffffff00u64 as i64)),
                    Val::Num(Num::I64(0xff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "xor",
                vec![
                    Val::Num(Num::I64(0xaaaaaaaaaaaaaaaau64 as i64)),
                    Val::Num(Num::I64(0x5555555555555555))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
    }

    #[test]
    fn test_i64_shl() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "shl",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            2
        );
        assert_eq!(
            call_function(
                &inst,
                "shl",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "shl",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0xfffffffffffffffeu64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "shl",
                vec![
                    Val::Num(Num::I64(0xffffffffffffffffu64 as i64)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0xfffffffffffffffeu64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "shl",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "shl",
                vec![
                    Val::Num(Num::I64(0x4000000000000000u64 as i64)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "shl",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(63))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "shl",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(64))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "shl",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(65))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            2
        );
        assert_eq!(
            call_function(
                &inst,
                "shl",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "shl",
                vec![
                    Val::Num(Num::I64(1)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8000000000000000u64 as i64
        );
    }

    #[test]
    fn test_i64_shr_s() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x3fffffffffffffff
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0xc000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![
                    Val::Num(Num::I64(0x4000000000000000u64 as i64)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x2000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(64))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(65))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![
                    Val::Num(Num::I64(1)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![
                    Val::Num(Num::I64(1)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(63))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(64))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(65))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![
                    Val::Num(Num::I64(-1)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_s",
                vec![
                    Val::Num(Num::I64(-1)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
    }

    #[test]
    fn test_i64_shr_u() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x7fffffffffffffff
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x3fffffffffffffff
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x4000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![
                    Val::Num(Num::I64(0x4000000000000000u64 as i64)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x2000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(64))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(65))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![
                    Val::Num(Num::I64(1)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![
                    Val::Num(Num::I64(1)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(63))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(64))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(65))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x7fffffffffffffff
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![
                    Val::Num(Num::I64(-1)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "shr_u",
                vec![
                    Val::Num(Num::I64(-1)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
    }

    #[test]
    fn test_i64_rotl() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "rotl",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            2
        );
        assert_eq!(
            call_function(
                &inst,
                "rotl",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "rotl",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "rotl",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(64))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "rotl",
                vec![
                    Val::Num(Num::I64(0xabcd987602468aceu64 as i64)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x579b30ec048d159du64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "rotl",
                vec![
                    Val::Num(Num::I64(0xfe000000dc000000u64 as i64)),
                    Val::Num(Num::I64(4))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0xe000000dc000000fu64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "rotl",
                vec![
                    Val::Num(Num::I64(0xabcd1234ef567809u64 as i64)),
                    Val::Num(Num::I64(53))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x013579a2469deacfu64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "rotl",
                vec![
                    Val::Num(Num::I64(0xabd1234ef567809cu64 as i64)),
                    Val::Num(Num::I64(63))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x55e891a77ab3c04eu64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "rotl",
                vec![
                    Val::Num(Num::I64(0xabcd1234ef567809u64 as i64)),
                    Val::Num(Num::I64(0xf5))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x013579a2469deacfu64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "rotl",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(63))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "rotl",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
    }

    #[test]
    fn test_i64_rotr() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "rotr",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x8000000000000000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "rotr",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "rotr",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            -1
        );
        assert_eq!(
            call_function(
                &inst,
                "rotr",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(64))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "rotr",
                vec![
                    Val::Num(Num::I64(0xabcd987602468aceu64 as i64)),
                    Val::Num(Num::I64(1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x55e6cc3b01234567u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "rotr",
                vec![
                    Val::Num(Num::I64(0xfe000000dc000000u64 as i64)),
                    Val::Num(Num::I64(4))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x0fe000000dc00000u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "rotr",
                vec![
                    Val::Num(Num::I64(0xabcd1234ef567809u64 as i64)),
                    Val::Num(Num::I64(53))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x6891a77ab3c04d5eu64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "rotr",
                vec![
                    Val::Num(Num::I64(0xabd1234ef567809cu64 as i64)),
                    Val::Num(Num::I64(63))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x57a2469deacf0139u64 as i64
        );
        assert_eq!(
            call_function(
                &inst,
                "rotr",
                vec![
                    Val::Num(Num::I64(0xabcd1234ef567809u64 as i64)),
                    Val::Num(Num::I64(0xf5))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0x6891a77ab3c04d5eu64 as i64
        );
    }

    #[test]
    fn test_i64_clz() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "clz",
                vec![Val::Num(Num::I64(0xffffffffffffffffu64 as i64))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(&inst, "clz", vec![Val::Num(Num::I64(0))])
                .unwrap()
                .last()
                .unwrap()
                .to_i64()
                .unwrap(),
            64
        );
        assert_eq!(
            call_function(&inst, "clz", vec![Val::Num(Num::I64(0x00008000))])
                .unwrap()
                .last()
                .unwrap()
                .to_i64()
                .unwrap(),
            48
        );
        assert_eq!(
            call_function(&inst, "clz", vec![Val::Num(Num::I64(0xff))])
                .unwrap()
                .last()
                .unwrap()
                .to_i64()
                .unwrap(),
            56
        );
        assert_eq!(
            call_function(
                &inst,
                "clz",
                vec![Val::Num(Num::I64(0x8000000000000000u64 as i64))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(&inst, "clz", vec![Val::Num(Num::I64(1))])
                .unwrap()
                .last()
                .unwrap()
                .to_i64()
                .unwrap(),
            63
        );
        assert_eq!(
            call_function(&inst, "clz", vec![Val::Num(Num::I64(2))])
                .unwrap()
                .last()
                .unwrap()
                .to_i64()
                .unwrap(),
            62
        );
        assert_eq!(
            call_function(&inst, "clz", vec![Val::Num(Num::I64(0x7fffffffffffffff))])
                .unwrap()
                .last()
                .unwrap()
                .to_i64()
                .unwrap(),
            1
        );
    }

    #[test]
    fn test_i64_ctz() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(&inst, "ctz", vec![Val::Num(Num::I64(-1))])
                .unwrap()
                .last()
                .unwrap()
                .to_i64()
                .unwrap(),
            0
        );
        assert_eq!(
            call_function(&inst, "ctz", vec![Val::Num(Num::I64(0))])
                .unwrap()
                .last()
                .unwrap()
                .to_i64()
                .unwrap(),
            64
        );
        assert_eq!(
            call_function(&inst, "ctz", vec![Val::Num(Num::I64(0x00008000))])
                .unwrap()
                .last()
                .unwrap()
                .to_i64()
                .unwrap(),
            15
        );
        assert_eq!(
            call_function(&inst, "ctz", vec![Val::Num(Num::I64(0x00010000))])
                .unwrap()
                .last()
                .unwrap()
                .to_i64()
                .unwrap(),
            16
        );
        assert_eq!(
            call_function(
                &inst,
                "ctz",
                vec![Val::Num(Num::I64(0x8000000000000000u64 as i64))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            63
        );
        assert_eq!(
            call_function(&inst, "ctz", vec![Val::Num(Num::I64(0x7fffffffffffffff))])
                .unwrap()
                .last()
                .unwrap()
                .to_i64()
                .unwrap(),
            0
        );
    }

    #[test]
    fn test_i64_popcnt() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(&inst, "popcnt", vec![Val::Num(Num::I64(-1))])
                .unwrap()
                .last()
                .unwrap()
                .to_i64()
                .unwrap(),
            64
        );
        assert_eq!(
            call_function(&inst, "popcnt", vec![Val::Num(Num::I64(0))])
                .unwrap()
                .last()
                .unwrap()
                .to_i64()
                .unwrap(),
            0
        );
        assert_eq!(
            call_function(&inst, "popcnt", vec![Val::Num(Num::I64(0x00008000))])
                .unwrap()
                .last()
                .unwrap()
                .to_i64()
                .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "popcnt",
                vec![Val::Num(Num::I64(0x8000800080008000u64 as i64))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            4
        );
        assert_eq!(
            call_function(
                &inst,
                "popcnt",
                vec![Val::Num(Num::I64(0x7fffffffffffffff))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            63
        );
        assert_eq!(
            call_function(
                &inst,
                "popcnt",
                vec![Val::Num(Num::I64(0xAAAAAAAA55555555u64 as i64))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            32
        );
        assert_eq!(
            call_function(
                &inst,
                "popcnt",
                vec![Val::Num(Num::I64(0x99999999AAAAAAAAu64 as i64))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            32
        );
        assert_eq!(
            call_function(
                &inst,
                "popcnt",
                vec![Val::Num(Num::I64(0xDEADBEEFDEADBEEFu64 as i64))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i64()
            .unwrap(),
            48
        );
    }

    #[test]
    fn test_i64_eq() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "eq",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "eq",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "eq",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "eq",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "eq",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "eq",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "eq",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "eq",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "eq",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "eq",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(-1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "eq",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
    }

    #[test]
    fn test_i64_ne() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "ne",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "ne",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "ne",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ne",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "ne",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "ne",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "ne",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ne",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ne",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ne",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(-1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ne",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
    }

    #[test]
    fn test_i64_lt_s() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "lt_s",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_s",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_s",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_s",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_s",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_s",
                vec![
                    Val::Num(Num::I64(0)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(-1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_s",
                vec![
                    Val::Num(Num::I64(-1)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_s",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
    }

    #[test]
    fn test_i64_lt_u() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "lt_u",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_u",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_u",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_u",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_u",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_u",
                vec![
                    Val::Num(Num::I64(0)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(-1))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_u",
                vec![
                    Val::Num(Num::I64(-1)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "lt_u",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
    }

    #[test]
    fn test_i64_gt_s() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "gt_s",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_s",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_s",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_s",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_s",
                vec![
                    Val::Num(Num::I64(0)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_s",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
    }

    #[test]
    fn test_i64_gt_u() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "gt_u",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_u",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_u",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_u",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_u",
                vec![
                    Val::Num(Num::I64(0)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "gt_u",
                vec![
                    Val::Num(Num::I64(-1)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
    }

    #[test]
    fn test_i64_le_s() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "le_s",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "le_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "le_s",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "le_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "le_s",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "le_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "le_s",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "le_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "le_s",
                vec![
                    Val::Num(Num::I64(0)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
    }

    #[test]
    fn test_i64_le_u() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "le_u",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "le_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "le_u",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "le_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "le_u",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "le_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "le_u",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "le_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "le_u",
                vec![
                    Val::Num(Num::I64(0)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
    }

    #[test]
    fn test_i64_ge_s() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "ge_s",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_s",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_s",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_s",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_s",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_s",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_s",
                vec![
                    Val::Num(Num::I64(0)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
    }

    #[test]
    fn test_i64_ge_u() {
        let inst = load_instance("tests/wasm/i64.wasm");

        assert_eq!(
            call_function(
                &inst,
                "ge_u",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_u",
                vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_u",
                vec![
                    Val::Num(Num::I64(0x7fffffffffffffff)),
                    Val::Num(Num::I64(0x7fffffffffffffff))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_u",
                vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_u",
                vec![Val::Num(Num::I64(0)), Val::Num(Num::I64(1))]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_u",
                vec![
                    Val::Num(Num::I64(0x8000000000000000u64 as i64)),
                    Val::Num(Num::I64(0))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            1
        );
        assert_eq!(
            call_function(
                &inst,
                "ge_u",
                vec![
                    Val::Num(Num::I64(0)),
                    Val::Num(Num::I64(0x8000000000000000u64 as i64))
                ]
            )
            .unwrap()
            .last()
            .unwrap()
            .to_i32()
            .unwrap(),
            0
        );
    }
}
