use chiwawa::{
    execution::module::*, execution::runtime::Runtime, execution::value::*, parser,
    structure::module::Module,
};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    fn load_instance(wasm_path: &str) -> Arc<ModuleInst> {
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path, true);
        let imports: ImportObjects = HashMap::new();
        ModuleInst::new(&module, imports, Vec::new()).unwrap()
    }

    fn call_function(
        inst: &Arc<ModuleInst>,
        func_name: &str,
        params: Vec<Val>,
    ) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func(func_name)?;
        let mut runtime = Runtime::new(Arc::clone(inst), &func_addr, params, true)?;
        runtime.run()
    }

    #[test]
    fn test_memory_fill1() {
        let inst = load_instance("tests/wasm/memoryfill-1.wasm");

        // First invoke "test" function
        let _result = call_function(&inst, "test", vec![]);

        // Check range 0 to 65280 should all be 0
        let result1 = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(65280)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(result1.unwrap().last().unwrap().to_i32().unwrap(), -1);

        // Check range 65280 to 65536 should all be 85
        let result2 = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(65280)),
                Val::Num(Num::I32(65536)),
                Val::Num(Num::I32(85)),
            ],
        );
        assert_eq!(result2.unwrap().last().unwrap().to_i32().unwrap(), -1);
    }

    #[test]
    fn test_memory_fill2() {
        let inst = load_instance("tests/wasm/memoryfill-2.wasm");

        // First invoke "test" function
        let _result = call_function(&inst, "test", vec![]);

        // Check range 0 to 65536 should all be 0
        let result = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(65536)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), -1);
    }
    #[test]
    fn test_memory_fill3() {
        let inst = load_instance("tests/wasm/memoryfill-3.wasm");

        // First invoke "test" function
        let _result = call_function(&inst, "test", vec![]);

        // Check range 0 to 1 should be 0
        let result1 = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(1)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(result1.unwrap().last().unwrap().to_i32().unwrap(), -1);

        // Check range 1 to 65535 should be 170 (0xAA)
        let result2 = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(1)),
                Val::Num(Num::I32(65535)),
                Val::Num(Num::I32(170)),
            ],
        );
        assert_eq!(result2.unwrap().last().unwrap().to_i32().unwrap(), -1);

        // Check range 65535 to 65536 should be 0
        let result3 = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(65535)),
                Val::Num(Num::I32(65536)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(result3.unwrap().last().unwrap().to_i32().unwrap(), -1);
    }
    #[test]
    fn test_memory_fill4() {
        let inst = load_instance("tests/wasm/memoryfill-4.wasm");

        // First invoke "test" function
        let _result = call_function(&inst, "test", vec![]);

        // Check range 0 to 18 should be 0
        let result1 = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(0)),
                Val::Num(Num::I32(18)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(result1.unwrap().last().unwrap().to_i32().unwrap(), -1);

        // Check range 18 to 21 should be 85
        let result2 = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(18)),
                Val::Num(Num::I32(21)),
                Val::Num(Num::I32(85)),
            ],
        );
        assert_eq!(result2.unwrap().last().unwrap().to_i32().unwrap(), -1);

        // Check range 21 to 25 should be 170
        let result3 = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(21)),
                Val::Num(Num::I32(25)),
                Val::Num(Num::I32(170)),
            ],
        );
        assert_eq!(result3.unwrap().last().unwrap().to_i32().unwrap(), -1);

        // Check range 25 to 28 should be 85
        let result4 = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(25)),
                Val::Num(Num::I32(28)),
                Val::Num(Num::I32(85)),
            ],
        );
        assert_eq!(result4.unwrap().last().unwrap().to_i32().unwrap(), -1);

        // Check range 28 to 65536 should be 0
        let result5 = call_function(
            &inst,
            "checkRange",
            vec![
                Val::Num(Num::I32(28)),
                Val::Num(Num::I32(65536)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(result5.unwrap().last().unwrap().to_i32().unwrap(), -1);
    }
    #[test]
    fn test_memory_fill5() {
        let inst = load_instance("tests/wasm/memoryfill-5.wasm");

        // First invoke "test" function
        let _result = call_function(&inst, "test", vec![]);

        // Check range 0 to 1 should be 0
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
    fn test_memory_fill6() {
        let inst = load_instance("tests/wasm/memoryfill-6.wasm");

        // First invoke "test" function
        let _result = call_function(&inst, "test", vec![]);

        // Check range 0 to 1 should be 0
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
    fn test_memory_fill7() {
        let inst = load_instance("tests/wasm/memoryfill-7.wasm");

        // First invoke "test" function
        let _result = call_function(&inst, "test", vec![]);

        // Check range 0 to 1 should be 0
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
