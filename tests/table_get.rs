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
        let mut runtime = Runtime::new(Rc::clone(inst), &func_addr, params, true, false, None)?;
        runtime.run()
    }

    #[test]
    fn test_table_get() {
        let inst = load_instance("tests/wasm/table_get.wasm");

        let ret = call_function(&inst, "get-externref", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        let ret = call_function(&inst, "get-funcref", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        let ret = call_function(&inst, "is_null-funcref", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Num(Num::I32(0)));

        let extern_val = Val::Ref(Ref::RefNull);
        let ret = call_function(&inst, "init", vec![extern_val]);
        assert!(ret.is_ok());

        let ret = call_function(&inst, "get-externref", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        let ret = call_function(&inst, "is_null-funcref", vec![Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Num(Num::I32(0)));

        let ret1 = call_function(&inst, "get-funcref", vec![Val::Num(Num::I32(1))]);
        let ret2 = call_function(&inst, "get-funcref", vec![Val::Num(Num::I32(2))]);

        assert!(matches!(
            ret1.unwrap().last().unwrap(),
            Val::Ref(Ref::FuncAddr(_))
        ));
        assert!(matches!(
            ret2.unwrap().last().unwrap(),
            Val::Ref(Ref::FuncAddr(_))
        ));
    }
}
