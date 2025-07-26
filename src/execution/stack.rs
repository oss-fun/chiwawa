use super::value::*;
use crate::error::RuntimeError;
use crate::execution::{func::*, module::*};
use crate::structure::types::LabelIdx as StructureLabelIdx;
use crate::structure::{instructions::*, types::*};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::arch::asm;
use std::fs;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Sub};
use std::path::Path;
use std::sync::Weak as SyncWeak;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Operand {
    None,
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    LocalIdx(LocalIdx),
    GlobalIdx(GlobalIdx),
    FuncIdx(FuncIdx),
    TableIdx(TableIdx),
    TypeIdx(TypeIdx),
    RefType(RefType),
    LabelIdx {
        target_ip: usize,
        arity: usize,
        target_label_stack_idx: usize,
        original_wasm_depth: usize,
        is_loop: bool,
    },
    MemArg(Memarg),
    BrTable {
        targets: Vec<Operand>,
        default: Box<Operand>,
    },
    CallIndirect {
        type_idx: TypeIdx,
        table_idx: TableIdx,
    },
    Block {
        arity: usize,
        param_count: usize,
        is_loop: bool,
        start_ip: usize,
        end_ip: usize,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessedInstr {
    pub handler_index: usize,
    pub operand: Operand,
}

pub struct ExecutionContext<'a> {
    pub frame: &'a mut crate::execution::stack::Frame,
    pub value_stack: &'a mut Vec<Val>,
    pub ip: usize,
}

#[derive(Clone, Debug)]
enum HandlerResult {
    Continue(usize),
    Return,
    Invoke(FuncAddr),
    Branch {
        target_ip: usize,
        target_label_stack_idx: usize,
        values_to_push: Vec<Val>,
        branch_depth: usize,
    },
    PushLabelStack {
        label: Label,
        next_ip: usize,
        start_ip: usize,
        end_ip: usize,
    },
    PopLabelStack {
        next_ip: usize,
    },
}

type HandlerFn = fn(&mut ExecutionContext, &Operand) -> Result<HandlerResult, RuntimeError>;

#[derive(Clone)]
pub enum ModuleLevelInstr {
    Invoke(FuncAddr),
    Return,
}

pub const HANDLER_IDX_UNREACHABLE: usize = 0x00;
pub const HANDLER_IDX_NOP: usize = 0x01;
pub const HANDLER_IDX_BLOCK: usize = 0x02;
pub const HANDLER_IDX_LOOP: usize = 0x03;
pub const HANDLER_IDX_IF: usize = 0x04;
pub const HANDLER_IDX_ELSE: usize = 0x05;
pub const HANDLER_IDX_END: usize = 0x0B;
pub const HANDLER_IDX_BR: usize = 0x0C;
pub const HANDLER_IDX_BR_IF: usize = 0x0D;
pub const HANDLER_IDX_BR_TABLE: usize = 0x0E;
pub const HANDLER_IDX_RETURN: usize = 0x0F;
pub const HANDLER_IDX_CALL: usize = 0x10;
pub const HANDLER_IDX_CALL_INDIRECT: usize = 0x11;

pub const HANDLER_IDX_DROP: usize = 0x1A;
pub const HANDLER_IDX_SELECT: usize = 0x1B;
// pub const HANDLER_IDX_SELECT_T: usize = 0x1C;

pub const HANDLER_IDX_LOCAL_GET: usize = 0x20;
pub const HANDLER_IDX_LOCAL_SET: usize = 0x21;
pub const HANDLER_IDX_LOCAL_TEE: usize = 0x22;
pub const HANDLER_IDX_GLOBAL_GET: usize = 0x23;
pub const HANDLER_IDX_GLOBAL_SET: usize = 0x24;
// pub const HANDLER_IDX_TABLE_GET: usize = 0x25;
// pub const HANDLER_IDX_TABLE_SET: usize = 0x26;

pub const HANDLER_IDX_I32_LOAD: usize = 0x28;
pub const HANDLER_IDX_I64_LOAD: usize = 0x29;
pub const HANDLER_IDX_F32_LOAD: usize = 0x2A;
pub const HANDLER_IDX_F64_LOAD: usize = 0x2B;
pub const HANDLER_IDX_I32_LOAD8_S: usize = 0x2C;
pub const HANDLER_IDX_I32_LOAD8_U: usize = 0x2D;
pub const HANDLER_IDX_I32_LOAD16_S: usize = 0x2E;
pub const HANDLER_IDX_I32_LOAD16_U: usize = 0x2F;
pub const HANDLER_IDX_I64_LOAD8_S: usize = 0x30;
pub const HANDLER_IDX_I64_LOAD8_U: usize = 0x31;
pub const HANDLER_IDX_I64_LOAD16_S: usize = 0x32;
pub const HANDLER_IDX_I64_LOAD16_U: usize = 0x33;
pub const HANDLER_IDX_I64_LOAD32_S: usize = 0x34;
pub const HANDLER_IDX_I64_LOAD32_U: usize = 0x35;
pub const HANDLER_IDX_I32_STORE: usize = 0x36;
pub const HANDLER_IDX_I64_STORE: usize = 0x37;
pub const HANDLER_IDX_F32_STORE: usize = 0x38;
pub const HANDLER_IDX_F64_STORE: usize = 0x39;
pub const HANDLER_IDX_I32_STORE8: usize = 0x3A;
pub const HANDLER_IDX_I32_STORE16: usize = 0x3B;
pub const HANDLER_IDX_I64_STORE8: usize = 0x3C;
pub const HANDLER_IDX_I64_STORE16: usize = 0x3D;
pub const HANDLER_IDX_I64_STORE32: usize = 0x3E;
pub const HANDLER_IDX_MEMORY_SIZE: usize = 0x3F;
pub const HANDLER_IDX_MEMORY_GROW: usize = 0x40;
pub const HANDLER_IDX_MEMORY_COPY: usize = 0xC5;
pub const HANDLER_IDX_MEMORY_INIT: usize = 0xC6;
pub const HANDLER_IDX_MEMORY_FILL: usize = 0xC7;
pub const HANDLER_IDX_REF_NULL: usize = 0xD0;
pub const HANDLER_IDX_REF_IS_NULL: usize = 0xD1;
pub const HANDLER_IDX_TABLE_GET: usize = 0xD2;
pub const HANDLER_IDX_TABLE_SET: usize = 0xD3;
pub const HANDLER_IDX_TABLE_FILL: usize = 0xD4;

pub const HANDLER_IDX_I32_CONST: usize = 0x41;
pub const HANDLER_IDX_I64_CONST: usize = 0x42;
pub const HANDLER_IDX_F32_CONST: usize = 0x43;
pub const HANDLER_IDX_F64_CONST: usize = 0x44;

pub const HANDLER_IDX_I32_EQZ: usize = 0x45;
pub const HANDLER_IDX_I32_EQ: usize = 0x46;
pub const HANDLER_IDX_I32_NE: usize = 0x47;
pub const HANDLER_IDX_I32_LT_S: usize = 0x48;
pub const HANDLER_IDX_I32_LT_U: usize = 0x49;
pub const HANDLER_IDX_I32_GT_S: usize = 0x4A;
pub const HANDLER_IDX_I32_GT_U: usize = 0x4B;
pub const HANDLER_IDX_I32_LE_S: usize = 0x4C;
pub const HANDLER_IDX_I32_LE_U: usize = 0x4D;
pub const HANDLER_IDX_I32_GE_S: usize = 0x4E;
pub const HANDLER_IDX_I32_GE_U: usize = 0x4F;

pub const HANDLER_IDX_I64_EQZ: usize = 0x50;
pub const HANDLER_IDX_I64_EQ: usize = 0x51;
pub const HANDLER_IDX_I64_NE: usize = 0x52;
pub const HANDLER_IDX_I64_LT_S: usize = 0x53;
pub const HANDLER_IDX_I64_LT_U: usize = 0x54;
pub const HANDLER_IDX_I64_GT_S: usize = 0x55;
pub const HANDLER_IDX_I64_GT_U: usize = 0x56;
pub const HANDLER_IDX_I64_LE_S: usize = 0x57;
pub const HANDLER_IDX_I64_LE_U: usize = 0x58;
pub const HANDLER_IDX_I64_GE_S: usize = 0x59;
pub const HANDLER_IDX_I64_GE_U: usize = 0x5A;

pub const HANDLER_IDX_F32_EQ: usize = 0x5B;
pub const HANDLER_IDX_F32_NE: usize = 0x5C;
pub const HANDLER_IDX_F32_LT: usize = 0x5D;
pub const HANDLER_IDX_F32_GT: usize = 0x5E;
pub const HANDLER_IDX_F32_LE: usize = 0x5F;
pub const HANDLER_IDX_F32_GE: usize = 0x60;

pub const HANDLER_IDX_F64_EQ: usize = 0x61;
pub const HANDLER_IDX_F64_NE: usize = 0x62;
pub const HANDLER_IDX_F64_LT: usize = 0x63;
pub const HANDLER_IDX_F64_GT: usize = 0x64;
pub const HANDLER_IDX_F64_LE: usize = 0x65;
pub const HANDLER_IDX_F64_GE: usize = 0x66;

pub const HANDLER_IDX_I32_CLZ: usize = 0x67;
pub const HANDLER_IDX_I32_CTZ: usize = 0x68;
pub const HANDLER_IDX_I32_POPCNT: usize = 0x69;
pub const HANDLER_IDX_I32_ADD: usize = 0x6A;
pub const HANDLER_IDX_I32_SUB: usize = 0x6B;
pub const HANDLER_IDX_I32_MUL: usize = 0x6C;
pub const HANDLER_IDX_I32_DIV_S: usize = 0x6D;
pub const HANDLER_IDX_I32_DIV_U: usize = 0x6E;
pub const HANDLER_IDX_I32_REM_S: usize = 0x6F;
pub const HANDLER_IDX_I32_REM_U: usize = 0x70;
pub const HANDLER_IDX_I32_AND: usize = 0x71;
pub const HANDLER_IDX_I32_OR: usize = 0x72;
pub const HANDLER_IDX_I32_XOR: usize = 0x73;
pub const HANDLER_IDX_I32_SHL: usize = 0x74;
pub const HANDLER_IDX_I32_SHR_S: usize = 0x75;
pub const HANDLER_IDX_I32_SHR_U: usize = 0x76;
pub const HANDLER_IDX_I32_ROTL: usize = 0x77;
pub const HANDLER_IDX_I32_ROTR: usize = 0x78;

pub const HANDLER_IDX_I64_CLZ: usize = 0x79;
pub const HANDLER_IDX_I64_CTZ: usize = 0x7A;
pub const HANDLER_IDX_I64_POPCNT: usize = 0x7B;
pub const HANDLER_IDX_I64_ADD: usize = 0x7C;
pub const HANDLER_IDX_I64_SUB: usize = 0x7D;
pub const HANDLER_IDX_I64_MUL: usize = 0x7E;
pub const HANDLER_IDX_I64_DIV_S: usize = 0x7F;
pub const HANDLER_IDX_I64_DIV_U: usize = 0x80;
pub const HANDLER_IDX_I64_REM_S: usize = 0x81;
pub const HANDLER_IDX_I64_REM_U: usize = 0x82;
pub const HANDLER_IDX_I64_AND: usize = 0x83;
pub const HANDLER_IDX_I64_OR: usize = 0x84;
pub const HANDLER_IDX_I64_XOR: usize = 0x85;
pub const HANDLER_IDX_I64_SHL: usize = 0x86;
pub const HANDLER_IDX_I64_SHR_S: usize = 0x87;
pub const HANDLER_IDX_I64_SHR_U: usize = 0x88;
pub const HANDLER_IDX_I64_ROTL: usize = 0x89;
pub const HANDLER_IDX_I64_ROTR: usize = 0x8A;

pub const HANDLER_IDX_F32_ABS: usize = 0x8B;
pub const HANDLER_IDX_F32_NEG: usize = 0x8C;
pub const HANDLER_IDX_F32_CEIL: usize = 0x8D;
pub const HANDLER_IDX_F32_FLOOR: usize = 0x8E;
pub const HANDLER_IDX_F32_TRUNC: usize = 0x8F;
pub const HANDLER_IDX_F32_NEAREST: usize = 0x90;
pub const HANDLER_IDX_F32_SQRT: usize = 0x91;
pub const HANDLER_IDX_F32_ADD: usize = 0x92;
pub const HANDLER_IDX_F32_SUB: usize = 0x93;
pub const HANDLER_IDX_F32_MUL: usize = 0x94;
pub const HANDLER_IDX_F32_DIV: usize = 0x95;
pub const HANDLER_IDX_F32_MIN: usize = 0x96;
pub const HANDLER_IDX_F32_MAX: usize = 0x97;
pub const HANDLER_IDX_F32_COPYSIGN: usize = 0x98;

pub const HANDLER_IDX_F64_ABS: usize = 0x99;
pub const HANDLER_IDX_F64_NEG: usize = 0x9A;
pub const HANDLER_IDX_F64_CEIL: usize = 0x9B;
pub const HANDLER_IDX_F64_FLOOR: usize = 0x9C;
pub const HANDLER_IDX_F64_TRUNC: usize = 0x9D;
pub const HANDLER_IDX_F64_NEAREST: usize = 0x9E;
pub const HANDLER_IDX_F64_SQRT: usize = 0x9F;
pub const HANDLER_IDX_F64_ADD: usize = 0xA0;
pub const HANDLER_IDX_F64_SUB: usize = 0xA1;
pub const HANDLER_IDX_F64_MUL: usize = 0xA2;
pub const HANDLER_IDX_F64_DIV: usize = 0xA3;
pub const HANDLER_IDX_F64_MIN: usize = 0xA4;
pub const HANDLER_IDX_F64_MAX: usize = 0xA5;
pub const HANDLER_IDX_F64_COPYSIGN: usize = 0xA6;

pub const HANDLER_IDX_I32_WRAP_I64: usize = 0xA7;
pub const HANDLER_IDX_I32_TRUNC_F32_S: usize = 0xA8;
pub const HANDLER_IDX_I32_TRUNC_F32_U: usize = 0xA9;
pub const HANDLER_IDX_I32_TRUNC_F64_S: usize = 0xAA;
pub const HANDLER_IDX_I32_TRUNC_F64_U: usize = 0xAB;
pub const HANDLER_IDX_I64_EXTEND_I32_S: usize = 0xAC;
pub const HANDLER_IDX_I64_EXTEND_I32_U: usize = 0xAD;
pub const HANDLER_IDX_I64_TRUNC_F32_S: usize = 0xAE;
pub const HANDLER_IDX_I64_TRUNC_F32_U: usize = 0xAF;
pub const HANDLER_IDX_I64_TRUNC_F64_S: usize = 0xB0;
pub const HANDLER_IDX_I64_TRUNC_F64_U: usize = 0xB1;
pub const HANDLER_IDX_F32_CONVERT_I32_S: usize = 0xB2;
pub const HANDLER_IDX_F32_CONVERT_I32_U: usize = 0xB3;
pub const HANDLER_IDX_F32_CONVERT_I64_S: usize = 0xB4;
pub const HANDLER_IDX_F32_CONVERT_I64_U: usize = 0xB5;
pub const HANDLER_IDX_F32_DEMOTE_F64: usize = 0xB6;
pub const HANDLER_IDX_F64_CONVERT_I32_S: usize = 0xB7;
pub const HANDLER_IDX_F64_CONVERT_I32_U: usize = 0xB8;
pub const HANDLER_IDX_F64_CONVERT_I64_S: usize = 0xB9;
pub const HANDLER_IDX_F64_CONVERT_I64_U: usize = 0xBA;
pub const HANDLER_IDX_F64_PROMOTE_F32: usize = 0xBB;
pub const HANDLER_IDX_I32_REINTERPRET_F32: usize = 0xBC;
pub const HANDLER_IDX_I64_REINTERPRET_F64: usize = 0xBD;
pub const HANDLER_IDX_F32_REINTERPRET_I32: usize = 0xBE;
pub const HANDLER_IDX_F64_REINTERPRET_I64: usize = 0xBF;

pub const HANDLER_IDX_I32_EXTEND8_S: usize = 0xC0;
pub const HANDLER_IDX_I32_EXTEND16_S: usize = 0xC1;
pub const HANDLER_IDX_I64_EXTEND8_S: usize = 0xC2;
pub const HANDLER_IDX_I64_EXTEND16_S: usize = 0xC3;
pub const HANDLER_IDX_I64_EXTEND32_S: usize = 0xC4;

// TruncSat instructions
pub const HANDLER_IDX_I32_TRUNC_SAT_F32_S: usize = 0xC8;
pub const HANDLER_IDX_I32_TRUNC_SAT_F32_U: usize = 0xC9;
pub const HANDLER_IDX_I32_TRUNC_SAT_F64_S: usize = 0xCA;
pub const HANDLER_IDX_I32_TRUNC_SAT_F64_U: usize = 0xCB;
pub const HANDLER_IDX_I64_TRUNC_SAT_F32_S: usize = 0xCC;
pub const HANDLER_IDX_I64_TRUNC_SAT_F32_U: usize = 0xCD;
pub const HANDLER_IDX_I64_TRUNC_SAT_F64_S: usize = 0xCE;
pub const HANDLER_IDX_I64_TRUNC_SAT_F64_U: usize = 0xCF;

// TODO: Add remaining indices (Ref types, Table, Bulk Memory, SIMD)

pub const MAX_HANDLER_INDEX: usize = 0xD5;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stacks {
    pub activation_frame_stack: Vec<FrameStack>,
}

impl Stacks {
    pub fn new(funcaddr: &FuncAddr, params: Vec<Val>) -> Result<Stacks, RuntimeError> {
        let func_inst_guard = funcaddr.read_lock().expect("RwLock poisoned");
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

                let initial_frame = FrameStack {
                    frame: Frame {
                        locals,
                        module: module.clone(),
                        n: type_.results.len(),
                    },
                    label_stack: vec![LabelStack {
                        label: Label {
                            locals_num: type_.results.len(),
                            arity: type_.results.len(),
                            is_loop: false,
                            stack_height: 0, // Function level starts with empty stack
                            return_ip: 0,    // No return needed for function level
                        },
                        processed_instrs: code.body.clone(),
                        value_stack: vec![],
                        ip: 0,
                    }],
                    void: type_.results.is_empty(),
                    instruction_count: 0,
                    global_value_stack: vec![],
                };

                Ok(Stacks {
                    activation_frame_stack: vec![initial_frame],
                })
            }
            FuncInst::HostFunc { .. } => {
                // TODO: Handle host function invocation setup if needed,
                // or rely on FuncAddr::call to handle it directly.
                Err(RuntimeError::UnimplementedHostFunction)
            }
            FuncInst::WasiFunc { .. } => {
                // WASI functions are handled directly in Runtime, not through Stacks
                Err(RuntimeError::UnimplementedHostFunction)
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub locals: Vec<Val>,
    #[serde(skip)]
    pub module: SyncWeak<ModuleInst>,
    pub n: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameStack {
    pub frame: Frame,
    pub label_stack: Vec<LabelStack>,
    pub void: bool,
    #[serde(default)]
    pub instruction_count: u64,
    // Global value stack shared across all label stacks
    pub global_value_stack: Vec<Val>,
}

impl FrameStack {
    /// Push a new label stack for a block or loop
    pub fn push_label_stack(&mut self, label: Label, instructions: Vec<ProcessedInstr>) {
        let new_label_stack = LabelStack {
            label,
            processed_instrs: instructions,
            value_stack: vec![],
            ip: 0,
        };
        self.label_stack.push(new_label_stack);
    }

    /// Pop the top label stack when exiting a block
    pub fn pop_label_stack(&mut self) -> Option<LabelStack> {
        if self.label_stack.len() > 1 {
            self.label_stack.pop()
        } else {
            None
        }
    }

    pub fn current_stack_height(&self) -> usize {
        if let Some(current_label) = self.label_stack.last() {
            current_label.value_stack.len()
        } else {
            0
        }
    }

    pub fn run_dtc_loop(
        &mut self,
        _called_func_addr_out: &mut Option<FuncAddr>,
    ) -> Result<Result<Option<ModuleLevelInstr>, RuntimeError>, RuntimeError> {
        let mut current_label_stack_idx = self
            .label_stack
            .len()
            .checked_sub(1)
            .ok_or(RuntimeError::StackError("Initial label stack empty"))?;

        const CHECKPOINT_TRIGGER_FILE: &str = "./checkpoint.trigger";
        const CHECKPOINT_CHECK_INTERVAL: u64 = 100;

        loop {
            self.instruction_count += 1;
            if self.instruction_count % CHECKPOINT_CHECK_INTERVAL == 0 {
                let trigger_path = Path::new(CHECKPOINT_TRIGGER_FILE);
                if trigger_path.exists() {
                    println!(
                        "Checkpoint trigger file found after {} instructions! Requesting checkpoint...",
                        self.instruction_count
                    );
                    let _ = fs::remove_file(trigger_path);
                    return Ok(Err(RuntimeError::CheckpointRequested));
                }
            }

            if current_label_stack_idx >= self.label_stack.len() {
                break;
            }

            let current_label_stack = &mut self.label_stack[current_label_stack_idx];
            let processed_code = current_label_stack.processed_instrs.clone();
            let ip = current_label_stack.ip;

            if ip >= processed_code.len() {
                current_label_stack.ip = ip;

                // Label stack index indicates nesting level:
                // Index 0: Function level (function body)
                // Index 1+: Block/loop/if levels (nested blocks)
                if current_label_stack_idx > 0 {
                    let return_ip = self.label_stack[current_label_stack_idx].label.return_ip;
                    let current_label = &self.label_stack[current_label_stack_idx].label;
                    let is_loop = current_label.is_loop;

                    // Check if returning to parent would cause re-execution of already processed code
                    // This handles cases where break jumps out of loops and shouldn't re-enter them
                    // Exception: For loops, it's normal for return_ip to be smaller than current_ip
                    if return_ip <= ip && !is_loop {
                        // This would cause re-execution - end the function instead
                        break;
                    }

                    self.label_stack.pop();
                    current_label_stack_idx -= 1;

                    // Continue execution at return point in parent level
                    if current_label_stack_idx < self.label_stack.len() {
                        let parent_label_stack = &mut self.label_stack[current_label_stack_idx];
                        parent_label_stack.ip = return_ip;
                    }
                    continue;
                } else {
                    // Function level (index 0): end of function execution
                    break;
                }
            }

            let instruction_ref = &processed_code[ip];

            let handler_fn = HANDLER_TABLE
                .get(instruction_ref.handler_index)
                .ok_or(RuntimeError::InvalidHandlerIndex)?;

            let mut context = ExecutionContext {
                frame: &mut self.frame,
                value_stack: &mut self.global_value_stack,
                ip,
            };

            let result = handler_fn(&mut context, &instruction_ref.operand);

            match result {
                Err(e) => {
                    eprintln!(
                        "Error at IP {}, handler_index: {}: {:?}",
                        ip, instruction_ref.handler_index, e
                    );
                    return Ok(Err(e));
                }
                Ok(handler_result) => {
                    match handler_result {
                        HandlerResult::Continue(next_ip) => {
                            self.label_stack[current_label_stack_idx].ip = next_ip;
                        }
                        HandlerResult::Return => {
                            return Ok(Ok(Some(ModuleLevelInstr::Return)));
                        }
                        HandlerResult::Invoke(func_addr) => {
                            self.label_stack[current_label_stack_idx].ip = ip + 1;
                            return Ok(Ok(Some(ModuleLevelInstr::Invoke(func_addr))));
                        }
                        HandlerResult::Branch {
                            target_ip,
                            target_label_stack_idx,
                            values_to_push,
                            branch_depth,
                        } => {
                            // Use the branch depth directly from the Branch result
                            // Calculate target label stack index from current position and branch depth
                            // Branch depth 0 = current block, 1 = parent block, etc.
                            if branch_depth <= current_label_stack_idx {
                                let target_depth = current_label_stack_idx - branch_depth;
                                let target_level = target_depth + 1;

                                let current_label_stack =
                                    &self.label_stack[current_label_stack_idx];
                                let current_stack_height = current_label_stack.label.stack_height;

                                // For branch depth 0, we need to exit the current block
                                if branch_depth == 0 {
                                    self.label_stack.pop();
                                    if self.label_stack.len() > 0 {
                                        current_label_stack_idx = self.label_stack.len() - 1;
                                    } else {
                                        return Err(RuntimeError::StackError(
                                            "Label stack underflow during branch",
                                        ));
                                    }
                                } else {
                                    // For deeper branches, truncate to the target level
                                    self.label_stack.truncate(target_level);
                                    if self.label_stack.len() > 0 {
                                        current_label_stack_idx = self.label_stack.len() - 1;
                                    } else {
                                        return Err(RuntimeError::StackError(
                                            "Label stack underflow during branch",
                                        ));
                                    }
                                }

                                let target_label_stack =
                                    &mut self.label_stack[current_label_stack_idx];

                                let stack_height = if branch_depth == 0 {
                                    current_stack_height
                                } else {
                                    target_label_stack.label.stack_height
                                };

                                self.global_value_stack.truncate(stack_height);
                                self.global_value_stack.extend(values_to_push);
                                target_label_stack.ip = target_ip;
                            } else {
                                return Err(RuntimeError::InvalidBranchTarget);
                            }
                            continue;
                        }
                        HandlerResult::PushLabelStack {
                            label,
                            next_ip,
                            start_ip,
                            end_ip,
                        } => {
                            let current_instrs =
                                &self.label_stack[current_label_stack_idx].processed_instrs;

                            let new_label_stack = LabelStack {
                                label,
                                processed_instrs: current_instrs.clone(),
                                value_stack: vec![],
                                ip: next_ip,
                            };

                            self.label_stack.push(new_label_stack);
                            current_label_stack_idx = self.label_stack.len() - 1;
                        }
                        HandlerResult::PopLabelStack { next_ip } => {
                            // Pop the current label stack when ending a block/loop
                            if self.label_stack.len() > 1 {
                                let current_label =
                                    &self.label_stack[current_label_stack_idx].label;
                                let return_ip = current_label.return_ip;
                                let stack_height = current_label.stack_height;
                                let arity = current_label.arity;

                                // Extract the result values from the global stack
                                let result_values = if arity > 0 {
                                    if self.global_value_stack.len() >= arity {
                                        let start_idx = self.global_value_stack.len() - arity;
                                        let values = self.global_value_stack[start_idx..].to_vec();
                                        values
                                    } else {
                                        // Not enough values on stack for the required arity
                                        if self.global_value_stack.len() > stack_height {
                                            // Use all available values that were added after this block started
                                            self.global_value_stack[stack_height..].to_vec()
                                        } else {
                                            // No new values available
                                            Vec::new()
                                        }
                                    }
                                } else {
                                    Vec::new()
                                };

                                // Restore the global stack to the entry state
                                if arity == 0 && result_values.is_empty() {
                                    // For void blocks (arity=0) with no results, preserve the current stack state
                                } else {
                                    // Normal case: restore to entry state and add result values
                                    let correct_stack_height =
                                        if arity > 0 && self.global_value_stack.len() >= arity {
                                            self.global_value_stack.len() - arity
                                        } else {
                                            stack_height
                                        };

                                    self.global_value_stack.truncate(correct_stack_height);
                                    self.global_value_stack.extend(result_values);
                                }

                                self.label_stack.pop();
                                current_label_stack_idx = self.label_stack.len() - 1;
                                self.label_stack[current_label_stack_idx].ip = next_ip;
                            } else {
                                let current_label =
                                    &self.label_stack[current_label_stack_idx].label;
                                let function_arity = current_label.arity;

                                // Extract the function result values from the global stack
                                let result_values = if function_arity > 0 {
                                    if self.global_value_stack.len() >= function_arity {
                                        let start_idx =
                                            self.global_value_stack.len() - function_arity;
                                        let values = self.global_value_stack[start_idx..].to_vec();
                                        values
                                    } else {
                                        return Err(RuntimeError::ValueStackUnderflow);
                                    }
                                } else {
                                    self.global_value_stack.clone()
                                };

                                self.global_value_stack.clear();
                                self.global_value_stack.extend(result_values);
                                break;
                            }
                        }
                    }
                }
            }
        }
        Ok(Ok(None))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Label {
    pub locals_num: usize,
    pub arity: usize,
    pub is_loop: bool,
    pub stack_height: usize,
    pub return_ip: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LabelStack {
    pub label: Label,
    pub processed_instrs: Vec<ProcessedInstr>,
    pub value_stack: Vec<Val>,
    pub ip: usize,
}

#[derive(Clone, Debug)]
pub enum FrameLevelInstr {
    Label(Label, Vec<Instr>),
    Br(StructureLabelIdx),
    EndLabel,
    Invoke(FuncAddr),
    Return,
}

#[derive(Clone, Debug)]
pub enum AdminInstr {
    Trap,
    Instr(Instr),
    Invoke(FuncAddr),
    Label(Label, Vec<Instr>),
    Br(StructureLabelIdx),
    Return,
    Ref(FuncAddr),
    RefExtern(ExternAddr),
}

macro_rules! binop {
    ($ctx:ident, $operand_type:ident, $op_trait:ident, $op_method:ident, $result_type:ident) => {{
        if $ctx.value_stack.len() < 2 {
            return Err(RuntimeError::ValueStackUnderflow);
        }
        let rhs_val = $ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let lhs_val = $ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let (lhs, rhs) = match (lhs_val, rhs_val) {
            (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l, r),
            _ => return Err(RuntimeError::TypeMismatch),
        };
        $ctx.value_stack
            .push(Val::Num(Num::$result_type($op_trait::$op_method(lhs, rhs))));
        Ok(HandlerResult::Continue($ctx.ip + 1))
    }};
    ($ctx:ident, $operand_type:ident, $op_trait:ident, $op_method:ident) => {
        binop!($ctx, $operand_type, $op_trait, $op_method, $operand_type)
    };
}

macro_rules! binop_wrapping {
    ($ctx:ident, $operand_type:ident, $op_method:ident, $result_type:ident) => {{
        if $ctx.value_stack.len() < 2 {
            return Err(RuntimeError::ValueStackUnderflow);
        }
        let rhs_val = $ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let lhs_val = $ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let (lhs, rhs) = match (lhs_val, rhs_val) {
            (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l, r),
            _ => return Err(RuntimeError::TypeMismatch),
        };
        let result_val = Val::Num(Num::$result_type(lhs.$op_method(rhs)));
        $ctx.value_stack.push(result_val);
        Ok(HandlerResult::Continue($ctx.ip + 1))
    }};
    ($ctx:ident, $operand_type:ident, $op_method:ident) => {
        binop_wrapping!($ctx, $operand_type, $op_method, $operand_type)
    };
}

macro_rules! cmpop {
    ($ctx:ident, $operand_type:ident, $op:tt) => {
        {
            let rhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let (lhs, rhs) = match (lhs_val, rhs_val) {
                 (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l, r),
                 _ => return Err(RuntimeError::TypeMismatch),
            };
            $ctx.value_stack.push(Val::Num(Num::I32((lhs $op rhs) as i32)));
             Ok(HandlerResult::Continue($ctx.ip + 1))
        }
    };
     ($ctx:ident, $operand_type:ident, $op:tt, $cast_type:ty) => {
        {
            let rhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let (lhs, rhs) = match (lhs_val, rhs_val) {
                 (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l as $cast_type, r as $cast_type),
                 _ => return Err(RuntimeError::TypeMismatch),
            };
            $ctx.value_stack.push(Val::Num(Num::I32((lhs $op rhs) as i32)));
             Ok(HandlerResult::Continue($ctx.ip + 1))
        }
    };
}

fn handle_unreachable(
    _ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    Err(RuntimeError::Unreachable)
}

fn handle_nop(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_block(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::Block {
        arity,
        param_count,
        is_loop,
        start_ip,
        end_ip,
    } = operand
    {
        // Create a new label for this block
        let current_stack_height = ctx.value_stack.len();
        let label = Label {
            locals_num: *param_count,
            arity: *arity,
            is_loop: *is_loop,
            stack_height: current_stack_height,
            return_ip: *end_ip + 1, // IP to return to after this block (after the end instruction)
        };

        Ok(HandlerResult::PushLabelStack {
            label,
            next_ip: ctx.ip + 1,
            start_ip: *start_ip,
            end_ip: *end_ip,
        })
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_loop(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::Block {
        arity,
        param_count,
        is_loop,
        start_ip,
        end_ip,
    } = operand
    {
        let current_stack_height = ctx.value_stack.len();
        let label = Label {
            locals_num: *param_count,
            arity: *arity,
            is_loop: *is_loop,
            stack_height: current_stack_height,
            return_ip: *end_ip + 1, // IP to return to after this loop (after the end instruction)
        };

        // Signal to the main execution loop to create a new label stack
        Ok(HandlerResult::PushLabelStack {
            label,
            next_ip: ctx.ip + 1,
            start_ip: *start_ip,
            end_ip: *end_ip,
        })
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_if(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let cond_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let cond = cond_val.to_i32()?;

    if let &Operand::LabelIdx {
        target_ip,
        arity,
        target_label_stack_idx: _,
        original_wasm_depth: _,
        is_loop: _,
    } = operand
    {
        if target_ip == usize::MAX {
            return Err(RuntimeError::ExecutionFailed(
                "Branch fixup not done for If",
            ));
        }

        let current_stack_height = ctx.value_stack.len();
        let label = Label {
            locals_num: 0, // If blocks don't have parameters
            arity,
            is_loop: false,
            stack_height: current_stack_height,
            return_ip: target_ip, // IP after the if block (either else or end)
        };

        // WebAssembly if: 0 = false (else/skip), non-zero = true (then)
        let next_ip = if cond != 0 { ctx.ip + 1 } else { target_ip };

        Ok(HandlerResult::PushLabelStack {
            label,
            next_ip,
            start_ip: ctx.ip + 1,
            end_ip: target_ip,
        })
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_else(
    _ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::LabelIdx {
        target_ip,
        arity: _,
        target_label_stack_idx: _,
        original_wasm_depth: _,
        is_loop: _,
    } = operand
    {
        if target_ip == usize::MAX {
            return Err(RuntimeError::ExecutionFailed(
                "Branch fixup not done for Else",
            ));
        }
        Ok(HandlerResult::Continue(target_ip))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_end(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    Ok(HandlerResult::PopLabelStack {
        next_ip: ctx.ip + 1,
    })
}

fn handle_br(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::LabelIdx {
        target_ip,
        arity,
        target_label_stack_idx,
        original_wasm_depth,
        is_loop: _,
    } = operand
    {
        if *target_ip == usize::MAX {
            return Err(RuntimeError::ExecutionFailed(
                "Branch fixup not done for Br",
            ));
        }

        let values_to_push = ctx.pop_n_values(*arity)?;

        Ok(HandlerResult::Branch {
            target_ip: *target_ip,
            target_label_stack_idx: *target_label_stack_idx,
            values_to_push,
            branch_depth: *original_wasm_depth,
        })
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_br_if(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let cond_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let cond = cond_val.to_i32()?;

    if cond != 0 {
        if let Operand::LabelIdx {
            target_ip,
            arity,
            target_label_stack_idx,
            original_wasm_depth,
            is_loop: _,
        } = operand
        {
            if *target_ip == usize::MAX {
                return Err(RuntimeError::ExecutionFailed(
                    "Branch fixup not done for BrIf",
                ));
            }

            let values_to_push = ctx.pop_n_values(*arity)?;

            Ok(HandlerResult::Branch {
                target_ip: *target_ip,
                target_label_stack_idx: *target_label_stack_idx,
                values_to_push,
                branch_depth: *original_wasm_depth,
            })
        } else {
            Err(RuntimeError::InvalidOperand)
        }
    } else {
        Ok(HandlerResult::Continue(ctx.ip + 1))
    }
}

fn handle_call(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::FuncIdx(func_idx) = operand {
        let instance = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        let func_addr = instance.func_addrs.get_by_idx(func_idx.clone()).clone();
        Ok(HandlerResult::Invoke(func_addr))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_const(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    match operand {
        Operand::I32(val) => {
            ctx.value_stack.push(Val::Num(Num::I32(*val)));
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
        _ => Err(RuntimeError::InvalidOperand),
    }
}

fn handle_i64_const(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::I64(val) = operand {
        ctx.value_stack.push(Val::Num(Num::I64(val)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f32_const(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::F32(val) = operand {
        ctx.value_stack.push(Val::Num(Num::F32(val)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f64_const(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::F64(val) = operand {
        ctx.value_stack.push(Val::Num(Num::F64(val)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

// --- Comparison Handlers (Operand not used, signature change only) ---
fn handle_i32_eqz(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i32()?;
    ctx.value_stack.push(Val::Num(Num::I32((val == 0) as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_eq(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I32, ==)
}
fn handle_i32_ne(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I32, !=)
}
fn handle_i32_lt_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I32, <)
}
fn handle_i32_lt_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I32, <, u32)
}
fn handle_i32_gt_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I32, >)
}
fn handle_i32_gt_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I32, >, u32)
}
fn handle_i32_le_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I32, <=)
}
fn handle_i32_le_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I32, <=, u32)
}
fn handle_i32_ge_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I32, >=)
}
fn handle_i32_ge_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I32, >=, u32)
}

fn handle_i64_eqz(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i64()?;
    ctx.value_stack.push(Val::Num(Num::I32((val == 0) as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_eq(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I64, ==)
}
fn handle_i64_ne(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I64, !=)
}
fn handle_i64_lt_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I64, <)
}
fn handle_i64_lt_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I64, <, u64)
}
fn handle_i64_gt_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I64, >)
}
fn handle_i64_gt_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I64, >, u64)
}
fn handle_i64_le_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I64, <=)
}
fn handle_i64_le_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I64, <=, u64)
}
fn handle_i64_ge_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I64, >=)
}
fn handle_i64_ge_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, I64, >=, u64)
}

fn handle_f32_eq(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, F32, ==)
}
fn handle_f32_ne(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, F32, !=)
}
fn handle_f32_lt(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, F32, <)
}
fn handle_f32_gt(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, F32, >)
}
fn handle_f32_le(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, F32, <=)
}
fn handle_f32_ge(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, F32, >=)
}

