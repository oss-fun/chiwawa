# chiwawa(CHeckpoint/restore and Instrumentation-specific WAsm runtime on WAsm runtime)

Chiwawa (the Japanese pronunciation of chihuahua) is a self-hosted Wasm runtime.
It makes live migration and instrumentation possible.
The execution method (for example, interpreter, JIT, or AOT), the architecture, and the host runtime have no effect on these functions.

U^ｪ^U

## Build and Run

Select the build alias for a host Wasm runtime with or without the tail-call proposal (see [Dispatcher Modes](#dispatcher-modes)):

```bash
# Tail-call dispatcher
cargo build-tco           # → target/tco/wasm32-wasip1/release/chiwawa.wasm
# Loop dispatcher
cargo build-legacy        # → target/legacy/wasm32-wasip1/release/chiwawa.wasm

# Call function with Wasm parameters (I32, I64, F32, F64)
somethingWasmRuntime target/<combo>/wasm32-wasip1/release/chiwawa.wasm test.wasm --invoke func-name --params "I64(100)"

# Pass command-line arguments to WASI-compiled program
somethingWasmRuntime target/<combo>/wasm32-wasip1/release/chiwawa.wasm test.wasm --app-args "--version"
```

## Dispatcher Modes

Chiwawa has two dispatcher implementations.
The `tco` Cargo feature selects one of them at build time:

| Mode | Cargo feature | Build alias | Condition |
|---|---|---|---|
| Loop dispatch | (none) | `cargo build-legacy` | All Wasm runtimes|
| Tail-call dispatch | `tco` | `cargo build-tco` | Wasm tail-call proposal|

With `tco`, each handler calls the next handler with a tail call (`return_call_indirect`), which removes the overhead of the dispatch loop.
Use the loop dispatcher when the host runtime does not implement the tail-call proposal.

## Checkpoint and Restore

The checkpoint trigger mechanism is different for each build target:

- `wasm32-wasip1-threads`: A background thread monitors the `checkpoint.trigger` file.
  We recommend this target, because its performance is better.
- `wasm32-wasip1`: The dispatcher looks for the file through WASI one time for each 1024 instructions.
  Use this target if the host runtime does not have WASI threads.

```bash
# Build for wasm32-wasip1-threads (atomic monitor thread).
# Use cargo build-legacy-threads instead if the host runtime lacks tail-call support.
cargo build-tco-threads

# Run with checkpoint enabled
somethingWasmRuntime target/tco-threads/wasm32-wasip1-threads/release/chiwawa.wasm test.wasm --invoke func-name --params "I64(100)" --cr
touch ./checkpoint.trigger # Trigger of Checkpointing
# Restore from checkpoint
somethingWasmRuntime target/tco-threads/wasm32-wasip1-threads/release/chiwawa.wasm test.wasm --restore checkpoint.bin
```

## wasi-threads

Chiwawa runs multi-threaded guests on a host runtime that supplies wasi-threads.
Each host thread has one interpreter instance, and all instances share one linear memory.
The `wasm32-wasip1-threads` target is necessary for this function.
Thus the `-threads` build aliases enable the `threads` feature.
A nightly toolchain is necessary for the `-threads` aliases, and a build with stable Rust stops with an error.

```bash
# Use cargo build-legacy-threads if the host runtime lacks tail-call support.
cargo build-tco-threads

# The host runtime needs its own switch too (`-S threads=y` on wasmtime).
somethingWasmRuntime target/tco-threads/wasm32-wasip1-threads/release/chiwawa.wasm guest.wasm --threads
```

Without `--threads`, the guest cannot make a thread, and it runs with one thread.
`--cr` and `--restore` operate with `--threads`.
See [doc/migration.md](doc/migration.md) for the contents of a checkpoint.

## Sockets (experimental)

Chiwawa sends the calls of the socket extensions of WAMR and WasmEdge to the host.
A guest that uses the socket library of one of these runtimes can run on each of the two hosts.
A build-time feature selects the host.
See [doc/sockets.md](doc/sockets.md).

```bash
# One host feature at a time; works with the -threads and legacy aliases too.
cargo build-tco --features socket-wamr
cargo build-tco --features socket-wasmedge
```

## Trace

The `trace` feature is necessary at compile time for a trace.
Only the loop dispatcher (a build without `tco`) makes a trace, the same as for statistics.
The tail-call dispatcher has no dispatch loop to hook.
Thus with `tco`, Chiwawa ignores `--trace` and shows a warning.

```bash
# Build with trace feature on top of the loop dispatcher
cargo build-legacy --features trace

# TRACE_EVENTS = (all,store,load,call,branch)
# TRACE_RESOURCE = (regs,locals,globals,pc)
somethingWasmRuntime target/legacy/wasm32-wasip1/release/chiwawa.wasm test.wasm --trace --trace-events [<TRACE_EVENTS>...] --trace-resource [<TRACE_RESOURCE>...]
```

If you use `--trace` without the feature, Chiwawa shows a warning and ignores the flag.
The same occurs when you use `--trace` with the `tco` feature.

## Statistics

The `stats` feature is necessary at compile time for the statistics output.
Only the loop dispatcher (a build without `tco`) collects statistics.
The tail-call dispatcher has no dispatch loop to hook.
Thus Chiwawa ignores `--stats` with `tco`.

```bash
# Build with stats feature on top of the loop dispatcher
cargo build-legacy --features stats

# Run with statistics output
somethingWasmRuntime target/legacy/wasm32-wasip1/release/chiwawa.wasm test.wasm --stats
```

If you use `--stats` without the feature, Chiwawa shows a warning and ignores the flag.

## Call Graph Analysis

The `call_graph` feature is necessary at compile time for call graph analysis.
The parser builds the call graph while it reads the bytecode, and it does not read the function bodies a second time.

```bash
# Build with call graph support (works with both tco and legacy dispatchers)
cargo build-tco --features call_graph

# Generate a call graph in DOT format
somethingWasmRuntime target/legacy/wasm32-wasip1/release/chiwawa.wasm your.wasm \
  --call-graph-output out.dot

# Render with Graphviz
dot -Tpng out.dot -o call_graph.png
```

The DOT output contains one node for each function:

- **Imported and WASI functions**: a box with gray fill, with the label `module::name`
- **Local functions**: an ellipse, with the export name or `func_N` as the label

Each edge shows a possible call:

- `call`: the one callee of the instruction
- `call_indirect`: all non-WASI functions with a type that agrees with the type index of the instruction (a conservative overapproximation)

If you use `--call-graph-output` without the feature, Chiwawa shows a warning and ignores the flag.

## Artifacts
### Academic
- Y. Nakata and K. Matsubara, [Self-Hosted WebAssembly Runtime for Runtime-Neutral Checkpoint/Restore in Edge–Cloud Continuum](https://dl.acm.org/doi/abs/10.1145/3774898.3778040)
  - The 3rd International Workshop on Middleware for the Computing Continuum (Mid4CC, 26th ACM/IFIP International Middleware Conference Co-located Workshop)
- Y. Nakata and K. Matsubara, [Feasibility of Runtime-Neutral Wasm Instrumentation for Edge-Cloud Workload Handover](https://ieeexplore.ieee.org/document/10817975)
  -  The Ninth ACM/IEEE Symposium on Edge Computing (SEC)


### Tech Community
- Y.Nakata, [What If the Runtime Was Portable Too? Self-Hosted Runtime Capabilities in Wasm](https://colocatedeventsna2025.sched.com/event/28D8u/cllightning-talk-what-if-the-runtime-was-portable-too-self-hosted-runtime-capabilities-in-wasm-yuki-nakata-sakura-internet-inc?iframe=no)
  - WasmCon NA (KubeCon + CloudNativeCon NA 2025 Co-located)
- Y. Nakata and D. Fujii, [Beyond Portability: Live Migration for Evolving WebAssembly Workloads](https://speakerdeck.com/chikuwait/beyond-portability-live-migration-for-evolving-webassembly-workloads)
  - Japan Community Day at KubeCon + CloudNativeCon Japan 2025

## References
I referred to these repositories for the initial implementation.
Their work helped very much!

- Wasm-rs
  - [GitHub Repo](https://github.com/kgtkr/wasm-rs)
  - [Article (Japanese)](https://qiita.com/kgtkr/items/f4b3e2d83c7067f3cfcb)
- chibiwasm
  - [GitHub Repo](https://github.com/skanehira/chibiwasm)
