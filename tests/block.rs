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
        ModuleInst::new(&module, imports, Vec::new(), Vec::new()).unwrap()
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
    fn test_empty() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "empty", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_singular() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "singular", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 7);
    }

    #[test]
    fn test_multi() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "multi", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 8);
    }

    #[test]
    fn test_nested() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "nested", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_deep() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "deep", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 150);
    }

    #[test]
    fn test_as_select_first() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-select-first", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_select_mid() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-select-mid", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_select_last() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-select-last", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_loop_first() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-loop-first", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_loop_mid() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-loop-mid", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_loop_last() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-loop-last", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_if_condition() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-if-condition", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_if_then() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-if-then", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_if_else() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-if-else", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_br_if_first() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-br_if-first", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_br_if_last() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-br_if-last", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_br_table_first() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-br_table-first", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_br_table_last() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-br_table-last", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_call_indirect_first() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-call_indirect-first", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_call_indirect_mid() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-call_indirect-mid", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_call_indirect_last() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-call_indirect-last", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_store_and_load() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-store-first", vec![]);
        assert!(ret.is_ok());

        let ret = call_function(&inst, "as-store-last", vec![]);
        assert!(ret.is_ok());

        let ret = call_function(&inst, "as-load-operand", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_memory_grow_value() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-memory.grow-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_call_value() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-call-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_return_value() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-return-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_drop_operand() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-drop-operand", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_br_value() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-br-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_local_set_value() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-local.set-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_local_tee_value() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-local.tee-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_global_set_value() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-global.set-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_unary_operand() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-unary-operand", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_as_binary_operand() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-binary-operand", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 12);
    }

    #[test]
    fn test_as_test_operand() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-test-operand", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_as_compare_operand() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-compare-operand", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_as_binary_operands() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-binary-operands", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 12);
    }

    #[test]
    fn test_as_compare_operands() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-compare-operands", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_as_mixed_operands() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "as-mixed-operands", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 27);
    }

    #[test]
    fn test_break_bare() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "break-bare", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 19);
    }

    #[test]
    fn test_break_value() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "break-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 18);
    }

    #[test]
    fn test_break_repeated() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "break-repeated", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 18);
    }

    #[test]
    fn test_break_inner() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "break-inner", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0xf);
    }

    #[test]
    fn test_param() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "param", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_params() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "params", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_params_id() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "params-id", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_param_break() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "param-break", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_params_break() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "params-break", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_params_id_break() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "params-id-break", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_effects() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "effects", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_type_use() {
        let inst = load_instance("tests/wasm/block.wasm");
        let ret = call_function(&inst, "type-use", vec![]);
        assert!(ret.is_ok());
    }
}
