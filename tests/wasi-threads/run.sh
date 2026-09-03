#!/bin/sh
# Runs the wasi-threads proposal's own testsuite against chiwawa.
#
# The modules are not vendored: they are fetched from the upstream repository
# at the commit pinned below, into `testsuite/` next to this script. Delete
# that directory to re-download.
#
# Each module runs as its own process: the suite reports its result through
# `proc_exit`, so the exit code is the assertion and an in-process `cargo test`
# cannot observe it. A module with a `.json` manifest must exit with the code
# it names; one without must exit 0.
#
# Environment:
#   CHIWAWA   path to chiwawa.wasm  (default: the tco-threads build)
#   RUNTIME   host runtime command  (default: wasmtime with threads enabled)
#   TIMEOUT   seconds per module    (default: 30)

set -u

# https://github.com/WebAssembly/wasi-threads/tree/main/test/testsuite
UPSTREAM=WebAssembly/wasi-threads
COMMIT=6b4e2e50a3929d9ebc3b1a54e36ab6f0c0ebc677

MODULES="wasi_threads_exit_main_block wasi_threads_exit_main_busy
wasi_threads_exit_main_wasi wasi_threads_exit_main_wasi_read
wasi_threads_exit_nonmain_block wasi_threads_exit_nonmain_busy
wasi_threads_exit_nonmain_wasi wasi_threads_exit_nonmain_wasi_read
wasi_threads_noop wasi_threads_return_main_block wasi_threads_return_main_busy
wasi_threads_return_main_wasi wasi_threads_return_main_wasi_read
wasi_threads_spawn"

DIR=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$DIR/../.." && pwd)
SUITE="$DIR/testsuite"
CHIWAWA=${CHIWAWA:-$ROOT/target/tco-threads/wasm32-wasip1-threads/release/chiwawa.wasm}
TIMEOUT=${TIMEOUT:-30}
RUNTIME=${RUNTIME:-"wasmtime run -S threads=y"}

for tool in curl wat2wasm wasmtime; do
    command -v "$tool" >/dev/null 2>&1 || {
        echo "error: $tool not found in PATH" >&2
        exit 1
    }
done

[ -f "$CHIWAWA" ] || {
    echo "error: $CHIWAWA not found" >&2
    echo "       build it with: cargo build-tco-threads" >&2
    exit 1
}

# Fetch on first run, or whenever the pinned commit changes.
if [ ! -f "$SUITE/.commit" ] || [ "$(cat "$SUITE/.commit")" != "$COMMIT" ]; then
    echo "fetching testsuite from $UPSTREAM at $COMMIT"
    rm -rf "$SUITE"
    mkdir -p "$SUITE"
    base="https://raw.githubusercontent.com/$UPSTREAM/$COMMIT/test/testsuite"
    for name in $MODULES; do
        curl -sfL "$base/$name.wat" -o "$SUITE/$name.wat" || {
            echo "error: failed to fetch $name.wat" >&2
            rm -rf "$SUITE"
            exit 1
        }
        # A manifest is optional: without one the module must exit 0.
        curl -sfL "$base/$name.json" -o "$SUITE/$name.json" || true
    done
    echo "$COMMIT" >"$SUITE/.commit"
fi

WORK=$(mktemp -d)

mkfifo "$WORK/stdin"
sleep 86400 >"$WORK/stdin" &
writer=$!
trap 'kill "$writer" 2>/dev/null; rm -rf "$WORK"' EXIT

passed=0
failed=0

for name in $MODULES; do
    expected=0
    if [ -f "$SUITE/$name.json" ]; then
        expected=$(sed -n 's/.*"exit_code"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p' "$SUITE/$name.json")
    fi

    if ! wat2wasm --enable-threads "$SUITE/$name.wat" -o "$WORK/$name.wasm" 2>"$WORK/$name.out"; then
        echo "FAIL $name (wat2wasm)"
        sed 's/^/     /' "$WORK/$name.out"
        failed=$((failed + 1))
        continue
    fi

    # shellcheck disable=SC2086
    timeout "$TIMEOUT" $RUNTIME --dir "$WORK::/t" "$CHIWAWA" "/t/$name.wasm" --threads \
        <"$WORK/stdin" >"$WORK/$name.out" 2>&1
    actual=$?

    if [ "$actual" -eq "$expected" ]; then
        echo "ok   $name"
        passed=$((passed + 1))
    else
        if [ "$actual" -eq 124 ]; then
            echo "FAIL $name (timed out after ${TIMEOUT}s, expected exit $expected)"
        else
            echo "FAIL $name (exit $actual, expected $expected)"
        fi
        grep -v '^[[:space:]]*$' "$WORK/$name.out" | head -2 | sed 's/^/     /'
        failed=$((failed + 1))
    fi
done

echo
echo "passed: $passed  failed: $failed"
[ "$failed" -eq 0 ]
