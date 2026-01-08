use chiwawa::{
    execution::module::*, execution::runtime::Runtime, execution::value::*, parser,
    structure::module::Module,
};
use rustc_hash::FxHashMap;
use std::rc::Rc;

#[cfg(test)]
mod tests {
    use super::*;

    fn load_instance(wasm_path: &str) -> Rc<ModuleInst> {
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path, true, "slot");
        let imports: ImportObjects = FxHashMap::default();
        ModuleInst::new(&module, imports, Vec::new()).unwrap()
    }

    fn call_function(
        inst: &Rc<ModuleInst>,
        func_name: &str,
        params: Vec<Val>,
    ) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func(func_name)?;
        let mut runtime = Runtime::new(Rc::clone(inst), &func_addr, params, true, false, None)?;
        runtime.run()
    }

    #[test]
    fn test_memory_init1() {
        let inst = load_instance("tests/wasm/memoryinit-1.wasm");

        // First invoke "test" function
        let _result = call_function(&inst, "test", vec![]);

        // Then check all memory locations sequentially
        let expected_values = [
            (0, 0),
            (1, 0),
            (2, 3),
            (3, 1),
            (4, 4),
            (5, 1),
            (6, 0),
            (7, 0),
            (8, 0),
            (9, 0),
            (10, 0),
            (11, 0),
            (12, 7),
            (13, 5),
            (14, 2),
            (15, 3),
            (16, 6),
            (17, 0),
            (18, 0),
            (19, 0),
            (20, 0),
            (21, 0),
            (22, 0),
            (23, 0),
            (24, 0),
            (25, 0),
            (26, 0),
            (27, 0),
            (28, 0),
            (29, 0),
        ];

        for (address, expected) in expected_values.iter() {
            let params = vec![Val::Num(Num::I32(*address))];
            let ret = call_function(&inst, "load8_u", params);
            assert_eq!(
                ret.unwrap().last().unwrap().to_i32().unwrap(),
                *expected,
                "Memory load at address {} should be {}",
                address,
                expected
            );
        }
    }

    #[test]
    fn test_memory_init2() {
        let inst = load_instance("tests/wasm/memoryinit-2.wasm");

        // First invoke "test" function
        let _result = call_function(&inst, "test", vec![]);

        // Then check all memory locations sequentially
        let expected_values = [
            (0, 0),
            (1, 0),
            (2, 3),
            (3, 1),
            (4, 4),
            (5, 1),
            (6, 0),
            (7, 2),
            (8, 7),
            (9, 1),
            (10, 8),
            (11, 0),
            (12, 7),
            (13, 5),
            (14, 2),
            (15, 3),
            (16, 6),
            (17, 0),
            (18, 0),
            (19, 0),
            (20, 0),
            (21, 0),
            (22, 0),
            (23, 0),
            (24, 0),
            (25, 0),
            (26, 0),
            (27, 0),
            (28, 0),
            (29, 0),
        ];

        for (address, expected) in expected_values.iter() {
            let params = vec![Val::Num(Num::I32(*address))];
            let ret = call_function(&inst, "load8_u", params);
            assert_eq!(
                ret.unwrap().last().unwrap().to_i32().unwrap(),
                *expected,
                "Memory load at address {} should be {}",
                address,
                expected
            );
        }
    }

    #[test]
    fn test_memory_init3() {
        let inst = load_instance("tests/wasm/memoryinit-3.wasm");

        // First invoke "test" function
        let _result = call_function(&inst, "test", vec![]);

        // Then check all memory locations sequentially
        let expected_values = [
            (0, 0),
            (1, 0),
            (2, 3),
            (3, 1),
            (4, 4),
            (5, 1),
            (6, 0),
            (7, 0),
            (8, 0),
            (9, 0),
            (10, 0),
            (11, 0),
            (12, 7),
            (13, 5),
            (14, 2),
            (15, 9),
            (16, 2),
            (17, 7),
            (18, 0),
            (19, 0),
            (20, 0),
            (21, 0),
            (22, 0),
            (23, 0),
            (24, 0),
            (25, 0),
            (26, 0),
            (27, 0),
            (28, 0),
            (29, 0),
        ];

        for (address, expected) in expected_values.iter() {
            let params = vec![Val::Num(Num::I32(*address))];
            let ret = call_function(&inst, "load8_u", params);
            assert_eq!(
                ret.unwrap().last().unwrap().to_i32().unwrap(),
                *expected,
                "Memory load at address {} should be {}",
                address,
                expected
            );
        }
    }

    #[test]
    fn test_memory_init4() {
        let inst = load_instance("tests/wasm/memoryinit-4.wasm");

        // First invoke "test" function
        let _result = call_function(&inst, "test", vec![]);

        // Then check all memory locations sequentially
        let expected_values = [
            (0, 0),
            (1, 0),
            (2, 3),
            (3, 1),
            (4, 4),
            (5, 1),
            (6, 0),
            (7, 2),
            (8, 7),
            (9, 1),
            (10, 8),
            (11, 0),
            (12, 7),
            (13, 0),
            (14, 7),
            (15, 5),
            (16, 2),
            (17, 7),
            (18, 0),
            (19, 9),
            (20, 0),
            (21, 7),
            (22, 0),
            (23, 8),
            (24, 8),
            (25, 0),
            (26, 0),
            (27, 0),
            (28, 0),
            (29, 0),
        ];

        for (address, expected) in expected_values.iter() {
            let params = vec![Val::Num(Num::I32(*address))];
            let ret = call_function(&inst, "load8_u", params);
            assert_eq!(
                ret.unwrap().last().unwrap().to_i32().unwrap(),
                *expected,
                "Memory load at address {} should be {}",
                address,
                expected
            );
        }
    }

    #[test]
    fn test_memory_init5() {
        let inst = load_instance("tests/wasm/memoryinit-5.wasm");

        let result = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(1)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_memory_init6() {
        let inst = load_instance("tests/wasm/memoryinit-6.wasm");

        let result = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(1)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_memory_init7() {
        let inst = load_instance("tests/wasm/memoryinit-7.wasm");

        let result = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(1)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_memory_init8() {
        let inst = load_instance("tests/wasm/memoryinit-8.wasm");

        let result = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(1)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_memory_init9() {
        let inst = load_instance("tests/wasm/memoryinit-9.wasm");

        let result = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(1)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_memory_init10() {
        let inst = load_instance("tests/wasm/memoryinit-10.wasm");

        let result = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(1)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), -1);
    }
}
