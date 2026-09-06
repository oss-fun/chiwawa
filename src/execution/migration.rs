//! Checkpoint and restore functionality for live migration.
//!
//! This module implements serialization and deserialization of runtime state,
//! enabling process migration, fault tolerance, and debugging capabilities.
//!
//! ## Serializable State
//!
//! The checkpoint captures:
//! - The register file (`RegFile`), which holds both operand-stack values and
//!   function locals (params + declared locals)
//! - Activation frame stack (per-frame `func_idx`, ip, result registers)
//! - Linear memory contents (LZ4 compressed)
//! - Global variable values
//!
//! `FrameStack` fields that depend on host addresses are `#[serde(skip)]`:
//! `cached_mem_ptr` is re-cached from the restored memory, while
//! `enable_checkpoint` is set by `Runtime::run`. A frame
//! names its function by `func_idx`, so the body and handler array need no
//! reconstruction at all. Tables are excluded: they are deterministically
//! initialized from element segments during module instantiation.
//!
//! ## Trigger Mechanisms
//!
//! `poll_checkpoint` is invoked per instruction by the v2 dispatcher
//! (`dispatch_loop` at the loop head, `dispatch_tco` via the `next_handler`
//! shim in the `advance!` macro). The check resolves to one of two paths at
//! compile time:
//!
//! - **wasm32-wasip1-threads** (`target_feature = "atomics"`): background
//!   thread set up by `setup_checkpoint_monitor` watches the
//!   `checkpoint.trigger` file and fills every registered handler table with
//!   `checkpoint_trap`, so the dispatcher polls nothing at all.
//! - **wasm32-wasip1** (no atomics): `poll_checkpoint` throttles itself with
//!   `VmState.checkpoint_poll_counter` and only issues the WASI file-existence
//!   syscall every `CHECKPOINT_POLL_MASK + 1` (= 1024) instructions to keep
//!   the dispatcher overhead bounded.
//!
//! Either path triggers `Outcome::Trap(CheckpointRequested)`, which
//! `runtime.rs` translates into a `checkpoint` call.

use crate::error::RuntimeError;
use crate::execution::global::GlobalAddr;
use crate::execution::mem::MemAddr;
use crate::execution::module::ModuleInst;
use crate::execution::state::{Stacks, VmState};
use crate::execution::value::Val;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::{Condvar, Mutex, MutexGuard};

const CHECKPOINT_TRIGGER_FILE: &str = "./checkpoint.trigger";

cfg_if::cfg_if! {
    if #[cfg(all(
        target_arch = "wasm32",
        target_os = "wasi",
        target_env = "p1",
        target_feature = "atomics"
    ))] {
        use crate::structure::module::HandlerTable;
        use std::thread;
        use std::time::Duration;

        /// Safety: the tables live in the `Module`, which outlives the
        /// monitor, and the monitor only calls `fill` on them.
        pub struct MonitoredTables(Vec<*const HandlerTable>);
        unsafe impl Send for MonitoredTables {}

        impl MonitoredTables {
            pub fn new(tables: Vec<*const HandlerTable>) -> Self {
                MonitoredTables(tables)
            }

            fn fill_all(&self, handler: crate::execution::ir::Handler) {
                for table in &self.0 {
                    unsafe { (**table).fill(handler) };
                }
            }
        }

        pub fn setup_checkpoint_monitor(tables: MonitoredTables) {
            thread::spawn(move || loop {
                if std::path::Path::new(CHECKPOINT_TRIGGER_FILE).exists() {
                    tables.fill_all(crate::execution::handlers::checkpoint_trap);
                    crate::execution::atomics::interrupt_waiters();
                    let _ = std::fs::remove_file(CHECKPOINT_TRIGGER_FILE);
                }
                thread::sleep(Duration::from_millis(100));
            });
        }

        #[inline(always)]
        pub fn poll_checkpoint(_state: &mut VmState) -> bool {
            false
        }
    } else {
        /// Counter-throttled file poll: actual syscall only once every
        /// `CHECKPOINT_POLL_MASK + 1` instructions.
        #[inline(always)]
        pub fn poll_checkpoint(state: &mut VmState) -> bool {
            if !state.enable_checkpoint {
                return false;
            }
            do_poll_checkpoint(state)
        }

        #[inline(never)]
        fn do_poll_checkpoint(state: &mut VmState) -> bool {
            state.checkpoint_poll_counter = state.checkpoint_poll_counter.wrapping_add(1);
            if state.checkpoint_poll_counter & CHECKPOINT_POLL_MASK != 0 {
                return false;
            }
            if let Some(ref wasi) = state.module().wasi_impl {
                if wasi.check_file_exists(CHECKPOINT_TRIGGER_FILE) {
                    let _ = std::fs::remove_file(CHECKPOINT_TRIGGER_FILE);
                    return true;
                }
            }
            false
        }

        /// Throttle interval for non-atomics file polling (= every 1024 instructions).
        const CHECKPOINT_POLL_MASK: u32 = 0x3FF;
    }
}

