use chiwawa::{
    execution::module::*, execution::runtime::Runtime, execution::value::*, parser,
    structure::module::Module,
};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    fn load_wasi_instance(wasm_path: &str) -> Arc<ModuleInst> {
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path, true);

        let imports: ImportObjects = HashMap::new();
        let app_args = vec![
            "path_open_read_write.wasm".to_string(),
            "tests/testdir".to_string(),
        ];
        ModuleInst::new(&module, imports, Vec::new(), app_args).unwrap()
    }

    fn run_wasi_module(inst: &Arc<ModuleInst>) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func("_start")?;
        let mut runtime = Runtime::new(Arc::clone(inst), &func_addr, vec![], true)?;
        runtime.run()
    }

    #[test]
    fn test_path_open_read_write() {
        let inst = load_wasi_instance("tests/wasi/path_open_read_write.wasm");
        let result = run_wasi_module(&inst);

        match result {
            Ok(_) => {
                println!("path_open_read_write test passed successfully");
            }
            Err(e) => {
                panic!("path_open_read_write test failed: {:?}", e);
            }
        }
    }
}
