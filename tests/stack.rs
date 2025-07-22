use chiwawa::{
    execution::module::*, execution::runtime::Runtime, execution::value::*, parser,
    structure::module::Module,
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
    fn test_fac_expr_25() {
        let inst = load_instance("tests/wasm/stack.wasm");
        let params = vec![Val::Num(Num::I64(25))];
        let ret = call_function(&inst, "fac-expr", params);
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            7034535277573963776
        );
    }

    #[test]
    fn test_fac_stack_25() {
        let inst = load_instance("tests/wasm/stack.wasm");
        let params = vec![Val::Num(Num::I64(25))];
        let ret = call_function(&inst, "fac-stack", params);
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            7034535277573963776
        );
    }

    #[test]
    fn test_fac_mixed_25() {
        let inst = load_instance("tests/wasm/stack.wasm");
        let params = vec![Val::Num(Num::I64(25))];
        let ret = call_function(&inst, "fac-mixed", params);
        assert_eq!(
            ret.unwrap().last().unwrap().to_i64().unwrap(),
            7034535277573963776
        );
    }

    #[test]
    fn test_not_quite_a_tree_first() {
        let inst = load_instance("tests/wasm/stack.wasm");
        let ret = call_function(&inst, "not-quite-a-tree", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_not_quite_a_tree_second() {
        let inst = load_instance("tests/wasm/stack.wasm");
        let ret1 = call_function(&inst, "not-quite-a-tree", vec![]);
        assert_eq!(ret1.unwrap().last().unwrap().to_i32().unwrap(), 3);

        let ret2 = call_function(&inst, "not-quite-a-tree", vec![]);
        assert_eq!(ret2.unwrap().last().unwrap().to_i32().unwrap(), 9);
    }
}
