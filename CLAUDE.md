# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## コードベース概要

chiwawaは、WebAssembly（Wasm）ランタイムをWasm上で実行するセルフホステッド型のランタイムです。ライブマイグレーション機能とインストルメンテーション機能を提供し、インタープリター、JIT、AOTなどの実行方式やアーキテクチャに中立な設計になっています。

### 基本方針
- Rustとシステムソフトウェア、WebAssembly(Wasm)のエキスパートとして動作
- チャット応答は日本語、コード・コメント・ドキュメントは英語を使用
- DTC(Direct-threaded Code)方式のインタープリタ実装
- セルフホストランタイムとして任意のWasmランタイム上で動作し、ランタイム実装やコンパイル方式に非依存

## 主要コマンド

### ビルド
```bash
# セルフホストビルド（必須：wasm32-wasip1ターゲット）
~/.cargo/bin/cargo build --target wasm32-wasip1 --release

# コンパイルエラー確認
~/.cargo/bin/cargo check --target wasm32-wasip1
```

### テスト実行
```bash
# 全テスト実行
~/.cargo/bin/cargo test

# 特定のテスト実行
~/.cargo/bin/cargo test <テスト名>
```

### Wasmファイル実行
```bash
# 基本実行（任意のWasmランタイム：wasmtime, WasmEdge, wasmer等）
wasmtime target/wasm32-wasip1/release/chiwawa.wasm test.wasm --invoke func-name --params "I64(100)"

# チェックポイント・リストア実行
touch ./checkpoint.trigger  # チェックポイントトリガー
wasmtime target/wasm32-wasip1/release/chiwawa.wasm test.wasm --invoke func-name --restore checkpoint.trigger

# 他のランタイム例：
# WasmEdge target/wasm32-wasip1/release/chiwawa.wasm test.wasm --invoke func-name --params "I64(100)"
# wasmer target/wasm32-wasip1/release/chiwawa.wasm test.wasm --invoke func-name --params "I64(100)"
```

## アーキテクチャ構造

### 主要モジュール
- `src/lib.rs`: ライブラリのエントリーポイント
- `src/main.rs`: CLIインターフェースとメイン実行ロジック、テストコード
- `src/parser.rs`: WebAssemblyバイトコード解析
- `src/structure/`: Wasmバイトコード内部保持用データ構造
  - `instructions.rs`: 命令定義
  - `module.rs`: モジュール構造
  - `types.rs`: 型定義
- `src/execution/`: DTCインタープリタ実行エンジン
  - `runtime.rs`: ランタイムコア
  - `stack.rs`: スタック管理
  - `store.rs`: ストア管理
  - `migration.rs`: マイグレーション機能
- `src/wasi/`: WASI実装（context.rs, standard.rs, types.rs等）
- `src/error.rs`: エラー定義

### 実行フロー
1. CLIでパラメータ解析（clap使用）
2. WebAssemblyモジュールのパース（wasmparser使用）
3. モジュールインスタンス作成
4. Runtime初期化（通常実行またはリストア実行）
5. 実行とチェックポイント/リストア処理

### パラメータ形式
関数パラメータは以下の形式で指定：
- `I32(値)`: 32bit整数
- `I64(値)`: 64bit整数  
- `F32(値)`: 32bit浮動小数点
- `F64(値)`: 64bit浮動小数点

### 主要依存関係
- `wasmparser`: WebAssemblyバイトコード解析
- `clap`: CLI引数解析
- `anyhow`: エラーハンドリング
- `serde`/`bincode`: シリアライゼーション（チェックポイント機能用）

## テスト構成

テストは`tests/`ディレクトリに配置されており、各WebAssembly機能別にテストファイルが分かれています：
- `call.rs`, `call_indirect.rs`: 関数呼び出しテスト
- `i32.rs`, `i64.rs`: 整数演算テスト
- `conversion.rs`: 型変換テスト
- `loop.rs`: ループ制御テスト
- `tests/wasm/`: テスト用Wasmファイル（.wasmと.watペア）

## 開発時の注意事項

### 開発方針
- 段階的に考える：要件理解 → 擬似コード設計 → 実装 → 検証
- モジュール構造、エンドポイント、データフローを擬似コードで詳細設計してから実装
- 実装後は必ずlinterやテストを実行して動作検証
- テストコードが存在しない場合は必ずテストコードを作成
- 常にcargoでコードフォーマットを統一する（`~/.cargo/bin/cargo fmt`）

### WebAssembly仕様参考
- Wasmコア仕様: https://webassembly.github.io/spec/core/bikeshed/