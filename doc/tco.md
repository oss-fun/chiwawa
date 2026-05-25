# Tail-Call Dispatcher (`tco` feature)

This document describes Chiwawa's tail-call dispatcher, an opt-in build
mode that uses the WebAssembly tail-call proposal to remove dispatch-loop
overhead from the threaded code interpreter.

## Motivation

The default (loop-style) dispatcher runs the classic threaded code shape:

```rust
loop {
    let h = handlers[pc];
    h(&mut state);
}
```

In a self-hosted runtime this loop expands to many host Wasm instructions
*per guest instruction* (the fetch, the indirect call, the return, the
branch back to the top). Tail-call dispatch collapses fetch → indirect call
→ return into a single `return_call_indirect`, so each handler hands
control directly to the next without ever returning to a dispatcher loop.

For arithmetic-heavy guest code, removing those per-instruction loop
operations is the difference between roughly two dispatch paths and one.

## Build Modes

The dispatcher is selected at build time by the `tco` Cargo feature.
Higher-level cargo aliases pre-bake the right target features and build
directory:

| Mode | Cargo feature | Build alias | `+tail-call` target-feature |
|---|---|---|---|
| Loop dispatch | (none) | `cargo build-legacy` | off |
| Tail-call dispatch | `tco` | `cargo build-tco` | on |

`-threads` variants exist for both modes (`build-legacy-threads`,
`build-tco-threads`) and additionally enable `+atomics` for the
`wasm32-wasip1-threads` target.

The two dispatcher modules are cfg-gated so the build picks exactly one:

```rust
// execution/dispatch.rs
#[cfg(not(feature = "tco"))] pub use crate::execution::dispatch_loop::execute_instructions;
#[cfg(feature = "tco")]      pub use crate::execution::dispatch_tco::execute_instructions;
```

## Dispatch Structure

### Loop dispatcher (default)

`dispatch_loop::execute_instructions` is a straight threaded-code loop. Each
iteration polls the checkpoint trigger, fetches a handler function pointer
from the function-local `handlers: Rc<Vec<Handler>>` array, calls it, and
inspects the returned `Outcome`.

### TCO dispatcher

`dispatch_tco::execute_instructions` only fires the *initial* call into the
handler chain. Once entered, control flows handler-to-handler via tail calls
and never returns to `execute_instructions` until a sentinel (`Halt` / `Trap`
/ `Yield`) terminates the chain. The mechanism that wires this up is the
`advance!` macro.

## The `advance!` Macro and the `next_handler` Shim

Every handler ends with `advance!(state)`. The macro has two expansions
chosen by `cfg(feature = "tco")`:

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

Sentinels are tiny handlers that terminate the tail-call chain by returning
an `Outcome` instead of calling `advance!`:

- `halt` — function body ended naturally (`Outcome::Halt`).
- `trap` — generic trap; `state.trap` already holds the `RuntimeError`.
- `checkpoint_trap` — selected by `next_handler` when `poll_checkpoint`
  signals a request; writes `RuntimeError::CheckpointRequested` to
  `state.trap` and returns `Outcome::Trap`. Keeping this on the trap side
  preserves the single tail-call site in `advance!`.
- `r#yield` — runtime yield (call / call_wasi / return); the
  `ModuleLevelInstr` is in `state.yielded`.

The per-function `handlers: Rc<Vec<Handler>>` array is sized
`body.len() + 1`, with `halt` planted at the final position so any
out-of-range dispatch in TCO mode lands on `halt` and terminates cleanly.

