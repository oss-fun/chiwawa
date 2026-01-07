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
    fn test_switch_stmt() {
        let inst = load_instance("tests/wasm/switch.wasm");

        let test_cases = [
            (0, 0),
            (1, -1),
            (2, -2),
            (3, -3),
            (4, 100),
            (5, 101),
            (6, 102),
            (7, 100),
            (-10, 102),
        ];

        for (input, expected) in test_cases {
            let params = vec![Val::Num(Num::I32(input))];
            let result = call_function(&inst, "stmt", params).expect("Runtime execution failed");
            assert_eq!(result.len(), 1);
            if let Val::Num(Num::I32(actual)) = result[0] {
                assert_eq!(
                    actual, expected,
                    "stmt({}) expected {} but got {}",
                    input, expected, actual
                );
            } else {
                panic!("Expected I32 result for stmt({})", input);
            }
        }
    }

    #[test]
    fn test_switch_expr() {
        let inst = load_instance("tests/wasm/switch.wasm");

        let test_cases = [
            (0, 0),
            (1, -1),
            (2, -2),
            (3, -3),
            (6, 101),
            (7, -5),
            (-10, 100),
        ];

        for (input, expected) in test_cases {
            let params = vec![Val::Num(Num::I64(input))];
            let result = call_function(&inst, "expr", params).expect("Runtime execution failed");
            assert_eq!(result.len(), 1);
            if let Val::Num(Num::I64(actual)) = result[0] {
                assert_eq!(
                    actual, expected,
                    "expr({}) expected {} but got {}",
                    input, expected, actual
                );
            } else {
                panic!("Expected I64 result for expr({})", input);
            }
        }
    }

    #[test]
    fn test_switch_arg() {
        let inst = load_instance("tests/wasm/switch.wasm");

        let test_cases = [
            (0, 110),
            (1, 12),
            (2, 4),
            (3, 1116),
            (4, 118),
            (5, 20),
            (6, 12),
            (7, 1124),
            (8, 126),
        ];

        for (input, expected) in test_cases {
            let params = vec![Val::Num(Num::I32(input))];
            let result = call_function(&inst, "arg", params).expect("Runtime execution failed");
            assert_eq!(result.len(), 1);
            if let Val::Num(Num::I32(actual)) = result[0] {
                assert_eq!(
                    actual, expected,
                    "arg({}) expected {} but got {}",
                    input, expected, actual
                );
            } else {
                panic!("Expected I32 result for arg({})", input);
            }
        }
    }

    #[test]
    fn test_switch_corner() {
        let inst = load_instance("tests/wasm/switch.wasm");

        let params = vec![];
        let result = call_function(&inst, "corner", params).expect("Runtime execution failed");
        assert_eq!(result.len(), 1);
        if let Val::Num(Num::I32(actual)) = result[0] {
            assert_eq!(actual, 1, "corner() expected 1 but got {}", actual);
        } else {
            panic!("Expected I32 result for corner()");
        }
    }
}
