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
        let mut runtime = Runtime::new(Rc::clone(inst), &func_addr, params, true, false)?;
        runtime.run()
    }

    #[test]
    fn test_get_externref_initial() {
        // (assert_return (invoke "get-externref" (i32.const 0)) (ref.null extern))
        let inst = load_instance("tests/wasm/table_set.wasm");
        let ret = call_function(&inst, "get-externref", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));
    }

    #[test]
    fn test_set_externref() {
        // (assert_return (invoke "set-externref" (i32.const 0) (ref.extern 1)))
        let inst = load_instance("tests/wasm/table_set.wasm");

        // Create an external reference value - for now we'll use RefNull as a placeholder
        // since actual external references are complex to create in tests
        let extern_val = Val::Ref(Ref::RefNull);

        let ret = call_function(
            &inst,
            "set-externref",
            vec![Val::Num(Num::I32(0)), extern_val],
        );
        assert!(ret.is_ok());
    }

    #[test]
    fn test_set_and_get_externref_null() {
        // (assert_return (invoke "set-externref" (i32.const 0) (ref.null extern)))
        // (assert_return (invoke "get-externref" (i32.const 0)) (ref.null extern))
        let inst = load_instance("tests/wasm/table_set.wasm");

        // Set to null
        let ret = call_function(
            &inst,
            "set-externref",
            vec![Val::Num(Num::I32(0)), Val::Ref(Ref::RefNull)],
        );
        assert!(ret.is_ok());

        // Get and verify it's null
        let ret = call_function(&inst, "get-externref", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));
    }

    #[test]
    fn test_get_funcref_initial() {
        // (assert_return (invoke "get-funcref" (i32.const 0)) (ref.null func))
        let inst = load_instance("tests/wasm/table_set.wasm");
        let ret = call_function(&inst, "get-funcref", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));
    }

    #[test]
    fn test_set_funcref_from() {
        // (assert_return (invoke "set-funcref-from" (i32.const 0) (i32.const 1)))
        let inst = load_instance("tests/wasm/table_set.wasm");
        let ret = call_function(
            &inst,
            "set-funcref-from",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert!(ret.is_ok());
    }

    #[test]
    fn test_is_null_funcref_after_set() {
        // (assert_return (invoke "is_null-funcref" (i32.const 0)) (i32.const 0))
        let inst = load_instance("tests/wasm/table_set.wasm");

        // First set a function reference from index 1 to index 0
        let ret = call_function(
            &inst,
            "set-funcref-from",
            vec![Val::Num(Num::I32(0)), Val::Num(Num::I32(1))],
        );
        assert!(ret.is_ok());

        // Check if it's null (should be 0 since we copied a function)
        let ret = call_function(&inst, "is_null-funcref", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Num(Num::I32(0)));
    }

    #[test]
    fn test_set_funcref_to_null() {
        // (assert_return (invoke "set-funcref" (i32.const 0) (ref.null func)))
        // (assert_return (invoke "get-funcref" (i32.const 0)) (ref.null func))
        let inst = load_instance("tests/wasm/table_set.wasm");

        // Set funcref to null
        let ret = call_function(
            &inst,
            "set-funcref",
            vec![Val::Num(Num::I32(0)), Val::Ref(Ref::RefNull)],
        );
        assert!(ret.is_ok());

        // Get and verify it's null
        let ret = call_function(&inst, "get-funcref", vec![Val::Num(Num::I32(0))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));
    }
}
