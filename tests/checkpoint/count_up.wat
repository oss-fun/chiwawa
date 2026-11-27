;; Prints 1, 2, ..., 2000 on their own lines, spinning between prints so a checkpoint trigger has time to land mid-run.
(module
  (import "wasi_snapshot_preview1" "fd_write"
    (func $fd_write (param i32 i32 i32 i32) (result i32)))
  (memory (export "memory") 1)

  ;; Memory layout: 0..8 iovec, 8..12 bytes written, 16..48 digit buffer.
  ;; The hook uses 64..83 for its marker line, 96..104 for its iovec, and 104..108 for its bytes written.
  (data (i32.const 64) "after-restore hook\n")

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
    (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 8))))

  (func $spin (param $k i32)
    (loop $l
      (local.set $k (i32.sub (local.get $k) (i32.const 1)))
      (br_if $l (local.get $k))))

  (func (export "_start")
    (local $i i32)
    (loop $count
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (call $print (local.get $i))
      (call $spin (i32.const 50000))
      (br_if $count (i32.lt_u (local.get $i) (i32.const 2000)))))

  ;; Chiwawa calls this export after a restore with --hook after-restore.
  ;; It prints the marker line, and then it grows the memory.
  ;; The restored counter must still read and write its own data after the growth.
  (func (export "chiwawa_after_restore")
    (i32.store (i32.const 96) (i32.const 64))
    (i32.store (i32.const 100) (i32.const 19))
    (drop (call $fd_write (i32.const 1) (i32.const 96) (i32.const 1) (i32.const 104)))
    (drop (memory.grow (i32.const 1))))
)
