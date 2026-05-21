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
