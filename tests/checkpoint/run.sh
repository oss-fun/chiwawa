#!/bin/sh
# Checks that a run interrupted by a checkpoint and resumed from it in a new process prints consecutive numbers across the two runs.
#
# `count_up.wat` prints 1, 2, 3, ..., COUNT one per line.
# The script runs it with `--cr`, creates the trigger file partway through, and restores the checkpoint in a second process.
# The numbers printed before and after the checkpoint, in order, must be 1 to COUNT with no gap and no repeat.
#
# Environment:
#   CHIWAWA        path to chiwawa.wasm       (default: the tco-threads build)
#   RUNTIME        host runtime command       (default: wasmtime with threads enabled)
#   TIMEOUT        seconds per run            (default: 60)
#   TRIGGER_AFTER  seconds before the trigger (default: 1)

set -u

DIR=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$DIR/../.." && pwd)
CHIWAWA=${CHIWAWA:-$ROOT/target/tco-threads/wasm32-wasip1-threads/release/chiwawa.wasm}
RUNTIME=${RUNTIME:-"wasmtime run -S threads=y"}
TIMEOUT=${TIMEOUT:-60}
TRIGGER_AFTER=${TRIGGER_AFTER:-1}
# The last number `count_up.wat` prints.
COUNT=2000

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
CHIWAWA=$(cd "$(dirname "$CHIWAWA")" && pwd)/$(basename "$CHIWAWA")

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

wat2wasm "$DIR/count_up.wat" -o "$WORK/count_up.wasm" || exit 1

# The trigger file and checkpoint.bin are created in the current directory, which `--dir .` makes visible to chiwawa.
cd "$WORK" || exit 1

fail() {
    echo "FAIL: $1"
    for f in before.txt after.txt; do
        [ -f "$f" ] && { echo "--- $f (last lines)"; tail -n 5 "$f"; }
    done
    exit 1
}

# shellcheck disable=SC2086
run() {
    timeout "$TIMEOUT" $RUNTIME --dir . "$CHIWAWA" count_up.wasm "$@" </dev/null
}

# 1. Run with --cr and create the trigger file partway through.
run --cr >before.txt 2>&1 &
sleep "$TRIGGER_AFTER"
touch checkpoint.trigger
wait $! || fail "checkpointed run exited with $?"

grep -q '^Execution stopped for checkpoint\.$' before.txt \
    || fail "the run did not stop for the checkpoint (did it finish within TRIGGER_AFTER=${TRIGGER_AFTER}s?)"
[ -f checkpoint.bin ] || fail "checkpoint.bin was not written"

# 2. Restore and run to completion.
run --restore checkpoint.bin >after.txt 2>&1 || fail "restored run exited with $?"

# 3. Keep only the guest's numbers; chiwawa's own status lines are dropped.
grep -E '^[0-9]+$' before.txt >before_numbers.txt
grep -E '^[0-9]+$' after.txt >after_numbers.txt
cat before_numbers.txt after_numbers.txt >actual.txt
before=$(wc -l <before_numbers.txt)

# Show the numbers around the checkpoint so the continuity is visible.
echo "before the checkpoint: 1 .. $(tail -n 3 before_numbers.txt | paste -sd ' ' -)"
echo "after the checkpoint:  $(head -n 3 after_numbers.txt | paste -sd ' ' -) .. $(tail -n 1 after_numbers.txt)"

[ "$before" -gt 0 ] || fail "the checkpoint landed before the first line was printed"
[ "$before" -lt "$COUNT" ] || fail "the checkpoint landed after the last line was printed"

seq 1 "$COUNT" | diff - actual.txt >diff.txt || {
    head -n 10 diff.txt
    fail "the numbers before and after the checkpoint are not 1 to $COUNT"
}

echo "ok   count_up: $before lines, then the checkpoint, then $((COUNT - before)) more"
