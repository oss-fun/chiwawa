//! TCO (tail-call) dispatcher.
//!
//! Active when the `tco` feature is set. Requires runtime support for the
//! Wasm `tail-call` proposal (Wasmtime v28+). Each handler tail-calls the
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
/// last entry set to a sentinel handler (e.g., `h_halt`) so out-of-range
/// dispatch terminates safely.
pub fn run(state: &mut VmState) -> Outcome {
    if state.enable_checkpoint {
        #[cfg(all(
            target_arch = "wasm32",
            target_os = "wasi",
            target_env = "p1",
            target_feature = "atomics"
        ))]
        {
            if migration::check_checkpoint_flag() {
                state.trap = Some(RuntimeError::CheckpointRequested);
                return Outcome::Trap;
            }
        }
    }
    if state.pc >= state.instrs_len {
        return Outcome::Halt;
    }
    let h = unsafe { *state.handlers.add(state.pc) };
    h(state)
}
