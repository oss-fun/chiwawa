//! Runtime core managing execution lifecycle and host function invocation.

use crate::error::RuntimeError;
use crate::execution::func::{FuncAddr, FuncInst};
use crate::execution::migration;
use crate::execution::module::ModuleInst;
use crate::execution::stats::ExecutionStats;
#[cfg(feature = "trace")]
use crate::execution::trace::{TraceConfig, Tracer};
use crate::execution::value::{Num, Val, Vec_};
use crate::execution::vm::{Frame, FrameStack, Label, LabelStack, ModuleLevelInstr, Stacks};
use crate::structure::module::WasiFuncType;
use crate::structure::types::{NumType, ValueType, VecType};
use crate::wasi::{WasiError, WasiResult};
use arrayvec::ArrayVec;
use std::path::Path;
use std::rc::Rc;
#[cfg(all(target_os = "wasi", target_env = "p1", target_feature = "atomics"))]
use std::sync::Once;

/// Execution entry point that manages the interpreter loop.
pub struct Runtime {
    module_inst: Rc<ModuleInst>,
    stacks: Stacks,
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
            module_inst,
            stacks,
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
            module_inst,
            stacks,
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

    /// Executes interpreter loop for a specific frame stack.
    fn run_dtc(
        &mut self,
        frame_stack_idx: usize,
        called_func_addr: &mut Option<FuncAddr>,
    ) -> Result<Result<Option<super::vm::ModuleLevelInstr>, RuntimeError>, RuntimeError> {
        let reg_file = &mut self.stacks.reg_file;
        let frame_stack = &mut self.stacks.activation_frame_stack[frame_stack_idx];
        let stats_ref = self.execution_stats.as_mut();
        #[cfg(feature = "trace")]
        let tracer_ref = self.tracer.as_mut();
        #[cfg(not(feature = "trace"))]
        let tracer_ref: Option<()> = None;
        frame_stack.run_dtc_loop(reg_file, called_func_addr, stats_ref, tracer_ref)
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
        if let Some(frame_stack) = self.stacks.activation_frame_stack.first_mut() {
            frame_stack.enable_checkpoint = self.enable_checkpoint;
        }

        while !self.stacks.activation_frame_stack.is_empty() {
            let frame_stack_idx = self.stacks.activation_frame_stack.len() - 1;
            let mut called_func_addr: Option<FuncAddr> = None;

            let module_level_instr_result = self.run_dtc(frame_stack_idx, &mut called_func_addr)?;

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
                            params,
                            result_regs,
                        }) => {
                            // Register-based function invocation - params already extracted
                            let func_inst_guard = func_addr.read_lock();
                            match &*func_inst_guard {
                                FuncInst::RuntimeFunc {
                                    type_,
                                    module: func_module_weak,
                                    code,
                                } => {
                                    let mut locals = params;
                                    for v in code.locals.iter() {
                                        for _ in 0..(v.0) {
                                            locals.push(Val::default_value(&v.1)?);
                                        }
                                    }

                                    if let Some(ref alloc) = code.reg_allocation {
                                        self.stacks.reg_file.save_offsets(alloc);
                                    }

                                    // Store result_regs in caller frame
                                    if let Some(caller) =
                                        self.stacks.activation_frame_stack.last_mut()
                                    {
                                        caller.result_regs = result_regs;
                                    }
                                    let new_frame = FrameStack {
                                        frame: Frame {
                                            locals,
                                            module: func_module_weak.clone(),
                                            n: type_.results.len(),
                                            result_reg: code.result_reg,
                                        },
                                        label_stack: vec![LabelStack {
                                            label: Label {
                                                locals_num: type_.results.len(),
                                                arity: type_.results.len(),
                                                is_loop: false,
                                                stack_height: 0,
                                                return_ip: 0,
                                            },
                                            processed_instrs: code.body.clone(),
                                            ip: 0,
                                        }],
                                        void: type_.results.is_empty(),
                                        instruction_count: 0,
                                        enable_checkpoint: self.enable_checkpoint,
                                        result_regs: ArrayVec::new(),
                                        return_result_regs: ArrayVec::new(),
                                    };
                                    self.stacks.activation_frame_stack.push(new_frame);
                                }
                                FuncInst::HostFunc { host_code, .. } => {
                                    // Host function with register-based params
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
                                // First read values from finished frame's registers (before restore)
                                // Use ArrayVec to avoid heap allocation
                                let values_to_pass: ArrayVec<Val, 8> = return_result_regs
                                    .iter()
                                    .map(|reg| self.stacks.reg_file.get_val(reg))
                                    .collect();

                                // Restore offsets to caller's frame
                                self.stacks.reg_file.restore_offsets();

                                // Write to caller's registers (after restore, in caller's coordinate system)
                                let (reg_file, frames) = self.stacks.get_reg_file_and_frames();
                                let caller_frame = frames.last_mut().unwrap();

                                if !caller_frame.result_regs.is_empty() {
                                    for (caller_reg, val) in
                                        caller_frame.result_regs.iter().zip(values_to_pass.iter())
                                    {
                                        reg_file.set_val(caller_reg, val);
                                    }
                                    caller_frame.result_regs.clear();
                                }
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

impl Val {
    /// Creates a default value for the given WebAssembly type.
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