/// Contains all information needed to restore execution:
/// - Call stack and register state
/// - Linear memory contents (LZ4 compressed)
/// - Global variable values
///
/// Tables are excluded: they are deterministically initialized from element
/// segments during module instantiation.
#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableState {
    pub memory_data_compressed: Vec<u8>,
    pub threads: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
pub struct ThreadState {
    pub stacks: Stacks,
    pub globals: Vec<Val>,
}
pub fn serialize_thread(
    stacks: &Stacks,
    global_addrs: &[GlobalAddr],
) -> Result<Vec<u8>, RuntimeError> {
    let state = ThreadState {
        stacks: stacks.clone(),
        globals: global_addrs.iter().map(|global| global.get()).collect(),
    };
    bincode::serialize(&state).map_err(|e| RuntimeError::SerializationError(e.to_string()))
}

struct Rendezvous {
    live_threads: usize,
    arrived: Vec<Vec<u8>>,
    written: bool,
}

static RENDEZVOUS: Mutex<Rendezvous> = Mutex::new(Rendezvous {
    live_threads: 0,
    arrived: Vec::new(),
    written: false,
});
static ARRIVED: Condvar = Condvar::new();

impl Rendezvous {
    fn all_arrived(&self) -> bool {
        !self.arrived.is_empty() && self.arrived.len() >= self.live_threads
    }
}

fn lock_rendezvous() -> MutexGuard<'static, Rendezvous> {
    RENDEZVOUS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn thread_started() {
    lock_rendezvous().live_threads += 1;
}

pub fn thread_finished() {
    let mut rendezvous = lock_rendezvous();
    rendezvous.live_threads = rendezvous.live_threads.saturating_sub(1);
    let all_arrived = rendezvous.all_arrived();
    drop(rendezvous);
    if all_arrived {
        ARRIVED.notify_all();
    }
}

pub fn rendezvous_and_checkpoint<P: AsRef<Path>>(
    thread: Vec<u8>,
    mem_addrs: &[MemAddr],
    output_path: P,
) -> Result<(), RuntimeError> {
    let mut rendezvous = lock_rendezvous();
    rendezvous.arrived.push(thread);
    loop {
        if rendezvous.written {
            return Ok(());
        }
        if rendezvous.all_arrived() {
            let threads = std::mem::take(&mut rendezvous.arrived);
            let result = write_checkpoint(threads, mem_addrs, output_path);
            rendezvous.written = true;
            drop(rendezvous);
            ARRIVED.notify_all();
            return result;
        }
        rendezvous = ARRIVED
            .wait(rendezvous)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }
}

