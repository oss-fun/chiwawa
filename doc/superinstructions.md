# Superinstructions in Chiwawa

Superinstruction optimization is a technique that fuses multiple WebAssembly instructions into a single optimized instruction, reducing dispatch overhead and stack operations, improving execution performance in the DTC interpreter.

## Overview

**Enable:** `--superinstructions` flag

**Implementation:** `src/parser.rs` (optimization phase during DTC preprocessing)

## Architecture

### 1. OptimizedOperand Structure

Superinstructions use the `Optimized` operand variant which supports fusion of multiple operations:

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Operand {
    // ... other variants ...

    Optimized(OptimizedOperand),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum OptimizedOperand {
    // For operations that need 1 value (load, unary operations)
    Single {
        value: Option<ValueSource>,
        memarg: Option<Memarg>,            // For memory operations
        store_target: Option<StoreTarget>, // Where to store the result
    },
    // For operations that need 2 values (binary ops, store)
    Double {
        first: Option<ValueSource>,        // binary op's left, or store's addr
        second: Option<ValueSource>,       // binary op's right, or store's value
        memarg: Option<Memarg>,            // For store operations
        store_target: Option<StoreTarget>, // Where to store the result
    },
}
```

### 2. Supporting Types

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValueSource {
    Stack,        // Value from stack (default)
    Const(Value), // Constant value
    Local(u32),   // Local variable
    Global(u32),  // Global variable
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StoreTarget {
    Local(u32),
    Global(u32),
}
```

### 3. Operation Type Classification

Instructions are classified by their operand patterns:

```rust
enum OperationType {
    Producer,    // 0 args, produces value (const, local.get, global.get)
    Unary,       // 1 arg, produces value (clz, ctz, popcnt, etc.)
    Binary,      // 2 args, produces value (add, sub, mul, etc.)
    MemoryLoad,  // 1 arg (address), produces value
    MemoryStore, // 2 args (address, value), no value produced
    ControlFlow, // Variable args, special handling
    Other,       // Non-optimizable
}
```

## Optimization Patterns

Superinstructions optimize the following patterns:

### Pattern 1: Binary Operation with Store

**Original sequence:**
```wat
const 10
local.get 0
i32.add
local.set 1
```

**Optimized:**
```rust
OptimizedOperand::Double {
    first: ValueSource::Const(Value::I32(10)),
    second: ValueSource::Local(0),
    memarg: None,
    store_target: Some(StoreTarget::Local(1)),
}
```

**Benefits:**
- 4 instructions → 1 instruction
- Eliminates 3 stack push/pop operations
- Single dispatch instead of 4

### Pattern 2: Unary Operation with Store

**Original sequence:**
```wat
local.get 0
i32.clz
local.set 1
```

**Optimized:**
```rust
OptimizedOperand::Single {
    value: ValueSource::Local(0),
    memarg: None,
    store_target: Some(StoreTarget::Local(1)),
}
```

**Benefits:**
- 3 instructions → 1 instruction
- Direct local-to-local operation
- No stack operations

### Pattern 3: Memory Load with Constant Address

**Original sequence:**
```wat
i32.const 100
i32.load offset=0
```

**Optimized:**
```rust
OptimizedOperand::Single {
    value: ValueSource::Const(Value::I32(100)),
    memarg: Some(Memarg { offset: 0, align: 2 }),
    store_target: None,
}
```

**Benefits:**
- 2 instructions → 1 instruction
- Constant address embedded in operand
- No stack operation for address

### Pattern 4: Memory Store with Constants

**Original sequence:**
```wat
i32.const 100   ; address
i32.const 42    ; value
i32.store offset=0
```

**Optimized:**
```rust
OptimizedOperand::Double {
    first: ValueSource::Const(Value::I32(100)),  // address
    second: ValueSource::Const(Value::I32(42)),  // value
    memarg: Some(Memarg { offset: 0, align: 2 }),
    store_target: None,
}
```

**Benefits:**
- 3 instructions → 1 instruction
- Both operands embedded
- No stack operations

### Pattern 5: Binary Operation without Store

**Original sequence:**
```wat
i32.const 10
local.get 0
i32.add
```

**Optimized:**
```rust
OptimizedOperand::Double {
    first: ValueSource::Const(Value::I32(10)),
    second: ValueSource::Local(0),
    memarg: None,
    store_target: None,  // Result pushed to stack
}
```

**Benefits:**
- 3 instructions → 1 instruction
- Direct operand access without stack

## Optimization Process

Superinstruction optimization happens during Phase 1 of preprocessing in `decode_processed_instrs_and_fixups()`.

### Algorithm

1. **Track Recent Instructions**
   - Maintain a queue of recent producer instructions (const, local.get, global.get)
   - Track their positions in the instruction stream

2. **Pattern Matching**
   - When processing each instruction, check if it can be optimized
   - Look back at recent instructions to find fusion candidates
   - Check operation type to determine required operands

3. **Look-Ahead for Store Target**
   - Peek at the next instruction
   - If it's `local.set` or `global.set`, include it in the fusion
   - Skip the store instruction in the main loop

4. **Operand Construction**
   - Create `ValueSource` from producer instructions
   - Create `StoreTarget` from store instructions (if present)
   - Build `OptimizedOperand::Single` or `Double`

5. **Instruction Removal**
   - Remove consumed producer instructions from the stream
   - Adjust PC counter
   - Skip store instruction if fused

## Supported Instructions

### Optimizable as Unary Operations
- **Integer:** `i32.clz`, `i32.ctz`, `i32.popcnt`, `i64.clz`, `i64.ctz`, `i64.popcnt`, `i32.eqz`, `i64.eqz`
- **Float:** `f32.abs`, `f32.neg`, `f32.ceil`, `f32.floor`, `f32.trunc`, `f32.nearest`, `f32.sqrt`
- **Float:** `f64.abs`, `f64.neg`, `f64.ceil`, `f64.floor`, `f64.trunc`, `f64.nearest`, `f64.sqrt`
- **Conversions:** `i32.wrap_i64`, `i64.extend_i32_s`, `i64.extend_i32_u`, `f32.demote_f64`, `f64.promote_f32`
- **Type conversions:** All reinterpret and convert operations

### Optimizable as Binary Operations
- **Arithmetic:** `add`, `sub`, `mul`, `div_s`, `div_u`, `rem_s`, `rem_u`
- **Bitwise:** `and`, `or`, `xor`, `shl`, `shr_s`, `shr_u`, `rotl`, `rotr`
- **Comparison:** `eq`, `ne`, `lt_s`, `lt_u`, `le_s`, `le_u`, `gt_s`, `gt_u`, `ge_s`, `ge_u`
- **Float arithmetic:** `add`, `sub`, `mul`, `div`, `min`, `max`, `copysign`
- **Float comparison:** `eq`, `ne`, `lt`, `le`, `gt`, `ge`

### Optimizable Memory Operations
- **Loads:** `i32.load`, `i64.load`, `f32.load`, `f64.load`, `i32.load8_s`, `i32.load8_u`, `i32.load16_s`, `i32.load16_u`, `i64.load8_s`, `i64.load8_u`, `i64.load16_s`, `i64.load16_u`, `i64.load32_s`, `i64.load32_u`
- **Stores:** `i32.store`, `i64.store`, `f32.store`, `f64.store`, `i32.store8`, `i32.store16`, `i64.store8`, `i64.store16`, `i64.store32`