# Direct Threaded Code (DTC) in stackopt.rs (Detailed Explanation)

This document provides a detailed explanation of the Direct Threaded Code (DTC) mechanism implemented in `src/execution/stackopt.rs` and the rationale behind its design.

## What is DTC? Why Use It?

Direct Threaded Code (DTC) is a technique for speeding up interpreters (especially for virtual machines and bytecode).

**Purpose:** To reduce the overhead of dispatch (determining which instruction to execute) when executing Wasm instructions.

**Comparison with Normal Interpreters:** A simple interpreter reads an instruction code (`opcode`) and uses `match` or `switch` statements to find the corresponding process.
If there are many types of instructions, instruction dispatch and branching can become a performance bottleneck.

**DTC Approach:** In DTC, the addresses of handlers (processes corresponding to each instruction) are stored in a table (`HANDLER_TABLE`) beforehand.
During execution, the index for the handler table is read from the instruction stream, the address of the handler function is obtained using the table, and the function at that address is called.
This eliminates large branching processes like `match` statements from the execution loop, leading to faster execution.

## DTC Implementation Details in `stackopt.rs`

DTC is achieved in `stackopt.rs` through the following key elements and steps:

### 1. Key Data Structures

Data structures are optimized for efficient DTC execution.

*   **`Operand` enum:**
    *   **Role:** Represents preprocessed instruction operands (arguments).
    *   **Rationale:** Eliminates the cost of parsing instruction arguments from a byte stream at runtime. Crucially, branch targets (`LabelIdx(usize)`) hold the **absolute instruction pointer (PC)** calculated during preprocessing, making runtime jump target calculation unnecessary. `BrTable` similarly holds a resolved list of targets.
    ```rust
    #[derive(Clone, Debug, PartialEq)]
    pub enum Operand {
        None,
        I32(i32), // Immediate value
        // ...
        LocalIdx(LocalIdx), // Local variable index
        GlobalIdx(GlobalIdx), // Global variable index
        LabelIdx(usize), // ★ Resolved absolute target PC for branches
        MemArg(Memarg), // Memory access info
        BrTable { targets: Vec<usize>, default: usize }, // ★ Resolved BrTable targets
    }
    ```

*   **`ProcessedInstr` struct:**
    *   **Role:** Represents a single preprocessed instruction.
    *   **Rationale:** Holds the information needed by the execution loop: "which handler to run next (`handler_index`)" and "what arguments does that handler need (`operand`)". This allows the execution loop to focus on simple index lookups and function calls.
    ```rust
    #[derive(Clone, Debug)]
    pub struct ProcessedInstr {
        handler_index: usize, // Index into the handler table
        operand: Operand,     // Resolved operand
    }
    ```

*   **`ExecutionContext` struct:**
    *   **Role:** The execution context passed to each handler function.
    *   **Rationale:** Provides handler functions with access to modify the current execution state (value stack, local variables, current instruction pointer `ip`, etc.). It's passed as `&mut` (mutable reference) to allow state changes. Including `ip` simplifies the common case where a handler just proceeds to the next instruction (`Ok(ctx.ip + 1)`).
    ```rust
    pub struct ExecutionContext<'a> {
        pub frame: &'a mut crate::execution::stackopt::Frame, // Access to locals, etc.
        pub value_stack: &'a mut Vec<Val>,                 // Value stack operations
        pub ip: usize,                                     // Current instruction pointer
    }
    ```

*   **`HandlerFn` type:**
    *   **Role:** Defines the unified signature (type) for instruction handler functions.
    *   **Rationale:** Allows storing pointers to different handler functions in the `HANDLER_TABLE`. The return type `Result<usize, RuntimeError>` is a standard way to return the next instruction pointer (`usize`) on success or an error. Special `usize` values (`usize::MAX`, `usize::MAX - 1`) are used as sentinels to efficiently signal special control flow transitions like Call and Return to the execution loop without changing the return type.

    ```rust
    type HandlerFn = fn(&mut ExecutionContext, Operand) -> Result<usize, RuntimeError>;
    ```

### 2. Preprocessing (`preprocess_instructions` function) and Branch Resolution (Fixup)

*   **Purpose:** To convert `Vec<Instr>` to `Vec<ProcessedInstr>` before execution, in order to reduce runtime costs (especially branch resolution) as much as possible.
This "preprocessing cost" is amortized when functions are called frequently.
*  Branch resolution has dependencies, requiring multiple passes.
    *   Pass 1 converts instructions while recording unresolved branch information in `fixups`.
    *   Pass 2 records the positions of `End` and `Else` in a `HashMap`. This is necessary for calculating branch targets in Pass 3/4.
    *   Pass 3/4 processes `fixups`, calculates the absolute target PC using the map information from Pass 2 and **control stack reconstruction**, and updates the `ProcessedInstr` operands. `BrTable` has multiple targets and is processed in Pass 4 after other branch instructions are resolved.
