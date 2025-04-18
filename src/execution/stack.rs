use super::{value::*};
use crate::structure::{instructions::Instr};
use crate::error::RuntimeError;
use crate::structure::types::{FuncIdx, GlobalIdx, LocalIdx, TableIdx, TypeIdx, LabelIdx as StructureLabelIdx};
use crate::structure::instructions::Memarg;
use std::collections::HashMap;
use std::ops::{BitAnd, BitOr, BitXor, Add, Sub, Mul, Div};
use lazy_static::lazy_static;
use crate::execution::value::Val;
use crate::execution::module::{ModuleInst, GetInstanceByIdx};
use crate::execution::func::{FuncAddr, FuncInst};
use std::sync::{Weak as SyncWeak};
use crate::structure::types::{ValueType, NumType, VecType};

#[derive(Clone, Debug, PartialEq)]
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
    LabelIdx {
        target_ip: usize,
        arity: usize,
        target_label_stack_idx: usize,
    },
    MemArg(Memarg),
    BrTable { targets: Vec<Operand>, default: Box<Operand> },
}

#[derive(Clone, Debug)]
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
    Continue(usize), // Continue execution at the given IP
    Return,          // Signal function return
    Invoke(FuncAddr), // Signal function invocation with the target address
    Branch { // Added for branch instructions with value transfer
        target_ip: usize,
        target_label_stack_idx: usize, // Index in FrameStack.label_stack
        values_to_push: Vec<Val>,
    },
}

type HandlerFn = fn(&mut ExecutionContext, &Operand) -> Result<HandlerResult, RuntimeError>;

#[derive(Clone)]
pub enum ModuleLevelInstr{
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

// TODO: Add remaining indices (Ref types, Table, Bulk Memory, SIMD, TruncSat)

pub const MAX_HANDLER_INDEX: usize = 0xC5;

pub struct Stacks {
    pub activation_frame_stack: Vec<FrameStack>,
}

impl Stacks {
    pub fn new(funcaddr: &FuncAddr, params: Vec<Val>) -> Result<Stacks, RuntimeError> {
        let func_inst_guard = funcaddr.read_lock().expect("RwLock poisoned");
        match &*func_inst_guard {
            FuncInst::RuntimeFunc { type_, module, code } => {
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
                            ValueType::RefType(_) => todo!("RefType local initialization"),
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
                            arity: type_.results.len(), // Initialize arity
                        },
                        processed_instrs: code.body.clone(),
                        value_stack: vec![],
                        ip: 0,
                    }],
                    void: type_.results.is_empty(),
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
        }
    }

    /*
    This Function Only Handle Instruction Spanning FrameStack.
    i.e., Invoke Wasm Function, Return Function and Call Host-function.
    */
    pub fn exec_instr(&mut self) -> Result<Vec<Val>, RuntimeError> {
        // This outer loop manages frame transitions (call/return)
        while !self.activation_frame_stack.is_empty() {
            let frame_stack_idx = self.activation_frame_stack.len() - 1;
            let mut called_func_addr: Option<FuncAddr> = None;

            let module_level_instr_result = {
                let current_frame_stack = &mut self.activation_frame_stack[frame_stack_idx];
                current_frame_stack.run_dtc_loop(&mut called_func_addr)?
            };

            match module_level_instr_result {
                Ok(Some(instr)) => { // Frame transition requested (Return/Call)
                    // Need mutable borrow for potential param passing in Invoke
                    let current_frame_stack = self.activation_frame_stack.last_mut().unwrap(); // Should not panic if loop condition holds
                    // Ensure label_stack is not empty before accessing last_mut
                    if current_frame_stack.label_stack.is_empty() {
                        // This should ideally not happen if run_dtc_loop returned Ok(Some(...))
                        return Err(RuntimeError::StackError("Label stack empty during frame transition"));
                    }
                    let cur_label_stack = current_frame_stack.label_stack.last_mut().unwrap();

                    match instr {
                        ModuleLevelInstr::Invoke(func_addr) => {
                            // Use read_lock() method instead of direct field access
                            let func_inst_guard = func_addr.read_lock().expect("RwLock poisoned");
                            match &*func_inst_guard { // Match on the guard
                                FuncInst::RuntimeFunc{type_,module,code} => {
                                    // Prepare locals for the new frame
                                    let params_len = type_.params.len();
                                    // Take params from the current value stack
                                    if cur_label_stack.value_stack.len() < params_len {
                                        return Err(RuntimeError::ValueStackUnderflow);
                                    }
                                    let params = cur_label_stack.value_stack.split_off(cur_label_stack.value_stack.len() - params_len); // No mut needed

                                    // Initialize locals (params + defaults)
                                    let mut locals = params;
                                    for v in code.locals.iter(){
                                        for _ in 0..(v.0){
                                            locals.push(
                                                match v.1{
                                                    ValueType::NumType(NumType::I32) => Val::Num(Num::I32(0)),
                                                    ValueType::NumType(NumType::I64) => Val::Num(Num::I64(0)),
                                                    ValueType::NumType(NumType::F32) => Val::Num(Num::F32(0.0)),
                                                    ValueType::NumType(NumType::F64) => Val::Num(Num::F64(0.0)),
                                                    ValueType::VecType(VecType::V128) => Val::Vec_(Vec_::V128(0)),
                                                    ValueType::RefType(_) => todo!("RefType local init in Invoke"),
                                                }
                                            );
                                        }
                                    };

                                    let frame = FrameStack{
                                        frame: Frame{
                                            locals,
                                            module: module.clone(),
                                            n: type_.results.len()
                                        },
                                        label_stack: vec![
                                            LabelStack{
                                                label: Label{
                                                    locals_num: type_.results.len(),
                                                    arity: type_.results.len(), // Initialize arity
                                                },
                                                processed_instrs: code.body.clone(),
                                                value_stack: vec![],
                                                ip: 0, // Initialize IP for new frame
                                            }
                                        ],
                                        void: type_.results.is_empty(),
                                    };
                                    self.activation_frame_stack.push(frame);
                                    // Continue to the next iteration to run the new frame's loop

                                },
                                FuncInst::HostFunc{type_, host_code} => {
                                    // Handle host function call
                                    let params_len = type_.params.len();
                                    if cur_label_stack.value_stack.len() < params_len {
                                        return Err(RuntimeError::ValueStackUnderflow);
                                    }
                                    let params = cur_label_stack.value_stack.split_off(cur_label_stack.value_stack.len() - params_len);
                                    match host_code(params) {
                                        Ok(Some(result_val)) => {
                                            // Push result onto the *caller's* stack (which is current_frame_stack)
                                            cur_label_stack.value_stack.push(result_val);
                                            // Host call finished, advance IP in the *caller* frame
                                            cur_label_stack.ip += 1;
                                        }
                                        Ok(None) => {
                                            // No return value, still advance IP in the caller frame
                                            cur_label_stack.ip += 1;
                                        }
                                        Err(e) => return Err(e), // Propagate host error
                                    }
                                    // Host call doesn't push a new Wasm frame, continue in current frame loop?
                                    // The loop continues naturally after IP is advanced.
                                },
                            }
                        },
                        ModuleLevelInstr::Return =>{ // Triggered by handle_return result
                            // Pop the current frame
                            let finished_frame = self.activation_frame_stack.pop().unwrap(); // Should not panic if loop condition holds

                            // Check if this was the last frame
                            if self.activation_frame_stack.is_empty() {
                                // Return the final result from the finished frame's value stack
                                // Check arity before returning
                                let finished_label_stack = finished_frame.label_stack.last().ok_or(RuntimeError::StackError("Finished frame has no label stack"))?;
                                let mut return_values = finished_label_stack.value_stack.clone();
                                let expected_n = finished_frame.frame.n;
                                if return_values.len() < expected_n {
                                    return Err(RuntimeError::Trap); // Trap if not enough values
                                }
                                // Drain excess values if any
                                let final_values = return_values.split_off(return_values.len().saturating_sub(expected_n));
                                return Ok(final_values);
                            }

                            // Get potential return value(s) from the finished frame's label stack
                            let finished_label_stack = finished_frame.label_stack.last().ok_or(RuntimeError::StackError("Finished frame has no label stack"))?;
                            let mut return_values = finished_label_stack.value_stack.clone(); // Clone return values
                            let expected_n = finished_frame.frame.n;

                            // Check arity and handle Trap/excess values
                            if return_values.len() < expected_n {
                                return Err(RuntimeError::Trap); // Trap if not enough values
                            }
                            let actual_return_values = return_values.split_off(return_values.len().saturating_sub(expected_n));

                            // Push return value(s) onto caller's stack
                            if let Some(caller_frame) = self.activation_frame_stack.last_mut() {
                                 if let Some(caller_label_stack) = caller_frame.label_stack.last_mut() {
                                     caller_label_stack.value_stack.extend(actual_return_values);
                                     // Caller's IP was already advanced by run_dtc_loop before returning the Return signal
                                 } else {
                                     return Err(RuntimeError::StackError("Caller label stack empty during return"));
                                 }
                            }
                            // Continue execution in the caller frame in the next iteration
                        }
                    }
                },
                Ok(None) => { // Frame completed normally (implicit return)
                     let finished_frame = self.activation_frame_stack.pop().unwrap(); // Should not panic

                     // Check if this was the last frame
                     if self.activation_frame_stack.is_empty() {
                         // Return the final result from the finished frame's value stack
                        let finished_label_stack = finished_frame.label_stack.last().ok_or(RuntimeError::StackError("Finished frame has no label stack on implicit return"))?;
                        let mut return_values = finished_label_stack.value_stack.clone();
                        let expected_n = finished_frame.frame.n;
                        if return_values.len() < expected_n {
                            return Err(RuntimeError::Trap);
                        }
                        let final_values = return_values.split_off(return_values.len().saturating_sub(expected_n));
                         return Ok(final_values);
                     }

                     // Handle potential return value(s) based on finished_frame.frame.n
                     let finished_label_stack = finished_frame.label_stack.last().ok_or(RuntimeError::StackError("Finished frame has no label stack on implicit return"))?;
                     let mut return_values = finished_label_stack.value_stack.clone();
                     let expected_n = finished_frame.frame.n;

                     // Check arity and handle Trap/excess values
                     if return_values.len() < expected_n {
                         return Err(RuntimeError::Trap);
                     }
                     let actual_return_values = return_values.split_off(return_values.len().saturating_sub(expected_n));

                     if let Some(caller_frame) = self.activation_frame_stack.last_mut() {
                          if let Some(caller_label_stack) = caller_frame.label_stack.last_mut() {
                             caller_label_stack.value_stack.extend(actual_return_values);
                             // Caller's IP was already advanced by run_dtc_loop before reaching the end
                         } else {
                             return Err(RuntimeError::StackError("Caller label stack empty during implicit return"));
                         }
                     }
                     // Continue execution in the caller frame
                },
                Err(e) => { // Propagate runtime errors
                    return Err(e);
                }
            }
        }
        // Should only be reached if the initial stack was empty or after the last frame is popped.
    Ok(vec![])
    }
}

