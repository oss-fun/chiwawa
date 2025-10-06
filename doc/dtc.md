# Direct Threaded Code (DTC) in stack.rs

Direct Threaded Code (DTC) is a technique for accelerating interpreters (especially virtual machines and bytecode interpreters).

**Purpose:** Reduce the overhead of dispatching (the process of determining which instruction to execute) when executing Wasm instructions.

**Comparison with Regular Interpreters:** In a simple interpreter, the instruction code (`opcode`) is read and the corresponding process is found using a `match` or `switch` statement. When there are many types of instructions, instruction dispatch and branching become a bottleneck in execution speed.

**DTC Approach:** In DTC, addresses of handlers corresponding to each instruction are stored in a table (`HANDLER_TABLE`) beforehand. During execution, the handler table index is read from the instruction sequence, the handler function address is obtained using the table, and that function is called. This eliminates large branching processes like `match` statements from the execution loop.

## DTC Implementation in `stack.rs`

### 1. Main Data Structures

*   **`Operand` enum:**
    -  **Role:** Represents preprocessed instruction operands (arguments).

    Eliminates the cost of parsing instruction arguments from byte sequences at runtime.
    Specifically, branch instruction jump targets (`LabelIdx`) hold **absolute instruction pointers (PC)** calculated by preprocessing, along with additional metadata like arity and depth information.
    `BrTable` holds a list of resolved target operands.

    ```rust
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub enum Operand {
        None,
        I32(i32),
        I64(i64),
        F32(f32),
        F64(f64),
        LocalIdx(LocalIdx),
        LocalIdxI32(LocalIdx, i32),  // Superinstruction: local.set with constant
        LocalIdxI64(LocalIdx, i64),
        LocalIdxF32(LocalIdx, f32),
        LocalIdxF64(LocalIdx, f64),
        MemArgI32(i32, Memarg),      // Superinstruction: memory op with constant
        MemArgI64(i64, Memarg),
        GlobalIdx(GlobalIdx),
        FuncIdx(FuncIdx),
        TableIdx(TableIdx),
        TypeIdx(TypeIdx),
        RefType(RefType),
        LabelIdx {                    // ★Branch target with metadata
            target_ip: usize,         // Absolute PC to jump to
            arity: usize,             // Number of values to transfer
            original_wasm_depth: usize, // Original relative depth from Wasm
            is_loop: bool,            // Whether target is a loop
        },
        MemArg(Memarg),
        BrTable {                     // ★BrTable with resolved targets
            targets: Vec<Operand>,    // Each target is a LabelIdx operand
            default: Box<Operand>,    // Default target as LabelIdx operand
        },
        CallIndirect {
            type_idx: TypeIdx,
            table_idx: TableIdx,
        },
        Block {
            arity: usize,
            param_count: usize,
            is_loop: bool,
            start_ip: usize,
            end_ip: usize,
        },

        // Unified optimized operand
        Optimized(OptimizedOperand),
    }

    // Supporting types for OptimizedOperand
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

*   **`ProcessedInstr` struct:**
    *   **Role:** Represents one preprocessed instruction.

    Holds the information needed by the execution loop: "which handler to execute next (`handler_index`)" and "what arguments that handler needs (`operand`)".
    The execution loop only performs simple index references and function calls.
    ```rust
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ProcessedInstr {
        pub handler_index: usize, // Index into handler table
        pub operand: Operand,     // Resolved operand
    }
    ```

*   **`ExecutionContext` struct:**
    *   **Role:** Execution context passed to each handler function.

    Allows handler functions to access and modify the current execution state (value stack, local variables, current instruction pointer `ip`, etc.).
    Passed as `&mut` (mutable reference) to enable state changes by handlers.
    ```rust
    pub struct ExecutionContext<'a> {
        pub frame: &'a mut crate::execution::stack::Frame,
        pub value_stack: &'a mut Vec<Val>,
        pub ip: usize,
        pub block_has_mutable_op: bool,                    // Track mutable operations
        pub accessed_globals: &'a mut Option<GlobalAccessTracker>, // Track global access
        pub accessed_locals: &'a mut Option<LocalAccessTracker>,   // Track local access
    }
    ```

*   **`HandlerResult` enum:**
    *   **Role:** Represents the result of executing an instruction handler.

    Provides type-safe control flow instead of using magic numbers. Handlers return different variants depending on the instruction semantics.
    ```rust
    enum HandlerResult {
        Continue(usize),           // Continue to next instruction at given IP
        Return,                    // Return from current function
        Invoke(FuncAddr),         // Invoke another function
        Branch {                   // Branch with value transfer
            target_ip: usize,
            values_to_push: Vec<Val>,
            branch_depth: usize,
        },
        PushLabelStack { ... },   // Enter new block/loop/if
        PopLabelStack { ... },    // Exit block/loop/if
    }
    ```

*   **`HandlerFn` type:**
    *   **Role:** Defines the unified signature (type) of instruction handler functions.

    ```rust
    type HandlerFn = fn(&mut ExecutionContext, &Operand) -> Result<HandlerResult, RuntimeError>;
    ```

### 2. BlockType

**BlockType** (from `wasmparser::BlockType`) represents the type signature of control flow structures (block/loop/if). It describes what parameters they accept and what results they produce.

**Variants:**
```rust
pub enum BlockType {
    Empty,              // No parameters, no results
    Type(ValType),      // No parameters, single result (i32/i64/f32/f64)
    FuncType(u32),      // Function type index - can have multiple params/results
}
```

**Role in Branch Resolution:**

BlockType is critical for calculating **arity** - the number of values a branch instruction must transfer:

- **For loops**: Branch provides **parameters** (input types from BlockType)
  - Example: `loop (param i32 i64) ... end` - branch to loop transfers 2 values

- **For blocks/if**: Branch provides **results** (output types from BlockType)
  - Example: `block (result i32) ... end` - branch to block end transfers 1 value

**Arity Calculation:**
```rust
let target_arity = if is_loop {
    calculate_loop_parameter_arity(&target_block_type, module, cache)  // Input types count
} else {
    calculate_block_arity(&target_block_type, module, cache)  // Output types count
};
```

This arity is stored in the `LabelIdx` operand and used at runtime to determine how many values to pop from the stack when executing a branch.

### 3. Preprocessing and Branch Resolution (Fixup)

**Purpose:** Convert Wasm instruction sequences to a DTC-friendly format (`Vec<ProcessedInstr>`) to minimize runtime cost (especially branch resolution). This is done in multiple phases, all within `parser.rs`.

#### **Phase 1: Decode and Map Building** (`decode_processed_instrs_and_fixups` in parser.rs)

This phase performs multiple tasks simultaneously while scanning through Wasm operators:

1. **Convert opcodes to ProcessedInstr:**
   - Each Wasm operator is converted to a `ProcessedInstr` with appropriate `handler_index`
   - Initial `Operand` is set (placeholders for branches)
   - Collect fixup information for branch instructions

2. **Build control flow maps:**
   - `block_end_map`: Maps block/loop/if start PC → end PC (after End instruction)
   - `if_else_map`: Maps if start PC → else PC (or end PC if no else)
   - `block_type_map`: Maps block/loop/if start PC → block type (for arity calculation)

3. **Apply superinstruction optimizations** (if enabled):
   - Fuse `const` + `local.set` into `LocalIdxI32/I64/F32/F64`
   - Fuse `const` + memory ops into `MemArgI32/I64`

**Control Stack for Map Building:**
```rust
// Tracks: (start_pc, is_if, else_pc_opt)
let mut control_stack_for_map_building: Vec<(usize, bool, Option<usize>)> = Vec::new();

