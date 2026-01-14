# Chiwawa's Interpreter Design

This document explains Chiwawa's interpreter architecture.

## Design Goal

Chiwawa's primary design goal is to minimize the overhead of dual-layer runtime execution.

Since Chiwawa is a WebAssembly runtime that itself runs as WebAssembly, every guest instruction is interpreted by Chiwawa, which is then executed by the host runtime. A single guest WebAssembly instruction (e.g., `i32.add`) is handled by Chiwawa's instruction handler, which itself compiles to many host WebAssembly instructions. This instruction expansion is the primary source of overhead in self-hosted execution.

To minimize this overhead, Chiwawa's interpreter is optimized for:

1. **Reduced dispatch cost**: Minimize work done per instruction fetch-decode-execute cycle
2. **Fewer stack operations**: Avoid redundant stack pushes/pops during interpretation
3. **Amortized preprocessing**: Move expensive computations (branch resolution, register allocation) to module load time

## Preprocessing Pipeline

Chiwawa instantiates a module through a multi-phase pipeline:

```
Phase 1: Decode
  Parse Wasm bytecode, build instruction-to-position mapping

Phase 2: Branch Resolution
  Resolve Br, BrIf, If, Else targets to absolute positions

Phase 3: BrTable Resolution
  Handle variable-target branch tables

Phase 4: Register Allocation
  Assign operands to typed registers
```

## Register-Based Execution

Unlike traditional stack-based WebAssembly interpreters, Chiwawa uses a register model:

```
Stack-based (traditional):
  push a, push b, add, pop result

Register-based (Chiwawa):
  add r0, r1 -> r2
```

Benefits:
- Fewer stack operations per instruction
- Operand locations known at preprocessing time

## Threaded Code Dispatch

Chiwawa uses a threaded code interpreter, where each instruction type maps to a dedicated handler function stored in a handler table.

```
Traditional switch-based:
  loop {
    match opcode {
      ADD => { ... }
      SUB => { ... }
      ...
    }
  }

Threaded code (Chiwawa):
  loop {
    handler = HANDLER_TABLE[opcode]
    handler(instr, regs, ...)
  }
```

The handler table approach eliminates the switch dispatch overhead. Each handler is a specialized function that:

1. Reads operands from registers
2. Performs the operation
3. Writes results to registers
4. Returns control to the main loop

Branch instructions are optimized by pre-resolving targets during preprocessing. Instead of computing `current_pc + offset` at runtime, branch handlers jump directly to absolute positions.
