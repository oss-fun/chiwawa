#!/bin/sh
# Fetches the wasi-threads proposal's own testsuite, which `run.sh` runs.
#
# The modules are not vendored: they come from the upstream repository at the
# commit pinned below, into `testsuite/` next to this script. Delete that
# directory to re-download.

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

SUITE=$(cd "$(dirname "$0")" && pwd)/testsuite

command -v curl >/dev/null 2>&1 || {
    echo "error: curl not found in PATH" >&2
    exit 1
}

if [ -f "$SUITE/.commit" ] && [ "$(cat "$SUITE/.commit")" = "$COMMIT" ]; then
    echo "testsuite already at $COMMIT"
    exit 0
fi

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