match op {
    Block { blockty } => {
        control_stack_for_map_building.push((current_pc, false, None));
    }
    If { blockty } => {
        control_stack_for_map_building.push((current_pc, true, None));
    }
    Else => {
        // Mark else position in the current if's stack entry
        if let Some((_, true, else_pc @ None)) = control_stack_for_map_building.last_mut() {
            *else_pc = Some(current_pc + 1);
        }
    }
    End => {
        if let Some((start_pc, is_if, else_pc_opt)) = control_stack_for_map_building.pop() {
            block_end_map.insert(start_pc, current_pc + 1);
            if is_if {
                let else_target = else_pc_opt.unwrap_or(current_pc + 1);
                if_else_map.insert(start_pc, else_target);
            }
        }
    }
}
```

**Output:** `(processed_instrs, fixups, block_end_map, if_else_map, block_type_map)`

#### **Phase 2: Resolve Br, BrIf, If, Else** (`preprocess_instructions` in parser.rs)

Uses the maps from Phase 1 to resolve branch targets.

**For each fixup (except BrTable):**

1. **Reconstruct control stack up to fixup point:**
   ```rust
   let mut current_control_stack: Vec<(usize, bool, BlockType)> = Vec::new();

   for (pc, instr) in processed.iter().enumerate().take(fixup_pc + 1) {
       match instr.handler_index {
           HANDLER_IDX_BLOCK | HANDLER_IDX_IF => {
               let block_type = block_type_map.get(&pc).cloned().unwrap_or(Empty);
               current_control_stack.push((pc, false, block_type));
           }
           HANDLER_IDX_LOOP => {
               let block_type = block_type_map.get(&pc).cloned().unwrap_or(Empty);
               current_control_stack.push((pc, true, block_type));
           }
           HANDLER_IDX_END => {
               current_control_stack.pop();
           }
           _ => {}
       }
   }
   ```

2. **Calculate target from relative depth:**
   ```rust
   let target_stack_level = current_control_stack.len() - 1 - relative_depth;
   let (target_start_pc, is_loop, target_block_type) = current_control_stack[target_stack_level];

   let target_ip = if is_loop {
       target_start_pc  // Loop: branch back to start
   } else {
       block_end_map[&target_start_pc]  // Block/If: branch to end
   };
   ```

3. **Calculate arity:**
   ```rust
   let target_arity = if is_loop {
       calculate_loop_parameter_arity(&target_block_type, module)  // Input types
   } else {
       calculate_block_arity(&target_block_type, module)  // Output types
   };
   ```

4. **Patch operand with full LabelIdx struct:**
   ```rust
   instr_to_patch.operand = Operand::LabelIdx {
       target_ip,
       arity: target_arity,
       original_wasm_depth: relative_depth,
       is_loop,
   };
   ```

**Special cases:**
- **If instruction (is_if_false_jump=true):** Target is else or end from `if_else_map`
- **Else instruction (is_else_jump=true):** Target is end from `block_end_map`

#### **Phase 3: Resolve BrTable Targets** (`preprocess_instructions` in parser.rs)

Similar to Phase 2 but processes BrTable instructions with multiple targets.

**Process:**

1. **Scan instructions and maintain control stack:**
   ```rust
   let mut current_control_stack: Vec<(usize, bool, BlockType)> = Vec::new();

   for pc in 0..processed.len() {
       // Update control stack
       match instr.handler_index {
           HANDLER_IDX_BLOCK | HANDLER_IDX_IF => { push to stack }
           HANDLER_IDX_LOOP => { push to stack with is_loop=true }
           HANDLER_IDX_END => { pop from stack }
       }

       // Check if this is a BrTable needing resolution
       if instr.handler_index == HANDLER_IDX_BR_TABLE && instr.operand == Operand::None {
           // Resolve all targets for this BrTable
       }
   }
   ```

2. **For each BrTable, resolve all targets:**
   ```rust
   // Get all fixups for this BrTable PC
   let mut fixup_indices = fixups.iter()
       .filter(|f| f.pc == pc && f.original_wasm_depth != usize::MAX)
       .collect();

   // Last fixup is default target
   let default_fixup = fixup_indices.pop();

   // Resolve each target using control stack
   for each fixup_depth {
       let target_level = control_stack.len() - 1 - fixup_depth;
       let (target_pc, is_loop, block_type) = control_stack[target_level];

       let target_ip = if is_loop { target_pc } else { block_end_map[&target_pc] };
       let arity = calculate_arity(&block_type, is_loop);

       targets.push(Operand::LabelIdx {
           target_ip,
           arity,
           original_wasm_depth: fixup_depth,
           is_loop,
       });
   }

   // Patch BrTable operand
   instr.operand = Operand::BrTable {
       targets: resolved_targets,
       default: Box::new(default_target_operand),
   };
   ```

#### **Phase 4: Sanity Check** (`preprocess_instructions` in parser.rs)

Verifies all fixups were processed:

```rust
for fixup in fixups {
    if fixup.original_wasm_depth != usize::MAX {
        return Err("Unprocessed fixup after preprocessing");
    }
}
```

### 4. Phase 2 Example: Br Fixup with Control Stack Reconstruction

```
| PC (ip) | Handler Index        | Description              |
| :------ | :------------------- | :----------------------- |
| 0       | HANDLER_IDX_BLOCK    | Outer Block start        |
| 1       | HANDLER_IDX_I32_CONST|                          |
| 2       | HANDLER_IDX_IF       | If start                 |
| 3       | HANDLER_IDX_I32_CONST| (then clause)            |
| 4       | HANDLER_IDX_LOCAL_SET| (then clause)            |
| 5       | HANDLER_IDX_BR       | Br 0 (fixup @ pc=5)     |
| 6       | HANDLER_IDX_END      | If end                   |
| 7       | HANDLER_IDX_END      | Outer Block end          |

