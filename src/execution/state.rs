//! v2 dispatcher execution state.
//!
//! `VmState` aggregates everything a handler needs to execute one instruction:
//! register file, locals, instruction stream, memory pointer,
//! module reference, and outcome channels. Fields are raw pointers so all
//! handlers share an identical `fn(&mut VmState) -> Outcome` signature
//! (required for `return_call_indirect` type identity in TCO mode).

use crate::error::RuntimeError;
use crate::execution::func::{FuncAddr, FuncInst};
use crate::execution::ir::{Handler, ProcessedInstr};
use crate::execution::module::ModuleInst;
use crate::execution::regs::{Reg, RegFile};
use crate::execution::value::Val;
use crate::structure::module::{Func, WasiFuncType};
use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};

cfg_if::cfg_if! {
    if #[cfg(all(
        target_arch = "wasm32",
        target_os = "wasi",
        target_env = "p1",
        target_feature = "atomics"
    ))] {
        use crate::execution::migration::HandlerControl;
        use std::sync::Arc;
    }
}

/// Per-call dispatcher state. Constructed at the entry of each
/// `dispatch::execute_instructions` call from the active `FrameStack`.
pub struct VmState {
    // Register file (holds operand-stack registers and locals)
    pub reg_file: *mut RegFile,

    // Frame's instruction stream + cached handler array
    pub pc: usize,
    pub instrs: *const ProcessedInstr,
    pub instrs_len: usize,
    pub handlers: *const Handler,

    // Memory fast path (load/store)
    pub mem_ptr: *mut u8,

    /// Code of the running function. `instrs` and `handlers` are hoisted out
    /// of it above because they are read every instruction.
    pub code: *const Func,

    // Module (call/call_indirect/global access)
    pub module: *const ModuleInst,

    // Activation frame stack; the running frame is its last entry.
    pub frames: *mut Vec<FrameStack>,

    // Outcome channels
    pub trap: Option<RuntimeError>,
    pub yielded: Option<ModuleLevelInstr>,

    // Per-frame flags
    pub enable_checkpoint: bool,

    /// Counter for non-atomics-target checkpoint poll throttling.
    /// Incremented by `migration::poll_checkpoint`
    pub checkpoint_poll_counter: u32,

    /// Execution statistics (loop dispatcher only)
    #[cfg(feature = "stats")]
    pub stats: *mut crate::instrument::stats::ExecutionStats,

    /// Execution tracer (loop dispatcher only)
    #[cfg(feature = "trace")]
    pub tracer: *mut crate::execution::trace::Tracer,
}

impl VmState {
    /// Reference to the instruction at the current `pc`.
    #[inline(always)]
    pub fn current_instr(&self) -> &ProcessedInstr {
        unsafe { &*self.instrs.add(self.pc) }
    }

    /// Handler function pointer at the given `pc`.
    #[inline(always)]
    pub fn handler_at(&self, pc: usize) -> Handler {
        unsafe { *self.handlers.add(pc) }
    }

    /// Shared reference to the register file.
    #[inline(always)]
    pub fn reg_file(&self) -> &RegFile {
        unsafe { &*self.reg_file }
    }

    /// Mutable reference to the register file.
    #[inline(always)]
    pub fn reg_file_mut(&mut self) -> &mut RegFile {
        unsafe { &mut *self.reg_file }
    }

    /// Reference to the module instance.
    #[inline(always)]
    pub fn module(&self) -> &ModuleInst {
        unsafe { &*self.module }
    }

    /// Like `module`, but does not borrow `self`. Sound because `Runtime` holds
    /// the module for the whole run.
    #[inline(always)]
    pub fn module_static(&self) -> &'static ModuleInst {
        unsafe { &*self.module }
    }

    /// Reference to the running function's code.
    #[inline(always)]
    pub fn code(&self) -> &Func {
        unsafe { &*self.code }
    }

    /// The running frame.
    #[inline(always)]
    pub fn frame_mut(&mut self) -> &mut FrameStack {
        let frames = unsafe { &mut *self.frames };
        unsafe { frames.last_mut().unwrap_unchecked() }
    }

    /// Mutable reference to the return-value register slot.
    #[inline(always)]
    pub fn return_result_regs_mut(&mut self) -> &mut ArrayVec<Reg, 8> {
        &mut self.frame_mut().return_result_regs
    }
}

