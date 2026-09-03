//! wasi-threads support.
//!
//! Each spawned thread gets its own instance of the module and the instances
//! share only the linear memory, so per-instance state such as
//! `__stack_pointer` stays private to its thread. [`ThreadContext`] holds what
//! that instantiation needs and is passed on so a worker can spawn further
//! threads.
//!
//! The thread itself comes from the host: `std::thread` compiles down to the
//! host's own `wasi::thread-spawn`. Only the guest's entry point is dispatched
//! here, since the host's callback lands on Chiwawa's `wasi_thread_start`.

use crate::error::RuntimeError;
use crate::execution::mem::MemAddr;
use crate::execution::module::{ImportObjects, ModuleInst};
use crate::execution::runtime::{Runtime, RuntimeConfig};
use crate::execution::value::{Externval, Num, Val};
use crate::structure::module::{ImportDesc, Module};
use rustc_hash::FxHashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

/// Export the guest must provide for a spawned thread to start on.
const THREAD_START_EXPORT: &str = "wasi_thread_start";

/// Shared state that `wasi::thread_spawn` needs to build a thread's instance.
pub struct ThreadContext {
    module: Arc<Module>,
    /// `(module, name)` of the memory import each instance binds, or `None`
    /// when the module defines the memory itself.
    mem_import: Option<(String, String)>,
    /// The one memory all threads share.
    memory: MemAddr,
    argv: Vec<String>,
    next_tid: AtomicI32,
}

impl ThreadContext {
    /// Builds a context for a module that can share a memory between threads,
    /// allocating that memory.
    ///
    /// The memory is either imported, or defined by the module and marked
    /// shared -- wasi-sdk emits the latter unless built with `--import-memory`.
    /// Returns `None` when the module has neither, since then each thread's
    /// instance would get a memory of its own.
    pub fn new(module: Arc<Module>, argv: Vec<String>) -> Option<Arc<Self>> {
        let imported = module.imports.iter().find_map(|import| match &import.desc {
            ImportDesc::Mem(mem_type) => {
                Some(((import.module.0.clone(), import.name.0.clone()), *mem_type))
            }
            _ => None,
        });
        let (mem_import, mem_type) = match imported {
            Some((names, mem_type)) => (Some(names), mem_type),
            None => {
                let defined = module.mems.iter().find(|mem| mem.type_.shared)?;
                (None, defined.type_)
            }
        };
        Some(Arc::new(ThreadContext {
            module,
            mem_import,
            memory: MemAddr::new(&mem_type),
            argv,
            next_tid: AtomicI32::new(1),
        }))
    }

    /// Instantiates the module bound to the shared memory. Every thread's
    /// instance, the first one included, is built through here.
    pub fn instantiate(&self) -> Result<Rc<ModuleInst>, RuntimeError> {
        let mut imports: ImportObjects = FxHashMap::default();
        let defined_mem = match &self.mem_import {
            Some((module_name, name)) => {
                imports
                    .entry(module_name.clone())
                    .or_default()
                    .insert(name.clone(), Externval::Mem(self.memory.clone()));
                None
            }
            None => Some(self.memory.clone()),
        };
        ModuleInst::new_with_shared_memory(&self.module, imports, self.argv.clone(), defined_mem)
    }

    /// Spawns a thread running `wasi_thread_start(tid, start_arg)` and returns
    /// its thread id.
    pub fn spawn(self: &Arc<Self>, start_arg: i32) -> Result<i32, RuntimeError> {
        let tid = self.next_tid.fetch_add(1, Ordering::Relaxed);
        let ctx = Arc::clone(self);
        std::thread::Builder::new()
            .spawn(move || {
                if let Err(e) = ctx.run(tid, start_arg) {
                    eprintln!("Thread {} failed: {:?}", tid, e);
                }
            })
            .map_err(|_| RuntimeError::ExecutionFailed("failed to spawn thread"))?;
        Ok(tid)
    }

    /// Instantiates the module against the shared memory and runs the guest's
    /// thread entry point.
    fn run(self: &Arc<Self>, tid: i32, start_arg: i32) -> Result<(), RuntimeError> {
        let inst = self.instantiate()?;
        let start = inst.get_export_func(THREAD_START_EXPORT)?;
        let params = vec![Val::Num(Num::I32(tid)), Val::Num(Num::I32(start_arg))];

        let config = RuntimeConfig {
            thread_ctx: Some(Arc::clone(self)),
            ..Default::default()
        };
        Runtime::new(inst, &start, params, config)?.run()?;
        Ok(())
    }
}
