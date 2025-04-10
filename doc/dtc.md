# Direct Threaded Code (DTC) in stackopt.rs
Direct Threaded Code (DTC) は、インタプリタ（特に仮想マシンやバイトコードインタプリタ）を高速化するためのテクニック。

**目的:** Wasm 命令を実行する際のディスパッチ（どの命令を実行するか決定する処理）のオーバーヘッドを削減する。

**通常のインタプリタとの比較:** 単純なインタプリタでは、命令コード (`opcode`) を読み込み、`match` や `switch` 文で対応する処理を探す。
命令の種類が多い場合、命令ディスパッチと分岐処理が実行速度のボトルネックになる。

**DTC のアプローチ:** DTC では、事前に各命令に対応する処理（ハンドラ）のアドレスをテーブル (`HANDLER_TABLE`) に格納する。
実行時には、命令列からハンドラテーブルのインデックスを読み取り、テーブルを使ってハンドラ関数のアドレスを取得し、そのアドレスの関数を呼び出す。
これにより、`match` 文のような大きな分岐処理を実行ループから排除できる。

## `stackopt.rs` における DTC 実装
### 1. 主要なデータ構造

*   **`Operand` enum:**
    -  **役割:** 前処理済みの命令オペランド（引数）を表す。

    実行時に命令の引数をバイト列からパースするコストをなくす。
    特に、分岐命令のジャンプ先 (`LabelIdx(usize)`) は、前処理によって計算された**絶対的な命令ポインタ (PC)** を保持するため、実行時のジャンプ先計算が不要になる。
    `BrTable` も同様に解決済みのターゲットリストを持つ。
    ```rust
    #[derive(Clone, Debug, PartialEq)]
    pub enum Operand {
        None,
        I32(i32), // 即値
        // ...
        LocalIdx(LocalIdx), // ローカル変数インデックス
        GlobalIdx(GlobalIdx), // グローバル変数インデックス
        LabelIdx(usize), // ★解決済みの分岐先絶対PC
        MemArg(Memarg), // メモリアクセス情報
        BrTable { targets: Vec<usize>, default: usize }, // ★解決済みのBrTableターゲット
    }
    ```

*   **`ProcessedInstr` struct:**
    *   **役割:** 前処理済みの一つの命令を表す。
    
    実行ループが必要とする情報、すなわち「次に実行すべきハンドラはどれか (`handler_index`)」と「そのハンドラが必要とする引数は何か (`operand`)」を持つ。
    実行ループは単純なインデックス参照と関数呼び出しのみをする。
    ```rust
    #[derive(Clone, Debug)]
    pub struct ProcessedInstr {
        handler_index: usize, // ハンドラテーブルへのインデックス
        operand: Operand,     // 解決済みのオペランド
    }
    ```

*   **`ExecutionContext` struct:**
    *   **役割:** 各ハンドラ関数に渡される実行コンテキスト。

    ハンドラ関数が現在の実行状態（値スタック、ローカル変数、現在の命令ポインタ `ip` など）にアクセスし、それを変更できるようにするため。
    `&mut` (可変参照) で渡すことで、ハンドラによる状態変更を可能に。
    `ip` を含むのは、多くの命令が単純に次の命令に進む (`Ok(ctx.ip + 1)`) ため、その計算を容易にする意図ため。
    ```rust
    pub struct ExecutionContext<'a> {
        pub frame: &'a mut crate::execution::stackopt::Frame, // ローカル変数等へのアクセス
        pub value_stack: &'a mut Vec<Val>,                 // 値スタック操作
        pub ip: usize,                                     // 現在の命令ポインタ
    }
    ```

*   **`HandlerFn` type:**
    *   **役割:** 命令ハンドラ関数の統一されたシグネチャ（型）を定義

    `HANDLER_TABLE` に異なるハンドラ関数へのポインタを格納するため。
    戻り値 `Result<usize, RuntimeError>` は、成功時には次の命令ポインタ (`usize`) を、失敗時にはエラーを返す。
    `usize::MAX` や `usize::MAX - 1` といった特別な `usize` を使うのは、戻り値の型を変えずに Call や Return といった特別な制御フロー遷移を効率的に実行ループへ通知するため。

    ```rust
    type HandlerFn = fn(&mut ExecutionContext, Operand) -> Result<usize, RuntimeError>;
    ```

### 2. 前処理と分岐解決 (Fixup)