fn handle_f64_eq(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, F64, ==)
}
fn handle_f64_ne(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, F64, !=)
}
fn handle_f64_lt(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, F64, <)
}
fn handle_f64_gt(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, F64, >)
}
fn handle_f64_le(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, F64, <=)
}
fn handle_f64_ge(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    cmpop!(ctx, F64, >=)
}

// --- Arithmetic Handlers ---
fn handle_i32_clz(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_i32()?;
    let result = x.leading_zeros() as i32;
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_ctz(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_i32()?;
    let result = x.trailing_zeros() as i32;
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_popcnt(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_i32()?;
    let result = x.count_ones() as i32;
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_add(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop_wrapping!(ctx, I32, wrapping_add)
}
fn handle_i32_sub(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop_wrapping!(ctx, I32, wrapping_sub)
}
fn handle_i32_mul(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop_wrapping!(ctx, I32, wrapping_mul)
}
fn handle_i32_div_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?;
    let lhs = lhs_val.to_i32()?;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i32;
        unsafe {
            asm!(
                "local.get {0}",
                "local.get {1}",
                "i32.div_s",
                "local.set {2}",
                in(local) lhs,
                in(local) rhs,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = if lhs == i32::MIN && rhs == -1 {
        0 // WebAssembly仕様: i32::MIN % -1 = 0
    } else {
        lhs / rhs
    };

    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_div_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()? as u32;
    let lhs = lhs_val.to_i32()? as u32;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i32;
        unsafe {
            asm!(
                "local.get {0}",
                "local.get {1}",
                "i32.div_u",
                "local.set {2}",
                in(local) lhs,
                in(local) rhs,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = (lhs / rhs) as i32;

    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_rem_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?;
    let lhs = lhs_val.to_i32()?;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i32;
        unsafe {
            asm!(
                "local.get {0}",
                "local.get {1}",
                "i32.rem_s",
                "local.set {2}",
                in(local) lhs,
                in(local) rhs,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = if lhs == i32::MIN && rhs == -1 {
        0 // WebAssembly仕様: i32::MIN % -1 = 0
    } else {
        lhs % rhs
    };

    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_rem_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()? as u32;
    let lhs = lhs_val.to_i32()? as u32;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i32;
        unsafe {
            asm!(
                "local.get {0}",
                "local.get {1}",
                "i32.rem_u",
                "local.set {2}",
                in(local) lhs,
                in(local) rhs,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = (lhs % rhs) as i32;

    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_and(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, I32, BitAnd, bitand)
}
fn handle_i32_or(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, I32, BitOr, bitor)
}
fn handle_i32_xor(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, I32, BitXor, bitxor)
}
fn handle_i32_shl(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?;
    let lhs = lhs_val.to_i32()?;
    let result = lhs.wrapping_shl(rhs as u32);
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_shr_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?;
    let lhs = lhs_val.to_i32()?;
    let result = lhs.wrapping_shr(rhs as u32);
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_shr_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()? as u32;
    let lhs = lhs_val.to_i32()? as u32;
    let result = lhs.wrapping_shr(rhs);
    ctx.value_stack.push(Val::Num(Num::I32(result as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_rotl(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?;
    let lhs = lhs_val.to_i32()?;
    let result = lhs.rotate_left(rhs as u32);
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_rotr(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?;
    let lhs = lhs_val.to_i32()?;
    let result = lhs.rotate_right(rhs as u32);
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_clz(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_i64()?;
    let result = x.leading_zeros() as i64;
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_ctz(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_i64()?;
    let result = x.trailing_zeros() as i64;
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_popcnt(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_i64()?;
    let result = x.count_ones() as i64;
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_add(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop_wrapping!(ctx, I64, wrapping_add)
}
fn handle_i64_sub(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop_wrapping!(ctx, I64, wrapping_sub)
}
fn handle_i64_mul(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop_wrapping!(ctx, I64, wrapping_mul)
}
fn handle_i64_div_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?;
    let lhs = lhs_val.to_i64()?;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i64;
        unsafe {
            asm!(
                "local.get {0}",
                "local.get {1}",
                "i64.div_s",
                "local.set {2}",
                in(local) lhs,
                in(local) rhs,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = lhs / rhs;

    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_div_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()? as u64;
    let lhs = lhs_val.to_i64()? as u64;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i64;
        unsafe {
            asm!(
                "local.get {0}",
                "local.get {1}",
                "i64.div_u",
                "local.set {2}",
                in(local) lhs,
                in(local) rhs,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = (lhs / rhs) as i64;

    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_rem_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?;
    let lhs = lhs_val.to_i64()?;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i64;
        unsafe {
            asm!(
                "local.get {0}",
                "local.get {1}",
                "i64.rem_s",
                "local.set {2}",
                in(local) lhs,
                in(local) rhs,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = lhs % rhs;

    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_rem_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()? as u64;
    let lhs = lhs_val.to_i64()? as u64;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i64;
        unsafe {
            asm!(
                "local.get {0}",
                "local.get {1}",
                "i64.rem_u",
                "local.set {2}",
                in(local) lhs,
                in(local) rhs,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = (lhs % rhs) as i64;

    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_and(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, I64, BitAnd, bitand)
}
fn handle_i64_or(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, I64, BitOr, bitor)
}
fn handle_i64_xor(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, I64, BitXor, bitxor)
}
fn handle_i64_shl(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?;
    let lhs = lhs_val.to_i64()?;
    let result = lhs.wrapping_shl(rhs as u32);
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_shr_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?;
    let lhs = lhs_val.to_i64()?;
    let result = lhs.wrapping_shr(rhs as u32);
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_shr_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()? as u64;
    let lhs = lhs_val.to_i64()? as u64;
    let result = lhs.wrapping_shr(rhs as u32);
    ctx.value_stack.push(Val::Num(Num::I64(result as i64)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_rotl(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?;
    let lhs = lhs_val.to_i64()?;
    let result = lhs.rotate_left(rhs as u32);
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_rotr(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?;
    let lhs = lhs_val.to_i64()?;
    let result = lhs.rotate_right(rhs as u32);
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f32_abs(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f32()?;
    let result = x.abs();
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_neg(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f32()?;
    let result = -x;
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_ceil(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f32()?;
    let result = x.ceil();
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_floor(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f32()?;
    let result = x.floor();
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_trunc(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f32()?;
    let result = x.trunc();
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_nearest(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f32()?;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: f32;
        unsafe {
            asm!(
                "local.get {0}",
                "f32.nearest",
                "local.set {1}",
                in(local) x,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = x.round_ties_even();

    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_sqrt(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f32()?;
    let result = x.sqrt();
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_add(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, F32, Add, add)
}
fn handle_f32_sub(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, F32, Sub, sub)
}
fn handle_f32_mul(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, F32, Mul, mul)
}
fn handle_f32_div(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, F32, Div, div)
}
fn handle_f32_min(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_f32()?;
    let lhs = lhs_val.to_f32()?;
    let result = if lhs.is_nan() || rhs.is_nan() {
        f32::NAN
    } else if lhs == 0.0 && rhs == 0.0 && (lhs.is_sign_negative() || rhs.is_sign_negative()) {
        -0.0 // min(±0, ±0) where at least one is negative
    } else {
        lhs.min(rhs)
    };
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_max(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_f32()?;
    let lhs = lhs_val.to_f32()?;
    let result = if lhs.is_nan() || rhs.is_nan() {
        f32::NAN
    } else if lhs == 0.0 && rhs == 0.0 && (lhs.is_sign_positive() || rhs.is_sign_positive()) {
        0.0 // max(±0, ±0) where at least one is positive
    } else {
        lhs.max(rhs)
    };
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_copysign(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_f32()?;
    let lhs = lhs_val.to_f32()?;
    let result = lhs.copysign(rhs);
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_abs(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f64()?;
    let result = x.abs();
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f64_neg(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f64()?;
    let result = -x;
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f64_ceil(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f64()?;
    let result = x.ceil();
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f64_floor(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f64()?;
    let result = x.floor();
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f64_trunc(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f64()?;
    let result = x.trunc();
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f64_nearest(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f64()?;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: f64;
        unsafe {
            asm!(
                "local.get {0}",
                "f64.nearest",
                "local.set {1}",
                in(local) x,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = x.round();

    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_extend8_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i32()?;
    let result = (val as i8) as i32;
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_extend16_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i32()?;
    let result = (val as i16) as i32;
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_extend8_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i64()?;
    let result = (val as i8) as i64;
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_extend16_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i64()?;
    let result = (val as i16) as i64;
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_extend32_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i32()?;
    let result = val as i64;
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_wrap_i64(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i64()?;
    let result = val as i32;
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_extend_i32_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i32()?;
    let result = (val as u32) as i64;
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_promote_f32(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_f32()?;
    let result = val as f64;
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f32_demote_f64(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_f64()?;
    let result = val as f32;
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_trunc_f32_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_f32()?;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i32;
        unsafe {
            asm!(
                "local.get {0}",
                "i32.trunc_f32_s",
                "local.set {1}",
                in(local) val,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = val.trunc() as i32;

    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_trunc_f32_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_f32()?;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i32;
        unsafe {
            asm!(
                "local.get {0}",
                "i32.trunc_f32_u",
                "local.set {1}",
                in(local) val,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = val.trunc() as u32 as i32;

    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_trunc_f64_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f64()?;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i32;
        unsafe {
            asm!(
                "local.get {0}",
                "i32.trunc_f64_s",
                "local.set {1}",
                in(local) val,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = val.trunc() as i32;

    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_trunc_f64_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f64 = val_opt.to_f64()?;
    if val_f64.is_nan() {
        return Err(RuntimeError::InvalidConversionToInt);
    }
    let truncated = val_f64.trunc();
    if !(truncated >= 0.0 && truncated < u32::MAX as f64 + 1.0) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.value_stack
        .push(Val::Num(Num::I32(truncated as u32 as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_trunc_f32_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f32()?;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i64;
        unsafe {
            asm!(
                "local.get {0}",
                "i64.trunc_f32_s",
                "local.set {1}",
                in(local) val,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = val.trunc() as i64;

    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_trunc_f32_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f32()?;

    // Check for overflow conditions according to WebAssembly spec
    if val.is_nan() || val.is_infinite() || val < 0.0 || val >= (u64::MAX as f32) {
        return Err(RuntimeError::IntegerOverflow);
    }

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i64;
        unsafe {
            asm!(
                "local.get {0}",
                "i64.trunc_f32_u",
                "local.set {1}",
                in(local) val,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = val.trunc() as u64 as i64;

    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_trunc_f64_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f64()?;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i64;
        unsafe {
            asm!(
                "local.get {0}",
                "i64.trunc_f64_s",
                "local.set {1}",
                in(local) val,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = val.trunc() as i64;

    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_trunc_f64_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f64()?;

    #[cfg(target_arch = "wasm32")]
    let result = {
        let mut result: i64;
        unsafe {
            asm!(
                "local.get {0}",
                "i64.trunc_f64_u",
                "local.set {1}",
                in(local) val,
                out(local) result,
            );
        }
        result
    };

    #[cfg(not(target_arch = "wasm32"))]
    let result = val.trunc() as u64 as i64;

    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_unimplemented(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    Err(RuntimeError::UnimplementedInstruction)
}

fn handle_br_table(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    // First pop the index
    let i_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let i = i_val.to_i32()?;

    if let Operand::BrTable { targets, default } = operand {
        let chosen_operand = if let Some(target_operand) = targets.get(i as usize) {
            target_operand
        } else {
            default
        };

        if let Operand::LabelIdx {
            target_ip,
            arity,
            target_label_stack_idx,
            original_wasm_depth,
            is_loop: _,
        } = chosen_operand
        {
            if *target_ip == usize::MAX {
                return Err(RuntimeError::ExecutionFailed(
                    "Branch fixup not done for BrTable target",
                ));
            }

            // Then pop the values needed for the branch target
            let values_to_push = if *arity > 0 {
                ctx.pop_n_values(*arity)?
            } else {
                Vec::new()
            };

            Ok(HandlerResult::Branch {
                target_ip: *target_ip,
                target_label_stack_idx: *target_label_stack_idx,
                values_to_push,
                branch_depth: *original_wasm_depth,
            })
        } else {
            Err(RuntimeError::InvalidOperand)
        }
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_call_indirect(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::CallIndirect {
        type_idx: expected_type_idx,
        table_idx,
    } = operand
    {
        let i_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let i = i_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        let table_addr = module_inst
            .table_addrs
            .get(table_idx.0 as usize)
            .ok_or(RuntimeError::TableNotFound)?;

        let func_ref_option = table_addr.get_func_addr(i as usize);

        if let Some(func_addr) = func_ref_option {
            let actual_type = func_addr.func_type();
            let expected_type = module_inst.types.get_by_idx(expected_type_idx.clone());

            if actual_type != *expected_type {
                return Err(RuntimeError::IndirectCallTypeMismatch);
            }
            Ok(HandlerResult::Invoke(func_addr.clone()))
        } else {
            // TODO: Distinguish between UninitializedElement and TableOutOfBounds if necessary
            Err(RuntimeError::UninitializedElement)
        }
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_select(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let cond_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let cond = cond_val.to_i32()?;
    let val2 = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val1 = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;

    if cond != 0 {
        ctx.value_stack.push(val1);
    } else {
        ctx.value_stack.push(val2);
    }
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_return(
    _ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    Ok(HandlerResult::Return)
}

fn handle_drop(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let _ = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_local_get(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::LocalIdx(LocalIdx(index_val)) = operand {
        let index = index_val as usize;
        if index >= ctx.frame.locals.len() {
            return Err(RuntimeError::LocalIndexOutOfBounds);
        }
        let val = ctx.frame.locals[index].clone();
        ctx.value_stack.push(val.clone());
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_local_set(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::LocalIdx(LocalIdx(index_val)) = operand {
        let index = index_val as usize;
        if index >= ctx.frame.locals.len() {
            return Err(RuntimeError::LocalIndexOutOfBounds);
        }
        let val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        ctx.frame.locals[index] = val;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_local_tee(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::LocalIdx(LocalIdx(index_val)) = operand {
        let index = index_val as usize;
        if index >= ctx.frame.locals.len() {
            return Err(RuntimeError::LocalIndexOutOfBounds);
        }
        let val = ctx
            .value_stack
            .last()
            .ok_or(RuntimeError::ValueStackUnderflow)?
            .clone();
        ctx.frame.locals[index] = val;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_global_get(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::GlobalIdx(GlobalIdx(index_val)) = operand {
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        let global_addr = module_inst
            .global_addrs
            .get_by_idx(GlobalIdx(index_val))
            .clone();
        ctx.value_stack.push(global_addr.get());
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_global_set(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::GlobalIdx(GlobalIdx(index_val)) = operand {
        let val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        let global_addr = module_inst
            .global_addrs
            .get_by_idx(GlobalIdx(index_val))
            .clone();
        global_addr.set(val)?;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val = mem_addr.load::<i32>(&arg, ptr)?;
        ctx.value_stack.push(Val::Num(Num::I32(val)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val = mem_addr.load::<i64>(&arg, ptr)?;
        ctx.value_stack.push(Val::Num(Num::I64(val)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f32_load(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val = mem_addr.load::<f32>(&arg, ptr)?;
        ctx.value_stack.push(Val::Num(Num::F32(val)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f64_load(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val = mem_addr.load::<f64>(&arg, ptr)?;
        ctx.value_stack.push(Val::Num(Num::F64(val)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load8_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i8 = mem_addr.load::<i8>(&arg, ptr)?;
        let val_i32 = val_i8 as i32;
        ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load8_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u8 = mem_addr.load::<u8>(&arg, ptr)?;
        let val_i32 = val_u8 as i32;
        ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load16_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i16 = mem_addr.load::<i16>(&arg, ptr)?;
        let val_i32 = val_i16 as i32;
        ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load16_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u16 = mem_addr.load::<u16>(&arg, ptr)?;
        let val_i32 = val_u16 as i32;
        ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load8_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i8 = mem_addr.load::<i8>(&arg, ptr)?;
        let val_i64 = val_i8 as i64;
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load8_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u8 = mem_addr.load::<u8>(&arg, ptr)?;
        let val_i64 = val_u8 as i64;
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load16_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i16 = mem_addr.load::<i16>(&arg, ptr)?;
        let val_i64 = val_i16 as i64;
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load16_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u16 = mem_addr.load::<u16>(&arg, ptr)?;
        let val_i64 = val_u16 as i64;
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load32_s(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i32 = mem_addr.load::<i32>(&arg, ptr)?;
        let val_i64 = val_i32 as i64;
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load32_u(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u32 = mem_addr.load::<u32>(&arg, ptr)?;
        let val_i64 = val_u32 as i64;
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_store(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i32>(&arg, ptr, val.to_i32()?)?;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_store(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i64>(&arg, ptr, val.to_i64()?)?;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f32_store(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<f32>(&arg, ptr, val.to_f32()?)?;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f64_store(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let data = val.to_f64()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<f64>(&arg, ptr, data)?;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_store8(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let val_i32 = val.to_i32()?;
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i8>(&arg, ptr, val_i32 as i8)?;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_store16(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let val_i32 = val.to_i32()?;
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i16>(&arg, ptr, val_i32 as i16)?;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_store8(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let val_i64 = val.to_i64()?;
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i8>(&arg, ptr, val_i64 as i8)?;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_store16(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let val_i64 = val.to_i64()?;
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i16>(&arg, ptr, val_i64 as i16)?;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_store32(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let val_i64 = val.to_i64()?;
        let ptr_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i32>(&arg, ptr, val_i64 as i32)?;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f64_sqrt(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f64()?;
    let result = x.sqrt();
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_add(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, F64, Add, add)
}
fn handle_f64_sub(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, F64, Sub, sub)
}
fn handle_f64_mul(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, F64, Mul, mul)
}
fn handle_f64_div(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    binop!(ctx, F64, Div, div)
}

fn handle_f64_min(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_f64()?;
    let lhs = lhs_val.to_f64()?;
    let result = lhs.min(rhs);
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_max(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_f64()?;
    let lhs = lhs_val.to_f64()?;
    let result = lhs.max(rhs);
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_copysign(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_f64()?;
    let lhs = lhs_val.to_f64()?;
    let result = lhs.copysign(rhs);
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_memory_size(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let module_inst = ctx
        .frame
        .module
        .upgrade()
        .ok_or(RuntimeError::ModuleInstanceGone)?;
    if module_inst.mem_addrs.is_empty() {
        return Err(RuntimeError::MemoryNotFound);
    }
    let mem_addr = &module_inst.mem_addrs[0];
    let size = mem_addr.mem_size();
    ctx.value_stack.push(Val::Num(Num::I32(size as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_memory_grow(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let delta_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let delta = delta_val.to_i32()?;
    let module_inst = ctx
        .frame
        .module
        .upgrade()
        .ok_or(RuntimeError::ModuleInstanceGone)?;
    if module_inst.mem_addrs.is_empty() {
        return Err(RuntimeError::MemoryNotFound);
    }
    let mem_addr = &module_inst.mem_addrs[0];
    let delta_u32: u32 = delta
        .try_into()
        .map_err(|_| RuntimeError::InvalidParameterCount)?;
    let prev_size = mem_addr.mem_grow(
        delta_u32
            .try_into()
            .map_err(|_| RuntimeError::InvalidParameterCount)?,
    );
    ctx.value_stack.push(Val::Num(Num::I32(prev_size as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_memory_copy(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let len_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let src_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let dest_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;

    let len = len_val.to_i32()?;
    let src = src_val.to_i32()?;
    let dest = dest_val.to_i32()?;

    let module_inst = ctx
        .frame
        .module
        .upgrade()
        .ok_or(RuntimeError::ModuleInstanceGone)?;

    if module_inst.mem_addrs.is_empty() {
        return Err(RuntimeError::MemoryNotFound);
    }

    let mem_addr = &module_inst.mem_addrs[0];
    mem_addr.memory_copy(dest, src, len)?;

    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_memory_init(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let len_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let offset_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let dest_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;

    let len = len_val.to_i32()? as usize;
    let offset = offset_val.to_i32()? as usize;
    let dest = dest_val.to_i32()? as usize;

    let data_index = match operand {
        Operand::I32(idx) => *idx as u32,
        _ => return Err(RuntimeError::InvalidOperand),
    };

    let module_inst = ctx
        .frame
        .module
        .upgrade()
        .ok_or(RuntimeError::ModuleInstanceGone)?;

    if module_inst.mem_addrs.is_empty() {
        return Err(RuntimeError::MemoryNotFound);
    }

    // Get data segment
    if data_index as usize >= module_inst.data_addrs.len() {
        return Err(RuntimeError::InvalidDataSegmentIndex);
    }

    let data_addr = &module_inst.data_addrs[data_index as usize];
    let data_bytes = data_addr.get_data();

    let mem_addr = &module_inst.mem_addrs[0];

    for i in 0..len {
        let byte_value = data_bytes[offset + i];
        let memarg = Memarg {
            offset: 0,
            align: 1,
        };
        mem_addr.store(&memarg, (dest as usize + i) as i32, byte_value)?;
    }

    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_memory_fill(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let size_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let dest_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;

    let size = size_val.to_i32()? as usize;
    let val = val_val.to_i32()? as u8;
    let dest = dest_val.to_i32()? as usize;

    let module_inst = ctx
        .frame
        .module
        .upgrade()
        .ok_or(RuntimeError::ModuleInstanceGone)?;

    if module_inst.mem_addrs.is_empty() {
        return Err(RuntimeError::MemoryNotFound);
    }

    let mem_addr = &module_inst.mem_addrs[0];

    // Fill memory with the specified value using the dedicated memory_fill method
    mem_addr.memory_fill(dest as i32, val, size as i32)?;

    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_ref_null(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    ctx.value_stack.push(Val::Ref(Ref::RefNull));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_ref_is_null(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let ref_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;

    let is_null = match ref_val {
        Val::Ref(Ref::RefNull) => 1,
        Val::Ref(_) => 0,
        _ => return Err(RuntimeError::TypeMismatch),
    };

    ctx.value_stack.push(Val::Num(Num::I32(is_null)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_table_get(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::TableIdx(TableIdx(table_idx)) = operand {
        let index_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let index = index_val.to_i32()? as usize;

        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;

        let table_addr = module_inst
            .table_addrs
            .get(table_idx as usize)
            .ok_or(RuntimeError::InvalidTableIndex)?;

        let ref_val = table_addr.get(index);
        ctx.value_stack.push(ref_val);
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_table_set(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::TableIdx(TableIdx(table_idx)) = operand {
        let ref_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let index_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let index = index_val.to_i32()? as usize;

        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;

        let table_addr = module_inst
            .table_addrs
            .get(table_idx as usize)
            .ok_or(RuntimeError::InvalidTableIndex)?;

        table_addr.set(index, ref_val)?;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_table_fill(
    ctx: &mut ExecutionContext,
    operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::TableIdx(TableIdx(table_idx)) = operand {
        // Pop arguments from stack: size, value, index (in reverse order)
        let size_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let size = size_val.to_i32()? as usize;

        let fill_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;

        let index_val = ctx
            .value_stack
            .pop()
            .ok_or(RuntimeError::ValueStackUnderflow)?;
        let index = index_val.to_i32()? as usize;

        let module_inst = ctx
            .frame
            .module
            .upgrade()
            .ok_or(RuntimeError::ModuleInstanceGone)?;

        let table_addr = module_inst
            .table_addrs
            .get(table_idx as usize)
            .ok_or(RuntimeError::InvalidTableIndex)?;

        // Fill the table with the specified value for the given range
        for i in 0..size {
            table_addr.set(index + i, fill_val.clone())?;
        }

        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f32_convert_i32_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i32()?;
    let result = val as f32;
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f32_convert_i32_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i32()?;
    let result = (val as u32) as f32;
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f32_convert_i64_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i64()?;
    let result = val as f32;
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f32_convert_i64_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i64()?;
    let result = (val as u64) as f32;
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_convert_i32_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i32()?;
    let result = val as f64;
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_convert_i32_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i32()?;
    let result = (val as u32) as f64;
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_convert_i64_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i64()?;
    let result = val as f64;
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_convert_i64_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i64()?;
    let result = (val as u64) as f64;
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_reinterpret_f32(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f32 = val_opt.to_f32()?;
    let val_i32 = unsafe { std::mem::transmute::<f32, i32>(val_f32) };
    ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_reinterpret_f64(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f64 = val_opt.to_f64()?;
    let val_i64 = unsafe { std::mem::transmute::<f64, i64>(val_f64) };
    ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f32_reinterpret_i32(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_i32 = val_opt.to_i32()?;
    let val_f32 = unsafe { std::mem::transmute::<i32, f32>(val_i32) };
    ctx.value_stack.push(Val::Num(Num::F32(val_f32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_reinterpret_i64(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_i64 = val_opt.to_i64()?;
    let val_f64 = unsafe { std::mem::transmute::<i64, f64>(val_i64) };
    ctx.value_stack.push(Val::Num(Num::F64(val_f64)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_trunc_sat_f32_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f32()?;

    let result = if val.is_nan() {
        0
    } else if val >= i32::MAX as f32 + 1.0 {
        i32::MAX
    } else if val <= i32::MIN as f32 - 1.0 {
        i32::MIN
    } else {
        val as i32
    };

    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_trunc_sat_f32_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f32()?;

    let result = if val.is_nan() || val <= -1.0 {
        0
    } else if val >= u32::MAX as f32 + 1.0 {
        u32::MAX as i32
    } else {
        val as u32 as i32
    };

    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_trunc_sat_f64_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f64()?;

    let result = if val.is_nan() {
        0
    } else if val >= i32::MAX as f64 + 1.0 {
        i32::MAX
    } else if val <= i32::MIN as f64 - 1.0 {
        i32::MIN
    } else {
        val as i32
    };

    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_trunc_sat_f64_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f64()?;

    let result = if val.is_nan() || val <= -1.0 {
        0
    } else if val >= u32::MAX as f64 + 1.0 {
        u32::MAX as i32
    } else {
        val as u32 as i32
    };

    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_trunc_sat_f32_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f32()?;

    let result = if val.is_nan() {
        0
    } else if val >= i64::MAX as f32 + 1.0 {
        i64::MAX
    } else if val <= i64::MIN as f32 - 1.0 {
        i64::MIN
    } else {
        val as i64
    };

    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_trunc_sat_f32_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f32()?;

    let result = if val.is_nan() || val <= -1.0 {
        0
    } else if val >= u64::MAX as f32 + 1.0 {
        u64::MAX as i64
    } else {
        val as u64 as i64
    };

    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_trunc_sat_f64_s(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f64()?;

    let result = if val.is_nan() {
        0
    } else if val >= i64::MAX as f64 + 1.0 {
        i64::MAX
    } else if val <= i64::MIN as f64 - 1.0 {
        i64::MIN
    } else {
        val as i64
    };

    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_trunc_sat_f64_u(
    ctx: &mut ExecutionContext,
    _operand: &Operand,
) -> Result<HandlerResult, RuntimeError> {
    let val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?
        .to_f64()?;

    let result = if val.is_nan() || val <= -1.0 {
        0
    } else if val >= u64::MAX as f64 + 1.0 {
        u64::MAX as i64
    } else {
        val as u64 as i64
    };

    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

lazy_static! {
    static ref HANDLER_TABLE: Vec<HandlerFn> = {
        let mut table: Vec<HandlerFn> = vec![handle_unimplemented; MAX_HANDLER_INDEX];
        table[HANDLER_IDX_UNREACHABLE] = handle_unreachable;
        table[HANDLER_IDX_NOP] = handle_nop;
        table[HANDLER_IDX_BLOCK] = handle_block;
        table[HANDLER_IDX_LOOP] = handle_loop;
        table[HANDLER_IDX_IF] = handle_if;
        table[HANDLER_IDX_ELSE] = handle_else;
        table[HANDLER_IDX_END] = handle_end;
        table[HANDLER_IDX_BR] = handle_br;
        table[HANDLER_IDX_BR_IF] = handle_br_if;
        table[HANDLER_IDX_BR_TABLE] = handle_br_table;
        table[HANDLER_IDX_RETURN] = handle_return;
        table[HANDLER_IDX_CALL] = handle_call;
        table[HANDLER_IDX_CALL_INDIRECT] = handle_call_indirect;
        table[HANDLER_IDX_DROP] = handle_drop;
        table[HANDLER_IDX_SELECT] = handle_select;
        table[HANDLER_IDX_LOCAL_GET] = handle_local_get;
        table[HANDLER_IDX_LOCAL_SET] = handle_local_set;
        table[HANDLER_IDX_LOCAL_TEE] = handle_local_tee;
        table[HANDLER_IDX_GLOBAL_GET] = handle_global_get;
        table[HANDLER_IDX_GLOBAL_SET] = handle_global_set;
        table[HANDLER_IDX_I32_LOAD] = handle_i32_load;
        table[HANDLER_IDX_I64_LOAD] = handle_i64_load;
        table[HANDLER_IDX_F32_LOAD] = handle_f32_load;
        table[HANDLER_IDX_F64_LOAD] = handle_f64_load;
        table[HANDLER_IDX_I32_LOAD8_S] = handle_i32_load8_s;
        table[HANDLER_IDX_I32_LOAD8_U] = handle_i32_load8_u;
        table[HANDLER_IDX_I32_LOAD16_S] = handle_i32_load16_s;
        table[HANDLER_IDX_I32_LOAD16_U] = handle_i32_load16_u;
        table[HANDLER_IDX_I64_LOAD8_S] = handle_i64_load8_s;
        table[HANDLER_IDX_I64_LOAD8_U] = handle_i64_load8_u;
        table[HANDLER_IDX_I64_LOAD16_S] = handle_i64_load16_s;
        table[HANDLER_IDX_I64_LOAD16_U] = handle_i64_load16_u;
        table[HANDLER_IDX_I64_LOAD32_S] = handle_i64_load32_s;
        table[HANDLER_IDX_I64_LOAD32_U] = handle_i64_load32_u;
        table[HANDLER_IDX_I32_STORE] = handle_i32_store;
        table[HANDLER_IDX_I64_STORE] = handle_i64_store;
        table[HANDLER_IDX_F32_STORE] = handle_f32_store;
        table[HANDLER_IDX_F64_STORE] = handle_f64_store;
        table[HANDLER_IDX_I32_STORE8] = handle_i32_store8;
        table[HANDLER_IDX_I32_STORE16] = handle_i32_store16;
        table[HANDLER_IDX_I64_STORE8] = handle_i64_store8;
        table[HANDLER_IDX_I64_STORE16] = handle_i64_store16;
        table[HANDLER_IDX_I64_STORE32] = handle_i64_store32;
        table[HANDLER_IDX_MEMORY_SIZE] = handle_memory_size;
        table[HANDLER_IDX_MEMORY_GROW] = handle_memory_grow;
        table[HANDLER_IDX_MEMORY_COPY] = handle_memory_copy;
        table[HANDLER_IDX_MEMORY_INIT] = handle_memory_init;
        table[HANDLER_IDX_MEMORY_FILL] = handle_memory_fill;
        table[HANDLER_IDX_REF_NULL] = handle_ref_null;
        table[HANDLER_IDX_REF_IS_NULL] = handle_ref_is_null;
        table[HANDLER_IDX_TABLE_GET] = handle_table_get;
        table[HANDLER_IDX_TABLE_SET] = handle_table_set;
        table[HANDLER_IDX_TABLE_FILL] = handle_table_fill;
        table[HANDLER_IDX_I32_CONST] = handle_i32_const;
        table[HANDLER_IDX_I64_CONST] = handle_i64_const;
        table[HANDLER_IDX_F32_CONST] = handle_f32_const;
        table[HANDLER_IDX_F64_CONST] = handle_f64_const;
        table[HANDLER_IDX_I32_EQZ] = handle_i32_eqz;
        table[HANDLER_IDX_I32_EQ] = handle_i32_eq;
        table[HANDLER_IDX_I32_NE] = handle_i32_ne;
        table[HANDLER_IDX_I32_LT_S] = handle_i32_lt_s;
        table[HANDLER_IDX_I32_LT_U] = handle_i32_lt_u;
        table[HANDLER_IDX_I32_GT_S] = handle_i32_gt_s;
        table[HANDLER_IDX_I32_GT_U] = handle_i32_gt_u;
        table[HANDLER_IDX_I32_LE_S] = handle_i32_le_s;
        table[HANDLER_IDX_I32_LE_U] = handle_i32_le_u;
        table[HANDLER_IDX_I32_GE_S] = handle_i32_ge_s;
        table[HANDLER_IDX_I32_GE_U] = handle_i32_ge_u;
        table[HANDLER_IDX_I64_EQZ] = handle_i64_eqz;
        table[HANDLER_IDX_I64_EQ] = handle_i64_eq;
        table[HANDLER_IDX_I64_NE] = handle_i64_ne;
        table[HANDLER_IDX_I64_LT_S] = handle_i64_lt_s;
        table[HANDLER_IDX_I64_LT_U] = handle_i64_lt_u;
        table[HANDLER_IDX_I64_GT_S] = handle_i64_gt_s;
        table[HANDLER_IDX_I64_GT_U] = handle_i64_gt_u;
        table[HANDLER_IDX_I64_LE_S] = handle_i64_le_s;
        table[HANDLER_IDX_I64_LE_U] = handle_i64_le_u;
        table[HANDLER_IDX_I64_GE_S] = handle_i64_ge_s;
        table[HANDLER_IDX_I64_GE_U] = handle_i64_ge_u;
        table[HANDLER_IDX_F32_EQ] = handle_f32_eq;
        table[HANDLER_IDX_F32_NE] = handle_f32_ne;
        table[HANDLER_IDX_F32_LT] = handle_f32_lt;
        table[HANDLER_IDX_F32_GT] = handle_f32_gt;
        table[HANDLER_IDX_F32_LE] = handle_f32_le;
        table[HANDLER_IDX_F32_GE] = handle_f32_ge;
        table[HANDLER_IDX_F64_EQ] = handle_f64_eq;
        table[HANDLER_IDX_F64_NE] = handle_f64_ne;
        table[HANDLER_IDX_F64_LT] = handle_f64_lt;
        table[HANDLER_IDX_F64_GT] = handle_f64_gt;
        table[HANDLER_IDX_F64_LE] = handle_f64_le;
        table[HANDLER_IDX_F64_GE] = handle_f64_ge;
        table[HANDLER_IDX_I32_CLZ] = handle_i32_clz;
        table[HANDLER_IDX_I32_CTZ] = handle_i32_ctz;
        table[HANDLER_IDX_I32_POPCNT] = handle_i32_popcnt;
        table[HANDLER_IDX_I32_ADD] = handle_i32_add;
        table[HANDLER_IDX_I32_SUB] = handle_i32_sub;
        table[HANDLER_IDX_I32_MUL] = handle_i32_mul;
        table[HANDLER_IDX_I32_DIV_S] = handle_i32_div_s;
        table[HANDLER_IDX_I32_DIV_U] = handle_i32_div_u;
        table[HANDLER_IDX_I32_REM_S] = handle_i32_rem_s;
        table[HANDLER_IDX_I32_REM_U] = handle_i32_rem_u;
        table[HANDLER_IDX_I32_AND] = handle_i32_and;
        table[HANDLER_IDX_I32_OR] = handle_i32_or;
        table[HANDLER_IDX_I32_XOR] = handle_i32_xor;
        table[HANDLER_IDX_I32_SHL] = handle_i32_shl;
        table[HANDLER_IDX_I32_SHR_S] = handle_i32_shr_s;
        table[HANDLER_IDX_I32_SHR_U] = handle_i32_shr_u;
        table[HANDLER_IDX_I32_ROTL] = handle_i32_rotl;
        table[HANDLER_IDX_I32_ROTR] = handle_i32_rotr;
        table[HANDLER_IDX_I64_CLZ] = handle_i64_clz;
        table[HANDLER_IDX_I64_CTZ] = handle_i64_ctz;
        table[HANDLER_IDX_I64_POPCNT] = handle_i64_popcnt;
        table[HANDLER_IDX_I64_ADD] = handle_i64_add;
        table[HANDLER_IDX_I64_SUB] = handle_i64_sub;
        table[HANDLER_IDX_I64_MUL] = handle_i64_mul;
        table[HANDLER_IDX_I64_DIV_S] = handle_i64_div_s;
        table[HANDLER_IDX_I64_DIV_U] = handle_i64_div_u;
        table[HANDLER_IDX_I64_REM_S] = handle_i64_rem_s;
        table[HANDLER_IDX_I64_REM_U] = handle_i64_rem_u;
        table[HANDLER_IDX_I64_AND] = handle_i64_and;
        table[HANDLER_IDX_I64_OR] = handle_i64_or;
        table[HANDLER_IDX_I64_XOR] = handle_i64_xor;
        table[HANDLER_IDX_I64_SHL] = handle_i64_shl;
        table[HANDLER_IDX_I64_SHR_S] = handle_i64_shr_s;
        table[HANDLER_IDX_I64_SHR_U] = handle_i64_shr_u;
        table[HANDLER_IDX_I64_ROTL] = handle_i64_rotl;
        table[HANDLER_IDX_I64_ROTR] = handle_i64_rotr;
        table[HANDLER_IDX_F32_ABS] = handle_f32_abs;
        table[HANDLER_IDX_F32_NEG] = handle_f32_neg;
        table[HANDLER_IDX_F32_CEIL] = handle_f32_ceil;
        table[HANDLER_IDX_F32_FLOOR] = handle_f32_floor;
        table[HANDLER_IDX_F32_TRUNC] = handle_f32_trunc;
        table[HANDLER_IDX_F32_NEAREST] = handle_f32_nearest;
        table[HANDLER_IDX_F32_SQRT] = handle_f32_sqrt;
        table[HANDLER_IDX_F32_ADD] = handle_f32_add;
        table[HANDLER_IDX_F32_SUB] = handle_f32_sub;
        table[HANDLER_IDX_F32_MUL] = handle_f32_mul;
        table[HANDLER_IDX_F32_DIV] = handle_f32_div;
        table[HANDLER_IDX_F32_MIN] = handle_f32_min;
        table[HANDLER_IDX_F32_MAX] = handle_f32_max;
        table[HANDLER_IDX_F32_COPYSIGN] = handle_f32_copysign;
        table[HANDLER_IDX_F64_ABS] = handle_f64_abs;
        table[HANDLER_IDX_F64_NEG] = handle_f64_neg;
        table[HANDLER_IDX_F64_CEIL] = handle_f64_ceil;
        table[HANDLER_IDX_F64_FLOOR] = handle_f64_floor;
        table[HANDLER_IDX_F64_TRUNC] = handle_f64_trunc;
        table[HANDLER_IDX_F64_NEAREST] = handle_f64_nearest;
        table[HANDLER_IDX_F64_SQRT] = handle_f64_sqrt;
        table[HANDLER_IDX_F64_ADD] = handle_f64_add;
        table[HANDLER_IDX_F64_SUB] = handle_f64_sub;
        table[HANDLER_IDX_F64_MUL] = handle_f64_mul;
        table[HANDLER_IDX_F64_DIV] = handle_f64_div;
        table[HANDLER_IDX_F64_MIN] = handle_f64_min;
        table[HANDLER_IDX_F64_MAX] = handle_f64_max;
        table[HANDLER_IDX_F64_COPYSIGN] = handle_f64_copysign;
        table[HANDLER_IDX_F32_CONVERT_I32_S] = handle_f32_convert_i32_s;
        table[HANDLER_IDX_F32_CONVERT_I32_U] = handle_f32_convert_i32_u;
        table[HANDLER_IDX_F32_CONVERT_I64_S] = handle_f32_convert_i64_s;
        table[HANDLER_IDX_F32_CONVERT_I64_U] = handle_f32_convert_i64_u;
        table[HANDLER_IDX_F64_CONVERT_I32_S] = handle_f64_convert_i32_s;
        table[HANDLER_IDX_F64_CONVERT_I32_U] = handle_f64_convert_i32_u;
        table[HANDLER_IDX_F64_CONVERT_I64_S] = handle_f64_convert_i64_s;
        table[HANDLER_IDX_F64_CONVERT_I64_U] = handle_f64_convert_i64_u;
        table[HANDLER_IDX_I32_REINTERPRET_F32] = handle_i32_reinterpret_f32;
        table[HANDLER_IDX_I64_REINTERPRET_F64] = handle_i64_reinterpret_f64;
        table[HANDLER_IDX_F32_REINTERPRET_I32] = handle_f32_reinterpret_i32;
        table[HANDLER_IDX_F64_REINTERPRET_I64] = handle_f64_reinterpret_i64;
        table[HANDLER_IDX_I32_WRAP_I64] = handle_i32_wrap_i64;
        table[HANDLER_IDX_I32_TRUNC_F32_S] = handle_i32_trunc_f32_s;
        table[HANDLER_IDX_I32_TRUNC_F32_U] = handle_i32_trunc_f32_u;
        table[HANDLER_IDX_I32_TRUNC_F64_S] = handle_i32_trunc_f64_s;
        table[HANDLER_IDX_I32_TRUNC_F64_U] = handle_i32_trunc_f64_u;
        table[HANDLER_IDX_I64_EXTEND_I32_S] = handle_i64_extend32_s;
        table[HANDLER_IDX_I64_EXTEND_I32_U] = handle_i64_extend_i32_u;
        table[HANDLER_IDX_I64_TRUNC_F32_S] = handle_i64_trunc_f32_s;
        table[HANDLER_IDX_I64_TRUNC_F32_U] = handle_i64_trunc_f32_u;
        table[HANDLER_IDX_I64_TRUNC_F64_S] = handle_i64_trunc_f64_s;
        table[HANDLER_IDX_I64_TRUNC_F64_U] = handle_i64_trunc_f64_u;
        table[HANDLER_IDX_F32_DEMOTE_F64] = handle_f32_demote_f64;
        table[HANDLER_IDX_F64_PROMOTE_F32] = handle_f64_promote_f32;
        table[HANDLER_IDX_I32_EXTEND8_S] = handle_i32_extend8_s;
        table[HANDLER_IDX_I32_EXTEND16_S] = handle_i32_extend16_s;
        table[HANDLER_IDX_I64_EXTEND8_S] = handle_i64_extend8_s;
        table[HANDLER_IDX_I64_EXTEND16_S] = handle_i64_extend16_s;
        table[HANDLER_IDX_I64_EXTEND32_S] = handle_i64_extend32_s;
        table[HANDLER_IDX_I32_TRUNC_SAT_F32_S] = handle_i32_trunc_sat_f32_s;
        table[HANDLER_IDX_I32_TRUNC_SAT_F32_U] = handle_i32_trunc_sat_f32_u;
        table[HANDLER_IDX_I32_TRUNC_SAT_F64_S] = handle_i32_trunc_sat_f64_s;
        table[HANDLER_IDX_I32_TRUNC_SAT_F64_U] = handle_i32_trunc_sat_f64_u;
        table[HANDLER_IDX_I64_TRUNC_SAT_F32_S] = handle_i64_trunc_sat_f32_s;
        table[HANDLER_IDX_I64_TRUNC_SAT_F32_U] = handle_i64_trunc_sat_f32_u;
        table[HANDLER_IDX_I64_TRUNC_SAT_F64_S] = handle_i64_trunc_sat_f64_s;
        table[HANDLER_IDX_I64_TRUNC_SAT_F64_U] = handle_i64_trunc_sat_f64_u;

        table
    };
}

impl<'a> ExecutionContext<'a> {
    fn pop_n_values(&mut self, n: usize) -> Result<Vec<Val>, RuntimeError> {
        let len = self.value_stack.len();
        if len < n {
            return Err(RuntimeError::ValueStackUnderflow);
        }
        let split_index = len - n;
        let second_part = self.value_stack.split_off(split_index);
        Ok(second_part.into_iter().rev().collect())
    }
}
