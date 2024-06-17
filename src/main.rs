use anyhow::{Result};
use clap::Parser;
mod parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,
}


fn main() -> Result <()>{
    let args = Args::parse();
    let _ = parser::parse_bytecode(&args.path);
    Ok(())
}
