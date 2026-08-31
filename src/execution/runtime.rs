//! Runtime core managing execution lifecycle and host function invocation.

use crate::error::RuntimeError;
use crate::execution::dispatch;
use crate::execution::func::{FuncAddr, FuncInst};
use crate::execution::ir::Outcome;
use crate::execution::mem::MemAddr;
use crate::execution::migration;
use crate::execution::module::ModuleInst;
use crate::execution::regs::{Reg, RegFile};
use crate::execution::state::VmState;
use crate::execution::state::{Frame, FrameStack, ModuleLevelInstr, Stacks};
use crate::execution::stats::ExecutionStats;
#[cfg(feature = "trace")]
use crate::execution::trace::{TraceConfig, Tracer};
use crate::execution::value::{Num, Val};
use crate::structure::module::{Func, WasiFuncType};
use crate::wasi::{WasiError, WasiResult};
use arrayvec::ArrayVec;
use std::path::Path;
use std::rc::Rc;
#[cfg(all(target_os = "wasi", target_env = "p1", target_feature = "atomics"))]
use std::sync::Once;

/// Execution entry point that manages the interpreter loop.
pub struct Runtime {
    module_inst: Rc<ModuleInst>,
    primary_mem: Option<MemAddr>,
    stacks: Stacks,
    checkpoint_poll_counter: u32,
    #[cfg_attr(not(feature = "stats"), allow(dead_code))]
    execution_stats: Option<ExecutionStats>,
    #[cfg(feature = "trace")]
    tracer: Option<Tracer>,
    #[cfg_attr(not(feature = "stats"), allow(dead_code))]
    enable_stats: bool,
    enable_checkpoint: bool,
}

impl Drop for Runtime {
    fn drop(&mut self) {
        #[cfg(feature = "stats")]
        if self.enable_stats {
            if let Some(ref stats) = self.execution_stats {
                stats.report();
            }
        }
    }
}

