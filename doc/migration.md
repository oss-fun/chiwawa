# Checkpoint and Restore for Live Migration

This document explains Chiwawa's checkpoint/restore mechanism for live migration.

## Motivation

Live migration enables moving WebAssembly workloads between hosts and resuming execution from where it left off. This is valuable for:

- **Edge-cloud continuum**: Moving computation between edge devices and cloud
- **Fault tolerance**: Recovering from failures by restoring checkpoints
- **Load balancing**: Redistributing workloads across infrastructure
- **Debugging**: Capturing execution state for analysis

## How It Works

### Checkpoint Process

1. **Trigger Detection**: Monitor for checkpoint signal
2. **State Capture**: Gather all runtime state
3. **Serialization**: Convert state to binary format
4. **Persistence**: Write to checkpoint file

### Captured State

A checkpoint captures:

- **Execution State**: Call stack, program counters, register values
- **Memory**: Complete linear memory contents
- **Globals**: All global variable values
- **Tables**: Function reference tables

### Restore Process

1. **Load**: Read checkpoint file
2. **Deserialize**: Reconstruct state structures
3. **Apply**: Restore memory, globals, tables to module instance
4. **Resume**: Continue execution from saved program counter

## Trigger Mechanisms

Traditional checkpoint systems use signals (e.g., SIGUSR1) to trigger checkpoints. However, WebAssembly's sandboxed execution model does not support signal handling. Chiwawa uses file-based triggers instead: the presence of a trigger file (`checkpoint.trigger`) signals that a checkpoint should be taken.

Chiwawa supports two detection mechanisms:

### Thread-based (wasm32-wasip1-threads)
A background thread monitors for a trigger file, enabling non-blocking checkpoint detection with minimal performance impact.

### Polling-based (wasm32-wasip1)
For hosts without thread support, WASI file operations check for the trigger at instruction boundaries.

## Runtime Neutrality

Because Chiwawa is self-hosted (runs as WebAssembly itself), checkpoints are portable across different host runtimes. A checkpoint created on Wasmtime can be restored on WasmEdge, Wasmtime or any other WASI-compliant runtime.