# Tail-Call Dispatcher (`tco` feature)

This document is about the tail-call dispatcher of Chiwawa.
It is an optional build mode.
It uses the WebAssembly tail-call proposal to remove the overhead of the dispatch loop from the threaded code interpreter.

## Problem

The default dispatcher (the loop dispatcher) runs the usual threaded code loop:

```rust
loop {
    let h = handlers[pc];
    h(&mut state);
}
```

In a self-hosted runtime, this loop becomes many host Wasm instructions *for each guest instruction*.
These instructions are the fetch, the indirect call, the return, and the branch back to the top.
Tail-call dispatch replaces the fetch, the indirect call, and the return with one `return_call_indirect`.
Thus each handler gives control directly to the next handler, and control does not return to a dispatcher loop.

For guest code that is almost all arithmetic, this change decreases the dispatch paths from approximately two to one.

## Build Modes

The `tco` Cargo feature selects the dispatcher at build time.
The cargo aliases set the correct target features and the build directory:

| Mode | Cargo feature | Build alias | `+tail-call` target-feature |
|---|---|---|---|
| Loop dispatch | (none) | `cargo build-legacy` | off |
| Tail-call dispatch | `tco` | `cargo build-tco` | on |

Each mode has a `-threads` variant (`build-legacy-threads`, `build-tco-threads`).
These variants also enable `+atomics` for the `wasm32-wasip1-threads` target.

Each of the two dispatcher modules has a `cfg` condition.
Thus the build uses only one of them:

```rust
// execution/dispatch.rs
#[cfg(not(feature = "tco"))] pub use crate::execution::dispatch_loop::execute_instructions;
#[cfg(feature = "tco")]      pub use crate::execution::dispatch_tco::execute_instructions;
```

## Dispatch Structure

### Loop dispatcher (default)

`dispatch_loop::execute_instructions` is a threaded code loop.
Each iteration does these steps:

1. It polls the checkpoint trigger.
2. It fetches a handler function pointer from the `handlers: Rc<Vec<Handler>>` array of the function.
3. It calls the handler.
4. It examines the `Outcome` that the handler returns.

### TCO dispatcher

`dispatch_tco::execute_instructions` makes the first call into the handler chain.
After that call, control goes from handler to handler through tail calls.
Control does not return until an outcome other than `Continue` stops the chain.
The `advance!` macro makes this connection between handlers.

When execution goes into or out of a frame, the handler returns `Continue`, and `execute_instructions` starts the chain again.
Thus a chain covers a maximum of one function body.
Rust does not make sure that a call is a tail call.
For a chain that includes more than one frame, it is possible that LLVM does not emit `return_call_indirect`.
Then the host stack depth follows the guest call depth, and a correct result is possible only with an optimization.

## The `advance!` Macro and the `next_handler` Shim

The last operation of each handler is `advance!(state)`.
The macro has two expansions, and `cfg(feature = "tco")` selects one of them:

```rust
// non-tco: handlers return Continue to the loop driver.
#[cfg(not(feature = "tco"))]
macro_rules! advance { ($s:expr) => { Outcome::Continue }; }

// tco: handlers tail-call the next handler.
#[cfg(feature = "tco")]
macro_rules! advance {
    ($s:expr) => {{
        let h = unsafe { handlers::next_handler($s) };
        h($s)
    }};
}
```

## Sentinel Handlers

Sentinels are small handlers that stop the tail-call chain.
A sentinel returns an `Outcome` and does not call `advance!`:

- `halt`: the function body came to its end (`Outcome::Halt`).
- `trap`: a general trap.
  `state.trap` contains the `RuntimeError` before this handler runs.
- `checkpoint_trap`: `next_handler` selects it when `poll_checkpoint` reports a checkpoint trigger.
  It writes `RuntimeError::CheckpointRequested` to `state.trap` and returns `Outcome::Trap`.
  Because this sentinel is a trap, `advance!` keeps its one tail-call site.
- `r#yield`: the runtime yields (WASI and host calls).
  The `ModuleLevelInstr` is in `state.yielded`.
  Wasm calls and returns do not yield, because the handlers push and pop the frames.

The `handlers: Rc<Vec<Handler>>` array of each function has `body.len() + 1` entries, and `halt` is in the last position.
Thus a dispatch that is out of range in TCO mode goes to `halt` and stops correctly.
