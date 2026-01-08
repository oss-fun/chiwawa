#!/bin/bash

set -e

echo "=== chiwawa WASI Write Benchmark ==="

# Build chiwawa if not already built
if [ ! -f "../target/wasm32-wasip1/release/chiwawa.wasm" ]; then
    echo "Building chiwawa..."
    cd ..
    ~/.cargo/bin/cargo build --target wasm32-wasip1 --release
    cd benchmarks
fi

# Build benchmark program
echo "Building write benchmark program..."
~/.cargo/bin/cargo build --target wasm32-wasip1 --release --bin write_bench

CHIWAWA_WASM="../target/wasm32-wasip1/release/chiwawa.wasm"
WRITE_WASM="target/wasm32-wasip1/release/write_bench.wasm"

# Check if files exist
if [ ! -f "$CHIWAWA_WASM" ]; then
    echo "Error: chiwawa.wasm not found at $CHIWAWA_WASM"
    exit 1
fi

if [ ! -f "$WRITE_WASM" ]; then
    echo "Error: write_bench.wasm not found at $WRITE_WASM"
    exit 1
fi

echo ""
echo "=== Running write benchmarks ==="

function run_write_benchmark() {
    local desc="$1"
    local size="$2"
    local iterations="$3"

    echo "=== $desc - Write Only ==="

    # Calculate total bytes
    local total_bytes=$((size * iterations))
    local total_mb=$(echo "scale=2; $total_bytes / 1024 / 1024" | bc)

    echo ""
    echo "Running with chiwawa runtime (superinstructions):"

    # Clean up any existing files
    rm -f write_test_* 2>/dev/null || true

    # Run chiwawa write benchmark with superinstructions and capture timing
    local start_time=$(date +%s.%N)
    wasmtime --dir . "$CHIWAWA_WASM" "$WRITE_WASM" --superinstructions --app-args "--size $size --iterations $iterations" 2>/dev/null
    local end_time=$(date +%s.%N)
    rm -f bench_test_file.dat_* 2>/dev/null || true

    # Calculate chiwawa superinstructions execution time and throughput
    local chiwawa_super_time=$(echo "$end_time - $start_time" | bc)
    local chiwawa_super_throughput=$(echo "scale=2; $total_mb / $chiwawa_super_time" | bc)

    echo "chiwawa (superinstructions) write time: ${chiwawa_super_time}s"
    echo "chiwawa (superinstructions) write throughput: ${chiwawa_super_throughput} MB/s"

    echo ""
    echo "Running with chiwawa runtime (superinstructions disabled):"

    # Run chiwawa write benchmark without superinstructions and capture timing
    local start_time=$(date +%s.%N)
    wasmtime --dir . "$CHIWAWA_WASM" "$WRITE_WASM" --app-args "--size $size --iterations $iterations" 2>/dev/null
    local end_time=$(date +%s.%N)
    rm -f bench_test_file.dat_* 2>/dev/null || true

    # Calculate chiwawa baseline execution time and throughput
    local chiwawa_time=$(echo "$end_time - $start_time" | bc)
    local chiwawa_throughput=$(echo "scale=2; $total_mb / $chiwawa_time" | bc)

    echo "chiwawa (baseline) write time: ${chiwawa_time}s"
    echo "chiwawa (baseline) write throughput: ${chiwawa_throughput} MB/s"

    echo ""
    echo "Running with wasmtime (native):"

    # Run wasmtime write benchmark and capture timing
    local start_time=$(date +%s.%N)
    wasmtime --dir . "$WRITE_WASM" --size $size --iterations $iterations >/dev/null 2>&1
    local end_time=$(date +%s.%N)
    rm -f bench_test_file.dat_* 2>/dev/null || true

    # Calculate wasmtime execution time and throughput
    local wasmtime_time=$(echo "$end_time - $start_time" | bc)
    local wasmtime_throughput=$(echo "scale=2; $total_mb / $wasmtime_time" | bc)

    echo "wasmtime write time: ${wasmtime_time}s"
    echo "wasmtime write throughput: ${wasmtime_throughput} MB/s"

    # Calculate performance ratios
    local super_vs_wasmtime_speed=$(echo "scale=2; $wasmtime_throughput / $chiwawa_super_throughput" | bc)
    local baseline_vs_wasmtime_speed=$(echo "scale=2; $wasmtime_throughput / $chiwawa_throughput" | bc)
    local super_vs_baseline_speed=$(echo "scale=2; $chiwawa_super_throughput / $chiwawa_throughput" | bc)

    echo "chiwawa (superinstructions) vs wasmtime:"
    echo "  wasmtime is ${super_vs_wasmtime_speed}x faster in throughput"
    echo ""
    echo "chiwawa (baseline) vs wasmtime:"
    echo "  wasmtime is ${baseline_vs_wasmtime_speed}x faster in throughput"
    echo ""
    echo "chiwawa superinstructions vs baseline:"
    echo "  superinstructions is ${super_vs_baseline_speed}x faster than baseline"
    echo ""
}

# Small file benchmark (1KB, 1000 iterations)
run_write_benchmark "Small files benchmark (1KB x 100)" 1024 1000

# Medium file benchmark (1MB, 1000 iterations)
run_write_benchmark "Medium files benchmark (1MB x 100)" 1048576 1000

# Large file benchmark (10MB, 1000 iterations)
run_write_benchmark "Large files benchmark (10MB x 100)" 10485760 1000

echo ""
echo "=== Write benchmark completed ==="
