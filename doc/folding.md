# Operand Folding

Some instructions only move a value (`i32.const`, `local.get`, `local.set`).
Operand folding removes such an instruction and puts its value into an adjacent instruction.
The instruction that stays does the same work as before.
Thus one dispatch goes away, and no handler does more work.

```
Before folding:
  i32.const 42    ; r0 = 42
  local.set 0     ; local[0] = r0

After folding:
  i32.const 42 -> local[0]  ; store 42 directly to local[0]
```

## Source Folding

A constant or a `local.get` becomes an operand of the instruction that uses its value.
This includes the address of a load or a store.

```
Before:
  i32.const 10   ; r0 = 10
  local.get 3    ; r1 = local[3]
  i32.add        ; r2 = r0 + r1

After:
  i32.add (const 10), local[3] -> r0
```

## Destination Folding

A `local.set` goes away.
The instruction that makes the value writes directly to the register of the local.

```
Before:
  i32.add        ; r0 = a + b
  local.set 0    ; local[0] = r0

After:
  i32.add -> local[0]
```

## Register Consumers

Call arguments, store values, branch conditions, branch values, and `select` operands are registers, not operand slots.
When a `local.get` supplies one of them, Chiwawa removes the `local.get`, and the consumer reads the register of the local.
For a constant, the copy stays.

```
Before:
  local.get 2    ; r0 = local[2]
  local.get 5    ; r1 = local[5]
  call $f        ; params r0, r1

After:
  call $f        ; params local[2], local[5]
```

The values of `br_if` and `local.tee` stay on the stack when execution continues after them.
Thus those instructions keep their copies.

## Implementation

The consumer makes the decision for each fold.
It looks back at the instructions that the parser emitted immediately before it.
A `local.get` cannot know if it will become an `i32.add` operand, a call argument, or a block result, but the consumer knows.

1. `local.get` and the constants always emit a copy into a new register.
2. A consumer goes back from the last emitted instruction, one operand at a time, from the top of the stack down.
   When a copy wrote the register of the operand, that copy becomes the operand, and Chiwawa changes the copy into a no-op.
3. `local.set` changes the destination of the previous instruction to the local, unless a `drop` came between them.
4. Compaction removes the no-ops and maps the branch targets again.

## Limits

- A copy folds only into the instruction that uses it next.
  An instruction between the two stops the search.
- Control flow instructions (block, loop, if) stop a sequence of folds.
- Chiwawa does not fold reference types (funcref, externref).
