use chiwawa::structure::module::{ImportDesc, WasiFuncType};

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
