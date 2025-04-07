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

### 2. 前処理 (`preprocess_instructions`) と分岐解決 (Fixup)

*   **目的:** 実行時のコスト（特に分岐解決）を可能な限り削減するため、実行前に `Vec<Instr>` を `Vec<ProcessedInstr>` へ変換する。
この「前処理コスト」は、関数が頻繁に呼び出される場合に実行時コストの削減によって十分に償却される。
*  分岐先の解決には依存関係があるため、複数回の走査が必要である。
    *   Phase 1 で命令を変換しつつ、未解決の分岐情報を `fixups` に記録する。
    *   Phase 2 で `End` や `Else` の位置情報を `HashMap` に記録する。これは Pass 3/4 で分岐先を計算するために必要。
    *   Phase 3/4 で `fixups` を処理し、Pass 2 のマップ情報と**制御スタックの再構築**を用いて絶対的な分岐先 PC を計算し、`ProcessedInstr` のオペランドを更新します。`BrTable` は複数のターゲットを持つため、他の分岐命令解決後 (Phase 4) に処理されます。
*   **制御スタック再構築の理由 (Phase 3/4):** Wasm の分岐は相対深度で行われるため、各分岐命令 (`fixup_pc`) が実行される時点での正しいネスト構造を知る必要がある。
そのため、各 Fixup ごとに命令列の先頭から `fixup_pc` までをスキャンし、その時点での制御スタック (`current_control_stack_passX`) を再現。
これにより、`relative_depth` を使って正しいターゲットブロックを特定できます。
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

### 3. ハンドラテーブル (`HANDLER_TABLE`)

*   **役割:** 命令コード（のインデックス）から対応するハンドラ関数へのマッピングを提供し。
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

### 4. 命令ハンドラ (`handle_*` 関数群)

*   **役割:** 個々の Wasm 命令のセマンティクス（スタック操作、計算、メモリ/テーブル/グローバルアクセスなど）を実装します。
*   **理由:** 各命令の処理を独立した関数に分離することで、コードのモジュール性と可読性を高めます。DTC の文脈では、これらの関数が実行ループから直接呼び出される単位となります。
*   **戻り値の設計理由:**
    *   `Ok(ctx.ip + 1)`: 最も一般的なケース。単純に次の命令に進むことを示す。
    *   `Ok(target_ip)`: 分岐命令用。前処理で計算済みの絶対 PC を返すことで、ループ側での計算を不要にする。
    *   `Ok(usize::MAX - 1)` / `Ok(usize::MAX)`: Call/Return 用のセンチネル値。実行ループに特別なアクション（フレーム操作）が必要であることを効率的に伝える。

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

### 5. 実行ループ (`FrameStack::run_dtc_loop` メソッド)

*   **役割:** DTC の心臓部。現在の関数フレーム内で `ProcessedInstr` 列を高速に実行します。
*   **理由:** `match` によるディスパッチを排除し、テーブルルックアップ (`HANDLER_TABLE.get(...)`) と関数呼び出し (`handler_fn(...)`) の単純な繰り返しにすることで、インタプリタの主要なオーバーヘッドを削減します。Call/Return のようなフレームを跨ぐ操作は、センチネル値を返すことで上位の `exec_instr` に委譲します。

    ```rust
    // run_dtc_loop のコア部分 (簡略化)
    loop {
        if ip >= processed_code.len() { break; } // コード終端
        let instruction = processed_code[ip].clone();
        let handler_fn = HANDLER_TABLE.get(instruction.handler_index)?; // ★テーブル参照
        let mut context = ExecutionContext { /* ... */ };
        let result = handler_fn(&mut context, instruction.operand.clone()); // ★ハンドラ呼び出し
        match result {
            Ok(next_ip) => {
                if next_ip == usize::MAX { /* Return シグナル */ return Ok(Ok(Some(ModuleLevelInstr::Return))); }
                else if next_ip == usize::MAX - 1 { /* Call シグナル */ /* ... */ return Ok(Ok(Some(ModuleLevelInstr::Invoke(func_addr)))); }
                ip = next_ip; // ★次の IP へ更新
            }
            Err(e) => { /* エラー */ return Ok(Err(e)); }
        }
    }
    ```

### 6. フレーム管理 (`Stacks::exec_instr` メソッド)

*   **役割:** 関数呼び出し全体の制御と、関数フレーム (`FrameStack`) の管理を行います。
*   **理由:** `run_dtc_loop` は単一フレーム内の実行に特化しているため、フレーム作成（`Invoke` 時、`preprocess_instructions` 呼び出しを含む）や破棄（`Return` 時）、ホスト関数呼び出しといった、フレームを跨ぐ処理をこのメソッドが担当します。
