# Register-Based Execution

This document is about the register-based execution model of Chiwawa.

## Problem

WebAssembly is a stack-based virtual machine.
The usual interpreter uses a value stack for each operation:

```
i32.add:
  1. pop operand b from stack
  2. pop operand a from stack
  3. compute a + b
  4. push result to stack
```

This method uses more than one stack operation for each instruction.
In a self-hosted runtime, these stack operations become many host instructions, which increases the overhead.

During preprocessing, Chiwawa changes stack operations to register references.
Thus there are no stack operations at run time.

## Register IR Is Faster on a Stack-Machine Host

At first, the architecture of Chiwawa looks unusual:

- The host is a Wasm runtime, and thus it is a stack machine.
- Chiwawa runs on the host as a Wasm module, and thus it also executes with the stack-machine model.
- But internally, Chiwawa implements **a register machine**.
- The guest bytecode that this register machine interprets is Wasm again, which is stack-machine code.

Chiwawa is an interpreter for a stack machine, it runs on a stack machine, and it is a register machine.
This architecture does not agree with the execution model of the host or with the instruction format of the guest.
An architecture that uses a stack machine in all layers looks more natural.

### The State That a Stack Machine Keeps at Run Time

A stack-machine interpreter must always know the position of the top of the stack.
This position is **dynamic state** (the stack pointer, SP).
Each instruction uses this state.

This is a trace of the execution of `i32.const 10; i32.const 20; i32.add; local.set 0`:

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

At each dispatch, the handler must do these steps:

1. It reads SP.
2. It calculates addresses relative to SP.
3. It moves values in and out.
4. It updates SP.

Also, some instructions only change the stack (`i32.const`, `local.get`, `local.set`, `drop`), and each of them uses one dispatch.

### The State That a Register IR Removes

At parse time, the parser gives each operand position a static name.
After that, there is no SP and no top of the stack.

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

With operand folding, the consumer absorbs the adjacent instructions that only move stack values:

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

### One Mechanism Decreases the Overhead on Two Axes

The removal of the stack abstraction decreases the overhead on two axes at the same time.

Axis 1: a smaller number of handler dispatches.
Instructions that only change the stack go away, or a different instruction absorbs them.
`drop` becomes a no-op, because no instruction uses the register.
`i32.const` and `local.get` fold into the immediate operand field of the consumer.
`local.set` folds into the destination field of the producer.

Axis 2: less work for each dispatch.
Handlers that calculate values do not keep a stack.
There is no SP to read, no address relative to SP to calculate, and no SP to write back.
Each handler reads from named registers, calculates, and writes to a named register.

The two axes come from one mechanism.
Each operand gets a static name, and thus the dynamic state (SP) of the stack abstraction is not necessary.

### Self-Hosted Execution Increases the Effect

Chiwawa is a WebAssembly module.
A Chiwawa handler executes each guest instruction, and each handler becomes many host Wasm instructions.
The work for the stack abstraction is in the handler layer of Chiwawa and also in the host Wasm instructions.

When the IR removes this overhead, the overhead goes away from the handler bodies, and thus from the stream of host Wasm instructions.
If the host runtime is a JIT or AOT compiler, the generated native code does not contain the overhead.
If the host runtime is an interpreter, it interprets a smaller number of instructions.
In the two conditions, each execution of a guest instruction uses less time.
On hot paths, the total effect is in proportion to the execution count.

To get this effect, Chiwawa must change the bytecode to register IR one time for each module load.
Most workloads execute each static instruction of the module many times.
Thus the time for this conversion is very small when you compare it with the total effect on the hot path.

## Type-Specialized Registers

Chiwawa keeps a different register array for each WebAssembly value type:

```
RegFile
├── i32_regs: Vec<i32>
├── i64_regs: Vec<i64>
├── f32_regs: Vec<f32>
├── f64_regs: Vec<f64>
├── ref_regs: Vec<Ref>
└── v128_regs: Vec<i128>
```

Type specialization has these results:

- Memory access without a type check at run time
- Natural alignment for each value type
- Fast operations on many values of the same type

## Infinite Register Model

Chiwawa uses an infinite register model, in which the number of registers has no limit.
The register allocator gives a new register to each stack slot.
The register file becomes larger at run time to hold the maximum stack depth of the execution.

Register allocation for a physical CPU is different.
There, the allocator must spill registers when the code uses more registers than the hardware has.
In the virtual register model of Chiwawa, each value has one register.
Thus spill and reload logic is not necessary.

## Register Allocation

During preprocessing, Chiwawa simulates the WebAssembly operand stack and gives a register to each stack slot:

```
WebAssembly:          After allocation:
  i32.const 10        →  const I32(0), 10
  i32.const 20        →  const I32(1), 20
  i32.add             →  add I32(0), I32(1) -> I32(0)
```

The allocator records these items:

- The stack depth for each type at that point
- The maximum depth (it sets the number of registers in the register file)
- The type stack for polymorphic instructions (`drop`, `select`)

## Global Register

Chiwawa does not make a new register file for each function call.
It uses one global register file that all frames share.
Each frame gets a region in this global file.

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

On a function call:

1. Chiwawa saves the offsets of the frame of the caller.
2. It extends the register arrays if necessary.
3. It copies the arguments from the frame of the caller into the local registers of the callee.
4. The new frame accesses registers relative to its offset.

`VmState` also keeps a pointer to the first register of each numeric type in the frame that runs.
Each call and each return updates this pointer.
A handler reads or writes a register through that pointer with one load.
It does not go through the register file and the frame offset.

On a function return:

1. Chiwawa copies the return values from the frame of the callee into the result registers of the caller.
2. It restores the offsets of the previous frame.
3. It does not release the register memory, and subsequent calls use the memory again.

The two frames index the same vectors.
Thus the transfer of arguments and results is a copy in one array.
A call uses no intermediate `Val` and no heap allocation.
The copy must occur before the offset restore, because the restore truncates the vectors down to the first register of the callee.

## Instruction Format

`ProcessedInstr` is an enum.
Each variant has an operand layout that is specialized for its instruction.
The type-specialized variants (`I32Reg`, `I64Reg`, `F32Reg`, `F64Reg`) contain a `handler_index` and three operand slots.
Variants for a different shape (memory, select, global, and other shapes) have a different field layout.

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

### Operand variants

The operand slots are small enums.
Thus one field can encode a register, a folded constant, or a folded local or parameter:

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

With these operands, the parser puts **source folding** (constants and `local.get`) and **destination folding** (`local.set`) directly in the IR, without more instructions.
See [folding.md](folding.md).
At execution time, each handler reads its `src*` operands, does the operation, and writes to `dst`, without stack operations.
