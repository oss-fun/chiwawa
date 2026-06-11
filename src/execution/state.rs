//! v2 dispatcher execution state.
//!
//! `VmState` aggregates everything a handler needs to execute one instruction:
//! register file, locals, instruction stream, label stack, memory pointer,
//! module reference, and outcome channels. Fields are raw pointers so all
//! handlers share an identical `fn(&mut VmState) -> Outcome` signature
//! (required for `return_call_indirect` type identity in TCO mode).

use crate::error::RuntimeError;
use crate::execution::func::{FuncAddr, FuncInst};
use crate::execution::ir::{Handler, ProcessedInstr};
use crate::execution::mem::MemAddr;
use crate::execution::module::ModuleInst;
use crate::execution::regs::{Reg, RegFile};
use crate::execution::value::Val;
use crate::structure::module::WasiFuncType;
use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use std::rc::{Rc, Weak};

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

    /// Counter for non-atomics-target checkpoint poll throttling.
    /// Incremented by `migration::poll_checkpoint`
    pub checkpoint_poll_counter: u32,

    /// Execution statistics (loop dispatcher only)
    #[cfg(feature = "stats")]
    pub stats: *mut crate::execution::stats::ExecutionStats,

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

    /// Shared reference to the label stack.
    #[inline(always)]
    pub fn label_stack(&self) -> &Vec<LabelStack> {
        unsafe { &*self.label_stack }
    }

    /// Mutable reference to the label stack.
    #[inline(always)]
    pub fn label_stack_mut(&mut self) -> &mut Vec<LabelStack> {
        unsafe { &mut *self.label_stack }
    }

    /// Reference to the module instance.
    #[inline(always)]
    pub fn module(&self) -> &ModuleInst {
        unsafe { &*self.module }
    }

    /// Mutable reference to the return-value register slot.
    #[inline(always)]
    pub fn return_result_regs_mut(&mut self) -> &mut ArrayVec<Reg, 8> {
        unsafe { &mut *self.return_result_regs }
    }
}

/// Module-level instructions that require runtime handling outside the DTC loop.
#[derive(Clone)]
pub enum ModuleLevelInstr {
    Return,
    InvokeWasiReg {
        wasi_func_type: WasiFuncType,
        params: Vec<Val>,
        result_reg: Option<Reg>,
    },
    InvokeReg {
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

                let primary_mem = module.upgrade().and_then(|m| m.mem_addrs.first().cloned());
                let cached_mem_ptr = primary_mem.as_ref().map(|m| m.data_ptr());

                let initial_frame = FrameStack {
                    frame: Frame {
                        module: module.clone(),
                        n: type_.results.len(),
                    },
                    label_stack: vec![LabelStack {
                        label: Label {
                            is_loop: false,
                            return_ip: 0,
                        },
                        ip: 0,
                    }],
                    processed_instrs: code.body.clone(),
                    enable_checkpoint: false,
                    result_regs: ArrayVec::new(),
                    return_result_regs: ArrayVec::new(),
                    primary_mem,
                    cached_mem_ptr,
                    handlers: code.handlers.clone(),
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
    #[serde(skip)]
    pub module: Weak<ModuleInst>,
    pub n: usize,
}

/// Activation frame stack with label stacks and execution state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameStack {
    pub frame: Frame,
    pub label_stack: Vec<LabelStack>,
    /// Function body shared by all labels in this frame (invariant per frame).
    #[serde(skip)]
    pub processed_instrs: Rc<Vec<ProcessedInstr>>,
    #[serde(skip)]
    pub enable_checkpoint: bool,
    pub result_regs: ArrayVec<Reg, 8>,
    pub return_result_regs: ArrayVec<Reg, 8>,
    #[serde(skip)]
    pub primary_mem: Option<MemAddr>,
    #[serde(skip)]
    pub cached_mem_ptr: Option<*mut u8>,
    #[serde(skip)]
    pub handlers: Rc<Vec<Handler>>,
    #[cfg(all(
        target_arch = "wasm32",
        target_os = "wasi",
        target_env = "p1",
        target_feature = "atomics"
    ))]
    #[serde(skip)]
    pub handler_ctrl: Option<Arc<HandlerControl>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Label {
    pub is_loop: bool,
    pub return_ip: usize,
}

/// Label stack entry: control label metadata and program counter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LabelStack {
    pub label: Label,
    pub ip: usize,
}
