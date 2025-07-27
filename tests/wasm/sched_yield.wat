(module
    (import "wasi_snapshot_preview1" "sched_yield" (func $sched_yield (result i32)))
    
    (func $main (export "_start") (result i32)
        ;; Call sched_yield
        call $sched_yield
        ;; Return the result (should be 0 for success)
    )
)