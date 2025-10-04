# chiwawa(CHeckpoint/restore and Instrumentation-specific WAsm runtime on WAsm runtime)

Chiwawa (Pronunciation of chihuahua) is a self-hosted Wasm runtime that enables live migration and instrumentation that is neutral to the execution methods (e.g., interpreter, JIT, and AOT) and architectures, and runtimes.


## Build and Run

```bash
cargo build --target wasm32-wasip1 --release

# Call function with Wasm parameters (I32, I64, F32, F64)
somethingWasmRuntime target/wasm32-wasip1/release/chiwawa.wasm test.wasm --invoke func-name --params "I64(100)"

# Pass command-line arguments to WASI-compiled program
somethingWasmRuntime target/wasm32-wasip1/release/chiwawa.wasm test.wasm --app-args "--version"
```

## Checkpoint and Restore

**Note**: The checkpoint trigger mechanism differs between build targets:
- `wasm32-wasip1-threads`: Uses a background thread to monitor `checkpoint.trigger` file (recommended for better performance)
- `wasm32-wasip1`: Checks file existence via WASI at each instruction (no threading support)

```bash
cargo build --target wasm32-wasip1-threads --release

# Run with checkpoint enabled
somethingWasmRuntime target/wasm32-wasip1-threads/release/chiwawa.wasm test.wasm --invoke func-name --params "I64(100)" --cr
touch ./checkpoint.trigger # Trigger of Checkpointing
# Restore from checkpoint
somethingWasmRuntime target/wasm32-wasip1-threads/release/chiwawa.wasm test.wasm --restore checkpoint.bin
```

## References
I referred to these repositories. I appreciate our ancestor's wisdom!

- Wasm-rs
  - [GitHub Repo](https://github.com/kgtkr/wasm-rs)
  - [Article (Japanese)](https://qiita.com/kgtkr/items/f4b3e2d83c7067f3cfcb)
- chibiwasm
  - [GitHub Repo](https://github.com/skanehira/chibiwasm)
