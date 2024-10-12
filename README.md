# chiwawa(CHeckpoint/restore and Instrumentation-specific WAsm runtime on WAsm runtime)

Chiwawa (Pronunciation of chihuahua) is a self-hosted Wasm runtime that enables live migration and instrumentation that is neutral to the execution methods (e.g., interpreter, JIT, and AOT) and architectures, and runtimes.


## Build and Run
### For native

```
cargo build
 ./target/debug/chiwawa --path something.wasm
```

### For self-hosted

```
cargo build --target wasm32-wasi
somethingWasmRuntime target/wasm32-wasi/debug/chiwawa.wasm --path something.wasm
```


## References
I referred to these repositories. I appreciate our ancestor's wisdom!

- Wasm-rs
  - [GitHub Repo](https://github.com/kgtkr/wasm-rs)
  - [Article (Japanese)](https://qiita.com/kgtkr/items/f4b3e2d83c7067f3cfcb)
- chibiwasm
  - [GitHub Repo](https://github.com/skanehira/chibiwasm)
