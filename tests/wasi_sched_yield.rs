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
        ModuleInst::new(&module, imports, Vec::new()).unwrap()
    }

    fn run_wasi_module(inst: &Arc<ModuleInst>) -> Result<Vec<Val>, chiwawa::error::RuntimeError> {
        match inst.get_export_func("_start") {
            Ok(func_addr) => {
                let mut runtime = Runtime::new(Arc::clone(inst), &func_addr, vec![], true)?;
                runtime.run()
            }
            Err(_) => {
                // If no _start function, try main
                let func_addr = inst.get_export_func("main")?;
                let mut runtime = Runtime::new(Arc::clone(inst), &func_addr, vec![], true)?;
                runtime.run()
            }
        }
    }

    #[test]
    fn test_sched_yield() {
        let inst = load_wasi_instance("tests/wasi/sched_yield.wasm");
        let result = run_wasi_module(&inst);

        match result {
            Ok(_) => {
                // Test passed
            }
            Err(e) => {
                panic!("sched_yield failed: {:?}", e);
            }
        }
    }
}
