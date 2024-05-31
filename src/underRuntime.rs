use wasmi::*;

pub trait UnderRuntime {
    fn new(bytecode: Vec<u8>) -> Self;
    fn extend_bytecode(&mut self, b: Vec<u8>);
    fn call_function(&mut self, func_name: &str);
}

pub struct Wasmi {
    bytecode: Vec<u8>,
    engine: wasmi::Engine,
    module: wasmi::Module,
    store: wasmi::Store<wasmi_wasi::WasiCtx>,
    linker: wasmi::Linker<wasmi_wasi::WasiCtx>,
    instance: wasmi::Instance,
}

impl UnderRuntime for Wasmi {
    fn new(bytecode: Vec<u8>) -> Wasmi{
        let engine = Engine::default();
        let module = Module::new_streaming(&engine, &bytecode[..]).unwrap();
        let mut linker = <wasmi::Linker<wasmi_wasi::WasiCtx>>::new(&engine);
        let _= wasmi_wasi::add_to_linker(&mut linker, |ctx| ctx);
        let mut store = Store::new(&engine, wasi_cap_std_sync::WasiCtxBuilder::new().inherit_stdio().inherit_args().unwrap().build());
        let instance = linker.instantiate(&mut store, &module).unwrap().start(&mut store).unwrap();
        Wasmi{
            bytecode,
            engine: engine,
            module: module,
            linker: linker,
            store: store, 
            instance: instance,
        }
    }

    fn extend_bytecode(&mut self, b: Vec<u8>) {
        self.bytecode.extend(b);
    }

    fn call_function(&mut self, func_name: &str) {
        let add = self.instance.get_typed_func::<(), ()>(&self.store, func_name).unwrap();
        let result = add.call(&mut self.store, ());
    }
}