// Frame represents the runtime information for a function activation.
// Note: Using wasmparser::Frame directly in ExecutionContext now.
// This Frame struct might be needed for FrameStack still.
pub struct Frame{
    pub locals: Vec<Val>,
    pub module: SyncWeak<ModuleInst>, // Use sync Weak
    pub n: usize, // Number of expected return values
}

pub struct FrameStack {
    pub frame: Frame,
    pub label_stack: Vec<LabelStack>, // Keep for block/loop/if context, maybe simplify later
    pub void: bool,
}

impl FrameStack{
/// Runs the Direct Threaded Code loop for this frame.
    /// Returns Ok(Some(ModuleLevelInstr)) if a frame transition (Call/Return) is needed,
    /// Ok(None) if the frame execution completes normally (reaches end),
    /// or Err on runtime error.
    fn run_dtc_loop(&mut self, _called_func_addr_out: &mut Option<FuncAddr>) -> Result<Result<Option<ModuleLevelInstr>, RuntimeError>, RuntimeError> { // Outer Result for panic safety
        // Get mutable access to the current label stack index
        let mut current_label_stack_idx = self.label_stack.len().checked_sub(1)
            .ok_or(RuntimeError::StackError("Initial label stack empty"))?;

        loop { // This loop now also handles label stack transitions implicitly via Branch result

            // Ensure the index is valid before borrowing mutably
            if current_label_stack_idx >= self.label_stack.len() {
                 // This might happen if a Branch popped the last label stack
                 // Or if code reaches end naturally
                 break; // Exit the loop, frame finished or needs popping
            }

            let current_label_stack = &mut self.label_stack[current_label_stack_idx];
            let processed_code = &current_label_stack.processed_instrs; // Immutable borrow is fine here
            let ip = current_label_stack.ip; // Get current IP, remove mut

            if ip >= processed_code.len() {
                // Reached end of this label's code block
                current_label_stack.ip = ip; // Save IP

                // Pop the current label stack if it's not the last one
                if current_label_stack_idx > 0 {
                    self.label_stack.pop();
                    current_label_stack_idx -= 1;
                    // Continue the loop with the parent label stack
                    continue;
                } else {
                    // This was the last label stack for the frame
                    break; // Exit the loop, frame finished
                }
            }

            // Clone instruction to avoid borrowing issues
            let instruction = processed_code[ip].clone();

            let handler_fn = HANDLER_TABLE.get(instruction.handler_index)
                                        .ok_or(RuntimeError::InvalidHandlerIndex)?;

            // Create context for the *current* label stack
            let mut context = ExecutionContext {
                frame: &mut self.frame,
                value_stack: &mut self.label_stack[current_label_stack_idx].value_stack,
                ip,
            };

            // --- Execute Handler ---
            let result = handler_fn(&mut context, &instruction.operand);
            // --- End Execute Handler ---


            // Update the IP in the *correct* label stack before handling result
            self.label_stack[current_label_stack_idx].ip = ip; // Save potentially modified IP

             match result {
                Ok(handler_result) => {
                    match handler_result {
                        HandlerResult::Continue(next_ip) => {
                            // Update IP for the *current* label stack
                            self.label_stack[current_label_stack_idx].ip = next_ip;
                            // The loop will continue with the updated IP
                        }
                        HandlerResult::Return => {
                            // No IP update needed here as we are exiting the loop
                            return Ok(Ok(Some(ModuleLevelInstr::Return)));
                        }
                        HandlerResult::Invoke(func_addr) => {
                             // Update IP in the *current* label stack to point *after* the call
                            self.label_stack[current_label_stack_idx].ip = ip + 1;
                            return Ok(Ok(Some(ModuleLevelInstr::Invoke(func_addr))));
                        }
                        HandlerResult::Branch { target_ip, target_label_stack_idx, values_to_push } => {
                            // 1. Validate target index
                            if target_label_stack_idx >= self.label_stack.len() {
                                return Ok(Err(RuntimeError::StackError("Invalid target label stack index for branch")));
                            }

                            // 2. Truncate the label stack
                            // Keep the target stack and its parents
                            self.label_stack.truncate(target_label_stack_idx + 1);

                            // 3. Push values onto the new top stack (which is the target)
                            // The index must now be the new len - 1
                            let new_top_idx = self.label_stack.len() - 1;
                            self.label_stack[new_top_idx].value_stack.extend(values_to_push);

                            // 4. Set IP for the target label stack
                            self.label_stack[new_top_idx].ip = target_ip;

                            // 5. Update the current index for the next loop iteration
                            current_label_stack_idx = new_top_idx;
                            // The loop continues with the new current_label_stack_idx and its updated IP
                        }
                    }
                }
                Err(e) => {
                    // Save IP before propagating error
                     if current_label_stack_idx < self.label_stack.len() { // Check index validity before saving IP
                         self.label_stack[current_label_stack_idx].ip = ip;
                     }
                    return Ok(Err(e)); // Wrap runtime error in Ok for panic safety layer
                }
            }
        }
        // Loop finished naturally (either by reaching end or after Branch handled internally)
        Ok(Ok(None)) // Frame completed normally
    }
}

