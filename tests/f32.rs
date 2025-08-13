use chiwawa::{
    execution::module::*, execution::runtime::Runtime, execution::value::*, parser,
    structure::module::Module,
};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to load module and get instance
    fn load_instance(wasm_path: &str) -> Arc<ModuleInst> {
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path, true);
        let imports: ImportObjects = HashMap::new();
        ModuleInst::new(&module, imports, Vec::new()).unwrap()
    }

    // Helper function to call a function using Runtime
    fn call_function(
        inst: &Arc<ModuleInst>,
        func_name: &str,
        params: Vec<Val>,
    ) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func(func_name)?;
        let mut runtime = Runtime::new(Arc::clone(inst), &func_addr, params, true)?;
        runtime.run()
    }

    // Helper function to compare f32 values considering NaN and special cases
    fn assert_f32_eq(actual: f32, expected: f32) {
        if expected.is_nan() {
            assert!(actual.is_nan(), "Expected NaN, got {}", actual);
        } else if expected.is_infinite() {
            assert_eq!(actual, expected, "Infinity mismatch");
        } else if expected == 0.0 {
            assert_eq!(actual, expected, "Zero sign mismatch");
        } else {
            assert_eq!(actual, expected, "Value mismatch");
        }
    }

    // --- f32.add tests ---
    #[test]
    fn test_f32_add() {
        let inst = load_instance("tests/wasm/f32.wasm");

        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(1.0)), Val::Num(Num::F32(1.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        // Zero cases
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(0.0)), Val::Num(Num::F32(0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(-0.0)), Val::Num(Num::F32(0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(-0.0)), Val::Num(Num::F32(-0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -0.0);

        // Infinity cases
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(f32::INFINITY)), Val::Num(Num::F32(1.0))],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::INFINITY,
        );

        let ret = call_function(
            &inst,
            "add",
            vec![
                Val::Num(Num::F32(f32::NEG_INFINITY)),
                Val::Num(Num::F32(1.0)),
            ],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::NEG_INFINITY,
        );

        // NaN cases
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(f32::NAN)), Val::Num(Num::F32(1.0))],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        let ret = call_function(
            &inst,
            "add",
            vec![
                Val::Num(Num::F32(f32::INFINITY)),
                Val::Num(Num::F32(f32::NEG_INFINITY)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        // Edge values
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(f32::MAX)), Val::Num(Num::F32(f32::MAX))],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::INFINITY,
        );

        // Subnormal numbers
        let min_positive = f32::MIN_POSITIVE;
        let ret = call_function(
            &inst,
            "add",
            vec![
                Val::Num(Num::F32(min_positive)),
                Val::Num(Num::F32(min_positive)),
            ],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            min_positive * 2.0,
        );
    }

    // --- f32.sub tests ---
    #[test]
    fn test_f32_sub() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Basic subtraction
        let ret = call_function(
            &inst,
            "sub",
            vec![Val::Num(Num::F32(1.0)), Val::Num(Num::F32(1.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        let ret = call_function(
            &inst,
            "sub",
            vec![Val::Num(Num::F32(1.0)), Val::Num(Num::F32(0.5))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.5);

        // Zero cases
        let ret = call_function(
            &inst,
            "sub",
            vec![Val::Num(Num::F32(0.0)), Val::Num(Num::F32(0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        let ret = call_function(
            &inst,
            "sub",
            vec![Val::Num(Num::F32(-0.0)), Val::Num(Num::F32(-0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        let ret = call_function(
            &inst,
            "sub",
            vec![Val::Num(Num::F32(0.0)), Val::Num(Num::F32(-0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        // Infinity cases
        let ret = call_function(
            &inst,
            "sub",
            vec![Val::Num(Num::F32(f32::INFINITY)), Val::Num(Num::F32(1.0))],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::INFINITY,
        );

        let ret = call_function(
            &inst,
            "sub",
            vec![
                Val::Num(Num::F32(f32::INFINITY)),
                Val::Num(Num::F32(f32::INFINITY)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        // NaN cases
        let ret = call_function(
            &inst,
            "sub",
            vec![Val::Num(Num::F32(f32::NAN)), Val::Num(Num::F32(1.0))],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());
    }

    // --- f32.mul tests ---
    #[test]
    fn test_f32_mul() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Basic multiplication
        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::F32(2.0)), Val::Num(Num::F32(3.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 6.0);

        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::F32(-2.0)), Val::Num(Num::F32(3.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -6.0);

        // Zero cases
        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::F32(0.0)), Val::Num(Num::F32(5.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::F32(-0.0)), Val::Num(Num::F32(5.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -0.0);

        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::F32(0.0)), Val::Num(Num::F32(-5.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -0.0);

        // Infinity cases
        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::F32(f32::INFINITY)), Val::Num(Num::F32(2.0))],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::INFINITY,
        );

        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::F32(f32::INFINITY)), Val::Num(Num::F32(-2.0))],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::NEG_INFINITY,
        );

        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::F32(f32::INFINITY)), Val::Num(Num::F32(0.0))],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        // NaN cases
        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::F32(f32::NAN)), Val::Num(Num::F32(1.0))],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        // Overflow to infinity
        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::F32(f32::MAX)), Val::Num(Num::F32(2.0))],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::INFINITY,
        );
    }

    // --- f32.div tests ---
    #[test]
    fn test_f32_div() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Basic division
        let ret = call_function(
            &inst,
            "div",
            vec![Val::Num(Num::F32(6.0)), Val::Num(Num::F32(2.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 3.0);

        let ret = call_function(
            &inst,
            "div",
            vec![Val::Num(Num::F32(-6.0)), Val::Num(Num::F32(2.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -3.0);

        // Division by zero
        let ret = call_function(
            &inst,
            "div",
            vec![Val::Num(Num::F32(1.0)), Val::Num(Num::F32(0.0))],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::INFINITY,
        );

        let ret = call_function(
            &inst,
            "div",
            vec![Val::Num(Num::F32(-1.0)), Val::Num(Num::F32(0.0))],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::NEG_INFINITY,
        );

        let ret = call_function(
            &inst,
            "div",
            vec![Val::Num(Num::F32(0.0)), Val::Num(Num::F32(0.0))],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        // Infinity cases
        let ret = call_function(
            &inst,
            "div",
            vec![Val::Num(Num::F32(f32::INFINITY)), Val::Num(Num::F32(2.0))],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::INFINITY,
        );

        let ret = call_function(
            &inst,
            "div",
            vec![
                Val::Num(Num::F32(f32::INFINITY)),
                Val::Num(Num::F32(f32::INFINITY)),
            ],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        // NaN cases
        let ret = call_function(
            &inst,
            "div",
            vec![Val::Num(Num::F32(f32::NAN)), Val::Num(Num::F32(1.0))],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());
    }

    // --- f32.sqrt tests ---
    #[test]
    fn test_f32_sqrt() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Basic square root
        let ret = call_function(&inst, "sqrt", vec![Val::Num(Num::F32(4.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        let ret = call_function(&inst, "sqrt", vec![Val::Num(Num::F32(9.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 3.0);

        // Zero cases
        let ret = call_function(&inst, "sqrt", vec![Val::Num(Num::F32(0.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        let ret = call_function(&inst, "sqrt", vec![Val::Num(Num::F32(-0.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -0.0);

        // Negative number (should be NaN)
        let ret = call_function(&inst, "sqrt", vec![Val::Num(Num::F32(-1.0))]);
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        // Infinity
        let ret = call_function(&inst, "sqrt", vec![Val::Num(Num::F32(f32::INFINITY))]);
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::INFINITY,
        );

        // NaN
        let ret = call_function(&inst, "sqrt", vec![Val::Num(Num::F32(f32::NAN))]);
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());
    }

    // --- f32.min tests ---
    #[test]
    fn test_f32_min() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Basic minimum
        let ret = call_function(
            &inst,
            "min",
            vec![Val::Num(Num::F32(1.0)), Val::Num(Num::F32(2.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        let ret = call_function(
            &inst,
            "min",
            vec![Val::Num(Num::F32(2.0)), Val::Num(Num::F32(1.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        // Zero cases
        let ret = call_function(
            &inst,
            "min",
            vec![Val::Num(Num::F32(0.0)), Val::Num(Num::F32(-0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -0.0);

        let ret = call_function(
            &inst,
            "min",
            vec![Val::Num(Num::F32(-0.0)), Val::Num(Num::F32(0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -0.0);

        // Infinity cases
        let ret = call_function(
            &inst,
            "min",
            vec![Val::Num(Num::F32(f32::INFINITY)), Val::Num(Num::F32(1.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        let ret = call_function(
            &inst,
            "min",
            vec![
                Val::Num(Num::F32(f32::NEG_INFINITY)),
                Val::Num(Num::F32(1.0)),
            ],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::NEG_INFINITY,
        );

        // WebAssembly spec: NaN propagation
        let ret = call_function(
            &inst,
            "min",
            vec![Val::Num(Num::F32(f32::NAN)), Val::Num(Num::F32(1.0))],
        );
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert!(result.is_nan(), "Expected NaN, got {}", result);

        let ret = call_function(
            &inst,
            "min",
            vec![Val::Num(Num::F32(1.0)), Val::Num(Num::F32(f32::NAN))],
        );
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert!(result.is_nan(), "Expected NaN, got {}", result);
    }

    // --- f32.max tests ---
    #[test]
    fn test_f32_max() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Basic maximum
        let ret = call_function(
            &inst,
            "max",
            vec![Val::Num(Num::F32(1.0)), Val::Num(Num::F32(2.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        let ret = call_function(
            &inst,
            "max",
            vec![Val::Num(Num::F32(2.0)), Val::Num(Num::F32(1.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        // Zero cases
        let ret = call_function(
            &inst,
            "max",
            vec![Val::Num(Num::F32(0.0)), Val::Num(Num::F32(-0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        let ret = call_function(
            &inst,
            "max",
            vec![Val::Num(Num::F32(-0.0)), Val::Num(Num::F32(0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        // Infinity cases
        let ret = call_function(
            &inst,
            "max",
            vec![Val::Num(Num::F32(f32::INFINITY)), Val::Num(Num::F32(1.0))],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::INFINITY,
        );

        let ret = call_function(
            &inst,
            "max",
            vec![
                Val::Num(Num::F32(f32::NEG_INFINITY)),
                Val::Num(Num::F32(1.0)),
            ],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        // WebAssembly spec: NaN propagation
        let ret = call_function(
            &inst,
            "max",
            vec![Val::Num(Num::F32(f32::NAN)), Val::Num(Num::F32(1.0))],
        );
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert!(result.is_nan(), "Expected NaN, got {}", result);

        let ret = call_function(
            &inst,
            "max",
            vec![Val::Num(Num::F32(1.0)), Val::Num(Num::F32(f32::NAN))],
        );
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert!(result.is_nan(), "Expected NaN, got {}", result);
    }

    // --- f32.ceil tests ---
    #[test]
    fn test_f32_ceil() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Basic ceiling
        let ret = call_function(&inst, "ceil", vec![Val::Num(Num::F32(1.5))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        let ret = call_function(&inst, "ceil", vec![Val::Num(Num::F32(-1.5))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -1.0);

        let ret = call_function(&inst, "ceil", vec![Val::Num(Num::F32(1.1))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        let ret = call_function(&inst, "ceil", vec![Val::Num(Num::F32(-1.1))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -1.0);

        // Integer values
        let ret = call_function(&inst, "ceil", vec![Val::Num(Num::F32(2.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        // Zero cases
        let ret = call_function(&inst, "ceil", vec![Val::Num(Num::F32(0.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        let ret = call_function(&inst, "ceil", vec![Val::Num(Num::F32(-0.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -0.0);

        // Infinity and NaN
        let ret = call_function(&inst, "ceil", vec![Val::Num(Num::F32(f32::INFINITY))]);
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::INFINITY,
        );

        let ret = call_function(&inst, "ceil", vec![Val::Num(Num::F32(f32::NEG_INFINITY))]);
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::NEG_INFINITY,
        );

        let ret = call_function(&inst, "ceil", vec![Val::Num(Num::F32(f32::NAN))]);
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());
    }

    // --- f32.floor tests ---
    #[test]
    fn test_f32_floor() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Basic floor
        let ret = call_function(&inst, "floor", vec![Val::Num(Num::F32(1.5))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        let ret = call_function(&inst, "floor", vec![Val::Num(Num::F32(-1.5))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -2.0);

        let ret = call_function(&inst, "floor", vec![Val::Num(Num::F32(1.9))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        let ret = call_function(&inst, "floor", vec![Val::Num(Num::F32(-1.1))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -2.0);

        // Integer values
        let ret = call_function(&inst, "floor", vec![Val::Num(Num::F32(2.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        // Zero cases
        let ret = call_function(&inst, "floor", vec![Val::Num(Num::F32(0.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        let ret = call_function(&inst, "floor", vec![Val::Num(Num::F32(-0.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -0.0);

        // Infinity and NaN
        let ret = call_function(&inst, "floor", vec![Val::Num(Num::F32(f32::INFINITY))]);
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::INFINITY,
        );

        let ret = call_function(&inst, "floor", vec![Val::Num(Num::F32(f32::NEG_INFINITY))]);
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::NEG_INFINITY,
        );

        let ret = call_function(&inst, "floor", vec![Val::Num(Num::F32(f32::NAN))]);
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());
    }

    // --- f32.trunc tests ---
    #[test]
    fn test_f32_trunc() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Basic truncation
        let ret = call_function(&inst, "trunc", vec![Val::Num(Num::F32(1.5))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        let ret = call_function(&inst, "trunc", vec![Val::Num(Num::F32(-1.5))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -1.0);

        let ret = call_function(&inst, "trunc", vec![Val::Num(Num::F32(1.9))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        let ret = call_function(&inst, "trunc", vec![Val::Num(Num::F32(-1.9))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -1.0);

        // Integer values
        let ret = call_function(&inst, "trunc", vec![Val::Num(Num::F32(2.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        // Zero cases
        let ret = call_function(&inst, "trunc", vec![Val::Num(Num::F32(0.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        let ret = call_function(&inst, "trunc", vec![Val::Num(Num::F32(-0.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -0.0);

        // Infinity and NaN
        let ret = call_function(&inst, "trunc", vec![Val::Num(Num::F32(f32::INFINITY))]);
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::INFINITY,
        );

        let ret = call_function(&inst, "trunc", vec![Val::Num(Num::F32(f32::NEG_INFINITY))]);
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::NEG_INFINITY,
        );

        let ret = call_function(&inst, "trunc", vec![Val::Num(Num::F32(f32::NAN))]);
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());
    }

    // --- f32.nearest tests ---
    #[test]
    fn test_f32_nearest() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Basic rounding to nearest
        let ret = call_function(&inst, "nearest", vec![Val::Num(Num::F32(1.4))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        let ret = call_function(&inst, "nearest", vec![Val::Num(Num::F32(1.6))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        let ret = call_function(&inst, "nearest", vec![Val::Num(Num::F32(-1.4))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -1.0);

        let ret = call_function(&inst, "nearest", vec![Val::Num(Num::F32(-1.6))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -2.0);

        // WebAssembly spec: round-to-even (banker's rounding)
        let ret = call_function(&inst, "nearest", vec![Val::Num(Num::F32(0.5))]);
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert_eq!(result, 0.0); // 0.5 should round to nearest even (0)

        let ret = call_function(&inst, "nearest", vec![Val::Num(Num::F32(1.5))]);
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert_eq!(result, 2.0); // 1.5 should round to nearest even (2)

        let ret = call_function(&inst, "nearest", vec![Val::Num(Num::F32(2.5))]);
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert_eq!(result, 2.0); // 2.5 should round to nearest even (2)

        let ret = call_function(&inst, "nearest", vec![Val::Num(Num::F32(-0.5))]);
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert_eq!(result, -0.0); // -0.5 should round to nearest even (-0)

        let ret = call_function(&inst, "nearest", vec![Val::Num(Num::F32(-1.5))]);
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert_eq!(result, -2.0); // -1.5 should round to nearest even (-2)

        // Integer values
        let ret = call_function(&inst, "nearest", vec![Val::Num(Num::F32(2.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 2.0);

        // Zero cases
        let ret = call_function(&inst, "nearest", vec![Val::Num(Num::F32(0.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        let ret = call_function(&inst, "nearest", vec![Val::Num(Num::F32(-0.0))]);
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -0.0);

        // Infinity and NaN
        let ret = call_function(&inst, "nearest", vec![Val::Num(Num::F32(f32::INFINITY))]);
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::INFINITY,
        );

        let ret = call_function(
            &inst,
            "nearest",
            vec![Val::Num(Num::F32(f32::NEG_INFINITY))],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::NEG_INFINITY,
        );

        let ret = call_function(&inst, "nearest", vec![Val::Num(Num::F32(f32::NAN))]);
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());
    }

    // --- Additional comprehensive tests ---

    #[test]
    fn test_f32_edge_cases() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Test very small numbers
        let tiny = f32::from_bits(0x00000001); // Smallest positive subnormal
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(tiny)), Val::Num(Num::F32(tiny))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), tiny * 2.0);

        // Test maximum finite value
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(f32::MAX)), Val::Num(Num::F32(0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), f32::MAX);

        // Test minimum finite value
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(-f32::MAX)), Val::Num(Num::F32(0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), -f32::MAX);
    }

    #[test]
    fn test_f32_special_arithmetic() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Test division edge cases
        let ret = call_function(
            &inst,
            "div",
            vec![
                Val::Num(Num::F32(f32::MIN_POSITIVE)),
                Val::Num(Num::F32(2.0)),
            ],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::MIN_POSITIVE / 2.0,
        );

        // Test multiplication with very small numbers
        let ret = call_function(
            &inst,
            "mul",
            vec![
                Val::Num(Num::F32(f32::MIN_POSITIVE)),
                Val::Num(Num::F32(0.5)),
            ],
        );
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert!(result > 0.0 && result.is_finite());
    }

    #[test]
    fn test_f32_spec_hex_values() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Test specific hex values from WebAssembly spec
        // 0x1p-149 (smallest positive subnormal)
        let val_1p149 = f32::from_bits(0x00000001);
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(val_1p149)), Val::Num(Num::F32(val_1p149))],
        );
        let expected = f32::from_bits(0x00000002); // 0x1p-148
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), expected);

        // 0x1p-126 (smallest positive normal)
        let val_1p126 = f32::from_bits(0x00800000);
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(0.0)), Val::Num(Num::F32(val_1p126))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), val_1p126);

        // 0x1.921fb6p+2 (approximately π)
        let val_pi = f32::from_bits(0x40490fdb);
        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::F32(val_pi)), Val::Num(Num::F32(1.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), val_pi);

        // 0x1.fffffep+127 (largest finite)
        let val_max = f32::from_bits(0x7f7fffff);
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(val_max)), Val::Num(Num::F32(0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), val_max);
    }

    #[test]
    fn test_f32_spec_overflow() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Max + Max should overflow to infinity
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(f32::MAX)), Val::Num(Num::F32(f32::MAX))],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::INFINITY,
        );

        // -Max + (-Max) should overflow to -infinity
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(-f32::MAX)), Val::Num(Num::F32(-f32::MAX))],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            f32::NEG_INFINITY,
        );
    }

    #[test]
    fn test_f32_spec_underflow() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Very small number divided by large number
        let tiny = f32::from_bits(0x00000001);
        let ret = call_function(
            &inst,
            "div",
            vec![Val::Num(Num::F32(tiny)), Val::Num(Num::F32(f32::MAX))],
        );
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_f32_spec_canonical_nan_arithmetic() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Canonical NaN: 0x7fc00000
        let canonical_nan = f32::from_bits(0x7fc00000);
        // Arithmetic NaN: 0x7fa00000
        let arithmetic_nan = f32::from_bits(0x7fa00000);

        // Test NaN propagation in all operations
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(canonical_nan)), Val::Num(Num::F32(1.0))],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        let ret = call_function(
            &inst,
            "sub",
            vec![Val::Num(Num::F32(arithmetic_nan)), Val::Num(Num::F32(1.0))],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::F32(canonical_nan)), Val::Num(Num::F32(1.0))],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        let ret = call_function(
            &inst,
            "div",
            vec![Val::Num(Num::F32(arithmetic_nan)), Val::Num(Num::F32(1.0))],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());
    }

    #[test]
    fn test_f32_spec_precise_zero_signs() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // Precise zero sign handling from WebAssembly spec
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(-0.0)), Val::Num(Num::F32(-0.0))],
        );
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert_eq!(result.to_bits(), (-0.0f32).to_bits());

        let ret = call_function(
            &inst,
            "sub",
            vec![Val::Num(Num::F32(0.0)), Val::Num(Num::F32(0.0))],
        );
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert_eq!(result.to_bits(), 0.0f32.to_bits());

        let ret = call_function(
            &inst,
            "mul",
            vec![Val::Num(Num::F32(-0.0)), Val::Num(Num::F32(1.0))],
        );
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert_eq!(result.to_bits(), (-0.0f32).to_bits());
    }

    #[test]
    fn test_f32_spec_subnormal_arithmetic() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // From WebAssembly spec: operations with subnormal numbers
        let smallest = f32::from_bits(0x00000001); // 0x1p-149
        let second_smallest = f32::from_bits(0x00000002); // 0x1p-148

        // smallest - smallest = 0
        let ret = call_function(
            &inst,
            "sub",
            vec![Val::Num(Num::F32(smallest)), Val::Num(Num::F32(smallest))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        // smallest + smallest = second_smallest
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(smallest)), Val::Num(Num::F32(smallest))],
        );
        assert_f32_eq(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            second_smallest,
        );

        // Test with negative subnormals
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::F32(-smallest)), Val::Num(Num::F32(-smallest))],
        );
        let expected = f32::from_bits(0x80000002); // -0x1p-148
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), expected);
    }

    #[test]
    fn test_f32_spec_min_max_precise() {
        let inst = load_instance("tests/wasm/f32.wasm");

        // WebAssembly spec: precise zero sign handling
        let ret = call_function(
            &inst,
            "min",
            vec![Val::Num(Num::F32(0.0)), Val::Num(Num::F32(-0.0))],
        );
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert_eq!(result.to_bits(), (-0.0f32).to_bits());

        let ret = call_function(
            &inst,
            "max",
            vec![Val::Num(Num::F32(0.0)), Val::Num(Num::F32(-0.0))],
        );
        let result = ret.unwrap().last().unwrap().to_f32().unwrap();
        assert_eq!(result.to_bits(), 0.0f32.to_bits());

        let tiny = f32::from_bits(0x00000001);
        let ret = call_function(
            &inst,
            "min",
            vec![Val::Num(Num::F32(tiny)), Val::Num(Num::F32(0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), 0.0);

        let ret = call_function(
            &inst,
            "max",
            vec![Val::Num(Num::F32(tiny)), Val::Num(Num::F32(0.0))],
        );
        assert_f32_eq(ret.unwrap().last().unwrap().to_f32().unwrap(), tiny);
    }
}
