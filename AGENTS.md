# AGENTS.md

For what Chiwawa is, see `README.md` and `doc/architecture.md`.

## Communication

- Use English for code, comments, and documentation.
- Respond in English. The user may write in Japanese, English, or both.
- Correct the user's English when an error affects clarity, or when asked.

## Where things are

Read the relevant document before changing a subsystem. Do not copy its contents into this file.

- `README.md`: build aliases and how to run each mode. For CLI options, run `chiwawa.wasm --help` on a host runtime.
- `doc/architecture.md`: overview, execution flow, module structure.
- `doc/interpreter.md`, `doc/tco.md`, `doc/registers.md`, `doc/folding.md`: the threaded-code interpreter and the two dispatchers.
- `doc/migration.md`: checkpoint/restore and what a checkpoint captures.
- `doc/component.md`, `doc/sockets.md`: Component Model and socket extensions.
- `tests/README.md`: test sources.

If code and a document disagree, the code is right. Fix the document in the same change.

## Commands

Use `~/.cargo/bin/cargo`.
Every build and test configuration is a cargo alias in `.cargo/config.toml`, and each alias loads its own `.cargo/config-*.toml` (target dir, rustflags, test runner).
Plain `cargo test --target wasm32-wasip1` loads none of them. Do not use it.

CI (`.github/workflows/test.yaml`) runs:

```bash
cargo build-tco
./tests/proposals/fetch.sh
cargo test-wasmtime-tco                 # one test: cargo test-wasmtime-tco <test-name>
./tests/component-cli/run.sh            # run a component from the CLI, compare the printed results
./tests/wasi-threads/fetch.sh
cargo build-tco-threads && ./tests/wasi-threads/run.sh
./tests/checkpoint/run.sh               # checkpoint mid-run, restore, compare output
cargo fmt --all --check
```

CI covers only the tco + threads + wasmtime combination.
Run the following yourself when a change can affect them, because CI will not catch a break there:

```bash
cargo test-wasmtime-legacy              # loop dispatcher, no threads (Rc path)
cargo build-legacy --features stats
cargo build-legacy --features trace
cargo build-tco --features call_graph
cargo build-tco --features socket-wamr
cargo build-tco --features socket-wasmedge
```

## Definition of done

Do not report a change as done without evidence. In the report, state:

1. Which commands above you ran, and their results (pass/fail counts for test runs).
2. For changes under `src/execution/`: results of both `cargo test-wasmtime-tco` and `cargo test-wasmtime-legacy`.
3. For changes to anything a checkpoint captures (see `doc/migration.md`): the result of `tests/checkpoint/run.sh` on both builds, see below.
4. That `cargo fmt` has been run.

If you could not run something, say so. Do not claim it works.

### Checkpoint/restore check

`tests/checkpoint/run.sh` runs `count_up.wat`, which prints 1, 2, 3, ... one per line, with `--cr` and a trigger file after one second, then again with `--restore`.
The numbers printed before and after the checkpoint must be consecutive from 1 to the last one.
CI runs it on the tco-threads build only, so run the polling-based trigger yourself:

```bash
cargo build-tco-threads && ./tests/checkpoint/run.sh
cargo build-legacy && CHIWAWA=target/legacy/wasm32-wasip1/release/chiwawa.wasm RUNTIME="wasmtime run" SKIP_INPUT_WAIT=1 ./tests/checkpoint/run.sh
```

The script prints how many lines came before the checkpoint; it fails if that is zero or all of them.
It then runs `wait_input.wat`, which is checkpointed during a read from stdin.
That guest receives its input only after the restore.
`SKIP_INPUT_WAIT=1` leaves that case out, because the polling-based build cannot checkpoint during a host call.
The test covers one thread; a multi-thread checkpoint has no automated test.

## Invariants

These are not obvious from file names. Keep them when editing.

