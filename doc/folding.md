# Operand Folding

Operand folding removes instructions that only *carry* a value (`i32.const`, `local.get`, `local.set`) by embedding them in a neighbour.
The surviving instruction does the same work as before, so a dispatch disappears without any handler having to do more.

```
Before folding:
  i32.const 42    ; r0 = 42
  local.set 0     ; local[0] = r0

After folding:
  i32.const 42 -> local[0]  ; store 42 directly to local[0]
```

## Source Folding

A constant or a `local.get` becomes an operand of the instruction that consumes it.
This includes the address of a load or store.

```
Before:
  i32.const 10   ; r0 = 10
  local.get 3    ; r1 = local[3]
  i32.add        ; r2 = r0 + r1

After:
  i32.add (const 10), local[3] -> r0
```

## Destination Folding

A `local.set` disappears, and the instruction that produces the value writes the local's register directly.

```
Before:
  i32.add        ; r0 = a + b
  local.set 0    ; local[0] = r0

After:
  i32.add -> local[0]
```

## Register Consumers

Call arguments, store values, branch conditions and values, and `select` operands are plain registers, not operand slots.
A `local.get` feeding one of them is dropped and the consumer reads the local's register; a constant still needs its copy.

```
Before:
  local.get 2    ; r0 = local[2]
  local.get 5    ; r1 = local[5]
  call $f        ; params r0, r1

After:
  call $f        ; params local[2], local[5]
```

The values of `br_if` and `local.tee` stay on the stack when execution continues past them, so those keep their copies.

## Implementation

Every fold is decided at the consumer, which looks back at the instructions emitted just before it.
A `local.get` cannot tell on its own whether it will end up as an `i32.add` operand, a call argument or a block result; the consumer knows.

1. `local.get` and the constants always emit a copy into a fresh register.
2. A consumer walks back from the last emitted instruction, one operand at a time from the top of the stack down.
   A copy that wrote the operand's register becomes the operand and is turned into a no-op.
3. `local.set` points the previous instruction's destination at the local instead, unless a `drop` came between them.
4. Compaction strips the no-ops and remaps branch targets.

## Limitations

- A copy folds only into the instruction that consumes it next; anything in between stops the walk
- Control flow instructions (block, loop, if) break folding chains
- Reference types (funcref, externref) are not folded
