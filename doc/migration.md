# Checkpoint and Restore for Live Migration

This document is about the checkpoint and restore mechanism of Chiwawa for live migration.

## Functions of Live Migration

Live migration moves a WebAssembly workload between hosts, and the workload continues from the point where it stopped.
Live migration helps in these conditions:

- **Edge-cloud continuum**: Computation moves between edge devices and the cloud.
- **Fault tolerance**: After a failure, a restore from a checkpoint starts the workload again.
- **Load balancing**: Workloads move across the infrastructure.
- **Analysis of defects**: A checkpoint captures the execution state for analysis.

## How It Works

### Checkpoint Process

1. **Trigger**: A monitor looks for the trigger file.
2. **Stop of all threads**: Each thread stops at an instruction boundary.
   See [How Each Thread Stops](#how-each-thread-stops).
3. **State capture**: Each thread serializes its state.
4. **Persistence**: The last thread that stops writes one file.
   The file contains the states of all threads and the shared memory.

### Captured State

A checkpoint captures these items:

- **For each thread**: the call stack (`func_idx`, program counter, and result registers for each frame), the register file, and the globals of that instance
- **Shared, saved one time**: the contents of the linear memory, the next thread id that `thread-spawn` gives, and the open files and sockets (see [Files and Sockets](#files-and-sockets))

Each instance has a different set of globals, because `__stack_pointer` has a different value in each thread.

The thread id is important because the pthread structures of the guest are in the restored memory.
Those structures continue to refer to the ids that they received.
A counter that starts again from zero gives the id of a live thread to a new thread.

A checkpoint does not contain the tables.
This is incorrect for a guest that writes tables, because `table.set` and `table.fill` write tables at run time.

### Restore Process

1. **Load**: Read the checkpoint file.
2. **Restore the shared memory**: Do this one time, before a thread runs.
3. **Open the files and sockets again**: Use their saved fd numbers.
4. **Restore the globals of the `_start` thread**: Apply the globals from its state to its module instance.
5. **Run the after-restore hook**: Do this only for `--hook after-restore`.
   See [Hook Points](#hook-points).
6. **Restore the thread id counter**.
7. **Make the other threads again**: Each thread gets a module instance and applies the globals from its state.
   It attaches `cached_mem_ptr` to the restored memory again, and it continues from its saved program counter.
8. **Resume the `_start` thread**: It attaches `cached_mem_ptr` to the memory again, and it continues from its saved program counter.

The hook returns before the other threads start.
The other threads start before the `_start` thread resumes.
When the `_start` thread returns, the process stops, and the other threads stop with it.

Each thread decodes its state, because `Stacks` contains `Rc` values and raw pointers.
Rust does not let these values go from one thread to a different thread.
The same limit applies during a save.
Thus the file contains one encoded blob for each thread, not one decoded structure.

Each frame records the `func_idx` of the function that it runs.
Thus it is not necessary to build the body and the handler array again, because `execute_frame` reads them from the module.
That index also gives the function of the frame at checkpoint time, without a search of `func_addrs`.
A frame also caches a pointer to that code.
Thus a return does not look up the code.
The checkpoint does not include the pointer, and a restored frame uses the index.

Chiwawa serializes the raw state, and it calculates again the data that it gets from raw pointers.
Thus the checkpoint stays small, and host pointers do not go into the file.

### Hook Points

A hook is an export of the guest that Chiwawa calls at a hook point.
Chiwawa has one hook point, `after-restore`.
With `--hook after-restore`, Chiwawa calls the export `chiwawa_after_restore` at step 5 of the restore process.
The export must have the type `() -> ()`.
No restored thread runs before the hook returns.

Before the hook runs, Chiwawa restores these items:

- the memory
- the open files and sockets
- the globals of the `_start` thread, for example `__stack_pointer`

Chiwawa finds the export before it reads the checkpoint file.
If the export is missing or has a different type, Chiwawa stops with an error, and the guest does not run.
If the hook traps, Chiwawa stops the restore.
A checkpoint cannot stop the hook.
You can use `--hook` only with `--restore`.
A component cannot use `--hook`.

The guest cannot control the instruction at which a checkpoint stops it.
Thus the hook has the limits of a signal handler.
The stopped code can hold a lock of libc, for example the lock of `malloc` or of stdio.
The hook must not call a function that uses such a lock.

The hook can run `memory.grow`.
Thus the `_start` thread attaches `cached_mem_ptr` to the memory again, after the hook returns.

## Trigger Mechanisms

WebAssembly has no signals.
Thus, to start a checkpoint, you make a trigger file (`checkpoint.trigger`).
The build target sets how Chiwawa finds that file.

### Thread-based (wasm32-wasip1-threads)

A background thread polls for the trigger file.
When that thread finds the file, it fills the handler table of each function with `checkpoint_trap` and wakes the parked threads.
The dispatcher does not poll.
Thus the checkpoint function adds no work to each instruction.

### Polling-based (wasm32-wasip1)

On hosts without threads, the dispatcher does the trigger check.
A WASI `path_exists` system call for each instruction uses too much time.
Thus `poll_checkpoint` keeps a counter (`VmState.checkpoint_poll_counter`).
It makes the system call only one time for each `CHECKPOINT_POLL_MASK + 1` (= 1024) instructions.
This limit keeps the hot dispatcher path short.
The time before the checkpoint starts is a maximum of 1024 instructions.

## How Each Thread Stops

Each thread must stop at an instruction boundary, from one of three states:

- **The thread runs**: The monitor fills each handler table with `checkpoint_trap`, and thus the next dispatched instruction traps.
- **The thread is parked in `memory.atomic.wait`**: The thread does not go to the dispatcher.
  Thus the monitor also notifies each shard of the parking table.
- **The thread waits for input in a WASI call**: It is not possible to wake the thread.
  Thus Chiwawa saves it where it is.
  See [Threads That Wait for Input](#threads-that-wait-for-input).

Chiwawa does not interrupt a woken thread that has a `notify` permit.
Without the permit, the count of `memory.atomic.notify` includes a wake-up that did not occur.
The other parked threads stop their `wait` without a change to `pc`.
Thus the restore runs the `wait` again.
The `wait` reads the address again, and then it reports not-equal or parks again with a new timeout.

Each thread that trapped serializes its state and then waits for the other threads.
The last thread that stops writes the file and releases the other threads.

### Threads That Wait for Input

It is not possible to wake a thread that is in a host WASI call.
WASI Preview 1 has no signal, and it has no method to cancel the call of a different thread.

`WasiFuncType::waits_for_input` gives the calls that wait for input:

- `poll_oneoff`
- `fd_read`
- `sock_accept`
- `sock_recv`
- the accept and receive calls of the socket extensions

A thread in one of these calls counts as stopped:

1. Before the call, `Runtime::run` moves the saved `ip` of the frame back to the call instruction.
2. `enter_input_wait` puts the thread in the registry.
3. The rendezvous is completed when each live thread is at the rendezvous or is in the registry.
4. The thread that writes the file serializes each thread in the registry.
   The state of a thread in the registry does not change during the call.
   Thus a different thread can read it.
5. When all threads are in the registry, no guest thread comes to the rendezvous.
   Thus the monitor thread writes the file.
6. The writer stops the process, because a thread in the registry does not return to stop it.

The restore runs the call again.
A timeout starts again from zero.
The same occurs for `memory.atomic.wait`.

Chiwawa keeps the result of a call that returns before the write of the file.
`leave_input_wait` removes the thread from the registry, and Chiwawa stores the result.
The thread then traps at its next instruction.

Chiwawa does not keep the result of a call that returns during the write of the file, or the input that the call consumed.
The process stops before Chiwawa stores the result.

Limits:

- This procedure does not apply to a call that sends output (`fd_write`, `sock_send`).
  Such a call can complete during the write of the file, and then it runs a second time after the restore.
  The checkpoint waits until such a call returns.
- The polling-based build has no monitor thread.
  Its one thread finds the trigger only after the call returns.

## Files and Sockets

A restore is a new process.
In that process, the fd numbers in the memory of the guest refer to no open file or socket.
`src/wasi/descriptors.rs` keeps a record of each fd that the guest opened, and the checkpoint saves the record.
The record is outside the WASI passthrough.

### One entry point

Each WASI call goes through `descriptors::call_with_host_fds`, which `Runtime::call_wasi_function` calls.
It does three steps:

1. It replaces each guest fd number in the parameters with the number of the host.
2. It runs the call.
3. After a call without an error, it updates the record and the table.

No other code uses a guest fd number.
The `Backend` with the name `Recorded` also records the socket calls.
It gets each address in the shared representation of `src/wasi/socket.rs`.

### Contents of the record

- **Files**: the parameters of `path_open`, subsequent `fd_fdstat_set_flags` calls, and the offset that `fd_tell` reports at the time of the checkpoint.
- **Sockets**: the family and type of `sock_open`, each option that the guest set, and the addresses of `bind`, `listen`, and `connect`.
  For a bind to port 0, the record contains the port that the host selected.
  A socket keeps one entry for each multicast group that the guest added with `IP_ADD_MEMBERSHIP`.
  `IP_DROP_MEMBERSHIP` removes that entry.

`fd_close` removes an entry, and `fd_renumber` moves it.

### Restore

`descriptors::restore` does the operations of the record again, in the sequence in which the guest opened the fds:

1. It opens the file or socket again.
   It opens a file without `O_EXCL` and `O_TRUNC`.
   Thus the data that the guest wrote before the checkpoint stays.
2. For a file, it seeks to the saved offset.
   For a socket, it sets the options, and then it does `bind`, `listen`, and `connect`.
3. The table maps the saved fd number to the new fd.

Chiwawa reports an fd that it cannot restore, and the guest gets `EBADF` for that number.

### The fd table

The host selects the number of a new fd, and no host lets the guest select it:

- wasmtime with threads does not use a number again.
- WAMR selects a random free number.
- WasmEdge selects a random number below 2^31.

`fd_renumber` does not help, because WAMR and WasmEdge do not accept a free target.
Thus the table maps the number that the guest knows to the number that the host gave.
A number that is not in the table is the same on the two sides.
Before the first restore, this is true for all numbers.

- A host fd that has a mapping gives `EBADF` for its host number, because the guest did not receive that number.
- When the host gives a number that the guest uses at that time, the guest gets a different free number, and the table maps it.
- `poll_oneoff` has fds in its subscriptions.
  Chiwawa translates them in a copy.
  Thus the memory of the guest keeps the guest numbers, and a checkpoint during the call saves them.

### Limits

- After the restore, a TCP connection is a new connection, and the peer gets a reconnect.
- Chiwawa does not restore a socket that `sock_accept` returned.
- The data in the socket buffers of the host does not stay.
- A file must be at the same path in the same preopened directory.
- The preopened directories must have the same fd numbers in the two processes.

The socket record uses the shared representation.
Thus a checkpoint from one socket host restores on the other.

## Runtime Neutrality

Chiwawa is self-hosted (it runs as WebAssembly).
Thus checkpoints are portable across different host runtimes.
You can restore a checkpoint from Wasmtime on WasmEdge, Wasmtime, or a different runtime that implements WASI.
