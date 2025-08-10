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
        ModuleInst::new(&module, imports, Vec::new(), Vec::new()).unwrap()
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
    fn test_type_i32() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "type-i32"))
        let ret = call_function(&inst, "type-i32", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_type_i64() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "type-i64"))
        let ret = call_function(&inst, "type-i64", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_type_f32() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "type-f32"))
        let ret = call_function(&inst, "type-f32", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_type_f64() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "type-f64"))
        let ret = call_function(&inst, "type-f64", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_type_i32_i32() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "type-i32-i32"))
        let ret = call_function(&inst, "type-i32-i32", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_type_i64_i64() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "type-i64-i64"))
        let ret = call_function(&inst, "type-i64-i64", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_type_f32_f32() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "type-f32-f32"))
        let ret = call_function(&inst, "type-f32-f32", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_type_f64_f64() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "type-f64-f64"))
        let ret = call_function(&inst, "type-f64-f64", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_type_i32_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "type-i32-value") (i32.const 1))
        let ret = call_function(&inst, "type-i32-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_type_i64_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "type-i64-value") (i64.const 2))
        let ret = call_function(&inst, "type-i64-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 2);
    }

    #[test]
    fn test_type_f32_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "type-f32-value") (f32.const 3))
        let ret = call_function(&inst, "type-f32-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 3.0);
    }

    #[test]
    fn test_type_f64_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "type-f64-value") (f64.const 4))
        let ret = call_function(&inst, "type-f64-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_f64().unwrap(), 4.0);
    }

    #[test]
    fn test_as_block_first() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-block-first"))
        let ret = call_function(&inst, "as-block-first", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_block_mid() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-block-mid"))
        let ret = call_function(&inst, "as-block-mid", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_block_last() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-block-last"))
        let ret = call_function(&inst, "as-block-last", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_block_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-block-value") (i32.const 2))
        let ret = call_function(&inst, "as-block-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_loop_first() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-loop-first") (i32.const 3))
        let ret = call_function(&inst, "as-loop-first", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_as_loop_mid() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-loop-mid") (i32.const 4))
        let ret = call_function(&inst, "as-loop-mid", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 4);
    }

    #[test]
    fn test_as_loop_last() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-loop-last") (i32.const 5))
        let ret = call_function(&inst, "as-loop-last", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 5);
    }

    #[test]
    fn test_as_br_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-br-value") (i32.const 9))
        let ret = call_function(&inst, "as-br-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_as_br_if_cond() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-br_if-cond"))
        let ret = call_function(&inst, "as-br_if-cond", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_br_if_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-br_if-value") (i32.const 8))
        let ret = call_function(&inst, "as-br_if-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 8);
    }

    #[test]
    fn test_as_br_if_value_cond() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-br_if-value-cond") (i32.const 9))
        let ret = call_function(&inst, "as-br_if-value-cond", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_as_br_table_index() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-br_table-index"))
        let ret = call_function(&inst, "as-br_table-index", vec![]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_br_table_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-br_table-value") (i32.const 10))
        let ret = call_function(&inst, "as-br_table-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 10);
    }

    #[test]
    fn test_as_br_table_value_index() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-br_table-value-index") (i32.const 11))
        let ret = call_function(&inst, "as-br_table-value-index", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 11);
    }

    #[test]
    fn test_as_return_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-return-value") (i64.const 7))
        let ret = call_function(&inst, "as-return-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 7);
    }

    #[test]
    fn test_as_if_cond() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-if-cond") (i32.const 2))
        let ret = call_function(&inst, "as-if-cond", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_if_then() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-if-then" (i32.const 1) (i32.const 6)) (i32.const 3))
        let ret = call_function(
            &inst,
            "as-if-then",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(6))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
        //(assert_return (invoke "as-if-then" (i32.const 0) (i32.const 6)) (i32.const 6))
        let ret = call_function(
            &inst,
            "as-if-then",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(6))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 6);
    }

    #[test]
    fn test_as_if_else() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-if-else" (i32.const 0) (i32.const 6)) (i32.const 4))
        let ret = call_function(
            &inst,
            "as-if-else",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(6))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 4);
        //(assert_return (invoke "as-if-else" (i32.const 1) (i32.const 6)) (i32.const 6))
        let ret = call_function(
            &inst,
            "as-if-else",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(6))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 6);
    }

    #[test]
    fn test_as_select_first() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-select-first" (i32.const 0) (i32.const 6)) (i32.const 5))
        let ret = call_function(
            &inst,
            "as-select-first",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(6))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 5);
        //(assert_return (invoke "as-select-first" (i32.const 1) (i32.const 6)) (i32.const 5))
        let ret = call_function(
            &inst,
            "as-select-first",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(6))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 5);
    }

    #[test]
    fn test_as_select_second() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-select-second" (i32.const 0) (i32.const 6)) (i32.const 6))
        let ret = call_function(
            &inst,
            "as-select-second",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(6))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 6);
        //(assert_return (invoke "as-select-second" (i32.const 1) (i32.const 6)) (i32.const 6))
        let ret = call_function(
            &inst,
            "as-select-second",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(6))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 6);
    }

    #[test]
    fn test_as_select_cond() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-select-cond") (i32.const 7))
        let ret = call_function(&inst, "as-select-cond", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 7);
    }

    #[test]
    fn test_as_select_all() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-select-all") (i32.const 8))
        let ret = call_function(&inst, "as-select-all", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 8);
    }

    #[test]
    fn test_as_call_first() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-call-first") (i32.const 12))
        let ret = call_function(&inst, "as-call-first", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 12);
    }

    #[test]
    fn test_as_call_mid() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-call-mid") (i32.const 13))
        let ret = call_function(&inst, "as-call-mid", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 13);
    }

    #[test]
    fn test_as_call_last() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-call-last") (i32.const 14))
        let ret = call_function(&inst, "as-call-last", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 14);
    }

    #[test]
    fn test_as_call_all() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-call-all") (i32.const 15))
        let ret = call_function(&inst, "as-call-all", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 15);
    }

    #[test]
    fn test_as_call_indirect_func() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-call_indirect-func") (i32.const 20))
        let ret = call_function(&inst, "as-call_indirect-func", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 20);
    }

    #[test]
    fn test_as_call_indirect_first() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-call_indirect-first") (i32.const 21))
        let ret = call_function(&inst, "as-call_indirect-first", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 21);
    }

    #[test]
    fn test_as_call_indirect_mid() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-call_indirect-mid") (i32.const 22))
        let ret = call_function(&inst, "as-call_indirect-mid", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 22);
    }

    #[test]
    fn test_as_call_indirect_last() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-call_indirect-last") (i32.const 23))
        let ret = call_function(&inst, "as-call_indirect-last", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 23);
    }

    #[test]
    fn test_as_call_indirect_all() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-call_indirect-all") (i32.const 24))
        let ret = call_function(&inst, "as-call_indirect-all", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 24);
    }

    #[test]
    fn test_as_local_set_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-local.set-value") (i32.const 17))
        let ret = call_function(&inst, "as-local.set-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 17);
    }

    #[test]
    fn test_as_local_tee_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-local.tee-value") (i32.const 1))
        let ret = call_function(&inst, "as-local.tee-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_global_set_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-global.set-value") (i32.const 1))
        let ret = call_function(&inst, "as-global.set-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_load_address() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-load-address") (f32.const 1.7))
        let ret = call_function(&inst, "as-load-address", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.7);
    }

    #[test]
    fn test_as_loadn_address() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-loadN-address") (i64.const 30))
        let ret = call_function(&inst, "as-loadN-address", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 30);
    }

    #[test]
    fn test_as_store_address() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-store-address") (i32.const 30))
        let ret = call_function(&inst, "as-store-address", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 30);
    }

    #[test]
    fn test_as_store_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-store-value") (i32.const 31))
        let ret = call_function(&inst, "as-store-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 31);
    }

    #[test]
    fn test_as_store_both() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-store-both") (i32.const 32))
        let ret = call_function(&inst, "as-store-both", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 32);
    }

    #[test]
    fn test_as_storen_address() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-storeN-address") (i32.const 32))
        let ret = call_function(&inst, "as-storeN-address", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 32);
    }

    #[test]
    fn test_as_storen_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-storeN-value") (i32.const 33))
        let ret = call_function(&inst, "as-storeN-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 33);
    }

    #[test]
    fn test_as_storen_both() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-storeN-both") (i32.const 34))
        let ret = call_function(&inst, "as-storeN-both", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 34);
    }

    #[test]
    fn test_as_unary_operand() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-unary-operand") (f32.const 3.4))
        let ret = call_function(&inst, "as-unary-operand", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 3.4);
    }

    #[test]
    fn test_as_binary_left() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-binary-left") (i32.const 3))
        let ret = call_function(&inst, "as-binary-left", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_as_binary_right() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-binary-right") (i64.const 45))
        let ret = call_function(&inst, "as-binary-right", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 45);
    }

    #[test]
    fn test_as_binary_both() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-binary-both") (i32.const 46))
        let ret = call_function(&inst, "as-binary-both", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 46);
    }

    #[test]
    fn test_as_test_operand() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-test-operand") (i32.const 44))
        let ret = call_function(&inst, "as-test-operand", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 44);
    }

    #[test]
    fn test_as_compare_left() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-compare-left") (i32.const 43))
        let ret = call_function(&inst, "as-compare-left", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 43);
    }

    #[test]
    fn test_as_compare_right() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-compare-right") (i32.const 42))
        let ret = call_function(&inst, "as-compare-right", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 42);
    }

    #[test]
    fn test_as_compare_both() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-compare-both") (i32.const 44))
        let ret = call_function(&inst, "as-compare-both", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 44);
    }

    #[test]
    fn test_as_convert_operand() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-convert-operand") (i32.const 41))
        let ret = call_function(&inst, "as-convert-operand", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 41);
    }

    #[test]
    fn test_as_memory_grow_size() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "as-memory.grow-size") (i32.const 40))
        let ret = call_function(&inst, "as-memory.grow-size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 40);
    }

    #[test]
    fn test_nested_block_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "nested-block-value") (i32.const 9))
        let ret = call_function(&inst, "nested-block-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_nested_br_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "nested-br-value") (i32.const 9))
        let ret = call_function(&inst, "nested-br-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_nested_br_if_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "nested-br_if-value") (i32.const 9))
        let ret = call_function(&inst, "nested-br_if-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_nested_br_if_value_cond() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "nested-br_if-value-cond") (i32.const 9))
        let ret = call_function(&inst, "nested-br_if-value-cond", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_nested_br_table_value() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "nested-br_table-value") (i32.const 9))
        let ret = call_function(&inst, "nested-br_table-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_nested_br_table_value_index() {
        let inst = load_instance("tests/wasm/br.wasm");
        //(assert_return (invoke "nested-br_table-value-index") (i32.const 9))
        let ret = call_function(&inst, "nested-br_table-value-index", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }
}
