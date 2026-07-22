# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Codebase Overview

Chiwawa is a self-hosted WebAssembly (Wasm) runtime that runs on top of WebAssembly. It provides live migration and instrumentation capabilities, with a design that is neutral to execution methods (interpreter, JIT, AOT) and architectures.

### Basic Policy
- Use English for code, comments, and documentation
- Claude responds in English to user messages (user may use Japanese, English, or both)
- Claude provides English grammar corrections when errors affect clarity or upon request
- DTC (Direct-Threaded Code) interpreter implementation
- Self-hosted runtime that runs on any Wasm runtime, independent of runtime implementation and compilation methods

## Main Commands

### Build
```bash
# Self-hosted build (required: wasm32-wasip1 target)
~/.cargo/bin/cargo build --target wasm32-wasip1 --release

# Check compilation errors
~/.cargo/bin/cargo check --target wasm32-wasip1
```

### Test Execution
```bash
# Run tests on Wasm target (self-hosted)
~/.cargo/bin/cargo test --target wasm32-wasip1

# Run specific test (Wasm target)
~/.cargo/bin/cargo test --target wasm32-wasip1 <test-name>

# Run tests with multiple Wasm runtimes
./test-wasmtime.sh <test-name>   # Run tests with wasmtime (default)
./test-wasmedge.sh <test-name>    # Run tests with wasmedge

# Note: Wasm target tests use wasmtime with --dir . option (configured in .cargo/config.toml)
# to enable file access
```

### Wasm File Execution

#### CLI Options
```
chiwawa [OPTIONS] <WASM_FILE>

Arguments:
  <WASM_FILE>              WebAssembly file to execute

Options:
  -i, --invoke <INVOKE>    Function name to invoke [default: _start]
  -p, --params <PARAMS>    Function parameters (comma-separated)
                           Format: I32(value), I64(value), F32(value), F64(value)
  --restore <FILE>         Restore from checkpoint file
  --app-args <ARGS>        Additional arguments to pass to WASM application
                           These become argv[1], argv[2], ... in the guest
  --cr                     Enable checkpoint/restore functionality
  --stats                  Enable statistics output
  -h, --help               Print help
  -v, --version            Print version
```

#### Execution Examples
```bash
# Basic execution (any Wasm runtime: wasmtime, WasmEdge, wasmer, etc.)
wasmtime target/wasm32-wasip1/release/chiwawa.wasm test.wasm

# Invoke specific function
wasmtime target/wasm32-wasip1/release/chiwawa.wasm test.wasm --invoke func-name

# Execute with parameters
wasmtime target/wasm32-wasip1/release/chiwawa.wasm test.wasm \
  --invoke add --params "I32(10),I32(20)"

# Specify application arguments
wasmtime target/wasm32-wasip1/release/chiwawa.wasm sqlite-bench.wasm \
  --app-args "--database test.db --iterations 1000"

# Execute with statistics output
wasmtime target/wasm32-wasip1/release/chiwawa.wasm test.wasm \
  --stats

# Checkpoint/Restore execution
# 1. Execute with checkpoint enabled
touch ./checkpoint.trigger  # Create checkpoint trigger file
wasmtime target/wasm32-wasip1/release/chiwawa.wasm test.wasm \
  --invoke func-name --cr
# checkpoint.bin is generated when checkpoint.trigger file is detected during execution

# 2. Restore from checkpoint
wasmtime target/wasm32-wasip1/release/chiwawa.wasm test.wasm \
  --invoke func-name --cr --restore checkpoint.bin

# Other runtime examples:
# WasmEdge target/wasm32-wasip1/release/chiwawa.wasm test.wasm --invoke func-name
# wasmer target/wasm32-wasip1/release/chiwawa.wasm test.wasm --invoke func-name
```

## Architecture

### Main Modules
- `src/lib.rs`: Library entry point
- `src/main.rs`: CLI interface and main execution logic
- `src/parser.rs`: WebAssembly bytecode parsing and DTC instruction generation
- `src/error.rs`: Error type definitions

- `src/structure/`: Data structures for internal Wasm bytecode representation
  - `instructions.rs`: WebAssembly instruction definitions
  - `module.rs`: Module structure (functions, memory, tables, etc.)
  - `types.rs`: WebAssembly type definitions (function types, value types, etc.)

