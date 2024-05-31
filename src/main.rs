mod underRuntime;

use std::fs::File;
use std::io::{Read, BufReader};
use std::env;
use crate::underRuntime::UnderRuntime;
use anyhow::{Result};
use wasmer::{Store, Module, Instance, Value, imports};
use wasmer_wasix::WasiEnv;
use std::thread::Builder;

fn main() -> Result <()>{
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let mut bytecode = Vec::new();
    
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    //let mut buf =[0; 8];
    //let _= reader.read(&mut buf);
    //bytecode.extend(buf);
    //let lf: [u8; 1] = [0x0a];
    //bytecode.extend(lf.to_vec());
    let _ = reader.read_to_end(&mut bytecode)?;

    //let mut runtime = underRuntime::Wasmi::new(bytecode);
    let mut store = wasmer::Store::default();
    let module = wasmer::Module::new(&store, &bytecode)?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap();
    let _guard = runtime.enter();

    let mut wasi_env = WasiEnv::builder("hello")
        // .args(&["world"])
        // .env("KEY", "Value")
        .finalize(&mut store)?;
    let import_object = wasi_env.import_object(&mut store, &module)?;
    let instance = Instance::new(&mut store, &module, &import_object)?;
    wasi_env.initialize(&mut store, instance.clone())?;
    let func = instance.exports.get_function("_start")?;
    let _ = func.call(&mut store, &[])?;

/*
    loop{
        let mut buf1 =[0; 4];
        match reader.read(&mut buf1)?{
            0 => break,
            _ =>{
               runtime.extend_bytecode(buf1.to_vec());
            }
        }
    }
    match runtime.call_function("_start"){
        Ok(_) => println!("Run Successful"),
        Err(_) => println!("Run Faild!"),
    }*/

    Ok(())
}
