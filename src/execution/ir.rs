//! v2 dispatcher IR types.
//!
//! Defines the `Handler` function pointer type and `Outcome` enum used by
//! both the loop-style and tail-call dispatchers.

use crate::execution::state::VmState;

/// Outcome of a handler invocation.
///
/// In the loop-style dispatcher, handlers return `Continue` to indicate the
/// outer loop should fetch the next instruction. In the tail-call dispatcher,
/// handlers tail-call the next handler directly and `Continue` is normally
/// not seen by the dispatcher driver.
///
/// Trap conditions store the error in `state.trap`. Function-level yields
/// (call/return/wasi) store the `ModuleLevelInstr` in `state.yielded`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Loop dispatcher: fetch next instruction. TCO dispatcher: chain continues.
    Continue = 0,
    /// Function body ended naturally (reached `end` at function level).
    Halt = 1,
    /// Trap occurred. Details in `state.trap`.
    Trap = 2,
    /// Yielding to runtime for frame transition. Details in `state.yielded`.
    Yield = 3,
}

/// All v2 dispatcher handlers share this signature so the function pointer
/// type matches identically — required for `return_call_indirect` in the
/// TCO dispatcher.
pub type Handler = fn(&mut VmState) -> Outcome;
