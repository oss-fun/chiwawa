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
        let _ = parser::parse_bytecode(&mut module, wasm_path, true);
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
    fn test_call_indirect_type_i32() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "type-i32", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x132);
    }

    #[test]
    fn test_call_indirect_type_i64() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "type-i64", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0x164);
    }

    #[test]
    fn test_call_indirect_type_f32() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "type-f32", vec![]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            3890.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_type_f64() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "type-f64", vec![]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            3940.0_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_type_index() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "type-index", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 100);
    }

    #[test]
    fn test_call_indirect_type_first_i32() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "type-first-i32", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 32);
    }

    #[test]
    fn test_call_indirect_type_first_i64() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "type-first-i64", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 64);
    }

    #[test]
    fn test_call_indirect_type_first_f32() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "type-first-f32", vec![]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            1.32_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_type_first_f64() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "type-first-f64", vec![]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            1.64_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_type_second_i32() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "type-second-i32", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 32);
    }

    #[test]
    fn test_call_indirect_type_second_i64() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "type-second-i64", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 64);
    }

    #[test]
    fn test_call_indirect_type_second_f32() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "type-second-f32", vec![]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            32.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_type_second_f64() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "type-second-f64", vec![]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            64.1_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_type_all_i32_i64() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "type-all-i32-i64", vec![]).unwrap();
        assert_eq!(ret.len(), 2);
        assert_eq!(ret[0].to_i64().unwrap(), 2);
        assert_eq!(ret[1].to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_dispatch() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(
            &inst,
            "dispatch",
            vec![Val::Num(Num::I32(5)), Val::Num(Num::I64(64))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 64);
        let ret = call_function(
            &inst,
            "dispatch",
            vec![Val::Num(Num::I32(5)), Val::Num(Num::I64(123))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 123);
    }

    #[test]
    fn test_call_indirect_dispatch_structural_i64() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(
            &inst,
            "dispatch-structural-i64",
            vec![Val::Num(Num::I32(20))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 9);
    }

    #[test]
    fn test_call_indirect_dispatch_structural_i32() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(
            &inst,
            "dispatch-structural-i32",
            vec![Val::Num(Num::I32(19))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_call_indirect_dispatch_structural_f32() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(
            &inst,
            "dispatch-structural-f32",
            vec![Val::Num(Num::I32(21))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            9.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_dispatch_structural_f64() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(
            &inst,
            "dispatch-structural-f64",
            vec![Val::Num(Num::I32(22))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            9.0_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_fac_i64() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "fac-i64", vec![Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(&inst, "fac-i64", vec![Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(&inst, "fac-i64", vec![Val::Num(Num::I64(5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 120);
        let ret = call_function(&inst, "fac-i64", vec![Val::Num(Num::I64(10))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 3628800);
    }

    #[test]
    fn test_call_indirect_fib_i64() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "fib-i64", vec![Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(&inst, "fib-i64", vec![Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(&inst, "fib-i64", vec![Val::Num(Num::I64(2))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 2);
        let ret = call_function(&inst, "fib-i64", vec![Val::Num(Num::I64(5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 8);
        let ret = call_function(&inst, "fib-i64", vec![Val::Num(Num::I64(10))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 89);
    }

    #[test]
    fn test_call_indirect_fac_i32() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "fac-i32", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "fac-i32", vec![Val::Num(Num::I32(5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 120);
    }

    #[test]
    fn test_call_indirect_fac_f32() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "fac-f32", vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            1.0_f32.to_bits()
        );
        let ret = call_function(&inst, "fac-f32", vec![Val::Num(Num::F32(5.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            120.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_fac_f64() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "fac-f64", vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            1.0_f64.to_bits()
        );
        let ret = call_function(&inst, "fac-f64", vec![Val::Num(Num::F64(5.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            120.0_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_fib_i32() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "fib-i32", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "fib-i32", vec![Val::Num(Num::I32(5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 8);
    }

    #[test]
    fn test_call_indirect_fib_f32() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "fib-f32", vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            1.0_f32.to_bits()
        );
        let ret = call_function(&inst, "fib-f32", vec![Val::Num(Num::F32(5.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            8.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_fib_f64() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "fib-f64", vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            1.0_f64.to_bits()
        );
        let ret = call_function(&inst, "fib-f64", vec![Val::Num(Num::F64(5.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            8.0_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_even_odd() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "even", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 44);
        let ret = call_function(&inst, "even", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 99);
        let ret = call_function(&inst, "even", vec![Val::Num(Num::I32(100))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 44);
        let ret = call_function(&inst, "even", vec![Val::Num(Num::I32(77))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 99);
        let ret = call_function(&inst, "odd", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 99);
        let ret = call_function(&inst, "odd", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 44);
        let ret = call_function(&inst, "odd", vec![Val::Num(Num::I32(200))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 99);
        let ret = call_function(&inst, "odd", vec![Val::Num(Num::I32(77))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 44);
    }

    #[test]
    fn test_call_indirect_as_select_first() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-select-first", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x132);
    }

    #[test]
    fn test_call_indirect_as_select_mid() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-select-mid", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_call_indirect_as_select_last() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-select-last", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_call_indirect_as_if_condition() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-if-condition", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_as_br_if_first() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-br_if-first", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0x164);
    }

    #[test]
    fn test_call_indirect_as_br_if_last() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-br_if-last", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_call_indirect_as_br_table_first() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-br_table-first", vec![]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            3890.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_as_br_table_last() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-br_table-last", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_call_indirect_as_store_first() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let result = call_function(&inst, "as-store-first", vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_indirect_as_store_last() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let result = call_function(&inst, "as-store-last", vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_indirect_as_memory_grow_value() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-memory.grow-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_as_return_value() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-return-value", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_as_drop_operand() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let result = call_function(&inst, "as-drop-operand", vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_indirect_as_br_value() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-br-value", vec![]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            1.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_as_local_set_value() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-local.set-value", vec![]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            1.0_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_as_local_tee_value() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-local.tee-value", vec![]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            1.0_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_as_global_set_value() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-global.set-value", vec![]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            1.0_f64.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_as_load_operand() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-load-operand", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_call_indirect_as_unary_operand() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-unary-operand", vec![]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0.0_f32.to_bits()
        );
    }

    #[test]
    fn test_call_indirect_as_binary_left() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-binary-left", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 11);
    }

    #[test]
    fn test_call_indirect_as_binary_right() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-binary-right", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }

    #[test]
    fn test_call_indirect_as_test_operand() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-test-operand", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_call_indirect_as_compare_left() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-compare-left", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_as_compare_right() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-compare-right", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_call_indirect_as_convert_operand() {
        let inst = load_instance("tests/wasm/call_indirect.wasm");
        let ret = call_function(&inst, "as-convert-operand", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
    }
}
