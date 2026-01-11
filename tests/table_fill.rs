use chiwawa::{
    execution::func::FuncAddr, execution::module::*, execution::runtime::Runtime,
    execution::value::*, parser, structure::module::Module,
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
        let mut runtime = Runtime::new(Rc::clone(inst), &func_addr, params, true, false, None)?;
        runtime.run()
    }

    #[test]
    fn test_table_fill() {
        let inst = load_instance("tests/wasm/table_fill.wasm");

        // Create external reference values simulating extern ref indices 1-5
        // In WebAssembly, (ref.extern N) refers to index N in external reference table
        let extern_refs = [
            ExternAddr::new(Externval::Func(FuncAddr::alloc_empty())), // index 0 (unused)
            ExternAddr::new(Externval::Func(FuncAddr::alloc_empty())), // index 1
            ExternAddr::new(Externval::Func(FuncAddr::alloc_empty())), // index 2
            ExternAddr::new(Externval::Func(FuncAddr::alloc_empty())), // index 3
            ExternAddr::new(Externval::Func(FuncAddr::alloc_empty())), // index 4
            ExternAddr::new(Externval::Func(FuncAddr::alloc_empty())), // index 5
        ];

        let extern_1 = Val::Ref(Ref::RefExtern(extern_refs[1].clone()));
        let extern_2 = Val::Ref(Ref::RefExtern(extern_refs[2].clone()));
        let extern_3 = Val::Ref(Ref::RefExtern(extern_refs[3].clone()));
        let extern_4 = Val::Ref(Ref::RefExtern(extern_refs[4].clone()));
        let extern_5 = Val::Ref(Ref::RefExtern(extern_refs[5].clone()));

        // (assert_return (invoke "get" (i32.const 1)) (ref.null extern))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        // (assert_return (invoke "get" (i32.const 2)) (ref.null extern))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        // (assert_return (invoke "get" (i32.const 3)) (ref.null extern))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(3))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        // (assert_return (invoke "get" (i32.const 4)) (ref.null extern))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(4))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        // (assert_return (invoke "get" (i32.const 5)) (ref.null extern))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(5))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        // (assert_return (invoke "fill" (i32.const 2) (ref.extern 1) (i32.const 3)))
        let ret = call_function(
            &inst,
            "fill",
            vec![
                Val::Num(Num::I32(2)),
                extern_1.clone(),
                Val::Num(Num::I32(3)),
            ],
        );
        assert!(ret.is_ok());

        // (assert_return (invoke "get" (i32.const 1)) (ref.null extern))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(1))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        // (assert_return (invoke "get" (i32.const 2)) (ref.extern 1))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(2))]);
        assert_eq!(ret.unwrap().last().unwrap(), &extern_1);

        // (assert_return (invoke "get" (i32.const 3)) (ref.extern 1))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(3))]);
        assert_eq!(ret.unwrap().last().unwrap(), &extern_1);

        // (assert_return (invoke "get" (i32.const 4)) (ref.extern 1))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(4))]);
        assert_eq!(ret.unwrap().last().unwrap(), &extern_1);

        // (assert_return (invoke "get" (i32.const 5)) (ref.null extern))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(5))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        // (assert_return (invoke "fill" (i32.const 4) (ref.extern 2) (i32.const 2)))
        let ret = call_function(
            &inst,
            "fill",
            vec![
                Val::Num(Num::I32(4)),
                extern_2.clone(),
                Val::Num(Num::I32(2)),
            ],
        );
        assert!(ret.is_ok());

        // (assert_return (invoke "get" (i32.const 3)) (ref.extern 1))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(3))]);
        assert_eq!(ret.unwrap().last().unwrap(), &extern_1);

        // (assert_return (invoke "get" (i32.const 4)) (ref.extern 2))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(4))]);
        assert_eq!(ret.unwrap().last().unwrap(), &extern_2);

        // (assert_return (invoke "get" (i32.const 5)) (ref.extern 2))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(5))]);
        assert_eq!(ret.unwrap().last().unwrap(), &extern_2);

        // (assert_return (invoke "get" (i32.const 6)) (ref.null extern))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(6))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        // (assert_return (invoke "fill" (i32.const 4) (ref.extern 3) (i32.const 0)))
        let ret = call_function(
            &inst,
            "fill",
            vec![
                Val::Num(Num::I32(4)),
                extern_3.clone(),
                Val::Num(Num::I32(0)),
            ],
        );
        assert!(ret.is_ok());

        // (assert_return (invoke "get" (i32.const 3)) (ref.extern 1))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(3))]);
        assert_eq!(ret.unwrap().last().unwrap(), &extern_1);

        // (assert_return (invoke "get" (i32.const 4)) (ref.extern 2))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(4))]);
        assert_eq!(ret.unwrap().last().unwrap(), &extern_2);

        // (assert_return (invoke "get" (i32.const 5)) (ref.extern 2))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(5))]);
        assert_eq!(ret.unwrap().last().unwrap(), &extern_2);

        // (assert_return (invoke "fill" (i32.const 8) (ref.extern 4) (i32.const 2)))
        let ret = call_function(
            &inst,
            "fill",
            vec![
                Val::Num(Num::I32(8)),
                extern_4.clone(),
                Val::Num(Num::I32(2)),
            ],
        );
        assert!(ret.is_ok());

        // (assert_return (invoke "get" (i32.const 7)) (ref.null extern))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(7))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        // (assert_return (invoke "get" (i32.const 8)) (ref.extern 4))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(8))]);
        assert_eq!(ret.unwrap().last().unwrap(), &extern_4);

        // (assert_return (invoke "get" (i32.const 9)) (ref.extern 4))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(9))]);
        assert_eq!(ret.unwrap().last().unwrap(), &extern_4);

        // (assert_return (invoke "fill-abbrev" (i32.const 9) (ref.null extern) (i32.const 1)))
        let ret = call_function(
            &inst,
            "fill-abbrev",
            vec![
                Val::Num(Num::I32(9)),
                Val::Ref(Ref::RefNull),
                Val::Num(Num::I32(1)),
            ],
        );
        assert!(ret.is_ok());

        // (assert_return (invoke "get" (i32.const 8)) (ref.extern 4))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(8))]);
        assert_eq!(ret.unwrap().last().unwrap(), &extern_4);

        // (assert_return (invoke "get" (i32.const 9)) (ref.null extern))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(9))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        // (assert_return (invoke "fill" (i32.const 10) (ref.extern 5) (i32.const 0)))
        let ret = call_function(
            &inst,
            "fill",
            vec![
                Val::Num(Num::I32(10)),
                extern_5.clone(),
                Val::Num(Num::I32(0)),
            ],
        );
        assert!(ret.is_ok());

        // (assert_return (invoke "get" (i32.const 9)) (ref.null extern))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(9))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        // (assert_return (invoke "get" (i32.const 7)) (ref.null extern))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(7))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));

        // (assert_return (invoke "get" (i32.const 8)) (ref.extern 4))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(8))]);
        assert_eq!(ret.unwrap().last().unwrap(), &extern_4);

        // (assert_return (invoke "get" (i32.const 9)) (ref.null extern))
        let ret = call_function(&inst, "get", vec![Val::Num(Num::I32(9))]);
        assert_eq!(ret.unwrap().last().unwrap(), &Val::Ref(Ref::RefNull));
    }
}
