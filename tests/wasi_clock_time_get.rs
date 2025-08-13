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
        let app_args = vec![];
        ModuleInst::new(&module, imports, app_args).unwrap()
    }

    fn run_wasi_module(inst: &Arc<ModuleInst>) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        let func_addr = inst.get_export_func("_start")?;
        let mut runtime = Runtime::new(Arc::clone(inst), &func_addr, vec![], true)?;
        runtime.run()
    }

    #[test]
    fn test_clock_time_get() {
        let inst = load_wasi_instance("tests/wasi/clock_time_get.wasm");
        let result = run_wasi_module(&inst);

        match result {
            Ok(_) => {
                println!("clock_time_get test passed successfully");
            }
            Err(e) => {
                panic!("clock_time_get test failed: {:?}", e);
            }
        }
    }
}
