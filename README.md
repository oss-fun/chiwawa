# PPWasm
Self-hosted Wasm Runtime

## Build and Run
### For native

```
cargo build
 ./target/debug/PPWasm --path something.wasm
```

### For self-hosted

```
cargo build --target wasm32-wasi
somethingWasmRuntime target/wasm32-wasi/debug/PPWasm.wasm --path something.wasm
```


### 
### References

- Wasm-rs
  - [GitHub Repo](https://github.com/kgtkr/wasm-rs)
  - [Article (Japanese)](https://qiita.com/kgtkr/items/f4b3e2d83c7067f3cfcb)
- chibiwasm
  - [GitHub Repo](https://github.com/skanehira/chibiwasm)
