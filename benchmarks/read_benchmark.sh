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
    
    # Calculate total bytes
    local total_bytes=$((size * iterations))
    local total_mb=$(echo "scale=2; $total_bytes / 1024 / 1024" | bc)
    
    echo ""
    echo "Running with chiwawa runtime (superinstructions + memoization):"
    rm -f bench_test_file.dat_* 2>/dev/null || true
    echo "Creating test files for chiwawa memoization test..."
    wasmtime --dir . "$WRITE_WASM" --size $size --iterations $iterations >/dev/null 2>&1
    echo "Test files created"
    
    # Run chiwawa read benchmark with superinstructions and memoization
    local start_time=$(date +%s.%N)
    wasmtime --dir . "$CHIWAWA_WASM" "$READ_WASM" --superinstructions --memoization --app-args "--size $size --iterations $iterations"  2>/dev/null
    local end_time=$(date +%s.%N)
    
    local chiwawa_super_memo_time=$(echo "$end_time - $start_time" | bc)
    echo "chiwawa (super + memo) execution time: ${chiwawa_super_memo_time}s"
    
    # Calculate chiwawa super + memo throughput
    local chiwawa_super_memo_throughput=$(echo "scale=2; $total_mb / $chiwawa_super_memo_time" | bc)
    echo "chiwawa (super + memo) throughput: ${chiwawa_super_memo_throughput} MB/s"
    
    echo ""
    echo "Running with chiwawa runtime (superinstructions only):"
    
    # Clean environment and create test files for chiwawa superinstructions test
    rm -f bench_test_file.dat_* 2>/dev/null || true
    echo "Creating test files for chiwawa superinstructions test..."
    wasmtime --dir . "$WRITE_WASM" --size $size --iterations $iterations >/dev/null 2>&1
    
    # Run chiwawa read benchmark with superinstructions and capture timing
    local start_time=$(date +%s.%N)
    wasmtime --dir . "$CHIWAWA_WASM" "$READ_WASM" --superinstructions --app-args "--size $size --iterations $iterations" 2>/dev/null
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
    echo "Running with chiwawa runtime (superinstructions disabled):"
    
    # Run chiwawa read benchmark without superinstructions and capture timing
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
    local super_vs_baseline_speed=$(echo "scale=2; $chiwawa_super_throughput / $chiwawa_throughput" | bc)
    local memo_vs_super_speed=$(echo "scale=2; $chiwawa_super_memo_throughput / $chiwawa_super_throughput" | bc)
    local super_memo_vs_wasmtime_time=$(echo "scale=2; $chiwawa_super_memo_time / $wasmtime_time" | bc)
    local super_vs_wasmtime_time=$(echo "scale=2; $chiwawa_super_time / $wasmtime_time" | bc)
    local baseline_vs_wasmtime_time=$(echo "scale=2; $chiwawa_time / $wasmtime_time" | bc)
    local super_vs_baseline_time=$(echo "scale=2; $chiwawa_time / $chiwawa_super_time" | bc)
    local memo_vs_super_time=$(echo "scale=2; $chiwawa_super_time / $chiwawa_super_memo_time" | bc)
    
    echo "chiwawa (super + memo) vs wasmtime:"
    echo "  wasmtime is ${super_memo_vs_wasmtime_speed}x faster in throughput"
    echo ""
    echo "chiwawa (superinstructions) vs wasmtime:"
    echo "  wasmtime is ${super_vs_wasmtime_speed}x faster in throughput"
    echo ""
    echo "chiwawa (baseline) vs wasmtime:"
    echo "  wasmtime is ${baseline_vs_wasmtime_speed}x faster in throughput"
    echo ""
    echo "chiwawa superinstructions vs baseline:"
    echo "  superinstructions is ${super_vs_baseline_speed}x faster than baseline"
    echo ""
    echo "chiwawa memoization vs superinstructions:"
    echo "  memoization is ${memo_vs_super_speed}x faster than superinstructions only"
    
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