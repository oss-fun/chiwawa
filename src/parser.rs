//! WebAssembly binary parsing.
//!
//! - [`core`]: core modules, decoded into a [`Module`] with its instructions preprocessed
//! - [`component`]: components, decoded into a [`Component`] with every index space resolved

use crate::shared::Shared;
use crate::structure::component::Component;
use crate::structure::module::Module;

pub mod component;
pub mod core;

pub use self::core::{parse_bytecode, parse_module_bytes, ParseOutput};

pub enum Parsed {
    Module(Module, ParseOutput),
    Component(Shared<Component>),
}

pub fn parse_file(path: &str) -> Result<Parsed, Box<dyn std::error::Error>> {
    parse_bytes(&std::fs::read(path)?)
}

pub fn parse_bytes(buf: &[u8]) -> Result<Parsed, Box<dyn std::error::Error>> {
    if wasmparser::Parser::is_component(buf) {
        return Ok(Parsed::Component(component::parse_component_bytes(buf)?));
    }
    let mut module = Module::new("main");
    let output = parse_module_bytes(&mut module, buf)?;
    Ok(Parsed::Module(module, output))
}
