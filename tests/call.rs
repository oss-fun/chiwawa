use chiwawa::{
    execution::module::*, execution::runtime::Runtime, execution::value::*, parser,
    structure::module::Module,
};
use std::collections::HashMap;
use std::rc::Rc;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to load module and get instance
    fn load_instance(wasm_path: &str) -> Rc<ModuleInst> {
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path, true);
        let imports: ImportObjects = HashMap::new();
        ModuleInst::new(&module, imports, Vec::new()).unwrap()
    }

    // Helper function to call a function using Runtime
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
    fn test_call_type_i32() {
        let inst = load_instance("tests/wasm/call.wasm");
        let ret = call_function(&inst, "type-i32", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x132);
    }

    #[test]
    fn test_call_fac() {
        let inst = load_instance("tests/wasm/call.wasm");
        let ret = call_function(&inst, "fac", vec![Val::Num(Num::I64(0))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(&inst, "fac", vec![Val::Num(Num::I64(1))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 1);
        let ret = call_function(&inst, "fac", vec![Val::Num(Num::I64(5))]);
        assert_eq!(ret.unwrap().last().unwrap().to_i64().unwrap(), 120);
    }
}