Fixup Target: pc = 5, relative_depth = 0 (Original instruction: Br 0)
----------------------------------------------------------
1. Control Stack Reconstruction (scan from pc = 0 to 5):
   pc = 0 (BLOCK): push (0, false, Empty) -> Stack: [(0, false, Empty)]
   pc = 1 (CONST): no change              -> Stack: [(0, false, Empty)]
   pc = 2 (IF):    push (2, false, Empty) -> Stack: [(0, false, Empty), (2, false, Empty)]
   pc = 3 (CONST): no change              -> Stack: [(0, false, Empty), (2, false, Empty)]
   pc = 4 (SET):   no change              -> Stack: [(0, false, Empty), (2, false, Empty)]
   pc = 5 (BR):    scan complete
   ==> Reconstructed Stack: [(0, false, Empty), (2, false, Empty)]

2. Target Block Identification:
   - relative_depth = 0, so target_stack_level = len - 1 - 0 = 1
   - target_block = control_stack[1] = (pc=2, is_loop=false, block_type=Empty)
   - target_start_pc = 2

3. Absolute Target PC Calculation:
   - is_loop is false, so use block_end_map
   - block_end_map[2] = 7 (end of if block)
   - target_ip = 7

4. Calculate Arity:
   - is_loop is false, so use calculate_block_arity(Empty) = 0
   - arity = 0

