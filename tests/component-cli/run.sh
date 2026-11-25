#!/bin/sh
# Checks that chiwawa runs a component from the command line.
# `--params` holds component values in the text form of `wast`, and chiwawa prints each result in the same form.
#
# `values.wasm` is `values.wat` encoded with `wasm-tools parse`.
# Each case calls one export of it and compares the output of chiwawa with the expected text.
# The last case calls a core module, to check that `--params` still takes `I32(1),I32(2)`.
#
# Environment:
#   CHIWAWA  path to chiwawa.wasm    (default: the tco build)
#   RUNTIME  host runtime command    (default: wasmtime run)

set -u

DIR=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$DIR/../.." && pwd)
CHIWAWA=${CHIWAWA:-$ROOT/target/tco/wasm32-wasip1/release/chiwawa.wasm}
RUNTIME=${RUNTIME:-"wasmtime run"}

command -v wasmtime >/dev/null 2>&1 || {
    echo "error: wasmtime not found in PATH" >&2
    exit 1
}
[ -f "$CHIWAWA" ] || {
    echo "error: $CHIWAWA not found" >&2
    echo "       build it with: cargo build-tco" >&2
    exit 1
}
CHIWAWA=$(cd "$(dirname "$CHIWAWA")" && pwd)/$(basename "$CHIWAWA")

# The guest paths are relative to tests/, which `--dir .` makes visible to chiwawa.
cd "$ROOT/tests" || exit 1
status=0

# expect NAME EXPECTED FILE ARGS...: runs chiwawa and compares its output, stdout and stderr together, with EXPECTED.
expect() {
    name=$1
    expected=$2
    shift 2
    # shellcheck disable=SC2086
    actual=$($RUNTIME --dir . "$CHIWAWA" "$@" 2>&1)
    if [ "$actual" = "$expected" ]; then
        echo "ok   $name"
    else
        echo "FAIL $name"
        echo "  expected: $expected"
        echo "  actual:   $actual"
        status=1
    fi
}

V=component-cli/values.wasm

expect "list parameter" 'Result: (u32.const 6)' \
    "$V" --invoke sum --params '(list.const (u32.const 1) (u32.const 2) (u32.const 3))'

expect "record parameter, tuple result" 'Result: (tuple.const (u32.const 2) (u32.const 1))' \
    "$V" --invoke swap --params '(record.const (field "a" u32.const 1) (field "b" u32.const 2))'

expect "string with a comma, quotes and a newline" 'Result: (str.const "a,b \"q\"\n")' \
    "$V" --invoke echo --params '(str.const "a,b \"q\"\n")'

expect "one value in two arguments" 'Result: (str.const "x")' \
    "$V" --invoke echo --params '(str.const' '"x")'

expect "parameter of the wrong type" 'Execution Error: TypeMismatch' \
    "$V" --invoke sum --params '(u32.const 1)'

expect "unknown export" 'Error: component-cli/values.wasm has no exported function nope' \
    "$V" --invoke nope

expect "checkpoint refused" 'Error: --cr and --restore are not available for a component' \
    "$V" --invoke sum --cr

expect "core module parameters" 'Result: Num(I32(3))' \
    wasm/i32.wasm --invoke add --params 'I32(1),I32(2)'

exit $status
