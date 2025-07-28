# Tests

This directory contains test cases based on:

- [WebAssembly/spec](https://github.com/WebAssembly/spec) - Core WebAssembly functionality tests
- [WebAssembly/wasi-testsuite](https://github.com/WebAssembly/wasi-testsuite) - WASI Preview 1 tests

## Running Tests

```bash

# WebAssembly target tests
~/.cargo/bin/cargo test --target wasm32-wasip1
```