*   **Reason for Control Stack Reconstruction (Pass 3/4):** Wasm branches use relative depth. To resolve this to an absolute PC, the correct nesting structure at the point of each branch instruction (`fixup_pc`) must be known.
Therefore, for each fixup, the instruction stream is scanned from the beginning up to `fixup_pc` to reproduce the control stack (`current_control_stack_passX`) at that point.
This allows identifying the correct target block using `relative_depth`.
    ```rust
    // Control stack reconstruction loop in Pass 3 (simplified)
    current_control_stack_pass3.clear();
    for (pc, instr) in processed.iter().enumerate().take(fixup_pc + 1) {
        match instr.handler_index {
            HANDLER_IDX_BLOCK | HANDLER_IDX_LOOP | HANDLER_IDX_IF => { /* push */ }
            HANDLER_IDX_END => { /* pop */ }
            _ => {}
        }
    }
    // At this point, current_control_stack_pass3 represents the nesting state at fixup_pc
    ```
*   **Pass 3: Br, BrIf, If, Else Fixup**
    *   Processes branch information in the `fixups` vector, excluding `BrTable`.
    *   For each Fixup info `(fixup_pc, relative_depth, ...)`:
        1.  **Control Stack Reconstruction:** Scan the `processed` list from the start up to `fixup_pc` and reproduce the nesting state of `Block`, `Loop`, `If` onto a stack.
        2.  **Identify Target Block:** Use `relative_depth` to identify the target block's start PC (`target_start_pc`) from the reconstructed stack.
        3.  **Calculate Absolute Target PC:** If the target is `Loop`, use `target_start_pc`; if `Block`/`If`, use `block_end_map` to find the PC after the corresponding `End` (`target_ip`).
        4.  **Update Operand:** Write the calculated `target_ip` into `processed[fixup_pc]`'s `operand` (`Operand::LabelIdx`). Also consult `if_else_map` for `If`/`Else`.

         ```text
        Example Instruction Stream (`processed`):
        | PC (ip) | Handler Index        | Description              |
        | :------ | :------------------- | :----------------------- |
        | 0       | HANDLER_IDX_BLOCK    | Outer Block Start        |
        | 1       | HANDLER_IDX_I32_CONST |                          |
        | 2       | HANDLER_IDX_IF       | If Start                 |
        | 3       | HANDLER_IDX_I32_CONST | (then clause)            |
        | 4       | HANDLER_IDX_LOCAL_SET | (then clause)            |
        | 5       | HANDLER_IDX_BR       | Br 0 (fixup target @ pc=5)|
        | 6       | HANDLER_IDX_END      | If End                   |
        | 7       | HANDLER_IDX_END      | Outer Block End        |

        Fixup Target: pc = 5, relative_depth = 0 (Original instruction: Br 0)
        ----------------------------------------------------------
        1. Control Stack Reconstruction (Scan pc = 0 to 5):
           pc = 0 (BLOCK): push (0, false) -> Stack: [(0, false)]
           pc = 1 (CONST): no change     -> Stack: [(0, false)]
           pc = 2 (IF):    push (2, false) -> Stack: [(0, false), (2, false)]
           pc = 3 (CONST): no change     -> Stack: [(0, false), (2, false)]
           pc = 4 (SET):   no change     -> Stack: [(0, false), (2, false)]
           pc = 5 (BR):    Scan finished.
           ==> Reconstructed Stack: [(0, false), (2, false)]

        2. Identify Target Block:
           - relative_depth = 0, so get the top element from the stack.
           - target_block = (pc=2, is_loop=false)
           - target_start_pc = 2
           - is_loop = false

        3. Calculate Absolute Target PC:
           - Since is_loop is false, use block_end_map.
           - Assume Pass 2 recorded that "Block starting at PC=2 ends after PC=6 (i.e., at PC=7)" (block_end_map[2] == 7).
           - target_ip = block_end_map[&target_start_pc] = block_end_map[&2] = 7

        4. Update Operand:
           - Update the operand of processed[fixup_pc] (i.e., processed[5]).
           - Original instruction was Br, so set the calculated target_ip.
           - processed[5].operand = Operand::LabelIdx(7)
        ----------------------------------------------------------
        ```

