mod intercept;

use std::env;
use anyhow::{Result};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,

    /// Number of times to greet
    #[arg(short, long, default_value = "Wasmi")]
    runtime: String,
    #[arg(short, long, default_value = "shadow")]
    execute: String,
}


fn main() -> Result <()>{
    let args = Args::parse();

    match &*args.execute{
        "intercept" =>{
            intercept::run(&args.path, &args.runtime);
        }, 
        "shadow" => {

        }
        &_ => todo!()
    }

    Ok(())
}
