(module
  (memory 1) ;; Include memory in case runtime assumes it exists

  (func (export "break-inner") (result i32)
    (local i32)
    (local.set 0 (i32.const 0))
    ;; Test 1: br 2 from nested block/loop
    (local.set 0 (i32.add (local.get 0) (block (result i32) (loop (result i32) (block (result i32) (br 2 (i32.const 0x1)))))))
    ;; Test 2: br 2 from nested loop/loop
    (local.set 0 (i32.add (local.get 0) (block (result i32) (loop (result i32) (loop (result i32) (br 2 (i32.const 0x2)))))))
    ;; Test 3: br 1 from nested block/loop/block/loop
    (local.set 0 (i32.add (local.get 0) (block (result i32) (loop (result i32) (block (result i32) (loop (result i32) (br 1 (i32.const 0x4))))))))
    ;; Test 4: br 1 from loop with expr
    (local.set 0 (i32.add (local.get 0) (block (result i32) (loop (result i32) (i32.ctz (br 1 (i32.const 0x8)))))))
    ;; Test 5: br 2 from nested loop with expr
    (local.set 0 (i32.add (local.get 0) (block (result i32) (loop (result i32) (i32.ctz (loop (result i32) (br 2 (i32.const 0x10))))))))
    (local.get 0)
  )
) 