#[derive(Clone)]
pub struct Label {
    pub locals_num: usize, // Renamed from arity for clarity, maybe keep as arity? Check spec. Let's keep locals_num for now.
    pub arity: usize, // Added: Number of result values expected by this block/label
}

pub struct LabelStack {
    pub label: Label,
    pub processed_instrs: Vec<ProcessedInstr>,
    pub value_stack: Vec<Val>,
    pub ip: usize,
}


#[derive(Clone)]
pub enum FrameLevelInstr{
    Label(Label, Vec<Instr>),
    Br(StructureLabelIdx),
    EndLabel,
    Invoke(FuncAddr),
    Return
}

#[derive(Clone)]
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
    ($ctx:ident, $operand_type:ident, $op_trait:ident, $op_method:ident, $result_type:ident) => {
        {
            let rhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let (lhs, rhs) = match (lhs_val, rhs_val) {
                (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l, r),
                _ => return Err(RuntimeError::TypeMismatch),
            };
            $ctx.value_stack.push(Val::Num(Num::$result_type($op_trait::$op_method(lhs, rhs))));
             Ok(HandlerResult::Continue($ctx.ip + 1))
        }
    };
     ($ctx:ident, $operand_type:ident, $op_trait:ident, $op_method:ident) => {
        binop!($ctx, $operand_type, $op_trait, $op_method, $operand_type)
     };
}

macro_rules! binop_wrapping {
    ($ctx:ident, $operand_type:ident, $op_method:ident, $result_type:ident) => {
        {
            let rhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
             let (lhs, rhs) = match (lhs_val, rhs_val) {
                (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l, r),
                _ => return Err(RuntimeError::TypeMismatch),
            };
            $ctx.value_stack.push(Val::Num(Num::$result_type(lhs.$op_method(rhs))));
             Ok(HandlerResult::Continue($ctx.ip + 1))
        }
    };
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
     ($ctx:ident, $operand_type:ident, $op:tt, $cast_type:ty) => { // For comparing unsigned
        {
            let rhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let lhs_val = $ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
            let (lhs, rhs) = match (lhs_val, rhs_val) {
                 (Val::Num(Num::$operand_type(l)), Val::Num(Num::$operand_type(r))) => (l as $cast_type, r as $cast_type), // Cast here
                 _ => return Err(RuntimeError::TypeMismatch),
            };
            $ctx.value_stack.push(Val::Num(Num::I32((lhs $op rhs) as i32)));
             Ok(HandlerResult::Continue($ctx.ip + 1))
        }
    };
}

fn handle_unreachable(_ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    Err(RuntimeError::Unreachable)
}

