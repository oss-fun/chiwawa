;; A component with three exports that take and return memory-backed values, for the CLI test.
;; `sum` adds the items of a list, `swap` returns the two fields of a record in the other order, and `echo` returns its string.
(component
  (core module $M
    (memory (export "mem") 1)
    (global $next (mut i32) (i32.const 1024))
    ;; A bump allocator that ignores the old pointer and size.
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (local $p i32)
      (local.set $p
        (i32.and
          (i32.add (global.get $next) (i32.sub (local.get 2) (i32.const 1)))
          (i32.sub (i32.const 0) (local.get 2))))
      (global.set $next (i32.add (local.get $p) (local.get 3)))
      (local.get $p))
    (func (export "sum") (param $ptr i32) (param $len i32) (result i32)
      (local $acc i32)
      (block $done
        (loop $next
          (br_if $done (i32.eqz (local.get $len)))
          (local.set $acc (i32.add (local.get $acc) (i32.load (local.get $ptr))))
          (local.set $ptr (i32.add (local.get $ptr) (i32.const 4)))
          (local.set $len (i32.sub (local.get $len) (i32.const 1)))
          (br $next)))
      (local.get $acc))
    ;; The tuple result does not fit in one core value, so the function returns a pointer to it.
    (func (export "swap") (param $a i32) (param $b i32) (result i32)
      (i32.store (i32.const 8) (local.get $b))
      (i32.store (i32.const 12) (local.get $a))
      (i32.const 8))
    (func (export "echo") (param $ptr i32) (param $len i32) (result i32)
      (i32.store (i32.const 16) (local.get $ptr))
      (i32.store (i32.const 20) (local.get $len))
      (i32.const 16)))
  (core instance $m (instantiate $M))
  (type $pair-type (record (field "a" u32) (field "b" u32)))
  ;; A record in an exported function type must be exported with a name.
  (export $pair "pair" (type $pair-type))
  (type $swapped (tuple u32 u32))
  (func (export "sum") (param "xs" (list u32)) (result u32)
    (canon lift (core func $m "sum") (memory $m "mem") (realloc (func $m "realloc"))))
  (func (export "swap") (param "r" $pair) (result $swapped)
    (canon lift (core func $m "swap") (memory $m "mem")))
  (func (export "echo") (param "s" string) (result string)
    (canon lift (core func $m "echo") (memory $m "mem") (realloc (func $m "realloc"))))
)
