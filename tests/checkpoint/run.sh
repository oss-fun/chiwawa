#!/bin/sh
# Checks that a run interrupted by a checkpoint and resumed from it in a new process prints consecutive numbers across the two runs.
#
# `count_up.wat` prints 1, 2, 3, ..., COUNT one per line.
# The script runs it with `--cr`, creates the trigger file partway through, and restores the checkpoint in a second process.
# The numbers printed before and after the checkpoint, in order, must be 1 to COUNT with no gap and no repeat.
# The restore runs with `--hook after-restore`, and the hook must print MARKER one time, before the first restored number.
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
SKIP_INPUT_WAIT=${SKIP_INPUT_WAIT:-0}
# The last number `count_up.wat` prints.
COUNT=2000
# The line that the after-restore hook of `count_up.wat` prints.
MARKER='after-restore hook'

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
wat2wasm "$DIR/wait_input.wat" -o "$WORK/wait_input.wasm" || exit 1
wat2wasm "$DIR/file_count.wat" -o "$WORK/file_count.wasm" || exit 1

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
    guest=$1
    shift
    timeout "$TIMEOUT" $RUNTIME --dir . "$CHIWAWA" "$guest" "$@"
}

# 1. Run with --cr and create the trigger file partway through.
run count_up.wasm --cr >before.txt 2>&1 </dev/null &
sleep "$TRIGGER_AFTER"
touch checkpoint.trigger
wait $! || fail "checkpointed run exited with $?"

grep -q '^Execution stopped for checkpoint\.$' before.txt \
    || fail "the run did not stop for the checkpoint (did it finish within TRIGGER_AFTER=${TRIGGER_AFTER}s?)"
[ -f checkpoint.bin ] || fail "checkpoint.bin was not written"

# 2. Restore with the after-restore hook and run to completion.
run count_up.wasm --restore checkpoint.bin --hook after-restore >after.txt 2>&1 </dev/null || fail "restored run exited with $?"

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

[ "$(grep -cx "$MARKER" after.txt)" = 1 ] || fail "the restored run did not print the hook marker exactly one time"
marker_line=$(grep -nx "$MARKER" after.txt | cut -d: -f1)
first_number_line=$(grep -nE '^[0-9]+$' after.txt | head -n 1 | cut -d: -f1)
[ "$marker_line" -lt "$first_number_line" ] || fail "the hook marker came after the first restored number"

echo "ok   count_up: $before lines, then the checkpoint, then the hook marker, then $((COUNT - before)) more"

rm -f checkpoint.bin before.txt after.txt
run file_count.wasm --cr >before.txt 2>&1 </dev/null &
sleep "$TRIGGER_AFTER"
touch checkpoint.trigger
wait $! || fail "checkpointed run of file_count exited with $?"

grep -q '^Execution stopped for checkpoint\.$' before.txt \
    || fail "file_count did not stop for the checkpoint"
[ -f checkpoint.bin ] || fail "checkpoint.bin was not written for file_count"
before=$(wc -l <count.txt)
[ "$before" -gt 0 ] || fail "the checkpoint landed before file_count wrote its first line"
[ "$before" -lt "$COUNT" ] || fail "the checkpoint landed after file_count wrote its last line"

# `file_count.wat` has no hook export, so a restore with `--hook after-restore` must stop before the guest runs.
run file_count.wasm --restore checkpoint.bin --hook after-restore >after.txt 2>&1 </dev/null \
    && fail "the restore of file_count with --hook after-restore did not fail"
grep -q chiwawa_after_restore after.txt || fail "the hook error does not name the export chiwawa_after_restore"
[ "$(wc -l <count.txt)" -eq "$before" ] || fail "file_count resumed although it has no hook export"

echo "ok   file_count: the restore with --hook after-restore stopped, because the guest has no hook export"

run file_count.wasm --restore checkpoint.bin >after.txt 2>&1 </dev/null \
    || fail "restored run of file_count exited with $?"

seq 1 "$COUNT" | diff - count.txt >diff.txt || {
    head -n 10 diff.txt
    fail "count.txt does not hold 1 to $COUNT after the restore"
}

echo "ok   file_count: $before lines in count.txt, then the checkpoint, then $((COUNT - before)) more"

[ "$SKIP_INPUT_WAIT" = 1 ] && exit 0

rm -f checkpoint.bin before.txt after.txt
mkfifo stdin.fifo
exec 3<>stdin.fifo
run wait_input.wasm --cr >before.txt 2>&1 <stdin.fifo &
sleep "$TRIGGER_AFTER"
touch checkpoint.trigger
wait $! || fail "checkpointed run of wait_input exited with $?"
exec 3>&-

grep -q '^Execution stopped for checkpoint\.$' before.txt \
    || fail "wait_input did not stop for the checkpoint"
[ -f checkpoint.bin ] || fail "checkpoint.bin was not written for wait_input"

printf 7 | run wait_input.wasm --restore checkpoint.bin >after.txt 2>&1 \
    || fail "restored run of wait_input exited with $?"

[ "$(grep -E '^[0-9]+$' before.txt | paste -sd ' ' -)" = 1 ] \
    || fail "wait_input did not print exactly 1 before the checkpoint"
[ "$(grep -E '^[0-9]+$' after.txt | paste -sd ' ' -)" = 7 ] \
    || fail "the restored wait_input did not print exactly the byte it was sent"

echo "ok   wait_input: 1, then the checkpoint during the read, then the byte sent after the restore"