*   **Pass 4: BrTable Fixup**
    *   Processes `BrTable` instruction operands (those still `Operand::None`).
    *   **Reason:** `BrTable` has multiple targets, so processing it after other branches are resolved (Pass 3 completed) ensures correct control stack reconstruction for each target resolution.
    *   For each `BrTable` instruction (`pc`):
        1.  **Identify Related Fixups:** Find all entries in `fixups` corresponding to `pc`.
        2.  **Resolve Each Target:** For each found fixup info, perform control stack reconstruction and map lookup similar to Pass 3 to calculate the absolute target PC (the last fixup corresponds to the default branch).
        3.  **Update Operand:** Write the list of all resolved target PCs and the default PC into `processed[pc]`'s `operand` as `Operand::BrTable { targets: [...], default: ... }`.

This multi-pass Fixup ensures that the execution loop only needs to use the absolute PC within the `operand` for branch instructions.

### 3. Handler Table (`HANDLER_TABLE`)

*   **Role:** Provides the mapping from an instruction's index to its corresponding handler function.
*   **Reason:** Enables fast O(1) lookup using `handler_index`.
`lazy_static!` is used to safely initialize the static vector containing function pointers, as direct assignment in `const` context is restricted.

    ```rust
    lazy_static! {
        static ref HANDLER_TABLE: Vec<HandlerFn> = {
            let mut table: Vec<HandlerFn> = vec![handle_unimplemented; MAX_HANDLER_INDEX];
            // ... (Assign handlers for each instruction index) ...
            table[HANDLER_IDX_I32_ADD] = handle_i32_add;
            table[HANDLER_IDX_LOCAL_GET] = handle_local_get;
            table[HANDLER_IDX_BR] = handle_br;
            table[HANDLER_IDX_CALL] = handle_call;
            // ... (Other implemented handlers) ...
            table
        };
    }
    ```

### 4. Instruction Handlers (`handle_*` )

*   **Role:** Implement the semantics (stack operations, computation, memory/table/global access, etc.) for individual Wasm instructions.
*   **Return Value:**
    *   `Ok(ctx.ip + 1)`: Most common case; simply proceed to the next instruction.
    *   `Ok(target_ip)`: For branch instructions; returns the pre-calculated absolute PC, making calculation by the loop unnecessary.
    *   `Ok(usize::MAX - 1)` / `Ok(usize::MAX)`: For Call/Return; signals to the execution loop that a special action (frame manipulation) is needed.

    ```rust
    // Example: i32.add (using macro)
    fn handle_i32_add(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
        // binop_wrapping! macro hides routine stack operations and calculation
        binop_wrapping!(ctx, I32, wrapping_add) // Result: Ok(ctx.ip + 1)
    }

    // Example: br (value transfer is TODO)
    fn handle_br(_ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
        if let Operand::LabelIdx(target_ip) = operand { // ★ Use resolved PC
            // TODO: Handle value transfer
            Ok(target_ip) // ★ Return target PC as next IP
        } else { /* Error */ }
    }

    // Example: call
    fn handle_call(_ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
        if let Operand::FuncIdx(_) = operand {
             Ok(usize::MAX - 1) // Return Call signal
        } else {
            Err(RuntimeError::InvalidOperand)
        }
    }

    // Example: i32.load
    fn handle_i32_load(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
        if let Operand::MemArg(arg) = operand {
            let ptr = ctx.value_stack.pop()?.to_i32();
            let mem_addr = &ctx.frame.module.upgrade()?.mem_addrs[0]; // Assume memidx 0
            let val = mem_addr.load::<i32>(&arg, ptr)?;
            ctx.value_stack.push(Val::Num(Num::I32(val)));
            Ok(ctx.ip + 1)
        } else { /* Error */ }
    }
    ```

### 5. Execution Loop (`FrameStack::run_dtc_loop` )

*   **Role:** Executes the `ProcessedInstr` sequence within the current function frame quickly.
*   Eliminates `match`-based dispatch, reducing interpreter overhead through simple table lookups (`HANDLER_TABLE.get(...)`) and function calls (`handler_fn(...)`). Operations spanning frames like Call/Return are delegated to `exec_instr`.

### 6. Frame Management (`Stacks::exec_instr` )

*   **Role:** Manages the overall function call control and the function frame (`FrameStack`).
*   Since `run_dtc_loop` specializes in intra-frame execution, this method handles processes spanning frames, such as frame creation (including calling `preprocess_instructions` during `Invoke`), destruction (during `Return`), and host function calls.
