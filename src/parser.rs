//! WebAssembly binary parsing.
//!
//! - [`core`]: core modules, decoded into a [`Module`](crate::structure::module::Module) with its instructions preprocessed
//! - [`component`]: components, decoded into a [`Component`](crate::structure::component::Component) with every index space resolved

pub mod component;
pub mod core;

pub use self::core::{parse_bytecode, parse_module_bytes, ParseOutput};
