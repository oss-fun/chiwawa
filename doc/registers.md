# Register-Based Execution

This document explains Chiwawa's register-based execution model.

## Motivation

WebAssembly is a stack-based virtual machine. A traditional interpreter would use a value stack for every operation:

```
i32.add:
  1. pop operand b from stack
  2. pop operand a from stack
  3. compute a + b
  4. push result to stack
```

This approach requires multiple stack operations per instruction. In a self-hosted runtime, these stack operations translate to many host instructions, amplifying overhead.

Chiwawa eliminates runtime stack manipulation by converting stack operations to register references during preprocessing.

## Why Register IR Speeds Things Up on a Stack-Machine Host

Chiwawa's architecture looks strange at first glance:

- The host is a Wasm runtime, so it is a stack machine
- Chiwawa itself, running on top, is a Wasm module and therefore also executes under the stack-machine model
- Yet what Chiwawa implements **internally is a register machine**
- And the guest bytecode this register machine interprets is, once again, Wasm: stack-machine code

Implementing an interpreter for a stack machine, running on a stack machine, as a register machine matches neither the host's execution model nor the guest's instruction format. 
Intuitively, aligning everything to a stack machine seems more natural.

### What a Stack Machine Maintains at Runtime

A stack-machine interpreter must track "where the top of the stack currently is" as **dynamic state** (the stack pointer). Every instruction interacts with this state.

Tracing the execution of `i32.const 10; i32.const 20; i32.add; local.set 0`:

```
                     i32.const 10        i32.const 20        i32.add            local.set 0

SP ──┐               SP ──┐              SP ──┐              SP ──┐             SP ──┐
     ↓                    ↓                   ↓                   ↓                  ↓
┌────┐               ┌────┐              ┌────┐              ┌────┐             ┌────┐
│    │               │    │              │    │              │    │             │    │
│    │               │    │              │    │              │    │             │    │
│    │               │    │              │ 20 │←push         │ 30 │←push        │    │
│    │               │ 10 │←push         │ 10 │              │    │             │    │
└────┘               └────┘              └────┘              └────┘             └────┘
 stack
                     SP=1                SP=2                SP=1               SP=0
                                                             pop, pop,          pop,
                                                             add, push          store→local[0]
```

At every dispatch, the handler must consult SP, compute SP-relative addresses, move values in and out, and update SP. In addition, instructions that exist solely to manipulate the stack (`i32.const`, `local.get`, `local.set`, `drop`) are each dispatched as their own handler.

### What a Register IR Eliminates

Once operand positions are statically named at parse time, there is no SP and no "top of the stack."

The same sequence executes as follows:

```
                     I32Const→R0         I32Const→R1         I32Add R0,R1→R2    LocalSet 0, R2

                     ┌────┐              ┌────┐              ┌────┐             ┌────┐
                     │ R0 │ 10           │ R0 │ 10           │ R0 │ 10          │ R0 │ 10
                     │ R1 │              │ R1 │ 20           │ R1 │ 20          │ R1 │ 20
                     │ R2 │              │ R2 │              │ R2 │ 30          │ R2 │ 30
                     │ R3 │              │ R3 │              │ R3 │             │ R3 │
                     └────┘              └────┘              └────┘             └────┘
                     Reg File
                                                                                local[0]=30

                     Write directly      Write directly      Read src1=R0,      Read src=R2;
                     to dst=R0           to dst=R1           src2=R1; write     write to local[0]
                                                             to dst=R2
```

With operand folding applied, adjacent stack-shuffling instructions are absorbed into their consumer:

```
                     I32Add Const(10), Const(20) → Local[0]

                     ┌────┐
                     │ R0 │
                     │ R1 │  (unused)
                     │ R2 │
                     └────┘

                                                                                local[0]=30

                     The constants 10 and 20 are read directly from the
                     instruction's operand fields; the sum is written to
                     local[0]. The register file is left untouched.
```

### Two Axes of Reduction from a Single Mechanism

Eliminating the stack abstraction reduces overhead along two axes simultaneously.

Axis 1: fewer handler dispatches.
Stack-only instructions either disappear or get absorbed. `drop` becomes a no-op, since the register simply goes unused. `i32.const` and `local.get` fold into the consumer's immediate operand field, and `local.set` folds into the producer's destination field.

Axis 2: less work per dispatch.** Compute handlers no longer maintain the stack.
There is no SP to read, no SP-relative address to compute, no SP to write back. Each handler simply reads from named registers, computes, and writes to a named register.

Both axes derive from a single mechanism: giving every operand a static name, which abolishes the dynamic state (SP) that the stack abstraction required.

### Why Self-Hosted Execution Amplifies the Savings

