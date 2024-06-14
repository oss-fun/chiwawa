mod underRuntime;

use std::fs::File;
use std::io::{Read, BufReader};
use crate::intercept::underRuntime::*;

pub fn run(path: &str, target: &str) -> Result<(), UnderRuntimeError>{
    let mut bytecode = Vec::new();    
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);

    let mut buf =[0; 8];
    let _= reader.read(&mut buf);
    bytecode.extend(buf);
    //let lf: [u8; 1] = [0x0a];
    //bytecode.extend(lf.to_vec());
    let _ = reader.read_to_end(&mut bytecode);

    let mut runtime = underRuntime::CreateInstance(target, bytecode);

/*
    loop{
        let mut buf1 =[0; 4];
        match reader.read(&mut buf1)?{
            0 => break,
            _ =>{
               runtime.extend_bytecode(buf1.to_vec());
            }
        }
    }*/
    runtime.call_function("_start")
}