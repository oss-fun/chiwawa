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
use crate::wasi::socket;
#[cfg(feature = "threads")]
use crate::wasi::threads::ThreadContext;
use crate::wasi::{Args, WasiError, WasiResult};
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
        Ok(Self::build_runtime(module_inst, stacks, config))
    }

    /// Creates a runtime restored from a checkpoint.
    ///
    /// Used to resume execution after restoring state from a checkpoint file.
    pub fn new_restored(
        module_inst: Rc<ModuleInst>,
        stacks: Stacks,
        config: RuntimeConfig,
    ) -> Self {
        Self::build_runtime(module_inst, stacks, config)
    }

    /// Assembles the runtime over `module_inst` and `stacks`, creating the
    /// stats collector and tracer that `config` asks for.
    fn build_runtime(module_inst: Rc<ModuleInst>, stacks: Stacks, config: RuntimeConfig) -> Self {
        #[cfg(feature = "trace")]
        let tracer = config
            .trace_config
            .and_then(|trace_config| match Tracer::new(trace_config) {
                Ok(tracer) => Some(tracer),
                Err(e) => {
                    eprintln!("Failed to create tracer: {:?}", e);
                    None
                }
            });

        Runtime {
            module_inst,
            stacks,
            #[cfg(feature = "stats")]
            execution_stats: config.enable_stats.then(ExecutionStats::new),
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

        let (i32_base, i64_base, f32_base, f64_base) = self.stacks.reg_file.frame_bases();
        VmState {
            reg_file: reg_file_ptr,
            i32_base,
            i64_base,
            f32_base,
            f64_base,
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

                    #[cfg(feature = "threads")]
                    let next_tid = self.thread_ctx.as_ref().map_or(1, |ctx| ctx.next_tid());
                    #[cfg(not(feature = "threads"))]
                    let next_tid = 1;

                    match migration::rendezvous_and_checkpoint(
                        thread,
                        &self.module_inst.mem_addrs,
                        next_tid,
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
                                // A WASI function reports failure through
                                // its errno, never by trapping.
                                Err(e) => match result_reg {
                                    Some(reg) => {
                                        let errno = Val::Num(Num::I32(e.to_errno()));
                                        self.stacks.reg_file.set_val(&reg, &errno);
                                    }
                                    None => {
                                        return Err(RuntimeError::ExecutionFailed(
                                            "WASI function without a result failed",
                                        ))
                                    }
                                },
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

    /// Calls a WASI function. The parser matched the import against the
    /// expected signature, so `params` has the right count and types.
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
        let memory = self.module_inst.mem_addrs.first().ok_or(WasiError::Fault)?;
        let a = Args(params);
        let errno = match func_type {
            WasiFuncType::FdWrite => {
                wasi_impl.fd_write(memory, a.i32(0)?, a.u32(1)?, a.u32(2)?, a.u32(3)?)?
            }
            WasiFuncType::FdRead => {
                wasi_impl.fd_read(memory, a.i32(0)?, a.u32(1)?, a.u32(2)?, a.u32(3)?)?
            }
            // Never returns.
            WasiFuncType::ProcExit => wasi_impl.proc_exit(a.i32(0)?)?,
            WasiFuncType::RandomGet => wasi_impl.random_get(memory, a.u32(0)?, a.u32(1)?)?,
            WasiFuncType::FdClose => wasi_impl.fd_close(a.i32(0)?)?,
            WasiFuncType::EnvironGet => wasi_impl.environ_get(memory, a.u32(0)?, a.u32(1)?)?,
            WasiFuncType::EnvironSizesGet => {
                wasi_impl.environ_sizes_get(memory, a.u32(0)?, a.u32(1)?)?
            }
            WasiFuncType::ArgsGet => wasi_impl.args_get(memory, a.u32(0)?, a.u32(1)?)?,
            WasiFuncType::ArgsSizesGet => wasi_impl.args_sizes_get(memory, a.u32(0)?, a.u32(1)?)?,
            WasiFuncType::ClockTimeGet => {
                wasi_impl.clock_time_get(memory, a.i32(0)?, a.i64(1)?, a.u32(2)?)?
            }
            WasiFuncType::ClockResGet => wasi_impl.clock_res_get(memory, a.i32(0)?, a.u32(1)?)?,
            WasiFuncType::FdPrestatGet => wasi_impl.fd_prestat_get(memory, a.i32(0)?, a.u32(1)?)?,
            WasiFuncType::FdPrestatDirName => {
                wasi_impl.fd_prestat_dir_name(memory, a.i32(0)?, a.u32(1)?, a.u32(2)?)?
            }
            WasiFuncType::SchedYield => wasi_impl.sched_yield()?,
            WasiFuncType::ThreadSpawn => self.thread_spawn(a.i32(0)?),
            WasiFuncType::FdFdstatGet => wasi_impl.fd_fdstat_get(memory, a.i32(0)?, a.u32(1)?)?,
            WasiFuncType::PathOpen => wasi_impl.path_open(
                memory,
                a.i32(0)?,
                a.u32(1)?,
                a.u32(2)?,
                a.u32(3)?,
                a.u32(4)?,
                a.u64(5)?,
                a.u64(6)?,
                a.u32(7)?,
                a.u32(8)?,
            )?,
            WasiFuncType::FdSeek => {
                wasi_impl.fd_seek(memory, a.i32(0)?, a.i64(1)?, a.u32(2)?, a.u32(3)?)?
            }
            WasiFuncType::FdTell => wasi_impl.fd_tell(memory, a.i32(0)?, a.u32(1)?)?,
            WasiFuncType::FdSync => wasi_impl.fd_sync(a.i32(0)?)?,
            WasiFuncType::FdFilestatGet => {
                wasi_impl.fd_filestat_get(memory, a.i32(0)?, a.u32(1)?)?
            }
            WasiFuncType::FdReaddir => wasi_impl.fd_readdir(
                memory,
                a.i32(0)?,
                a.u32(1)?,
                a.u32(2)?,
                a.u64(3)?,
                a.u32(4)?,
            )?,
            WasiFuncType::FdPread => wasi_impl.fd_pread(
                memory,
                a.i32(0)?,
                a.u32(1)?,
                a.u32(2)?,
                a.u64(3)?,
                a.u32(4)?,
            )?,
            WasiFuncType::FdDatasync => wasi_impl.fd_datasync(a.i32(0)?)?,
            WasiFuncType::FdFdstatSetFlags => {
                wasi_impl.fd_fdstat_set_flags(a.i32(0)?, a.u32(1)?)?
            }
            WasiFuncType::FdFilestatSetSize => {
                wasi_impl.fd_filestat_set_size(a.i32(0)?, a.u64(1)?)?
            }
            WasiFuncType::FdPwrite => wasi_impl.fd_pwrite(
                memory,
                a.i32(0)?,
                a.u32(1)?,
                a.u32(2)?,
                a.u64(3)?,
                a.u32(4)?,
            )?,
            WasiFuncType::PathCreateDirectory => {
                wasi_impl.path_create_directory(memory, a.i32(0)?, a.u32(1)?, a.u32(2)?)?
            }
            WasiFuncType::PathFilestatGet => wasi_impl.path_filestat_get(
                memory,
                a.i32(0)?,
                a.u32(1)?,
                a.u32(2)?,
                a.u32(3)?,
                a.u32(4)?,
            )?,
            WasiFuncType::PathFilestatSetTimes => wasi_impl.path_filestat_set_times(
                memory,
                a.i32(0)?,
                a.u32(1)?,
                a.u32(2)?,
                a.u32(3)?,
                a.u64(4)?,
                a.u64(5)?,
                a.u32(6)?,
            )?,
            WasiFuncType::PathReadlink => wasi_impl.path_readlink(
                memory,
                a.i32(0)?,
                a.u32(1)?,
                a.u32(2)?,
                a.u32(3)?,
                a.u32(4)?,
                a.u32(5)?,
            )?,
            WasiFuncType::PathRemoveDirectory => {
                wasi_impl.path_remove_directory(memory, a.i32(0)?, a.u32(1)?, a.u32(2)?)?
            }
            WasiFuncType::PathUnlinkFile => {
                wasi_impl.path_unlink_file(memory, a.i32(0)?, a.u32(1)?, a.u32(2)?)?
            }
            WasiFuncType::PollOneoff => {
                wasi_impl.poll_oneoff(memory, a.u32(0)?, a.u32(1)?, a.u32(2)?, a.u32(3)?)?
            }
            WasiFuncType::FdFilestatSetTimes => {
                wasi_impl.fd_filestat_set_times(a.u32(0)?, a.u64(1)?, a.u64(2)?, a.u32(3)?)?
            }
            WasiFuncType::PathLink => wasi_impl.path_link(
                memory,
                a.u32(0)?,
                a.u32(1)?,
                a.u32(2)?,
                a.u32(3)?,
                a.u32(4)?,
                a.u32(5)?,
                a.u32(6)?,
            )?,
            WasiFuncType::PathRename => wasi_impl.path_rename(
                memory,
                a.u32(0)?,
                a.u32(1)?,
                a.u32(2)?,
                a.u32(3)?,
                a.u32(4)?,
                a.u32(5)?,
            )?,
            WasiFuncType::PathSymlink => wasi_impl.path_symlink(
                memory,
                a.u32(0)?,
                a.u32(1)?,
                a.u32(2)?,
                a.u32(3)?,
                a.u32(4)?,
            )?,
            WasiFuncType::SockAccept => {
                wasi_impl.sock_accept(memory, a.u32(0)?, a.u32(1)?, a.u32(2)?)?
            }
            WasiFuncType::SockRecv => wasi_impl.sock_recv(
                memory,
                a.u32(0)?,
                a.u32(1)?,
                a.u32(2)?,
                a.u32(3)?,
                a.u32(4)?,
                a.u32(5)?,
            )?,
            WasiFuncType::SockSend => wasi_impl.sock_send(
                memory,
                a.u32(0)?,
                a.u32(1)?,
                a.u32(2)?,
                a.u32(3)?,
                a.u32(4)?,
            )?,
            WasiFuncType::SockShutdown => wasi_impl.sock_shutdown(a.u32(0)?, a.u32(1)?)?,
            WasiFuncType::FdFdstatSetRights => {
                wasi_impl.fd_fdstat_set_rights(a.u32(0)?, a.u64(1)?, a.u64(2)?)?
            }
            WasiFuncType::FdAdvise => {
                wasi_impl.fd_advise(a.u32(0)?, a.u64(1)?, a.u64(2)?, a.u32(3)?)?
            }
            WasiFuncType::FdAllocate => wasi_impl.fd_allocate(a.u32(0)?, a.u64(1)?, a.u64(2)?)?,
            WasiFuncType::FdRenumber => wasi_impl.fd_renumber(a.u32(0)?, a.u32(1)?)?,
            WasiFuncType::SocketExt(ext) => {
                socket::call(*ext, wasi_impl, memory, params)?;
                0
            }
            WasiFuncType::ProcRaise => return Err(WasiError::NoSys),
        };
        Ok(Some(Val::Num(Num::I32(errno))))
    }
}
