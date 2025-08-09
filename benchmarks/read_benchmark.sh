#!/bin/bash

set -e

echo "=== chiwawa WASI Read Benchmark ==="

# Build chiwawa if not already built
if [ ! -f "../target/wasm32-wasip1/release/chiwawa.wasm" ]; then
    echo "Building chiwawa..."
    cd ..
    ~/.cargo/bin/cargo build --target wasm32-wasip1 --release
    cd benchmarks
fi

# Build benchmark program
echo "Building read benchmark program..."
~/.cargo/bin/cargo build --target wasm32-wasip1 --release --bin read_bench

CHIWAWA_WASM="../target/wasm32-wasip1/release/chiwawa.wasm"
READ_WASM="target/wasm32-wasip1/release/read_bench.wasm"
WRITE_WASM="target/wasm32-wasip1/release/write_bench.wasm"

# Check if files exist
if [ ! -f "$CHIWAWA_WASM" ]; then
    echo "Error: chiwawa.wasm not found at $CHIWAWA_WASM"
    exit 1
fi

if [ ! -f "$READ_WASM" ]; then
    echo "Error: read_bench.wasm not found at $READ_WASM"
    exit 1
fi

echo ""
echo "=== Running read benchmarks ==="

function run_read_benchmark() {
    local desc="$1"
    local size="$2" 
    local iterations="$3"
    
    echo "=== $desc - Read Only ==="
    echo "Size: $size bytes, Iterations: $iterations"
    
    # Calculate total bytes
    local total_bytes=$((size * iterations))
    local total_mb=$(echo "scale=2; $total_bytes / 1024 / 1024" | bc)
    
    echo ""
    echo "Running with chiwawa runtime (superinstructions + memoization):"
    
    # Clean environment and create test files for chiwawa superinstructions + memoization test
    rm -f bench_test_file.dat_* 2>/dev/null || true
    echo "Creating test files for chiwawa superinstructions + memoization test..."
    wasmtime --dir . "$WRITE_WASM" --size $size --iterations $iterations >/dev/null 2>&1
    
    # Run chiwawa read benchmark with superinstructions and memoization and capture timing
    local start_time=$(date +%s.%N)
    wasmtime --dir . "$CHIWAWA_WASM" "$READ_WASM" --enable-superinstructions --enable-memoization --app-args "--size $size --iterations $iterations" 2>/dev/null
    local end_time=$(date +%s.%N)
    
    # Calculate chiwawa superinstructions + memoization execution time and throughput
    local chiwawa_super_memo_time=$(echo "$end_time - $start_time" | bc)
    local chiwawa_super_memo_throughput=$(echo "scale=2; $total_mb / $chiwawa_super_memo_time" | bc)
    
    echo "chiwawa (superinstructions + memoization) read time: ${chiwawa_super_memo_time}s"
    echo "chiwawa (superinstructions + memoization) read throughput: ${chiwawa_super_memo_throughput} MB/s"
    
    # Clean environment and create test files for chiwawa superinstructions test
    rm -f bench_test_file.dat_* 2>/dev/null || true
    echo "Creating test files for chiwawa superinstructions test..."
    wasmtime --dir . "$WRITE_WASM" --size $size --iterations $iterations >/dev/null 2>&1
    
    echo ""
    echo "Running with chiwawa runtime (superinstructions only):"
    
    # Run chiwawa read benchmark with superinstructions only and capture timing
    local start_time=$(date +%s.%N)
    wasmtime --dir . "$CHIWAWA_WASM" "$READ_WASM" --enable-superinstructions --app-args "--size $size --iterations $iterations" 2>/dev/null
    local end_time=$(date +%s.%N)
    
    # Calculate chiwawa superinstructions execution time and throughput
    local chiwawa_super_time=$(echo "$end_time - $start_time" | bc)
    local chiwawa_super_throughput=$(echo "scale=2; $total_mb / $chiwawa_super_time" | bc)
    
    echo "chiwawa (superinstructions) read time: ${chiwawa_super_time}s"
    echo "chiwawa (superinstructions) read throughput: ${chiwawa_super_throughput} MB/s"
    
    # Clean environment and create test files for chiwawa baseline test
    rm -f bench_test_file.dat_* 2>/dev/null || true
    echo "Creating test files for chiwawa baseline test..."
    wasmtime --dir . "$WRITE_WASM" --size $size --iterations $iterations >/dev/null 2>&1
    
    echo ""
    echo "Running with chiwawa runtime (baseline):"
    
    # Run chiwawa read benchmark without superinstructions and memoization and capture timing
    local start_time=$(date +%s.%N)
    wasmtime --dir . "$CHIWAWA_WASM" "$READ_WASM" --app-args "--size $size --iterations $iterations" 2>/dev/null
    local end_time=$(date +%s.%N)
    
    # Calculate chiwawa baseline execution time and throughput
    local chiwawa_time=$(echo "$end_time - $start_time" | bc)
    local chiwawa_throughput=$(echo "scale=2; $total_mb / $chiwawa_time" | bc)
    
    echo "chiwawa (baseline) read time: ${chiwawa_time}s"
    echo "chiwawa (baseline) read throughput: ${chiwawa_throughput} MB/s"
    
    # Clean environment and create test files for wasmtime test
    rm -f bench_test_file.dat_* 2>/dev/null || true
    echo "Creating test files for wasmtime test..."
    wasmtime --dir . "$WRITE_WASM" --size $size --iterations $iterations >/dev/null 2>&1
    
    echo ""
    echo "Running with wasmtime (native):"
    
    # Run wasmtime read benchmark and capture timing
    local start_time=$(date +%s.%N)
    wasmtime --dir . "$READ_WASM" --size $size --iterations $iterations >/dev/null 2>&1
    local end_time=$(date +%s.%N)
    
    # Calculate wasmtime execution time and throughput
    local wasmtime_time=$(echo "$end_time - $start_time" | bc)
    local wasmtime_throughput=$(echo "scale=2; $total_mb / $wasmtime_time" | bc)
    
    echo "wasmtime read time: ${wasmtime_time}s"
    echo "wasmtime read throughput: ${wasmtime_throughput} MB/s"
    
    # Calculate performance ratios
    local super_memo_vs_wasmtime_speed=$(echo "scale=2; $wasmtime_throughput / $chiwawa_super_memo_throughput" | bc)
    local super_vs_wasmtime_speed=$(echo "scale=2; $wasmtime_throughput / $chiwawa_super_throughput" | bc)
    local baseline_vs_wasmtime_speed=$(echo "scale=2; $wasmtime_throughput / $chiwawa_throughput" | bc)
    local super_memo_vs_super_speed=$(echo "scale=2; $chiwawa_super_memo_throughput / $chiwawa_super_throughput" | bc)
    local super_vs_baseline_speed=$(echo "scale=2; $chiwawa_super_throughput / $chiwawa_throughput" | bc)
    local super_memo_vs_baseline_speed=$(echo "scale=2; $chiwawa_super_memo_throughput / $chiwawa_throughput" | bc)
    local super_memo_vs_wasmtime_time=$(echo "scale=2; $chiwawa_super_memo_time / $wasmtime_time" | bc)
    local super_vs_wasmtime_time=$(echo "scale=2; $chiwawa_super_time / $wasmtime_time" | bc)
    local baseline_vs_wasmtime_time=$(echo "scale=2; $chiwawa_time / $wasmtime_time" | bc)
    local super_memo_vs_super_time=$(echo "scale=2; $chiwawa_super_time / $chiwawa_super_memo_time" | bc)
    local super_vs_baseline_time=$(echo "scale=2; $chiwawa_time / $chiwawa_super_time" | bc)
    local super_memo_vs_baseline_time=$(echo "scale=2; $chiwawa_time / $chiwawa_super_memo_time" | bc)
    
    echo ""
    echo "Read performance comparison:"
    echo "Total data read: ${total_mb} MB"
    echo ""
    echo "chiwawa (superinstructions + memoization) vs wasmtime:"
    echo "  wasmtime is ${super_memo_vs_wasmtime_speed}x faster in throughput"
    echo "  chiwawa (superinstructions + memoization) takes ${super_memo_vs_wasmtime_time}x longer"
    echo ""
    echo "chiwawa (superinstructions) vs wasmtime:"
    echo "  wasmtime is ${super_vs_wasmtime_speed}x faster in throughput"
    echo "  chiwawa (superinstructions) takes ${super_vs_wasmtime_time}x longer"
    echo ""
    echo "chiwawa (baseline) vs wasmtime:"
    echo "  wasmtime is ${baseline_vs_wasmtime_speed}x faster in throughput"
    echo "  chiwawa (baseline) takes ${baseline_vs_wasmtime_time}x longer"
    echo ""
    echo "chiwawa memoization impact:"
    echo "  superinstructions + memoization is ${super_memo_vs_super_speed}x faster than superinstructions only"
    echo "  memoization reduces superinstructions execution time by ${super_memo_vs_super_time}x"
    echo ""
    echo "chiwawa superinstructions vs baseline:"
    echo "  superinstructions is ${super_vs_baseline_speed}x faster than baseline"
    echo "  superinstructions reduces execution time by ${super_vs_baseline_time}x"
    echo ""
    echo "chiwawa full optimization vs baseline:"
    echo "  superinstructions + memoization is ${super_memo_vs_baseline_speed}x faster than baseline"
    echo "  full optimization reduces execution time by ${super_memo_vs_baseline_time}x"
    
    # Cleanup test files
    echo "Cleaning up test files..."
    for i in $(seq 0 $((iterations-1))); do
        rm -f "bench_test_file.dat_$i" 2>/dev/null || true
    done
    echo ""
}

# Small file benchmark (1KB, 500 iterations)
run_read_benchmark "Small files benchmark (1KB x 500)" 1024 500

# Medium file benchmark (1MB, 50 iterations) 
run_read_benchmark "Medium files benchmark (1MB x 50)" 1048576 50

# Large file benchmark (10MB, 5 iterations)
run_read_benchmark "Large files benchmark (10MB x 5)" 10485760 5

echo ""
echo "=== Read benchmark completed ==="