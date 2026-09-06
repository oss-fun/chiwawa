//! Runtime core managing execution lifecycle and host function invocation.

use crate::error::RuntimeError;
use crate::execution::dispatch;
use crate::execution::func::{FuncAddr, FuncInst};
use crate::execution::ir::Outcome;
use crate::execution::migration;
use crate::execution::module::ModuleInst;
use crate::execution::regs::RegFile;
use crate::execution::state::VmState;
use crate::execution::state::{FrameStack, ModuleLevelInstr, Stacks};
use crate::execution::value::{Num, Val};
#[cfg(feature = "stats")]
use crate::instrument::stats::ExecutionStats;
#[cfg(feature = "trace")]
use crate::instrument::trace::{TraceConfig, Tracer};
use crate::structure::module::{Func, WasiFuncType};
#[cfg(feature = "threads")]
use crate::wasi::threads::ThreadContext;
use crate::wasi::{WasiError, WasiResult};
use std::path::Path;
use std::rc::Rc;
#[cfg(feature = "threads")]
use std::sync::Arc;
#[cfg(all(target_os = "wasi", target_env = "p1", target_feature = "atomics"))]
use std::sync::Once;

/// Optional runtime settings.
///
/// Each field exists only when its feature is enabled, so the constructor
/// signature is the same in every build.
#[derive(Default)]
pub struct RuntimeConfig {
    pub enable_checkpoint: bool,
    #[cfg(feature = "stats")]
    pub enable_stats: bool,
    #[cfg(feature = "trace")]
    pub trace_config: Option<TraceConfig>,
    /// Present when wasi-threads is enabled; `None` makes `thread_spawn` fail.
    #[cfg(feature = "threads")]
    pub thread_ctx: Option<Arc<ThreadContext>>,
}

/// Runs the module's start section, if it has one.
pub fn run_start_section(module_inst: &Rc<ModuleInst>) -> Result<(), RuntimeError> {
    let Some(start) = module_inst.start_section.clone() else {
        return Ok(());
    };
    let mut runtime = Runtime::new(
        Rc::clone(module_inst),
        &start,
        Vec::new(),
        RuntimeConfig::default(),
    )?;
    runtime.run()?;
    Ok(())
}

