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
        let mut runtime = Runtime::new(Arc::clone(inst), &func_addr, params)?;
        runtime.run()
    }

    #[test]
    fn test_empty() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "empty", vec![Val::Num(Num::I32(0))]);
        assert!(ret.is_ok());
        let ret = call_function(&inst, "empty", vec![Val::Num(Num::I32(1))]);
        assert!(ret.is_ok());
        let ret = call_function(&inst, "empty", vec![Val::Num(Num::I32(100))]);
        assert!(ret.is_ok());
        let ret = call_function(&inst, "empty", vec![Val::Num(Num::I32(-2))]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_singular() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "singular", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 8);
        let ret = call_function(&inst, "singular", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 7);
        let ret = call_function(&inst, "singular", vec![Val::Num(Num::I32(10))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 7);
        let ret = call_function(&inst, "singular", vec![Val::Num(Num::I32(-10))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 7);
    }

    #[test]
    fn test_nested() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(
            &inst,
            "nested",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 11);
        let ret = call_function(
            &inst,
            "nested",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 10);
        let ret = call_function(
            &inst,
            "nested",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 10);
        let ret = call_function(
            &inst,
            "nested",
            vec![Val::Num(Num::I32(3)), Val::Num(Num::I32(2))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
        let ret = call_function(
            &inst,
            "nested",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(-100))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 10);
        let ret = call_function(
            &inst,
            "nested",
            vec![Val::Num(Num::I32(10)), Val::Num(Num::I32(10))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
        let ret = call_function(
            &inst,
            "nested",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 10);
        let ret = call_function(
            &inst,
            "nested",
            vec![Val::Num(Num::I32(-111)), Val::Num(Num::I32(-2))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_as_select_first() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-select-first", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-select-first", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_select_mid() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-select-mid", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(&inst, "as-select-mid", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_select_last() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-select-last", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
        let ret = call_function(&inst, "as-select-last", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_loop_first() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-loop-first", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-loop-first", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_loop_mid() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-loop-mid", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-loop-mid", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_loop_last() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-loop-last", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-loop-last", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_if_condition() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-if-condition", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
        let ret = call_function(&inst, "as-if-condition", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_br_if_first() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-br_if-first", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-br_if-first", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_br_if_last() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-br_if-last", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
        let ret = call_function(&inst, "as-br_if-last", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_br_table_first() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-br_table-first", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-br_table-first", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_br_table_last() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-br_table-last", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(&inst, "as-br_table-last", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_call_indirect_first() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-call_indirect-first", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-call_indirect-first", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_call_indirect_mid() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-call_indirect-mid", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(&inst, "as-call_indirect-mid", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_call_indirect_last() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-call_indirect-last", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_store_first() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-store-first", vec![Val::Num(Num::I32(0))]);
        assert!(ret.is_ok());
        let ret = call_function(&inst, "as-store-first", vec![Val::Num(Num::I32(1))]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_store_last() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-store-last", vec![Val::Num(Num::I32(0))]);
        assert!(ret.is_ok());
        let ret = call_function(&inst, "as-store-last", vec![Val::Num(Num::I32(1))]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_memory_grow_value() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-memory.grow-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "as-memory.grow-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_call_value() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-call-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-call-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_return_value() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-return-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-return-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_drop_operand() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-drop-operand", vec![Val::Num(Num::I32(0))]);
        assert!(ret.is_ok());
        let ret = call_function(&inst, "as-drop-operand", vec![Val::Num(Num::I32(1))]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_br_value() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-br-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-br-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_local_set_value() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-local.set-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-local.set-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_local_tee_value() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-local.tee-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-local.tee-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_global_set_value() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-global.set-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-global.set-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_load_operand() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-load-operand", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-load-operand", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_as_unary_operand() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-unary-operand", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-unary-operand", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "as-unary-operand", vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_as_binary_operand() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(
            &inst,
            "as-binary-operand",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 15);
        let ret = call_function(
            &inst,
            "as-binary-operand",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -12);
        let ret = call_function(
            &inst,
            "as-binary-operand",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -15);
        let ret = call_function(
            &inst,
            "as-binary-operand",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 12);
    }

    #[test]
    fn test_as_test_operand() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-test-operand", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "as-test-operand", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_as_compare_operand() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(
            &inst,
            "as-compare-operand",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "as-compare-operand",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "as-compare-operand",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "as-compare-operand",
            vec![Val::Num(Num::I32(1)), Val::Num(Num::I32(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_as_binary_operands() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-binary-operands", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -12);
        let ret = call_function(&inst, "as-binary-operands", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 12);
    }

    #[test]
    fn test_as_compare_operands() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-compare-operands", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "as-compare-operands", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_as_mixed_operands() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "as-mixed-operands", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -3);
        let ret = call_function(&inst, "as-mixed-operands", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 27);
    }

    #[test]
    fn test_break_bare() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "break-bare", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 19);
    }

    #[test]
    fn test_break_value() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "break-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 18);
        let ret = call_function(&inst, "break-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 21);
    }

    #[test]
    fn test_param() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "param", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(&inst, "param", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_params() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "params", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(&inst, "params", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_params_id() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "params-id", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
        let ret = call_function(&inst, "params-id", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_param_break() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "param-break", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(&inst, "param-break", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_params_break() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "params-break", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(&inst, "params-break", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_params_id_break() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "params-id-break", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
        let ret = call_function(&inst, "params-id-break", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_effects() {
        let inst = load_instance("tests/wasm/if.wasm");
        let ret = call_function(&inst, "effects", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -14);
        let ret = call_function(&inst, "effects", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -6);
    }
}