impl Runtime {
    /// Creates a new runtime for executing a function.
    pub fn new(
        module_inst: Rc<ModuleInst>,
        func_addr: &FuncAddr,
        params: Vec<Val>,
        enable_stats: bool,
        enable_checkpoint: bool,
        #[cfg(feature = "trace")] trace_config: Option<TraceConfig>,
    ) -> Result<Self, RuntimeError> {
        let stacks = Stacks::new(func_addr, params)?;

        #[cfg(feature = "trace")]
        let tracer = if let Some(config) = trace_config {
            match Tracer::new(config) {
                Ok(tracer) => Some(tracer),
                Err(e) => {
                    eprintln!("Failed to create tracer: {:?}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Runtime {
            primary_mem: module_inst.mem_addrs.first().cloned(),
            module_inst,
            stacks,
            checkpoint_poll_counter: 0,
            execution_stats: if enable_stats {
                Some(ExecutionStats::new())
            } else {
                None
            },
            #[cfg(feature = "trace")]
            tracer,
            enable_stats,
            enable_checkpoint,
        })
    }

    /// Creates a runtime restored from a checkpoint.
    ///
    /// Used to resume execution after restoring state from a checkpoint file.
    pub fn new_restored(
        module_inst: Rc<ModuleInst>,
        stacks: Stacks,
        enable_stats: bool,
        enable_checkpoint: bool,
        #[cfg(feature = "trace")] trace_config: Option<TraceConfig>,
    ) -> Self {
        #[cfg(feature = "trace")]
        let tracer = if let Some(config) = trace_config {
            match Tracer::new(config) {
                Ok(tracer) => Some(tracer),
                Err(e) => {
                    eprintln!("Failed to create tracer: {:?}", e);
                    None
                }
            }
        } else {
            None
        };

        Runtime {
            primary_mem: module_inst.mem_addrs.first().cloned(),
            module_inst,
            stacks,
            checkpoint_poll_counter: 0,
            execution_stats: if enable_stats {
                Some(ExecutionStats::new())
            } else {
                None
            },
            #[cfg(feature = "trace")]
            tracer,
            enable_stats,
            enable_checkpoint,
        }
    }

    /// Executes interpreter loop for a specific frame stack via the v2
    /// dispatcher (`dispatch::execute_instructions`). Constructs a `VmState`,
    /// runs dispatch, writes back state, and translates `Outcome` into the
    /// legacy result.
    fn execute_frame(
        &mut self,
        frame_stack_idx: usize,
        _called_func_addr: &mut Option<FuncAddr>,
    ) -> Result<Result<Option<ModuleLevelInstr>, RuntimeError>, RuntimeError> {
        let module_ptr: *const ModuleInst = Rc::as_ptr(&self.module_inst);
        let reg_file_ptr: *mut RegFile = &mut self.stacks.reg_file as *mut RegFile;

        // Body and handlers stay owned by the module for its whole lifetime, so
        // the frame names its function by index rather than holding an `Rc`.
        let func_idx = self.stacks.activation_frame_stack[frame_stack_idx].func_idx;
        let (body_ptr, body_len, code_handlers_ptr, code_ptr) =
            match self.module_inst.func_addrs[func_idx as usize].read_lock() {
                FuncInst::RuntimeFunc { code, .. } => (
                    code.body.as_ptr(),
                    code.body.len(),
                    code.handlers.as_ptr(),
                    code as *const Func,
                ),
                _ => (std::ptr::null(), 0, std::ptr::null(), std::ptr::null()),
            };

        let frame_stack = &mut self.stacks.activation_frame_stack[frame_stack_idx];

        let (instrs_ptr, instrs_len, pc) = (body_ptr, body_len, frame_stack.ip);
        let handlers_ptr = {
            cfg_if::cfg_if! {
                if #[cfg(all(
                    target_arch = "wasm32",
                    target_os = "wasi",
                    target_env = "p1",
                    target_feature = "atomics"
                ))] {
                    match &frame_stack.handler_ctrl {
                        Some(ctrl) => ctrl.handlers_ptr(),
                        None => code_handlers_ptr,
                    }
                } else {
                    code_handlers_ptr
                }
            }
        };
        let mem_ptr = frame_stack.cached_mem_ptr.unwrap_or(std::ptr::null_mut());
        let return_result_regs_ptr: *mut ArrayVec<Reg, 8> =
            &mut frame_stack.return_result_regs as *mut ArrayVec<Reg, 8>;
        let enable_checkpoint = frame_stack.enable_checkpoint;

        #[cfg(feature = "stats")]
        let stats_ptr = self
            .execution_stats
            .as_mut()
            .map_or(std::ptr::null_mut(), |s| s as *mut ExecutionStats);

        #[cfg(feature = "trace")]
        let tracer_ptr = self
            .tracer
            .as_mut()
            .map_or(std::ptr::null_mut(), |t| t as *mut Tracer);

        let mut state = VmState {
            reg_file: reg_file_ptr,
            pc,
            instrs: instrs_ptr,
            instrs_len,
            handlers: handlers_ptr,
            mem_ptr,
            code: code_ptr,
            module: module_ptr,
            trap: None,
            yielded: None,
            return_result_regs: return_result_regs_ptr,
            enable_checkpoint,
            checkpoint_poll_counter: self.checkpoint_poll_counter,
            #[cfg(feature = "stats")]
            stats: stats_ptr,
            #[cfg(feature = "trace")]
            tracer: tracer_ptr,
        };

        let outcome = dispatch::execute_instructions(&mut state);

        self.checkpoint_poll_counter = state.checkpoint_poll_counter;
        frame_stack.ip = state.pc;
        frame_stack.cached_mem_ptr = if state.mem_ptr.is_null() {
            None
        } else {
            Some(state.mem_ptr)
        };

