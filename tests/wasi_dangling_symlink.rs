use chiwawa::{
    execution::module::*, execution::runtime::Runtime, execution::value::*, parser,
    structure::module::Module,
};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    fn load_wasi_instance_with_args(wasm_path: &str, args: Vec<String>) -> Arc<ModuleInst> {
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path, true);

        let imports: ImportObjects = HashMap::new();
        ModuleInst::new(&module, imports, args).unwrap()
    }

    fn run_wasi_module(inst: &Arc<ModuleInst>) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func("_start")?;
        let mut runtime = Runtime::new(Arc::clone(inst), &func_addr, vec![], true)?;
        runtime.run()
    }

    #[test]
    fn test_dangling_symlink() {
        let scratch_dir = "tests/testdir";

        // Clean up any existing dangling_symlink test files
        let cleanup_files = vec![
            format!("{}/dangling_symlink_file.cleanup", scratch_dir),
            format!("{}/dangling_symlink_link.cleanup", scratch_dir),
        ];

        for file in &cleanup_files {
            if std::path::Path::new(file).exists() {
                std::fs::remove_file(file).ok();
            }
        }

        let args = vec![
            "dangling_symlink.wasm".to_string(), // argv[0] - program name
            scratch_dir.to_string(),             // argv[1] - scratch directory
        ];

        let inst = load_wasi_instance_with_args("tests/wasi/dangling_symlink.wasm", args);
        let result = run_wasi_module(&inst);

        // Clean up after test
        for file in &cleanup_files {
            if std::path::Path::new(file).exists() {
                std::fs::remove_file(file).ok();
            }
        }

        result.expect("dangling_symlink should succeed");
    }
}