Chiwawa is itself a WebAssembly module. Each guest instruction is handled by a Chiwawa handler, and each handler consists of multiple host-Wasm instructions. The cost of maintaining the stack abstraction shows up at the Chiwawa-handler layer and also as host-Wasm instructions.

Removing this overhead at the IR level eliminates it from the handler bodies, and therefore from the host-Wasm instruction stream as well. If the host runtime is a JIT or AOT compiler, the overhead is gone from the generated native code. If the host runtime is an interpreter, there are simply fewer instructions to interpret. Either way, the savings occur at every guest-instruction execution and accumulate over hot paths in proportion to execution count.

The cost paid in exchange is the conversion of bytecode to register IR at module load, but this is incurred only once per module load. For real workloads, where the ratio of dynamic guest-instruction executions to static instructions in the module is large, this one-time conversion cost is negligible compared to the cumulative savings on the hot path.

## Type-Specialized Registers

Chiwawa maintains separate register arrays for each WebAssembly value type:

```
RegFile
├── i32_regs: Vec<i32>
├── i64_regs: Vec<i64>
├── f32_regs: Vec<f32>
├── f64_regs: Vec<f64>
├── ref_regs: Vec<Ref>
└── v128_regs: Vec<i128>
```

Type specialization provides:
- Direct memory access without type checking at runtime
- Natural alignment for each value type
- Efficient bulk operations on same-typed values

## Infinite Register Model

Chiwawa uses an infinite register model: there is no fixed limit on the number of available registers. The register allocator assigns a new register for each stack slot, and the register file grows dynamically to accommodate the maximum depth reached during execution.

This differs from physical CPU register allocation, which must handle register spilling when demands exceed fixed hardware limits. In Chiwawa's virtual register model, every value has a dedicated register, eliminating the need for spill/reload logic.

## Register Allocation

During preprocessing, Chiwawa simulates the WebAssembly operand stack and assigns registers:

```
WebAssembly:          After allocation:
  i32.const 10        →  const I32(0), 10
  i32.const 20        →  const I32(1), 20
  i32.add             →  add I32(0), I32(1) -> I32(0)
```

The allocator tracks:
- Current stack depth per type
- Maximum depth reached (determines register file size)
- Type stack for polymorphic instructions (`drop`, `select`)

## Global Register

Rather than creating a new register file for each function call, Chiwawa uses a single global register file shared across all frames. Each frame is allocated a region within this global file.

```
Global Register File (i32_regs):
+------------------+------------------+------------------+
| Frame 0 (main)   | Frame 1 (foo)    | Frame 2 (bar)    |
| regs 0-5         | regs 6-10        | regs 11-15       |
+------------------+------------------+------------------+
        ^                  ^                  ^
        |                  |                  |
   offset=0           offset=6          offset=11
```

On function call:
1. Save current frame offsets
2. Extend register arrays if needed
3. New frame accesses registers relative to its offset

On function return:
1. Restore previous frame offsets
2. Register memory is not deallocated (reused by subsequent calls)

## Instruction Format

`ProcessedInstr` is an enum where each variant carries operand layout that is
specialized for the instruction it represents. Type-specialized variants
(`I32Reg`, `I64Reg`, `F32Reg`, `F64Reg`) hold a `handler_index` and three
operand slots; variants that touch a different shape (memory, select,
global, etc.) have their own field layout.

```rust
pub enum ProcessedInstr {
    I32Reg {
        handler_index: usize,
        dst: I32RegOperand,
        src1: I32RegOperand,
        src2: Option<I32RegOperand>,  // None for unary ops
    },
    MemoryLoadReg {
        handler_index: usize,
        dst: RegOrLocal,
        addr: I32RegOperand,
        offset: u64,
    },
    BrReg {
        relative_depth: u32,
        target_ip: usize,
        source_regs: RegSlice,
        target_result_regs: RegSlice,
    },
    // ...one variant per logical instruction shape
}
```

### Operand kinds

The operand slots are themselves small enums, so a single field can encode a
register, a folded constant, or a folded local/parameter:

```rust
pub enum I32RegOperand {
    Reg(u16),    // read/write a register
    Const(i32),  // folded immediate constant
    Param(u16),  // read/write a local (function parameter or local var)
}
// I64RegOperand / F32RegOperand / F64RegOperand share the same shape.

pub enum RegOrLocal {
    Reg(u16),    // destination is a register
    Local(u16),  // destination is folded into a local slot
}
```

This lets the parser embed both **source folding** (constants and
`local.get`s) and **destination folding** (`local.set`) directly in the IR
without introducing extra instructions — see `doc/folding.md`. At execution
time, each handler reads its `src*` operands, performs the operation, and
writes to `dst`, without any stack manipulation.
