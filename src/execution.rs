mod data;
mod elem;
mod export;
pub mod func;
mod global;
pub mod mem;
pub mod migration;
pub mod module;
pub mod regs;
pub mod runtime;
pub mod stats;
mod table;
#[cfg(feature = "trace")]
pub mod trace;
pub mod value;
pub mod vm;