- **Two dispatchers.** `src/execution/dispatch.rs` selects `dispatch_loop.rs` or `dispatch_tco.rs` by the `tco` feature. Every handler in `handlers.rs` runs under both; the `advance!` macro is where they differ. All handlers share one signature (`ir.rs`), which `return_call_indirect` requires under `tco`.
- **Instrumentation hooks the loop dispatcher only.** `stats` and `trace` are collected only without `tco`; under `tco` the flags are ignored with a warning, because there is no central loop to hook. `call_graph` works under both.
- **`Shared<T>`** (`src/shared.rs`) is `Arc` with the `threads` feature and `Rc` without it. Code must compile both ways.
- **WASI is passthrough only.** WASI functions delegate to wasi-libc. There is no in-runtime WASI implementation. Do not add one.
- **WASI failures are errno, never traps.** A WASI function returns `WasiResult`; `src/execution/runtime.rs` converts an `Err` into the guest's errno in one place. Do not convert or trap anywhere else.
- **Socket extensions.** Each lives in `src/wasi/socket/<host>/`: `layout.rs` (numbers and structs), `guest.rs` (calls from a guest built against that extension), `host.rs` (calls into that host, compiled only under its feature).
- **Call graph nodes follow FuncIdx order:** imports first, then locally defined functions.

## Comments

Only doc comments (`///`, `//!`) may be added, with no exceptions.

- Do not explain why a workaround is acceptable. Fix the root cause.
- A constraint the code must keep belongs in a type, a test, or a lint, with the reason in the check's failure message.
- Justify `unsafe` in a `/// # Safety` section on the function.

## Tests

- `tests/*.rs`: one file per Wasm core or WASI Preview 1 feature, named after it.
- `tests/proposals/`: spec `.wast` scripts fetched by `fetch.sh` (proposals, multi-module linking, Component Model).
- `tests/wasi-threads/`: the wasi-threads testsuite, fetched and run by its own scripts.
- `tests/wasm/`, `tests/wasi/`: binaries the tests load. `tests/testdir/`: working directory for WASI tests.

Test behavior, not implementation details: no tests of private functions or internal data layout.
Behavior means spec conformance, and also what the CLI does (checkpoint/restore results, instrumentation output).

## When a mistake recurs

If the same kind of mistake is corrected a second time, do not add a line to this file.
Propose a check that makes it impossible or visible instead, in this order of preference:

1. A type or API change so the wrong code does not compile.
2. A test.
3. A clippy `disallowed-methods` / `disallowed-types` entry with a `reason` that tells how to fix the root cause.
4. A CI step.

Add prose here only for judgment that no check can express.

## Documentation

- Keep `README.md` to what chiwawa is, how to build it, and how to run each mode. Details, caveats, and design notes go under `doc/`.
- Update documentation as the last step of a change, once the code is settled.

### Simplified Technical English

Write `README.md` and the files under `doc/` in ASD-STE100 Simplified Technical English (https://www.asd-ste100.org/).

- Use a word only with the meaning and the part of speech that the STE dictionary approves.
- Names and verbs that belong to Chiwawa, Wasm, WASI, or Rust are technical names and technical verbs, and STE permits them (for example: checkpoint, dispatcher, handler, to compile).
- Use the same word for the same thing each time.
- Write a maximum of 20 words in a sentence that gives an instruction.
- Write a maximum of 25 words in a sentence that gives a description.
- Write a maximum of six sentences in a paragraph.
- Write only one instruction in each sentence, and use the imperative.
- Use the active voice.
- Use only the simple present, simple past, and simple future tenses.
- Do not use the -ing form of a verb, unless it is part of a technical name.
- Do not put more than three nouns together as one name.
- Do not omit articles or other words to make a sentence shorter.
- Use a vertical list when a sentence has three or more parallel items.

The STE dictionary is not in this repository.
If you are not sure that the dictionary approves a word, use a simpler word.
List in your report the words that you could not check.

## Specification references

- Wasm Core Spec: https://webassembly.github.io/spec/core/bikeshed/
- WASI function list: https://github.com/WebAssembly/wasi-libc/blob/main/libc-bottom-half/headers/public/wasi/api.h