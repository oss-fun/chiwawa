# Operand Folding

Operand folding is a preprocessing optimization that identifies patterns of consecutive instructions that can be merged into a single operation. This eliminates intermediate register allocations and reduces dispatch overhead.

```
Before folding:
  i32.const 42    ; r0 = 42
  local.set 0     ; local[0] = r0

After folding:
  i32.const 42 -> local[0]  ; store 42 directly to local[0]
```

## Folding Types

### Source Folding

Folds constant values and local.get operations into consuming instructions.

```
Before:
  i32.const 10   ; r0 = 10
  i32.const 20   ; r1 = 20
  i32.add        ; r2 = r0 + r1

After:
  i32.add (const 10), (const 20) -> r0
```

Supported source operands:
- `i32.const`, `i64.const`, `f32.const`, `f64.const`
- `local.get` (typed: i32, i64, f32, f64)

### Destination Folding

Folds `local.set` into the preceding instruction that produces the value.

```
Before:
  i32.add        ; r0 = a + b
  local.set 0    ; local[0] = r0

After:
  i32.add -> local[0]  ; result directly to local
```

When destination folding is applied, the instruction uses `RegOrLocal::Local` instead of `RegOrLocal::Reg` for its destination.

### Address Folding (Memory Operations)

For memory load/store operations, folds constant addresses.

```
Before:
  i32.const 100  ; r0 = 100 (address)
  i32.load       ; r1 = memory[r0]

After:
  i32.load (addr: const 100) -> r1
```

## Implementation

Folding is performed during the preprocessing phase using a peek-ahead mechanism:

1. **Pending Operand Stack**: When a foldable source instruction (const, local.get) is encountered, it is pushed to a pending stack instead of generating a register instruction.

2. **Consumer Check**: When a consuming instruction is processed, it checks the pending stack for compatible operands.

3. **Destination Check**: After processing an instruction, the parser peeks ahead to check if the next instruction is `local.set`. If so, the destination is changed from register to local.

## Limitations

- Folding only occurs for immediately adjacent instructions
- Control flow instructions (block, loop, if) break folding chains
- Reference types (funcref, externref) are not folded
- Type mismatch between pending operand and consumer prevents folding

