#!/bin/sh
# Fetches the spec test scripts chiwawa runs as `.wast`:
# proposal scripts, the core scripts whose directives link several modules,
# and the Component Model reference tests.
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

# Component Model reference tests: `test/<path>` upstream, `component/<path>` here.
# https://github.com/WebAssembly/component-model
COMPONENT_UPSTREAM=WebAssembly/component-model
COMPONENT_COMMIT=0220da84209397d9067c19cac83dae969aa85a7b
COMPONENT="binary/binary.wast
linking/unit.wast linking/link-time-virtualization.wast linking/shared-everything-dynamic-linking.wast
values/alignment.wast values/concat.wast values/numerics.wast values/post-return.wast
values/realloc.wast values/strings.wast values/transcode.wast values/variants.wast
resources/borrows.wast resources/handle-table.wast resources/multiple-resources.wast
validation/abi.wast validation/annotated-names.wast validation/attributes.wast
validation/core-modules.wast validation/defined-types.wast validation/extern-names.wast
validation/external-visibility.wast validation/indicies.wast validation/instantiation.wast
validation/kebab.wast validation/max-value-size.wast validation/outer-alias.wast
validation/resources.wast"

SUITE=$(cd "$(dirname "$0")" && pwd)
STAMP="$PROPOSALS_COMMIT $CORE_COMMIT $COMPONENT_COMMIT"

command -v curl >/dev/null 2>&1 || {
    echo "error: curl not found in PATH" >&2
    exit 1
}

if [ -f "$SUITE/.commit" ] && [ "$(cat "$SUITE/.commit")" = "$STAMP" ]; then
    echo "spec tests already at $STAMP"
    exit 0
fi

# fetch <repository> <commit> <upstream path> <local path>
fetch() {
    mkdir -p "$SUITE/$(dirname "$4")"
    curl -sfL "https://raw.githubusercontent.com/$1/$2/$3" -o "$SUITE/$4" || {
        echo "error: failed to fetch $3" >&2
        exit 1
    }
}

echo "fetching spec tests"
for f in $PROPOSALS; do
    fetch "$UPSTREAM" "$PROPOSALS_COMMIT" "proposals/$f" "$f"
done
for f in $CORE; do
    fetch "$UPSTREAM" "$CORE_COMMIT" "$f" "core/$f"
done
for f in $COMPONENT; do
    fetch "$COMPONENT_UPSTREAM" "$COMPONENT_COMMIT" "test/$f" "component/$f"
done
echo "$STAMP" >"$SUITE/.commit"
