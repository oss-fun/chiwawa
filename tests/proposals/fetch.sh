#!/bin/sh
# Fetches the proposal test scripts chiwawa tests against. Core tests are not
# here: `tests/*.rs` already cover those.
#
# The files are not vendored: they come from the upstream testsuite at the
# commit pinned below, into this directory. Delete them to re-download. Run
# this once, then `cargo test` as usual.

set -u

# https://github.com/WebAssembly/testsuite
UPSTREAM=WebAssembly/testsuite
COMMIT=6051b2174a851150a2fc4087cf40816f2bf81c3a

# Upstream path -> local path, both relative to this script.
FILES="threads/atomic.wast"

SUITE=$(cd "$(dirname "$0")" && pwd)

command -v curl >/dev/null 2>&1 || {
    echo "error: curl not found in PATH" >&2
    exit 1
}

if [ -f "$SUITE/.commit" ] && [ "$(cat "$SUITE/.commit")" = "$COMMIT" ]; then
    echo "proposal tests already at $COMMIT"
    exit 0
fi

echo "fetching proposal tests from $UPSTREAM at $COMMIT"
for f in $FILES; do
    mkdir -p "$SUITE/$(dirname "$f")"
    curl -sfL "https://raw.githubusercontent.com/$UPSTREAM/$COMMIT/proposals/$f" -o "$SUITE/$f" || {
        echo "error: failed to fetch $f" >&2
        exit 1
    }
done
echo "$COMMIT" >"$SUITE/.commit"
