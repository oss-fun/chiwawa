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
        let _ = parser::parse_bytecode(&mut module, wasm_path, true, "slot");
        let imports: ImportObjects = FxHashMap::default();
        ModuleInst::new(&module, imports, Vec::new()).unwrap()
    }

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
    fn test_externref() {
        let inst = load_instance("tests/wasm/ref_null.wasm");
        let ret = call_function(&inst, "externref", vec![]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));
    }

    #[test]
    fn test_funcref() {
        let inst = load_instance("tests/wasm/ref_null.wasm");
        let ret = call_function(&inst, "funcref", vec![]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));
    }
}
