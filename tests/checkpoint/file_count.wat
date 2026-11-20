(module
  (import "wasi_snapshot_preview1" "fd_write"
    (func $fd_write (param i32 i32 i32 i32) (result i32)))
  (import "wasi_snapshot_preview1" "path_open"
    (func $path_open (param i32 i32 i32 i32 i32 i64 i64 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (global $fd (mut i32) (i32.const 0))

  (data (i32.const 64) "count.txt")

  (func $print (param $n i32)
    (local $p i32)
    (local.set $p (i32.const 47))
    (i32.store8 (local.get $p) (i32.const 10))
    (loop $digits
      (local.set $p (i32.sub (local.get $p) (i32.const 1)))
      (i32.store8 (local.get $p)
        (i32.add (i32.const 48) (i32.rem_u (local.get $n) (i32.const 10))))
      (local.set $n (i32.div_u (local.get $n) (i32.const 10)))
      (br_if $digits (local.get $n)))
    (i32.store (i32.const 0) (local.get $p))
    (i32.store (i32.const 4) (i32.sub (i32.const 48) (local.get $p)))
    (drop (call $fd_write (global.get $fd) (i32.const 0) (i32.const 1) (i32.const 8))))

  (func $spin (param $k i32)
    (loop $l
      (local.set $k (i32.sub (local.get $k) (i32.const 1)))
      (br_if $l (local.get $k))))

  (func (export "_start")
    (local $i i32)
    (drop (call $path_open
      (i32.const 3) (i32.const 0) (i32.const 64) (i32.const 9) (i32.const 9)
      (i64.const 0x1fffffff) (i64.const 0x1fffffff) (i32.const 0) (i32.const 12)))
    (global.set $fd (i32.load (i32.const 12)))
    (loop $count
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (call $print (local.get $i))
      (call $spin (i32.const 50000))
      (br_if $count (i32.lt_u (local.get $i) (i32.const 2000)))))
)