*   **目的:** 実行時のコスト（特に分岐解決）を可能な限り削減するため、Wasm 命令列を DTC 実行に適した形式 (`Vec<ProcessedInstr>`) へ変換する。この変換は複数のフェーズで行われる。
*   **フェーズ:**
    *   **Phase 1 (パーサー内: `parser.rs`):**
        *   Wasm バイトコードの関数ボディをパースする際に、各オペコードを対応する `handler_index` と初期 `Operand` を持つ `ProcessedInstr` に直接変換する。
        *   分岐命令 (`Br`, `BrIf`, `If`, `Else`) については、オペランドをプレースホルダー (`Operand::LabelIdx(usize::MAX)`) とし、解決に必要な情報 (`pc`, `relative_depth`, フラグ) を `FixupInfo` として収集する。
        *   パーサーはこの `Vec<ProcessedInstr>` と `Vec<FixupInfo>` を出力する。
    *   **Phase 2 (実行時前処理: `stack.rs::preprocess_instructions`):**
        *   Phase 1 で生成された `ProcessedInstr` 列を入力として受け取る。
        *   命令列を走査し、`End` や `Else` の位置情報を `HashMap` (`block_end_map`, `if_else_map`) に記録する。これは Phase 3/4 で分岐先を計算するために必要。
    *   **Phase 3 (実行時前処理: `stack.rs::preprocess_instructions`):**
        *   Phase 1 で生成された `FixupInfo` リストと Phase 2 で作成されたマップ情報を使用する。
        *   `Br`, `BrIf`, `If`, `Else` に対応する `FixupInfo` を処理する。
        *   各 Fixup 対象命令 (`fixup_pc`) について、命令列の先頭から `fixup_pc` までをスキャンして**制御スタックを再構築**し、`relative_depth` とマップ情報を用いて絶対的な分岐先 PC を計算する。
        *   計算した絶対 PC を `ProcessedInstr` の `operand` に書き込む (パッチする)。
    *   **Phase 4 (実行時前処理: `stack.rs::preprocess_instructions`):**
        *   `BrTable` 命令に対応する `FixupInfo` を処理する。
        *   Phase 3 と同様に、制御スタックの再構築とマップ情報を用いて、各ターゲット（およびデフォルト）の絶対分岐先 PC を計算する。
        *   解決した全ターゲット PC のリストとデフォルト PC を `ProcessedInstr` の `operand` に `Operand::BrTable { ... }` として書き込む。
*   **制御スタック再構築の理由 (Phase 3/4):** Wasm の分岐 (`relative_depth`) は、その命令が存在する時点での制御フローのネスト構造に依存する。
そのため、各 Fixup ごとに命令列の先頭から `fixup_pc` までをスキャンし、その時点での制御スタック (`current_control_stack_passX`) を再現する必要がある。
これにより、`relative_depth` を使って正しいターゲットブロックを特定できる。
そのため、各 Fixup ごとに命令列の先頭から `fixup_pc` までをスキャンし、その時点での制御スタック (`current_control_stack_passX`) を再現。
これにより、`relative_depth` を使って正しいターゲットブロックを特定できる。
    ```rust
    // Phase 3 内の制御スタック再構築ループ (簡略化)
    current_control_stack_pass3.clear();
    for (pc, instr) in processed.iter().enumerate().take(fixup_pc + 1) {
        match instr.handler_index {
            HANDLER_IDX_BLOCK | HANDLER_IDX_LOOP | HANDLER_IDX_IF => { /* push */ }
            HANDLER_IDX_END => { /* pop */ }
            _ => {}
        }
    }
    // この時点で current_control_stack_pass3 は fixup_pc 時点のネスト状態を表す
    ```
