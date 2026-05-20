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
use crate::execution::value::{Num, Ref, Val, Vec_};
use crate::structure::module::WasiFuncType;
use crate::structure::types::{NumType, ValueType, VecType};
use arrayvec::ArrayVec;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::rc::{Rc, Weak};

/// Per-call dispatcher state. Constructed at the entry of each
/// `dispatch::run` call from the active `FrameStack`.
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

    /// Counter for non-atomics-target checkpoint poll throttling.
    /// Incremented by `migration::poll_checkpoint`
    pub checkpoint_poll_counter: u32,
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
            } => {
                if params.len() != type_.params.len() {
                    return Err(RuntimeError::InvalidParameterCount);
                }

                let mut locals = params;
                for v in code.locals.iter() {
                    for _ in 0..(v.0) {
                        locals.push(match v.1 {
                            ValueType::NumType(NumType::I32) => Val::Num(Num::I32(0)),
                            ValueType::NumType(NumType::I64) => Val::Num(Num::I64(0)),
                            ValueType::NumType(NumType::F32) => Val::Num(Num::F32(0.0)),
                            ValueType::NumType(NumType::F64) => Val::Num(Num::F64(0.0)),
                            ValueType::VecType(VecType::V128) => Val::Vec_(Vec_::V128(0)),
                            ValueType::RefType(_) => Val::Ref(Ref::RefNull),
                        });
                    }
                }

                let mut reg_file = RegFile::new_global();
                if let Some(alloc) = code.reg_allocation.as_ref() {
                    reg_file.save_offsets(alloc);
                }

                let primary_mem = module.upgrade().and_then(|m| m.mem_addrs.first().cloned());
                let cached_mem_ptr = primary_mem.as_ref().map(|m| m.data_ptr());

                let initial_frame = FrameStack {
                    frame: Frame {
                        locals,
                        module: module.clone(),
                        n: type_.results.len(),
                    },
                    label_stack: vec![LabelStack {
                        label: Label {
                            is_loop: false,
                            return_ip: 0,
                        },
                        processed_instrs: code.body.clone(),
                        ip: 0,
                    }],
                    enable_checkpoint: false,
                    result_regs: ArrayVec::new(),
                    return_result_regs: ArrayVec::new(),
                    primary_mem,
                    cached_mem_ptr,
                    handlers: code.handlers.clone(),
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
    pub locals: Vec<Val>,
    #[serde(skip)]
    pub module: Weak<ModuleInst>,
    pub n: usize,
}

/// Activation frame stack with label stacks and execution state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameStack {
    pub frame: Frame,
    pub label_stack: Vec<LabelStack>,
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Label {
    pub is_loop: bool,
    pub return_ip: usize,
}

/// Label stack containing instructions and program counter.
#[derive(Clone, Debug)]
pub struct LabelStack {
    pub label: Label,
    pub processed_instrs: Rc<Vec<ProcessedInstr>>,
    pub ip: usize,
}

impl Serialize for LabelStack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("LabelStack", 2)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("ip", &self.ip)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for LabelStack {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct LabelStackData {
            label: Label,
            ip: usize,
        }
        let data = LabelStackData::deserialize(deserializer)?;
        Ok(LabelStack {
            label: data.label,
            processed_instrs: Rc::new(Vec::new()),
            ip: data.ip,
        })
    }
}
