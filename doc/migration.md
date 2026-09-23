# Checkpoint and Restore for Live Migration

This document explains Chiwawa's checkpoint/restore mechanism for live migration.

## Motivation

Live migration enables moving WebAssembly workloads between hosts and resuming execution from where it left off.
This is valuable for:

- **Edge-cloud continuum**: Moving computation between edge devices and cloud
- **Fault tolerance**: Recovering from failures by restoring checkpoints
- **Load balancing**: Redistributing workloads across infrastructure
- **Debugging**: Capturing execution state for analysis

## How It Works

### Checkpoint Process

1. **Trigger Detection**: a monitor watches for the trigger file
2. **Stop every thread**: at an instruction boundary, see [Stopping Every Thread](#stopping-every-thread)
3. **State Capture**: each thread serializes its own state
4. **Persistence**: the last thread to arrive writes one file holding all of them plus the shared memory

### Captured State

A checkpoint captures:

- **Per thread**: the call stack (per-frame `func_idx`, program counter, result registers), the register file, and that instance's globals
- **Shared, saved once**: the linear memory contents, and the next thread id `thread-spawn` would hand out

Globals are per-instance rather than shared: `__stack_pointer` holds a different value in every thread.

The thread id matters because the guest's own pthread structures live in the restored memory and still refer to the ids they were given.
Restarting the counter would hand a live thread's id to a new one.

Tables and WASI state are left out, and both are wrong for a guest that uses them: `table.set` and `table.fill` write tables at runtime, and a restore is a new process whose file descriptors have nothing to do with the saved ones.

### Restore Process

1. **Load**: Read checkpoint file
2. **Restore the shared memory**: once, before any thread runs
3. **Restore the thread id counter**
4. **Recreate the other threads**: each gets its own module instance, applies its own globals, re-attaches `cached_mem_ptr` to the restored memory, and resumes from its saved program counter
5. **Resume this thread**: the one that ran `_start`

The other threads start before the `_start` thread resumes, because that thread returning ends the process and would take the others with it.

Each thread decodes its own state, because `Stacks` holds `Rc`s and raw pointers that Rust will not let cross a thread boundary.
The same constraint applies while saving, so the file holds one encoded blob per thread rather than one decoded structure.

Each frame records the `func_idx` of the function it runs, so the body and handler array need no reconstruction: `execute_frame` reads them from the module.
That index also identifies the frame's function at checkpoint time, with no search over `func_addrs`.

This split (serialize raw state vs. re-derive what depends on raw pointers) keeps the checkpoint small and avoids leaking host pointers into the file.

## Trigger Mechanisms

WebAssembly has no signal handling, so a checkpoint is requested by creating a trigger file (`checkpoint.trigger`).
How that file is noticed depends on the build target.

### Thread-based (wasm32-wasip1-threads)
A background thread polls for the trigger file.
When it appears, that thread fills every function's handler table with `checkpoint_trap` and wakes the parked waiters.
The dispatcher itself polls nothing, so checkpointing costs nothing per instruction.

### Polling-based (wasm32-wasip1)
On hosts without thread support, the dispatcher itself does the trigger check.
Issuing a WASI `path_exists` syscall on every instruction would be too expensive, so `poll_checkpoint` keeps a counter (`VmState.checkpoint_poll_counter`) and only fires the syscall once every `CHECKPOINT_POLL_MASK + 1` (= 1024) instructions.
The throttle keeps the hot dispatcher path tight while still bounding checkpoint latency to a small, fixed number of instructions.

## Stopping Every Thread

Every thread has to stop at an instruction boundary, from one of two places:

- **Running**: the monitor fills every handler table with `checkpoint_trap`, so the next instruction dispatched traps.
- **Parked in `memory.atomic.wait`**: the thread never reaches the dispatcher, so the monitor also notifies every shard of the parking table.

A woken waiter holding a `notify` permit is not interrupted, because dropping it would lose a wake-up `memory.atomic.notify` already counted.
The rest leave without advancing `pc`, so the restore runs the `wait` again: it re-reads the address and either reports not-equal or parks again, its timeout restarted.

Each trapped thread serializes its own state and then waits for the others.
The last one to arrive writes the file and releases them.
A thread inside a WASI call only stops when the call returns, so a checkpoint cannot complete while one is blocked on input that never arrives.

## Runtime Neutrality

Because Chiwawa is self-hosted (runs as WebAssembly itself), checkpoints are portable across different host runtimes.
A checkpoint created on Wasmtime can be restored on WasmEdge, Wasmtime or any other WASI-compliant runtime.