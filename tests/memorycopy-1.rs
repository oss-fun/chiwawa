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
        let _ = parser::parse_bytecode(&mut module, wasm_path);
        let imports: ImportObjects = HashMap::new();
        ModuleInst::new(&module, imports, Vec::new()).unwrap()
    }

    fn call_function(
        inst: &Arc<ModuleInst>,
        func_name: &str,
        params: Vec<Val>,
    ) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func(func_name)?;
        let mut runtime = Runtime::new(Arc::clone(inst), &func_addr, params)?;
        runtime.run()
    }

    #[test]
    fn test_memory_copy_sequence() {
        let inst = load_instance("tests/wasm/memorycopy-1.wasm");

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
}
