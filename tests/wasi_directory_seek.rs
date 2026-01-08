use chiwawa::{
    execution::module::*, execution::runtime::Runtime, execution::value::*, parser,
    structure::module::Module,
};
use rustc_hash::FxHashMap;
use std::rc::Rc;

#[cfg(test)]
mod tests {
    use super::*;

    fn load_wasi_instance_with_args(wasm_path: &str, args: Vec<String>) -> Rc<ModuleInst> {
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path, true, "slot");

        let imports: ImportObjects = FxHashMap::default();
        ModuleInst::new(&module, imports, args).unwrap()
    }

    fn run_wasi_module(inst: &Rc<ModuleInst>) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func("_start")?;
        let mut runtime = Runtime::new(Rc::clone(inst), &func_addr, vec![], true, false, None)?;
        runtime.run()
    }

    #[test]
    fn test_directory_seek() {
        let scratch_dir = "tests/testdir";

        let args = vec![
            "directory_seek.wasm".to_string(), // argv[0] - program name
            scratch_dir.to_string(),           // argv[1] - scratch directory
        ];

        let inst = load_wasi_instance_with_args("tests/wasi/directory_seek.wasm", args);
        let result = run_wasi_module(&inst);

        // Clean up after test
        let cleanup_dir = format!("{}/directory_seek_dir.cleanup", scratch_dir);
        if std::path::Path::new(&cleanup_dir).exists() {
            std::fs::remove_dir_all(&cleanup_dir).ok();
        }

        result.expect("directory_seek should succeed");
    }
}
