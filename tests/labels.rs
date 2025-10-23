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
    fn test_block() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "block", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop1() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "loop1", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 5);
    }

    #[test]
    fn test_loop2() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "loop2", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 8);
    }

    #[test]
    fn test_loop3() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "loop3", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_loop4() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let params = vec![Val::Num(Num::I32(8))];
        let ret = call_function(&inst, "loop4", params);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 16);
    }

    #[test]
    fn test_loop5() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "loop5", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_loop6() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "loop6", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_if() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "if", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 5);
    }

    #[test]
    fn test_if2() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "if2", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 5);
    }

    #[test]
    fn test_switch_0() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let params = vec![Val::Num(Num::I32(0))];
        let ret = call_function(&inst, "switch", params);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 50);
    }

    #[test]
    fn test_switch_1() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let params = vec![Val::Num(Num::I32(1))];
        let ret = call_function(&inst, "switch", params);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 20);
    }

    #[test]
    fn test_switch_2() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let params = vec![Val::Num(Num::I32(2))];
        let ret = call_function(&inst, "switch", params);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 20);
    }

    #[test]
    fn test_switch_3() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let params = vec![Val::Num(Num::I32(3))];
        let ret = call_function(&inst, "switch", params);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 3);
    }

    #[test]
    fn test_switch_4() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let params = vec![Val::Num(Num::I32(4))];
        let ret = call_function(&inst, "switch", params);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 50);
    }

    #[test]
    fn test_switch_5() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let params = vec![Val::Num(Num::I32(5))];
        let ret = call_function(&inst, "switch", params);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 50);
    }

    #[test]
    fn test_return_0() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let params = vec![Val::Num(Num::I32(0))];
        let ret = call_function(&inst, "return", params);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0);
    }

    #[test]
    fn test_return_1() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let params = vec![Val::Num(Num::I32(1))];
        let ret = call_function(&inst, "return", params);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_return_2() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let params = vec![Val::Num(Num::I32(2))];
        let ret = call_function(&inst, "return", params);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_br_if0() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "br_if0", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 0x1d);
    }

    #[test]
    fn test_br_if1() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "br_if1", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_br_if2() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "br_if2", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_br_if3() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "br_if3", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 2);
    }

    #[test]
    fn test_br() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "br", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_shadowing() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "shadowing", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 1);
    }

    #[test]
    fn test_redefinition() {
        let inst = load_instance("tests/wasm/labels.wasm");
        let ret = call_function(&inst, "redefinition", vec![]);
        assert_eq!(ret.unwrap().last().unwrap().to_i32().unwrap(), 5);
    }
}
