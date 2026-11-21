# Chiwawa Architecture

This document gives a general view of the architecture of Chiwawa.
It does not tell you how the implementation operates.

## General View

Chiwawa is a self-hosted WebAssembly runtime.
It is a WebAssembly interpreter, and it runs as a WebAssembly module.
Thus the implementation of the host runtime has no effect on its functions.

```
+------------------+
|  Guest Wasm      |  (User application)
+------------------+
        | interpreted by
        v
+------------------+
|    Chiwawa       |  (WebAssembly module)
+------------------+
        | executed by
        v
+------------------+
|  Host Runtime    |  (Wasmtime, WasmEdge, Wasmer, etc.)
+------------------+
```

## Core Components

### Parser

The parser changes a WebAssembly binary module into an internal representation that the interpreter can run quickly.
During preprocessing, the parser resolves the branch targets, which makes execution faster.

### Execution Engine

The execution engine is a register-based threaded code interpreter.
It dispatches each instruction through a handler table.
In the table, there is one handler function for each instruction type.

Chiwawa has two dispatcher implementations, and the `tco` Cargo feature selects one of them at build time:

- The loop dispatcher is the default (`cargo build-legacy`), and it is portable.
- The tail-call dispatcher (`cargo build-tco`) uses the Wasm tail-call proposal to remove the loop overhead of each instruction.

See [tco.md](tco.md).

### Module Instance

A module instance is the runtime representation of an instantiated WebAssembly module.
It contains these items:

- Linear memory
- Tables (function references)
- Global variables
- Function instances

### Checkpoint/Restore Mechanism

This mechanism serializes the runtime state, which makes live migration possible.
A checkpoint contains these items:

- Execution stacks and program counters
- Memory contents
- Global values

A checkpoint does not contain the tables.
Instantiation initializes the tables again from the element segments of the module, and the result is always the same.
Thus it is not necessary to store them.

## Execution Flow

1. **Parse**: Load and validate the WebAssembly binary.
2. **Preprocess**: Convert the instructions to the register-based format, and resolve the branch targets.
3. **Instantiate**: Create the module instance, and initialize its memory, tables, and globals.
4. **Execute**: Run the loop of the threaded code interpreter.
5. **Checkpoint** (optional): Serialize the state to a file when a trigger occurs.

## WASI Support

Chiwawa implements WASI Preview 1 as a passthrough.
It sends each system call to the wasi-libc implementation of the host.
Thus Chiwawa is compatible with all hosts that implement WASI.

## References

- [WebAssembly Core Specification](https://webassembly.github.io/spec/core/)
- [WASI Preview 1](https://github.com/WebAssembly/WASI/blob/main/legacy/preview1/docs.md)