*   **Phase 3: Br, BrIf, If, Else の Fixup (詳細)**
    *   `preprocess_instructions` 内で、Phase 1 で生成された `fixups` ベクタ内の `BrTable` 以外の分岐情報を処理。
    *   各 Fixup 情報 `(fixup_pc, relative_depth, is_if_false_jump, is_else_jump)` について：
        1.  **制御スタック再構築:** `processed` (Phase 1 の結果) を先頭から `fixup_pc` までスキャンし、その時点での `Block`, `Loop`, `If` のネスト状態をスタック (`current_control_stack_pass3`) に再現。
        2.  **ターゲットブロック特定:** `relative_depth` を使って、再構築したスタックからジャンプ対象ブロックの開始 PC (`target_start_pc`) と種類 (`is_loop`) を特定。
        3.  **絶対ターゲット PC 計算:** ターゲットが `Loop` なら `target_start_pc`。`Block`/`If` なら Phase 2 で作成した `block_end_map` を使って対応する `End` の次の PC を `target_ip` として計算。
        4.  **オペランド更新:** 計算した `target_ip` を `processed[fixup_pc]` の `operand` (`Operand::LabelIdx`) に書き込む。`If` (`is_if_false_jump=true`) や `Else` (`is_else_jump=true`) の場合は、`if_else_map` も参照して適切なターゲット (`Else` の開始位置 + 1 または `End` の終了位置 + 1) を計算する。

         ```
        | PC (ip) | Handler Index        | 説明                     |
        | :------ | :------------------- | :----------------------- |
        | 0       | HANDLER_IDX_BLOCK    | 外側の Block 開始        |
        | 1       | HANDLER_IDX_I32_CONST |                          |
        | 2       | HANDLER_IDX_IF       | If 開始                  |
        | 3       | HANDLER_IDX_I32_CONST | (then 節)                |
        | 4       | HANDLER_IDX_LOCAL_SET | (then 節)                |
        | 5       | HANDLER_IDX_BR       | Br 0 (fixup対象 @ pc=5) |
        | 6       | HANDLER_IDX_END      | If 終了                  |
        | 7       | HANDLER_IDX_END      | 外側の Block 終了        |

        Fixup 対象: pc = 5, relative_depth = 0 (元の命令: Br 0)
        ----------------------------------------------------------
        1. 制御スタック再構築 (pc = 0 から 5 までスキャン):
        pc = 0 (BLOCK): push (0, false) -> Stack: [(0, false)]
        pc = 1 (CONST): no change     -> Stack: [(0, false)]
        pc = 2 (IF):    push (2, false) -> Stack: [(0, false), (2, false)]
        pc = 3 (CONST): no change     -> Stack: [(0, false), (2, false)]
        pc = 4 (SET):   no change     -> Stack: [(0, false), (2, false)]
        pc = 5 (BR):    スキャン終了
        ==> 再構築されたスタック: [(0, false), (2, false)]

        2. ターゲットブロック特定:
        - relative_depth = 0 なので、スタックのトップ要素を取得。
        - target_block = (pc=2, is_loop=false)
        - target_start_pc = 2
        - is_loop = false

        3. 絶対ターゲット PC 計算:
        - is_loop が false なので、block_end_map を使用。
        - block_end_map には、Phase 2 で「PC=2 で始まるブロックは PC=6 の End の次 (PC=7) で終わる」という情報が記録されていると仮定 (block_end_map[2] == 7)。
        - target_ip = block_end_map[&target_start_pc] = block_end_map[&2] = 7

        4. オペランド更新:
        - processed[fixup_pc] (つまり processed[5]) の operand を更新。
        - 元の命令は Br なので、計算した target_ip を設定。
        - processed[5].operand = Operand::LabelIdx(7)
        ```

*   **Phase 4: BrTable の Fixup (詳細)**
    *   `preprocess_instructions` 内で、`BrTable` 命令 (`handler_index == HANDLER_IDX_BR_TABLE`) で、かつオペランドがまだ初期状態 (`Operand::None`) のものを処理。
    *   **理由:** `BrTable` は複数のターゲットを持つため、他の分岐が解決された後 (Phase 3 完了後) に処理する。
    *   各 `BrTable` 命令 (`pc`) について：
        1.  **関連 Fixup 特定:** Phase 1 で生成された `fixups` リストから、この `pc` に関連付けられたエントリを全て見つける。
        2.  **各ターゲット解決:** 見つけた各 Fixup 情報 (`relative_depth`) について、Phase 3 と同様に、その時点までの制御スタックを再構築 (`current_control_stack_pass4`) し、マップ情報 (`block_end_map`) を参照して絶対宛先 PC を計算する（リストの最後の Fixup はデフォルト分岐に対応）。
        3.  **オペランド更新:** 解決した全ターゲット PC のリストとデフォルト PC を `processed[pc]` の `operand` に `Operand::BrTable { targets: [...], default: ... }` として書き込む。

この複数フェーズによる前処理（パーサーでの Phase 1 + `stack.rs` での Phase 2-4）により、最終的に得られる `ProcessedInstr` 列では全ての分岐先が絶対 PC として解決済みとなり、実行ループ (`run_dtc_loop`) は分岐命令に対して単純に `operand` 内の絶対 PC を返すだけで良くなる。

