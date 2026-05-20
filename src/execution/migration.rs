//! Checkpoint and restore functionality for live migration.
//!
//! This module implements serialization and deserialization of runtime state,
//! enabling process migration, fault tolerance, and debugging capabilities.
//!
//! ## Serializable State
//!
//! The checkpoint captures:
//! - Execution stacks (call frames and register state)
//! - Linear memory contents
//! - Global variable values
//!
//! ## Trigger Mechanisms
//!
//! - **wasm32-wasip1-threads**: Background thread monitors `checkpoint.trigger` file
//! - **wasm32-wasip1**: WASI-based file existence check at instruction boundaries

use crate::error::RuntimeError;
use crate::execution::func::FuncInst;
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
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

static CHECKPOINT_TRIGGERED: AtomicBool = AtomicBool::new(false);
const CHECKPOINT_TRIGGER_FILE: &str = "./checkpoint.trigger";

/// Starts background thread to monitor checkpoint trigger file.
///
/// Used on `wasm32-wasip1-threads` target for non-blocking checkpoint detection.
pub fn setup_checkpoint_monitor() {
    thread::spawn(|| loop {
        if std::path::Path::new(CHECKPOINT_TRIGGER_FILE).exists() {
            CHECKPOINT_TRIGGERED.store(true, Ordering::Relaxed);
            let _ = std::fs::remove_file(CHECKPOINT_TRIGGER_FILE);
        }
        thread::sleep(Duration::from_millis(10));
    });
}

/// Checks if checkpoint has been triggered via atomic flag.
#[inline(always)]
pub fn check_checkpoint_flag() -> bool {
    CHECKPOINT_TRIGGERED.load(Ordering::Relaxed)
}

/// Polls for a checkpoint request from the dispatcher hot path.
///
/// Returns `true` if a checkpoint should be taken. Designed to be called per
/// instruction from the v2 dispatcher (`advance!` macro in TCO mode, loop
/// header in legacy mode).
///
/// # Granularity
/// - **atomics target** (`wasm32-wasip1-threads`): every call hits a cheap
///   atomic load. The flag is set asynchronously by the monitor thread.
/// - **non-atomics target** (`wasm32-wasip1`): an internal counter throttles
///   the WASI file-existence syscall to once every `CHECKPOINT_POLL_INTERVAL`
///   instructions. This balances responsiveness with overhead.
///
/// Per-instruction polling is the design intent: checkpoints can fire at
/// any execution point, not just function/loop boundaries.
///
/// Hot path (checkpoint disabled) is inlined to a single branch; the slow
/// path is `#[inline(never)]` so the dispatcher's tail-call to the next
/// handler is not displaced by inlined syscall code.
#[inline(always)]
pub fn poll_checkpoint(state: &mut VmState) -> bool {
    if !state.enable_checkpoint {
        return false;
    }
    do_poll_checkpoint(state)
}

#[inline(never)]
fn do_poll_checkpoint(state: &mut VmState) -> bool {
    #[cfg(all(
        target_arch = "wasm32",
        target_os = "wasi",
        target_env = "p1",
        target_feature = "atomics"
    ))]
    {
        let _ = state;
        check_checkpoint_flag()
    }

    #[cfg(not(all(
        target_arch = "wasm32",
        target_os = "wasi",
        target_env = "p1",
        target_feature = "atomics"
    )))]
    {
        state.checkpoint_poll_counter = state.checkpoint_poll_counter.wrapping_add(1);
        if state.checkpoint_poll_counter & CHECKPOINT_POLL_MASK != 0 {
            return false;
        }
        unsafe {
            if let Some(ref wasi) = (*state.module).wasi_impl {
                if wasi.check_file_exists(CHECKPOINT_TRIGGER_FILE) {
                    let _ = std::fs::remove_file(CHECKPOINT_TRIGGER_FILE);
                    return true;
                }
            }
        }
        false
    }
}

/// Throttle interval for non-atomics file polling (= every 1024 instructions).
const CHECKPOINT_POLL_MASK: u32 = 0x3FF;

/// Complete runtime state for checkpoint serialization.
///
/// Contains all information needed to restore execution:
/// - Call stack and register state
/// - Linear memory contents (LZ4 compressed)
/// - Global variable values
///
/// Tables are excluded: they are deterministically initialized from element
/// segments during module instantiation.
#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableState {
    pub stacks: Stacks,
    pub memory_data_compressed: Vec<u8>,
    pub global_values: Vec<Val>,
    pub frame_func_indices: Vec<u32>,
}

