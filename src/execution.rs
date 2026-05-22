mod data;
pub mod dispatch;
#[cfg(not(feature = "tco"))]
pub mod dispatch_loop;
#[cfg(feature = "tco")]
pub mod dispatch_tco;
mod elem;
mod export;
pub mod func;
mod global;
pub mod handlers;
pub mod ir;
pub mod mem;
pub mod migration;
pub mod module;
pub mod operand;
pub mod ops;
pub mod regs;
pub mod runtime;
pub mod state;
pub mod stats;
mod table;
#[cfg(feature = "trace")]
pub mod trace;
pub mod value;