fn handle_nop(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_block(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_loop(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_if(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let cond_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let cond = cond_val.to_i32()?; // Ensure ? is applied
    if let &Operand::LabelIdx { target_ip, arity: _, target_label_stack_idx: _ } = operand { // Ignore arity and index
         if target_ip == usize::MAX { return Err(RuntimeError::ExecutionFailed("Branch fixup not done for If")); }
        if cond == 0 { // Compare after using ?
            Ok(HandlerResult::Continue(target_ip)) // Jump if condition is false
        } else {
            Ok(HandlerResult::Continue(ctx.ip + 1))
        }
    } else {
         Err(RuntimeError::InvalidOperand) // Should not happen if preprocessing is correct
    }
}

fn handle_else(_ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> { 
     if let &Operand::LabelIdx { target_ip, arity: _, target_label_stack_idx: _ } = operand { // Ignore arity and index
        if target_ip == usize::MAX { return Err(RuntimeError::ExecutionFailed("Branch fixup not done for Else")); }
        Ok(HandlerResult::Continue(target_ip)) // Unconditional jump to end
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}


fn handle_end(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    Ok(HandlerResult::Continue(ctx.ip + 1)) 
}

fn handle_br(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> { 
    if let Operand::LabelIdx { target_ip, arity, target_label_stack_idx } = operand { // Keep arity and index here
        // Check if target_ip is still the placeholder (fixup needed)
        if *target_ip == usize::MAX { 
            return Err(RuntimeError::ExecutionFailed("Branch fixup not done for Br")); 
        }

        // Use arity and target_label_stack_idx from the operand
        let values_to_push = ctx.pop_n_values(*arity)?; // Use arity from operand

        Ok(HandlerResult::Branch { 
            target_ip: *target_ip, 
            target_label_stack_idx: *target_label_stack_idx, // Use index from operand
            values_to_push 
        })
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_br_if(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let cond_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let cond = cond_val.to_i32()?; // Ensure ? is applied

    if cond != 0 { // Condition is true, perform the branch
        if let Operand::LabelIdx { target_ip, arity, target_label_stack_idx } = operand { // Keep arity and index here
            // Check if target_ip is still the placeholder (fixup needed)
             if *target_ip == usize::MAX { 
                 return Err(RuntimeError::ExecutionFailed("Branch fixup not done for BrIf")); 
             }

             // Use arity and target_label_stack_idx from the operand
             let values_to_push = ctx.pop_n_values(*arity)?; // Use arity from operand

             Ok(HandlerResult::Branch { 
                 target_ip: *target_ip, 
                 target_label_stack_idx: *target_label_stack_idx, // Use index from operand
                 values_to_push 
             })
        } else {
            Err(RuntimeError::InvalidOperand)
        }
    } else {
        // Condition is false, continue to the next instruction
        Ok(HandlerResult::Continue(ctx.ip + 1))
    }
}

fn handle_call(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    // Match on the reference
    if let Operand::FuncIdx(func_idx) = operand {
        let instance = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        let func_addr = instance.func_addrs.get_by_idx(func_idx.clone()).clone();
        Ok(HandlerResult::Invoke(func_addr))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}


fn handle_i32_const(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    // Match on reference
    if let &Operand::I32(val) = operand {
        ctx.value_stack.push(Val::Num(Num::I32(val))); // No dereference needed
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_const(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    // Match on reference
    if let &Operand::I64(val) = operand {
        ctx.value_stack.push(Val::Num(Num::I64(val))); // No dereference needed
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f32_const(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    // Match on reference
    if let &Operand::F32(val) = operand {
        ctx.value_stack.push(Val::Num(Num::F32(val))); // No dereference needed
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f64_const(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    // Match on reference
    if let &Operand::F64(val) = operand {
        ctx.value_stack.push(Val::Num(Num::F64(val))); // No dereference needed
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

// --- Comparison Handlers (Operand not used, signature change only) ---
fn handle_i32_eqz(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i32()?; // Add ?
    ctx.value_stack.push(Val::Num(Num::I32((val == 0) as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_eq(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I32, ==) }
fn handle_i32_ne(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I32, !=) }
fn handle_i32_lt_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I32, <) }
fn handle_i32_lt_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I32, <, u32) }
fn handle_i32_gt_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I32, >) }
fn handle_i32_gt_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I32, >, u32) }
fn handle_i32_le_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I32, <=) }
fn handle_i32_le_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I32, <=, u32) }
fn handle_i32_ge_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I32, >=) }
fn handle_i32_ge_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I32, >=, u32) }

fn handle_i64_eqz(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i64()?; // Add ?
    ctx.value_stack.push(Val::Num(Num::I32((val == 0) as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_eq(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I64, ==) }
fn handle_i64_ne(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I64, !=) }
fn handle_i64_lt_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I64, <) }
fn handle_i64_lt_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I64, <, u64) }
fn handle_i64_gt_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I64, >) }
fn handle_i64_gt_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I64, >, u64) }
fn handle_i64_le_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I64, <=) }
fn handle_i64_le_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I64, <=, u64) }
fn handle_i64_ge_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I64, >=) }
fn handle_i64_ge_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, I64, >=, u64) }

fn handle_f32_eq(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, F32, ==) }
fn handle_f32_ne(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, F32, !=) }
fn handle_f32_lt(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, F32, <) }
fn handle_f32_gt(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, F32, >) }
fn handle_f32_le(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, F32, <=) }
fn handle_f32_ge(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, F32, >=) }