fn write_checkpoint<P: AsRef<Path>>(
    threads: Vec<Vec<u8>>,
    mem_addrs: &[MemAddr],
    output_path: P,
) -> Result<(), RuntimeError> {
    println!("Checkpointing state to {:?}...", output_path.as_ref());

    let (memory_data_compressed, mem_raw_size) = match mem_addrs.first() {
        Some(mem_addr) => {
            let raw = mem_addr.get_data();
            let raw_len = raw.len();
            (lz4_flex::compress_prepend_size(&raw), raw_len)
        }
        None => (Vec::new(), 0),
    };

    // No compaction: `restore_offsets` truncates the register file on every return.
    let state = SerializableState {
        memory_data_compressed,
        threads,
    };

    println!("Checkpoint component sizes:");
    println!(
        "  threads:            {} ({} bytes)",
        state.threads.len(),
        state.threads.iter().map(Vec::len).sum::<usize>()
    );
    println!(
        "  memory_data:        {} bytes (raw {} bytes, LZ4 compressed)",
        state.memory_data_compressed.len(),
        mem_raw_size
    );

    let encoded: Vec<u8> =
        bincode::serialize(&state).map_err(|e| RuntimeError::SerializationError(e.to_string()))?;

    println!("  total encoded:      {} bytes", encoded.len());

    let mut file =
        File::create(output_path).map_err(|e| RuntimeError::CheckpointSaveError(e.to_string()))?;
    file.write_all(&encoded)
        .map_err(|e| RuntimeError::CheckpointSaveError(e.to_string()))?;

    println!("Checkpoint successful ({} threads).", state.threads.len());
    Ok(())
}

/// Restores runtime state from a checkpoint file.
///
/// Reads serialized state and restores memory, globals, and stacks.
pub fn restore<P: AsRef<Path>>(
    module_inst: Rc<ModuleInst>,
    input_path: P,
) -> Result<Stacks, RuntimeError> {
    println!("Restoring state from {:?}...", input_path.as_ref());

    // 1. Read from file
    let mut file =
        File::open(input_path).map_err(|e| RuntimeError::CheckpointLoadError(e.to_string()))?;
    let mut encoded = Vec::new();
    file.read_to_end(&mut encoded)
        .map_err(|e| RuntimeError::CheckpointLoadError(e.to_string()))?;

    // 2. Deserialize the state using bincode
    let mut state: SerializableState = bincode::deserialize(&encoded[..])
        .map_err(|e| RuntimeError::DeserializationError(e.to_string()))?;

    // 3. Restore memory state (LZ4 decompress)
    if let Some(mem_addr) = module_inst.mem_addrs.get(0) {
        let memory_data = lz4_flex::decompress_size_prepended(&state.memory_data_compressed)
            .map_err(|e| {
                RuntimeError::DeserializationError(format!("LZ4 decompression failed: {}", e))
            })?;
        mem_addr.set_data(memory_data);
        println!("Memory state restored into module instance.");
    } else if !state.memory_data_compressed.is_empty() {
        eprintln!("Warning: Checkpoint contains memory data, but module has no memory instance.");
    }

    // 4. Restore globals
    let Some(encoded) = state.threads.first() else {
        return Err(RuntimeError::CheckpointLoadError(
            "checkpoint holds no threads".to_string(),
        ));
    };
    let mut thread: ThreadState = bincode::deserialize(encoded)
        .map_err(|e| RuntimeError::DeserializationError(e.to_string()))?;
    if module_inst.global_addrs.len() == thread.globals.len() {
        for (global_addr, value) in module_inst
            .global_addrs
            .iter()
            .zip(thread.globals.drain(..))
        {
            global_addr.set(value)?;
        }
        println!("Global state restored into module instance.");
    } else {
        eprintln!(
            "Warning: Mismatch in global variable count between module ({}) and checkpoint ({}). Globals not restored.",
            module_inst.global_addrs.len(),
            thread.globals.len()
        );
    }
    if state.threads.len() > 1 {
        eprintln!(
            "Warning: checkpoint holds {} threads; only the first is restored.",
            state.threads.len()
        );
    }

    // 5. Reconstruct the one skipped field that execution needs: the cached
    // pointer to the freshly restored memory. The body and handler array are
    // not stored per frame; `execute_frame` reads them via `func_idx`.
    let mem_ptr = module_inst.mem_addrs.first().map(|m| m.data_ptr());
    let mut stacks = thread.stacks;
    for frame_stack in stacks.activation_frame_stack.iter_mut() {
        frame_stack.cached_mem_ptr = mem_ptr;
    }
    println!("Frame memory references restored.");

    println!("Restore successful (state applied to module). Returning Stacks.");
    Ok(stacks)
}
