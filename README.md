# chiwawa(CHeckpoint/restore and Instrumentation-specific WAsm runtime on WAsm runtime)

Chiwawa (Pronunciation of chihuahua) is a self-hosted Wasm runtime that enables live migration and instrumentation that is neutral to the execution methods (e.g., interpreter, JIT, and AOT) and architectures, and runtimes.


## Build and Run
### For native

```
cargo build
 ./target/debug/chiwawa something.wasm --invoke func-name
```

### For self-hosted

```
cargo build --target wasm32-wasip1 --release --features interp #Normal Interpreter
cargo build --target wasm32-wasip1 --release --features fast #Inline Wasm Bytecode Optimization
somethingWasmRuntime target/wasm32-wasip1/release/chiwawa.wasm something.wasm --invoke func-name --params "I64(100)"
```


## References
I referred to these repositories. I appreciate our ancestor's wisdom!

- Wasm-rs
  - [GitHub Repo](https://github.com/kgtkr/wasm-rs)
  - [Article (Japanese)](https://qiita.com/kgtkr/items/f4b3e2d83c7067f3cfcb)
- chibiwasm
  - [GitHub Repo](https://github.com/skanehira/chibiwasm)
