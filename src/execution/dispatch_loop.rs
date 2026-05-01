//! Non-TCO (loop-style) dispatcher.
//!
//! Active when the `tco` feature is **not** set. WAMR-compatible default.
//! Runs a `loop { fetch handler; call; match outcome }` until one of the
//! sentinel outcomes (Halt/Trap/Yield) terminates dispatch.

#![cfg(not(feature = "tco"))]

use crate::error::RuntimeError;
use crate::execution::ir::Outcome;
use crate::execution::migration;
use crate::execution::state::VmState;

/// Drive the dispatcher until an outcome other than `Continue` is returned.
///
/// # Safety
/// `state` must have all pointer fields valid for the duration of the call.
pub fn run(state: &mut VmState) -> Outcome {
    loop {
        // Checkpoint hook: same placement as legacy run_dtc_loop.
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

        // Natural pc-overflow handling: pop nested label or halt at function level.
        if state.pc >= state.instrs_len {
            if state.current_label_idx > 0 {
                let (return_ip, is_loop) = unsafe {
                    let ls = &(*state.label_stack)[state.current_label_idx];
                    (ls.label.return_ip, ls.label.is_loop)
                };
                let cur_ip = state.pc;
                if return_ip <= cur_ip && !is_loop {
                    return Outcome::Halt;
                }
                unsafe {
                    (*state.label_stack).pop();
                }
                state.current_label_idx -= 1;
                state.pc = if is_loop { cur_ip + 1 } else { return_ip };
                continue;
            } else {
                return Outcome::Halt;
            }
        }

        let h = unsafe { *state.handlers.add(state.pc) };
        match h(state) {
            Outcome::Continue => continue,
            other => return other,
        }
    }
}
