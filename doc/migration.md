# Checkpoint and Restore for Live Migration

This document explains Chiwawa's checkpoint/restore mechanism for live migration.

## Motivation

Live migration enables moving WebAssembly workloads between hosts and resuming execution from where it left off. This is valuable for:

- **Edge-cloud continuum**: Moving computation between edge devices and cloud
- **Fault tolerance**: Recovering from failures by restoring checkpoints
- **Load balancing**: Redistributing workloads across infrastructure
- **Debugging**: Capturing execution state for analysis

## How It Works

### Checkpoint Process

1. **Trigger Detection**: Monitor for checkpoint signal
2. **State Capture**: Gather all runtime state
3. **Serialization**: Convert state to binary format
4. **Persistence**: Write to checkpoint file

### Captured State

A checkpoint captures:

- **Execution State**: Call stack, program counters, register values
- **Memory**: Complete linear memory contents
- **Globals**: All global variable values

Tables are intentionally excluded. They are deterministically initialized
from the module's element segments at instantiation time, so the original
contents can always be reproduced from the (immutable) module bytes —
serializing them would only bloat the checkpoint.

### Restore Process

1. **Load**: Read checkpoint file
2. **Deserialize**: Reconstruct state structures
3. **Apply**: Restore memory and globals to the module instance
4. **Rebuild derived state**: Re-attach fields that the checkpoint deliberately
   skipped (they can be re-derived from the module):
   - `processed_instrs` — refilled from each frame's function body
   - `handlers` — per-frame handler function-pointer array, refilled from
     `Func.handlers`
   - `primary_mem` / `cached_mem_ptr` — re-cached from the freshly restored
     memory instance
   - `Frame.module` — re-linked to the live `ModuleInst`
5. **Resume**: Continue execution from the saved program counter

This split (serialize raw state vs. re-derive what depends on `Rc`/raw
pointers) keeps the checkpoint small and avoids leaking host pointers into
the file.

## Trigger Mechanisms

Traditional checkpoint systems use signals (e.g., SIGUSR1) to trigger checkpoints. However, WebAssembly's sandboxed execution model does not support signal handling. Chiwawa uses file-based triggers instead: the presence of a trigger file (`checkpoint.trigger`) signals that a checkpoint should be taken.

Chiwawa supports two detection mechanisms:

### Thread-based (wasm32-wasip1-threads)
A background thread polls for the trigger file and toggles an atomic flag
(`CHECKPOINT_TRIGGERED`). The dispatcher's per-instruction
`poll_checkpoint` hook then only needs a cheap relaxed atomic load to detect
the request, so checkpointing introduces virtually no per-instruction overhead.

### Polling-based (wasm32-wasip1)
On hosts without thread support, the dispatcher itself does the trigger
check. Issuing a WASI `path_exists` syscall on every instruction would be
too expensive, so `poll_checkpoint` keeps a counter
(`VmState.checkpoint_poll_counter`) and only fires the syscall once every
`CHECKPOINT_POLL_MASK + 1` (= 1024) instructions. The throttle keeps the
hot dispatcher path tight while still bounding checkpoint latency to a
small, fixed number of instructions.

## Runtime Neutrality

Because Chiwawa is self-hosted (runs as WebAssembly itself), checkpoints are portable across different host runtimes. A checkpoint created on Wasmtime can be restored on WasmEdge, Wasmtime or any other WASI-compliant runtime.