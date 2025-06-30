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
        let _ = parser::parse_bytecode(&mut module, wasm_path);
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
        let mut runtime = Runtime::new(Arc::clone(inst), &func_addr, params)?;
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
}
