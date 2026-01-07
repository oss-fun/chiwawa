use chiwawa::parser;
use chiwawa::structure::module::{ImportDesc, WasiFuncType};
use chiwawa::structure::{instructions::*, module::*, types::*};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasi_func_type_expected_types() {
        // proc_exitの型チェック
        let proc_exit_type = WasiFuncType::ProcExit.expected_func_type();
        assert_eq!(proc_exit_type.params.len(), 1);
        assert_eq!(proc_exit_type.results.len(), 0);

        // fd_writeの型チェック
        let fd_write_type = WasiFuncType::FdWrite.expected_func_type();
        assert_eq!(fd_write_type.params.len(), 4);
        assert_eq!(fd_write_type.results.len(), 1);

        // random_getの型チェック
        let random_get_type = WasiFuncType::RandomGet.expected_func_type();
        assert_eq!(random_get_type.params.len(), 2);
        assert_eq!(random_get_type.results.len(), 1);

        // fd_prestat_getの型チェック
        let fd_prestat_get_type = WasiFuncType::FdPrestatGet.expected_func_type();
        assert_eq!(fd_prestat_get_type.params.len(), 2);
        assert_eq!(fd_prestat_get_type.results.len(), 1);

        // fd_prestat_dir_nameの型チェック
        let fd_prestat_dir_name_type = WasiFuncType::FdPrestatDirName.expected_func_type();
        assert_eq!(fd_prestat_dir_name_type.params.len(), 3);
        assert_eq!(fd_prestat_dir_name_type.results.len(), 1);

        // fd_closeの型チェック
        let fd_close_type = WasiFuncType::FdClose.expected_func_type();
        assert_eq!(fd_close_type.params.len(), 1);
        assert_eq!(fd_close_type.results.len(), 1);
    }

    #[test]
    fn test_wasi_func_type_variants() {
        // 各WASI関数型のバリアントが正しく定義されているかテスト
        let variants = vec![
            WasiFuncType::ProcExit,
            WasiFuncType::FdWrite,
            WasiFuncType::FdRead,
            WasiFuncType::RandomGet,
            WasiFuncType::FdPrestatGet,
            WasiFuncType::FdPrestatDirName,
            WasiFuncType::FdClose,
        ];

        // すべてのバリアントが期待される関数型を返すことを確認
        for variant in variants {
            let func_type = variant.expected_func_type();
            assert!(!func_type.params.is_empty() || matches!(variant, WasiFuncType::ProcExit));
            // proc_exit以外はすべて戻り値を持つ
            if !matches!(variant, WasiFuncType::ProcExit) {
                assert_eq!(func_type.results.len(), 1);
            }
        }
    }

    #[test]
    fn test_import_desc_wasi_func() {
        // ImportDesc::WasiFuncが正しく動作することをテスト
        let wasi_func = ImportDesc::WasiFunc(WasiFuncType::FdWrite);

        match wasi_func {
            ImportDesc::WasiFunc(func_type) => {
                assert!(matches!(func_type, WasiFuncType::FdWrite));
                let expected_type = func_type.expected_func_type();
                assert_eq!(expected_type.params.len(), 4);
                assert_eq!(expected_type.results.len(), 1);
            }
            _ => panic!("Expected WasiFunc variant"),
        }
    }

    #[test]
    fn parse_wasi_proc_exit_import() {
        let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "proc_exit" (func $proc_exit (param i32)))
            (func $main
                i32.const 0
                call $proc_exit
            )
            (start $main)
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_wasi_proc_exit.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true, "slot");
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        assert_eq!(module.imports.len(), 1);
        let import = &module.imports[0];
        assert_eq!(import.module.0, "wasi_snapshot_preview1");
        assert_eq!(import.name.0, "proc_exit");

        if let ImportDesc::WasiFunc(wasi_func_type) = &import.desc {
            assert_eq!(*wasi_func_type, WasiFuncType::ProcExit);
        } else {
            panic!("Expected WasiFunc import descriptor");
        }
    }

    #[test]
    fn parse_wasi_fd_write_import() {
        let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func $main
                i32.const 1  ;; stdout
                i32.const 0  ;; iovs ptr
                i32.const 1  ;; iovs len
                i32.const 4  ;; nwritten ptr
                call $fd_write
                drop
            )
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_wasi_fd_write.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true, "slot");
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        // インポートセクションの確認
        assert_eq!(module.imports.len(), 1);
        let import = &module.imports[0];
        assert_eq!(import.module.0, "wasi_snapshot_preview1");
        assert_eq!(import.name.0, "fd_write");

        // WASI関数として認識されているか確認
        if let ImportDesc::WasiFunc(wasi_func_type) = &import.desc {
            assert_eq!(*wasi_func_type, WasiFuncType::FdWrite);
        } else {
            panic!("Expected WasiFunc import descriptor");
        }
    }

    #[test]
    fn parse_wasi_fd_read_import() {
        let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_read" (func $fd_read (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func $main
                i32.const 0  ;; stdin
                i32.const 0  ;; iovs ptr
                i32.const 1  ;; iovs len
                i32.const 4  ;; nread ptr
                call $fd_read
                drop
            )
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_wasi_fd_read.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true, "slot");
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        // インポートセクションの確認
        assert_eq!(module.imports.len(), 1);
        let import = &module.imports[0];
        assert_eq!(import.module.0, "wasi_snapshot_preview1");
        assert_eq!(import.name.0, "fd_read");

        // WASI関数として認識されているか確認
        if let ImportDesc::WasiFunc(wasi_func_type) = &import.desc {
            assert_eq!(*wasi_func_type, WasiFuncType::FdRead);
        } else {
            panic!("Expected WasiFunc import descriptor");
        }
    }

    #[test]
    fn parse_wasi_random_get_import() {
        let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "random_get" (func $random_get (param i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func $main
                i32.const 0  ;; buf ptr
                i32.const 8  ;; buf len
                call $random_get
                drop
            )
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_wasi_random_get.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true, "slot");
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        // インポートセクションの確認
        assert_eq!(module.imports.len(), 1);
        let import = &module.imports[0];
        assert_eq!(import.module.0, "wasi_snapshot_preview1");
        assert_eq!(import.name.0, "random_get");

        // WASI関数として認識されているか確認
        if let ImportDesc::WasiFunc(wasi_func_type) = &import.desc {
            assert_eq!(*wasi_func_type, WasiFuncType::RandomGet);
        } else {
            panic!("Expected WasiFunc import descriptor");
        }
    }

    #[test]
    fn parse_multiple_wasi_imports() {
        let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "proc_exit" (func $proc_exit (param i32)))
            (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_read" (func $fd_read (param i32 i32 i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "random_get" (func $random_get (param i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_prestat_get" (func $fd_prestat_get (param i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_prestat_dir_name" (func $fd_prestat_dir_name (param i32 i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_close" (func $fd_close (param i32) (result i32)))
            (memory (export "memory") 1)
            (func $main
                i32.const 0
                call $proc_exit
            )
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_wasi_multiple_imports.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true, "slot");
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        // インポートセクションの確認
        assert_eq!(module.imports.len(), 7);

        let expected_wasi_funcs = [
            ("proc_exit", WasiFuncType::ProcExit),
            ("fd_write", WasiFuncType::FdWrite),
            ("fd_read", WasiFuncType::FdRead),
            ("random_get", WasiFuncType::RandomGet),
            ("fd_prestat_get", WasiFuncType::FdPrestatGet),
            ("fd_prestat_dir_name", WasiFuncType::FdPrestatDirName),
            ("fd_close", WasiFuncType::FdClose),
        ];

        for (i, (expected_name, expected_type)) in expected_wasi_funcs.iter().enumerate() {
            let import = &module.imports[i];
            assert_eq!(import.module.0, "wasi_snapshot_preview1");
            assert_eq!(import.name.0, *expected_name);

            if let ImportDesc::WasiFunc(wasi_func_type) = &import.desc {
                assert_eq!(*wasi_func_type, *expected_type);
            } else {
                panic!("Expected WasiFunc import descriptor for {}", expected_name);
            }
        }
    }

    #[test]
    fn parse_mixed_wasi_and_regular_imports() {
        let wat = r#"
        (module
            (import "env" "regular_func" (func $regular_func (param i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (import "env" "another_func" (func $another_func))
            (import "wasi_snapshot_preview1" "proc_exit" (func $proc_exit (param i32)))
            (memory (export "memory") 1)
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_wasi_mixed_imports.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true, "slot");
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        // インポートセクションの確認
        assert_eq!(module.imports.len(), 4);

        // 最初のインポート: 通常の関数
        let import0 = &module.imports[0];
        assert_eq!(import0.module.0, "env");
        assert_eq!(import0.name.0, "regular_func");
        assert!(matches!(import0.desc, ImportDesc::Func(_)));

        // 2番目のインポート: WASI関数
        let import1 = &module.imports[1];
        assert_eq!(import1.module.0, "wasi_snapshot_preview1");
        assert_eq!(import1.name.0, "fd_write");
        if let ImportDesc::WasiFunc(wasi_func_type) = &import1.desc {
            assert_eq!(*wasi_func_type, WasiFuncType::FdWrite);
        } else {
            panic!("Expected WasiFunc import descriptor");
        }

        // 3番目のインポート: 通常の関数
        let import2 = &module.imports[2];
        assert_eq!(import2.module.0, "env");
        assert_eq!(import2.name.0, "another_func");
        assert!(matches!(import2.desc, ImportDesc::Func(_)));

        // 4番目のインポート: WASI関数
        let import3 = &module.imports[3];
        assert_eq!(import3.module.0, "wasi_snapshot_preview1");
        assert_eq!(import3.name.0, "proc_exit");
        if let ImportDesc::WasiFunc(wasi_func_type) = &import3.desc {
            assert_eq!(*wasi_func_type, WasiFuncType::ProcExit);
        } else {
            panic!("Expected WasiFunc import descriptor");
        }
    }

    #[test]
    fn parse_unknown_wasi_function() {
        let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "unknown_wasi_func" (func $unknown (param i32) (result i32)))
            (memory (export "memory") 1)
        )"#;

        let binary = wat::parse_str(wat).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_wasi_unknown.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true, "slot");
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        // インポートセクションの確認
        assert_eq!(module.imports.len(), 1);
        let import = &module.imports[0];
        assert_eq!(import.module.0, "wasi_snapshot_preview1");
        assert_eq!(import.name.0, "unknown_wasi_func");

        // 未知のWASI関数は通常の関数インポートとして扱われる
        assert!(matches!(import.desc, ImportDesc::Func(_)));
    }

    #[test]
    fn parse_wasi_function_type_validation() {
        // 正しい型のfd_write
        let wat_correct = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
        )"#;

        let binary = wat::parse_str(wat_correct).unwrap();
        let mut module = Module::new("test");

        let temp_file = "temp_wasi_type_validation_correct.wasm";
        std::fs::write(temp_file, &binary).unwrap();

        let result = parser::parse_bytecode(&mut module, temp_file, true, "slot");
        assert!(result.is_ok());

        std::fs::remove_file(temp_file).unwrap();

        // WASI関数として認識されているか確認
        assert_eq!(module.imports.len(), 1);
        let import = &module.imports[0];
        if let ImportDesc::WasiFunc(wasi_func_type) = &import.desc {
            assert_eq!(*wasi_func_type, WasiFuncType::FdWrite);
        } else {
            panic!("Expected WasiFunc import descriptor");
        }
    }
}