/// Module-level instructions that require runtime handling outside the DTC loop.
#[derive(Clone)]
pub enum ModuleLevelInstr {
    InvokeWasiReg {
        wasi_func_type: WasiFuncType,
        params: ArrayVec<Val, 12>,
        result_reg: Option<Reg>,
    },
    InvokeHost {
        func_addr: FuncAddr,
        params: Vec<Val>,
        result_regs: ArrayVec<Reg, 8>,
    },
}

/// VM execution state - holds all runtime state for WebAssembly execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VMState {
    pub reg_file: RegFile,
    pub activation_frame_stack: Vec<FrameStack>,
}

/// Type alias for backward compatibility.
pub type Stacks = VMState;

impl VMState {
    pub fn new(funcaddr: &FuncAddr, params: Vec<Val>) -> Result<VMState, RuntimeError> {
        let func_inst_guard = funcaddr.read_lock();
        match &*func_inst_guard {
            FuncInst::RuntimeFunc {
                type_,
                module,
                code,
                func_idx,
                ..
            } => {
                if params.len() != type_.params.len() {
                    return Err(RuntimeError::InvalidParameterCount);
                }

                // Locals live in the register file. `save_offsets` opens this
                // frame's register window and zero-initializes the declared
                // locals; the params are then scattered into their local slots.
                let mut reg_file = RegFile::new_global();
                if let Some(alloc) = code.reg_allocation.as_ref() {
                    reg_file.save_offsets(alloc);
                    reg_file.write_params(&params, &alloc.local_regs);
                }

                let cached_mem_ptr = module
                    .upgrade()
                    .and_then(|m| m.mem_addrs.first().map(|mem| mem.data_ptr()));

                let initial_frame = FrameStack {
                    func_idx: *func_idx,
                    frame: Frame {
                        n: type_.results.len(),
                    },
                    ip: 0,
                    enable_checkpoint: false,
                    result_regs: ArrayVec::new(),
                    return_result_regs: ArrayVec::new(),
                    cached_mem_ptr,
                    #[cfg(all(
                        target_arch = "wasm32",
                        target_os = "wasi",
                        target_env = "p1",
                        target_feature = "atomics"
                    ))]
                    handler_ctrl: None,
                };

                Ok(VMState {
                    reg_file,
                    activation_frame_stack: vec![initial_frame],
                })
            }
            FuncInst::HostFunc { .. } => Err(RuntimeError::UnimplementedHostFunction),
            FuncInst::WasiFunc { .. } => Err(RuntimeError::UnimplementedHostFunction),
        }
    }

    pub fn get_reg_file_and_frames(&mut self) -> (&mut RegFile, &mut Vec<FrameStack>) {
        (&mut self.reg_file, &mut self.activation_frame_stack)
    }
}

/// Call frame containing locals and module reference.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub n: usize,
}

/// Activation frame with execution state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameStack {
    pub frame: Frame,
    /// Index in the module's `func_addrs` of the function this frame runs.
    pub func_idx: u32,
    /// Program counter within this frame's body. Saved when the frame yields
    /// (call/checkpoint) and used to resume execution.
    pub ip: usize,
    #[serde(skip)]
    pub enable_checkpoint: bool,
    pub result_regs: ArrayVec<Reg, 8>,
    pub return_result_regs: ArrayVec<Reg, 8>,
    #[serde(skip)]
    pub cached_mem_ptr: Option<*mut u8>,
    #[cfg(all(
        target_arch = "wasm32",
        target_os = "wasi",
        target_env = "p1",
        target_feature = "atomics"
    ))]
    #[serde(skip)]
    pub handler_ctrl: Option<Arc<HandlerControl>>,
}
