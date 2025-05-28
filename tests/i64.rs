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
        ModuleInst::new(&module, imports).unwrap()
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
    fn test_i64_add() {
        let inst = load_instance("tests/wasm/i64.wasm");
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 2);
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::I64(1)), Val::Num(Num::I64(0))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(-1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), -2);
        let ret = call_function(
            &inst,
            "add",
            vec![Val::Num(Num::I64(-1)), Val::Num(Num::I64(1))],
        );
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 0);
        let ret = call_function(
            &inst,
            "add",
            vec![
                Val::Num(Num::I64(0x3fffffffu64 as i64)),
                Val::Num(Num::I64(1)),
            ],
        );
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            0x40000000u64 as i64
        );
    }
}
