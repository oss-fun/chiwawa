use crate::error::RuntimeError;
use crate::execution::global::GlobalAddr;
use crate::execution::mem::MemAddr;
use crate::execution::module::ModuleInst;
use crate::execution::table::TableAddr;
use crate::execution::value::{Ref, Val};
use crate::execution::vm::{Frame, Stacks};
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

pub fn setup_checkpoint_monitor() {
    thread::spawn(|| loop {
        if std::path::Path::new(CHECKPOINT_TRIGGER_FILE).exists() {
            CHECKPOINT_TRIGGERED.store(true, Ordering::Relaxed);
            let _ = std::fs::remove_file(CHECKPOINT_TRIGGER_FILE);
        }
        thread::sleep(Duration::from_millis(10));
    });
}

#[inline(always)]
pub fn check_checkpoint_flag() -> bool {
    CHECKPOINT_TRIGGERED.load(Ordering::Relaxed)
}

pub fn check_checkpoint_trigger(frame: &Frame) -> Result<bool, RuntimeError> {
    if let Some(module) = frame.module.upgrade() {
        if let Some(ref wasi) = module.wasi_impl {
            if wasi.check_file_exists(CHECKPOINT_TRIGGER_FILE) {
                let _ = std::fs::remove_file(CHECKPOINT_TRIGGER_FILE);
                return Ok(true);
            }
        }
    }
    Ok(false)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableState {
    pub stacks: Stacks,
    pub memory_data: Vec<u8>,
    pub global_values: Vec<Val>,
    pub tables_data: Vec<Vec<Option<u32>>>,
}

pub fn checkpoint<P: AsRef<Path>>(
    module_inst: &ModuleInst,
    stacks: &Stacks,
    mem_addrs: &[MemAddr],
    global_addrs: &[GlobalAddr],
    table_addrs: &[TableAddr],
    output_path: P,
) -> Result<(), RuntimeError> {
    println!("Checkpointing state to {:?}...", output_path.as_ref());

    // 1. Gather Memory state
    let memory_data = if let Some(mem_addr) = mem_addrs.get(0) {
        mem_addr.get_data()?
    } else {
        Vec::new()
    };

    // 2. Gather Global state
    let global_values = global_addrs
        .iter()
        .map(|global_addr| Ok(global_addr.get()))
        .collect::<Result<Vec<Val>, RuntimeError>>()?;

    // 3. Gather Table state (using Arc::ptr_eq)
    let tables_data = table_addrs
        .iter()
        .map(|table_addr| -> Result<Vec<Option<u32>>, RuntimeError> {
            let table_inst = table_addr.read_lock();
            let mut table_indices = Vec::with_capacity(table_inst.elem.len());
            for val in table_inst.elem.iter() {
                if let Val::Ref(Ref::FuncAddr(target_func_addr)) = val {
                    let found_index = module_inst.func_addrs.iter().position(|module_func_addr| {
                        Rc::ptr_eq(target_func_addr.get_rc(), module_func_addr.get_rc())
                    });
                    if let Some(index) = found_index {
                        table_indices.push(Some(index as u32));
                    } else {
                        eprintln!("Warning: FuncAddr in table not found in module func_addrs during checkpoint.");
                        table_indices.push(None);
                    }
                } else {
                    table_indices.push(None);
                }
            }
            Ok(table_indices)
        })
        .collect::<Result<Vec<Vec<Option<u32>>>, _>>()?;

    // 4. Assemble state
    let state = SerializableState {
        stacks: stacks.clone(),
        memory_data,
        global_values,
        tables_data,
    };

    // 5. Serialize and write
    let encoded: Vec<u8> =
        bincode::serialize(&state).map_err(|e| RuntimeError::SerializationError(e.to_string()))?;
    let mut file =
        File::create(output_path).map_err(|e| RuntimeError::CheckpointSaveError(e.to_string()))?;
    file.write_all(&encoded)
        .map_err(|e| RuntimeError::CheckpointSaveError(e.to_string()))?;

    println!("Checkpoint successful.");
    Ok(())
}

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

    // 3. Restore memory state into module_inst
    // Assuming only one memory instance for now
    if let Some(mem_addr) = module_inst.mem_addrs.get(0) {
        mem_addr.set_data(state.memory_data)?;
        println!("Memory state restored into module instance.");
    } else if !state.memory_data.is_empty() {
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

    // 5. Restore table state into module_inst
    if module_inst.table_addrs.len() == state.tables_data.len() {
        for (table_idx, table_indices) in state.tables_data.iter().enumerate() {
            let table_addr = &module_inst.table_addrs[table_idx];
            let restored_elements = table_indices
                .iter()
                .map(|maybe_index| {
                    maybe_index
                        .map(|index| {
                            module_inst
                                .func_addrs
                                .get(index as usize)
                                .cloned()
                                .ok_or_else(|| {
                                    RuntimeError::RestoreError(format!(
                                        "Invalid function index {} found in table data",
                                        index
                                    ))
                                })
                        })
                        .transpose()
                })
                .collect::<Result<Vec<Option<_>>, _>>()?;
            table_addr.set_elements(restored_elements)?;
        }
        println!("Table state restored into module instance.");
    } else {
        eprintln!(
            "Warning: Mismatch in table count between module ({}) and checkpoint ({}). Tables not restored.",
            module_inst.table_addrs.len(),
            state.tables_data.len()
        );
    }

    // 6. Reconstruct skipped fields in Stacks (Frame::module)
    for frame_stack in state.stacks.activation_frame_stack.iter_mut() {
        frame_stack.frame.module = Rc::downgrade(&module_inst);
    }
    println!("Frame module references restored.");

    println!("Restore successful (state applied to module). Returning Stacks.");
    Ok(state.stacks)
}
