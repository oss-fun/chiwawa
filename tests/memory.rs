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
    fn test_data() {
        let inst = load_instance("tests/wasm/memory.wasm");
        let ret = call_function(&inst, "data", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_cast() {
        let inst = load_instance("tests/wasm/memory.wasm");
        let ret = call_function(&inst, "cast", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_f64().unwrap(), 42.0);
    }

    // i32_load8_s tests
    #[test]
    fn test_i32_load8_s() {
        let inst = load_instance("tests/wasm/memory.wasm");
        let ret = call_function(&inst, "i32_load8_s", vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);

        let ret = call_function(&inst, "i32_load8_s", vec![Val::Num(Num::I32(100))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 100);

        let ret = call_function(
            &inst,
            "i32_load8_s",
            vec![Val::Num(Num::I32(0xfedc6543u32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x43);

        let ret = call_function(
            &inst,
            "i32_load8_s",
            vec![Val::Num(Num::I32(0x3456cdefu32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffffffefu32 as i32
        );
    }

    #[test]
    fn test_i32_load8_u() {
        let inst = load_instance("tests/wasm/memory.wasm");
        let ret = call_function(&inst, "i32_load8_u", vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 255);

        let ret = call_function(&inst, "i32_load8_u", vec![Val::Num(Num::I32(200))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 200);

        let ret = call_function(
            &inst,
            "i32_load8_u",
            vec![Val::Num(Num::I32(0xfedc6543u32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x43);

        let ret = call_function(
            &inst,
            "i32_load8_u",
            vec![Val::Num(Num::I32(0x3456cdefu32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0xef);
    }

    #[test]
    fn test_i32_load16_s() {
        let inst = load_instance("tests/wasm/memory.wasm");
        let ret = call_function(&inst, "i32_load16_s", vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), -1);

        let ret = call_function(&inst, "i32_load16_s", vec![Val::Num(Num::I32(20000))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 20000);

        let ret = call_function(
            &inst,
            "i32_load16_s",
            vec![Val::Num(Num::I32(0xfedc6543u32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x6543);

        let ret = call_function(
            &inst,
            "i32_load16_s",
            vec![Val::Num(Num::I32(0x3456cdefu32 as i32))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i32().unwrap(),
            0xffffcdefu32 as i32
        );
    }

    #[test]
    fn test_i32_load16_u() {
        let inst = load_instance("tests/wasm/memory.wasm");
        let ret = call_function(&inst, "i32_load16_u", vec![Val::Num(Num::I32(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 65535);

        let ret = call_function(&inst, "i32_load16_u", vec![Val::Num(Num::I32(40000))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 40000);

        let ret = call_function(
            &inst,
            "i32_load16_u",
            vec![Val::Num(Num::I32(0xfedc6543u32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x6543);

        let ret = call_function(
            &inst,
            "i32_load16_u",
            vec![Val::Num(Num::I32(0x3456cdefu32 as i32))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0xcdef);
    }

    // i64_load8_s tests
    #[test]
    fn test_i64_load8_s() {
        let inst = load_instance("tests/wasm/memory.wasm");
        let ret = call_function(&inst, "i64_load8_s", vec![Val::Num(Num::I64(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);

        let ret = call_function(&inst, "i64_load8_s", vec![Val::Num(Num::I64(100))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 100);

        let ret = call_function(
            &inst,
            "i64_load8_s",
            vec![Val::Num(Num::I64(0xfedcba9856346543u64 as i64))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0x43);

        let ret = call_function(
            &inst,
            "i64_load8_s",
            vec![Val::Num(Num::I64(0x3456436598bacdefu64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0xffffffffffffffefu64 as i64
        );
    }

    // i64_load8_u tests
    #[test]
    fn test_i64_load8_u() {
        let inst = load_instance("tests/wasm/memory.wasm");
        let ret = call_function(&inst, "i64_load8_u", vec![Val::Num(Num::I64(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 255);

        let ret = call_function(&inst, "i64_load8_u", vec![Val::Num(Num::I64(200))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 200);

        let ret = call_function(
            &inst,
            "i64_load8_u",
            vec![Val::Num(Num::I64(0xfedcba9856346543u64 as i64))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0x43);

        let ret = call_function(
            &inst,
            "i64_load8_u",
            vec![Val::Num(Num::I64(0x3456436598bacdefu64 as i64))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0xef);
    }

    // i64_load16_s tests
    #[test]
    fn test_i64_load16_s() {
        let inst = load_instance("tests/wasm/memory.wasm");
        let ret = call_function(&inst, "i64_load16_s", vec![Val::Num(Num::I64(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);

        let ret = call_function(&inst, "i64_load16_s", vec![Val::Num(Num::I64(20000))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 20000);

        let ret = call_function(
            &inst,
            "i64_load16_s",
            vec![Val::Num(Num::I64(0xfedcba9856346543u64 as i64))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0x6543);

        let ret = call_function(
            &inst,
            "i64_load16_s",
            vec![Val::Num(Num::I64(0x3456436598bacdefu64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0xffffffffffffcdefu64 as i64
        );
    }

    // i64_load16_u tests
    #[test]
    fn test_i64_load16_u() {
        let inst = load_instance("tests/wasm/memory.wasm");
        let ret = call_function(&inst, "i64_load16_u", vec![Val::Num(Num::I64(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 65535);

        let ret = call_function(&inst, "i64_load16_u", vec![Val::Num(Num::I64(40000))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 40000);

        let ret = call_function(
            &inst,
            "i64_load16_u",
            vec![Val::Num(Num::I64(0xfedcba9856346543u64 as i64))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0x6543);

        let ret = call_function(
            &inst,
            "i64_load16_u",
            vec![Val::Num(Num::I64(0x3456436598bacdefu64 as i64))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0xcdef);
    }

    // i64_load32_s tests
    #[test]
    fn test_i64_load32_s() {
        let inst = load_instance("tests/wasm/memory.wasm");
        let ret = call_function(&inst, "i64_load32_s", vec![Val::Num(Num::I64(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -1);

        let ret = call_function(&inst, "i64_load32_s", vec![Val::Num(Num::I64(20000))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 20000);

        let ret = call_function(
            &inst,
            "i64_load32_s",
            vec![Val::Num(Num::I64(0xfedcba9856346543u64 as i64))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0x56346543);

        let ret = call_function(
            &inst,
            "i64_load32_s",
            vec![Val::Num(Num::I64(0x3456436598bacdefu64 as i64))],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0xffffffff98bacdefu64 as i64
        );
    }

    // i64_load32_u tests
    #[test]
    fn test_i64_load32_u() {
        let inst = load_instance("tests/wasm/memory.wasm");
        let ret = call_function(&inst, "i64_load32_u", vec![Val::Num(Num::I64(-1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 4294967295);

        let ret = call_function(&inst, "i64_load32_u", vec![Val::Num(Num::I64(40000))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 40000);

        let ret = call_function(
            &inst,
            "i64_load32_u",
            vec![Val::Num(Num::I64(0xfedcba9856346543u64 as i64))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0x56346543);

        let ret = call_function(
            &inst,
            "i64_load32_u",
            vec![Val::Num(Num::I64(0x3456436598bacdefu64 as i64))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0x98bacdef);
    }
}
