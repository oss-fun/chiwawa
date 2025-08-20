use chiwawa::{
    execution::module::*, execution::runtime::Runtime, execution::value::*, parser,
    structure::module::Module,
};
use std::collections::HashMap;
use std::rc::Rc;

#[cfg(test)]
mod tests {
    use super::*;

    fn load_instance(wasm_path: &str) -> Rc<ModuleInst> {
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path, true);
        let imports: ImportObjects = HashMap::new();
        ModuleInst::new(&module, imports, Vec::new()).unwrap()
    }

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
    fn test_i64_extend_i32_s() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i64.extend_i32_s", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(&inst, "i64.extend_i32_s", vec![Val::Num(Num::I32(10000))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 10000);
        let ret = call_function(&inst, "i64.extend_i32_s", vec![Val::Num(Num::I32(-10000))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -10000);
        let ret = call_function(&inst, "i64.extend_i32_s", vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);
        let ret = call_function(
            &inst,
            "i64.extend_i32_s",
            vec![Val::Num(Num::I32(0x7fffffff))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x000000007fffffff
        );
        let ret = call_function(
            &inst,
            "i64.extend_i32_s",
            vec![Val::Num(Num::I32(0x80000000u32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0xffffffff80000000u64 as i64
        );
    }
    #[test]
    fn test_i64_extend_i32_u() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i64.extend_i32_u", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(&inst, "i64.extend_i32_u", vec![Val::Num(Num::I32(10000))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 10000);
        let ret = call_function(&inst, "i64.extend_i32_u", vec![Val::Num(Num::I32(-10000))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x00000000ffffd8f0
        );
        let ret = call_function(&inst, "i64.extend_i32_u", vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0xffffffff);
        let ret = call_function(
            &inst,
            "i64.extend_i32_u",
            vec![Val::Num(Num::I32(0x7fffffff))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x000000007fffffff
        );
        let ret = call_function(
            &inst,
            "i64.extend_i32_u",
            vec![Val::Num(Num::I32(0x80000000u32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x0000000080000000
        );
    }
    #[test]
    fn test_i32_wrap_i64() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i32.wrap_i64", vec![Val::Num(Num::I64(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(&inst, "i32.wrap_i64", vec![Val::Num(Num::I64(-100000))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -100000);
        let ret = call_function(&inst, "i32.wrap_i64", vec![Val::Num(Num::I64(0x80000000))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );
        let ret = call_function(
            &inst,
            "i32.wrap_i64",
            vec![Val::Num(Num::I64(0xffffffff7fffffffu64 as i64))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x7fffffff);
        let ret = call_function(
            &inst,
            "i32.wrap_i64",
            vec![Val::Num(Num::I64(0xffffffff00000000u64 as i64))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x00000000);
        let ret = call_function(
            &inst,
            "i32.wrap_i64",
            vec![Val::Num(Num::I64(0xfffffffeffffffffu64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffffffffu32 as i32
        );
        let ret = call_function(
            &inst,
            "i32.wrap_i64",
            vec![Val::Num(Num::I64(0xffffffff00000001u64 as i64))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x00000001);
        let ret = call_function(&inst, "i32.wrap_i64", vec![Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i32.wrap_i64",
            vec![Val::Num(Num::I64(1311768467463790320))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x9abcdef0u32 as i32
        );
        let ret = call_function(
            &inst,
            "i32.wrap_i64",
            vec![Val::Num(Num::I64(0x00000000ffffffff))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffffffffu32 as i32
        );
        let ret = call_function(
            &inst,
            "i32.wrap_i64",
            vec![Val::Num(Num::I64(0x0000000100000000))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x00000000);
        let ret = call_function(
            &inst,
            "i32.wrap_i64",
            vec![Val::Num(Num::I64(0x0000000100000001))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x00000001);
    }
    #[test]
    fn test_i32_trunc_f32_s() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i32.trunc_f32_s", vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "i32.trunc_f32_s", vec![Val::Num(Num::F32(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i32.trunc_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0x1)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i32.trunc_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0x80000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "i32.trunc_f32_s", vec![Val::Num(Num::F32(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "i32.trunc_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0x3f8ccccd)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "i32.trunc_f32_s", vec![Val::Num(Num::F32(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "i32.trunc_f32_s", vec![Val::Num(Num::F32(-1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(
            &inst,
            "i32.trunc_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0xbf8ccccd)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(&inst, "i32.trunc_f32_s", vec![Val::Num(Num::F32(-1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(&inst, "i32.trunc_f32_s", vec![Val::Num(Num::F32(-1.9))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(&inst, "i32.trunc_f32_s", vec![Val::Num(Num::F32(-2.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2);
        let ret = call_function(
            &inst,
            "i32.trunc_f32_s",
            vec![Val::Num(Num::F32(2147483520.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2147483520);
        let ret = call_function(
            &inst,
            "i32.trunc_f32_s",
            vec![Val::Num(Num::F32(-2147483648.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2147483648);
    }

    #[test]
    fn test_i32_trunc_f32_u() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i32.trunc_f32_u", vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "i32.trunc_f32_u", vec![Val::Num(Num::F32(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i32.trunc_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0x1)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i32.trunc_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0x80000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "i32.trunc_f32_u", vec![Val::Num(Num::F32(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "i32.trunc_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0x3f8ccccd)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "i32.trunc_f32_u", vec![Val::Num(Num::F32(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "i32.trunc_f32_u", vec![Val::Num(Num::F32(1.9))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "i32.trunc_f32_u", vec![Val::Num(Num::F32(2.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(
            &inst,
            "i32.trunc_f32_u",
            vec![Val::Num(Num::F32(2147483648.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2147483648);
        let ret = call_function(
            &inst,
            "i32.trunc_f32_u",
            vec![Val::Num(Num::F32(4294967040.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -256);
        let ret = call_function(
            &inst,
            "i32.trunc_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0xbe666666)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i32.trunc_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0xbf7fffff)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }
    #[test]
    fn test_i32_trunc_f64_s() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i32.trunc_f64_s", vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "i32.trunc_f64_s", vec![Val::Num(Num::F64(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0x1)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0x8000000000000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "i32.trunc_f64_s", vec![Val::Num(Num::F64(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0x3ff199999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "i32.trunc_f64_s", vec![Val::Num(Num::F64(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "i32.trunc_f64_s", vec![Val::Num(Num::F64(-1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0xbff199999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(&inst, "i32.trunc_f64_s", vec![Val::Num(Num::F64(-1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(&inst, "i32.trunc_f64_s", vec![Val::Num(Num::F64(-1.9))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(&inst, "i32.trunc_f64_s", vec![Val::Num(Num::F64(-2.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_s",
            vec![Val::Num(Num::F64(2147483647.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2147483647);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_s",
            vec![Val::Num(Num::F64(-2147483648.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2147483648);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_s",
            vec![Val::Num(Num::F64(-2147483648.9))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2147483648);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_s",
            vec![Val::Num(Num::F64(2147483647.9))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2147483647);
    }
    #[test]
    fn test_i32_trunc_f64_u() {
        let inst = load_instance("tests/wasm/conversions.wasm");
        let ret = call_function(&inst, "i32.trunc_f64_u", vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "i32.trunc_f64_u", vec![Val::Num(Num::F64(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0x1)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0x8000000000000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "i32.trunc_f64_u", vec![Val::Num(Num::F64(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0x3ff199999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "i32.trunc_f64_u", vec![Val::Num(Num::F64(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "i32.trunc_f64_u", vec![Val::Num(Num::F64(1.9))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
        let ret = call_function(&inst, "i32.trunc_f64_u", vec![Val::Num(Num::F64(2.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_u",
            vec![Val::Num(Num::F64(2147483648.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2147483648);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_u",
            vec![Val::Num(Num::F64(4294967295.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0xbfeccccccccccccd)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0xbfefffffffffffff)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "i32.trunc_f64_u", vec![Val::Num(Num::F64(1e8))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 100000000);
        let ret = call_function(&inst, "i32.trunc_f64_u", vec![Val::Num(Num::F64(-0.9))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i32.trunc_f64_u",
            vec![Val::Num(Num::F64(4294967295.9))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            4294967295u32 as i32
        );
    }
    #[test]
    fn test_i64_trunc_f32_s() {
        let inst = load_instance("tests/wasm/conversions.wasm");
        let ret = call_function(&inst, "i64.trunc_f32_s", vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(&inst, "i64.trunc_f32_s", vec![Val::Num(Num::F32(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i64.trunc_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0x1)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i64.trunc_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0x80000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(&inst, "i64.trunc_f32_s", vec![Val::Num(Num::F32(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(
            &inst,
            "i64.trunc_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0x3f8ccccd)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(&inst, "i64.trunc_f32_s", vec![Val::Num(Num::F32(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(&inst, "i64.trunc_f32_s", vec![Val::Num(Num::F32(-1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);
        let ret = call_function(
            &inst,
            "i64.trunc_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0xbf8ccccd)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);
        let ret = call_function(&inst, "i64.trunc_f32_s", vec![Val::Num(Num::F32(-1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);
        let ret = call_function(&inst, "i64.trunc_f32_s", vec![Val::Num(Num::F32(-1.9))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);
        let ret = call_function(&inst, "i64.trunc_f32_s", vec![Val::Num(Num::F32(-2.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -2);
        let ret = call_function(
            &inst,
            "i64.trunc_f32_s",
            vec![Val::Num(Num::F32(4294967296.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 4294967296);
        let ret = call_function(
            &inst,
            "i64.trunc_f32_s",
            vec![Val::Num(Num::F32(-4294967296.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -4294967296);
        let ret = call_function(
            &inst,
            "i64.trunc_f32_s",
            vec![Val::Num(Num::F32(9223371487098961920.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            9223371487098961920
        );
        let ret = call_function(
            &inst,
            "i64.trunc_f32_s",
            vec![Val::Num(Num::F32(-9223372036854775808.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            -9223372036854775808
        );
    }
    #[test]
    fn test_i64_trunc_f32_u() {
        let inst = load_instance("tests/wasm/conversions.wasm");
        let ret = call_function(&inst, "i64.trunc_f32_u", vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(&inst, "i64.trunc_f32_u", vec![Val::Num(Num::F32(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i64.trunc_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0x00000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0x80000001)))],
        );
        // Either success with value 0 or IntegerOverflow error is acceptable
        // Depends on the Host Runtime
        if let Ok(result) = &ret {
            assert_eq!(result.last().unwrap().to_i64().unwrap(), 0);
        } else if let Err(err) = &ret {
            assert!(matches!(err, chiwawa::error::RuntimeError::IntegerOverflow));
        }

        let ret = call_function(&inst, "i64.trunc_f32_u", vec![Val::Num(Num::F32(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(
            &inst,
            "i64.trunc_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0x3f8ccccd)))],
        );
        if let Ok(result) = &ret {
            assert_eq!(result.last().unwrap().to_i64().unwrap(), 1);
        } else if let Err(err) = &ret {
            assert!(matches!(err, chiwawa::error::RuntimeError::IntegerOverflow));
        }

        let ret = call_function(&inst, "i64.trunc_f32_u", vec![Val::Num(Num::F32(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(
            &inst,
            "i64.trunc_f32_u",
            vec![Val::Num(Num::F32(4294967296.0))],
        );
        if let Ok(result) = &ret {
            assert_eq!(result.last().unwrap().to_i64().unwrap(), 4294967296);
        } else if let Err(err) = &ret {
            assert!(matches!(err, chiwawa::error::RuntimeError::IntegerOverflow));
        }

        let ret = call_function(
            &inst,
            "i64.trunc_f32_u",
            vec![Val::Num(Num::F32(18446742974197923840.0))],
        );
        if let Ok(result) = &ret {
            assert_eq!(result.last().unwrap().to_i64().unwrap(), -1099511627776);
        } else if let Err(err) = &ret {
            assert!(matches!(err, chiwawa::error::RuntimeError::IntegerOverflow));
        }

        let ret = call_function(
            &inst,
            "i64.trunc_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0xbf666666)))],
        );
        if let Ok(result) = &ret {
            assert_eq!(result.last().unwrap().to_i64().unwrap(), 0);
        } else if let Err(err) = &ret {
            assert!(matches!(err, chiwawa::error::RuntimeError::IntegerOverflow));
        }

        let ret = call_function(
            &inst,
            "i64.trunc_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0xbfffffff)))],
        );
        if let Ok(result) = &ret {
            assert_eq!(result.last().unwrap().to_i64().unwrap(), 0);
        } else if let Err(err) = &ret {
            assert!(matches!(err, chiwawa::error::RuntimeError::IntegerOverflow));
        }
    }
    #[test]
    fn test_i64_trunc_f64_s() {
        let inst = load_instance("tests/wasm/conversions.wasm");
        let ret = call_function(&inst, "i64.trunc_f64_s", vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(&inst, "i64.trunc_f64_s", vec![Val::Num(Num::F64(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0x0000000000000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0x8000000000000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(&inst, "i64.trunc_f64_s", vec![Val::Num(Num::F64(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0x3ff199999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(&inst, "i64.trunc_f64_s", vec![Val::Num(Num::F64(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(&inst, "i64.trunc_f64_s", vec![Val::Num(Num::F64(-1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0xbff199999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);
        let ret = call_function(&inst, "i64.trunc_f64_s", vec![Val::Num(Num::F64(-1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);
        let ret = call_function(&inst, "i64.trunc_f64_s", vec![Val::Num(Num::F64(-1.9))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);
        let ret = call_function(&inst, "i64.trunc_f64_s", vec![Val::Num(Num::F64(-2.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -2);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_s",
            vec![Val::Num(Num::F64(4294967296.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 4294967296);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_s",
            vec![Val::Num(Num::F64(-4294967296.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -4294967296);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_s",
            vec![Val::Num(Num::F64(9223372036854774784.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            9223372036854774784
        );
        let ret = call_function(
            &inst,
            "i64.trunc_f64_s",
            vec![Val::Num(Num::F64(-9223372036854775808.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            -9223372036854775808
        );
    }
    #[test]
    fn test_i64_trunc_f64_u() {
        let inst = load_instance("tests/wasm/conversions.wasm");
        let ret = call_function(&inst, "i64.trunc_f64_u", vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(&inst, "i64.trunc_f64_u", vec![Val::Num(Num::F64(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0x0000000000000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0x8000000000000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(&inst, "i64.trunc_f64_u", vec![Val::Num(Num::F64(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0x3ff199999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(&inst, "i64.trunc_f64_u", vec![Val::Num(Num::F64(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_u",
            vec![Val::Num(Num::F64(4294967295.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0xffffffff);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_u",
            vec![Val::Num(Num::F64(4294967296.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0x100000000);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_u",
            vec![Val::Num(Num::F64(18446744073709549568.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -2048);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0xbfe999999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i64.trunc_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0xbfefffffffffffff)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(&inst, "i64.trunc_f64_u", vec![Val::Num(Num::F64(1e8))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 100000000);
        let ret = call_function(&inst, "i64.trunc_f64_u", vec![Val::Num(Num::F64(1e16))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            10000000000000000
        );
        let ret = call_function(
            &inst,
            "i64.trunc_f64_u",
            vec![Val::Num(Num::F64(9223372036854775808.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            -9223372036854775808
        );
    }
    #[test]
    fn test_f32_convert_i32_s() {
        let inst = load_instance("tests/wasm/conversions.wasm");
        let ret = call_function(&inst, "f32.convert_i32_s", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);
        let ret = call_function(&inst, "f32.convert_i32_s", vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), -1.0);
        let ret = call_function(&inst, "f32.convert_i32_s", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(), 0);
        let ret = call_function(
            &inst,
            "f32.convert_i32_s",
            vec![Val::Num(Num::I32(2147483647))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 2147483648.0);
        let ret = call_function(
            &inst,
            "f32.convert_i32_s",
            vec![Val::Num(Num::I32(-2147483648))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            -2147483648.0
        );
        let ret = call_function(
            &inst,
            "f32.convert_i32_s",
            vec![Val::Num(Num::I32(1234567890))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            1234567890.0f32
        );
        let ret = call_function(
            &inst,
            "f32.convert_i32_s",
            vec![Val::Num(Num::I32(16777217))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 16777216.0);
        let ret = call_function(
            &inst,
            "f32.convert_i32_s",
            vec![Val::Num(Num::I32(-16777217))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), -16777216.0);
        let ret = call_function(
            &inst,
            "f32.convert_i32_s",
            vec![Val::Num(Num::I32(16777219))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 16777220.0);
        let ret = call_function(
            &inst,
            "f32.convert_i32_s",
            vec![Val::Num(Num::I32(-16777219))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), -16777220.0);
    }
    #[test]
    fn test_i32_trunc_sat_f32_s() {
        let inst = load_instance("tests/wasm/conversions.wasm");
        let ret = call_function(&inst, "i32.trunc_sat_f32_s", vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "i32.trunc_sat_f32_s", vec![Val::Num(Num::F32(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0x00000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0x80000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
        let ret = call_function(&inst, "i32.trunc_sat_f32_s", vec![Val::Num(Num::F32(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0x3f8ccccd)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(&inst, "i32.trunc_sat_f32_s", vec![Val::Num(Num::F32(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(&inst, "i32.trunc_sat_f32_s", vec![Val::Num(Num::F32(-1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0xbf8ccccd)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);

        let ret = call_function(&inst, "i32.trunc_sat_f32_s", vec![Val::Num(Num::F32(-1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);

        let ret = call_function(&inst, "i32.trunc_sat_f32_s", vec![Val::Num(Num::F32(-1.9))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);

        let ret = call_function(&inst, "i32.trunc_sat_f32_s", vec![Val::Num(Num::F32(-2.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(2147483520.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2147483520);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(-2147483648.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2147483648);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(2147483648.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x7fffffff);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(-2147483904.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::INFINITY))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x7fffffff);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::NEG_INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0x7fa00000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(-f32::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0xffa00000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }
    #[test]
    fn test_i32_trunc_sat_f32_u() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i32.trunc_sat_f32_u", vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(&inst, "i32.trunc_sat_f32_u", vec![Val::Num(Num::F32(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0x00000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0x80000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(&inst, "i32.trunc_sat_f32_u", vec![Val::Num(Num::F32(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0x3f8ccccd)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(&inst, "i32.trunc_sat_f32_u", vec![Val::Num(Num::F32(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(&inst, "i32.trunc_sat_f32_u", vec![Val::Num(Num::F32(1.9))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(&inst, "i32.trunc_sat_f32_u", vec![Val::Num(Num::F32(2.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(2147483648.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2147483648);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(4294967040.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -256);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0xbf666666)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0xbfffffff)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(4294967296.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffffffffu32 as i32
        );

        let ret = call_function(&inst, "i32.trunc_sat_f32_u", vec![Val::Num(Num::F32(-1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x00000000);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffffffffu32 as i32
        );

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::NEG_INFINITY))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x00000000);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0x7fa00000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(-f32::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0xffa00000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }
    #[test]
    fn test_i32_trunc_sat_f64_s() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i32.trunc_sat_f64_s", vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(&inst, "i32.trunc_sat_f64_s", vec![Val::Num(Num::F64(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0x0000000000000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0x8000000000000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(&inst, "i32.trunc_sat_f64_s", vec![Val::Num(Num::F64(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0x3ff199999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(&inst, "i32.trunc_sat_f64_s", vec![Val::Num(Num::F64(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(&inst, "i32.trunc_sat_f64_s", vec![Val::Num(Num::F64(-1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0xbff199999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);

        let ret = call_function(&inst, "i32.trunc_sat_f64_s", vec![Val::Num(Num::F64(-1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);

        let ret = call_function(&inst, "i32.trunc_sat_f64_s", vec![Val::Num(Num::F64(-1.9))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);

        let ret = call_function(&inst, "i32.trunc_sat_f64_s", vec![Val::Num(Num::F64(-2.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(2147483647.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2147483647);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(-2147483648.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2147483648);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(2147483648.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x7fffffff);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(-2147483649.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::INFINITY))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x7fffffff);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::NEG_INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0x7ff4000000000000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(-f64::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0xfff4000000000000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }
    #[test]
    fn test_i32_trunc_sat_f64_u() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i32.trunc_sat_f64_u", vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(&inst, "i32.trunc_sat_f64_u", vec![Val::Num(Num::F64(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0x0000000000000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0x8000000000000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(&inst, "i32.trunc_sat_f64_u", vec![Val::Num(Num::F64(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0x3ff199999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(&inst, "i32.trunc_sat_f64_u", vec![Val::Num(Num::F64(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(&inst, "i32.trunc_sat_f64_u", vec![Val::Num(Num::F64(1.9))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(&inst, "i32.trunc_sat_f64_u", vec![Val::Num(Num::F64(2.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(2147483648.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -2147483648);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(4294967295.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0xbfe999999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0xbfefffffffffffff)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(&inst, "i32.trunc_sat_f64_u", vec![Val::Num(Num::F64(1e8))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 100000000);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(4294967296.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffffffffu32 as i32
        );

        let ret = call_function(&inst, "i32.trunc_sat_f64_u", vec![Val::Num(Num::F64(-1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x00000000);

        let ret = call_function(&inst, "i32.trunc_sat_f64_u", vec![Val::Num(Num::F64(1e16))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffffffffu32 as i32
        );

        let ret = call_function(&inst, "i32.trunc_sat_f64_u", vec![Val::Num(Num::F64(1e30))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffffffffu32 as i32
        );

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(9223372036854775808.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffffffffu32 as i32
        );

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffffffffu32 as i32
        );

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::NEG_INFINITY))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x00000000);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0x7ff4000000000000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(-f64::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i32.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0xfff4000000000000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }
    #[test]
    fn test_i64_trunc_sat_f32_s() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i64.trunc_sat_f32_s", vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(&inst, "i64.trunc_sat_f32_s", vec![Val::Num(Num::F32(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0x00000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0x80000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(&inst, "i64.trunc_sat_f32_s", vec![Val::Num(Num::F32(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0x3f8ccccd)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(&inst, "i64.trunc_sat_f32_s", vec![Val::Num(Num::F32(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(&inst, "i64.trunc_sat_f32_s", vec![Val::Num(Num::F32(-1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0xbf8ccccd)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);

        let ret = call_function(&inst, "i64.trunc_sat_f32_s", vec![Val::Num(Num::F32(-1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);

        let ret = call_function(&inst, "i64.trunc_sat_f32_s", vec![Val::Num(Num::F32(-1.9))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);

        let ret = call_function(&inst, "i64.trunc_sat_f32_s", vec![Val::Num(Num::F32(-2.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -2);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(4294967296.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 4294967296);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(-4294967296.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -4294967296);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(9223371487098961920.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            9223371487098961920
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(-9223372036854775808.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            -9223372036854775808
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(9223372036854775808.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x7fffffffffffffff
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(-9223373136366403584.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x8000000000000000u64 as i64
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x7fffffffffffffff
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::NEG_INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x8000000000000000u64 as i64
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0x7fa00000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(-f32::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_s",
            vec![Val::Num(Num::F32(f32::from_bits(0xffa00000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
    }
    #[test]
    fn test_i64_trunc_sat_f32_u() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i64.trunc_sat_f32_u", vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(&inst, "i64.trunc_sat_f32_u", vec![Val::Num(Num::F32(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0x00000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0x80000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(&inst, "i64.trunc_sat_f32_u", vec![Val::Num(Num::F32(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0x3f8ccccd)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(&inst, "i64.trunc_sat_f32_u", vec![Val::Num(Num::F32(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(4294967296.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 4294967296);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(18446742974197923840.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            -1099511627776
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0xbf666666)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0xbfffffff)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(18446744073709551616.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0xffffffffffffffffu64 as i64
        );

        let ret = call_function(&inst, "i64.trunc_sat_f32_u", vec![Val::Num(Num::F32(-1.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x0000000000000000
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0xffffffffffffffffu64 as i64
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::NEG_INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x0000000000000000
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0x7fa00000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(-f32::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f32_u",
            vec![Val::Num(Num::F32(f32::from_bits(0xffa00000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
    }
    #[test]
    fn test_i64_trunc_sat_f64_s() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i64.trunc_sat_f64_s", vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(&inst, "i64.trunc_sat_f64_s", vec![Val::Num(Num::F64(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0x0000000000000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0x8000000000000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(&inst, "i64.trunc_sat_f64_s", vec![Val::Num(Num::F64(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0x3ff199999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(&inst, "i64.trunc_sat_f64_s", vec![Val::Num(Num::F64(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(&inst, "i64.trunc_sat_f64_s", vec![Val::Num(Num::F64(-1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0xbff199999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);

        let ret = call_function(&inst, "i64.trunc_sat_f64_s", vec![Val::Num(Num::F64(-1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);

        let ret = call_function(&inst, "i64.trunc_sat_f64_s", vec![Val::Num(Num::F64(-1.9))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);

        let ret = call_function(&inst, "i64.trunc_sat_f64_s", vec![Val::Num(Num::F64(-2.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -2);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(4294967296.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 4294967296);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(-4294967296.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -4294967296);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(9223372036854774784.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            9223372036854774784
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(-9223372036854775808.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            -9223372036854775808
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(9223372036854775808.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x7fffffffffffffff
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(-9223372036854777856.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x8000000000000000u64 as i64
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x7fffffffffffffff
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::NEG_INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x8000000000000000u64 as i64
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0x7ff4000000000000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(-f64::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_s",
            vec![Val::Num(Num::F64(f64::from_bits(0xfff4000000000000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
    }
    #[test]
    fn test_i64_trunc_sat_f64_u() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i64.trunc_sat_f64_u", vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(&inst, "i64.trunc_sat_f64_u", vec![Val::Num(Num::F64(-0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0x0000000000000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0x8000000000000001)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(&inst, "i64.trunc_sat_f64_u", vec![Val::Num(Num::F64(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0x3ff199999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(&inst, "i64.trunc_sat_f64_u", vec![Val::Num(Num::F64(1.5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(4294967295.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0xffffffff);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(4294967296.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0x100000000);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(18446744073709549568.0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -2048);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0xbfe999999999999a)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0xbfefffffffffffff)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(&inst, "i64.trunc_sat_f64_u", vec![Val::Num(Num::F64(1e8))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 100000000);

        let ret = call_function(&inst, "i64.trunc_sat_f64_u", vec![Val::Num(Num::F64(1e16))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            10000000000000000
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(9223372036854775808.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            -9223372036854775808
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(18446744073709551616.0))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0xffffffffffffffffu64 as i64
        );

        let ret = call_function(&inst, "i64.trunc_sat_f64_u", vec![Val::Num(Num::F64(-1.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x0000000000000000
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0xffffffffffffffffu64 as i64
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::NEG_INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x0000000000000000
        );

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0x7ff4000000000000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(-f64::NAN))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(
            &inst,
            "i64.trunc_sat_f64_u",
            vec![Val::Num(Num::F64(f64::from_bits(0xfff4000000000000)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
    }
    #[test]
    fn test_f32_convert_i64_s() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "f32.convert_i64_s", vec![Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 1.0);

        let ret = call_function(&inst, "f32.convert_i64_s", vec![Val::Num(Num::I64(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), -1.0);

        let ret = call_function(&inst, "f32.convert_i64_s", vec![Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(), 0);

        let ret = call_function(
            &inst,
            "f32.convert_i64_s",
            vec![Val::Num(Num::I64(9223372036854775807))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            9223372036854775807.0
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_s",
            vec![Val::Num(Num::I64(-9223372036854775808))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap(),
            -9223372036854775808.0
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_s",
            vec![Val::Num(Num::I64(314159265358979))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x578edcf4
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_s",
            vec![Val::Num(Num::I64(16777217))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 16777216.0);

        let ret = call_function(
            &inst,
            "f32.convert_i64_s",
            vec![Val::Num(Num::I64(-16777217))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), -16777216.0);

        let ret = call_function(
            &inst,
            "f32.convert_i64_s",
            vec![Val::Num(Num::I64(16777219))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), 16777220.0);

        let ret = call_function(
            &inst,
            "f32.convert_i64_s",
            vec![Val::Num(Num::I64(-16777219))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap(), -16777220.0);

        let ret = call_function(
            &inst,
            "f32.convert_i64_s",
            vec![Val::Num(Num::I64(0x7fffff4000000001))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x5effffff
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_s",
            vec![Val::Num(Num::I64(0x8000004000000001u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0xdeffffff
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_s",
            vec![Val::Num(Num::I64(0x0020000020000001))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x5a000001
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_s",
            vec![Val::Num(Num::I64(0xffdfffffdfffffffu64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0xda000001
        );
    }
    #[test]
    fn test_f64_convert_i32_s() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "f64.convert_i32_s", vec![Val::Num(Num::I32(1))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            1.0f64.to_bits()
        );

        let ret = call_function(&inst, "f64.convert_i32_s", vec![Val::Num(Num::I32(-1))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            (-1.0f64).to_bits()
        );

        let ret = call_function(&inst, "f64.convert_i32_s", vec![Val::Num(Num::I32(0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i32_s",
            vec![Val::Num(Num::I32(2147483647))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            2147483647.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i32_s",
            vec![Val::Num(Num::I32(-2147483648))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            (-2147483648.0f64).to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i32_s",
            vec![Val::Num(Num::I32(987654321))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            987654321.0f64.to_bits()
        );
    }

    #[test]
    fn test_f64_convert_i64_s() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "f64.convert_i64_s", vec![Val::Num(Num::I64(1))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            1.0f64.to_bits()
        );

        let ret = call_function(&inst, "f64.convert_i64_s", vec![Val::Num(Num::I64(-1))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            (-1.0f64).to_bits()
        );

        let ret = call_function(&inst, "f64.convert_i64_s", vec![Val::Num(Num::I64(0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_s",
            vec![Val::Num(Num::I64(9223372036854775807))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            9223372036854775807.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_s",
            vec![Val::Num(Num::I64(-9223372036854775808))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            (-9223372036854775808.0f64).to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_s",
            vec![Val::Num(Num::I64(4669201609102990))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            4669201609102990.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_s",
            vec![Val::Num(Num::I64(9007199254740993))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            9007199254740992.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_s",
            vec![Val::Num(Num::I64(-9007199254740993))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            (-9007199254740992.0f64).to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_s",
            vec![Val::Num(Num::I64(9007199254740995))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            9007199254740996.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_s",
            vec![Val::Num(Num::I64(-9007199254740995))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            (-9007199254740996.0f64).to_bits()
        );
    }

    #[test]
    fn test_f32_convert_i32_u() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "f32.convert_i32_u", vec![Val::Num(Num::I32(1))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            1.0f32.to_bits()
        );

        let ret = call_function(&inst, "f32.convert_i32_u", vec![Val::Num(Num::I32(0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.convert_i32_u",
            vec![Val::Num(Num::I32(2147483647))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            2147483648.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.convert_i32_u",
            vec![Val::Num(Num::I32(-2147483648))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            2147483648.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.convert_i32_u",
            vec![Val::Num(Num::I32(0x12345678))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x4d91a2b4
        );

        let ret = call_function(
            &inst,
            "f32.convert_i32_u",
            vec![Val::Num(Num::I32(0xffffffff_u32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            4294967296.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.convert_i32_u",
            vec![Val::Num(Num::I32(0x80000080_u32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x4f000000
        );

        let ret = call_function(
            &inst,
            "f32.convert_i32_u",
            vec![Val::Num(Num::I32(0x80000081_u32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x4f000001
        );

        let ret = call_function(
            &inst,
            "f32.convert_i32_u",
            vec![Val::Num(Num::I32(0x80000082_u32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x4f000001
        );

        let ret = call_function(
            &inst,
            "f32.convert_i32_u",
            vec![Val::Num(Num::I32(0xfffffe80_u32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x4f7ffffe
        );

        let ret = call_function(
            &inst,
            "f32.convert_i32_u",
            vec![Val::Num(Num::I32(0xfffffe81_u32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x4f7fffff
        );

        let ret = call_function(
            &inst,
            "f32.convert_i32_u",
            vec![Val::Num(Num::I32(0xfffffe82_u32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x4f7fffff
        );

        let ret = call_function(
            &inst,
            "f32.convert_i32_u",
            vec![Val::Num(Num::I32(16777217))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            16777216.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.convert_i32_u",
            vec![Val::Num(Num::I32(16777219))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            16777220.0f32.to_bits()
        );
    }

    #[test]
    fn test_f32_convert_i64_u() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "f32.convert_i64_u", vec![Val::Num(Num::I64(1))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            1.0f32.to_bits()
        );

        let ret = call_function(&inst, "f32.convert_i64_u", vec![Val::Num(Num::I64(0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_u",
            vec![Val::Num(Num::I64(9223372036854775807))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            9223372036854775807.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_u",
            vec![Val::Num(Num::I64(-9223372036854775808))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            9223372036854775808.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_u",
            vec![Val::Num(Num::I64(0xffffffffffffffff_u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            18446744073709551616.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_u",
            vec![Val::Num(Num::I64(16777217))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            16777216.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_u",
            vec![Val::Num(Num::I64(16777219))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            16777220.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_u",
            vec![Val::Num(Num::I64(0x0020000020000001_u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x5a000001
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_u",
            vec![Val::Num(Num::I64(0x7fffffbfffffffff_u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x5effffff
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_u",
            vec![Val::Num(Num::I64(0x8000008000000001_u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x5f000001
        );

        let ret = call_function(
            &inst,
            "f32.convert_i64_u",
            vec![Val::Num(Num::I64(0xfffffe8000000001_u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x5f7fffff
        );
    }

    #[test]
    fn test_f64_convert_i32_u() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "f64.convert_i32_u", vec![Val::Num(Num::I32(1))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            1.0f64.to_bits()
        );

        let ret = call_function(&inst, "f64.convert_i32_u", vec![Val::Num(Num::I32(0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i32_u",
            vec![Val::Num(Num::I32(2147483647))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            2147483647.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i32_u",
            vec![Val::Num(Num::I32(-2147483648))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            2147483648.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i32_u",
            vec![Val::Num(Num::I32(0xffffffff_u32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            4294967295.0f64.to_bits()
        );
    }

    #[test]
    fn test_f64_convert_i64_u() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "f64.convert_i64_u", vec![Val::Num(Num::I64(1))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            1.0f64.to_bits()
        );

        let ret = call_function(&inst, "f64.convert_i64_u", vec![Val::Num(Num::I64(0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_u",
            vec![Val::Num(Num::I64(9223372036854775807))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            9223372036854775807.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_u",
            vec![Val::Num(Num::I64(-9223372036854775808))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            9223372036854775808.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_u",
            vec![Val::Num(Num::I64(0xffffffffffffffff_u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            18446744073709551616.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_u",
            vec![Val::Num(Num::I64(0x8000000000000400_u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0x43e0000000000000
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_u",
            vec![Val::Num(Num::I64(0x8000000000000401_u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0x43e0000000000001
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_u",
            vec![Val::Num(Num::I64(0x8000000000000402_u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0x43e0000000000001
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_u",
            vec![Val::Num(Num::I64(0xfffffffffffff400_u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0x43effffffffffffe
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_u",
            vec![Val::Num(Num::I64(0xfffffffffffff401_u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0x43efffffffffffff
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_u",
            vec![Val::Num(Num::I64(0xfffffffffffff402_u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0x43efffffffffffff
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_u",
            vec![Val::Num(Num::I64(9007199254740993))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            9007199254740992.0f64.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.convert_i64_u",
            vec![Val::Num(Num::I64(9007199254740995))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            9007199254740996.0f64.to_bits()
        );
    }

    #[test]
    fn test_f64_promote_f32() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "f64.promote_f32", vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0.0f64.to_bits()
        );
        let ret = call_function(&inst, "f64.promote_f32", vec![Val::Num(Num::F32(-0.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            (-0.0f64).to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.promote_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0x1)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0x36a0000000000000
        );

        let ret = call_function(
            &inst,
            "f64.promote_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0x80000001)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0xb6a0000000000000
        );

        let ret = call_function(&inst, "f64.promote_f32", vec![Val::Num(Num::F32(1.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            1.0f64.to_bits()
        );

        let ret = call_function(&inst, "f64.promote_f32", vec![Val::Num(Num::F32(-1.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            (-1.0f64).to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.promote_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0xff7fffff)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            (f32::from_bits(0xff7fffff) as f64).to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.promote_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0x7f7fffff)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            (f32::from_bits(0x7f7fffff) as f64).to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.promote_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0x4000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            (f32::from_bits(0x4000000) as f64).to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.promote_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0x7c47c33f)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            (f32::from_bits(0x7c47c33f) as f64).to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.promote_f32",
            vec![Val::Num(Num::F32(f32::INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            f64::INFINITY.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.promote_f32",
            vec![Val::Num(Num::F32(f32::NEG_INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            f64::NEG_INFINITY.to_bits()
        );

        let ret = call_function(&inst, "f64.promote_f32", vec![Val::Num(Num::F32(f32::NAN))]);
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());

        let ret = call_function(
            &inst,
            "f64.promote_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0x7fa00000)))],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());

        let ret = call_function(
            &inst,
            "f64.promote_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0xffc00000)))],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());

        let ret = call_function(
            &inst,
            "f64.promote_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0xffa00000)))],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());
    }

    #[test]
    fn test_f32_demote_f64() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "f32.demote_f64", vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0.0f32.to_bits()
        );

        let ret = call_function(&inst, "f32.demote_f64", vec![Val::Num(Num::F64(-0.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            (-0.0f32).to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x1)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x8000000000000001)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            (-0.0f32).to_bits()
        );

        let ret = call_function(&inst, "f32.demote_f64", vec![Val::Num(Num::F64(1.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            1.0f32.to_bits()
        );

        let ret = call_function(&inst, "f32.demote_f64", vec![Val::Num(Num::F64(-1.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            (-1.0f32).to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x380fffffe0000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x00800000
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xb80fffffe0000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x80800000
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x380fffffdfffffff)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x007fffff
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xb80fffffdfffffff)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x807fffff
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x36a0000000000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x00000001
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xb6a0000000000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x80000001
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x47efffffd0000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x7f7ffffe
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xc7efffffd0000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0xff7ffffe
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x47efffffd0000001)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x7f7fffff
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xc7efffffd0000001)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0xff7fffff
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x47efffffe0000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x7f7fffff
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xc7efffffe0000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0xff7fffff
        );

        let input_val = f64::from_bits(0x47efffffefffff);
        let ret = call_function(&inst, "f32.demote_f64", vec![Val::Num(Num::F64(input_val))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xc7efffffefffff)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x47effffff0000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            f32::INFINITY.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xc7effffff0000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            f32::NEG_INFINITY.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x3880000000000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x4000000
        );

        let input_val = f64::from_bits(0x47c8f867e0000000);
        let ret = call_function(&inst, "f32.demote_f64", vec![Val::Num(Num::F64(input_val))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x7e47c33f
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            f32::INFINITY.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::NEG_INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            f32::NEG_INFINITY.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x3ff0000000000001)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            1.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x3fefffffffffffff)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            1.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x3ff0000010000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x3f800000
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x3ff0000010000001)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x3f800001
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x3ff000002fffffff)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x3f800001
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x3ff0000030000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x3f800002
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x3ff0000050000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x3f800002
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x4170000010000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x4b800000
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x4170000010000001)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x4b800001
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x417000002fffffff)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x4b800001
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x4170000030000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x4b800002
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x46b4eae4f7024c70)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x75a75728
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x38ea12e71e358685)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x7509739
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x380cb98354d521ff)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x72e60d
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xc006972b30cfb562)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0xc034b95a
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xc6fbedbe4819d4c4)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0xf7df6df2
        );

        let ret = call_function(&inst, "f32.demote_f64", vec![Val::Num(Num::F64(f64::NAN))]);
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x7ff4000000000000)))],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xfff8000000000000)))],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xfff4000000000000)))],
        );
        assert!(ret.unwrap().last().unwrap().to_f32().unwrap().is_nan());

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x0010000000000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x8010000000000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            (-0.0f32).to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x3690000000000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0.0f32.to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xb690000000000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            (-0.0f32).to_bits()
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x3690000000000001)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x00000001
        );

        let ret = call_function(
            &inst,
            "f32.demote_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xb690000000000001)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x80000001
        );
    }

    #[test]
    fn test_f32_reinterpret_i32() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "f32.reinterpret_i32", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(), 0);

        let ret = call_function(
            &inst,
            "f32.reinterpret_i32",
            vec![Val::Num(Num::I32(0x80000000_u32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x80000000
        );

        let ret = call_function(&inst, "f32.reinterpret_i32", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(), 1);

        let ret = call_function(&inst, "f32.reinterpret_i32", vec![Val::Num(Num::I32(-1))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0xffffffff
        );

        let ret = call_function(
            &inst,
            "f32.reinterpret_i32",
            vec![Val::Num(Num::I32(123456789))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            123456789
        );

        let ret = call_function(
            &inst,
            "f32.reinterpret_i32",
            vec![Val::Num(Num::I32(-2147483647))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            (-2147483647i32) as u32
        );

        let ret = call_function(
            &inst,
            "f32.reinterpret_i32",
            vec![Val::Num(Num::I32(0x7f800000))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x7f800000
        );

        let ret = call_function(
            &inst,
            "f32.reinterpret_i32",
            vec![Val::Num(Num::I32(0xff800000_u32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0xff800000
        );

        let ret = call_function(
            &inst,
            "f32.reinterpret_i32",
            vec![Val::Num(Num::I32(0x7fc00000))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x7fc00000
        );

        let ret = call_function(
            &inst,
            "f32.reinterpret_i32",
            vec![Val::Num(Num::I32(0xffc00000_u32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0xffc00000
        );

        let ret = call_function(
            &inst,
            "f32.reinterpret_i32",
            vec![Val::Num(Num::I32(0x7fa00000))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0x7fa00000
        );

        let ret = call_function(
            &inst,
            "f32.reinterpret_i32",
            vec![Val::Num(Num::I32(0xffa00000_u32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f32().unwrap().to_bits(),
            0xffa00000
        );
    }

    #[test]
    fn test_f64_reinterpret_i64() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "f64.reinterpret_i64", vec![Val::Num(Num::I64(0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0.0f64.to_bits()
        );

        let ret = call_function(&inst, "f64.reinterpret_i64", vec![Val::Num(Num::I64(1))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0x1
        );

        let ret = call_function(&inst, "f64.reinterpret_i64", vec![Val::Num(Num::I64(-1))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            0xffffffffffffffff
        );

        let ret = call_function(
            &inst,
            "f64.reinterpret_i64",
            vec![Val::Num(Num::I64(0x8000000000000000_u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            (-0.0f64).to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.reinterpret_i64",
            vec![Val::Num(Num::I64(1234567890))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            f64::from_bits(1234567890u64).to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.reinterpret_i64",
            vec![Val::Num(Num::I64(-9223372036854775807))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            f64::from_bits((-9223372036854775807i64) as u64).to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.reinterpret_i64",
            vec![Val::Num(Num::I64(0x7ff0000000000000))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            f64::INFINITY.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.reinterpret_i64",
            vec![Val::Num(Num::I64(0xfff0000000000000_u64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_f64().unwrap().to_bits(),
            f64::NEG_INFINITY.to_bits()
        );

        let ret = call_function(
            &inst,
            "f64.reinterpret_i64",
            vec![Val::Num(Num::I64(0x7ff8000000000000))],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());

        let ret = call_function(
            &inst,
            "f64.reinterpret_i64",
            vec![Val::Num(Num::I64(0xfff8000000000000_u64 as i64))],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());

        let ret = call_function(
            &inst,
            "f64.reinterpret_i64",
            vec![Val::Num(Num::I64(0x7ff4000000000000))],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());

        let ret = call_function(
            &inst,
            "f64.reinterpret_i64",
            vec![Val::Num(Num::I64(0xfff4000000000000_u64 as i64))],
        );
        assert!(ret.unwrap().last().unwrap().to_f64().unwrap().is_nan());
    }

    #[test]
    fn test_i32_reinterpret_f32() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i32.reinterpret_f32", vec![Val::Num(Num::F32(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        let ret = call_function(&inst, "i32.reinterpret_f32", vec![Val::Num(Num::F32(-0.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x80000000u32 as i32
        );

        let ret = call_function(
            &inst,
            "i32.reinterpret_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0x1)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        let ret = call_function(
            &inst,
            "i32.reinterpret_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0xffffffff)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);

        let ret = call_function(
            &inst,
            "i32.reinterpret_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0x80000001)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x80000001u32 as i32
        );

        let ret = call_function(&inst, "i32.reinterpret_f32", vec![Val::Num(Num::F32(1.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1065353216);

        let ret = call_function(
            &inst,
            "i32.reinterpret_f32",
            vec![Val::Num(Num::F32(3.1415926))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1078530010);

        let ret = call_function(
            &inst,
            "i32.reinterpret_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0x7f7fffff)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2139095039);

        let ret = call_function(
            &inst,
            "i32.reinterpret_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0xff7fffff)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -8388609);

        let ret = call_function(
            &inst,
            "i32.reinterpret_f32",
            vec![Val::Num(Num::F32(f32::INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x7f800000u32 as i32
        );

        let ret = call_function(
            &inst,
            "i32.reinterpret_f32",
            vec![Val::Num(Num::F32(f32::NEG_INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xff800000u32 as i32
        );

        let ret = call_function(
            &inst,
            "i32.reinterpret_f32",
            vec![Val::Num(Num::F32(f32::NAN))],
        );
        let result = ret.unwrap().last().unwrap().to_i32().unwrap() as u32;
        assert!((result & 0x7f800000) == 0x7f800000 && (result & 0x7fffff) != 0);

        let ret = call_function(
            &inst,
            "i32.reinterpret_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0xffc00000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffc00000u32 as i32
        );

        let ret = call_function(
            &inst,
            "i32.reinterpret_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0x7fa00000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0x7fa00000u32 as i32
        );

        let ret = call_function(
            &inst,
            "i32.reinterpret_f32",
            vec![Val::Num(Num::F32(f32::from_bits(0xffa00000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffa00000u32 as i32
        );
    }

    #[test]
    fn test_i64_reinterpret_f64() {
        let inst = load_instance("tests/wasm/conversions.wasm");

        let ret = call_function(&inst, "i64.reinterpret_f64", vec![Val::Num(Num::F64(0.0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);

        let ret = call_function(&inst, "i64.reinterpret_f64", vec![Val::Num(Num::F64(-0.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            (-0.0f64).to_bits() as i64
        );

        let ret = call_function(
            &inst,
            "i64.reinterpret_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x1)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);

        let ret = call_function(
            &inst,
            "i64.reinterpret_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xffffffffffffffff)))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);

        let ret = call_function(
            &inst,
            "i64.reinterpret_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x8000000000000001)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x8000000000000001_u64 as i64
        );

        let ret = call_function(&inst, "i64.reinterpret_f64", vec![Val::Num(Num::F64(1.0))]);
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            1.0f64.to_bits() as i64
        );

        let ret = call_function(
            &inst,
            "i64.reinterpret_f64",
            vec![Val::Num(Num::F64(3.14159265358979))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            3.14159265358979f64.to_bits() as i64
        );

        let ret = call_function(
            &inst,
            "i64.reinterpret_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x7fefffffffffffff)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x7fefffffffffffff_u64 as i64
        );

        let ret = call_function(
            &inst,
            "i64.reinterpret_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xffefffffffffffff)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0xffefffffffffffff_u64 as i64
        );

        let ret = call_function(
            &inst,
            "i64.reinterpret_f64",
            vec![Val::Num(Num::F64(f64::INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            f64::INFINITY.to_bits() as i64
        );

        let ret = call_function(
            &inst,
            "i64.reinterpret_f64",
            vec![Val::Num(Num::F64(f64::NEG_INFINITY))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            f64::NEG_INFINITY.to_bits() as i64
        );

        let ret = call_function(
            &inst,
            "i64.reinterpret_f64",
            vec![Val::Num(Num::F64(f64::NAN))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            f64::NAN.to_bits() as i64
        );

        let ret = call_function(
            &inst,
            "i64.reinterpret_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xfff8000000000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0xfff8000000000000_u64 as i64
        );

        let ret = call_function(
            &inst,
            "i64.reinterpret_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0x7ff4000000000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x7ff4000000000000_u64 as i64
        );

        let ret = call_function(
            &inst,
            "i64.reinterpret_f64",
            vec![Val::Num(Num::F64(f64::from_bits(0xfff4000000000000)))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0xfff4000000000000_u64 as i64
        );
    }
}
