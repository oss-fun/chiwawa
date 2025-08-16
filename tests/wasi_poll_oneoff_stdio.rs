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
        let func_addr = inst.get_export_func("_start")?;
        let mut runtime = Runtime::new(Arc::clone(inst), &func_addr, vec![], true)?;
        runtime.run()
    }

    #[test]
    fn test_poll_oneoff_stdio() {
        let inst = load_wasi_instance("tests/wasi/poll_oneoff_stdio.wasm");
        let result = run_wasi_module(&inst);

        result.expect("poll_oneoff_stdio should succeed");
    }
}
