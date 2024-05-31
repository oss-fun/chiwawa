mod underRuntime;

use std::fs::File;
use std::io::{Read, BufReader};
use std::env;
use crate::underRuntime::UnderRuntime;
use anyhow::{Result};


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

    let mut runtime = underRuntime::Wasmi::new(bytecode);

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
    }

    Ok(())
}