/// Serializes runtime state to a checkpoint file.
///
/// Captures memory, globals, and stack state for later restoration.
pub fn checkpoint<P: AsRef<Path>>(
    module_inst: &ModuleInst,
    stacks: &Stacks,
    mem_addrs: &[MemAddr],
    global_addrs: &[GlobalAddr],
    output_path: P,
) -> Result<(), RuntimeError> {
    println!("Checkpointing state to {:?}...", output_path.as_ref());

    // 1. Gather Memory state (LZ4 compressed)
    let (memory_data_compressed, mem_raw_size) = if let Some(mem_addr) = mem_addrs.get(0) {
        let raw = mem_addr.get_data();
        let raw_len = raw.len();
        let compressed = lz4_flex::compress_prepend_size(&raw);
        (compressed, raw_len)
    } else {
        (Vec::new(), 0)
    };

    // 2. Gather Global state
    let global_values = global_addrs
        .iter()
        .map(|global_addr| Ok(global_addr.get()))
        .collect::<Result<Vec<Val>, RuntimeError>>()?;

    // 3. Compute function indices for each activation frame (using Rc::ptr_eq)
    let frame_func_indices = stacks
        .activation_frame_stack
        .iter()
        .map(|frame_stack| {
            let frame_instrs = &frame_stack.label_stack[0].processed_instrs;
            module_inst
                .func_addrs
                .iter()
                .position(|func_addr| {
                    let inst = func_addr.read_lock();
                    if let FuncInst::RuntimeFunc { code, .. } = inst {
                        Rc::ptr_eq(frame_instrs, &code.body)
                    } else {
                        false
                    }
                })
                .expect("Function not found in module func_addrs during checkpoint")
                as u32
        })
        .collect::<Vec<u32>>();

    // 4. Assemble state
    // Note: Register file is already compact because restore_offsets() truncates
    // register vectors on function return, so no checkpoint-time compaction needed.
    let state = SerializableState {
        stacks: stacks.clone(),
        memory_data_compressed,
        global_values,
        frame_func_indices,
    };

    // 6. Serialize and write (with per-component size diagnostics)
    let reg_file_size = bincode::serialize(&state.stacks.reg_file)
        .map(|v| v.len())
        .unwrap_or(0);
    let frames_count = state.stacks.activation_frame_stack.len();
    let total_labels: usize = state
        .stacks
        .activation_frame_stack
        .iter()
        .map(|f| f.label_stack.len())
        .sum();
    let frames_size = bincode::serialize(&state.stacks.activation_frame_stack)
        .map(|v| v.len())
        .unwrap_or(0);
    let total_locals: usize = state
        .stacks
        .activation_frame_stack
        .iter()
        .map(|f| f.frame.locals.len())
        .sum();
    let _stacks_size = reg_file_size + frames_size;
    let memory_size = bincode::serialize(&state.memory_data_compressed)
        .map(|v| v.len())
        .unwrap_or(0);
    let globals_size = bincode::serialize(&state.global_values)
        .map(|v| v.len())
        .unwrap_or(0);
    let indices_size = bincode::serialize(&state.frame_func_indices)
        .map(|v| v.len())
        .unwrap_or(0);
    println!("Checkpoint component sizes:");
    println!("  reg_file:           {} bytes", reg_file_size);
    println!(
        "  frames:             {} bytes ({} frames, {} labels, {} locals total)",
        frames_size, frames_count, total_labels, total_locals
    );
    println!(
        "  memory_data:        {} bytes (raw {} bytes, LZ4 compressed)",
        memory_size, mem_raw_size
    );
    println!("  global_values:      {} bytes", globals_size);
    println!("  frame_func_indices: {} bytes", indices_size);

    let encoded: Vec<u8> =
        bincode::serialize(&state).map_err(|e| RuntimeError::SerializationError(e.to_string()))?;

    println!("  total encoded:      {} bytes", encoded.len());

    let mut file =
        File::create(output_path).map_err(|e| RuntimeError::CheckpointSaveError(e.to_string()))?;
    file.write_all(&encoded)
        .map_err(|e| RuntimeError::CheckpointSaveError(e.to_string()))?;

    println!("Checkpoint successful.");
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

    // 4. Restore global state into module_inst
    if module_inst.global_addrs.len() == state.global_values.len() {
        for (global_addr, value) in module_inst.global_addrs.iter().zip(state.global_values) {
            global_addr.set(value)?;
        }
        println!("Global state restored into module instance.");
    } else {
        eprintln!(
            "Warning: Mismatch in global variable count between module ({}) and checkpoint ({}). Globals not restored.",
            module_inst.global_addrs.len(),
            state.global_values.len()
        );
    }

    // 5. Reconstruct skipped fields in Stacks (Frame::module, primary_mem, processed_instrs)
    let primary_mem = module_inst.mem_addrs.first().cloned();
    for (frame_stack, &func_idx) in state
        .stacks
        .activation_frame_stack
        .iter_mut()
        .zip(state.frame_func_indices.iter())
    {
        frame_stack.frame.module = Rc::downgrade(&module_inst);
        frame_stack.primary_mem = primary_mem.clone();

        // Reconstruct skipped fields from module function body
        let func_addr = &module_inst.func_addrs[func_idx as usize];
        let func_inst = func_addr.read_lock();
        if let FuncInst::RuntimeFunc { code, .. } = func_inst {
            let body = code.body.clone();
            for label_stack in frame_stack.label_stack.iter_mut() {
                label_stack.processed_instrs = body.clone();
            }
            // v2 dispatcher: handler array (function pointers) — Rc<Vec<Handler>>
            frame_stack.handlers = code.handlers.clone();
        }

        // v2 dispatcher: cached raw pointer to memory data
        frame_stack.cached_mem_ptr = primary_mem.as_ref().map(|m| m.data_ptr());
    }
    println!("Frame module references and processed instructions restored.");

    println!("Restore successful (state applied to module). Returning Stacks.");
    Ok(state.stacks)
}
