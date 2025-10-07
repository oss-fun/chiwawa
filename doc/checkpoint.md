# Checkpoint and Restore in Chiwawa

Chiwawa provides live migration capabilities through checkpoint and restore functionality. This allows capturing the complete execution state of a running WebAssembly program and restoring it later, enabling process migration, fault tolerance, and debugging scenarios.

## Overview

**Checkpoint** serializes the entire runtime state (stacks, memory, globals, tables) to a binary file.

**Restore** deserializes the checkpoint file and reconstructs the exact execution state, allowing the program to resume from where it was checkpointed.

**Implementation:** `src/execution/migration.rs`

## Architecture

### 1. Serializable State

The complete execution state is captured in `SerializableState`:

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableState {
    pub stacks: Stacks,              // Activation frames, label stacks, value stacks
    pub memory_data: Vec<u8>,        // Linear memory contents
    pub global_values: Vec<Val>,     // Global variable values
    pub tables_data: Vec<Vec<Option<u32>>>, // Table elements (as function indices)
}
```

**Components:**

- **`stacks`**: All execution stacks including:
  - Activation frame stack (`Vec<FrameStack>`)
  - Each frame's label stack and value stack
  - Local variables and instruction pointers
  - Processed instructions (DTC code)

- **`memory_data`**: Complete linear memory contents as raw bytes

- **`global_values`**: All global variable values (mutable and immutable)

- **`tables_data`**: Table elements stored as function indices
  - Function addresses converted to indices for serialization
  - Reconstructed to FuncAddr references during restore

### 2. Stacks Structure

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stacks {
    pub activation_frame_stack: Vec<FrameStack>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameStack {
    pub frame: Frame,                // Function frame (locals, module ref)
    pub label_stack: Vec<LabelStack>, // Control flow labels
    pub void: bool,                   // Whether function returns void
    pub instruction_count: u64,       // Execution counter
    pub global_value_stack: Vec<Val>, // Value stack for this frame

    // Skipped during serialization (reconstructed on restore)
    #[serde(skip)]
    pub current_block_accessed_globals: Option<GlobalAccessTracker>,
    #[serde(skip)]
    pub current_block_accessed_locals: Option<LocalAccessTracker>,
    #[serde(skip)]
    pub enable_checkpoint: bool,
}
```

**Key Points:**

- `Frame::module` (Weak reference) is skipped during serialization and reconstructed during restore

### 3. Checkpoint Triggering

Checkpoints are triggered by file-based signaling:

**Trigger File:** `./checkpoint.trigger`

**The checkpoint triggering mechanism differs based on the build target:**

#### wasm32-wasip1-threads: Background Thread Monitor (Recommended)
```rust
pub fn setup_checkpoint_monitor() {
    thread::spawn(|| loop {
        if std::path::Path::new(CHECKPOINT_TRIGGER_FILE).exists() {
            CHECKPOINT_TRIGGERED.store(true, Ordering::Relaxed);
            let _ = std::fs::remove_file(CHECKPOINT_TRIGGER_FILE);
        }
        thread::sleep(Duration::from_millis(10));
    });
}
```

**Build target:** `wasm32-wasip1-threads`

**How it works:**
- Background thread spawned at runtime initialization
- Checks for trigger file every 10ms
- Sets atomic flag (`CHECKPOINT_TRIGGERED`) when file is detected
- Automatically removes trigger file
- Main execution loop checks atomic flag periodically

**Advantages:**
- Low overhead (atomic flag check is very fast)
- Minimal impact on execution performance
- Responsive checkpoint triggering

#### wasm32-wasip1: WASI-based File Check (Fallback)
```rust
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
```

**Build target:** `wasm32-wasip1`

**How it works:**
- No background thread (wasm32-wasip1 doesn't support threads)
- Checks for trigger file via WASI file system interface
- Only checked at specific points: before function calls
- Removes trigger file after detection

**Disadvantages:**
- Higher overhead (file system check is slower than atomic flag)
- Only checked before function calls (less frequent)
- May miss checkpoint trigger if stuck in long-running computation

**When to use:**
- For compatibility with host runtimes that don't support wasm32-wasip1-threads

### 4. Checkpoint Process

The `checkpoint()` function captures the complete state:

```rust
pub fn checkpoint<P: AsRef<Path>>(
    module_inst: &ModuleInst,
    stacks: &Stacks,
    mem_addrs: &[MemAddr],
    global_addrs: &[GlobalAddr],
    table_addrs: &[TableAddr],
    output_path: P,
) -> Result<(), RuntimeError>
```

**What it does:**

1. **Gather Memory State** - Copies entire linear memory contents
2. **Gather Global State** - Collects all global variable values
3. **Gather Table State** - Converts table FuncAddr references to function indices
4. **Assemble State** - Creates `SerializableState` with all components
5. **Serialize and Write** - Encodes with `bincode` and writes to file

### 5. Restore Process

The `restore()` function reconstructs the execution state:

```rust
pub fn restore<P: AsRef<Path>>(
    module_inst: Rc<ModuleInst>,
    input_path: P,
) -> Result<Stacks, RuntimeError>
```

**What it does:**

1. **Read Checkpoint File** - Loads serialized data from file
2. **Deserialize State** - Decodes `SerializableState` using `bincode`
3. **Restore Memory State** - Copies memory data back into module instance
4. **Restore Global State** - Sets all global variable values
5. **Restore Table State** - Converts function indices back to FuncAddr references
6. **Reconstruct Frame Module References** - Rebuilds weak references to module (critical step)