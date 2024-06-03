use wasmi::*;
use wasmer::{Store, Module, Instance, Value, imports};
use wasmer_wasix::WasiEnv;
use std::thread::Builder;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UnderRuntimeError{
    #[error("Failed to Create Instance")]
    CreateInstance,
    #[error("Failed to Call Function")]
    FunctionCall,
    #[error("Unknown Error")]
    Unknown,
}

pub trait UnderRuntime {
    fn extend_bytecode(&mut self, b: Vec<u8>);
    fn call_function(&mut self, func_name: &str) -> Result<(),UnderRuntimeError>;
}

pub struct Wasmi {
    bytecode: Vec<u8>,
    engine: wasmi::Engine,
    module: wasmi::Module,
    store: wasmi::Store<wasmi_wasi::WasiCtx>,
    linker: wasmi::Linker<wasmi_wasi::WasiCtx>,
    instance: wasmi::Instance,
}

pub struct Wasmer {
    bytecode: Vec<u8>,
    store: wasmer::Store,
    module: wasmer::Module,
    runtime: tokio::runtime::Runtime,
    instance: wasmer::Instance,
}

pub fn CreateInstance(target: &str, bytecode: Vec<u8>) -> Box<dyn UnderRuntime>{
    match target{
        "Wasmi" => {
            let engine = wasmi::Engine::default();
            let module = wasmi::Module::new_streaming(&engine, &bytecode[..]).unwrap();
            let mut linker = <wasmi::Linker<wasmi_wasi::WasiCtx>>::new(&engine);
            let _= wasmi_wasi::add_to_linker(&mut linker, |ctx| ctx);
            let mut store = wasmi::Store::new(&engine, wasi_cap_std_sync::WasiCtxBuilder::new().inherit_stdio().inherit_args().unwrap().build());
            let instance = linker.instantiate(&mut store, &module).unwrap().start(&mut store).unwrap();
            Box::new(Wasmi{bytecode, engine, module, store, linker, instance})
        },
        "Wasmer" =>{
            let mut store = wasmer::Store::default();
            let module = wasmer::Module::new(&store, &bytecode).unwrap();
            let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
            let _guard = runtime.enter();
            let mut wasi_env = wasmer_wasix::WasiEnv::builder("test").finalize(&mut store).unwrap();
            let import_object = wasi_env.import_object(&mut store, &module).unwrap();
            let instance = wasmer::Instance::new(&mut store, &module, &import_object).unwrap();
            wasi_env.initialize(&mut store, instance.clone());
            Box::new(Wasmer{bytecode, store, module, runtime, instance})
        }
        &_ => todo!()
    }
}

impl UnderRuntime for Wasmi {
    fn extend_bytecode(&mut self, b: Vec<u8>) {
        self.bytecode.extend(b);
    }

    fn call_function(&mut self, func_name: &str) -> Result<(),UnderRuntimeError>{
        let add = self.instance.get_typed_func::<(), ()>(&self.store, func_name).unwrap();
        let result = add.call(&mut self.store, ());
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(UnderRuntimeError::FunctionCall),
        }
    }
}

impl UnderRuntime for Wasmer {

    fn extend_bytecode(&mut self, b: Vec<u8>) {
        self.bytecode.extend(b);
    }

    fn call_function(&mut self, func_name: &str) -> Result<(), UnderRuntimeError>{
        let func = self.instance.exports.get_function(func_name).unwrap();
        let result = func.call(&mut self.store, &[]);
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(UnderRuntimeError::FunctionCall),
        }
    }
}