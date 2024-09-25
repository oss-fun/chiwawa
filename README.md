# PPWasm
Para Path-through Wasm Runtime
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
