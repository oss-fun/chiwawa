mod data;
pub mod dispatch;
#[cfg(not(feature = "tco"))]
pub mod dispatch_loop;
#[cfg(feature = "tco")]
pub mod dispatch_tco;
mod elem;
mod export;
pub mod func;
pub(crate) mod global;
pub mod handlers;
pub mod ir;
pub mod mem;
pub mod migration;
pub mod module;
pub mod operand;
pub mod regs;
pub mod runtime;
pub mod state;
mod table;
pub mod value;
