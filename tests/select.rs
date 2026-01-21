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
        let _ = parser::parse_bytecode(&mut module, wasm_path, true);
        let imports: ImportObjects = FxHashMap::default();
        ModuleInst::new(&module, imports, Vec::new()).unwrap()
    }

    fn call_function(
        inst: &Rc<ModuleInst>,
        func_name: &str,
        params: Vec<Val>,
    ) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func(func_name)?;
        let mut runtime = Runtime::new(Rc::clone(inst), &func_addr, params, true, false)?;
        runtime.run()
    }

    #[test]
    fn test_select_i32_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "select-i32" (i32.const 1) (i32.const 2) (i32.const 1)) (i32.const 1))
        let ret = call_function(
            &inst,
            "select-i32",
            vec![
                Val::Num(Num::I32(1)),
                Val::Num(Num::I32(2)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        // (assert_return (invoke "select-i32" (i32.const 1) (i32.const 2) (i32.const 0)) (i32.const 2))
        let ret = call_function(
            &inst,
            "select-i32",
            vec![
                Val::Num(Num::I32(1)),
                Val::Num(Num::I32(2)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);

        // (assert_return (invoke "select-i32" (i32.const 2) (i32.const 1) (i32.const 0)) (i32.const 1))
        let ret = call_function(
            &inst,
            "select-i32",
            vec![
                Val::Num(Num::I32(2)),
                Val::Num(Num::I32(1)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_select_i64_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "select-i64" (i64.const 2) (i64.const 1) (i32.const 1)) (i64.const 2))
        let ret = call_function(
            &inst,
            "select-i64",
            vec![
                Val::Num(Num::I64(2)),
                Val::Num(Num::I64(1)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 2);

        // (assert_return (invoke "select-i64" (i64.const 2) (i64.const 1) (i32.const -1)) (i64.const 2))
        let ret = call_function(
            &inst,
            "select-i64",
            vec![
                Val::Num(Num::I64(2)),
                Val::Num(Num::I64(1)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 2);

        // (assert_return (invoke "select-i64" (i64.const 2) (i64.const 1) (i32.const 0xf0f0f0f0)) (i64.const 2))
        let ret = call_function(
            &inst,
            "select-i64",
            vec![
                Val::Num(Num::I64(2)),
                Val::Num(Num::I64(1)),
                Val::Num(Num::I32(0xf0f0f0f0u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 2);
    }

    #[test]
    fn test_select_f32_spec() {
        let inst = load_instance("tests/wasm/select.wasm");
        //(assert_return (invoke "select-f32" (f32.const 1) (f32.const 2) (i32.const 1)) (f32.const 1))
        let ret = call_function(
            &inst,
            "select-f32",
            vec![
                Val::Num(Num::F32(1.0)),
                Val::Num(Num::F32(2.0)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        // (assert_return (invoke "select-f32" (f32.const nan) (f32.const 1) (i32.const 1)) (f32.const nan))
        let ret = call_function(
            &inst,
            "select-f32",
            vec![
                Val::Num(Num::F32(f32::NAN)),
                Val::Num(Num::F32(1.0)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        // (assert_return (invoke "select-f32" (f32.const nan:0x20304) (f32.const 1) (i32.const 1)) (f32.const nan:0x20304))
        let ret = call_function(
            &inst,
            "select-f32",
            vec![
                Val::Num(Num::F32(f32::from_bits(0x7fc20304))),
                Val::Num(Num::F32(1.0)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        // (assert_return (invoke "select-f32" (f32.const nan) (f32.const 1) (i32.const 0)) (f32.const 1))
        let ret = call_function(
            &inst,
            "select-f32",
            vec![
                Val::Num(Num::F32(f32::NAN)),
                Val::Num(Num::F32(1.0)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        // (assert_return (invoke "select-f32" (f32.const nan:0x20304) (f32.const 1) (i32.const 0)) (f32.const 1))
        let ret = call_function(
            &inst,
            "select-f32",
            vec![
                Val::Num(Num::F32(f32::from_bits(0x7fc20304))),
                Val::Num(Num::F32(1.0)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        // (assert_return (invoke "select-f32" (f32.const 2) (f32.const nan) (i32.const 1)) (f32.const 2))
        let ret = call_function(
            &inst,
            "select-f32",
            vec![
                Val::Num(Num::F32(2.0)),
                Val::Num(Num::F32(f32::NAN)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        // (assert_return (invoke "select-f32" (f32.const 2) (f32.const nan:0x20304) (i32.const 1)) (f32.const 2))
        let ret = call_function(
            &inst,
            "select-f32",
            vec![
                Val::Num(Num::F32(2.0)),
                Val::Num(Num::F32(f32::from_bits(0x7fc20304))),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        // (assert_return (invoke "select-f32" (f32.const 2) (f32.const nan) (i32.const 0)) (f32.const nan))
        let ret = call_function(
            &inst,
            "select-f32",
            vec![
                Val::Num(Num::F32(2.0)),
                Val::Num(Num::F32(f32::NAN)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        // (assert_return (invoke "select-f32" (f32.const 2) (f32.const nan:0x20304) (i32.const 0)) (f32.const nan:0x20304))
        let ret = call_function(
            &inst,
            "select-f32",
            vec![
                Val::Num(Num::F32(2.0)),
                Val::Num(Num::F32(f32::from_bits(0x7fc20304))),
                Val::Num(Num::I32(0)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());
    }

    #[test]
    fn test_select_f64_spec() {
        let inst = load_instance("tests/wasm/select.wasm");
        //(assert_return (invoke "select-f64" (f64.const 1) (f64.const 2) (i32.const 1)) (f64.const 1))
        let ret = call_function(
            &inst,
            "select-f64",
            vec![
                Val::Num(Num::F64(1.0)),
                Val::Num(Num::F64(2.0)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f64().unwrap(), 1.0);

        // (assert_return (invoke "select-f64" (f64.const nan) (f64.const 1) (i32.const 1)) (f64.const nan))
        let ret = call_function(
            &inst,
            "select-f64",
            vec![
                Val::Num(Num::F64(f64::NAN)),
                Val::Num(Num::F64(1.0)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());

        // (assert_return (invoke "select-f64" (f64.const nan:0x4020304) (f64.const 1) (i32.const 1)) (f64.const nan:0x4020304))
        let ret = call_function(
            &inst,
            "select-f64",
            vec![
                Val::Num(Num::F64(f64::from_bits(0x7ff8400203040000))),
                Val::Num(Num::F64(1.0)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());

        // (assert_return (invoke "select-f64" (f64.const nan) (f64.const 1) (i32.const 0)) (f64.const 1))
        let ret = call_function(
            &inst,
            "select-f64",
            vec![
                Val::Num(Num::F64(f64::NAN)),
                Val::Num(Num::F64(1.0)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f64().unwrap(), 1.0);

        // (assert_return (invoke "select-f64" (f64.const nan:0x4020304) (f64.const 1) (i32.const 0)) (f64.const 1))
        let ret = call_function(
            &inst,
            "select-f64",
            vec![
                Val::Num(Num::F64(f64::from_bits(0x7ff8400203040000))),
                Val::Num(Num::F64(1.0)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f64().unwrap(), 1.0);

        // (assert_return (invoke "select-f64" (f64.const 2) (f64.const nan) (i32.const 1)) (f64.const 2))
        let ret = call_function(
            &inst,
            "select-f64",
            vec![
                Val::Num(Num::F64(2.0)),
                Val::Num(Num::F64(f64::NAN)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f64().unwrap(), 2.0);

        // (assert_return (invoke "select-f64" (f64.const 2) (f64.const nan:0x4020304) (i32.const 1)) (f64.const 2))
        let ret = call_function(
            &inst,
            "select-f64",
            vec![
                Val::Num(Num::F64(2.0)),
                Val::Num(Num::F64(f64::from_bits(0x7ff8400203040000))),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f64().unwrap(), 2.0);

        // (assert_return (invoke "select-f64" (f64.const 2) (f64.const nan) (i32.const 0)) (f64.const nan))
        let ret = call_function(
            &inst,
            "select-f64",
            vec![
                Val::Num(Num::F64(2.0)),
                Val::Num(Num::F64(f64::NAN)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());

        // (assert_return (invoke "select-f64" (f64.const 2) (f64.const nan:0x4020304) (i32.const 0)) (f64.const nan:0x4020304))
        let ret = call_function(
            &inst,
            "select-f64",
            vec![
                Val::Num(Num::F64(2.0)),
                Val::Num(Num::F64(f64::from_bits(0x7ff8400203040000))),
                Val::Num(Num::I32(0)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());
    }

    #[test]
    fn test_select_typed_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "select-i32-t" (i32.const 1) (i32.const 2) (i32.const 1)) (i32.const 1))
        let ret = call_function(
            &inst,
            "select-i32-t",
            vec![
                Val::Num(Num::I32(1)),
                Val::Num(Num::I32(2)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        //(assert_return (invoke "select-i32-t" (i32.const 1) (i32.const 2) (i32.const 0)) (i32.const 2))
        let ret = call_function(
            &inst,
            "select-i32-t",
            vec![
                Val::Num(Num::I32(1)),
                Val::Num(Num::I32(2)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        // (assert_return (invoke "select-i32-t" (i32.const 2) (i32.const 1) (i32.const 0)) (i32.const 1))
        let ret = call_function(
            &inst,
            "select-i32-t",
            vec![
                Val::Num(Num::I32(2)),
                Val::Num(Num::I32(1)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        // (assert_return (invoke "select-i64-t" (i64.const 2) (i64.const 1) (i32.const 1)) (i64.const 2))
        let ret = call_function(
            &inst,
            "select-i64-t",
            vec![
                Val::Num(Num::I64(2)),
                Val::Num(Num::I64(1)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 2);
        //(assert_return (invoke "select-i64-t" (i64.const 2) (i64.const 1) (i32.const -1)) (i64.const 2))
        let ret = call_function(
            &inst,
            "select-i64-t",
            vec![
                Val::Num(Num::I64(2)),
                Val::Num(Num::I64(1)),
                Val::Num(Num::I32(-1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 2);
        //(assert_return (invoke "select-i64-t" (i64.const 2) (i64.const 1) (i32.const 0xf0f0f0f0)) (i64.const 2))
        let ret = call_function(
            &inst,
            "select-i64-t",
            vec![
                Val::Num(Num::I64(2)),
                Val::Num(Num::I64(1)),
                Val::Num(Num::I32(0xf0f0f0f0u32 as i32)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 2);

        // (assert_return (invoke "select-f32-t" (f32.const nan) (f32.const 1) (i32.const 1)) (f32.const nan))
        let ret = call_function(
            &inst,
            "select-f32-t",
            vec![
                Val::Num(Num::F32(f32::NAN)),
                Val::Num(Num::F32(1.0)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        // (assert_return (invoke "select-f32-t" (f32.const nan:0x20304) (f32.const 1) (i32.const 1)) (f32.const nan:0x20304))
        let ret = call_function(
            &inst,
            "select-f32-t",
            vec![
                Val::Num(Num::F32(f32::from_bits(0x7fc20304))),
                Val::Num(Num::F32(1.0)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        // (assert_return (invoke "select-f32-t" (f32.const nan) (f32.const 1) (i32.const 0)) (f32.const 1))
        let ret = call_function(
            &inst,
            "select-f32-t",
            vec![
                Val::Num(Num::F32(f32::NAN)),
                Val::Num(Num::F32(1.0)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        // (assert_return (invoke "select-f32-t" (f32.const nan:0x20304) (f32.const 1) (i32.const 0)) (f32.const 1))
        let ret = call_function(
            &inst,
            "select-f32-t",
            vec![
                Val::Num(Num::F32(f32::from_bits(0x7fc20304))),
                Val::Num(Num::F32(1.0)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        // (assert_return (invoke "select-f32-t" (f32.const 2) (f32.const nan) (i32.const 1)) (f32.const 2))
        let ret = call_function(
            &inst,
            "select-f32-t",
            vec![
                Val::Num(Num::F32(2.0)),
                Val::Num(Num::F32(f32::NAN)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        // (assert_return (invoke "select-f32-t" (f32.const 2) (f32.const nan:0x20304) (i32.const 1)) (f32.const 2))
        let ret = call_function(
            &inst,
            "select-f32-t",
            vec![
                Val::Num(Num::F32(2.0)),
                Val::Num(Num::F32(f32::from_bits(0x7fc20304))),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        // (assert_return (invoke "select-f32-t" (f32.const 2) (f32.const nan) (i32.const 0)) (f32.const nan))
        let ret = call_function(
            &inst,
            "select-f32-t",
            vec![
                Val::Num(Num::F32(2.0)),
                Val::Num(Num::F32(f32::NAN)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        // (assert_return (invoke "select-f32-t" (f32.const 2) (f32.const nan:0x20304) (i32.const 0)) (f32.const nan:0x20304))
        let ret = call_function(
            &inst,
            "select-f32-t",
            vec![
                Val::Num(Num::F32(2.0)),
                Val::Num(Num::F32(f32::from_bits(0x7fc20304))),
                Val::Num(Num::I32(0)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        // (assert_return (invoke "select-f64-t" (f64.const nan) (f64.const 1) (i32.const 1)) (f64.const nan))
        let ret = call_function(
            &inst,
            "select-f64-t",
            vec![
                Val::Num(Num::F64(f64::NAN)),
                Val::Num(Num::F64(1.0)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());

        // (assert_return (invoke "select-f64-t" (f64.const nan:0x4020304) (f64.const 1) (i32.const 1)) (f64.const nan:0x4020304))
        let ret = call_function(
            &inst,
            "select-f64-t",
            vec![
                Val::Num(Num::F64(f64::from_bits(0x7ff8400203040000))),
                Val::Num(Num::F64(1.0)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());

        // (assert_return (invoke "select-f64-t" (f64.const nan) (f64.const 1) (i32.const 0)) (f64.const 1))
        let ret = call_function(
            &inst,
            "select-f64-t",
            vec![
                Val::Num(Num::F64(f64::NAN)),
                Val::Num(Num::F64(1.0)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f64().unwrap(), 1.0);

        // (assert_return (invoke "select-f64-t" (f64.const nan:0x4020304) (f64.const 1) (i32.const 0)) (f64.const 1))
        let ret = call_function(
            &inst,
            "select-f64-t",
            vec![
                Val::Num(Num::F64(f64::from_bits(0x7ff8400203040000))),
                Val::Num(Num::F64(1.0)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f64().unwrap(), 1.0);

        // (assert_return (invoke "select-f64-t" (f64.const 2) (f64.const nan) (i32.const 1)) (f64.const 2))
        let ret = call_function(
            &inst,
            "select-f64-t",
            vec![
                Val::Num(Num::F64(2.0)),
                Val::Num(Num::F64(f64::NAN)),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f64().unwrap(), 2.0);

        // (assert_return (invoke "select-f64-t" (f64.const 2) (f64.const nan:0x4020304) (i32.const 1)) (f64.const 2))
        let ret = call_function(
            &inst,
            "select-f64-t",
            vec![
                Val::Num(Num::F64(2.0)),
                Val::Num(Num::F64(f64::from_bits(0x7ff8400203040000))),
                Val::Num(Num::I32(1)),
            ],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f64().unwrap(), 2.0);

        // (assert_return (invoke "select-f64-t" (f64.const 2) (f64.const nan) (i32.const 0)) (f64.const nan))
        let ret = call_function(
            &inst,
            "select-f64-t",
            vec![
                Val::Num(Num::F64(2.0)),
                Val::Num(Num::F64(f64::NAN)),
                Val::Num(Num::I32(0)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());

        // (assert_return (invoke "select-f64-t" (f64.const 2) (f64.const nan:0x4020304) (i32.const 0)) (f64.const nan:0x4020304))
        let ret = call_function(
            &inst,
            "select-f64-t",
            vec![
                Val::Num(Num::F64(2.0)),
                Val::Num(Num::F64(f64::from_bits(0x7ff8400203040000))),
                Val::Num(Num::I32(0)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());

        // Note: funcref and externref tests require reference types support
    }

    #[test]
    fn test_as_select_first_spec() {
        let inst = load_instance("tests/wasm/select.wasm");
        // (assert_return (invoke "as-select-first" (i32.const 0)) (i32.const 1))
        let ret = call_function(&inst, "as-select-first", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        // (assert_return (invoke "as-select-first" (i32.const 1)) (i32.const 0))
        let ret = call_function(&inst, "as-select-first", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_as_select_mid_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-select-mid" (i32.const 0)) (i32.const 2))
        let ret = call_function(&inst, "as-select-mid", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);

        // (assert_return (invoke "as-select-mid" (i32.const 1)) (i32.const 2))
        let ret = call_function(&inst, "as-select-mid", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_select_last_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-select-last" (i32.const 0)) (i32.const 2))
        let ret = call_function(&inst, "as-select-last", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);

        // (assert_return (invoke "as-select-last" (i32.const 1)) (i32.const 3))
        let ret = call_function(&inst, "as-select-last", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_as_loop_first_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-loop-first" (i32.const 0)) (i32.const 3))
        let ret = call_function(&inst, "as-loop-first", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);

        // (assert_return (invoke "as-loop-first" (i32.const 1)) (i32.const 2))
        let ret = call_function(&inst, "as-loop-first", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_loop_mid_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-loop-mid" (i32.const 0)) (i32.const 3))
        let ret = call_function(&inst, "as-loop-mid", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);

        // (assert_return (invoke "as-loop-mid" (i32.const 1)) (i32.const 2))
        let ret = call_function(&inst, "as-loop-mid", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_loop_last_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-loop-last" (i32.const 0)) (i32.const 3))
        let ret = call_function(&inst, "as-loop-last", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);

        // (assert_return (invoke "as-loop-last" (i32.const 1)) (i32.const 2)
        let ret = call_function(&inst, "as-loop-last", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_if_condition_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-if-condition" (i32.const 0)))
        let ret = call_function(&inst, "as-if-condition", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().is_none(), true);
        // (assert_return (invoke "as-if-condition" (i32.const 1)))
        let ret = call_function(&inst, "as-if-condition", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().is_none(), true);
    }

    #[test]
    fn test_as_if_then_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-if-then" (i32.const 0)) (i32.const 3))
        let ret = call_function(&inst, "as-if-then", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);

        // (assert_return (invoke "as-if-then" (i32.const 1)) (i32.const 2))
        let ret = call_function(&inst, "as-if-then", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_if_else_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-if-else" (i32.const 0)) (i32.const 3))
        let ret = call_function(&inst, "as-if-else", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);

        // (assert_return (invoke "as-if-else" (i32.const 1)) (i32.const 2))
        let ret = call_function(&inst, "as-if-else", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_br_if_first_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-br_if-first" (i32.const 0)) (i32.const 3))
        let ret = call_function(&inst, "as-br_if-first", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);

        // (assert_return (invoke "as-br_if-first" (i32.const 1)) (i32.const 2))
        let ret = call_function(&inst, "as-br_if-first", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_br_if_last_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-br_if-last" (i32.const 0)) (i32.const 2))
        let ret = call_function(&inst, "as-br_if-last", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);

        // (assert_return (invoke "as-br_if-last" (i32.const 1)) (i32.const 2))
        let ret = call_function(&inst, "as-br_if-last", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_br_table_first_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-br_table-first" (i32.const 0)) (i32.const 3))
        let ret = call_function(&inst, "as-br_table-first", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);

        // (assert_return (invoke "as-br_table-first" (i32.const 1)) (i32.const 2))
        let ret = call_function(&inst, "as-br_table-first", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_br_table_last_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-br_table-last" (i32.const 0)) (i32.const 2))
        let ret = call_function(&inst, "as-br_table-last", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);

        // (assert_return (invoke "as-br_table-last" (i32.const 1)) (i32.const 2))
        let ret = call_function(&inst, "as-br_table-last", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_call_indirect_first_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-call_indirect-first" (i32.const 0)) (i32.const 3))
        let ret = call_function(&inst, "as-call_indirect-first", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);

        // (assert_return (invoke "as-call_indirect-first" (i32.const 1)) (i32.const 2))
        let ret = call_function(&inst, "as-call_indirect-first", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_as_call_indirect_mid_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-call_indirect-mid" (i32.const 0)) (i32.const 1))
        let ret = call_function(&inst, "as-call_indirect-mid", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        // (assert_return (invoke "as-call_indirect-mid" (i32.const 1)) (i32.const 1))
        let ret = call_function(&inst, "as-call_indirect-mid", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_store_and_load_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-store-first" (i32.const 0)))
        let ret = call_function(&inst, "as-store-first", vec![Val::Num(Num::I32(0))]);
        assert!(ret.is_ok());

        // (assert_return (invoke "as-store-first" (i32.const 1)))
        let ret = call_function(&inst, "as-store-first", vec![Val::Num(Num::I32(1))]);
        assert!(ret.is_ok());

        // (assert_return (invoke "as-store-last" (i32.const 0)))
        let ret = call_function(&inst, "as-store-last", vec![Val::Num(Num::I32(0))]);
        assert!(ret.is_ok());

        // (assert_return (invoke "as-store-last" (i32.const 1)))
        let ret = call_function(&inst, "as-store-last", vec![Val::Num(Num::I32(1))]);
        assert!(ret.is_ok());

        // (assert_return (invoke "as-load-operand" (i32.const 0)) (i32.const 1))
        let ret = call_function(&inst, "as-load-operand", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        // (assert_return (invoke "as-load-operand" (i32.const 1)) (i32.const 1))
        let ret = call_function(&inst, "as-load-operand", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_memory_grow_value_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-memory.grow-value" (i32.const 0)) (i32.const 1))
        let ret = call_function(&inst, "as-memory.grow-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        // (assert_return (invoke "as-memory.grow-value" (i32.const 1)) (i32.const 3))
        let ret = call_function(&inst, "as-memory.grow-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_as_call_value_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-call-value" (i32.const 0)) (i32.const 2))
        let ret = call_function(&inst, "as-call-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);

        // (assert_return (invoke "as-call-value" (i32.const 1)) (i32.const 1))
        let ret = call_function(&inst, "as-call-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_return_value_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-return-value" (i32.const 0)) (i32.const 2))
        let ret = call_function(&inst, "as-return-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);

        // (assert_return (invoke "as-return-value" (i32.const 1)) (i32.const 1))
        let ret = call_function(&inst, "as-return-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_drop_operand_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-drop-operand" (i32.const 0)))
        let ret = call_function(&inst, "as-drop-operand", vec![Val::Num(Num::I32(0))]);
        assert!(ret.is_ok());

        // (assert_return (invoke "as-drop-operand" (i32.const 1)))
        let ret = call_function(&inst, "as-drop-operand", vec![Val::Num(Num::I32(1))]);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_as_br_value_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-br-value" (i32.const 0)) (i32.const 2))
        let ret = call_function(&inst, "as-br-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);

        // (assert_return (invoke "as-br-value" (i32.const 1)) (i32.const 1))
        let ret = call_function(&inst, "as-br-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_local_set_value_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-local.set-value" (i32.const 0)) (i32.const 2))
        let ret = call_function(&inst, "as-local.set-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);

        // (assert_return (invoke "as-local.set-value" (i32.const 1)) (i32.const 1))
        let ret = call_function(&inst, "as-local.set-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_local_tee_value_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-local.tee-value" (i32.const 0)) (i32.const 2))
        let ret = call_function(&inst, "as-local.tee-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);

        // (assert_return (invoke "as-local.tee-value" (i32.const 1)) (i32.const 1))
        let ret = call_function(&inst, "as-local.tee-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_global_set_value_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-global.set-value" (i32.const 0)) (i32.const 2))
        let ret = call_function(&inst, "as-global.set-value", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);

        // (assert_return (invoke "as-global.set-value" (i32.const 1)) (i32.const 1))
        let ret = call_function(&inst, "as-global.set-value", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_unary_operand_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-unary-operand" (i32.const 0)) (i32.const 0))
        let ret = call_function(&inst, "as-unary-operand", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        // (assert_return (invoke "as-unary-operand" (i32.const 1)) (i32.const 1))
        let ret = call_function(&inst, "as-unary-operand", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_binary_operand_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-binary-operand" (i32.const 0)) (i32.const 4))
        let ret = call_function(&inst, "as-binary-operand", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 4);

        // (assert_return (invoke "as-binary-operand" (i32.const 1)) (i32.const 1))
        let ret = call_function(&inst, "as-binary-operand", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_test_operand_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-test-operand" (i32.const 0)) (i32.const 0))
        let ret = call_function(&inst, "as-test-operand", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        // (assert_return (invoke "as-test-operand" (i32.const 1)) (i32.const 1))
        let ret = call_function(&inst, "as-test-operand", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_compare_operand_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        //(assert_return (invoke "as-compare-left" (i32.const 0)) (i32.const 0))
        let ret = call_function(&inst, "as-compare-left", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        //(assert_return (invoke "as-compare-left" (i32.const 1)) (i32.const 1))
        let ret = call_function(&inst, "as-compare-left", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        //(assert_return (invoke "as-compare-right" (i32.const 0)) (i32.const 0))
        let ret = call_function(&inst, "as-compare-right", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        //(assert_return (invoke "as-compare-right" (i32.const 1)) (i32.const 1))
        let ret = call_function(&inst, "as-compare-right", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_as_convert_operand_spec() {
        let inst = load_instance("tests/wasm/select.wasm");

        // (assert_return (invoke "as-convert-operand" (i32.const 0)) (i32.const 0))
        let ret = call_function(&inst, "as-convert-operand", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        // (assert_return (invoke "as-convert-operand" (i32.const 1)) (i32.const 1))
        let ret = call_function(&inst, "as-convert-operand", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }
}
