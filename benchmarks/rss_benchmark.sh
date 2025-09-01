#!/bin/bash

set -e

# Check if WASM file is provided as argument
if [ $# -eq 0 ]; then
    echo "Usage: $0 <wasm-file> [program-args...]"
    echo "Example: $0 pi-leibniz.wasm"
    echo "Example: $0 sqlite-bench.wasm --num=1000"
    exit 1
fi

WASM_FILE="$1"
shift  # Remove first argument, rest are program arguments
PROGRAM_ARGS="$@"

echo "=== RSS Memory Usage Benchmark ==="
echo "Comparing chiwawa vs wasmtime memory overhead"
echo "Target WASM: $WASM_FILE"
if [ -n "$PROGRAM_ARGS" ]; then
    echo "Program arguments: $PROGRAM_ARGS"
fi
echo ""

# Check if WASM file exists
if [ ! -f "$WASM_FILE" ]; then
    echo "Error: WASM file not found: $WASM_FILE"
    exit 1
fi

# Build chiwawa if not already built
if [ ! -f "../target/wasm32-wasip1/release/chiwawa.wasm" ]; then
    echo "Building chiwawa..."
    cd ..
    ~/.cargo/bin/cargo build --target wasm32-wasip1 --release
    cd benchmarks
fi

CHIWAWA_WASM="../target/wasm32-wasip1/release/chiwawa.wasm"

# Check if chiwawa.wasm exists
if [ ! -f "$CHIWAWA_WASM" ]; then
    echo "Error: chiwawa.wasm not found at $CHIWAWA_WASM"
    exit 1
fi

echo ""
echo "=== Measuring RSS usage for chiwawa (10 runs) ==="
chiwawa_rss_values=()

for i in {1..10}; do
    echo -n "Run $i: "
    # Run chiwawa and extract RSS from time output
    if [ -n "$PROGRAM_ARGS" ]; then
        # If program arguments exist, pass them via --app-args
        rss=$(/usr/bin/time -v wasmtime --dir . "$CHIWAWA_WASM" "$WASM_FILE" --enable-superinstructions --app-args "$PROGRAM_ARGS" 2>&1 | grep "Maximum resident set size" | awk '{print $6}')
    else
        # No program arguments
        rss=$(/usr/bin/time -v wasmtime --dir . "$CHIWAWA_WASM" "$WASM_FILE" --enable-superinstructions 2>&1 | grep "Maximum resident set size" | awk '{print $6}')
    fi
    echo "$rss KB"
    chiwawa_rss_values+=($rss)
done

# Calculate chiwawa average
sum=0
for rss in "${chiwawa_rss_values[@]}"; do
    sum=$((sum + rss))
done
chiwawa_avg=$((sum / 10))

echo ""
echo "Chiwawa RSS values: ${chiwawa_rss_values[@]}"
echo "Chiwawa average RSS: $chiwawa_avg KB"

echo ""
echo "=== Measuring RSS usage for wasmtime direct (10 runs) ==="
wasmtime_rss_values=()

for i in {1..10}; do
    echo -n "Run $i: "
    # Run wasmtime directly and extract RSS
    rss=$(/usr/bin/time -v wasmtime "$WASM_FILE" $PROGRAM_ARGS 2>&1 | grep "Maximum resident set size" | awk '{print $6}')
    echo "$rss KB"
    wasmtime_rss_values+=($rss)
done

# Calculate wasmtime average
sum=0
for rss in "${wasmtime_rss_values[@]}"; do
    sum=$((sum + rss))
done
wasmtime_avg=$((sum / 10))

echo ""
echo "Wasmtime RSS values: ${wasmtime_rss_values[@]}"
echo "Wasmtime average RSS: $wasmtime_avg KB"

# Calculate overhead
overhead=$((chiwawa_avg - wasmtime_avg))
overhead_percent=$(echo "scale=2; ($overhead * 100) / $wasmtime_avg" | bc)
memory_ratio=$(echo "scale=2; $chiwawa_avg / $wasmtime_avg" | bc)

echo ""
echo "=== Memory Usage Comparison ==="
echo "Configuration: $WASM_FILE"
if [ -n "$PROGRAM_ARGS" ]; then
    echo "Arguments: $PROGRAM_ARGS"
fi
echo ""
echo "Chiwawa (with superinstructions):"
echo "  Average RSS: $chiwawa_avg KB"
echo ""
echo "Wasmtime (direct execution):"
echo "  Average RSS: $wasmtime_avg KB"
echo ""
echo "Memory Usage Ratio:"
echo "  Chiwawa uses ${memory_ratio}x more memory than wasmtime"
echo ""
echo "Memory Overhead:"
echo "  Absolute: $overhead KB"
echo "  Relative: $overhead_percent%"
echo ""

# Also show the overhead in MB for easier understanding
overhead_mb=$(echo "scale=2; $overhead / 1024" | bc)
chiwawa_mb=$(echo "scale=2; $chiwawa_avg / 1024" | bc)
wasmtime_mb=$(echo "scale=2; $wasmtime_avg / 1024" | bc)

echo "In MB:"
echo "  Chiwawa: $chiwawa_mb MB"
echo "  Wasmtime: $wasmtime_mb MB"
echo "  Overhead: $overhead_mb MB"

echo ""
echo "=== RSS benchmark completed ==="