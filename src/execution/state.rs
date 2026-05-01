//! v2 dispatcher execution state.
//!
//! `VmState` aggregates everything a handler needs to execute one instruction:
//! register file, locals, instruction stream, label stack, memory pointer,
//! module reference, and outcome channels. Fields are raw pointers so all
//! handlers share an identical `fn(&mut VmState) -> Outcome` signature
//! (required for `return_call_indirect` type identity in TCO mode).

use crate::error::RuntimeError;
use crate::execution::ir::Handler;
use crate::execution::module::ModuleInst;
use crate::execution::regs::{Reg, RegFile};
use crate::execution::value::Val;
use crate::execution::vm::{LabelStack, ModuleLevelInstr, ProcessedInstr};
use arrayvec::ArrayVec;

/// Per-call dispatcher state. Constructed at the entry of each
/// `dispatch::run` call from the active `FrameStack`.
///
/// # Safety
/// All raw pointers must outlive the dispatch call. The runtime constructs
/// VmState from a `&mut FrameStack`, runs dispatch, and discards VmState
/// before the FrameStack is dropped or borrowed elsewhere.
pub struct VmState {
    // Register / locals
    pub reg_file: *mut RegFile,
    pub locals: *mut Val,
    pub locals_len: usize,

    // Active label's instruction stream + cached handler array
    // (invariant within a frame because all label stacks share the same Rc)
    pub pc: usize,
    pub instrs: *const ProcessedInstr,
    pub instrs_len: usize,
    pub handlers: *const Handler,

    // Label stack management (Br/BrIf/End/Block/If/Jump mutate these)
    pub label_stack: *mut Vec<LabelStack>,
    pub current_label_idx: usize,

    // Memory fast path (load/store)
    pub mem_ptr: *mut u8,

    // Module (call/call_indirect/global access)
    pub module: *const ModuleInst,

    // Outcome channels
    pub trap: Option<RuntimeError>,
    pub yielded: Option<ModuleLevelInstr>,
    pub return_result_regs: *mut ArrayVec<Reg, 8>,

    // Per-frame flags
    pub enable_checkpoint: bool,
}

impl VmState {
    /// Reload `pc`/`instrs`/`instrs_len` from the active label stack entry.
    /// Called by control-flow handlers after `current_label_idx` changes.
    ///
    /// # Safety
    /// `label_stack` and `current_label_idx` must point to a valid entry.
    #[inline]
    pub unsafe fn reload_active_label(&mut self) {
        let ls = &(*self.label_stack)[self.current_label_idx];
        self.pc = ls.ip;
        self.instrs = ls.processed_instrs.as_ptr();
        self.instrs_len = ls.processed_instrs.len();
    }

    /// Write the current `pc` back into the active label stack entry's `ip`.
    /// Called by the dispatcher driver before yielding to runtime.
    ///
    /// # Safety
    /// `label_stack` and `current_label_idx` must point to a valid entry.
    #[inline]
    pub unsafe fn writeback_pc(&mut self) {
        (*self.label_stack)[self.current_label_idx].ip = self.pc;
    }
}
