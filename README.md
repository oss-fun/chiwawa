# chiwawa(CHeckpoint/restore and Instrumentation-specific WAsm runtime on WAsm runtime)

Chiwawa (Japanese-like Pronunciation of chihuahua) is a self-hosted Wasm runtime that enables live migration and instrumentation that is neutral to the execution methods (e.g., interpreter, JIT, and AOT) and architectures, and runtimes.

U^ｪ^U

## Build and Run

Pick a build alias based on the host Wasm runtime's tail-call support
(see [Dispatcher Modes](#dispatcher-modes) below):

```bash
# Tail-call dispatcher (Wasmtime v28+, WasmEdge 0.14+)
cargo build-tco           # → target/tco/wasm32-wasip1/release/chiwawa.wasm
# Loop dispatcher (works on any modern Wasm runtime, incl. WAMR)
cargo build-legacy        # → target/legacy/wasm32-wasip1/release/chiwawa.wasm

# Call function with Wasm parameters (I32, I64, F32, F64)
somethingWasmRuntime target/<combo>/wasm32-wasip1/release/chiwawa.wasm test.wasm --invoke func-name --params "I64(100)"

# Pass command-line arguments to WASI-compiled program
somethingWasmRuntime target/<combo>/wasm32-wasip1/release/chiwawa.wasm test.wasm --app-args "--version"
```

## Dispatcher Modes

Chiwawa ships two dispatcher implementations selected at build time via the
`tco` Cargo feature:

| Mode | Cargo feature | Build alias | Requirement |
|---|---|---|---|
| Loop dispatch | (none) | `cargo build-legacy` | Any modern Wasm runtime (incl. WAMR) |
| Tail-call dispatch | `tco` | `cargo build-tco` | Wasm tail-call proposal (Wasmtime v28+, WasmEdge 0.14+) |

With `tco`, each handler tail-calls (`return_call_indirect`) the next handler,
removing the dispatch loop overhead. Use the loop dispatcher when the host
runtime does not implement the tail-call proposal.

## Checkpoint and Restore

**Note**: The checkpoint trigger mechanism differs between build targets:
- `wasm32-wasip1-threads`: Uses a background thread to monitor `checkpoint.trigger` file (recommended for better performance)
- `wasm32-wasip1`: Checks file existence via WASI every 1024 instructions (use if host runtime does not support WASI threads)

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
## Tracing

Tracing requires the `trace` feature to be enabled at compile time. Stack the
feature onto a build alias to keep the per-dispatch-mode output directory:

```bash
# Build with trace feature on top of the loop dispatcher
cargo build-legacy --features trace
# Or with the tail-call dispatcher
cargo build-tco --features trace

# TRACE_EVENTS = (all,store,load,call,branch)
# TRACE_RESOURCE = (regs,memory,locals,globals,pc)
somethingWasmRuntime target/legacy/wasm32-wasip1/release/chiwawa.wasm test.wasm --trace --trace-events [<TRACE_EVENTS>...] --trace-resource [<TRACE_RESOURCE>...]
```

If `--trace` is used without the feature enabled, a warning is displayed and the flag is ignored.

## Statistics

Statistics output requires the `stats` feature to be enabled at compile time:

```bash
# Build with stats feature on top of a build alias
cargo build-legacy --features stats
# Or with the tail-call dispatcher
cargo build-tco --features stats

# Run with statistics output
somethingWasmRuntime target/legacy/wasm32-wasip1/release/chiwawa.wasm test.wasm --stats
```

If `--stats` is used without the feature enabled, a warning is displayed and the flag is ignored.

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
I referred to these repositories for initial implementation. I appreciate ancestor's wisdom!

- Wasm-rs
  - [GitHub Repo](https://github.com/kgtkr/wasm-rs)
  - [Article (Japanese)](https://qiita.com/kgtkr/items/f4b3e2d83c7067f3cfcb)
- chibiwasm
  - [GitHub Repo](https://github.com/skanehira/chibiwasm)
