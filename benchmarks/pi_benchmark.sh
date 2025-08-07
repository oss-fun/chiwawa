#!/bin/bash

set -e

echo "=== chiwawa Pi Calculation Benchmark ==="

# Build chiwawa if not already built
if [ ! -f "../target/wasm32-wasip1/release/chiwawa.wasm" ]; then
    echo "Building chiwawa..."
    cd ..
    ~/.cargo/bin/cargo build --target wasm32-wasip1 --release
    cd benchmarks
fi

# Build benchmark program
echo "Building pi benchmark program..."
~/.cargo/bin/cargo build --target wasm32-wasip1 --release --bin pi-leibniz

CHIWAWA_WASM="../target/wasm32-wasip1/release/chiwawa.wasm"
PI_WASM="target/wasm32-wasip1/release/pi-leibniz.wasm"

# Check if files exist
if [ ! -f "$CHIWAWA_WASM" ]; then
    echo "Error: chiwawa.wasm not found at $CHIWAWA_WASM"
    exit 1
fi

if [ ! -f "$PI_WASM" ]; then
    echo "Error: pi-leibniz.wasm not found at $PI_WASM"
    exit 1
fi

echo ""
echo "=== Running pi calculation benchmarks ==="

function run_pi_benchmark() {
    local desc="$1"
    local runs="$2"
    
    echo "=== $desc ==="
    echo "Runs: $runs"
    
    echo ""
    echo "Running with chiwawa runtime (superinstructions + memoization):"
    
    local total_chiwawa_super_memo_time=0
    
    # Run chiwawa pi benchmark with superinstructions and memoization multiple times
    for i in $(seq 1 $runs); do
        local start_time=$(date +%s.%N)
        wasmtime --dir . "$CHIWAWA_WASM" "$PI_WASM" --enable-superinstructions --enable-memoization 2>/dev/null
        local end_time=$(date +%s.%N)
        
        local run_time=$(echo "$end_time - $start_time" | bc)
        total_chiwawa_super_memo_time=$(echo "$total_chiwawa_super_memo_time + $run_time" | bc)
        
        echo "Run $i: ${run_time}s"
    done
    
    # Calculate chiwawa superinstructions + memoization average execution time
    local chiwawa_super_memo_avg_time=$(echo "scale=6; $total_chiwawa_super_memo_time / $runs" | bc)
    
    echo "chiwawa (superinstructions + memoization) average time: ${chiwawa_super_memo_avg_time}s"
    echo "chiwawa (superinstructions + memoization) total time: ${total_chiwawa_super_memo_time}s"
    
    echo ""
    echo "Running with chiwawa runtime (superinstructions only):"
    
    local total_chiwawa_super_time=0
    
    # Run chiwawa pi benchmark with superinstructions only multiple times
    for i in $(seq 1 $runs); do
        local start_time=$(date +%s.%N)
        wasmtime --dir . "$CHIWAWA_WASM" "$PI_WASM" --enable-superinstructions 2>/dev/null
        local end_time=$(date +%s.%N)
        
        local run_time=$(echo "$end_time - $start_time" | bc)
        total_chiwawa_super_time=$(echo "$total_chiwawa_super_time + $run_time" | bc)
        
        echo "Run $i: ${run_time}s"
    done
    
    # Calculate chiwawa superinstructions average execution time
    local chiwawa_super_avg_time=$(echo "scale=6; $total_chiwawa_super_time / $runs" | bc)
    
    echo "chiwawa (superinstructions) average time: ${chiwawa_super_avg_time}s"
    echo "chiwawa (superinstructions) total time: ${total_chiwawa_super_time}s"
    
    echo ""
    echo "Running with chiwawa runtime (baseline):"
    
    local total_chiwawa_time=0
    
    # Run chiwawa pi benchmark without superinstructions and memoization multiple times
    for i in $(seq 1 $runs); do
        local start_time=$(date +%s.%N)
        wasmtime --dir . "$CHIWAWA_WASM" "$PI_WASM" 2>/dev/null
        local end_time=$(date +%s.%N)
        
        local run_time=$(echo "$end_time - $start_time" | bc)
        total_chiwawa_time=$(echo "$total_chiwawa_time + $run_time" | bc)
        
        echo "Run $i: ${run_time}s"
    done
    
    # Calculate chiwawa baseline average execution time
    local chiwawa_avg_time=$(echo "scale=6; $total_chiwawa_time / $runs" | bc)
    
    echo "chiwawa (baseline) average time: ${chiwawa_avg_time}s"
    echo "chiwawa (baseline) total time: ${total_chiwawa_time}s"
    
    echo ""
    echo "Running with wasmtime (native):"
    
    local total_wasmtime_time=0
    
    # Run wasmtime pi benchmark multiple times
    for i in $(seq 1 $runs); do
        local start_time=$(date +%s.%N)
        wasmtime --dir . "$PI_WASM" >/dev/null 2>&1
        local end_time=$(date +%s.%N)
        
        local run_time=$(echo "$end_time - $start_time" | bc)
        total_wasmtime_time=$(echo "$total_wasmtime_time + $run_time" | bc)
        
        echo "Run $i: ${run_time}s"
    done
    
    # Calculate wasmtime average execution time
    local wasmtime_avg_time=$(echo "scale=6; $total_wasmtime_time / $runs" | bc)
    
    echo "wasmtime average time: ${wasmtime_avg_time}s"
    echo "wasmtime total time: ${total_wasmtime_time}s"
    
    # Calculate performance ratios
    local super_memo_vs_wasmtime_ratio=$(echo "scale=2; $chiwawa_super_memo_avg_time / $wasmtime_avg_time" | bc)
    local super_vs_wasmtime_ratio=$(echo "scale=2; $chiwawa_super_avg_time / $wasmtime_avg_time" | bc)
    local baseline_vs_wasmtime_ratio=$(echo "scale=2; $chiwawa_avg_time / $wasmtime_avg_time" | bc)
    local super_memo_vs_super_ratio=$(echo "scale=2; $chiwawa_super_avg_time / $chiwawa_super_memo_avg_time" | bc)
    local super_vs_baseline_ratio=$(echo "scale=2; $chiwawa_avg_time / $chiwawa_super_avg_time" | bc)
    local super_memo_vs_baseline_ratio=$(echo "scale=2; $chiwawa_avg_time / $chiwawa_super_memo_avg_time" | bc)
    
    echo ""
    echo "Performance comparison:"
    echo "Iterations per benchmark: 100,000"
    echo ""
    echo "chiwawa (superinstructions + memoization) vs wasmtime:"
    echo "  chiwawa (superinstructions + memoization) takes ${super_memo_vs_wasmtime_ratio}x longer than wasmtime"
    echo ""
    echo "chiwawa (superinstructions) vs wasmtime:"
    echo "  chiwawa (superinstructions) takes ${super_vs_wasmtime_ratio}x longer than wasmtime"
    echo ""
    echo "chiwawa (baseline) vs wasmtime:"
    echo "  chiwawa (baseline) takes ${baseline_vs_wasmtime_ratio}x longer than wasmtime"
    echo ""
    echo "chiwawa memoization impact:"
    echo "  memoization reduces superinstructions execution time by ${super_memo_vs_super_ratio}x"
    echo ""
    echo "chiwawa superinstructions vs baseline:"
    echo "  superinstructions reduces execution time by ${super_vs_baseline_ratio}x"
    echo ""
    echo "chiwawa full optimization vs baseline:"
    echo "  superinstructions + memoization reduces execution time by ${super_memo_vs_baseline_ratio}x"
    echo ""
}

# Pi calculation benchmark (10 runs each)
run_pi_benchmark "Pi Calculation Benchmark (100k iterations x 10 runs)" 10

echo ""
echo "=== Pi benchmark completed ==="