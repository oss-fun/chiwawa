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
        let _ = parser::parse_bytecode(&mut module, wasm_path, true);

        let imports: ImportObjects = FxHashMap::default();
        ModuleInst::new(&module, imports, args).unwrap()
    }

    fn run_wasi_module(inst: &Rc<ModuleInst>) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func("_start")?;
        let mut runtime = Runtime::new(
            Rc::clone(inst),
            &func_addr,
            vec![],
            true,
            false,
            false,
            None,
        )?;
        runtime.run()
    }

    #[test]
    fn test_file_seek_tell() {
        let scratch_dir = "tests/testdir";

        // Clean up any existing file_seek_tell_dir.cleanup directory
        let cleanup_dir = format!("{}/file_seek_tell_dir.cleanup", scratch_dir);
        if std::path::Path::new(&cleanup_dir).exists() {
            std::fs::remove_dir_all(&cleanup_dir).ok();
        }

        let args = vec![
            "file_seek_tell.wasm".to_string(), // argv[0] - program name
            scratch_dir.to_string(),           // argv[1] - scratch directory
        ];

        let inst = load_wasi_instance_with_args("tests/wasi/file_seek_tell.wasm", args);
        let result = run_wasi_module(&inst);

        result.expect("file_seek_tell should succeed");
    }
}
