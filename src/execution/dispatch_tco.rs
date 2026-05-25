//! TCO (tail-call) dispatcher.
//!
//! Active when the `tco` feature is set. Requires runtime support for the
//! Wasm `tail-call` proposal.  Each handler tail-calls the
//! next handler; LLVM with `+tail-call` emits `return_call_indirect`.

#![cfg(feature = "tco")]

use crate::error::RuntimeError;
use crate::execution::ir::Outcome;
use crate::execution::migration;
use crate::execution::state::VmState;

/// Kick off the tail-call chain at `state.pc`. Each handler self-perpetuates
/// the chain via the `advance!` macro until a sentinel returns Halt/Trap/Yield.
///
/// # Safety
/// `state` must have all pointer fields valid for the duration of the call.
/// `state.handlers` must be at least `state.instrs_len + 1` long, with the
/// last entry set to a sentinel handler (e.g., `halt`) so out-of-range
/// dispatch terminates safely.
pub fn run(state: &mut VmState) -> Outcome {
    if migration::poll_checkpoint(state) {
        state.trap = Some(RuntimeError::CheckpointRequested);
        return Outcome::Trap;
    }
    if state.pc >= state.instrs_len {
        return Outcome::Halt;
    }
    let h = unsafe { *state.handlers.add(state.pc) };
    h(state)
}
