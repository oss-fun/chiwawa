use chiwawa::{
    execution::module::*, execution::runtime::Runtime, execution::value::*, parser,
    structure::module::Module,
};
use std::collections::HashMap;
use std::rc::Rc;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to load module and get instance
    fn load_instance(wasm_path: &str) -> Rc<ModuleInst> {
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path, true);
        let imports: ImportObjects = HashMap::new();
        ModuleInst::new(&module, imports, Vec::new()).unwrap()
    }

    // Helper function to call a function using Runtime
    fn call_function(
        inst: &Rc<ModuleInst>,
        func_name: &str,
        params: Vec<Val>,
    ) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func(func_name)?;
        let mut runtime = Runtime::new(Rc::clone(inst), &func_addr, params, true)?;
        runtime.run()
    }

    #[test]
    fn test_store_operations() {
        let inst = load_instance("tests/wasm/store.wasm");

        // Test all store operations
        let test_cases = vec![
            "as-block-value",
            "as-loop-value",
            "as-br-value",
            "as-br_if-value",
            "as-br_if-value-cond",
            "as-br_table-value",
            "as-return-value",
            "as-if-then",
            "as-if-else",
        ];

        for func_name in test_cases {
            let result = call_function(&inst, func_name, vec![]);
            assert!(result.is_ok(), "Failed to execute {}", func_name);
            assert_eq!(
                result.unwrap(),
                vec![],
                "Unexpected result for {}",
                func_name
            );
        }
    }
}