### 3. ハンドラテーブル (`HANDLER_TABLE`)

*   **役割:** 命令コード（のインデックス）から対応するハンドラ関数へのマッピングを提供。
*   **理由:** `handler_index` を使った O(1) の高速なルックアップを可能にする。
`lazy_static!` を使うのは、関数ポインタを含む静的ベクターを安全に初期化するため。
`const` 文脈では関数ポインタの直接代入が制限される。

    ```rust
    lazy_static! {
        static ref HANDLER_TABLE: Vec<HandlerFn> = {
            let mut table: Vec<HandlerFn> = vec![handle_unimplemented; MAX_HANDLER_INDEX];
            // ... (各命令のハンドラを代入) ...
            table[HANDLER_IDX_I32_ADD] = handle_i32_add;
            table[HANDLER_IDX_LOCAL_GET] = handle_local_get;
            table[HANDLER_IDX_BR] = handle_br;
            table[HANDLER_IDX_CALL] = handle_call;
            // ... (他の実装済みハンドラ) ...
            table
        };
    }
    ```

### 4. 命令ハンドラ (`handle_*` )

*   **役割:** 個々の Wasm 命令のセマンティクス（スタック操作、計算、メモリ/テーブル/グローバルアクセスなど）を実装する。
*   **戻り値:**
    *   `Ok(ctx.ip + 1)`: 最も一般的なケース。単純に次の命令に進む。
    *   `Ok(target_ip)`: 分岐命令用。前処理で計算済みの絶対 PC を返すことで、ループ側での計算が不要。
    *   `Ok(usize::MAX - 1)` / `Ok(usize::MAX)`: Call/Return 用の値。実行ループに特別なアクション（フレーム操作）が必要であることを伝える。

    ```rust
    // 例: i32.add (マクロ使用)
    fn handle_i32_add(ctx: &mut ExecutionContext, _operand: Operand) -> Result<usize, RuntimeError> {
        // binop_wrapping! マクロで定型的なスタック操作と計算を隠蔽
        binop_wrapping!(ctx, I32, wrapping_add) // 結果: Ok(ctx.ip + 1)
    }

    // 例: br (値の受け渡しは TODO)
    fn handle_br(_ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
        if let Operand::LabelIdx(target_ip) = operand { // ★解決済み PC を利用
            // TODO: Handle value transfer
            Ok(target_ip) // ★次の IP としてターゲット PC を返す
        } else { /* エラー */ }
    }

    // 例: call
    fn handle_call(_ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
        if let Operand::FuncIdx(_) = operand {
             Ok(usize::MAX - 1) // Call シグナルを返す
        } else {
            Err(RuntimeError::InvalidOperand)
        }
    }

    // 例: i32.load
    fn handle_i32_load(ctx: &mut ExecutionContext, operand: Operand) -> Result<usize, RuntimeError> {
        if let Operand::MemArg(arg) = operand {
            let ptr = ctx.value_stack.pop()?.to_i32();
            let mem_addr = &ctx.frame.module.upgrade()?.mem_addrs[0]; // memidx 0 を仮定
            let val = mem_addr.load::<i32>(&arg, ptr)?;
            ctx.value_stack.push(Val::Num(Num::I32(val)));
            Ok(ctx.ip + 1)
        } else { /* エラー */ }
    }
    ```

### 5. 実行ループ (`FrameStack::run_dtc_loop` )

*   **役割:** 現在の関数フレーム内で `ProcessedInstr` 列を高速。
*   `match` によるディスパッチを排除し、テーブルルックアップ (`HANDLER_TABLE.get(...)`) と関数呼び出し (`handler_fn(...)`) の単純な繰り返しにすることで、インタプリタの主要なオーバーヘッドを削減。Call/Return のようなフレームを跨ぐ操作は、 `exec_instr` に委譲。

### 6. フレーム管理 (`Stacks::exec_instr` )

*   **役割:** 関数呼び出し全体の制御と、関数フレーム (`FrameStack`) の管理。
*   `run_dtc_loop` は単一フレーム内の実行に特化しているため、フレーム作成（`Invoke` 時、`preprocess_instructions` 呼び出しを含む）や破棄（`Return` 時）、ホスト関数呼び出しといった、フレームを跨ぐ処理をこのメソッドが担当。