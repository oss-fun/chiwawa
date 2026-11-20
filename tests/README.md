# Tests

This directory contains test cases based on:

- [WebAssembly/spec](https://github.com/WebAssembly/spec) - Core WebAssembly functionality tests
- [WebAssembly/wasi-testsuite](https://github.com/WebAssembly/wasi-testsuite) - WASI Preview 1 tests

`checkpoint/` tests checkpoint/restore end to end: `run.sh` runs chiwawa on `count_up.wat` under a host runtime, checkpoints it partway, restores it, and checks that the printed numbers stay consecutive.
It also checkpoints `file_count.wat`, which counts into a file it opened.
The restored process must continue that file at the right offset.
It also checkpoints `wait_input.wat` during a read from stdin.
The restored process must receive the input.

## Running Tests

```bash

# WebAssembly target tests
~/.cargo/bin/cargo test --target wasm32-wasip1
```