- `src/execution/`: DTC interpreter execution engine
  - `runtime.rs`: Runtime core and host function invocation
  - `stack.rs`: Stack management and DTC execution loop (preprocessing, branch resolution)
  - `module.rs`: Module instance management
  - `func.rs`: Function instances and call handling
  - `mem.rs`: Memory instances and load/store operations
  - `table.rs`: Table instances and reference management
  - `global.rs`: Global variable instance management
  - `data.rs`: Data segment initialization
  - `elem.rs`: Element segment initialization
  - `export.rs`: Export resolution and mapping
  - `value.rs`: Value types and stack value definitions
  - `migration.rs`: Checkpoint/restore functionality (serialization)

- `src/wasi/`: WASI Preview 1 implementation (passthrough only)
  - `context.rs`: WASI context and file descriptor management
  - `passthrough.rs`: WASI function passthrough implementation (delegates to wasi-libc)
  - `types.rs`: WASI type definitions and mappings
  - `error.rs`: WASI error type definitions

### Execution Flow
1. Parse CLI parameters (using clap, optimization flags, etc.)
2. Parse WebAssembly module (using wasmparser)
3. DTC preprocessing
   - Convert WebAssembly instructions to DTC format (ProcessedInstr)
   - Pre-calculate absolute jump targets for branch instructions (Fixup process)
4. Create module instance (memory, tables, global variables, etc.)
5. Initialize Runtime
   - Normal execution: Start from entry point function
   - Restore execution: Restore state from checkpoint file
6. DTC execution loop
   - Fast dispatch via handler table
7. Checkpoint processing (when `--cr` enabled)
   - Create snapshot when checkpoint.trigger file is detected
   - Serialize execution state to checkpoint.bin

### Parameter Format
Function parameters are specified in the following format:
- `I32(value)`: 32-bit integer
- `I64(value)`: 64-bit integer
- `F32(value)`: 32-bit floating point
- `F64(value)`: 64-bit floating point

### Main Dependencies

**Core:**
- `wasmparser`: WebAssembly bytecode parsing
- `clap`: CLI argument parsing (with derive features)
- `anyhow`: Error handling
- `thiserror`: Error type derivation macro

**Optimization & Performance:**
- `rustc-hash`: Fast hash map (FxHashMap)
- `lazy_static`: Static variable initialization (for DTC handler table)

**Serialization:**
- `serde`: Serialization framework
- `bincode`: Binary serialization (for checkpoints)
- `rmp`: MessagePack serialization

**WASI:**
- `getrandom`: Random number generation (for WASI random_get implementation)

**Utilities:**
- `fancy-regex`: Regular expressions (for parameter parsing)
- `itertools`: Iterator extensions
- `byteorder`: Byte order conversion
- `num`: Numeric type traits
- `typenum`: Type-level numbers
- `maplit`: Macro-based collection initialization

**Development & Testing:**
- `wat`: WAT→Wasm compiler (for test code)

## Test Configuration

Tests are located in the `tests/` directory and are based on WebAssembly and WASI specifications.

### WebAssembly Core Tests (23 files)
**Control Flow:**
- `block.rs`, `br.rs`, `br_if.rs`, `if.rs`, `loop.rs`, `labels.rs`, `switch.rs`

**Function Calls:**
- `call.rs`, `call_indirect.rs`

**Numeric Operations:**
- `i32.rs`, `i64.rs`, `f32.rs`, `conversions.rs`

**Memory Operations:**
- `memory.rs`, `memorycopy.rs`, `memoryfill.rs`, `memoryinit.rs`, `memorysize.rs`

**Table Operations:**
- `table_get.rs`, `table_set.rs`, `table_fill.rs`

**Reference Types:**
- `ref_null.rs`, `ref_is_null.rs`

**Others:**
- `select.rs`, `stack.rs`, `store.rs`, `parser.rs`

### WASI Preview 1 Tests (25 files)
**File I/O:**
- `wasi_file_pread_pwrite.rs`, `wasi_file_seek_tell.rs`, `wasi_file_allocate.rs`

