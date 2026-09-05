pub(crate) mod atomics;
mod data;
mod dispatch;
#[cfg(not(feature = "tco"))]
mod dispatch_loop;
#[cfg(feature = "tco")]
mod dispatch_tco;
mod elem;
mod export;
pub mod func;
pub(crate) mod global;
pub(crate) mod handlers;
pub(crate) mod ir;
pub(crate) mod mem;
pub mod migration;
pub mod module;
mod operand;
pub(crate) mod regs;
pub mod runtime;
pub mod state;
mod table;
pub mod value;
