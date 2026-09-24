# Operand Folding

Operand folding removes instructions that only *carry* a value (`i32.const`, `local.get`, `local.set`) by embedding them in a neighbour's operand slot.
The surviving instruction does the same work as before, so a dispatch disappears without any handler having to do more.

```
Before folding:
  i32.const 42    ; r0 = 42
  local.set 0     ; local[0] = r0

After folding:
  i32.const 42 -> local[0]  ; store 42 directly to local[0]
```

## Source Folding

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

## Destination Folding

Folds `local.set` into the preceding instruction that produces the value.

```
Before:
  i32.add        ; r0 = a + b
  local.set 0    ; local[0] = r0

After:
  i32.add -> local[0]  ; result directly to local
```

When destination folding is applied, the instruction uses `RegOrLocal::Local` instead of `RegOrLocal::Reg` for its destination.

## Address Folding (Memory Operations)

For memory load/store operations, folds constant addresses.

```
Before:
  i32.const 100  ; r0 = 100 (address)
  i32.load       ; r1 = memory[r0]

After:
  i32.load (addr: const 100) -> r1
```

## Operand Folding (Register Consumers)

A `local.get` whose value goes straight into a register operand is dropped, and the consumer reads the local's own register.

```
Before:
  local.get 2    ; r0 = local[2]
  local.get 5    ; r1 = local[5]
  call $f        ; params r0, r1

After:
  call $f        ; params local[2], local[5]
```

Unlike source folding, this is decided at the consumer, looking back at the copies that produced its trailing operands, not at the `local.get` looking ahead.
A `local.get` cannot tell on its own whether it will end up as a call argument or a block result: that depends on the function type or the block type and on how many values sit between it and the consumer.
The consumer has all of that in hand, so one check there covers every case.
These consumers hold a plain register, not an operand slot, so only locals fold this way; a constant still needs a register write.
It applies to call arguments (`call`, `call_indirect` with its index, WASI calls), store values, the condition of `if` and `br_if`, the values of `br`, `br_table`, `return` and `end`, the value of `local.set`, and the operands of `select`.
The values of `br_if` and `local.tee` stay on the stack when execution continues past them, so those are left as copies.

## Implementation

Folding is performed during instruction decoding using a peek-ahead mechanism:

1. **Pending Operand Stack**: When a foldable source instruction (const, local.get) is encountered, it is pushed to a pending stack instead of generating a register instruction.

2. **Consumer Check**: When a consuming instruction is processed, it checks the pending stack for compatible operands.

3. **Destination Check**: After processing an instruction, the parser peeks ahead to check if the next instruction is `local.set`. If so, the destination is changed from register to local.

4. **Look-back**: A consumer that takes plain registers checks whether the instructions just before it are the `local.get` copies feeding those registers. If so, it takes the locals' registers instead and the copies become no-ops, which compaction strips.

## Limitations

- Folding only occurs for immediately adjacent instructions
- Control flow instructions (block, loop, if) break folding chains
- Reference types (funcref, externref) are not folded
- Type mismatch between pending operand and consumer prevents folding
