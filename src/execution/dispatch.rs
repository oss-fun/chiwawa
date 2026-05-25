//! v2 dispatcher entry point — cfg-gated re-export of either the loop-style
//! or tail-call dispatcher.

#[cfg(feature = "tco")]
pub use crate::execution::dispatch_tco::execute_instructions;

#[cfg(not(feature = "tco"))]
pub use crate::execution::dispatch_loop::execute_instructions;