**Directory Operations:**
- `wasi_fd_readdir.rs`, `wasi_directory_seek.rs`

**Path Operations:**
- `wasi_path_open_preopen.rs`, `wasi_path_open_read_write.rs`, `wasi_path_filestat.rs`
- `wasi_path_link.rs`, `wasi_path_rename.rs`, `wasi_path_rename_dir_trailing_slashes.rs`
- `wasi_unlink_file_trailing_slashes.rs`

**File Descriptors:**
- `wasi_fd_advise.rs`, `wasi_fd_fdstat_set_rights.rs`, `wasi_fd_filestat_set.rs`
- `wasi_close_preopen.rs`, `wasi_dangling_fd.rs`, `wasi_renumber.rs`

**System:**
- `wasi_clock_time_get.rs`, `wasi_sched_yield.rs`, `wasi_poll_oneoff_stdio.rs`

**Others:**
- `wasi_big_random_buf.rs`, `wasi_readlink.rs`, `wasi_dangling_symlink.rs`
- `wasi_parser_test.rs`

### Test Files
- `tests/wasm/`: WebAssembly core test binaries (.wasm and .wat pairs)
- `tests/wasi/`: WASI test binaries

## Instrumentation Features

### Statistics Output
Enable at runtime with the `--stats` flag. Requires building with the `stats`
cargo feature (`cargo build --features stats`); without it, `--stats` is ignored
with a warning.

**Scope:**
- Collected only by the non-TCO (loop) dispatcher. When the `tco` feature is
  enabled, `--stats` is ignored with a warning, because the tail-call dispatcher
  has no central loop to hook.

**Output Information (printed to stderr on exit):**
- Total instructions executed
- Top instructions by execution count, with per-instruction counts and
  percentage of total

**Use Cases:**
- Performance analysis
- Identifying hot instructions / bottlenecks

### Call Graph Analysis
Enable at build time with the `call_graph` cargo feature. Without it,
`--call-graph-output` is ignored with a warning.

```bash
# Build with call graph support (works with both tco and legacy dispatchers)
~/.cargo/bin/cargo build --target wasm32-wasip1 --release --features call_graph

# Generate call graph DOT file during execution
wasmtime --dir . target/wasm32-wasip1/release/chiwawa.wasm your.wasm \
  --call-graph-output out.dot

# Render with Graphviz
dot -Tpng out.dot -o call_graph.png
xdg-open call_graph.png
```

**How the call graph is built:**
- Constructed incrementally during bytecode parsing (no second pass over function bodies)
- One node per function in FuncIdx order: imports first (`0..num_imported_funcs`),
  then locally-defined functions (`num_imported_funcs..`)
- Direct calls (`call`): exact callee recorded as an edge
- Indirect calls (`call_indirect`): conservative type-based approximation — all
  non-WASI functions whose type matches the `call_indirect` type index become
  candidate callees (overapproximation; safe for reachability analysis)
- WASI functions are excluded from `call_indirect` candidates (they have no
  Wasm body to discard)

**DOT output format:**
- Imported/WASI nodes: box shape with gray fill
- Local function nodes: default ellipse
- Node labels: export name if exported, otherwise `func_N` (N = raw FuncIdx)
- Import node labels: `module::name` (e.g., `wasi_snapshot_preview1::fd_write`)

**Use Cases:**
- Static reachability analysis for checkpoint/restore optimization
  (identify functions unreachable from the restored call stack)
- Visualizing module dependency structure

## Development Guidelines

### Development Approach
- Think step by step: Understand requirements → Design pseudocode → Implement → Verify
- Design module structure, endpoints, and data flow in pseudocode before implementation
- Always format code with cargo after completing features (`~/.cargo/bin/cargo fmt`)

### WASI Implementation
- Chiwawa uses **passthrough implementation only** (standard implementation is not used)
- WASI functions should return errno, not terminate with Err
- Passthrough delegates processing to wasi-libc implementation

### WebAssembly Specification References
- Wasm Core Spec: https://webassembly.github.io/spec/core/bikeshed/
- WASI function list: https://github.com/WebAssembly/wasi-libc/blob/main/libc-bottom-half/headers/public/wasi/api.h