/// Execution entry point that manages the interpreter loop.
pub struct Runtime {
    module_inst: Rc<ModuleInst>,
    stacks: Stacks,
    #[cfg(feature = "stats")]
    execution_stats: Option<ExecutionStats>,
    #[cfg(feature = "trace")]
    tracer: Option<Tracer>,
    #[cfg(feature = "stats")]
    enable_stats: bool,
    enable_checkpoint: bool,
    #[cfg(feature = "threads")]
    thread_ctx: Option<Arc<ThreadContext>>,
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
        config: RuntimeConfig,
    ) -> Result<Self, RuntimeError> {
        let stacks = Stacks::new(func_addr, params)?;

        #[cfg(feature = "trace")]
        let tracer = if let Some(trace_config) = config.trace_config {
            match Tracer::new(trace_config) {
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
            #[cfg(feature = "stats")]
            execution_stats: if config.enable_stats {
                Some(ExecutionStats::new())
            } else {
                None
            },
            #[cfg(feature = "trace")]
            tracer,
            #[cfg(feature = "stats")]
            enable_stats: config.enable_stats,
            enable_checkpoint: config.enable_checkpoint,
            #[cfg(feature = "threads")]
            thread_ctx: config.thread_ctx,
        })
    }

    /// Creates a runtime restored from a checkpoint.
    ///
    /// Used to resume execution after restoring state from a checkpoint file.
    pub fn new_restored(
        module_inst: Rc<ModuleInst>,
        stacks: Stacks,
        config: RuntimeConfig,
    ) -> Self {
        #[cfg(feature = "trace")]
        let tracer = if let Some(trace_config) = config.trace_config {
            match Tracer::new(trace_config) {
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
            #[cfg(feature = "stats")]
            execution_stats: if config.enable_stats {
                Some(ExecutionStats::new())
            } else {
                None
            },
            #[cfg(feature = "trace")]
            tracer,
            #[cfg(feature = "stats")]
            enable_stats: config.enable_stats,
            enable_checkpoint: config.enable_checkpoint,
            #[cfg(feature = "threads")]
            thread_ctx: config.thread_ctx,
        }
    }

    /// Builds the dispatcher state for the frame on top of the stack. Called
    /// once per `run`; frame switches update the state in place.
    fn build_vm_state(&mut self) -> VmState {
        let module_ptr: *const ModuleInst = Rc::as_ptr(&self.module_inst);
        let reg_file_ptr: *mut RegFile = &mut self.stacks.reg_file as *mut RegFile;
        let frames_ptr: *mut Vec<FrameStack> =
            &mut self.stacks.activation_frame_stack as *mut Vec<FrameStack>;

        // Body and handlers stay owned by the module for its whole lifetime, so
        // the frame names its function by index rather than holding an `Rc`.
        let frame_stack = self.stacks.activation_frame_stack.last().unwrap();
        let func_idx = frame_stack.func_idx;
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

        VmState {
            reg_file: reg_file_ptr,
            pc: frame_stack.ip,
            instrs: body_ptr,
            instrs_len: body_len,
            handlers: code_handlers_ptr,
            mem_ptr: frame_stack.cached_mem_ptr.unwrap_or(std::ptr::null_mut()),
            code: code_ptr,
            module: module_ptr,
            frames: frames_ptr,
            trap: None,
            yielded: None,
            enable_checkpoint: frame_stack.enable_checkpoint,
            checkpoint_poll_counter: 0,
            #[cfg(feature = "stats")]
            stats: self
                .execution_stats
                .as_mut()
                .map_or(std::ptr::null_mut(), |s| s as *mut ExecutionStats),
            #[cfg(feature = "trace")]
            tracer: self
                .tracer
                .as_mut()
                .map_or(std::ptr::null_mut(), |t| t as *mut Tracer),
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
                    // One table per function, shared by every thread's instance.
                    let tables = self
                        .module_inst
                        .func_addrs
                        .iter()
                        .filter_map(|func_addr| match func_addr.read_lock() {
                            FuncInst::RuntimeFunc { code, .. } => Some(&*code.handlers as *const _),
                            _ => None,
                        })
                        .collect();
                    migration::setup_checkpoint_monitor(migration::MonitoredTables::new(tables));
                });
            }
        }

        if let Some(frame_stack) = self.stacks.activation_frame_stack.first_mut() {
            frame_stack.enable_checkpoint = self.enable_checkpoint;
        }

        // One state for the whole run: frame switches update it in place, so a
        // WASI or host call resumes without rebuilding it.
        let mut state = self.build_vm_state();

        loop {
            let outcome = dispatch::execute_instructions(&mut state);

            // The dispatcher may have entered callees, so write back to the
            // frame it ended in.
            if let Some(frame) = self.stacks.activation_frame_stack.last_mut() {
                frame.ip = state.pc;
                frame.cached_mem_ptr = if state.mem_ptr.is_null() {
                    None
                } else {
                    Some(state.mem_ptr)
                };
            }

            let module_level_instr_result: Result<Option<ModuleLevelInstr>, RuntimeError> =
                match outcome {
                    Outcome::Halt => Ok(None),
                    Outcome::Yield => Ok(state.yielded.take()),
                    Outcome::Trap => {
                        let err = state
                            .trap
                            .take()
                            .expect("Outcome::Trap returned without state.trap set");
                        if matches!(err, RuntimeError::CheckpointRequested) {
                            Err(err)
                        } else {
                            return Err(err);
                        }
                    }
                    Outcome::Continue => unreachable!("dispatcher must not return Continue"),
                };

            match module_level_instr_result {
                Err(RuntimeError::CheckpointRequested) => {
                    println!("Runtime handling checkpoint request...");
                    let thread =
                        migration::serialize_thread(&self.stacks, &self.module_inst.global_addrs)?;
                    match migration::rendezvous_and_checkpoint(
                        thread,
                        &self.module_inst.mem_addrs,
                        Path::new("./checkpoint.bin"),
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
                        Some(ModuleLevelInstr::InvokeHost {
                            func_addr,
                            params,
                            result_regs,
                        }) => {
                            let func_inst_guard = func_addr.read_lock();
                            match &*func_inst_guard {
                                FuncInst::HostFunc { host_code, .. } => match host_code(params) {
                                    Ok(results) => {
                                        for (reg, val) in result_regs.iter().zip(results.iter()) {
                                            self.stacks.reg_file.set_val(reg, val);
                                        }
                                    }
                                    Err(e) => return Err(e),
                                },
                                _ => {
                                    return Err(RuntimeError::ExecutionFailed(
                                        "WASI function called via InvokeHost - use CallWasiReg",
                                    ));
                                }
                            }
                        }
                        // Halt only reaches here from the outermost frame;
                        // nested returns are handled by the dispatcher.
                        None => {
                            let finished = self.stacks.activation_frame_stack.pop().unwrap();
                            let values: Vec<Val> = finished
                                .return_result_regs
                                .iter()
                                .take(finished.frame.n)
                                .map(|reg| self.stacks.reg_file.get_val(reg))
                                .collect();
                            self.stacks.reg_file.restore_offsets();
                            return Ok(values);
                        }
                    }
                }
            }
        }
    }

    /// wasi-threads `thread_spawn`: starts a thread on the guest's
    /// `wasi_thread_start` export and returns its thread id, or a negative
    /// errno when threads are unavailable or the host refused the spawn.
    fn thread_spawn(&self, start_arg: i32) -> i32 {
        #[cfg(feature = "threads")]
        if let Some(ctx) = self.thread_ctx.as_ref() {
            return match ctx.spawn(start_arg) {
                Ok(tid) => tid,
                Err(e) => {
                    eprintln!("thread_spawn failed: {:?}", e);
                    -WasiError::Again.to_errno()
                }
            };
        }
        let _ = start_arg;
        -WasiError::NoSys.to_errno()
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
            WasiFuncType::ThreadSpawn => {
                if params.len() != 1 {
                    return Err(WasiError::Inval);
                }
                let start_arg = params[0].to_i32().map_err(|_| WasiError::Inval)?;
                Ok(Some(Val::Num(Num::I32(self.thread_spawn(start_arg)))))
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
