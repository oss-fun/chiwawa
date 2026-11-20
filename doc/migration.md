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
- **Shared, saved once**: the linear memory contents, the next thread id `thread-spawn` would hand out, and the open files and sockets (see [Files and Sockets](#files-and-sockets))

Globals are per-instance rather than shared: `__stack_pointer` holds a different value in every thread.

The thread id matters because the guest's own pthread structures live in the restored memory and still refer to the ids they were given.
Restarting the counter would hand a live thread's id to a new one.

Tables are left out, which is wrong for a guest that writes them: `table.set` and `table.fill` write tables at runtime.

### Restore Process

1. **Load**: Read checkpoint file
2. **Restore the shared memory**: once, before any thread runs
3. **Open the files and sockets again**: under their saved fd numbers
4. **Restore the thread id counter**
5. **Recreate the other threads**: each gets its own module instance, applies its own globals, re-attaches `cached_mem_ptr` to the restored memory, and resumes from its saved program counter
6. **Resume this thread**: the one that ran `_start`

The other threads start before the `_start` thread resumes, because that thread returning ends the process and would take the others with it.

Each thread decodes its own state, because `Stacks` holds `Rc`s and raw pointers that Rust will not let cross a thread boundary.
The same constraint applies while saving, so the file holds one encoded blob per thread rather than one decoded structure.

Each frame records the `func_idx` of the function it runs, so the body and handler array need no reconstruction: `execute_frame` reads them from the module.
That index also identifies the frame's function at checkpoint time, with no search over `func_addrs`.
A frame also caches a pointer to that code so a return does not look it up; the pointer is skipped by the checkpoint and a restored frame falls back to the index.

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

Every thread has to stop at an instruction boundary, from one of three places:

- **Running**: the monitor fills every handler table with `checkpoint_trap`, so the next instruction dispatched traps.
- **Parked in `memory.atomic.wait`**: the thread never reaches the dispatcher, so the monitor also notifies every shard of the parking table.
- **Waiting for input in a WASI call**: nothing can wake the thread, so it is saved where it is.
  See [Threads Waiting for Input](#threads-waiting-for-input).

A woken waiter holding a `notify` permit is not interrupted, because dropping it would lose a wake-up `memory.atomic.notify` already counted.
The rest leave without advancing `pc`, so the restore runs the `wait` again: it re-reads the address and either reports not-equal or parks again, its timeout restarted.

Each trapped thread serializes its own state and then waits for the others.
The last one to arrive writes the file and releases them.

### Threads Waiting for Input

A thread inside a host WASI call cannot be woken.
WASI Preview 1 has no signal and no way to cancel another thread's call.

`WasiFuncType::waits_for_input` names the calls that wait for input:

- `poll_oneoff`
- `fd_read`
- `sock_accept`
- `sock_recv`
- the accept and receive calls of the socket extensions

A thread inside one of them counts as stopped:

1. Before the call, `Runtime::run` moves the frame's saved `ip` back to the call instruction.
2. `enter_input_wait` registers the thread.
3. The rendezvous is complete when every live thread has arrived or is registered.
4. The thread that writes the file serializes each registered thread.
   A registered thread's state does not change during the call, so another thread can read it.
5. The monitor thread writes the file when every thread is registered, because no guest thread arrives.
6. The writer ends the process, because a registered thread never returns to end it.

The restore runs the call again.
A timeout starts over, as it does for `memory.atomic.wait`.

A call that returns before the file is written is not lost.
`leave_input_wait` removes the thread from the registry and the result is stored.
The thread then traps at its next instruction.

A call that returns while the file is being written is lost, together with any input it consumed.
The process ends before the result is stored.

Limits:

- A call that sends output (`fd_write`, `sock_send`) is not treated this way.
  One that completed while the file was being written would run a second time after the restore.
  A thread blocked in such a call holds the checkpoint back until the call returns.
- The polling-based build has no monitor thread.
  Its one thread sees the trigger only after the call returns.

## Files and Sockets

A restore is a new process, where the fd numbers in the guest's memory name nothing.
`src/wasi/descriptors.rs` keeps a record of every fd the guest opened, and the checkpoint saves it.
The record lives outside the WASI passthrough.

### One entry point

Every WASI call goes through `descriptors::call_with_host_fds`, called from `Runtime::call_wasi_function`.
It does three things:

1. It replaces each guest fd number in the parameters with the host's number.
2. It runs the call.
3. After a successful call, it updates the record and the table.

No other code handles a guest fd number.
The socket calls are also recorded by the `Backend` named `Recorded`, which sees each address in the common form of `src/wasi/socket.rs`.

### What is recorded

- **Files**: the parameters of `path_open`, later `fd_fdstat_set_flags` calls, and the offset `fd_tell` reports when the checkpoint is written.
- **Sockets**: the family and type of `sock_open`, each option set, and the addresses of `bind`, `listen` and `connect`.
  A bind to port 0 records the port the host chose.
  A socket keeps one entry per multicast group it joined, and leaving a group removes that entry.

`fd_close` removes an entry and `fd_renumber` moves it.

### Restore

`descriptors::restore` replays the record in the order the fds were opened:

1. It opens the file or socket again.
   A file is opened without `O_EXCL` and `O_TRUNC`, so the data it already wrote stays.
2. A file seeks to its saved offset.
   A socket gets its options, then `bind`, `listen` and `connect`.
3. The table maps the saved fd number to the new fd.

An fd that cannot be restored is reported, and the guest sees `EBADF` for its number.

### The fd table

The host picks the number of a new fd, and no host lets the guest choose it:

- wasmtime with threads never reuses a number.
- WAMR picks a random free number.
- WasmEdge picks a random number below 2^31.

`fd_renumber` does not help: WAMR and WasmEdge refuse a free target.
The table therefore maps the number the guest remembers to the number the host handed out.
A number missing from the table is the same on both sides, which is every number before the first restore.

- A host fd behind a mapping is `EBADF` under its own number, because the guest was never given that number.
- When the host hands out a number the guest already uses, the guest gets another free number and the table maps it.
- `poll_oneoff` carries fds inside its subscriptions.
  They are translated in a copy, so the guest's memory keeps the guest's numbers and a checkpoint during the call saves them.

### Limits

- A TCP connection is a new connection after the restore, so the peer sees a reconnect.
- A socket returned by `sock_accept` is not restored.
- Data in the host's socket buffers is lost.
- A file must exist at the same path under the same preopened directory.
- The preopened directories must have the same fd numbers in both processes.

The socket record uses the common form, so a checkpoint from one socket host restores on the other.

## Runtime Neutrality

Because Chiwawa is self-hosted (runs as WebAssembly itself), checkpoints are portable across different host runtimes.
A checkpoint created on Wasmtime can be restored on WasmEdge, Wasmtime or any other WASI-compliant runtime.