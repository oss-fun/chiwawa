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
    fn test_type_i32() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "type-i32", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_type_i64() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "type-i64", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_type_f32() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "type-f32", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_type_f64() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "type-f64", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_type_i32_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "type-i32-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_type_i64_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "type-i64-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 2);
    }

    #[test]
    fn test_type_f32_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "type-f32-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 3.0);
    }

    #[test]
    fn test_type_f64_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "type-f64-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_f64().unwrap(), 4.0);
    }

    #[test]
    fn test_as_block_first() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-block-first", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(&inst, "as-block-first", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_as_block_mid() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-block-mid", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(&inst, "as-block-mid", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_as_block_last() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-block-last", vec![Val::Num(Num::I32(0))]);
        assert!(ret.is_ok());
        let ret = call_function(&inst, "as-block-last", vec![Val::Num(Num::I32(1))]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_block_first_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-block-first-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 11);
        let ret = call_function(&inst, "as-block-first-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 10);
    }

    #[test]
    fn test_as_block_mid_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-block-mid-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 21);
        let ret = call_function(&inst, "as-block-mid-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 20);
    }

    #[test]
    fn test_as_block_last_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-block-last-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 11);
        let ret = call_function(&inst, "as-block-last-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 11);
    }

    #[test]
    fn test_as_loop_first() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-loop-first", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(&inst, "as-loop-first", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_as_loop_mid() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-loop-mid", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(&inst, "as-loop-mid", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 4);
    }

    #[test]
    fn test_as_loop_last() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-loop-last", vec![Val::Num(Num::I32(0))]);
        assert!(ret.is_ok());
        let ret = call_function(&inst, "as-loop-last", vec![Val::Num(Num::I32(1))]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_br_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-br-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_br_if_cond() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-br_if-cond", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_br_if_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-br_if-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_br_if_value_cond() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-br_if-value-cond", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(&inst, "as-br_if-value-cond", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_br_table_index() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-br_table-index", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_br_table_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-br_table-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_br_table_value_index() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-br_table-value-index", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_return_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-return-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
    }

    #[test]
    fn test_as_if_cond() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-if-cond", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(&inst, "as-if-cond", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_if_then() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(
            &inst,
            "as-if-then",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert!(ret.is_ok());
        let ret = call_function(
            &inst,
            "as-if-then",
            vec![Val::Num(Num::I32(4)), Val::Num(Num::I32(0))],
        );
        assert!(ret.is_ok());
        let ret = call_function(
            &inst,
            "as-if-then",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert!(ret.is_ok());
        let ret = call_function(
            &inst,
            "as-if-then",
            vec![Val::Num(Num::I32(4)), Val::Num(Num::I32(1))],
        );
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_if_else() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(
            &inst,
            "as-if-else",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert!(ret.is_ok());
        let ret = call_function(
            &inst,
            "as-if-else",
            vec![Val::Num(Num::I32(3)), Val::Num(Num::I32(0))],
        );
        assert!(ret.is_ok());
        let ret = call_function(
            &inst,
            "as-if-else",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert!(ret.is_ok());
        let ret = call_function(
            &inst,
            "as-if-else",
            vec![Val::Num(Num::I32(3)), Val::Num(Num::I32(1))],
        );
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_select_first() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-select-first", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
        let ret = call_function(&inst, "as-select-first", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_as_select_second() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-select-second", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
        let ret = call_function(&inst, "as-select-second", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_as_select_cond() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-select-cond", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_as_call_first() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-call-first", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 12);
    }

    #[test]
    fn test_as_call_mid() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-call-mid", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 13);
    }

    #[test]
    fn test_as_call_last() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-call-last", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 14);
    }

    #[test]
    fn test_as_call_indirect_func() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-call_indirect-func", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 4);
    }

    #[test]
    fn test_as_call_indirect_first() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-call_indirect-first", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 4);
    }

    #[test]
    fn test_as_call_indirect_mid() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-call_indirect-mid", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 4);
    }

    #[test]
    fn test_as_call_indirect_last() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-call_indirect-last", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 4);
    }

    #[test]
    fn test_as_local_set_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-local.set-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(&inst, "as-local.set-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 17);
    }

    #[test]
    fn test_as_local_tee_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-local.tee-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(&inst, "as-local.tee-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_global_set_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-global.set-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(&inst, "as-global.set-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_load_address() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-load-address", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_loadn_address() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-loadN-address", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 30);
    }

    #[test]
    fn test_as_store_address() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-store-address", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 30);
    }

    #[test]
    fn test_as_store_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-store-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 31);
    }

    #[test]
    fn test_as_storen_address() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-storeN-address", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 32);
    }

    #[test]
    fn test_as_storen_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-storeN-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 33);
    }

    #[test]
    fn test_as_unary_operand() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-unary-operand", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_as_binary_left() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-binary-left", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_binary_right() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-binary-right", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_test_operand() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-test-operand", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_as_compare_left() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-compare-left", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_compare_right() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-compare-right", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_memory_grow_size() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "as-memory.grow-size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_nested_block_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "nested-block-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 21);
        let ret = call_function(&inst, "nested-block-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_nested_br_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "nested-br-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 5);
        let ret = call_function(&inst, "nested-br-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_nested_br_if_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "nested-br_if-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 5);
        let ret = call_function(&inst, "nested-br_if-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_nested_br_if_value_cond() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(
            &inst,
            "nested-br_if-value-cond",
            vec![Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 5);
        let ret = call_function(
            &inst,
            "nested-br_if-value-cond",
            vec![Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_nested_br_table_value() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(&inst, "nested-br_table-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 5);
        let ret = call_function(&inst, "nested-br_table-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_nested_br_table_value_index() {
        let inst = load_instance("tests/wasm/br_if.wasm");
        let ret = call_function(
            &inst,
            "nested-br_table-value-index",
            vec![Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 5);
        let ret = call_function(
            &inst,
            "nested-br_table-value-index",
            vec![Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }
}
