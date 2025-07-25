#!/bin/bash
# Test with wasmtime runtime
cp .cargo/config-wasmtime.toml .cargo/config.toml
~/.cargo/bin/cargo test --target wasm32-wasip1 "$@"