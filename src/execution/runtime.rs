use crate::error::RuntimeError;
use crate::execution::func::{FuncAddr, FuncInst};
use crate::execution::migration;
use crate::execution::module::ModuleInst;
use crate::execution::stack::{Frame, FrameStack, Label, LabelStack, ModuleLevelInstr, Stacks};
use crate::execution::value::{Num, Val, Vec_};
use crate::structure::module::WasiFuncType;
use crate::structure::types::{NumType, ValueType, VecType};
use crate::wasi::{WasiError, WasiResult};
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
                                FuncInst::WasiFunc {
                                    type_,
                                    wasi_func_addr,
                                } => {
                                    let params_len = type_.params.len();
                                    if cur_label_stack_mut.value_stack.len() < params_len {
                                        return Err(RuntimeError::ValueStackUnderflow);
                                    }
                                    let params = cur_label_stack_mut.value_stack.split_off(
                                        cur_label_stack_mut.value_stack.len() - params_len,
                                    );

                                    let wasi_func_type = wasi_func_addr.func_type.clone();
                                    drop(func_inst_guard); // Release the lock before calling WASI function

                                    // Call WASI function
                                    match self.call_wasi_function(&wasi_func_type, params) {
                                        Ok(result) => {
                                            if let Some(val) = result {
                                                let current_frame_stack_mut = self
                                                    .stacks
                                                    .activation_frame_stack
                                                    .last_mut()
                                                    .unwrap();
                                                let cur_label_stack_mut = current_frame_stack_mut
                                                    .label_stack
                                                    .last_mut()
                                                    .unwrap();
                                                cur_label_stack_mut.value_stack.push(val);
                                            }
                                            let current_frame_stack_mut = self
                                                .stacks
                                                .activation_frame_stack
                                                .last_mut()
                                                .unwrap();
                                            let cur_label_stack_mut = current_frame_stack_mut
                                                .label_stack
                                                .last_mut()
                                                .unwrap();
                                            cur_label_stack_mut.ip += 1;
                                        }
                                        Err(WasiError::ProcessExit(_code)) => {
                                            return Err(RuntimeError::ExecutionFailed(
                                                "Process exit",
                                            ));
                                        }
                                        Err(_e) => {
                                            return Err(RuntimeError::ExecutionFailed(
                                                "WASI function failed",
                                            ));
                                        }
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

    fn call_wasi_function(
        &self,
        func_type: &WasiFuncType,
        params: Vec<Val>,
    ) -> WasiResult<Option<Val>> {
        let wasi_impl = self
            .module_inst
            .wasi_impl
            .as_ref()
            .ok_or(WasiError::NotImplemented)?;

        // Get memory address for WASI functions that need it
        let memory = if self.module_inst.mem_addrs.is_empty() {
            return Err(WasiError::MemoryAccessError);
        } else {
            &self.module_inst.mem_addrs[0]
        };

        match func_type {
            WasiFuncType::FdWrite => {
                if params.len() != 4 {
                    return Err(WasiError::InvalidArgument);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::InvalidArgument)?;
                let iovs_ptr = params[1].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;
                let iovs_len = params[2].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;
                let nwritten_ptr =
                    params[3].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;

                let result = wasi_impl.fd_write(memory, fd, iovs_ptr, iovs_len, nwritten_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdRead => {
                if params.len() != 4 {
                    return Err(WasiError::InvalidArgument);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::InvalidArgument)?;
                let iovs_ptr = params[1].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;
                let iovs_len = params[2].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;
                let nread_ptr = params[3].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;

                let result = wasi_impl.fd_read(memory, fd, iovs_ptr, iovs_len, nread_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::ProcExit => {
                if params.len() != 1 {
                    return Err(WasiError::InvalidArgument);
                }
                let exit_code = params[0].to_i32().map_err(|_| WasiError::InvalidArgument)?;
                wasi_impl.proc_exit(exit_code)?;
                Ok(None) // This should never be reached due to ProcessExit error
            }
            WasiFuncType::RandomGet => {
                if params.len() != 2 {
                    return Err(WasiError::InvalidArgument);
                }
                let buf_ptr = params[0].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;
                let buf_len = params[1].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;

                let result = wasi_impl.random_get(memory, buf_ptr, buf_len)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdClose => {
                if params.len() != 1 {
                    return Err(WasiError::InvalidArgument);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::InvalidArgument)?;

                let result = wasi_impl.fd_close(fd)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::EnvironGet => {
                if params.len() != 2 {
                    return Err(WasiError::InvalidArgument);
                }
                let environ_ptr =
                    params[0].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;
                let environ_buf_ptr =
                    params[1].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;

                let result = wasi_impl.environ_get(memory, environ_ptr, environ_buf_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::EnvironSizesGet => {
                if params.len() != 2 {
                    return Err(WasiError::InvalidArgument);
                }
                let environ_count_ptr =
                    params[0].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;
                let environ_buf_size_ptr =
                    params[1].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;

                let result =
                    wasi_impl.environ_sizes_get(memory, environ_count_ptr, environ_buf_size_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::ArgsGet => {
                if params.len() != 2 {
                    return Err(WasiError::InvalidArgument);
                }
                let argv_ptr = params[0].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;
                let argv_buf_ptr =
                    params[1].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;

                let result = wasi_impl.args_get(memory, argv_ptr, argv_buf_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::ArgsSizesGet => {
                if params.len() != 2 {
                    return Err(WasiError::InvalidArgument);
                }
                let argc_ptr = params[0].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;
                let argv_buf_size_ptr =
                    params[1].to_i32().map_err(|_| WasiError::InvalidArgument)? as u32;

                let result = wasi_impl.args_sizes_get(memory, argc_ptr, argv_buf_size_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            _ => Err(WasiError::NotImplemented),
        }
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
