#!/bin/bash
# Test with wasmedge runtime
cp .cargo/config-wasmedge.toml .cargo/config.toml
~/.cargo/bin/cargo test --target wasm32-wasip1 "$@"
cp .cargo/config-wasmtime.toml .cargo/config.toml  # restore default