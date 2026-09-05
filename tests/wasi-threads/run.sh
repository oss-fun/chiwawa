#!/bin/sh
# Runs the wasi-threads proposal's own testsuite against chiwawa.
#
# `fetch.sh` downloads the modules; this script only runs them.
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

DIR=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$DIR/../.." && pwd)
SUITE="$DIR/testsuite"
CHIWAWA=${CHIWAWA:-$ROOT/target/tco-threads/wasm32-wasip1-threads/release/chiwawa.wasm}
TIMEOUT=${TIMEOUT:-30}
RUNTIME=${RUNTIME:-"wasmtime run -S threads=y"}

for tool in wat2wasm wasmtime; do
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

[ -f "$SUITE/.commit" ] || {
    echo "error: testsuite not downloaded" >&2
    echo "       run $DIR/fetch.sh" >&2
    exit 1
}

WORK=$(mktemp -d)

mkfifo "$WORK/stdin"
sleep 86400 >"$WORK/stdin" &
writer=$!
trap 'kill "$writer" 2>/dev/null; rm -rf "$WORK"' EXIT

passed=0
failed=0

for wat in "$SUITE"/*.wat; do
    name=$(basename "$wat" .wat)
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
