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
        let _ = parser::parse_bytecode(&mut module, wasm_path, true, "stack");
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
    fn test_loop_empty() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "empty", vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_loop_singular() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "singular", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 7);
    }

    #[test]
    fn test_loop_multi() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "multi", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 8);
    }

    #[test]
    fn test_loop_nested() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "nested", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_loop_deep() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "deep", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 150);
    }

    #[test]
    fn test_loop_as_select_first() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-select-first", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_select_mid() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-select-mid", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop_as_select_last() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-select-last", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop_as_if_condition() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-if-condition", vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_loop_as_if_then() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-if-then", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_if_else() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-if-else", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop_as_br_if_first() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-br_if-first", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_br_if_last() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-br_if-last", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop_as_br_table_first() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-br_table-first", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_br_table_last() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-br_table-last", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop_as_call_indirect_first() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-call_indirect-first", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_call_indirect_mid() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-call_indirect-mid", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop_as_call_indirect_last() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-call_indirect-last", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_store_first() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-store-first", vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_loop_as_store_last() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-store-last", vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_loop_as_memory_grow_value() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-memory.grow-value", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_call_value() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-call-value", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_return_value() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-return-value", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_drop_operand() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-drop-operand", vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_loop_as_br_value() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-br-value", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_local_set_value() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-local.set-value", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_local_tee_value() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-local.tee-value", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_global_set_value() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-global.set-value", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_load_operand() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-store-first", vec![]);
        let result = call_function(&inst, "as-store-last", vec![]);
        let result = call_function(&inst, "as-load-operand", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop_as_unary_operand() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-unary-operand", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_loop_as_binary_operand() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-binary-operand", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 12);
    }

    #[test]
    fn test_loop_as_test_operand() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-test-operand", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_loop_as_compare_operand() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-compare-operand", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_loop_as_binary_operands() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-binary-operands", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 12);
    }

    #[test]
    fn test_loop_as_compare_operands() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-compare-operands", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_loop_as_mixed_operands() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "as-mixed-operands", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 27);
    }

    #[test]
    fn test_loop_break_bare() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "break-bare", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 19);
    }

    #[test]
    fn test_loop_break_value() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "break-value", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 18);
    }

    #[test]
    fn test_loop_break_repeated() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "break-repeated", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 18);
    }

    #[test]
    fn test_loop_break_inner() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "break-inner", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 0x1f);
    }

    #[test]
    fn test_loop_param() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "param", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_loop_params() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "params", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_loop_params_id() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "params-id", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_loop_param_break() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "param-break", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 13);
    }

    #[test]
    fn test_loop_params_break() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "params-break", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 12);
    }

    #[test]
    fn test_loop_params_id_break() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "params-id-break", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_loop_effects() {
        let inst = load_instance("tests/wasm/loop.wasm");
        let result = call_function(&inst, "effects", vec![]);
        assert_eq!(result.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }
}
