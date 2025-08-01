use chiwawa::parser;
use chiwawa::structure::{instructions::*, module::*, types::*};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_type_section() {
        let wat = r#"
        (module
            (func (param i32 i32 i64))
            (func (result i32 i32))
            (func (param i32 i32) (result i32))
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        // parse_bytecodeを使用してテストする
        let temp_file = "temp_decode_type_section.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true);
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        let len = module.types.len();
        let exptect_param = [3, 0, 2];
        let exptect_result = [0, 2, 1];
        for i in 0..len {
            let params = &module.types[i].params;
            let results = &module.types[i].results;
            assert_eq!(params.len(), exptect_param[i]);
            assert_eq!(results.len(), exptect_result[i]);
        }
        assert_eq!(len, 3);
    }

    #[test]
    fn decode_func_section() {
        let wat = r#"
        (module
            (import "test" "test" (func (param i32))) 
            (import "test" "test" (func (param i32 i32) (result i32))) 
            (func (param i32 i32 i64))
            (func (result i32 i32)) 
            (func (param i32 i32) (result i32))
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_decode_func_section.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true);
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        let funcs_num = module.funcs.len();
        assert_eq!(funcs_num, 3);

        let exptect_idx = [2, 3, 1];
        for i in 0..funcs_num {
            let idx = &module.funcs[i].type_;
            assert_eq!(idx.0, exptect_idx[i]);
        }
    }

    #[test]
    fn decode_import_section() {
        let wat = r#"
        (module
            (import "module1" "func1" (func (param i32))) 
            (import "module2" "func1" (func (param i32 i32) (result i32)))
            (import "module2" "func2" (func (param i32 i64) (result f32))) 
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_decode_import_section.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true);
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        let imports_num = module.imports.len();
        assert_eq!(imports_num, 3);

        let module_names = ["module1", "module2", "module2"];
        let names = ["func1", "func1", "func2"];
        for i in 0..imports_num {
            let module_name = &module.imports[i].module.0;
            let name = &module.imports[i].name.0;
            let desc = &module.imports[i].desc;
            assert_eq!(module_name, module_names[i]);
            assert_eq!(name, names[i]);
            assert!(matches!(desc, ImportDesc::Func(TypeIdx(_))));
        }
    }

    #[test]
    fn decode_export_section() {
        let wat = r#"
        (module
            (memory (export "memory") 2 3)
            (func $add (export "add") (param $a i32) (param $b i32) (result i32)
                (i32.add (local.get $a) (local.get $b))
            )
            (func $sub (export "sub") (param $a i32) (param $b i32) (result i32)
                (i32.sub (local.get $a) (local.get $b))
            )
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_decode_export_section.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true);
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        let exports_num = module.exports.len();
        assert_eq!(exports_num, 3);

        let names = ["memory", "add", "sub"];
        for i in 0..exports_num {
            let name = &module.exports[i].name.0;
            assert_eq!(name, names[i]);
        }
    }

    #[test]
    fn decode_mem_section() {
        let wat = r#"
        (module
            (memory (export "memory") 2 3)
            (memory (export "mem") 1)
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_decode_mem_section.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true);
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        let memory_num = module.mems.len();
        assert_eq!(memory_num, 2);

        let expects_min = [2, 1];
        for i in 0..memory_num {
            let limits = &module.mems[i].type_.0;
            let min = limits.min;
            let max = limits.max;
            assert_eq!(min, expects_min[i]);
            if i == 0 {
                assert_eq!(max, Some(3));
            } else {
                assert_eq!(max, None);
            }
        }
    }

    #[test]
    fn decode_table_section() {
        let wat = r#"
        (module
            (table $t0 (export "table1") 2 externref)
            (table $t1 (export "table2") 3 funcref)
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_decode_table_section.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true);
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        let table_num = module.tables.len();
        assert_eq!(table_num, 2);

        let mut limits = &module.tables[0].type_.0;
        let mut reftype = &module.tables[0].type_.1;
        let mut min = limits.min;
        let mut max = limits.max;
        assert_eq!(min, 2);
        assert_eq!(max, None);
        assert!(matches!(reftype, RefType::ExternalRef));

        limits = &module.tables[1].type_.0;
        reftype = &module.tables[1].type_.1;
        min = limits.min;
        max = limits.max;
        assert_eq!(min, 3);
        assert_eq!(max, None);
        assert!(matches!(reftype, RefType::FuncRef));
    }

    #[test]
    fn decode_global_section() {
        let wat = r#"
        (module
            (global $f32 f32)
            (global $f64 (mut i64)(i64.const 2024))
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_decode_global_section.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true);
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        let global_num = module.globals.len();
        assert_eq!(global_num, 2);

        let mut type_ = &module.globals[0].type_;
        let mut init = &module.globals[0].init;
        assert!(matches!(type_.0, Mut::Const));
        assert!(matches!(type_.1, ValueType::NumType(NumType::F32)));
        assert_eq!(init.0.len(), 0);

        type_ = &module.globals[1].type_;
        init = &module.globals[1].init;
        assert!(matches!(type_.0, Mut::Var));
        assert!(matches!(type_.1, ValueType::NumType(NumType::I64)));
        assert_eq!(init.0.len(), 1);
        assert!(matches!(init.0[0], Instr::I64Const(2024)));
    }

    #[test]
    fn decode_elem_section() {
        let wat = r#"
        (module
            (table $t0 (export "table1") 2 funcref)
       
            (func $f1 (result i32)
                i32.const 1
            )
            (func $f2 (result i32)
                i32.const 2
            )
            (func $f3 (result i32)
                i32.const 3
            )
            (elem $t0(i32.const 0) $f1 $f2)
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_decode_elem_section.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true);
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        let elem_num = module.elems.len();
        assert_eq!(elem_num, 1);

        let elem = &module.elems[0];
        assert!(matches!(elem.type_, RefType::FuncRef));
        assert!(matches!(elem.mode, ElemMode::Active));
        assert_eq!(elem.table_idx, Some(TableIdx(0)));
    }

    #[test]
    fn decode_data_section() {
        //Test Code: https://github.com/eliben/wasm-wat-samples
        let wat = r#"
        (module
            (memory (export "memory") 1 100)
                (data (i32.const 0x0000)
                    "\67\68\69\70\AA\FF\DF\CB"
                    "\12\A1\32\B3\A5\1F\01\02"
                )
                (data (i32.const 0x020)
                    "\01\03\05\07\09\0B\0D\0F"
                )
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_decode_data_section.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true);
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        let data_num = module.datas.len();
        assert_eq!(data_num, 2);

        let mut init = &module.datas[0].init;
        let mut mode = &module.datas[0].mode;
        let mut memory = &module.datas[0].memory;
        let mut offset = &module.datas[0].offset;
        assert_eq!(init[0].0, 0x67);
        assert_eq!(init[7].0, 0xCB);
        assert!(matches!(mode, DataMode::Active));
        assert!(matches!(memory, Some(MemIdx(0))));
        let expected = Expr(vec![Instr::I32Const(0)]);
        assert_eq!(*offset, Some(expected));

        init = &module.datas[1].init;
        mode = &module.datas[1].mode;
        memory = &module.datas[1].memory;
        offset = &module.datas[1].offset;
        assert_eq!(init[0].0, 0x01);
        assert_eq!(init[7].0, 0x0F);
        assert!(matches!(mode, DataMode::Active));
        assert!(matches!(memory, Some(MemIdx(0))));
        let expected = Expr(vec![Instr::I32Const(0x020)]);
        assert_eq!(*offset, Some(expected));
    }

    #[test]
    fn decode_code_section_if_else() {
        //Test Code: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Control_flow/if...else
        let wat = r#"
        (module
            (func $ifexpr (result i32)
                i32.const 0
                (if (result i32)
                (then
                    ;; do something
                    (i32.const 1)
                )
                (else
                    ;; do something else
                    (i32.const 2)
                )
                )
            )
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_decode_code_section_if_else.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true);
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        let func_num = module.funcs.len();
        assert_eq!(func_num, 1);
    }

    #[test]
    fn decode_code_section_loop() {
        //Test Code: https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Control_flow/loop
        let wat = r#"
        (module
            (func $loop (result i32)
                (local $i i32)
                (local.set $i (i32.const 0))
                (loop $my_loop (result i32)
                    (local.set $i (i32.add (local.get $i) (i32.const 1)))
                    (br_if $my_loop (i32.lt_s (local.get $i) (i32.const 10)))
                    (local.get $i)
                )
            )
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_decode_code_section_loop.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true);
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        let func_num = module.funcs.len();
        assert_eq!(func_num, 1);
    }
}
