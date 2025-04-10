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

### 2. Preprocessing and Branch Resolution (Fixup)

*   **Purpose:** To convert the Wasm instruction sequence into a format suitable for DTC execution (`Vec<ProcessedInstr>`), minimizing runtime costs, especially for branch resolution. This conversion happens in multiple phases.
*   **Phases:**
    *   **Phase 1 (In Parser: `parser.rs`):**
        *   As the Wasm bytecode function body is parsed, each opcode is directly converted into a `ProcessedInstr` containing the corresponding `handler_index` and an initial `Operand`.
        *   For branch instructions (`Br`, `BrIf`, `If`, `Else`), the operand is set to a placeholder (`Operand::LabelIdx(usize::MAX)`), and the necessary information for later resolution (`pc`, `relative_depth`, flags) is collected into `FixupInfo`.
        *   The parser outputs this `Vec<ProcessedInstr>` and `Vec<FixupInfo`.
    *   **Phase 2 (Runtime Preprocessing: `stack.rs::preprocess_instructions`):**
        *   Takes the `ProcessedInstr` list generated in Phase 1 as input.
        *   Scans the instruction list to record the positions of `End` and `Else` instructions in `HashMap`s (`block_end_map`, `if_else_map`). This is needed for calculating branch targets in Phases 3/4.
    *   **Phase 3 (Runtime Preprocessing: `stack.rs::preprocess_instructions`):**
        *   Uses the `FixupInfo` list from Phase 1 and the maps created in Phase 2.
        *   Processes `FixupInfo` entries corresponding to `Br`, `BrIf`, `If`, and `Else`.
        *   For each fixup target instruction (`fixup_pc`), it **reconstructs the control stack** by scanning the instruction list from the beginning up to `fixup_pc`.
        *   Calculates the absolute target PC using the `relative_depth` and map information.
        *   Updates (patches) the `operand` of the corresponding `ProcessedInstr` with the calculated absolute PC.
    *   **Phase 4 (Runtime Preprocessing: `stack.rs::preprocess_instructions`):**
        *   Processes `FixupInfo` entries corresponding to `BrTable` instructions.
        *   Similar to Phase 3, uses control stack reconstruction and map information to calculate the absolute target PCs for each target (including the default).
        *   Updates the `operand` of the `BrTable` `ProcessedInstr` with the resolved targets and default PC (`Operand::BrTable { ... }`).
*   **Reason for Control Stack Reconstruction (Phases 3/4):** Wasm branches (`relative_depth`) depend on the control flow nesting structure at the point where the branch instruction occurs.
Therefore, for each fixup, it's necessary to scan the instruction list from the start up to the `fixup_pc` to reproduce the control stack (`current_control_stack_passX`) at that specific point.
This allows the correct target block to be identified using `relative_depth`.
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
*   **Phase 3: Br, BrIf, If, Else Fixup (Details)**
    *   Within `preprocess_instructions`, processes the non-`BrTable` branch information from the `fixups` vector generated in Phase 1.
    *   For each Fixup info `(fixup_pc, relative_depth, is_if_false_jump, is_else_jump)`:
        1.  **Control Stack Reconstruction:** Scan `processed` (from Phase 1) up to `fixup_pc` to reproduce the `Block`, `Loop`, `If` nesting state onto a stack (`current_control_stack_pass3`).
        2.  **Identify Target Block:** Use `relative_depth` to find the target block's start PC (`target_start_pc`) and type (`is_loop`) from the reconstructed stack.
        3.  **Calculate Absolute Target PC:** If `is_loop`, the target is `target_start_pc`. If `Block`/`If`, use the `block_end_map` (from Phase 2) to find the PC after the corresponding `End` (`target_ip`).
        4.  **Update Operand:** Write the calculated `target_ip` into `processed[fixup_pc]`'s `operand` (`Operand::LabelIdx`). For `If` (`is_if_false_jump=true`) or `Else` (`is_else_jump=true`), consult `if_else_map` to calculate the correct target (start of `Else` + 1 or end of `End` + 1).

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

*   **Phase 4: BrTable Fixup (Details)**
    *   Within `preprocess_instructions`, processes `BrTable` instructions (`handler_index == HANDLER_IDX_BR_TABLE`) whose operands are still in the initial state (`Operand::None`).
    *   **Reason:** Processed after other branches (Phase 3) because `BrTable` has multiple targets.
    *   For each `BrTable` instruction (`pc`):
        1.  **Identify Related Fixups:** Find all entries in the `fixups` list (from Phase 1) associated with this `pc`.
        2.  **Resolve Each Target:** For each associated Fixup info (`relative_depth`), perform control stack reconstruction up to that point (`current_control_stack_pass4`) and use map information (`block_end_map`) similar to Phase 3 to calculate the absolute target PC. (The last fixup in the list corresponds to the default branch).
        3.  **Update Operand:** Write the list of all resolved target PCs and the default PC into `processed[pc]`'s `operand` as `Operand::BrTable { targets: [...], default: ... }`.

This multi-phase preprocessing (Phase 1 in parser + Phases 2-4 in `stack.rs`) ensures that the final `ProcessedInstr` list has all branch targets resolved to absolute PCs. Consequently, the execution loop (`run_dtc_loop`) can simply use the absolute PC found in the `operand` for any branch instruction.

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
