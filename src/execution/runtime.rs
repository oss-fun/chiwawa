use crate::error::RuntimeError;
use crate::execution::func::{FuncAddr, FuncInst};
use crate::execution::migration;
use crate::execution::module::ModuleInst;
use crate::execution::stack::{Frame, FrameStack, Label, LabelStack, ModuleLevelInstr, Stacks};
use crate::execution::value::{Num, Val, Vec_};
use crate::structure::types::{NumType, ValueType, VecType};
use std::path::Path;
use std::sync::Arc;

pub struct Runtime {
    module_inst: Arc<ModuleInst>,
    stacks: Stacks,
}

impl Runtime {
    pub fn new(
        module_inst: Arc<ModuleInst>,
        func_addr: &FuncAddr,
        params: Vec<Val>,
    ) -> Result<Self, RuntimeError> {
        let stacks = Stacks::new(func_addr, params)?;

        Ok(Runtime {
            module_inst,
            stacks,
        })
    }

    pub fn new_restored(module_inst: Arc<ModuleInst>, stacks: Stacks) -> Self {
        Runtime {
            module_inst,
            stacks,
        }
    }

    pub fn run(&mut self) -> Result<Vec<Val>, RuntimeError> {
        while !self.stacks.activation_frame_stack.is_empty() {
            let frame_stack_idx = self.stacks.activation_frame_stack.len() - 1;
            let mut called_func_addr: Option<FuncAddr> = None;

            let module_level_instr_result = {
                let current_frame_stack = &mut self.stacks.activation_frame_stack[frame_stack_idx];
                current_frame_stack.run_dtc_loop(&mut called_func_addr)?
            };

            match module_level_instr_result {
                Err(RuntimeError::CheckpointRequested) => {
                    println!("Runtime handling checkpoint request...");
                    let checkpoint_path = Path::new("./checkpoint.bin");
                    let mem_addrs = &self.module_inst.mem_addrs;
                    let global_addrs = &self.module_inst.global_addrs;
                    let table_addrs = &self.module_inst.table_addrs;

                    match migration::checkpoint(
                        &self.module_inst,
                        &self.stacks,
                        mem_addrs,
                        global_addrs,
                        table_addrs,
                        checkpoint_path,
                    ) {
                        Ok(_) => {
                            println!("Checkpoint successful (Runtime).");
                            return Err(RuntimeError::CheckpointRequested);
                        }
                        Err(e) => {
                            eprintln!("Checkpoint failed during runtime handling: {:?}", e);
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    return Err(e);
                }

                Ok(instr_option) => {
                    let current_frame_stack_mut =
                        self.stacks.activation_frame_stack.last_mut().unwrap();

                    if current_frame_stack_mut.label_stack.is_empty() {
                        return Err(RuntimeError::StackError(
                            "Label stack empty during frame transition",
                        ));
                    }
                    let cur_label_stack_mut =
                        current_frame_stack_mut.label_stack.last_mut().unwrap();

                    match instr_option {
                        Some(ModuleLevelInstr::Invoke(func_addr)) => {
                            let func_inst_guard = func_addr.read_lock().expect("RwLock poisoned");
                            match &*func_inst_guard {
                                FuncInst::RuntimeFunc {
                                    type_,
                                    module: func_module_weak,
                                    code,
                                } => {
                                    let params_len = type_.params.len();
                                    if cur_label_stack_mut.value_stack.len() < params_len {
                                        return Err(RuntimeError::ValueStackUnderflow);
                                    }
                                    let params = cur_label_stack_mut.value_stack.split_off(
                                        cur_label_stack_mut.value_stack.len() - params_len,
                                    );

                                    let mut locals = params;
                                    for v in code.locals.iter() {
                                        for _ in 0..(v.0) {
                                            locals.push(Val::default_value(&v.1)?);
                                        }
                                    }

                                    let new_frame = FrameStack {
                                        frame: Frame {
                                            locals,
                                            module: func_module_weak.clone(),
                                            n: type_.results.len(),
                                        },
                                        label_stack: vec![LabelStack {
                                            label: Label {
                                                locals_num: type_.results.len(),
                                                arity: type_.results.len(),
                                            },
                                            processed_instrs: code.body.clone(),
                                            value_stack: vec![],
                                            ip: 0,
                                        }],
                                        void: type_.results.is_empty(),
                                        instruction_count: 0,
                                    };
                                    self.stacks.activation_frame_stack.push(new_frame);
                                }
                                FuncInst::HostFunc { type_, host_code } => {
                                    let params_len = type_.params.len();
                                    if cur_label_stack_mut.value_stack.len() < params_len {
                                        return Err(RuntimeError::ValueStackUnderflow);
                                    }
                                    let params = cur_label_stack_mut.value_stack.split_off(
                                        cur_label_stack_mut.value_stack.len() - params_len,
                                    );
                                    match host_code(params) {
                                        Ok(results) => {
                                            cur_label_stack_mut.value_stack.extend(results);
                                            cur_label_stack_mut.ip += 1;
                                        }
                                        Err(e) => return Err(e),
                                    }
                                }
                            }
                        }
                        Some(ModuleLevelInstr::Return) => {
                            let finished_frame = self.stacks.activation_frame_stack.pop().unwrap();
                            let finished_label_stack = finished_frame.label_stack.last().ok_or(
                                RuntimeError::StackError("Finished frame has no label stack"),
                            )?;
                            let expected_n = finished_frame.frame.n;

                            if finished_label_stack.value_stack.len() < expected_n {
                                return Err(RuntimeError::Trap);
                            }
                            let values_to_pass = finished_label_stack
                                .value_stack
                                .iter()
                                .rev()
                                .take(expected_n)
                                .cloned()
                                .collect::<Vec<_>>()
                                .into_iter()
                                .rev()
                                .collect();

                            if self.stacks.activation_frame_stack.is_empty() {
                                return Ok(values_to_pass);
                            } else {
                                let caller_frame =
                                    self.stacks.activation_frame_stack.last_mut().unwrap();
                                let caller_label_stack = caller_frame
                                    .label_stack
                                    .last_mut()
                                    .ok_or(RuntimeError::StackError("Caller label stack empty"))?;
                                caller_label_stack.value_stack.extend(values_to_pass);
                                caller_label_stack.ip += 1;
                            }
                        }
                        None => {
                            let finished_frame = self.stacks.activation_frame_stack.pop().unwrap();
                            let finished_label_stack = finished_frame.label_stack.last().ok_or(
                                RuntimeError::StackError("Finished frame has no label stack"),
                            )?;
                            let expected_n = finished_frame.frame.n;

                            if finished_label_stack.value_stack.len() < expected_n {
                                return Err(RuntimeError::Trap);
                            }
                            let values_to_pass = finished_label_stack
                                .value_stack
                                .iter()
                                .rev()
                                .take(expected_n)
                                .cloned()
                                .collect::<Vec<_>>()
                                .into_iter()
                                .rev()
                                .collect();

                            if self.stacks.activation_frame_stack.is_empty() {
                                return Ok(values_to_pass);
                            } else {
                                let caller_frame =
                                    self.stacks.activation_frame_stack.last_mut().unwrap();
                                let caller_label_stack = caller_frame
                                    .label_stack
                                    .last_mut()
                                    .ok_or(RuntimeError::StackError("Caller label stack empty"))?;
                                caller_label_stack.value_stack.extend(values_to_pass);
                            }
                        }
                    }
                }
            }
        }
        Ok(vec![])
    }
}

impl Val {
    fn default_value(value_type: &ValueType) -> Result<Val, RuntimeError> {
        match value_type {
            ValueType::NumType(NumType::I32) => Ok(Val::Num(Num::I32(0))),
            ValueType::NumType(NumType::I64) => Ok(Val::Num(Num::I64(0))),
            ValueType::NumType(NumType::F32) => Ok(Val::Num(Num::F32(0.0))),
            ValueType::NumType(NumType::F64) => Ok(Val::Num(Num::F64(0.0))),
            ValueType::VecType(VecType::V128) => Ok(Val::Vec_(Vec_::V128(0))),
            ValueType::RefType(_) => todo!("Default value for RefType"),
        }
    }
}