fn handle_f64_eq(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, F64, ==) }
fn handle_f64_ne(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, F64, !=) }
fn handle_f64_lt(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, F64, <) }
fn handle_f64_gt(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, F64, >) }
fn handle_f64_le(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, F64, <=) }
fn handle_f64_ge(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { cmpop!(ctx, F64, >=) }

// --- Arithmetic Handlers ---
fn handle_i32_clz(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_i32()?; // Add ?
    let result = x.leading_zeros() as i32;
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_ctz(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_i32()?; // Add ?
    let result = x.trailing_zeros() as i32;
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_popcnt(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_i32()?; // Add ?
    let result = x.count_ones() as i32;
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_add(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop_wrapping!(ctx, I32, wrapping_add) }
fn handle_i32_sub(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop_wrapping!(ctx, I32, wrapping_sub) }
fn handle_i32_mul(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop_wrapping!(ctx, I32, wrapping_mul) }
fn handle_i32_div_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?; // Add ?
    let lhs = lhs_val.to_i32()?; // Add ?
    if rhs == 0 { return Err(RuntimeError::ZeroDivideError); }
    if lhs == i32::MIN && rhs == -1 { return Err(RuntimeError::IntegerOverflow); }
    let result = lhs / rhs;
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_div_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?; // Add ?
    let lhs = lhs_val.to_i32()?; // Add ?
    let rhs_u = rhs as u32; // Cast after ?
    let lhs_u = lhs as u32; // Cast after ?
    if rhs_u == 0 { return Err(RuntimeError::ZeroDivideError); }
    let result = lhs_u / rhs_u;
    ctx.value_stack.push(Val::Num(Num::I32(result as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_rem_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?; 
    let lhs = lhs_val.to_i32()?; 
    if rhs == 0 { return Err(RuntimeError::ZeroDivideError); } 
    let result = lhs.overflowing_rem(rhs).0; // Call method after ? on lhs
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_rem_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?; // Add ?
    let lhs = lhs_val.to_i32()?; // Add ?
    let rhs_u = rhs as u32; // Cast after ?
    let lhs_u = lhs as u32; // Cast after ?
    if rhs_u == 0 { return Err(RuntimeError::ZeroDivideError); }
    let result = lhs_u % rhs_u;
    ctx.value_stack.push(Val::Num(Num::I32(result as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_and(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop!(ctx, I32, BitAnd, bitand) }
fn handle_i32_or(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop!(ctx, I32, BitOr, bitor) }
fn handle_i32_xor(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop!(ctx, I32, BitXor, bitxor) }
fn handle_i32_shl(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?; // Add ?
    let lhs = lhs_val.to_i32()?; // Add ?
    let result = lhs.wrapping_shl(rhs as u32); // Cast rhs after ?
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_shr_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?; // Add ?
    let lhs = lhs_val.to_i32()?; // Add ?
    let result = lhs.wrapping_shr(rhs as u32); // Cast rhs after ?
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_shr_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?; // Add ?
    let lhs = lhs_val.to_i32()?; // Add ?
    let lhs_u = lhs as u32; // Cast lhs after ?
    let rhs_u = rhs as u32; // Cast rhs after ?
    let result = lhs_u.wrapping_shr(rhs_u);
    ctx.value_stack.push(Val::Num(Num::I32(result as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_rotl(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?; 
    let lhs = lhs_val.to_i32()?; 
    let result = lhs.rotate_left(rhs as u32); // Call method after ? on lhs
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i32_rotr(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i32()?; 
    let lhs = lhs_val.to_i32()?; 
    let result = lhs.rotate_right(rhs as u32); // Call method after ? on lhs
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_clz(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_i64()?; // Add ?
    let result = x.leading_zeros() as i64;
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_ctz(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_i64()?; // Add ?
    let result = x.trailing_zeros() as i64;
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_popcnt(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_i64()?; // Add ?
    let result = x.count_ones() as i64;
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_add(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop_wrapping!(ctx, I64, wrapping_add) }
fn handle_i64_sub(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop_wrapping!(ctx, I64, wrapping_sub) }
fn handle_i64_mul(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop_wrapping!(ctx, I64, wrapping_mul) }
fn handle_i64_div_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?; // Add ?
    let lhs = lhs_val.to_i64()?; // Add ?
    if rhs == 0 { return Err(RuntimeError::ZeroDivideError); }
    if lhs == i64::MIN && rhs == -1 { return Err(RuntimeError::IntegerOverflow); }
    let result = lhs / rhs;
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_div_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?; // Add ?
    let lhs = lhs_val.to_i64()?; // Add ?
    let rhs_u = rhs as u64; // Cast after ?
    let lhs_u = lhs as u64; // Cast after ?
    if rhs_u == 0 { return Err(RuntimeError::ZeroDivideError); }
    let result = lhs_u / rhs_u;
    ctx.value_stack.push(Val::Num(Num::I64(result as i64)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_rem_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?; 
    let lhs = lhs_val.to_i64()?; 
    if rhs == 0 { return Err(RuntimeError::ZeroDivideError); }
    let result = lhs.overflowing_rem(rhs).0; // Call method after ? on lhs
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_rem_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?; // Add ?
    let lhs = lhs_val.to_i64()?; // Add ?
    let rhs_u = rhs as u64; // Cast after ?
    let lhs_u = lhs as u64; // Cast after ?
    if rhs_u == 0 { return Err(RuntimeError::ZeroDivideError); }
    let result = lhs_u % rhs_u;
    ctx.value_stack.push(Val::Num(Num::I64(result as i64)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_and(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop!(ctx, I64, BitAnd, bitand) }
fn handle_i64_or(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop!(ctx, I64, BitOr, bitor) }
fn handle_i64_xor(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop!(ctx, I64, BitXor, bitxor) }
fn handle_i64_shl(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?; // Add ? (Wasm uses i64 for shift amount)
    let lhs = lhs_val.to_i64()?; // Add ?
    let result = lhs.wrapping_shl(rhs as u32); // Cast rhs after ?
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_shr_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?; // Add ?
    let lhs = lhs_val.to_i64()?; // Add ?
    let result = lhs.wrapping_shr(rhs as u32); // Cast rhs after ?
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_shr_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?; // Add ?
    let lhs = lhs_val.to_i64()?; // Add ?
    let rhs_u = rhs as u64; // Cast rhs after ?
    let lhs_u = lhs as u64; // Cast lhs after ?
    let result = lhs_u.wrapping_shr(rhs_u as u32); // Cast rhs_u after definition
    ctx.value_stack.push(Val::Num(Num::I64(result as i64)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_rotl(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?; 
    let lhs = lhs_val.to_i64()?; 
    let result = lhs.rotate_left(rhs as u32); // Call method after ? on lhs
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_i64_rotr(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_i64()?; 
    let lhs = lhs_val.to_i64()?; 
    let result = lhs.rotate_right(rhs as u32); // Call method after ? on lhs
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f32_abs(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f32()?; // Add ?
    let result = x.abs();
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_neg(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f32()?; // Add ?
    let result = -x;
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_ceil(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f32()?; // Add ?
    let result = x.ceil();
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_floor(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f32()?; // Add ?
    let result = x.floor();
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_trunc(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f32()?; // Add ?
    let result = x.trunc();
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_nearest(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f32()?; // Add ?
    let y = x.fract();
    let result = if y == 0.5 { x.floor() + 1.0 } else if y == -0.5 { x.ceil() - 1.0 } else { x.round() };
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_sqrt(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f32()?; // Add ?
    let result = x.sqrt();
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_add(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop!(ctx, F32, Add, add) }
fn handle_f32_sub(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop!(ctx, F32, Sub, sub) }
fn handle_f32_mul(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop!(ctx, F32, Mul, mul) } 
fn handle_f32_div(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop!(ctx, F32, Div, div) } 
fn handle_f32_min(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_f32()?; 
    let lhs = lhs_val.to_f32()?; 
    let result = lhs.min(rhs); // Call method after ? on lhs
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_max(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_f32()?; 
    let lhs = lhs_val.to_f32()?; 
    let result = lhs.max(rhs); // Call method after ? on lhs
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f32_copysign(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_f32()?; 
    let lhs = lhs_val.to_f32()?; 
    let result = lhs.copysign(rhs); // Call method after ? on lhs
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_abs(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f64()?; // Add ?
    let result = x.abs(); 
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f64_neg(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f64()?; // Add ?
    let result = -x;
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f64_ceil(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f64()?; // Add ?
    let result = x.ceil();
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f64_floor(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f64()?; // Add ?
    let result = x.floor();
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f64_trunc(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f64()?; // Add ?
    let result = x.trunc();
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}
fn handle_f64_nearest(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f64()?; // Add ?
    let y = x.fract();
    let result = if y == 0.5 { x.floor() + 1.0 } else if y == -0.5 { x.ceil() - 1.0 } else { x.round() };
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_extend8_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_i32 = val_opt.to_i32()?; 
    let result = (val_i32 as i8) as i32; // Cast val_i32 *after* ?
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_extend16_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_i32 = val_opt.to_i32()?; 
    let result = (val_i32 as i16) as i32; // Cast val_i32 *after* ?
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_extend8_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_i64 = val_opt.to_i64()?; 
    let result = (val_i64 as i8) as i64; // Cast val_i64 *after* ?
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_extend16_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_i64 = val_opt.to_i64()?; 
    let result = (val_i64 as i16) as i64; // Cast val_i64 *after* ?
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_extend32_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_i32 = val_opt.to_i32()?; 
    let result = val_i32 as i64; // Cast val_i32 *after* ?
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_wrap_i64(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_i64 = val_opt.to_i64()?; 
    let result = val_i64 as i32; // Cast val_i64 *after* ?
    ctx.value_stack.push(Val::Num(Num::I32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_extend_i32_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_i32 = val_opt.to_i32()?; 
    let result = (val_i32 as u32) as i64; // Cast val_i32 *after* ?
    ctx.value_stack.push(Val::Num(Num::I64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_promote_f32(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f32 = val_opt.to_f32()?; 
    let result = val_f32 as f64; // Cast val_f32 *after* ?
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f32_demote_f64(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f64 = val_opt.to_f64()?; 
    let result = val_f64 as f32; // Cast val_f64 *after* ?
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_trunc_f32_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f32 = val_opt.to_f32()?; 
    // Check for NaN or out-of-range values
    if val_f32.is_nan() { return Err(RuntimeError::InvalidConversionToInt); } // Call method after ? on val_f32
    let truncated = val_f32.trunc(); // Call method after ? on val_f32
    if !(truncated >= (i32::MIN as f32) && truncated < ((i32::MAX as u32 + 1) as f32)) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.value_stack.push(Val::Num(Num::I32(truncated as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_trunc_f32_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f32 = val_opt.to_f32()?; 
    // Check for NaN or out-of-range values
    if val_f32.is_nan() { return Err(RuntimeError::InvalidConversionToInt); } // Call method after ? on val_f32
    let truncated = val_f32.trunc(); // Call method after ? on val_f32
    if !(truncated >= 0.0 && truncated < 4294967296.0) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.value_stack.push(Val::Num(Num::I32(truncated as u32 as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_trunc_f64_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f64 = val_opt.to_f64()?; 
    // Check for NaN or out-of-range values
    if val_f64.is_nan() { return Err(RuntimeError::InvalidConversionToInt); } // Call method after ? on val_f64
    let truncated = val_f64.trunc(); // Call method after ? on val_f64
    if !(truncated >= (i32::MIN as f64) && truncated < 2147483648.0) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.value_stack.push(Val::Num(Num::I32(truncated as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_trunc_f64_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f64 = val_opt.to_f64()?; 
    // Check for NaN or out-of-range values
    if val_f64.is_nan() { return Err(RuntimeError::InvalidConversionToInt); } // Call method after ? on val_f64
    let truncated = val_f64.trunc(); // Call method after ? on val_f64
    if !(truncated >= 0.0 && truncated < 4294967296.0) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.value_stack.push(Val::Num(Num::I32(truncated as u32 as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_trunc_f32_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f32 = val_opt.to_f32()?; 
    // Check for NaN or out-of-range values
    if val_f32.is_nan() { return Err(RuntimeError::InvalidConversionToInt); } // Call method after ? on val_f32
    let truncated = val_f32.trunc(); // Call method after ? on val_f32
    // Check range carefully for i64 from f32
    if !(truncated > (i64::MIN as f32) && truncated < (i64::MAX as f32)) { 
         if truncated.is_finite() && truncated.is_sign_positive() {
             // Check upper bound (2^63)
             if truncated >= 9223372036854775808.0_f32 { return Err(RuntimeError::IntegerOverflow); }
         } else if truncated.is_finite() && truncated.is_sign_negative() {
             // Check lower bound (-2^63)
             if truncated <= -9223372036854775808.0_f32 { return Err(RuntimeError::IntegerOverflow); }
         } else {
             // NaN or Infinity
             return Err(RuntimeError::IntegerOverflow);
         }
    }
    ctx.value_stack.push(Val::Num(Num::I64(truncated as i64)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_trunc_f32_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f32 = val_opt.to_f32()?; 
    // Check for NaN or out-of-range values
    if val_f32.is_nan() { return Err(RuntimeError::InvalidConversionToInt); } // Call method after ? on val_f32
    let truncated = val_f32.trunc(); // Call method after ? on val_f32
    // Check range 0 to 2^64
    if !(truncated > -1.0 && truncated < 18446744073709551616.0_f32) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.value_stack.push(Val::Num(Num::I64(truncated as u64 as i64)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_trunc_f64_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f64 = val_opt.to_f64()?; 
    // Check for NaN or out-of-range values
    if val_f64.is_nan() { return Err(RuntimeError::InvalidConversionToInt); } // Call method after ? on val_f64
    let truncated = val_f64.trunc(); // Call method after ? on val_f64
    // Check range -2^63 to 2^63
    if !(truncated > (i64::MIN as f64) && truncated < 9223372036854775808.0) { 
         if truncated.is_finite() && truncated.is_sign_positive() {
             if truncated >= 9223372036854775808.0 { return Err(RuntimeError::IntegerOverflow); }
         } else if truncated.is_finite() && truncated.is_sign_negative() {
             if truncated <= -9223372036854775808.0 { return Err(RuntimeError::IntegerOverflow); }
         } else {
             return Err(RuntimeError::IntegerOverflow);
         }
    }
    ctx.value_stack.push(Val::Num(Num::I64(truncated as i64)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_trunc_f64_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f64 = val_opt.to_f64()?; 
    // Check for NaN or out-of-range values
    if val_f64.is_nan() { return Err(RuntimeError::InvalidConversionToInt); } // Call method after ? on val_f64
    let truncated = val_f64.trunc(); // Call method after ? on val_f64
    // Check range 0 to 2^64
    if !(truncated > -1.0 && truncated < 18446744073709551616.0) {
        return Err(RuntimeError::IntegerOverflow);
    }
    ctx.value_stack.push(Val::Num(Num::I64(truncated as u64 as i64)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_unimplemented(_ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    println!("Error: Unimplemented instruction reached with operand: {:?}", operand);
    Err(RuntimeError::UnimplementedInstruction)
}

fn handle_br_table(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let i_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let i = i_val.to_i32()?; // Ensure ? is applied

    if let Operand::BrTable { targets, default } = operand {
        let chosen_operand = if let Some(target_operand) = targets.get(i as usize) {
            target_operand // This is already an Operand::LabelIdx struct
        } else {
            default      // This is already an Operand::LabelIdx struct
        };

        // Now extract info from the chosen Operand::LabelIdx
        if let Operand::LabelIdx { target_ip, arity, target_label_stack_idx } = chosen_operand {
            // Check if target_ip is still the placeholder (fixup needed)
            if *target_ip == usize::MAX { 
                return Err(RuntimeError::ExecutionFailed("Branch fixup not done for BrTable target")); 
            }

            // Use arity and target_label_stack_idx from the chosen operand
            let values_to_push = ctx.pop_n_values(*arity)?; // Use arity from chosen operand

            Ok(HandlerResult::Branch { 
                target_ip: *target_ip, 
                target_label_stack_idx: *target_label_stack_idx, // Use index from chosen operand
                values_to_push 
            })
        } else {
             // This should not happen if parser ensures BrTable targets are LabelIdx
            Err(RuntimeError::InvalidOperand) 
        }
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_call_indirect(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::TypeIdx(expected_type_idx) = operand {
        let table_idx = 0; // Assuming table 0 for now
        let i_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let i = i_val.to_i32()?; // Ensure ? is applied
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        let table_addr = module_inst.table_addrs.get(table_idx).ok_or(RuntimeError::TableNotFound)?;

        let func_ref_option = table_addr.get(i as usize); // Cast i after using ?

        if let Some(func_addr) = func_ref_option {
             let actual_type = func_addr.func_type();
             let expected_type = module_inst.types.get_by_idx(expected_type_idx.clone()); // Clone expected_type_idx

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

fn handle_select(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let cond_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let cond = cond_val.to_i32()?; // Ensure ? is applied
    let val2 = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val1 = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;

    if cond != 0 { // Compare after using ?
        ctx.value_stack.push(val1);
    } else {
        ctx.value_stack.push(val2);
    }
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_return(_ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    Ok(HandlerResult::Return)
}

fn handle_drop(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let _ = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_local_get(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::LocalIdx(LocalIdx(index_val)) = operand {
        let index = index_val as usize;
        if index >= ctx.frame.locals.len() {
            return Err(RuntimeError::LocalIndexOutOfBounds);
        }
        let val = ctx.frame.locals[index].clone();
        ctx.value_stack.push(val);
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_local_set(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::LocalIdx(LocalIdx(index_val)) = operand {
        let index = index_val as usize;
        if index >= ctx.frame.locals.len() {
            return Err(RuntimeError::LocalIndexOutOfBounds);
        }
        let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        ctx.frame.locals[index] = val;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_local_tee(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::LocalIdx(LocalIdx(index_val)) = operand {
        let index = index_val as usize;
        if index >= ctx.frame.locals.len() {
            return Err(RuntimeError::LocalIndexOutOfBounds);
        }
        let val = ctx.value_stack.last().ok_or(RuntimeError::ValueStackUnderflow)?.clone(); // Use last() for tee
        ctx.frame.locals[index] = val; // No pop for tee
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_global_get(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::GlobalIdx(GlobalIdx(index_val)) = operand {
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        let global_addr = module_inst.global_addrs.get_by_idx(GlobalIdx(index_val)).clone(); // Reconstruct GlobalIdx
        ctx.value_stack.push(global_addr.get());
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_global_set(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let &Operand::GlobalIdx(GlobalIdx(index_val)) = operand {
        let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        let global_addr = module_inst.global_addrs.get_by_idx(GlobalIdx(index_val)).clone();
        global_addr.set(val)?;
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
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

fn handle_i64_load(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
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

fn handle_f32_load(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
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

fn handle_f64_load(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
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

fn handle_i32_load8_s(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i8 = mem_addr.load::<i8>(&arg, ptr)?;
        let val_i32 = val_i8 as i32;
        ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load8_u(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u8 = mem_addr.load::<u8>(&arg, ptr)?;
        let val_i32 = val_u8 as i32;
        ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load16_s(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i16 = mem_addr.load::<i16>(&arg, ptr)?;
        let val_i32 = val_i16 as i32;
        ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_load16_u(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u16 = mem_addr.load::<u16>(&arg, ptr)?;
        let val_i32 = val_u16 as i32;
        ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load8_s(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i8 = mem_addr.load::<i8>(&arg, ptr)?;
        let val_i64 = val_i8 as i64;
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load8_u(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u8 = mem_addr.load::<u8>(&arg, ptr)?;
        let val_i64 = val_u8 as i64;
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load16_s(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i16 = mem_addr.load::<i16>(&arg, ptr)?;
        let val_i64 = val_i16 as i64;
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load16_u(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u16 = mem_addr.load::<u16>(&arg, ptr)?;
        let val_i64 = val_u16 as i64;
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load32_s(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_i32 = mem_addr.load::<i32>(&arg, ptr)?;
        let val_i64 = val_i32 as i64;
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_load32_u(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        let val_u32 = mem_addr.load::<u32>(&arg, ptr)?;
        let val_i64 = val_u32 as i64;
        ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_store(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() {
            return Err(RuntimeError::MemoryNotFound);
        }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i32>(&arg, ptr, val.to_i32()?)?; // Add ? to value conversion
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_store(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i64>(&arg, ptr, val.to_i64()?)?; // Add ? to value conversion
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f32_store(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; // Add ?
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        // Use val.to_f32()? instead of undefined 'data'
        mem_addr.store::<f32>(&arg, ptr, val.to_f32()?)?; 
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f64_store(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?;
        let data = val.to_f64()?; // Use ? here to get f64
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<f64>(&arg, ptr, data)?; // Pass ptr and data (values after ?) 
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_store8(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let val_i32 = val.to_i32()?; 
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; 
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i8>(&arg, ptr, val_i32 as i8)?; // Ensure ptr and (val_i32 as i8) are correct types
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i32_store16(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let val_i32 = val.to_i32()?; 
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; 
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i16>(&arg, ptr, val_i32 as i16)?; // Ensure ptr and (val_i32 as i16) are correct types
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_store8(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let val_i64 = val.to_i64()?; 
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; 
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i8>(&arg, ptr, val_i64 as i8)?; // Ensure ptr and (val_i64 as i8) are correct types
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_store16(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let val_i64 = val.to_i64()?; 
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; 
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i16>(&arg, ptr, val_i64 as i16)?; // Ensure ptr and (val_i64 as i16) are correct types
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_i64_store32(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::MemArg(arg) = operand {
        let val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let val_i64 = val.to_i64()?; 
        let ptr_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
        let ptr = ptr_val.to_i32()?; 
        let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
        if module_inst.mem_addrs.is_empty() { return Err(RuntimeError::MemoryNotFound); }
        let mem_addr = &module_inst.mem_addrs[0];
        mem_addr.store::<i32>(&arg, ptr, val_i64 as i32)?; // Ensure ptr and (val_i64 as i32) are correct types
        Ok(HandlerResult::Continue(ctx.ip + 1))
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

fn handle_f64_sqrt(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let x_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let x = x_val.to_f64()?; 
    let result = x.sqrt(); // Call method after ? on x
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_add(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop!(ctx, F64, Add, add) }
fn handle_f64_sub(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop!(ctx, F64, Sub, sub) }
fn handle_f64_mul(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop!(ctx, F64, Mul, mul) }
fn handle_f64_div(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> { binop!(ctx, F64, Div, div) }

fn handle_f64_min(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_f64()?; 
    let lhs = lhs_val.to_f64()?; 
    let result = lhs.min(rhs); // Call method after ? on lhs
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_max(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_f64()?; 
    let lhs = lhs_val.to_f64()?; 
    let result = lhs.max(rhs); // Call method after ? on lhs
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_copysign(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let rhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let lhs_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let rhs = rhs_val.to_f64()?; 
    let lhs = lhs_val.to_f64()?; 
    let result = lhs.copysign(rhs); // Call method after ? on lhs
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_memory_size(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
    if module_inst.mem_addrs.is_empty() {
        return Err(RuntimeError::MemoryNotFound);
    }
    let mem_addr = &module_inst.mem_addrs[0];
    let size = mem_addr.mem_size();
    ctx.value_stack.push(Val::Num(Num::I32(size as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_memory_grow(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let delta_val = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let delta = delta_val.to_i32()?; // Add ?
    let module_inst = ctx.frame.module.upgrade().ok_or(RuntimeError::ModuleInstanceGone)?;
    if module_inst.mem_addrs.is_empty() {
        return Err(RuntimeError::MemoryNotFound);
    }
    let mem_addr = &module_inst.mem_addrs[0]; 
    let prev_size = mem_addr.mem_grow((delta as u32).try_into().unwrap());
    ctx.value_stack.push(Val::Num(Num::I32(prev_size as i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f32_convert_i32_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i32()?; 
    let result = val as f32; // Cast val *after* ?
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f32_convert_i32_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i32()?; 
    let result = (val as u32) as f32; // Cast val *after* ?
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f32_convert_i64_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i64()?; 
    let result = val as f32; // Cast val *after* ?
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f32_convert_i64_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i64()?; 
    let result = (val as u64) as f32; // Cast val *after* ?
    ctx.value_stack.push(Val::Num(Num::F32(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_convert_i32_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i32()?; 
    let result = val as f64; // Cast val *after* ?
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_convert_i32_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i32()?; 
    let result = (val as u32) as f64; // Cast val *after* ?
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_convert_i64_s(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i64()?; 
    let result = val as f64; // Cast val *after* ?
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_convert_i64_u(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val = val_opt.to_i64()?; 
    let result = (val as u64) as f64; // Cast val *after* ?
    ctx.value_stack.push(Val::Num(Num::F64(result)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i32_reinterpret_f32(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f32 = val_opt.to_f32()?; 
    let val_i32 = unsafe { std::mem::transmute::<f32, i32>(val_f32) }; // Use val_f32 after ?
    ctx.value_stack.push(Val::Num(Num::I32(val_i32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_i64_reinterpret_f64(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_f64 = val_opt.to_f64()?; 
    let val_i64 = unsafe { std::mem::transmute::<f64, i64>(val_f64) }; // Use val_f64 after ?
    ctx.value_stack.push(Val::Num(Num::I64(val_i64)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f32_reinterpret_i32(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_i32 = val_opt.to_i32()?; 
    let val_f32 = unsafe { std::mem::transmute::<i32, f32>(val_i32) }; // Use val_i32 after ?
    ctx.value_stack.push(Val::Num(Num::F32(val_f32)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

fn handle_f64_reinterpret_i64(ctx: &mut ExecutionContext, _operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    let val_opt = ctx.value_stack.pop().ok_or(RuntimeError::ValueStackUnderflow)?;
    let val_i64 = val_opt.to_i64()?; 
    let val_f64 = unsafe { std::mem::transmute::<i64, f64>(val_i64) }; // Use val_i64 after ?
    ctx.value_stack.push(Val::Num(Num::F64(val_f64)));
    Ok(HandlerResult::Continue(ctx.ip + 1))
}

lazy_static! {
    static ref HANDLER_TABLE: Vec<HandlerFn> = {
        let mut table: Vec<HandlerFn> = vec![handle_unimplemented; MAX_HANDLER_INDEX];
        table[HANDLER_IDX_UNREACHABLE] = handle_unreachable;
        table[HANDLER_IDX_NOP] = handle_nop;
        table[HANDLER_IDX_BLOCK] = handle_block; // Needs update if it requires arity
        table[HANDLER_IDX_LOOP] = handle_loop;   // Needs update if it requires arity
        table[HANDLER_IDX_IF] = handle_if;       // Needs update if it requires arity
        table[HANDLER_IDX_ELSE] = handle_else;
        table[HANDLER_IDX_END] = handle_end;     // Might need update to handle block results?
        table[HANDLER_IDX_BR] = handle_br;       // Updated
        table[HANDLER_IDX_BR_IF] = handle_br_if;   // Updated
        table[HANDLER_IDX_BR_TABLE] = handle_br_table; // Updated
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
        table[HANDLER_IDX_I64_EXTEND_I32_S] = handle_i64_extend32_s; // Note: Name vs Index mismatch? Check spec. Assuming index is correct.
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
        table[HANDLER_IDX_I64_EXTEND32_S] = handle_i64_extend32_s; // Note: Name vs Index mismatch? Check spec. Assuming index is correct.

        table
    };
}

impl<'a> ExecutionContext<'a> {
    // Helper to pop N values from the value stack
    fn pop_n_values(&mut self, n: usize) -> Result<Vec<Val>, RuntimeError> {
        if self.value_stack.len() < n {
            Err(RuntimeError::ValueStackUnderflow)
        } else {
            Ok(self.value_stack.split_off(self.value_stack.len() - n))
        }
    }
}
