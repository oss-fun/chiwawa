# Chiwawa Architecture

This document provides a high-level overview of Chiwawa's architecture for those seeking to understand the system design without diving into implementation details.

## Overview

Chiwawa is a self-hosted WebAssembly runtime - a WebAssembly interpreter that itself runs as WebAssembly. This enables runtime-neutral capabilities that are independent of the host runtime implementation.

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

Transforms WebAssembly binary modules into an internal representation optimized for interpretation. The parser performs preprocessing that resolves branch targets ahead of time, enabling efficient execution.

### Execution Engine

Implements a register-based threaded code interpreter. Instructions are dispatched through a handler table where each instruction type maps to a specialized handler function.

### Module Instance

Runtime representation of an instantiated WebAssembly module, containing:
- Linear memory
- Tables (function references)
- Global variables
- Function instances

### Checkpoint/Restore Mechanism

Enables live migration by serializing complete runtime state:
- Execution stacks and program counters
- Memory contents
- Global values
- Table entries

## Execution Flow

1. **Parse**: Load and validate WebAssembly binary
2. **Preprocess**: Convert instructions to register-based format with resolved branch targets
3. **Instantiate**: Create module instance with initialized memory, tables, and globals
4. **Execute**: Run threaded interpreter loop
5. **Checkpoint** (optional): Serialize state to file when triggered

## WASI Support

Chiwawa implements WASI Preview 1 through a passthrough architecture that delegates system calls to the host's wasi-libc implementation. This ensures compatibility with any WASI-compliant host.

## References

- [WebAssembly Core Specification](https://webassembly.github.io/spec/core/)
- [WASI Preview 1](https://github.com/WebAssembly/WASI/blob/main/legacy/preview1/docs.md)
