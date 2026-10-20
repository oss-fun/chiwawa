#!/bin/sh
# Fetches the spec test scripts chiwawa runs as `.wast`:
# proposal scripts, and the core scripts whose directives link several modules. 
# The other core tests are `tests/*.rs`.
#
# The files are not vendored: they come from the upstream testsuite at the
# commits pinned below, into this directory. 
# Delete them to re-download. Run
# this once, then `cargo test` as usual.

set -u

# https://github.com/WebAssembly/testsuite
UPSTREAM=WebAssembly/testsuite

# Proposal scripts: `proposals/<path>` upstream, `<path>` here.
PROPOSALS_COMMIT=6051b2174a851150a2fc4087cf40816f2bf81c3a
PROPOSALS="threads/atomic.wast"

# Core scripts: `<path>` upstream, `core/<path>` here. 
# Pinned to the last commit before the GC proposal's typed references were merged into them.
CORE_COMMIT=6aacfd8929504d8e02a5144a14d184196ede6790
CORE="linking.wast"

SUITE=$(cd "$(dirname "$0")" && pwd)
STAMP="$PROPOSALS_COMMIT $CORE_COMMIT"

command -v curl >/dev/null 2>&1 || {
    echo "error: curl not found in PATH" >&2
    exit 1
}

if [ -f "$SUITE/.commit" ] && [ "$(cat "$SUITE/.commit")" = "$STAMP" ]; then
    echo "spec tests already at $STAMP"
    exit 0
fi

# fetch <commit> <upstream path> <local path>
fetch() {
    mkdir -p "$SUITE/$(dirname "$3")"
    curl -sfL "https://raw.githubusercontent.com/$UPSTREAM/$1/$2" -o "$SUITE/$3" || {
        echo "error: failed to fetch $2" >&2
        exit 1
    }
}

echo "fetching spec tests from $UPSTREAM"
for f in $PROPOSALS; do
    fetch "$PROPOSALS_COMMIT" "proposals/$f" "$f"
done
for f in $CORE; do
    fetch "$CORE_COMMIT" "$f" "core/$f"
done
echo "$STAMP" >"$SUITE/.commit"