        match outcome {
            Outcome::Halt => Ok(Ok(None)),
            Outcome::Yield => Ok(Ok(state.yielded)),
            Outcome::Trap => {
                let err = state
                    .trap
                    .expect("Outcome::Trap returned without state.trap set");
                if matches!(err, RuntimeError::CheckpointRequested) {
                    Ok(Err(err))
                } else {
                    Err(err)
                }
            }
            Outcome::Continue => unreachable!("dispatcher must not return Continue"),
        }
    }

    /// Executes the runtime and returns the result values.
    pub fn run(&mut self) -> Result<Vec<Val>, RuntimeError> {
        // Setup checkpoint monitor thread (only for wasm32-wasip1-threads)
        #[cfg(all(
            target_arch = "wasm32",
            target_os = "wasi",
            target_env = "p1",
            target_feature = "atomics"
        ))]
        {
            if self.enable_checkpoint {
                static INIT: Once = Once::new();
                INIT.call_once(|| {
                    migration::setup_checkpoint_monitor();
                });
            }
        }

        // Set checkpoint enabled flag for initial frame stack
        #[cfg(all(
            target_arch = "wasm32",
            target_os = "wasi",
            target_env = "p1",
            target_feature = "atomics"
        ))]
        let entry_ctrl = if self.enable_checkpoint {
            let func_idx = self
                .stacks
                .activation_frame_stack
                .first()
                .map(|f| f.func_idx);
            match func_idx {
                Some(idx) => match self.module_inst.func_addrs[idx as usize].read_lock() {
                    FuncInst::RuntimeFunc { code, .. } => {
                        Some(migration::HandlerControl::new(&code.handlers))
                    }
                    _ => None,
                },
                None => None,
            }
        } else {
            None
        };
        if let Some(frame_stack) = self.stacks.activation_frame_stack.first_mut() {
            frame_stack.enable_checkpoint = self.enable_checkpoint;
            #[cfg(all(
                target_arch = "wasm32",
                target_os = "wasi",
                target_env = "p1",
                target_feature = "atomics"
            ))]
            if frame_stack.handler_ctrl.is_none() {
                frame_stack.handler_ctrl = entry_ctrl;
            }
        }

        while !self.stacks.activation_frame_stack.is_empty() {
            let frame_stack_idx = self.stacks.activation_frame_stack.len() - 1;
            let mut called_func_addr: Option<FuncAddr> = None;

            let module_level_instr_result =
                self.execute_frame(frame_stack_idx, &mut called_func_addr)?;

            match module_level_instr_result {
                Err(RuntimeError::CheckpointRequested) => {
                    println!("Runtime handling checkpoint request...");
                    let checkpoint_path = Path::new("./checkpoint.bin");
                    let mem_addrs = &self.module_inst.mem_addrs;
                    let global_addrs = &self.module_inst.global_addrs;

                    match migration::checkpoint(
                        &self.stacks,
                        mem_addrs,
                        global_addrs,
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
                    match instr_option {
                        Some(ModuleLevelInstr::InvokeWasiReg {
                            wasi_func_type,
                            params,
                            result_reg,
                        }) => {
                            // Call WASI function directly with params from registers
                            match self.call_wasi_function(&wasi_func_type, &params) {
                                Ok(result) => {
                                    if let Some(reg) = result_reg {
                                        if let Some(val) = result {
                                            self.stacks.reg_file.set_val(&reg, &val);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "WASI register function failed: {:?}, error: {:?}",
                                        wasi_func_type, e
                                    );
                                    return Err(RuntimeError::ExecutionFailed(
                                        "WASI register function failed",
                                    ));
                                }
                            }
                        }
                        Some(ModuleLevelInstr::InvokeReg {
                            func_addr,
                            param_regs,
                            result_regs,
                        }) => {
                            let func_inst_guard = func_addr.read_lock();
                            match &*func_inst_guard {
                                FuncInst::RuntimeFunc {
                                    type_,
                                    code,
                                    func_idx,
                                    #[cfg(all(
                                        target_arch = "wasm32",
                                        target_os = "wasi",
                                        target_env = "p1",
                                        target_feature = "atomics"
                                    ))]
                                        handler_ctrl: cached_ctrl,
                                    ..
                                } => {
                                    // Locals live in the register file: open the
                                    // callee's register window (zero-initializing
                                    // declared locals) and move the args into
                                    // their local slots.
                                    if let Some(ref alloc) = code.reg_allocation {
                                        self.stacks
                                            .reg_file
                                            .push_frame_with_params(alloc, param_regs);
                                    }

                                    // Store result_regs in caller frame
                                    if let Some(caller) =
                                        self.stacks.activation_frame_stack.last_mut()
                                    {
                                        caller.result_regs = result_regs.iter().copied().collect();
                                    }
                                    // Cache primary memory address and raw pointer
                                    let cached_mem_ptr =
                                        self.primary_mem.as_ref().map(|m| m.data_ptr());

                                    let new_frame = FrameStack {
                                        func_idx: *func_idx,
                                        frame: Frame {
                                            n: type_.results.len(),
                                        },
                                        ip: 0,
                                        enable_checkpoint: self.enable_checkpoint,
                                        result_regs: ArrayVec::new(),
                                        return_result_regs: ArrayVec::new(),
                                        cached_mem_ptr,
                                        #[cfg(all(
                                            target_arch = "wasm32",
                                            target_os = "wasi",
                                            target_env = "p1",
                                            target_feature = "atomics"
                                        ))]
                                        handler_ctrl: if self.enable_checkpoint {
                                            // Reuse this function's cached HandlerControl
                                            // (created once on its first call with --cr on).
                                            Some(std::sync::Arc::clone(cached_ctrl.get_or_init(
                                                || migration::HandlerControl::new(&code.handlers),
                                            )))
                                        } else {
                                            None
                                        },
                                    };
                                    self.stacks.activation_frame_stack.push(new_frame);
                                }
                                FuncInst::HostFunc { host_code, .. } => {
                                    let params: Vec<Val> = param_regs
                                        .iter()
                                        .map(|r| self.stacks.reg_file.get_val(r))
                                        .collect();
                                    match host_code(params) {
                                        Ok(results) => {
                                            // Write results directly to registers
                                            for (reg, val) in result_regs.iter().zip(results.iter())
                                            {
                                                self.stacks.reg_file.set_val(reg, val);
                                            }
                                        }
                                        Err(e) => return Err(e),
                                    }
                                }
                                FuncInst::WasiFunc { .. } => {
                                    return Err(RuntimeError::ExecutionFailed(
                                        "WASI function called via InvokeReg - use CallWasiReg",
                                    ));
                                }
                            }
                        }
                        Some(ModuleLevelInstr::Return) | None => {
                            // Pop register file frame but keep reference for reading return values
                            let finished_frame = self.stacks.activation_frame_stack.pop().unwrap();
                            let expected_n = finished_frame.frame.n;
                            let return_result_regs = finished_frame.return_result_regs;

                            if self.stacks.activation_frame_stack.is_empty() {
                                // Read values from registers before restoring
                                // Use ArrayVec to avoid heap allocation (most functions return 0-2 values)
                                let values_to_pass: ArrayVec<Val, 8> = return_result_regs
                                    .iter()
                                    .take(expected_n)
                                    .map(|reg| self.stacks.reg_file.get_val(reg))
                                    .collect();
                                self.stacks.reg_file.restore_offsets();
                                return Ok(values_to_pass.into_iter().collect());
                            } else {
                                // Refresh cached memory pointer (may have changed due to memory.grow in callee)
                                let mem_ptr = self.primary_mem.as_ref().map(|m| m.data_ptr());

                                let (reg_file, frames) = self.stacks.get_reg_file_and_frames();
                                let caller_frame = frames.last_mut().unwrap();
                                reg_file.pop_frame_with_results(
                                    &return_result_regs,
                                    &caller_frame.result_regs,
                                );
                                caller_frame.result_regs.clear();
                                caller_frame.cached_mem_ptr = mem_ptr;
                            }
                        }
                    }
                }
            }
        }
        Ok(vec![])
    }

    /// Calls a WASI function with the given parameters.
    fn call_wasi_function(
        &self,
        func_type: &WasiFuncType,
        params: &[Val],
    ) -> WasiResult<Option<Val>> {
        let wasi_impl = self
            .module_inst
            .wasi_impl
            .as_ref()
            .ok_or(WasiError::NoSys)?;

        // Get memory address for WASI functions that need it
        let memory = if self.module_inst.mem_addrs.is_empty() {
            return Err(WasiError::Fault);
        } else {
            &self.module_inst.mem_addrs[0]
        };

        match func_type {
            WasiFuncType::FdWrite => {
                if params.len() != 4 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let iovs_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let iovs_len = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let nwritten_ptr = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.fd_write(memory, fd, iovs_ptr, iovs_len, nwritten_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdRead => {
                if params.len() != 4 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let iovs_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let iovs_len = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let nread_ptr = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.fd_read(memory, fd, iovs_ptr, iovs_len, nread_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::ProcExit => {
                if params.len() != 1 {
                    return Err(WasiError::Inval);
                }
                let exit_code = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                wasi_impl.proc_exit(exit_code)?;
                Ok(None) // This should never be reached due to ProcessExit error
            }
            WasiFuncType::RandomGet => {
                if params.len() != 2 {
                    return Err(WasiError::Inval);
                }
                let buf_ptr = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let buf_len = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.random_get(memory, buf_ptr, buf_len)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdClose => {
                if params.len() != 1 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;

                let result = wasi_impl.fd_close(fd)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::EnvironGet => {
                if params.len() != 2 {
                    return Err(WasiError::Inval);
                }
                let environ_ptr = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let environ_buf_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.environ_get(memory, environ_ptr, environ_buf_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::EnvironSizesGet => {
                if params.len() != 2 {
                    return Err(WasiError::Inval);
                }
                let environ_count_ptr = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let environ_buf_size_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result =
                    wasi_impl.environ_sizes_get(memory, environ_count_ptr, environ_buf_size_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::ArgsGet => {
                if params.len() != 2 {
                    return Err(WasiError::Inval);
                }
                let argv_ptr = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let argv_buf_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.args_get(memory, argv_ptr, argv_buf_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::ArgsSizesGet => {
                if params.len() != 2 {
                    return Err(WasiError::Inval);
                }
                let argc_ptr = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let argv_buf_size_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.args_sizes_get(memory, argc_ptr, argv_buf_size_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::ClockTimeGet => {
                if params.len() != 3 {
                    return Err(WasiError::Inval);
                }
                let clock_id = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let precision = params[1].to_i64().map_err(|_| WasiError::Inval)?;
                let time_ptr = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.clock_time_get(memory, clock_id, precision, time_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::ClockResGet => {
                if params.len() != 2 {
                    return Err(WasiError::Inval);
                }
                let clock_id = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let resolution_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.clock_res_get(memory, clock_id, resolution_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdPrestatGet => {
                if params.len() != 2 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let prestat_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.fd_prestat_get(memory, fd, prestat_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdPrestatDirName => {
                if params.len() != 3 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let path_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let path_len = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.fd_prestat_dir_name(memory, fd, path_ptr, path_len)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::SchedYield => {
                if params.len() != 0 {
                    return Err(WasiError::Inval);
                }

                let result = wasi_impl.sched_yield()?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdFdstatGet => {
                if params.len() != 2 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let stat_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.fd_fdstat_get(memory, fd, stat_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::PathOpen => {
                if params.len() != 9 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let dirflags = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let path_ptr = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let path_len = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let oflags = params[4].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let fs_rights_base = params[5].to_i64().map_err(|_| WasiError::Inval)? as u64;
                let fs_rights_inheriting = params[6].to_i64().map_err(|_| WasiError::Inval)? as u64;
                let fdflags = params[7].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let opened_fd_ptr = params[8].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.path_open(
                    memory,
                    fd,
                    dirflags,
                    path_ptr,
                    path_len,
                    oflags,
                    fs_rights_base,
                    fs_rights_inheriting,
                    fdflags,
                    opened_fd_ptr,
                )?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdSeek => {
                if params.len() != 4 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let offset = params[1].to_i64().map_err(|_| WasiError::Inval)?;
                let whence = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let newoffset_ptr = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.fd_seek(&memory, fd, offset, whence, newoffset_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdTell => {
                if params.len() != 2 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let offset_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.fd_tell(memory, fd, offset_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdSync => {
                if params.len() != 1 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;

                let result = wasi_impl.fd_sync(fd)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdFilestatGet => {
                if params.len() != 2 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let filestat_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.fd_filestat_get(memory, fd, filestat_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdReaddir => {
                if params.len() != 5 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let buf_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let buf_len = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let cookie = params[3].to_i64().map_err(|_| WasiError::Inval)? as u64;
                let buf_used_ptr = params[4].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result =
                    wasi_impl.fd_readdir(memory, fd, buf_ptr, buf_len, cookie, buf_used_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdPread => {
                if params.len() != 5 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let iovs_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let iovs_len = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let offset = params[3].to_i64().map_err(|_| WasiError::Inval)? as u64;
                let nread_ptr = params[4].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result =
                    wasi_impl.fd_pread(memory, fd, iovs_ptr, iovs_len, offset, nread_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdDatasync => {
                if params.len() != 1 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;

                let result = wasi_impl.fd_datasync(fd)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdFdstatSetFlags => {
                if params.len() != 2 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let flags = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.fd_fdstat_set_flags(fd, flags)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdFilestatSetSize => {
                if params.len() != 2 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let size = params[1].to_i64().map_err(|_| WasiError::Inval)? as u64;

                let result = wasi_impl.fd_filestat_set_size(fd, size)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdPwrite => {
                if params.len() != 5 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let iovs_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let iovs_len = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let offset = params[3].to_i64().map_err(|_| WasiError::Inval)? as u64;
                let nwritten_ptr = params[4].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result =
                    wasi_impl.fd_pwrite(memory, fd, iovs_ptr, iovs_len, offset, nwritten_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::PathCreateDirectory => {
                if params.len() != 3 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let path_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let path_len = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.path_create_directory(memory, fd, path_ptr, path_len)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::PathFilestatGet => {
                if params.len() != 5 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let flags = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let path_ptr = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let path_len = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let filestat_ptr = params[4].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.path_filestat_get(
                    memory,
                    fd,
                    flags,
                    path_ptr,
                    path_len,
                    filestat_ptr,
                )?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::PathFilestatSetTimes => {
                if params.len() != 7 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let flags = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let path_ptr = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let path_len = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let atim = params[4].to_i64().map_err(|_| WasiError::Inval)? as u64;
                let mtim = params[5].to_i64().map_err(|_| WasiError::Inval)? as u64;
                let fst_flags = params[6].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.path_filestat_set_times(
                    memory, fd, flags, path_ptr, path_len, atim, mtim, fst_flags,
                )?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::PathReadlink => {
                if params.len() != 6 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let path_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let path_len = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let buf_ptr = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let buf_len = params[4].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let buf_used_ptr = params[5].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.path_readlink(
                    memory,
                    fd,
                    path_ptr,
                    path_len,
                    buf_ptr,
                    buf_len,
                    buf_used_ptr,
                )?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::PathRemoveDirectory => {
                if params.len() != 3 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let path_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let path_len = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.path_remove_directory(memory, fd, path_ptr, path_len)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::PathUnlinkFile => {
                if params.len() != 3 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let path_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let path_len = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.path_unlink_file(memory, fd, path_ptr, path_len)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::PollOneoff => {
                if params.len() != 4 {
                    return Err(WasiError::Inval);
                }
                let in_ptr = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let out_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let nsubscriptions = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let nevents_ptr = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result =
                    wasi_impl.poll_oneoff(memory, in_ptr, out_ptr, nsubscriptions, nevents_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdFilestatSetTimes => {
                if params.len() != 4 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                let atim = params[1].to_i64().map_err(|_| WasiError::Inval)? as u64;
                let mtim = params[2].to_i64().map_err(|_| WasiError::Inval)? as u64;
                let fst_flags = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result =
                    wasi_impl.fd_filestat_set_times(memory, fd as u32, atim, mtim, fst_flags)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::PathLink => {
                if params.len() != 7 {
                    return Err(WasiError::Inval);
                }
                let old_fd = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let old_flags = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let old_path_ptr = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let old_path_len = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let new_fd = params[4].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let new_path_ptr = params[5].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let new_path_len = params[6].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.path_link(
                    memory,
                    old_fd,
                    old_flags,
                    old_path_ptr,
                    old_path_len,
                    new_fd,
                    new_path_ptr,
                    new_path_len,
                )?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::PathRename => {
                if params.len() != 6 {
                    return Err(WasiError::Inval);
                }
                let old_fd = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let old_path_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let old_path_len = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let new_fd = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let new_path_ptr = params[4].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let new_path_len = params[5].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.path_rename(
                    memory,
                    old_fd,
                    old_path_ptr,
                    old_path_len,
                    new_fd,
                    new_path_ptr,
                    new_path_len,
                )?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::PathSymlink => {
                if params.len() != 5 {
                    return Err(WasiError::Inval);
                }
                let old_path_ptr = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let old_path_len = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let fd = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let new_path_ptr = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let new_path_len = params[4].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.path_symlink(
                    memory,
                    old_path_ptr,
                    old_path_len,
                    fd,
                    new_path_ptr,
                    new_path_len,
                )?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::SockAccept => {
                if params.len() != 3 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let flags = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let fd_ptr = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.sock_accept(memory, fd, flags, fd_ptr)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::SockRecv => {
                if params.len() != 6 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let ri_data_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let ri_data_len = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let ri_flags = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let ro_datalen_ptr = params[4].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let ro_flags_ptr = params[5].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.sock_recv(
                    memory,
                    fd,
                    ri_data_ptr,
                    ri_data_len,
                    ri_flags,
                    ro_datalen_ptr,
                    ro_flags_ptr,
                )?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::SockSend => {
                if params.len() != 5 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let si_data_ptr = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let si_data_len = params[2].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let si_flags = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let so_datalen_ptr = params[4].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.sock_send(
                    memory,
                    fd,
                    si_data_ptr,
                    si_data_len,
                    si_flags,
                    so_datalen_ptr,
                )?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::SockShutdown => {
                if params.len() != 2 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let how = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;

                let result = wasi_impl.sock_shutdown(memory, fd, how)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdFdstatSetRights => {
                if params.len() != 3 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let fs_rights_base = params[1].to_i64().map_err(|_| WasiError::Inval)? as u64;
                let fs_rights_inheriting = params[2].to_i64().map_err(|_| WasiError::Inval)? as u64;

                let result = wasi_impl.fd_fdstat_set_rights(
                    &memory,
                    fd,
                    fs_rights_base,
                    fs_rights_inheriting,
                )?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdAdvise => {
                if params.len() != 4 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let offset = params[1].to_i64().map_err(|_| WasiError::Inval)? as u64;
                let len = params[2].to_i64().map_err(|_| WasiError::Inval)? as u64;
                let advice = params[3].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let result = wasi_impl.fd_advise(memory, fd, offset, len, advice)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdAllocate => {
                if params.len() != 3 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let offset = params[1].to_i64().map_err(|_| WasiError::Inval)? as u64;
                let len = params[2].to_i64().map_err(|_| WasiError::Inval)? as u64;
                let result = wasi_impl.fd_allocate(memory, fd, offset, len)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            WasiFuncType::FdRenumber => {
                if params.len() != 2 {
                    return Err(WasiError::Inval);
                }
                let fd = params[0].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let to = params[1].to_i32().map_err(|_| WasiError::Inval)? as u32;
                let result = wasi_impl.fd_renumber(memory, fd, to)?;
                Ok(Some(Val::Num(Num::I32(result))))
            }
            _ => Err(WasiError::NoSys),
        }
    }
}
