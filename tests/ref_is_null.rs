use chiwawa::{
    execution::module::*,
    execution::runtime::Runtime,
    execution::value::*,
    parser,
    structure::module::{Module, WasiFuncType},
};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    fn load_instance(wasm_path: &str) -> Arc<ModuleInst> {
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path);
        let imports: ImportObjects = HashMap::new();
        ModuleInst::new(&module, imports, Vec::new(), Vec::new()).unwrap()
    }

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
    fn test_funcref_null() {
        // (assert_return (invoke "funcref" (ref.null func)) (i32.const 1))
        let inst = load_instance("tests/wasm/ref_is_null.wasm");
        let ret = call_function(&inst, "funcref", vec![Val::Ref(Ref::RefNull)]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Num(Num::I32(1)));
    }

    #[test]
    fn test_externref_null() {
        // (assert_return (invoke "externref" (ref.null extern)) (i32.const 1))
        let inst = load_instance("tests/wasm/ref_is_null.wasm");
        let ret = call_function(&inst, "externref", vec![Val::Ref(Ref::RefNull)]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Num(Num::I32(1)));
    }

    #[test]
    fn test_funcref_elem_null() {
        // (assert_return (invoke "funcref-elem" (i32.const 0)) (i32.const 1))
        let inst = load_instance("tests/wasm/ref_is_null.wasm");
        let ret = call_function(&inst, "funcref-elem", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Num(Num::I32(1)));
    }

    #[test]
    fn test_externref_elem_null() {
        // (assert_return (invoke "externref-elem" (i32.const 0)) (i32.const 1))
        let inst = load_instance("tests/wasm/ref_is_null.wasm");
        let ret = call_function(&inst, "externref-elem", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Num(Num::I32(1)));
    }

    #[test]
    fn test_funcref_elem_non_null() {
        // (assert_return (invoke "funcref-elem" (i32.const 1)) (i32.const 0))
        let inst = load_instance("tests/wasm/ref_is_null.wasm");
        let ret = call_function(&inst, "funcref-elem", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Num(Num::I32(0)));
    }
}
