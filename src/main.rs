use wasi_cap_std_sync::WasiCtxBuilder;
use anyhow::{anyhow, Result};
use wasmi::*;
use std::fs::File;
use std::io::{Read, BufReader};
use wasmi_wasi::{add_to_linker, WasiCtx};

fn main() -> Result <()>{
    let engine = Engine::default();

    let mut bytecode = Vec::new();
    let mut file = File::open("/home/chikuwait/migration-sample/hello.wasm")?;
    let mut reader = BufReader::new(file);

    let mut buf =[0; 20];
    reader.read(&mut buf);
    bytecode.extend(buf);
    let _ = reader.read_to_end(&mut bytecode)?;

    let module = Module::new_streaming(&engine, &bytecode[..]).unwrap();
    let mut linker = <wasmi::Linker<wasmi_wasi::WasiCtx>>::new(&engine);

    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .unwrap()
        .build();
    let mut store = Store::new(&engine, wasi);

    wasmi_wasi::add_to_linker(&mut linker, |ctx| ctx).map_err(|error| anyhow!("failed to add WASI definitions to the linker: {error}"))?;    ;
    let instance = linker
        .instantiate(&mut store, &module)?
        .start(&mut store)?;
        
    loop{
        let mut buf1 =[0; 4];
        match reader.read(&mut buf1)?{
            0 => break,
            n =>{
                bytecode.extend(buf1);
            }
        }
    }

    let add = instance.get_typed_func::<(), ()>(&store, "_start")?;
    let result = add.call(&mut store, ())?;
    Ok(())
}
