# Chiwawa's Interpreter Design

This document is about the interpreter architecture of Chiwawa.

## Primary Function

The primary function of the interpreter architecture is to decrease the overhead of execution in two runtime layers.

Chiwawa is a WebAssembly runtime that runs as WebAssembly.
Thus Chiwawa interprets each guest instruction, and the host runtime executes Chiwawa.
An instruction handler of Chiwawa executes one guest WebAssembly instruction (for example, `i32.add`).
The compiler changes that handler into many host WebAssembly instructions.
These added instructions are the primary source of overhead in self-hosted execution.

To decrease this overhead, the interpreter has these properties:

1. **Less dispatch work**: Each fetch-decode-execute cycle of an instruction does less work.
2. **A smaller number of stack operations**: The interpreter does not push or pop values when this is not necessary.
3. **Preprocessing at load time**: Chiwawa does the long calculations (branch resolution, register allocation) when it loads the module.

## Preprocessing Pipeline

Chiwawa preprocesses each function body in three phases:

```
Phase 1: Decode
  Parse Wasm bytecode, assign operands to typed registers,
  apply operand folding, record where each block ends

Phase 2: Branch Resolution
  Resolve br, br_if, br_table, if and else targets to absolute positions
  in a single forward walk that maintains the control stack

Phase 3: Compaction
  Strip no-op instructions and remap branch targets
```

Register allocation and operand folding occur in the decode phase, not in subsequent phases.
See [folding.md](folding.md).

## Register-Based Execution

The usual WebAssembly interpreter is stack-based, but Chiwawa uses a register model:

```
Stack-based (traditional):
  push a, push b, add, pop result

Register-based (Chiwawa):
  add r0, r1 -> r2
```

Results:

- Each instruction uses a smaller number of stack operations.
- Preprocessing gives the location of each operand.

## Threaded Code Dispatch

Chiwawa uses a threaded code interpreter.
A handler table has one handler function for each instruction type.

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

The handler table removes the overhead of switch dispatch.
Each handler is a specialized function that does these steps:

1. It reads the operands from registers.
2. It does the operation.
3. It writes the results to registers.
4. It returns control to the dispatch loop.

Preprocessing resolves the targets of the branch instructions.
Thus a branch handler does not calculate `current_pc + offset` at run time, and it jumps directly to an absolute position.

## Dispatch Modes

Chiwawa has two dispatcher implementations, and the `tco` Cargo feature selects one of them at build time:

- **Loop dispatcher** (default, `cargo build-legacy`): This is the usual threaded code loop that the code above shows.
  It operates on all Wasm runtimes, WAMR included.
- **Tail-call dispatcher** (`cargo build-tco`): Each handler calls the next handler with a tail call (`return_call_indirect`), which removes the fetch, call, and return cycle.
  A host runtime that implements the Wasm tail-call proposal is necessary.

The two modes use the same handler bodies, operand layout, and checkpoint protocol.
Only the transfer of control between handlers is different.
See [tco.md](tco.md) for more information.
