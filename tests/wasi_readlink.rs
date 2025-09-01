use chiwawa::{
    execution::module::*, execution::runtime::Runtime, execution::value::*, parser,
    structure::module::Module,
};
use rustc_hash::FxHashMap;
use std::rc::Rc;

#[cfg(test)]
mod tests {
    use super::*;

    fn load_wasi_instance(wasm_path: &str) -> Rc<ModuleInst> {
        let mut module = Module::new("test");
        let _ = parser::parse_bytecode(&mut module, wasm_path, true);

        let imports: ImportObjects = FxHashMap::default();
        let app_args = vec!["readlink.wasm".to_string(), "tests/testdir".to_string()];
        ModuleInst::new(&module, imports, app_args).unwrap()
    }

    fn run_wasi_module(inst: &Rc<ModuleInst>) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func("_start")?;
        let mut runtime = Runtime::new(Rc::clone(inst), &func_addr, vec![], true, false, false)?;
        runtime.run()
    }

    #[test]
    fn test_readlink() {
        // Clean up any leftover files from previous test runs
        let _ = std::fs::remove_file("tests/testdir/symlink");
        let _ = std::fs::remove_file("tests/testdir/target");

        let inst = load_wasi_instance("tests/wasi/readlink.wasm");
        let result = run_wasi_module(&inst);

        result.expect("readlink test should succeed");
    }
}
