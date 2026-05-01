//! v2 dispatcher entry point — cfg-gated re-export of either the loop-style
//! or tail-call dispatcher.

#[cfg(feature = "tco")]
pub use crate::execution::dispatch_tco::run;

#[cfg(not(feature = "tco"))]
pub use crate::execution::dispatch_loop::run;
