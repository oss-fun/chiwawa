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
        let _ = parser::parse_bytecode(&mut module, wasm_path, true);
        let imports: ImportObjects = HashMap::new();
        ModuleInst::new(&module, imports, Vec::new()).unwrap()
    }

    fn call_function(
        inst: &Arc<ModuleInst>,
        func_name: &str,
        params: Vec<Val>,
    ) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func(func_name)?;
        let mut runtime = Runtime::new(Arc::clone(inst), &func_addr, params, true)?;
        runtime.run()
    }

    #[test]
    fn test_memorysize_1() {
        let inst = load_instance("tests/wasm/memorysize-1.wasm");

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(1))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(4))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 5);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(0))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 5);
    }

    #[test]
    fn test_memorysize_2() {
        let inst = load_instance("tests/wasm/memorysize-2.wasm");

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(1))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(4))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 6);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(0))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 6);
    }

    #[test]
    fn test_memorysize_3() {
        let inst = load_instance("tests/wasm/memorysize-3.wasm");

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(3))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(1))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(0))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(4))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(1))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_memorysize_4() {
        let inst = load_instance("tests/wasm/memorysize-4.wasm");

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(1))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 4);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(3))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 7);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(0))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 7);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(2))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 7);

        call_function(&inst, "grow", vec![Val::Num(Num::I32(1))]).unwrap();

        let ret = call_function(&inst, "size", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 8);
    }
}