5. Operand Update:
   - processed[5].operand = Operand::LabelIdx {
       target_ip: 7,
       arity: 0,
       original_wasm_depth: 0,
       is_loop: false,
   }
```

### 5. Handler Table (`HANDLER_TABLE`)

*   **Role:** Provides mapping from instruction index to the corresponding handler function.
*   **Implementation:** Uses `lazy_static!` for safe initialization of function pointer array.

```rust
lazy_static! {
    static ref HANDLER_TABLE: Vec<HandlerFn> = {
        let mut table: Vec<HandlerFn> = vec![handle_unimplemented; MAX_HANDLER_INDEX];
        table[HANDLER_IDX_I32_ADD] = handle_i32_add;
        table[HANDLER_IDX_LOCAL_GET] = handle_local_get;
        table[HANDLER_IDX_BR] = handle_br;
        table[HANDLER_IDX_CALL] = handle_call;
        // ... other handlers ...
        table
    };
}
```

### 6. Instruction Handlers (`handle_*`)

*   **Role:** Implement the semantics of individual Wasm instructions.
*   **Return Values:**
    *   `Ok(HandlerResult::Continue(ctx.ip + 1))`: Proceed to next instruction
    *   `Ok(HandlerResult::Branch { target_ip, ... })`: Branch to target with value transfer
    *   `Ok(HandlerResult::Invoke(func_addr))`: Call function
    *   `Ok(HandlerResult::Return)`: Return from function

```rust
// Example: br
fn handle_br(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    if let Operand::LabelIdx {
        target_ip,
        arity,
        original_wasm_depth,
        is_loop: _,
    } = operand
    {
        if *target_ip == usize::MAX {
            return Err(RuntimeError::ExecutionFailed(
                "Branch fixup not done for Br",
            ));
        }

        let values_to_push = ctx.pop_n_values(*arity)?;

        Ok(HandlerResult::Branch {
            target_ip: *target_ip,
            values_to_push,
            branch_depth: *original_wasm_depth,
        })
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}

// Example: br_table
fn handle_br_table(ctx: &mut ExecutionContext, operand: &Operand) -> Result<HandlerResult, RuntimeError> {
    // First pop the index
    let i_val = ctx
        .value_stack
        .pop()
        .ok_or(RuntimeError::ValueStackUnderflow)?;
    let i = i_val.to_i32()?;

    if let Operand::BrTable { targets, default } = operand {
        let chosen_operand = if let Some(target_operand) = targets.get(i as usize) {
            target_operand
        } else {
            default
        };

        if let Operand::LabelIdx {
            target_ip,
            arity,
            original_wasm_depth,
            is_loop: _,
        } = chosen_operand
        {
            if *target_ip == usize::MAX {
                return Err(RuntimeError::ExecutionFailed(
                    "Branch fixup not done for BrTable target",
                ));
            }

            // Then pop the values needed for the branch target
            let values_to_push = if *arity > 0 {
                ctx.pop_n_values(*arity)?
            } else {
                Vec::new()
            };

            Ok(HandlerResult::Branch {
                target_ip: *target_ip,
                values_to_push,
                branch_depth: *original_wasm_depth,
            })
        } else {
            Err(RuntimeError::InvalidOperand)
        }
    } else {
        Err(RuntimeError::InvalidOperand)
    }
}
```

### 7. Execution Loop (`FrameStack::run_dtc_loop`)

*   **Role:** Fast execution of `ProcessedInstr` sequences within the current function frame.
*   Eliminates `match`-based dispatch through simple table lookup and function calls.

```rust
pub fn run_dtc_loop(&mut self, ...) -> Result<...> {
    loop {
        let ip = current_label_stack.ip;
        let instruction_ref = &processed_code[ip];

        let handler_fn = HANDLER_TABLE.get(instruction_ref.handler_index)?;

        let mut context = ExecutionContext {
            frame: &mut self.frame,
            value_stack: &mut self.global_value_stack,
            ip,
            block_has_mutable_op: false,
            accessed_globals: &mut self.current_block_accessed_globals,
            accessed_locals: &mut self.current_block_accessed_locals,
        };

        let result = handler_fn(&mut context, &instruction_ref.operand);

        // Process result (Continue, Branch, Invoke, Return, etc.)
    }
}
```

### 8. Tracking Mechanisms

The current implementation includes tracking mechanisms for optimization:

*   **Global Access Tracking (`accessed_globals`):** Tracks which global variables are accessed during block execution
*   **Local Access Tracking (`accessed_locals`):** Tracks which local variables are accessed during block execution
*   **Mutable Operation Tracking (`block_has_mutable_op`):** Tracks whether a block contains operations that mutate state

These tracking mechanisms are used for caching and optimization strategies.
